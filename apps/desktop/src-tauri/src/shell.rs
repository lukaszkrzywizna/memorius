//! Windows and tray. The main window and the popover exist only while shown:
//! closing destroys the `WebView` so idle recording stays small.

use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{
    AppHandle, Manager, PhysicalPosition, Rect, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};

use crate::recording::Recording;

const MAIN: &str = "main";
const POPOVER: &str = "popover";
const POPOVER_WIDTH: f64 = 360.0;
const POPOVER_HEIGHT: f64 = 560.0;

pub fn show_main(app: &AppHandle) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Regular)?;
    if let Some(window) = app.get_webview_window(MAIN) {
        window.unminimize()?;
        window.show()?;
        return window.set_focus();
    }
    WebviewWindowBuilder::new(app, MAIN, WebviewUrl::App("index.html".into()))
        .title("Memorius")
        .inner_size(1280.0, 860.0)
        .min_inner_size(720.0, 520.0)
        .build()?;
    Ok(())
}

/// Without a main window the app lives in the menu bar only (no Dock icon).
pub fn on_main_destroyed(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    #[cfg(not(target_os = "macos"))]
    let _ = app;
}

/// Marks the menu bar icon while something needs the user, for example lost
/// title access.
pub fn set_tray_alert(app: &AppHandle, alert: bool) {
    if let Some(tray) = app.tray_by_id("memorius") {
        report(tray.set_title(alert.then_some("!")));
        report(tray.set_tooltip(Some(if alert {
            "Memorius needs you: open it to fix"
        } else {
            "Memorius"
        })));
    }
}

pub fn create_tray(app: &AppHandle) -> tauri::Result<()> {
    let menu = MenuBuilder::new(app)
        .item(&MenuItemBuilder::with_id("open", "Open Memorius").build(app)?)
        .item(&MenuItemBuilder::with_id("pause", "Pause for 30 minutes").build(app)?)
        .item(&MenuItemBuilder::with_id("resume", "Resume recording").build(app)?)
        .separator()
        .item(&MenuItemBuilder::with_id("quit", "Quit Memorius").build(app)?)
        .build()?;
    TrayIconBuilder::with_id("memorius")
        .icon(tauri::include_image!("icons/tray.png"))
        .icon_as_template(true)
        .tooltip("Memorius")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => report(show_main(app)),
            "pause" => app.state::<Recording>().pause(30),
            "resume" => app.state::<Recording>().resume(),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            {
                report(toggle_popover(tray.app_handle(), rect));
            }
        })
        .build(app)?;
    Ok(())
}

fn toggle_popover(app: &AppHandle, anchor: Rect) -> tauri::Result<()> {
    if let Some(popover) = app.get_webview_window(POPOVER) {
        return popover.destroy();
    }
    let scale = app
        .primary_monitor()?
        .map_or(1.0, |monitor| monitor.scale_factor());
    let icon = anchor.position.to_physical::<f64>(scale);
    let icon_size = anchor.size.to_physical::<f64>(scale);
    let x = icon.x + icon_size.width / 2.0 - POPOVER_WIDTH * scale / 2.0;
    // Menu bar at the top (macOS) opens downwards; a taskbar at the bottom
    // (Windows) opens upwards.
    let y = if icon.y < 200.0 * scale {
        icon.y + icon_size.height
    } else {
        icon.y - POPOVER_HEIGHT * scale
    };
    let popover =
        WebviewWindowBuilder::new(app, POPOVER, WebviewUrl::App("index.html#/popover".into()))
            .title("Memorius")
            .inner_size(POPOVER_WIDTH, POPOVER_HEIGHT)
            .resizable(false)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .visible(false)
            .build()?;
    popover.set_position(PhysicalPosition::new(x, y))?;
    popover.show()?;
    popover.set_focus()?;
    let closing = popover.clone();
    popover.on_window_event(move |event| {
        if matches!(event, WindowEvent::Focused(false)) {
            let _ = closing.destroy();
        }
    });
    Ok(())
}

fn report(result: tauri::Result<()>) {
    if let Err(error) = result {
        eprintln!("memorius: window error: {error}");
    }
}
