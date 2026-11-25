use std::collections::{HashSet, VecDeque};
use std::ptr::NonNull;
use std::time::{Duration, Instant};

use memorius_core::model::{AppRef, Title, TitleUnavailable, Window, url_without_query};
use objc2::AllocAnyThread;
use objc2::rc::autoreleasepool;
use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRep, NSWorkspace};
use objc2_application_services::{
    AXError, AXIsProcessTrusted, AXIsProcessTrustedWithOptions, AXUIElement,
    kAXTrustedCheckOptionPrompt,
};
use objc2_core_foundation::{
    CFArray, CFBoolean, CFDictionary, CFRetained, CFString, CFType, CFURL,
};
use objc2_core_graphics::{
    CGEventSource, CGEventSourceStateID, CGEventType, CGSessionCopyCurrentDictionary,
};
use objc2_foundation::{NSDictionary, NSPoint, NSRect, NSSize, NSString, NSURL};

use crate::{Reading, TitleAccess};

/// An unresponsive app must not stall the recorder thread.
const AX_TIMEOUT_SECONDS: f32 = 0.25;

/// `kCGAnyInputEventType`.
const ANY_INPUT_EVENT: CGEventType = CGEventType(u32::MAX);

/// Browsers whose address Memorius reads through Accessibility.
const BROWSERS: [&str; 10] = [
    "com.apple.Safari",
    "com.apple.SafariTechnologyPreview",
    "com.google.Chrome",
    "com.google.Chrome.canary",
    "com.brave.Browser",
    "com.microsoft.edgemac",
    "company.thebrowser.Browser",
    "company.thebrowser.dia",
    "com.vivaldi.Vivaldi",
    "org.chromium.Chromium",
];

/// Chromium builds its accessibility tree only for clients that ask with
/// `AXManualAccessibility`; Safari always has one.
fn is_chromium(bundle_id: &str) -> bool {
    !bundle_id.starts_with("com.apple.")
}

/// Bounds of one search through a window's element tree.
const SEARCH_NODES: usize = 300;
const SEARCH_DEPTH: usize = 12;

/// After a search finds nothing, try the same page again only this much later.
const SEARCH_AGAIN: Duration = Duration::from_secs(3);

/// One tab of one window. A new title means another tab or page, so the
/// address element is searched again: a background tab's web area can stay
/// alive and still report its own URL.
#[derive(PartialEq)]
struct Page {
    window: CFRetained<AXUIElement>,
    title: String,
}

/// The element that holds the address of a page: its web area (`AXURL`) or
/// the address field (`AXValue`).
struct AddressSource {
    page: Page,
    element: CFRetained<AXUIElement>,
}

#[derive(Default)]
pub struct Probe {
    address: Option<AddressSource>,
    /// The page searched without result, and when.
    missed: Option<(Page, Instant)>,
    /// Chromium processes already asked to build their accessibility tree.
    prepared: HashSet<i32>,
}

impl Probe {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title_access(&self) -> TitleAccess {
        // SAFETY: AXIsProcessTrusted has no preconditions.
        if unsafe { AXIsProcessTrusted() } {
            TitleAccess::Granted
        } else {
            TitleAccess::Denied
        }
    }

    /// Shows the macOS Accessibility prompt and opens the Accessibility pane
    /// of System Settings: macOS shows its prompt only once per app, so the
    /// pane is the reliable way to the switch.
    pub fn request_title_access(&self) {
        // SAFETY: the prompt key is a CFString constant from the framework.
        let key: &CFString = unsafe { kAXTrustedCheckOptionPrompt };
        let options =
            CFDictionary::<CFString, CFBoolean>::from_slices(&[key], &[CFBoolean::new(true)]);
        // SAFETY: the dictionary has the documented key and value types.
        unsafe { AXIsProcessTrustedWithOptions(Some(options.as_opaque())) };
        let pane = NSString::from_str(
            "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
        );
        if let Some(url) = NSURL::URLWithString(&pane) {
            NSWorkspace::sharedWorkspace().openURL(&url);
        }
    }

    pub fn read(&mut self) -> Reading {
        Reading {
            idle_ms: idle_ms(),
            locked: screen_locked(),
            window: self.foreground_window(),
        }
    }

