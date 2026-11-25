#![allow(
    clippy::needless_pass_by_value,
    reason = "Tauri injects command state by value"
)]

use std::sync::{Mutex, PoisonError};

use memorius_core::model::{AppUsage, Rule, RuleKind, SessionRef};
use memorius_core::proposal::{Proposal, ProposalEdit};
use memorius_core::search::SearchResults;
use memorius_core::store::Store;
use memorius_core::time_rules::{TIME_RULES, TimeRules, minutes_ms};
use memorius_core::timeline::app_usage;
use memorius_core::timeline::{DayTimeline, slot_ms};
use memorius_core::week::{DaySummary, RuleMatch, WeekSummary};
use memorius_sources::git::GitHistories;
use memorius_sources::views::{self, day_bounds};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use ts_rs::TS;

use crate::recording::{RecorderStatus, Recording};

/// Read-only connection for UI queries; the recorder thread owns the writer.
pub struct Reader(pub Mutex<Store>);

/// Path of the database, for commands that write (rules).
pub struct Database(pub std::path::PathBuf);

/// Git histories of session folders, for commit subjects on the timeline.
#[derive(Default)]
pub struct Git(pub Mutex<GitHistories>);

/// Activity for one local calendar day (`YYYY-MM-DD`) in slots of
/// `zoom_minutes`.
#[tauri::command]
pub fn day_timeline(
    reader: State<'_, Reader>,
    git: State<'_, Git>,
    date: &str,
    zoom_minutes: u32,
) -> Result<DayTimeline, String> {
    let (from_ms, to_ms) = day_bounds(date).map_err(|error| error.to_string())?;
    let slot_ms =
        slot_ms(zoom_minutes).ok_or_else(|| format!("zoom {zoom_minutes} is not offered"))?;
    let store = reader.0.lock().unwrap_or_else(PoisonError::into_inner);
    let mut git = git.0.lock().unwrap_or_else(PoisonError::into_inner);
    views::day_timeline(&store, &mut git, from_ms, to_ms, slot_ms)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn recorder_status(recording: State<'_, Recording>) -> RecorderStatus {
    recording.status()
}

#[tauri::command]
pub fn pause_recording(recording: State<'_, Recording>, minutes: u32) {
    recording.pause(minutes);
}

#[tauri::command]
pub fn resume_recording(recording: State<'_, Recording>) {
    recording.resume();
}

#[tauri::command]
pub fn request_title_access(recording: State<'_, Recording>) {
    recording.request_title_access();
}

#[tauri::command]
pub fn rules(reader: State<'_, Reader>) -> Result<Vec<Rule>, String> {
    let store = reader.0.lock().unwrap_or_else(PoisonError::into_inner);
    store.rules().map_err(|error| error.to_string())
}

/// Apps used in the last 14 days, for choosing apps in rules.
#[tauri::command]
pub fn recent_apps(reader: State<'_, Reader>) -> Result<Vec<AppUsage>, String> {
    let since = crate::recording::now_ms() - 14 * 24 * 3_600_000;
    let store = reader.0.lock().unwrap_or_else(PoisonError::into_inner);
    store.recent_apps(since).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn add_rule(
    database: State<'_, Database>,
    recording: State<'_, Recording>,
    rule: RuleKind,
) -> Result<(), String> {
    change_rules(&database, &recording, |store| {
        store.add_rule(&rule).map(drop)
    })
}

#[tauri::command]
pub fn remove_rule(
    database: State<'_, Database>,
    recording: State<'_, Recording>,
    id: i64,
) -> Result<(), String> {
    change_rules(&database, &recording, |store| store.remove_rule(id))
}

fn change_rules(
    database: &Database,
    recording: &Recording,
    change: impl FnOnce(&Store) -> Result<(), memorius_core::store::StoreError>,
) -> Result<(), String> {
    let store = Store::open(&database.0).map_err(|error| error.to_string())?;
    change(&store).map_err(|error| error.to_string())?;
    let private = crate::recording::private_apps(&store).map_err(|error| error.to_string())?;
    recording.set_private_apps(private);
    Ok(())
}

/// A proposal in the log queue with the apps recorded in its time range.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct QueueItem {
    pub proposal: Proposal,
    pub apps: Vec<AppUsage>,
}

/// Decided proposals stay in the queue this long, so a decision can be undone.
const UNDO_WINDOW_MS: i64 = 24 * 3_600_000;

#[tauri::command]
pub fn log_queue(reader: State<'_, Reader>) -> Result<Vec<QueueItem>, String> {
    let store = reader.0.lock().unwrap_or_else(PoisonError::into_inner);
    let proposals = store
        .queue_proposals(crate::recording::now_ms() - UNDO_WINDOW_MS)
        .map_err(|error| error.to_string())?;
    proposals
        .into_iter()
        .map(|proposal| {
            let rows = store
                .rows_between(proposal.start_ms, proposal.end_ms)
                .map_err(|error| error.to_string())?;
            let apps = app_usage(&rows, proposal.start_ms, proposal.end_ms);
            Ok(QueueItem { proposal, apps })
        })
        .collect()
}

/// Marks these sessions private, or not private again (undo): their
/// prompts and agent time never count and they name no issue.
#[tauri::command]
pub fn set_sessions_private(
    database: State<'_, Database>,
    sessions: Vec<SessionRef>,
    private: bool,
) -> Result<(), String> {
    Store::open(&database.0)
        .and_then(|store| {
            store.set_sessions_private(&sessions, private, crate::recording::now_ms())
        })
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn approve_proposal(
    database: State<'_, Database>,
    id: i64,
    revision: u32,
    edit: Option<ProposalEdit>,
) -> Result<Proposal, String> {
    let store = Store::open(&database.0).map_err(|error| error.to_string())?;
    store
        .approve_proposal(id, revision, edit, crate::recording::now_ms())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn reject_proposal(database: State<'_, Database>, id: i64) -> Result<Proposal, String> {
    let store = Store::open(&database.0).map_err(|error| error.to_string())?;
    store
        .reject_proposal(id, crate::recording::now_ms())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn undo_proposal(database: State<'_, Database>, id: i64) -> Result<Proposal, String> {
    let store = Store::open(&database.0).map_err(|error| error.to_string())?;
    store.undo_proposal(id).map_err(|error| error.to_string())
}

/// The week timesheet for an ISO week (`2026-W41`) or a date in it.
#[tauri::command]
pub fn week_summary(
    reader: State<'_, Reader>,
    git: State<'_, Git>,
    week: &str,
) -> Result<WeekSummary, String> {
    let store = reader.0.lock().unwrap_or_else(PoisonError::into_inner);
    let mut git = git.0.lock().unwrap_or_else(PoisonError::into_inner);
    views::week_summary(&store, &mut git, week).map_err(|error| error.to_string())
}

/// Totals of the given days, for the Log queue's day rows.
#[tauri::command]
pub fn day_summaries(
    reader: State<'_, Reader>,
    git: State<'_, Git>,
    dates: Vec<String>,
) -> Result<Vec<DaySummary>, String> {
    let store = reader.0.lock().unwrap_or_else(PoisonError::into_inner);
    let mut git = git.0.lock().unwrap_or_else(PoisonError::into_inner);
    views::day_summaries(&store, &mut git, &dates).map_err(|error| error.to_string())
}

/// The time each rule named in the last 7 days, for the rules list.
#[tauri::command]
pub fn rule_matches(
    reader: State<'_, Reader>,
    git: State<'_, Git>,
) -> Result<Vec<RuleMatch>, String> {
    let today = jiff::Zoned::now().date().to_string();
    let store = reader.0.lock().unwrap_or_else(PoisonError::into_inner);
    let mut git = git.0.lock().unwrap_or_else(PoisonError::into_inner);
    views::rule_matches_last_week(&store, &mut git, &today).map_err(|error| error.to_string())
}

/// Stored windows, sessions and proposals of the last 90 days that match
/// `query`, for the search palette. Never prompt text; not offered to MCP.
#[tauri::command]
pub fn search(reader: State<'_, Reader>, query: &str) -> Result<SearchResults, String> {
    let since = crate::recording::now_ms() - 90 * 24 * 3_600_000;
    let store = reader.0.lock().unwrap_or_else(PoisonError::into_inner);
    store
        .search(query, since)
        .map_err(|error| error.to_string())
}

/// The text of the prompt sent at `at_ms` in one of `sessions`, read from
/// its session file on demand for the hover card; `None` when the file or
/// line is gone. Never stored and never offered to MCP or the CLI.
#[tauri::command]
pub fn prompt_text(
    reader: State<'_, Reader>,
    sessions: Vec<SessionRef>,
    at_ms: i64,
) -> Result<Option<String>, String> {
    let line = {
        let store = reader.0.lock().unwrap_or_else(PoisonError::into_inner);
        store
            .prompt_line(&sessions, at_ms)
            .map_err(|error| error.to_string())?
    };
    let Some((source, path, offset)) = line else {
        return Ok(None);
    };
    Ok(
        memorius_sources::sessions::prompt_text(std::path::Path::new(&path), source, offset)
            .unwrap_or(None),
    )
}

/// Opens (or focuses) the main window, for the menu bar popover.
#[tauri::command]
pub fn show_main_window(app: AppHandle) -> Result<(), String> {
    crate::shell::show_main(&app).map_err(|error| error.to_string())
}

/// Restarts Memorius, for a permission that macOS applies only to a new
/// process.
#[tauri::command]
pub fn restart_app(app: AppHandle) {
    app.state::<Recording>().stop();
    app.restart();
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub fn time_rules(database: State<'_, Database>) -> Result<TimeRules, String> {
    Store::open(&database.0)
        .and_then(|store| store.setting::<TimeRules>(TIME_RULES))
        .map(|rules| rules.unwrap_or_default().clamped())
        .map_err(|error| error.to_string())
}

/// Saves the rules within their bounds; the recorder takes the away time
/// at once, and every view counts with the new rules.
#[tauri::command]
pub fn save_time_rules(
    database: State<'_, Database>,
    recording: State<'_, Recording>,
    rules: TimeRules,
) -> Result<TimeRules, String> {
    let rules = rules.clamped();
    Store::open(&database.0)
        .and_then(|store| store.set_setting(TIME_RULES, &rules))
        .map_err(|error| error.to_string())?;
    recording.set_away_after(minutes_ms(rules.away_after_minutes));
    Ok(rules)
}
