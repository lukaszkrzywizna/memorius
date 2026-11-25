use std::collections::HashMap;
use std::path::Path;

use memorius_core::model::{AppRef, Title, Window, url_without_query};
use windows::Win32::Foundation::{CloseHandle, HWND};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, DeleteObject, GetDC, GetDIBits,
    GetObjectW, HBITMAP, ReleaseDC,
};
use windows::Win32::Storage::FileSystem::{
    FILE_FLAGS_AND_ATTRIBUTES, GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
};
use windows::Win32::System::StationsAndDesktops::{
    CloseDesktop, DESKTOP_CONTROL_FLAGS, DESKTOP_SWITCHDESKTOP, OpenInputDesktop,
};
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationElement, IUIAutomationValuePattern,
    TreeScope_Descendants, UIA_ControlTypePropertyId, UIA_EditControlTypeId, UIA_ValuePatternId,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
use windows::Win32::UI::Shell::{SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGetFileInfoW};
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyIcon, GetForegroundWindow, GetIconInfo, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId, HICON, ICONINFO,
};
use windows::core::{HSTRING, PCWSTR, PWSTR};

use crate::{Reading, TitleAccess};

/// The app's icon as a PNG, for the timeline. `app_id` is the executable path.
pub fn app_icon_png(app_id: &str) -> Option<Vec<u8>> {
    let mut info = SHFILEINFOW::default();
    // SAFETY: `info` is a valid out struct of the given size.
    let found = unsafe {
        SHGetFileInfoW(
            &HSTRING::from(app_id),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&raw mut info),
            u32::try_from(size_of::<SHFILEINFOW>()).ok()?,
            SHGFI_ICON | SHGFI_LARGEICON,
        )
    };
    if found == 0 || info.hIcon.is_invalid() {
        return None;
    }
    let pixels = icon_rgba(info.hIcon);
    // SAFETY: SHGetFileInfoW gave this icon to the caller to destroy.
    let _ = unsafe { DestroyIcon(info.hIcon) };
    let (width, height, rgba) = pixels?;
    let mut png = Vec::new();
    let mut encoder = png::Encoder::new(&mut png, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header().ok()?.write_image_data(&rgba).ok()?;
    Some(png)
}

fn icon_rgba(icon: HICON) -> Option<(u32, u32, Vec<u8>)> {
    let mut info = ICONINFO::default();
    // SAFETY: `info` is a valid out struct; the bitmaps are released below.
    unsafe { GetIconInfo(icon, &raw mut info) }.ok()?;
    let pixels = bitmap_rgba(info.hbmColor);
    // SAFETY: GetIconInfo created both bitmaps for the caller.
    unsafe {
        let _ = DeleteObject(info.hbmColor.into());
        let _ = DeleteObject(info.hbmMask.into());
    }
    pixels
}

fn bitmap_rgba(bitmap: HBITMAP) -> Option<(u32, u32, Vec<u8>)> {
    if bitmap.is_invalid() {
        return None;
    }
    let mut size = BITMAP::default();
    // SAFETY: `size` is a valid BITMAP of the given size.
    let read = unsafe {
        GetObjectW(
            bitmap.into(),
            i32::try_from(size_of::<BITMAP>()).ok()?,
            Some((&raw mut size).cast()),
        )
    };
    let width = u32::try_from(size.bmWidth).ok()?;
    let height = u32::try_from(size.bmHeight).ok()?;
    if read == 0 || width == 0 || height == 0 {
        return None;
    }
    let mut info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: u32::try_from(size_of::<BITMAPINFOHEADER>()).ok()?,
            biWidth: size.bmWidth,
            // Negative height: rows top-down, as PNG stores them.
            biHeight: -size.bmHeight,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut pixels = vec![0_u8; usize::try_from(width * height * 4).ok()?];
    // SAFETY: the screen DC is released below; `pixels` holds width × height
    // 32-bit pixels as `info` describes.
    let lines = unsafe {
        let dc = GetDC(None);
        let lines = GetDIBits(
            dc,
            bitmap,
            0,
            height,
            Some(pixels.as_mut_ptr().cast()),
            &raw mut info,
            DIB_RGB_COLORS,
        );
        ReleaseDC(None, dc);
        lines
    };
    if lines == 0 {
        return None;
    }
    // Old icons have no alpha channel: all zero means fully opaque.
    let opaque = pixels.chunks_exact(4).all(|pixel| pixel[3] == 0);
    for pixel in pixels.chunks_exact_mut(4) {
        pixel.swap(0, 2);
        if opaque {
            pixel[3] = u8::MAX;
        }
    }
    Some((width, height, pixels))
}