    fn foreground_window(&mut self) -> Option<Window> {
        let app = NSWorkspace::sharedWorkspace().frontmostApplication()?;
        let executable = app.executableURL().and_then(|url| url.path());
        let id = app
            .bundleIdentifier()
            .or(executable)
            .map(|id| id.to_string())?;
        let name = app
            .localizedName()
            .map_or_else(|| id.clone(), |name| name.to_string());
        let pid = app.processIdentifier();
        let (title, document, url) = match self.title_access() {
            TitleAccess::Granted => {
                let element = application(pid);
                match focused_window(&element) {
                    Some(window) => {
                        let text = string_attribute(&window, "AXTitle").unwrap_or_default();
                        let document = window_document(&window);
                        let url = if BROWSERS.contains(&id.as_str()) {
                            let chromium = is_chromium(&id);
                            self.browser_url(pid, &element, window, text.clone(), chromium)
                        } else {
                            None
                        };
                        (Title::Known { text }, document, url)
                    }
                    None => (
                        Title::Unavailable {
                            reason: TitleUnavailable::NoWindow,
                        },
                        None,
                        None,
                    ),
                }
            }
            TitleAccess::Denied => (
                Title::Unavailable {
                    reason: TitleUnavailable::PermissionDenied,
                },
                None,
                None,
            ),
        };
        Some(Window {
            app: AppRef { id, name },
            title,
            document,
            url,
        })
    }

    /// The active tab's address, read through Accessibility (no per-browser
    /// permission). The element holding it is found once per page.
    fn browser_url(
        &mut self,
        pid: i32,
        app: &AXUIElement,
        window: CFRetained<AXUIElement>,
        title: String,
        chromium: bool,
    ) -> Option<String> {
        if chromium && self.prepared.insert(pid) {
            let key = CFString::from_static_str("AXManualAccessibility");
            // SAFETY: a boolean attribute on the application element.
            unsafe { app.set_attribute_value(&key, CFBoolean::new(true)) };
        }
        let page = Page { window, title };
        if let Some(source) = self.address.as_ref().filter(|s| s.page == page) {
            if let Some(url) = address_of(&source.element) {
                return Some(url);
            }
            self.address = None;
        }
        if self
            .missed
            .as_ref()
            .is_some_and(|(missed, at)| *missed == page && at.elapsed() < SEARCH_AGAIN)
        {
            return None;
        }
        let Some(element) = find_address_element(&page.window) else {
            self.missed = Some((page, Instant::now()));
            return None;
        };
        let url = address_of(&element);
        self.address = Some(AddressSource { page, element });
        url
    }
}

/// The app's icon as a PNG of about [`ICON_PX`] pixels, for the timeline.
/// `app_id` is a bundle id, or an executable path for apps without one.
pub fn app_icon_png(app_id: &str) -> Option<Vec<u8>> {
    autoreleasepool(|_| {
        let workspace = NSWorkspace::sharedWorkspace();
        let path = if app_id.starts_with('/') {
            NSString::from_str(app_id)
        } else {
            workspace
                .URLForApplicationWithBundleIdentifier(&NSString::from_str(app_id))?
                .path()?
        };
        let icon = workspace.iconForFile(&path);
        let mut rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(ICON_PX, ICON_PX));
        // SAFETY: `rect` is a valid in/out rectangle; no context or hints.
        let image =
            unsafe { icon.CGImageForProposedRect_context_hints(&raw mut rect, None, None) }?;
        let bitmap = NSBitmapImageRep::initWithCGImage(NSBitmapImageRep::alloc(), &image);
        // SAFETY: an empty properties dictionary is valid for PNG.
        let png = unsafe {
            bitmap.representationUsingType_properties(
                NSBitmapImageFileType::PNG,
                &NSDictionary::new(),
            )
        }?;
        Some(png.to_vec())
    })
}

/// Icons show at up to 32 CSS pixels on Retina screens.
const ICON_PX: f64 = 64.0;

fn application(pid: i32) -> CFRetained<AXUIElement> {
    // SAFETY: creating an element for any pid is allowed; calls on it fail
    // softly for processes without accessibility support.
    let app = unsafe { AXUIElement::new_application(pid) };
    // SAFETY: the element is valid for the duration of the call.
    unsafe { app.set_messaging_timeout(AX_TIMEOUT_SECONDS) };
    app
}

fn focused_window(app: &AXUIElement) -> Option<CFRetained<AXUIElement>> {
    attribute(app, "AXFocusedWindow").and_then(|value| value.downcast::<AXUIElement>().ok())
}

fn window_document(window: &AXUIElement) -> Option<String> {
    let url = string_attribute(window, "AXDocument")?;
    NSURL::URLWithString(&NSString::from_str(&url))
        .and_then(|url| url.path())
        .map(|path| path.to_string())
}

