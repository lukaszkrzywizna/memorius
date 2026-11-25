//! Updates from GitHub releases through the Tauri updater: a daily check
//! that notifies once per new version, and the commands behind Settings.

use std::time::Duration;

use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::{Update, UpdaterExt};
use ts_rs::TS;

const CHECK_EVERY: Duration = Duration::from_hours(24);

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateStatus {
    pub current: String,
    /// A newer release, if there is one.
    pub available: Option<String>,
    /// Why the check did not work; the installed version still shows.
    pub problem: Option<String>,
}

async fn newer(app: &AppHandle) -> Result<Option<Update>, String> {
    let checked = match app.updater() {
        Ok(updater) => updater.check().await,
        Err(error) => Err(error),
    };
    checked.map_err(|error| match error {
        tauri_plugin_updater::Error::ReleaseNotFound => "No published release yet".to_owned(),
        other => other.to_string(),
    })
}

/// Checks now and then once a day; notifies once per new version.
pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut notified: Option<String> = None;
        loop {
            match tauri::async_runtime::block_on(newer(&app)) {
                Ok(Some(update)) if notified.as_deref() != Some(update.version.as_str()) => {
                    let shown = app
                        .notification()
                        .builder()
                        .title(format!("Memorius {} is available", update.version))
                        .body("Open Settings to install it.")
                        .show();
                    if let Err(error) = shown {
                        eprintln!("memorius: cannot show the update notification: {error}");
                    }
                    notified = Some(update.version);
                }
                Ok(_) => {}
                Err(error) => eprintln!("memorius: the update check failed: {error}"),
            }
            std::thread::sleep(CHECK_EVERY);
        }
    });
}

#[tauri::command]
pub async fn check_update(app: AppHandle) -> UpdateStatus {
    let current = app.package_info().version.to_string();
    match newer(&app).await {
        Ok(update) => UpdateStatus {
            current,
            available: update.map(|update| update.version),
            problem: None,
        },
        Err(problem) => UpdateStatus {
            current,
            available: None,
            problem: Some(problem),
        },
    }
}

/// Downloads and installs the newer release, then restarts Memorius.
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    let update = newer(&app).await?.ok_or("Memorius is up to date")?;
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|error| error.to_string())?;
    app.restart()
}