/// Product names are read from version resources once per executable.
/// Distinct executables are few; the cache resets if it ever grows past this.
const NAME_CACHE_LIMIT: usize = 512;

/// Browsers whose first edit control is the address bar.
const BROWSERS: [&str; 4] = ["chrome.exe", "msedge.exe", "brave.exe", "vivaldi.exe"];

/// Address bars found per browser window; reset past this many windows.
const ADDRESS_BAR_LIMIT: usize = 64;

pub struct Probe {
    names: HashMap<String, String>,
    automation: Option<IUIAutomation>,
    address_bars: HashMap<isize, IUIAutomationElement>,
}

impl Default for Probe {
    fn default() -> Self {
        Self::new()
    }
}

impl Probe {
    /// Call on the thread that reads; UI Automation is bound to it.
    pub fn new() -> Self {
        // SAFETY: initializing COM for this thread; a repeat call is harmless.
        let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        // SAFETY: standard in-process creation of the UI Automation client.
        let automation =
            unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) }.ok();
        Self {
            names: HashMap::new(),
            automation,
            address_bars: HashMap::new(),
        }
    }

    /// Windows does not restrict reading window titles.
    pub fn title_access(&self) -> TitleAccess {
        TitleAccess::Granted
    }

    /// Nothing to request on Windows.
    pub fn request_title_access(&self) {}

    pub fn read(&mut self) -> Reading {
        Reading {
            idle_ms: idle_ms(),
            locked: workstation_locked(),
            window: self.foreground_window(),
        }
    }

    fn foreground_window(&mut self) -> Option<Window> {
        // SAFETY: no preconditions; a null handle means no foreground window.
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd.is_invalid() {
            return None;
        }
        let path = process_path(hwnd)?;
        if self.names.len() > NAME_CACHE_LIMIT {
            self.names.clear();
        }
        let name = self
            .names
            .entry(path.clone())
            .or_insert_with(|| product_name(&path))
            .clone();
        let url = self.browser_url(hwnd, &path);
        Some(Window {
            app: AppRef { id: path, name },
            title: Title::Known {
                text: window_text(hwnd),
            },
            document: None,
            url,
        })
    }

    /// The address bar text of Chrome, Edge, Brave or Vivaldi, read with UI
    /// Automation. The bar is looked up once per window.
    fn browser_url(&mut self, hwnd: HWND, path: &str) -> Option<String> {
        let exe = Path::new(path)
            .file_name()?
            .to_string_lossy()
            .to_ascii_lowercase();
        if !BROWSERS.contains(&exe.as_str()) {
            return None;
        }
        let automation = self.automation.as_ref()?;
        let key = hwnd.0 as isize;
        if !self.address_bars.contains_key(&key) {
            if self.address_bars.len() >= ADDRESS_BAR_LIMIT {
                self.address_bars.clear();
            }
            self.address_bars
                .insert(key, address_bar(automation, hwnd)?);
        }
        let bar = self.address_bars.get(&key)?;
        // SAFETY: a valid element; a closed window makes the call fail.
        let value =
            unsafe { bar.GetCurrentPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId) }
                .and_then(|pattern| unsafe { pattern.CurrentValue() });
        let Ok(value) = value else {
            self.address_bars.remove(&key);
            return None;
        };
        let text = value.to_string();
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        // The bar hides "https://" while it is not focused.
        let url = if text.contains("://") {
            text.to_owned()
        } else {
            format!("https://{text}")
        };
        Some(url_without_query(&url).to_owned())
    }
}

fn address_bar(automation: &IUIAutomation, hwnd: HWND) -> Option<IUIAutomationElement> {
    // SAFETY: plain UI Automation queries on a live top-level window.
    unsafe {
        let window = automation.ElementFromHandle(hwnd).ok()?;
        let condition = automation
            .CreatePropertyCondition(
                UIA_ControlTypePropertyId,
                &VARIANT::from(UIA_EditControlTypeId.0),
            )
            .ok()?;
        window.FindFirst(TreeScope_Descendants, &condition).ok()
    }
}

fn process_path(hwnd: HWND) -> Option<String> {
    let mut pid = 0_u32;
    // SAFETY: `pid` is a valid out pointer for the call.
    unsafe { GetWindowThreadProcessId(hwnd, Some(&raw mut pid)) };
    // SAFETY: opening with query-only rights; the handle is closed below.
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
    let mut buffer = [0_u16; 1024];
    let mut size = u32::try_from(buffer.len()).unwrap_or(u32::MAX);
    // SAFETY: `buffer` holds `size` UTF-16 units; `size` is updated in place.
    let result = unsafe {
        QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &raw mut size,
        )
    };
    // SAFETY: the handle came from OpenProcess above.
    let _ = unsafe { CloseHandle(process) };
    result.ok()?;
    let length = usize::try_from(size).ok()?;
    Some(String::from_utf16_lossy(&buffer[..length]))
}

