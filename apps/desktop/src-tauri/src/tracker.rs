//! The YouTrack connection: the token in the vault, a sync of the user's
//! work items every 15 minutes, and logging approved worklogs.
#![allow(
    clippy::needless_pass_by_value,
    reason = "Tauri injects command state by value"
)]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use jiff::{ToSpan, Zoned};
use memorius_core::agent::RunState;
use memorius_core::proposal::ProposalState;
use memorius_core::store::Store;
use memorius_core::tracker::{PROJECTS, SyncState, TrackerSettings, WORK_TYPES};
use memorius_sources::youtrack;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use ts_rs::TS;

use crate::commands::Database;
use crate::recording::now_ms;
use crate::vault::Vault;

const TOKEN: &str = "youtrack-token";
const SETTINGS: &str = "tracker";
const SYNC_STATE: &str = "trackerSync";
const SYNC_EVERY: Duration = Duration::from_mins(15);
const SYNC_DAYS: i32 = 90;

pub struct Tracker {
    database: PathBuf,
    vault: Arc<Vault>,
}

impl Tracker {
    pub fn new(database: &Path, vault: Arc<Vault>) -> Self {
        Self {
            database: database.to_owned(),
            vault,
        }
    }

    /// The token saved for this address; none when the address changed.
    fn token(&self, url: &str) -> Result<Option<String>, String> {
        if url.is_empty() {
            return Ok(None);
        }
        self.vault.get(TOKEN, url)
    }

    /// Reads the last 90 days of work items and records the outcome; a
    /// failure keeps the last good data.
    fn sync(&self) -> Result<SyncState, String> {
        let store = Store::open(&self.database).map_err(|error| error.to_string())?;
        let settings: TrackerSettings = store
            .setting(SETTINGS)
            .map_err(|error| error.to_string())?
            .unwrap_or_default();
        let Some(token) = self.token(&settings.url)? else {
            return Ok(SyncState::Never);
        };
        let today = Zoned::now().date();
        let from = today
            .checked_sub(SYNC_DAYS.days())
            .map_err(|error| error.to_string())?
            .to_string();
        let to = today
            .checked_add(1.day())
            .map_err(|error| error.to_string())?
            .to_string();
        // The agent and the edit form offer only the tracker's work types.
        match youtrack::work_item_types(&settings.url, &token) {
            Ok(types) => {
                let names: Vec<String> = types.into_iter().map(|(_, name)| name).collect();
                store
                    .set_setting(WORK_TYPES, &names)
                    .map_err(|error| error.to_string())?;
            }
            Err(error) => eprintln!("memorius: cannot read YouTrack work types: {error}"),
        }
        // Only these projects' keys count as issues in the day views.
        match youtrack::projects(&settings.url, &token) {
            Ok(projects) => store
                .set_setting(PROJECTS, &projects)
                .map_err(|error| error.to_string())?,
            Err(error) => eprintln!("memorius: cannot read YouTrack projects: {error}"),
        }
        let state = match youtrack::work_items(&settings.url, &token, &from, &to) {
            Ok(items) => {
                store
                    .replace_tracker_worklogs(&from, &to, &items)
                    .map_err(|error| error.to_string())?;
                SyncState::Synced {
                    at_ms: now_ms(),
                    items: u32::try_from(items.len()).unwrap_or(u32::MAX),
                }
            }
            Err(error) => SyncState::Failed {
                at_ms: now_ms(),
                error,
            },
        };
        store
            .set_setting(SYNC_STATE, &state)
            .map_err(|error| error.to_string())?;
        Ok(state)
    }
}