/// Breadth-first through the window: the page's web area wins (its `AXURL`
/// is exact); an address field is the fallback.
fn find_address_element(window: &AXUIElement) -> Option<CFRetained<AXUIElement>> {
    let mut queue: VecDeque<(CFRetained<AXUIElement>, usize)> = children(window)
        .into_iter()
        .map(|child| (child, 1))
        .collect();
    let mut field = None;
    let mut visited = 0;
    while let Some((element, depth)) = queue.pop_front() {
        visited += 1;
        if visited > SEARCH_NODES {
            break;
        }
        match string_attribute(&element, "AXRole").as_deref() {
            Some("AXWebArea") if address_of(&element).is_some() => return Some(element),
            Some("AXTextField" | "AXComboBox") if field.is_none() && is_address_field(&element) => {
                field = Some(element);
                continue;
            }
            _ => {}
        }
        if depth < SEARCH_DEPTH {
            queue.extend(
                children(&element)
                    .into_iter()
                    .map(|child| (child, depth + 1)),
            );
        }
    }
    field
}

fn is_address_field(element: &AXUIElement) -> bool {
    [
        "AXDescription",
        "AXTitle",
        "AXIdentifier",
        "AXPlaceholderValue",
    ]
    .iter()
    .filter_map(|name| string_attribute(element, name))
    .any(|text| {
        let text = text.to_lowercase();
        text.contains("address") || text.contains("url") || text.contains("search bar")
    })
}

/// `scheme://host/path` from a web area's `AXURL` or an address field's
/// text; only web pages count.
fn address_of(element: &AXUIElement) -> Option<String> {
    let raw = attribute(element, "AXURL")
        .and_then(|value| value.downcast::<CFURL>().ok())
        .map(|url| url.string())
        .map(|text| text.to_string())
        .or_else(|| string_attribute(element, "AXValue"))?;
    let text = raw.trim();
    if text.is_empty() || text.contains(' ') {
        return None;
    }
    // Address fields hide the scheme while they are not focused.
    let url = if text.contains("://") {
        text.to_owned()
    } else {
        format!("https://{text}")
    };
    (url.starts_with("https://") || url.starts_with("http://"))
        .then(|| url_without_query(&url).to_owned())
}

fn children(element: &AXUIElement) -> Vec<CFRetained<AXUIElement>> {
    let Some(array) =
        attribute(element, "AXChildren").and_then(|value| value.downcast::<CFArray>().ok())
    else {
        return Vec::new();
    };
    // SAFETY: AXChildren is documented as an array of AXUIElements; each item
    // is still checked with `downcast`.
    let array: CFRetained<CFArray<CFType>> = unsafe { CFRetained::cast_unchecked(array) };
    (0..array.len())
        .filter_map(|index| array.get(index))
        .filter_map(|item| item.downcast::<AXUIElement>().ok())
        .collect()
}

fn string_attribute(element: &AXUIElement, name: &'static str) -> Option<String> {
    attribute(element, name)
        .and_then(|value| value.downcast::<CFString>().ok())
        .map(|text| text.to_string())
}

fn attribute(element: &AXUIElement, name: &'static str) -> Option<CFRetained<CFType>> {
    let name = CFString::from_static_str(name);
    let mut value: *const CFType = std::ptr::null();
    // SAFETY: `value` is a valid out pointer; on success it holds a +1
    // reference that `CFRetained::from_raw` takes over.
    let error = unsafe { element.copy_attribute_value(&name, NonNull::from(&mut value)) };
    if error != AXError::Success {
        return None;
    }
    // SAFETY: see above; a null value maps to `None`.
    NonNull::new(value.cast_mut()).map(|value| unsafe { CFRetained::from_raw(value) })
}

fn idle_ms() -> i64 {
    let seconds = CGEventSource::seconds_since_last_event_type(
        CGEventSourceStateID::CombinedSessionState,
        ANY_INPUT_EVENT,
    );
    // Whole milliseconds are enough; idle time is never negative.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "idle seconds fit in i64 ms"
    )]
    let ms = (seconds.max(0.0) * 1_000.0) as i64;
    ms
}

fn screen_locked() -> bool {
    let Some(session) = CGSessionCopyCurrentDictionary() else {
        return false;
    };
    let key = CFString::from_static_str("CGSSessionScreenIsLocked");
    // SAFETY: the key is a valid CFString; the dictionary returns a borrowed
    // value or null.
    let value = unsafe { session.value(std::ptr::from_ref(&*key).cast()) };
    // SAFETY: a non-null value from the session dictionary is a CF object.
    let value = unsafe { value.cast::<CFType>().as_ref() };
    value
        .and_then(|value| value.downcast_ref::<CFBoolean>())
        .is_some_and(CFBoolean::value)
}
