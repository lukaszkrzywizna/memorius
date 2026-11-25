//! Work items read from the user's issue tracker (YouTrack), so Week and
//! the agent see time logged outside Memorius too.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The setting that holds the tracker's work item type names, read at each
/// sync; empty until the first one.
pub const WORK_TYPES: &str = "trackerWorkTypes";

/// The setting that holds the tracker's project keys (`INT`, `SEL`), read at
/// each sync, so only real projects count as issue keys.
pub const PROJECTS: &str = "trackerProjects";

/// One logged work item: an issue, a day and minutes. Trackers keep no
/// start or end time.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackerWorklog {
    pub id: String,
    pub issue: String,
    pub summary: Option<String>,
    /// `YYYY-MM-DD`.
    pub date: String,
    pub minutes: u32,
    pub text: Option<String>,
    pub work_type: Option<String>,
}

/// Where the tracker is; the token lives in the system keychain.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TrackerSettings {
    /// For example `https://example.youtrack.cloud`; empty when not set up.
    pub url: String,
}

/// The last sync's outcome.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum SyncState {
    Never,
    #[serde(rename_all = "camelCase")]
    Synced {
        #[ts(type = "number")]
        at_ms: i64,
        items: u32,
    },
    #[serde(rename_all = "camelCase")]
    Failed {
        #[ts(type = "number")]
        at_ms: i64,
        error: String,
    },
}