/// Syncs now and then every 15 minutes.
pub fn start_sync(app: AppHandle) {
    std::thread::spawn(move || {
        loop {
            if let Err(error) = app.state::<Tracker>().sync() {
                eprintln!("memorius: the YouTrack sync failed: {error}");
            }
            std::thread::sleep(SYNC_EVERY);
        }
    });
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TrackerStatus {
    pub url: String,
    pub has_token: bool,
    pub sync: SyncState,
}

#[tauri::command]
pub fn tracker_status(
    database: State<'_, Database>,
    tracker: State<'_, Tracker>,
) -> Result<TrackerStatus, String> {
    let store = Store::open(&database.0).map_err(|error| error.to_string())?;
    let settings: TrackerSettings = store
        .setting(SETTINGS)
        .map_err(|error| error.to_string())?
        .unwrap_or_default();
    Ok(TrackerStatus {
        has_token: tracker.token(&settings.url)?.is_some(),
        url: settings.url,
        sync: store
            .setting(SYNC_STATE)
            .map_err(|error| error.to_string())?
            .unwrap_or(SyncState::Never),
    })
}

/// Saves the address, and the token when one is given, after checking them
/// against YouTrack; returns the token owner's name.
/// Off the main thread: it waits for YouTrack.
#[tauri::command(async)]
pub fn connect_tracker(
    database: State<'_, Database>,
    tracker: State<'_, Tracker>,
    url: String,
    token: Option<String>,
) -> Result<String, String> {
    let url = tracker_url(&url)?;
    let saved: TrackerSettings = Store::open(&database.0)
        .and_then(|store| store.setting(SETTINGS))
        .map_err(|error| error.to_string())?
        .unwrap_or_default();
    let token = match token.filter(|token| !token.trim().is_empty()) {
        Some(token) => token,
        // The saved token goes only to the address it was saved for.
        None if saved.url == url => tracker.token(&url)?.ok_or("Paste a YouTrack token")?,
        None => return Err("Paste the token again for a new address".into()),
    };
    let name = youtrack::me(&url, &token)?;
    tracker.vault.put(TOKEN, &url, token.trim())?;
    Store::open(&database.0)
        .and_then(|store| store.set_setting(SETTINGS, &TrackerSettings { url }))
        .map_err(|error| error.to_string())?;
    tracker.sync()?;
    Ok(name)
}

/// The tracker address as stored: https only (http just for this computer,
/// in tests), no trailing slash, so the token never travels in clear text.
fn tracker_url(url: &str) -> Result<String, String> {
    let parsed = url::Url::parse(url.trim()).map_err(|_| "Not a valid address".to_owned())?;
    let local = matches!(parsed.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if parsed.scheme() != "https" && !(parsed.scheme() == "http" && local) {
        return Err("Use an https:// address".into());
    }
    Ok(parsed.as_str().trim_end_matches('/').to_owned())
}

#[tauri::command]
pub fn disconnect_tracker(tracker: State<'_, Tracker>) -> Result<(), String> {
    tracker.vault.delete(TOKEN)
}

/// Off the main thread: it waits for YouTrack.
#[tauri::command(async)]
pub fn sync_tracker(tracker: State<'_, Tracker>) -> Result<SyncState, String> {
    tracker.sync()
}

/// The tracker's work types from the last sync; empty before one.
#[tauri::command]
pub fn tracker_work_types(database: State<'_, Database>) -> Result<Vec<String>, String> {
    Store::open(&database.0)
        .and_then(|store| store.setting(WORK_TYPES))
        .map(Option::unwrap_or_default)
        .map_err(|error| error.to_string())
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LogFailure {
    pub issue: String,
    pub date: String,
    pub error: String,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LogResult {
    pub logged: u32,
    pub failed: Vec<LogFailure>,
}

/// Logs the approved proposals (all, or only `ids`) in YouTrack with the
/// user's token, exactly as approved. A proposal YouTrack already holds (an agent or an earlier
/// try logged it) is only marked logged; a failure keeps it approved.
#[tauri::command(async)]
pub fn log_approved(
    database: State<'_, Database>,
    tracker: State<'_, Tracker>,
    ids: Option<Vec<i64>>,
) -> Result<LogResult, String> {
    let store = Store::open(&database.0).map_err(|error| error.to_string())?;
    let running = store
        .agent_runs(1)
        .map_err(|error| error.to_string())?
        .first()
        .is_some_and(|run| run.state == RunState::Running);
    if running {
        return Err("An agent run is still going; log when it ends".into());
    }
    let settings: TrackerSettings = store
        .setting(SETTINGS)
        .map_err(|error| error.to_string())?
        .unwrap_or_default();
    let token = tracker
        .token(&settings.url)?
        .ok_or("Connect YouTrack in Settings first")?;
    let approved: Vec<_> = store
        .open_proposals()
        .map_err(|error| error.to_string())?
        .into_iter()
        .filter(|proposal| matches!(proposal.state, ProposalState::Approved { .. }))
        .filter(|proposal| ids.as_ref().is_none_or(|ids| ids.contains(&proposal.id)))
        .collect();
    let mut result = LogResult {
        logged: 0,
        failed: Vec::new(),
    };
    let (Some(from), Some(to)) = (
        approved.iter().map(|p| p.date.as_str()).min(),
        approved.iter().map(|p| p.date.as_str()).max(),
    ) else {
        return Ok(result);
    };
    let existing = youtrack::work_items(&settings.url, &token, from, to)?;
    let types = if approved.iter().any(|p| p.work_type.is_some()) {
        youtrack::work_item_types(&settings.url, &token)?
    } else {
        Vec::new()
    };
    for proposal in &approved {
        let logged = match youtrack::already_logged(
            &existing,
            &proposal.issue,
            &proposal.date,
            proposal.minutes,
            &proposal.text,
        ) {
            Some(item) => Ok(item.id.clone()),
            None => match proposal.work_type.as_deref().map(|name| {
                types
                    .iter()
                    .find(|(_, known)| known.eq_ignore_ascii_case(name))
                    .map(|(id, _)| id.as_str())
                    .ok_or_else(|| format!("YouTrack has no work type {name}"))
            }) {
                Some(Err(error)) => Err(error),
                type_id => youtrack::log_work(
                    &settings.url,
                    &token,
                    &proposal.issue,
                    &proposal.date,
                    proposal.minutes,
                    &proposal.text,
                    type_id.and_then(Result::ok),
                ),
            },
        };
        let saved = match &logged {
            Ok(work_item_id) => store.mark_logged(proposal.id, work_item_id, now_ms()),
            Err(error) => store.mark_failed(proposal.id, error),
        };
        saved.map_err(|error| error.to_string())?;
        match logged {
            Ok(_) => result.logged += 1,
            Err(error) => result.failed.push(LogFailure {
                issue: proposal.issue.clone(),
                date: proposal.date.clone(),
                error,
            }),
        }
    }
    if let Err(error) = tracker.sync() {
        eprintln!("memorius: the YouTrack sync after logging failed: {error}");
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_token_goes_only_to_https_or_this_computer() {
        assert_eq!(
            tracker_url(" https://acme.youtrack.cloud/ ").unwrap(),
            "https://acme.youtrack.cloud"
        );
        assert!(tracker_url("http://127.0.0.1:8080").is_ok());
        assert!(tracker_url("http://acme.youtrack.cloud").is_err());
        assert!(tracker_url("acme.youtrack.cloud").is_err());
    }
}
