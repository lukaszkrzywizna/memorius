//! Once a day at the user's time, a notification about this week's time
//! that nothing logged or proposed covers yet.
#![allow(
    clippy::needless_pass_by_value,
    reason = "Tauri injects command state by value"
)]

use std::path::{Path, PathBuf};
use std::time::Duration;

use jiff::Zoned;
use memorius_core::reminder::ReminderSettings;
use memorius_core::store::Store;
use memorius_sources::agent::daily_due;
use memorius_sources::git::GitHistories;
use memorius_sources::views;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_notification::NotificationExt;

use crate::commands::Database;

const SETTINGS: &str = "reminder";
/// The local date of the last reminder, so it shows once a day.
const LAST_DATE: &str = "reminderDate";
/// Less than this is not worth a reminder.
const MIN_MS: i64 = 15 * 60_000;

pub struct Reminder {
    database: PathBuf,
}

impl Reminder {
    pub fn new(database: &Path) -> Self {
        Self {
            database: database.to_owned(),
        }
    }

    /// This week's and today's time that is not logged or proposed yet.
    fn unlogged(&self, now: &Zoned) -> Result<(i64, i64), String> {
        let store = Store::open(&self.database).map_err(|error| error.to_string())?;
        let today = now.date().to_string();
        let week = views::week_summary(&store, &mut GitHistories::default(), &today)
            .map_err(|error| error.to_string())?;
        let days = week.days.iter().filter(|day| day.date <= today);
        let (mut week_ms, mut today_ms) = (0, 0);
        for day in days {
            week_ms += day.unlogged_ms;
            if day.date == today {
                today_ms = day.unlogged_ms;
            }
        }
        Ok((week_ms, today_ms))
    }

    /// Shows the reminder when it is due; called every minute, so one
    /// missed while the Mac slept shows after it wakes.
    fn tick(&self, app: &AppHandle) -> Result<(), String> {
        let store = Store::open(&self.database).map_err(|error| error.to_string())?;
        let settings: ReminderSettings = store
            .setting(SETTINGS)
            .map_err(|error| error.to_string())?
            .unwrap_or_default();
        let now = Zoned::now();
        let last: Option<String> = store
            .setting(LAST_DATE)
            .map_err(|error| error.to_string())?;
        if !daily_due(settings.enabled, &settings.time, &now, last.as_deref()) {
            return Ok(());
        }
        store
            .set_setting(LAST_DATE, &now.date().to_string())
            .map_err(|error| error.to_string())?;
        let (week_ms, today_ms) = self.unlogged(&now)?;
        if let Some((title, body)) = message(week_ms, today_ms) {
            notify(app, &title, &body)?;
        }
        Ok(())
    }
}

fn notify(app: &AppHandle, title: &str, body: &str) -> Result<(), String> {
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|error| error.to_string())
}

fn duration(ms: i64) -> String {
    let minutes = ms / 60_000;
    match (minutes / 60, minutes % 60) {
        (0, minutes) => format!("{minutes}m"),
        (hours, 0) => format!("{hours}h"),
        (hours, minutes) => format!("{hours}h {minutes:02}m"),
    }
}

/// The notification's title and text; none when little is left.
fn message(week_ms: i64, today_ms: i64) -> Option<(String, String)> {
    (week_ms >= MIN_MS).then(|| {
        (
            format!("{} not logged this week", duration(week_ms)),
            format!(
                "{} of it today. Open Memorius to prepare proposals.",
                duration(today_ms)
            ),
        )
    })
}

/// Checks every minute whether the reminder is due.
pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_mins(1));
            if let Err(error) = app.state::<Reminder>().tick(&app) {
                eprintln!("memorius: the daily reminder failed: {error}");
            }
        }
    });
}

#[tauri::command]
pub fn reminder_settings(database: State<'_, Database>) -> Result<ReminderSettings, String> {
    Store::open(&database.0)
        .and_then(|store| store.setting(SETTINGS))
        .map(Option::unwrap_or_default)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_reminder_settings(
    database: State<'_, Database>,
    settings: ReminderSettings,
) -> Result<(), String> {
    Store::open(&database.0)
        .and_then(|store| store.set_setting(SETTINGS, &settings))
        .map_err(|error| error.to_string())
}

/// Shows the reminder now, so the user can check that notifications work.
/// Off the main thread: it builds this week's summary.
#[tauri::command(async)]
pub fn test_reminder(app: AppHandle, reminder: State<'_, Reminder>) -> Result<(), String> {
    let (week_ms, today_ms) = reminder.unlogged(&Zoned::now())?;
    let (title, body) = message(week_ms, today_ms).unwrap_or_else(|| {
        (
            "Everything is logged".into(),
            "Memorius will remind you here when time is not logged.".into(),
        )
    });
    notify(&app, &title, &body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_message_names_the_week_and_today_and_skips_small_leftovers() {
        let minutes = |count: i64| count * 60_000;

        assert_eq!(
            message(minutes(200), minutes(65)),
            Some((
                "3h 20m not logged this week".into(),
                "1h 05m of it today. Open Memorius to prepare proposals.".into()
            ))
        );
        assert_eq!(message(minutes(14), minutes(14)), None);
        assert_eq!(duration(minutes(120)), "2h");
    }
}
