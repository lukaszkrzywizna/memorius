//! The daily reminder about time that is not logged yet.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct ReminderSettings {
    pub enabled: bool,
    /// Local time, `HH:MM`.
    pub time: String,
}

impl Default for ReminderSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            time: "16:00".into(),
        }
    }
}