fn window_text(hwnd: HWND) -> String {
    // SAFETY: valid window handle from GetForegroundWindow.
    let length = unsafe { GetWindowTextLengthW(hwnd) };
    let Ok(capacity) = usize::try_from(length) else {
        return String::new();
    };
    let mut buffer = vec![0_u16; capacity + 1];
    // SAFETY: the buffer has room for the text and the terminator.
    let copied = unsafe { GetWindowTextW(hwnd, &mut buffer) };
    let copied = usize::try_from(copied).unwrap_or(0);
    String::from_utf16_lossy(&buffer[..copied])
}

/// `FileDescription` from the executable's version resource (for example
/// "Microsoft Teams"), falling back to the file stem.
fn product_name(path: &str) -> String {
    let fallback = || {
        Path::new(path).file_stem().map_or_else(
            || path.to_owned(),
            |stem| stem.to_string_lossy().into_owned(),
        )
    };
    let file = HSTRING::from(path);
    // SAFETY: `file` is a valid null-terminated wide string.
    let size = unsafe { GetFileVersionInfoSizeW(&file, None) };
    let Ok(length) = usize::try_from(size) else {
        return fallback();
    };
    if length == 0 {
        return fallback();
    }
    let mut data = vec![0_u8; length];
    // SAFETY: `data` has `size` bytes, as the call requires.
    if unsafe { GetFileVersionInfoW(&file, None, size, data.as_mut_ptr().cast()) }.is_err() {
        return fallback();
    }
    let Some(translation) = query(&data, "\\VarFileInfo\\Translation") else {
        return fallback();
    };
    let [l0, l1, c0, c1, ..] = *translation else {
        return fallback();
    };
    let language = u16::from_le_bytes([l0, l1]);
    let code_page = u16::from_le_bytes([c0, c1]);
    let key = format!("\\StringFileInfo\\{language:04x}{code_page:04x}\\FileDescription");
    query(&data, &key)
        .map(|bytes| {
            let units: Vec<u16> = bytes
                .chunks_exact(2)
                .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
                .take_while(|unit| *unit != 0)
                .collect();
            String::from_utf16_lossy(&units).trim().to_owned()
        })
        .filter(|name| !name.is_empty())
        .unwrap_or_else(fallback)
}

fn query<'a>(data: &'a [u8], key: &str) -> Option<&'a [u8]> {
    let key = HSTRING::from(key);
    let mut value: *mut core::ffi::c_void = std::ptr::null_mut();
    let mut length = 0_u32;
    // SAFETY: `data` is a version resource from GetFileVersionInfoW; the
    // returned pointer points into it.
    let found = unsafe {
        VerQueryValueW(
            data.as_ptr().cast(),
            PCWSTR(key.as_ptr()),
            &raw mut value,
            &raw mut length,
        )
    }
    .as_bool();
    if !found || value.is_null() {
        return None;
    }
    let start = (value as usize).checked_sub(data.as_ptr() as usize)?;
    // String values report UTF-16 units and binary values bytes; reading the
    // larger count is clamped to the resource by `get`.
    let bytes = usize::try_from(length).ok()?.saturating_mul(2);
    data.get(start..start.saturating_add(bytes).min(data.len()))
}

fn idle_ms() -> i64 {
    let mut info = LASTINPUTINFO {
        cbSize: u32::try_from(size_of::<LASTINPUTINFO>()).unwrap_or(0),
        dwTime: 0,
    };
    // SAFETY: `info` is a valid, sized LASTINPUTINFO.
    if !unsafe { GetLastInputInfo(&raw mut info) }.as_bool() {
        return 0;
    }
    // SAFETY: no preconditions. Both counters wrap after ~49 days.
    let now = unsafe { GetTickCount() };
    i64::from(now.wrapping_sub(info.dwTime))
}

/// The input desktop cannot be opened while the workstation is locked.
fn workstation_locked() -> bool {
    // SAFETY: the handle is closed right away when the call succeeds.
    match unsafe { OpenInputDesktop(DESKTOP_CONTROL_FLAGS(0), false, DESKTOP_SWITCHDESKTOP) } {
        Ok(desktop) => {
            // SAFETY: handle from OpenInputDesktop.
            let _ = unsafe { CloseDesktop(desktop) };
            false
        }
        Err(_) => true,
    }
}
