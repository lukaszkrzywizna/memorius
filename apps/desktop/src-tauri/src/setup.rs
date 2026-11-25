//! First-run setup: what is still off (start at login, notifications) and
//! the actions that turn it on. Title access comes from the recorder, the
//! agent and YouTrack from their own commands.
#![allow(
    clippy::needless_pass_by_value,
    reason = "Tauri injects command state by value"
)]

use memorius_core::store::Store;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;
use ts_rs::TS;

use crate::commands::Database;

const DONE: &str = "setupDone";
const NOTIFICATIONS: &str = "setupNotifications";

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SetupStatus {
    /// The user finished setup once; later visits come from Settings.
    pub done: bool,
    pub start_at_login: bool,
    /// The user saw the test notification arrive (no API tells whether
    /// macOS shows Memorius's notifications).
    pub notifications_seen: bool,
}

/// A System Settings page with a switch Memorius needs.
#[derive(Clone, Copy, Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SystemPane {
    LoginItems,
    Notifications,
}

fn flag(store: &Store, key: &str) -> Result<bool, String> {
    Ok(store
        .setting::<bool>(key)
        .map_err(|error| error.to_string())?
        .unwrap_or(false))
}

fn set_flag(database: &Database, key: &str) -> Result<(), String> {
    Store::open(&database.0)
        .and_then(|store| store.set_setting(key, &true))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn setup_status(app: AppHandle, database: State<'_, Database>) -> Result<SetupStatus, String> {
    let store = Store::open(&database.0).map_err(|error| error.to_string())?;
    Ok(SetupStatus {
        done: flag(&store, DONE)?,
        start_at_login: app.autolaunch().is_enabled().unwrap_or(false),
        notifications_seen: flag(&store, NOTIFICATIONS)?,
    })
}

#[tauri::command]
pub fn finish_setup(database: State<'_, Database>) -> Result<(), String> {
    set_flag(&database, DONE)
}

#[tauri::command]
pub fn confirm_notifications(database: State<'_, Database>) -> Result<(), String> {
    set_flag(&database, NOTIFICATIONS)
}

#[tauri::command]
pub fn start_at_login(app: AppHandle) -> Result<(), String> {
    app.autolaunch().enable().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn open_system_pane(pane: SystemPane) -> Result<(), String> {
    let url = match (pane, cfg!(windows)) {
        (SystemPane::LoginItems, false) => {
            "x-apple.systempreferences:com.apple.LoginItems-Settings.extension"
        }
        (SystemPane::Notifications, false) => {
            "x-apple.systempreferences:com.apple.Notifications-Settings.extension?id=app.memorius.desktop"
        }
        (SystemPane::LoginItems, true) => "ms-settings:startupapps",
        (SystemPane::Notifications, true) => "ms-settings:notifications",
    };
    let mut command = if cfg!(windows) {
        let mut start = std::process::Command::new("cmd");
        start.args(["/C", "start", "", url]);
        start
    } else {
        let mut open = std::process::Command::new("open");
        open.arg(url);
        open
    };
    command
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}
