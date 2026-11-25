//! Worklog proposals: agents propose, the user approves or rejects, the agent
//! logs the approved values to YouTrack and reports the work item id.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::issue::issue_keys;

/// Longest proposal text; YouTrack work item descriptions stay readable.
const MAX_TEXT: usize = 2_000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum ProposalState {
    Proposed,
    #[serde(rename_all = "camelCase")]
    Approved {
        #[ts(type = "number")]
        at_ms: i64,
    },
    #[serde(rename_all = "camelCase")]
    Rejected {
        #[ts(type = "number")]
        at_ms: i64,
    },
    #[serde(rename_all = "camelCase")]
    Logged {
        #[ts(type = "number")]
        at_ms: i64,
        work_item_id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Proposal {
    #[ts(type = "number")]
    pub id: i64,
    pub issue: String,
    /// The local day it is logged on, `YYYY-MM-DD`: trackers log per issue
    /// and day, with no start or end time.
    pub date: String,
    /// The first and last moment of the work behind it, as evidence.
    #[ts(type = "number")]
    pub start_ms: i64,
    #[ts(type = "number")]
    pub end_ms: i64,
    pub minutes: u32,
    pub work_type: Option<String>,
    pub text: String,
    /// Who proposed it, for example "Claude Code".
    pub agent: String,
    #[ts(type = "number")]
    pub created_ms: i64,
    pub state: ProposalState,
    /// The last failure to log an approved proposal.
    pub last_error: Option<String>,
    /// Grows each time an agent changes the waiting proposal; approving
    /// names the revision the user saw.
    pub revision: u32,
}

/// What an agent sends to propose a worklog.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewProposal {
    pub issue: String,
    pub date: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub minutes: u32,
    pub work_type: Option<String>,
    pub text: String,
    pub agent: String,
}

/// Values the user changes while approving.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ProposalEdit {
    pub issue: String,
    pub minutes: u32,
    pub work_type: Option<String>,
    pub text: String,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ProposalError {
    #[error("{0:?} is not a YouTrack issue key such as INT-290")]
    IssueKey(String),
    #[error("the time range must end after it starts")]
    Range,
    #[error("{0:?} is not a date such as 2026-10-16")]
    Date(String),
    #[error("YouTrack has no work type {0:?}; use one of {1}, or leave it out")]
    WorkType(String, String),
    #[error("minutes must be between 1 and the {0} minutes of the range")]
    Minutes(u32),
    #[error("the text must not be empty and must stay under {MAX_TEXT} characters")]
    Text,
    #[error("a {from} proposal cannot be {action}")]
    Transition {
        from: &'static str,
        action: &'static str,
    },
    #[error("proposal {0} is already logged as {1}")]
    AlreadyLogged(i64, String),
    #[error("the YouTrack work item id must not be empty")]
    WorkItemId,
    #[error("the proposal changed since you saw it; review it again")]
    Changed,
}

impl NewProposal {
    /// The proposal with a normalized issue key, or why it is not valid.
    pub fn validated(mut self) -> Result<Self, ProposalError> {
        self.issue = normalized_issue(&self.issue)?;
        if !is_iso_date(&self.date) {
            return Err(ProposalError::Date(self.date));
        }
        if self.end_ms <= self.start_ms {
            return Err(ProposalError::Range);
        }
        check_minutes(self.minutes, self.start_ms, self.end_ms)?;
        self.text = checked_text(&self.text)?;
        Ok(self)
    }
}

impl ProposalEdit {
    pub fn validated(mut self, start_ms: i64, end_ms: i64) -> Result<Self, ProposalError> {
        self.issue = normalized_issue(&self.issue)?;
        check_minutes(self.minutes, start_ms, end_ms)?;
        self.text = checked_text(&self.text)?;
        Ok(self)
    }
}

fn normalized_issue(issue: &str) -> Result<String, ProposalError> {
    let upper = issue.trim().to_ascii_uppercase();
    if issue_keys(&upper) == [upper.clone()] {
        Ok(upper)
    } else {
        Err(ProposalError::IssueKey(issue.to_owned()))
    }
}

fn is_iso_date(text: &str) -> bool {
    text.len() == 10
        && text.char_indices().all(|(index, c)| match index {
            4 | 7 => c == '-',
            _ => c.is_ascii_digit(),
        })
}

pub(crate) fn check_minutes(minutes: u32, start_ms: i64, end_ms: i64) -> Result<(), ProposalError> {
    let range = u32::try_from((end_ms - start_ms) / 60_000).unwrap_or(u32::MAX);
    if minutes == 0 || minutes > range.max(1) {
        return Err(ProposalError::Minutes(range));
    }
    Ok(())
}

fn checked_text(text: &str) -> Result<String, ProposalError> {
    let text = text.trim();
    if text.is_empty() || text.chars().count() > MAX_TEXT {
        return Err(ProposalError::Text);
    }
    Ok(text.to_owned())
}

fn name(state: &ProposalState) -> &'static str {
    match state {
        ProposalState::Proposed => "proposed",
        ProposalState::Approved { .. } => "approved",
        ProposalState::Rejected { .. } => "rejected",
        ProposalState::Logged { .. } => "logged",
    }
}

