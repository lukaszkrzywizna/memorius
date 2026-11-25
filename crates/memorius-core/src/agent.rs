//! The coding agent Memorius starts to prepare and log worklogs, and the
//! record of each run.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Which agent CLI Memorius starts. The CLI uses its own login, so the
/// user's subscription pays and Memorius holds no key.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum AgentKind {
    #[default]
    ClaudeCode,
    Codex,
    /// The user's own command line; `{prompt}` is replaced by the prompt.
    Custom,
}

impl AgentKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ClaudeCode => "claudeCode",
            Self::Codex => "codex",
            Self::Custom => "custom",
        }
    }

    pub fn parse(text: &str) -> Self {
        match text {
            "codex" => Self::Codex,
            "custom" => Self::Custom,
            _ => Self::ClaudeCode,
        }
    }
}

/// How far back a proposals run looks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum RunRange {
    /// From the end of the last finished proposals run (48 hours the first
    /// time), at most a month.
    #[default]
    SinceLastRun,
    Day,
    TwoDays,
    Week,
    Month,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct AgentSettings {
    pub agent: AgentKind,
    /// The command line for [`AgentKind::Custom`].
    pub custom_command: String,
    /// Starts a proposals run every day at `daily_time`, up to that moment.
    pub daily: bool,
    /// Local time of the daily run, `HH:MM`.
    pub daily_time: String,
    pub default_range: RunRange,
    /// Read-only tracker tools a proposals run may call to check issues,
    /// `server/tool` separated by commas; empty for none.
    pub lookup_tools: String,
    /// The user's own words added to the proposals prompt.
    pub instructions: String,
    /// Where Claude Code keeps its sessions; empty: `CLAUDE_CONFIG_DIR` or
    /// `~/.claude`.
    pub claude_home: String,
    /// Where Codex keeps its sessions; empty: `CODEX_HOME` or `~/.codex`.
    pub codex_home: String,
}

impl Default for AgentSettings {
    fn default() -> Self {
        Self {
            agent: AgentKind::ClaudeCode,
            custom_command: String::new(),
            daily: false,
            daily_time: "08:00".into(),
            default_range: RunRange::SinceLastRun,
            lookup_tools: String::new(),
            instructions: String::new(),
            claude_home: String::new(),
            codex_home: String::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum RunState {
    Running,
    #[serde(rename_all = "camelCase")]
    Done {
        #[ts(type = "number")]
        finished_ms: i64,
        /// The agent's last message.
        summary: String,
    },
    #[serde(rename_all = "camelCase")]
    Failed {
        #[ts(type = "number")]
        finished_ms: i64,
        error: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AgentRun {
    #[ts(type = "number")]
    pub id: i64,
    pub agent: AgentKind,
    /// The range the run proposed worklogs for.
    #[ts(type = "number")]
    pub from_ms: i64,
    #[ts(type = "number")]
    pub to_ms: i64,
    #[ts(type = "number")]
    pub started_ms: i64,
    pub state: RunState,
}
