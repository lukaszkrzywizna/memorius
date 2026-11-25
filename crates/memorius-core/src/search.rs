//! "When did I work on this?": stored windows, agent sessions and
//! proposals that match a few words, over a bounded period.

use serde::Serialize;
use ts_rs::TS;

use crate::model::{AgentSource, AppRef};

/// Shorter queries match too much to be useful.
pub const MIN_QUERY_CHARS: usize = 2;
pub const MAX_QUERY_CHARS: usize = 100;
/// Hits kept per group, most recent first.
pub const MAX_HITS: usize = 20;
/// Rows read per group before the result says it is cut short.
pub const MAX_ROWS: usize = 5_000;
/// Time ranges kept per hit; enough to name the days.
pub const MAX_SPANS: usize = 30;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SearchResults {
    pub issues: Vec<IssueHit>,
    pub windows: Vec<WindowHit>,
    pub sessions: Vec<SessionHit>,
    /// More rows matched than were read: refine the query.
    pub truncated: bool,
}

/// An issue with its proposals: logged and proposed minutes stay apart.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct IssueHit {
    pub issue: String,
    pub worklogs: Vec<IssueWorklog>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct IssueWorklog {
    #[ts(type = "number")]
    pub start_ms: i64,
    pub minutes: u32,
    pub logged: bool,
}

/// One window title (or address) of one app, with when it was open.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WindowHit {
    pub app: AppRef,
    pub title: String,
    pub url: Option<String>,
    #[ts(type = "number")]
    pub total_ms: i64,
    /// Most recent first.
    #[ts(type = "Array<[number, number]>")]
    pub spans: Vec<(i64, i64)>,
}

/// One agent session, named by its title or branch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SessionHit {
    pub source: AgentSource,
    pub title: Option<String>,
    pub branch: Option<String>,
    #[ts(type = "number")]
    pub start_ms: i64,
    #[ts(type = "number")]
    pub end_ms: i64,
}

/// The query as a `LIKE` pattern, with `%`, `_` and `\` taken literally;
/// `None` when it is too short or too long.
pub fn like_pattern(query: &str) -> Option<String> {
    let query = query.trim();
    let chars = query.chars().count();
    if !(MIN_QUERY_CHARS..=MAX_QUERY_CHARS).contains(&chars) {
        return None;
    }
    let mut pattern = String::with_capacity(query.len() + 2);
    pattern.push('%');
    for char in query.chars() {
        if matches!(char, '%' | '_' | '\\') {
            pattern.push('\\');
        }
        pattern.push(char);
    }
    pattern.push('%');
    Some(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcards_in_the_query_match_only_themselves() {
        assert_eq!(
            like_pattern(" 50%_off\\ ").as_deref(),
            Some("%50\\%\\_off\\\\%")
        );
        assert_eq!(like_pattern("a"), None);
        assert_eq!(like_pattern(&"x".repeat(101)), None);
    }
}
