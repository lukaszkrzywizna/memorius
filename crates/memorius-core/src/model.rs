use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The application that owns the foreground window.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AppRef {
    /// Stable identity: bundle id on macOS, executable path on Windows.
    pub id: String,
    pub name: String,
}

/// Why a window title is not known. Never represented as an empty string.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum TitleUnavailable {
    /// macOS Accessibility permission is missing.
    PermissionDenied,
    /// The app has no focused window.
    NoWindow,
    /// The user marked the app private.
    Private,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum Title {
    Known { text: String },
    Unavailable { reason: TitleUnavailable },
}

/// What the foreground window shows at one moment.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Window {
    pub app: AppRef,
    pub title: Title,
    /// File path of the focused document, when the app exposes it.
    pub document: Option<String>,
    /// Address of the active browser tab without query or fragment.
    pub url: Option<String>,
}

/// `scheme://host/path` of a URL: queries and fragments often hold tokens.
pub fn url_without_query(url: &str) -> &str {
    url.split(['?', '#']).next().unwrap_or(url)
}

/// One reading of the user's state, taken by the platform recorder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Observation {
    Active(Window),
    /// No input since `since_ms`; the time from then on is away time.
    Away {
        since_ms: i64,
    },
    Locked,
}

/// A coding agent whose session files show what the user worked on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum AgentSource {
    ClaudeCode,
    Codex,
}

/// Where reading one agent session file stopped, and what it said so far.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SessionCursor {
    pub offset: u64,
    pub session_id: Option<String>,
    pub cwd: Option<String>,
    pub branch: Option<String>,
    pub title: Option<String>,
    /// A helper agent's file: its time counts for the parent session, its
    /// messages are not the user's prompts.
    pub subagent: bool,
}

/// A stretch of one agent session with no pause longer than
/// [`crate::context::SESSION_GAP_MS`] and one branch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextSpan {
    pub source: AgentSource,
    pub session_id: String,
    pub title: Option<String>,
    pub cwd: Option<String>,
    pub branch: Option<String>,
    pub start_ms: i64,
    pub end_ms: i64,
    /// When the user typed a message in this stretch.
    pub prompt_times: Vec<i64>,
}

/// A user rule: hide an app's details, or name the issue for matching work.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum RuleKind {
    /// Record only the app and its time, never titles, documents or URLs.
    #[serde(rename_all = "camelCase")]
    PrivateApp { app_id: String },
    #[serde(rename_all = "camelCase")]
    AppIssue { app_id: String, issue: String },
    /// Matches the URL host and its subdomains.
    HostIssue { host: String, issue: String },
    /// Matches window titles containing the text, ignoring case.
    TitleIssue { text: String, issue: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Rule {
    #[ts(type = "number")]
    pub id: i64,
    pub rule: RuleKind,
}

/// Time spent in one app, for choosing apps in rules.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AppUsage {
    pub app: AppRef,
    #[ts(type = "number")]
    pub duration_ms: i64,
}

/// A private app's window: the app stays, its details go.
pub fn redact_private(window: Window) -> Window {
    Window {
        app: window.app,
        title: Title::Unavailable {
            reason: TitleUnavailable::Private,
        },
        document: None,
        url: None,
    }
}

/// One agent session: its tool and the id in its log.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SessionRef {
    pub source: AgentSource,
    pub session_id: String,
}