/// The user approves a proposed worklog.
pub fn approve(state: &ProposalState, now_ms: i64) -> Result<ProposalState, ProposalError> {
    match state {
        ProposalState::Proposed => Ok(ProposalState::Approved { at_ms: now_ms }),
        other => Err(ProposalError::Transition {
            from: name(other),
            action: "approved",
        }),
    }
}

/// The user rejects a proposed worklog; the agent will not propose it again.
pub fn reject(state: &ProposalState, now_ms: i64) -> Result<ProposalState, ProposalError> {
    match state {
        ProposalState::Proposed => Ok(ProposalState::Rejected { at_ms: now_ms }),
        other => Err(ProposalError::Transition {
            from: name(other),
            action: "rejected",
        }),
    }
}

/// The user takes back an approval or a rejection, until the agent logs it.
pub fn undo(state: &ProposalState) -> Result<ProposalState, ProposalError> {
    match state {
        ProposalState::Approved { .. } | ProposalState::Rejected { .. } => {
            Ok(ProposalState::Proposed)
        }
        other => Err(ProposalError::Transition {
            from: name(other),
            action: "undone",
        }),
    }
}

/// The agent logged an approved worklog. Reporting the same work item again
/// is fine; logging twice is not.
pub fn mark_logged(
    id: i64,
    state: &ProposalState,
    work_item_id: &str,
    now_ms: i64,
) -> Result<ProposalState, ProposalError> {
    if work_item_id.trim().is_empty() {
        return Err(ProposalError::WorkItemId);
    }
    match state {
        ProposalState::Approved { .. } => Ok(ProposalState::Logged {
            at_ms: now_ms,
            work_item_id: work_item_id.to_owned(),
        }),
        ProposalState::Logged {
            work_item_id: logged,
            ..
        } if logged == work_item_id => Ok(state.clone()),
        ProposalState::Logged {
            work_item_id: logged,
            ..
        } => Err(ProposalError::AlreadyLogged(id, logged.clone())),
        other => Err(ProposalError::Transition {
            from: name(other),
            action: "logged",
        }),
    }
}

/// Only approved proposals record a logging failure.
pub fn check_can_fail(state: &ProposalState) -> Result<(), ProposalError> {
    match state {
        ProposalState::Approved { .. } => Ok(()),
        other => Err(ProposalError::Transition {
            from: name(other),
            action: "marked failed",
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOUR: i64 = 3_600_000;

    fn new(issue: &str, minutes: u32, text: &str) -> NewProposal {
        NewProposal {
            issue: issue.into(),
            date: "2026-10-16".into(),
            start_ms: 0,
            end_ms: HOUR,
            minutes,
            work_type: None,
            text: text.into(),
            agent: "Claude Code".into(),
        }
    }

    #[test]
    fn validation_normalizes_keys_and_rejects_bad_input() {
        assert_eq!(
            new(" int-290 ", 45, " Fixed it ")
                .validated()
                .unwrap()
                .issue,
            "INT-290"
        );
        assert_eq!(
            new("INT 290", 45, "x").validated(),
            Err(ProposalError::IssueKey("INT 290".into()))
        );
        assert_eq!(
            new("INT-290", 61, "x").validated(),
            Err(ProposalError::Minutes(60))
        );
        assert_eq!(
            new("INT-290", 0, "x").validated(),
            Err(ProposalError::Minutes(60))
        );
        assert_eq!(
            new("INT-290", 10, "  ").validated(),
            Err(ProposalError::Text)
        );
    }

    #[test]
    fn approval_flow_and_undo() {
        let approved = approve(&ProposalState::Proposed, 5).unwrap();
        assert_eq!(approved, ProposalState::Approved { at_ms: 5 });
        assert_eq!(undo(&approved).unwrap(), ProposalState::Proposed);
        assert!(approve(&approved, 6).is_err());
        let rejected = reject(&ProposalState::Proposed, 7).unwrap();
        assert!(approve(&rejected, 8).is_err());
    }

    #[test]
    fn logging_needs_approval_and_happens_once() {
        assert!(mark_logged(1, &ProposalState::Proposed, "2-123", 9).is_err());
        let logged = mark_logged(1, &ProposalState::Approved { at_ms: 5 }, "2-123", 9).unwrap();

        assert_eq!(mark_logged(1, &logged, "2-123", 10).unwrap(), logged);
        assert_eq!(
            mark_logged(1, &logged, "2-999", 10),
            Err(ProposalError::AlreadyLogged(1, "2-123".into()))
        );
        assert!(undo(&logged).is_err());
    }
}
