//! The operations agents run, shared by the command line and the MCP server.
//! Results are JSON with local ISO times and minutes.

use std::path::Path;

use jiff::ToSpan;
use jiff::civil::Date;
use memorius_core::model::Title;
use memorius_core::proposal::{NewProposal, Proposal, ProposalState};
use memorius_core::store::{SCHEMA_VERSION, Store};
use memorius_core::time_rules::WeekStart;
use memorius_core::timeline::{Block, DEFAULT_SLOT_MS, TimeSplit, WorkSpan};
use memorius_core::week::{DayLedger, MIN_UNLOGGED_MS, WeekCell};
use memorius_sources::git::GitHistories;
use memorius_sources::views::{day_bounds, day_timeline, week_dates, week_summary};
use serde_json::{Value, json};

use crate::time::{local_iso, parse_time};

/// Bumped when the JSON shape changes in a way agents must notice.
pub const OUTPUT_VERSION: u32 = 9;

/// The Git commit this binary was built from (see `build.rs`).
pub const BUILD: &str = env!("MEMORIUS_BUILD");

/// Longest range for one `unlogged` or `proposals` call.
const MAX_DAYS: i64 = 31;

/// Windows listed per block; the rest add little for a worklog.
const WINDOWS_PER_BLOCK: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Op {
    Day {
        date: String,
    },
    Week {
        week: String,
    },
    Unlogged {
        from: String,
        to: String,
    },
    Proposals {
        from: String,
        to: String,
    },
    Propose {
        issue: String,
        date: String,
        start: Option<String>,
        end: Option<String>,
        minutes: u32,
        text: String,
        work_type: Option<String>,
        agent: String,
    },
    /// Which build answers, its output and schema versions, and how fresh
    /// the imported sessions are.
    Status,
}

pub fn run(op: &Op, database: &Path, now_ms: i64) -> Result<Value, String> {
    let text = |error: &dyn std::fmt::Display| error.to_string();
    match op {
        Op::Status => {
            let store = Store::open_read_only(database).map_err(|e| text(&e))?;
            let newest = store.newest_session_file_ms().map_err(|e| text(&e))?;
            Ok(json!({
                "version": OUTPUT_VERSION,
                "build": BUILD,
                "schemaVersion": SCHEMA_VERSION,
                "newestImportedSession": newest.map(local_iso),
            }))
        }
        Op::Day { date } => {
            let store = Store::open_read_only(database).map_err(|e| text(&e))?;
            day(&store, date)
        }
        Op::Week { week } => {
            let store = Store::open_read_only(database).map_err(|e| text(&e))?;
            week_view(&store, week)
        }
        Op::Unlogged { from, to } => {
            let store = Store::open_read_only(database).map_err(|e| text(&e))?;
            unlogged(&store, from, to)
        }
        Op::Proposals { from, to } => {
            let store = Store::open_read_only(database).map_err(|e| text(&e))?;
            let (from_ms, _) = day_bounds(from).map_err(|e| text(&e))?;
            let (_, to_ms) = day_bounds(to).map_err(|e| text(&e))?;
            let proposals = store
                .proposals_between(from_ms, to_ms)
                .map_err(|e| text(&e))?;
            Ok(
                json!({ "version": OUTPUT_VERSION, "proposals": proposals.iter().map(proposal_view).collect::<Vec<_>>() }),
            )
        }
        Op::Propose {
            issue,
            date,
            start,
            end,
            minutes,
            text: description,
            work_type,
            agent,
        } => {
            let store = Store::open_existing(database).map_err(|e| text(&e))?;
            let (day_start, day_end) = day_bounds(date).map_err(|e| text(&e))?;
            let new = NewProposal {
                issue: issue.clone(),
                date: date.clone(),
                start_ms: start.as_deref().map_or(Ok(day_start), parse_time)?,
                end_ms: end.as_deref().map_or(Ok(day_end), parse_time)?,
                minutes: *minutes,
                work_type: work_type.clone(),
                text: description.clone(),
                agent: agent.clone(),
            };
            let (proposal, created) = store.propose(new, now_ms).map_err(|e| text(&e))?;
            let note = if created {
                "Proposed. It waits for the user's approval in Memorius, which logs it once approved.".to_owned()
            } else {
                format!(
                    "This issue and day was already proposed; it now holds these values if it was still waiting. Its state is {}.",
                    state_name(&proposal.state)
                )
            };
            Ok(
                json!({ "version": OUTPUT_VERSION, "created": created, "note": note, "proposal": proposal_view(&proposal) }),
            )
        }
    }
}

fn day(store: &Store, date: &str) -> Result<Value, String> {
    let (from_ms, to_ms) = day_bounds(date).map_err(|e| e.to_string())?;
    let timeline = day_timeline(
        store,
        &mut GitHistories::default(),
        from_ms,
        to_ms,
        DEFAULT_SLOT_MS,
    )
    .map_err(|e| e.to_string())?;
    let proposals = store
        .proposals_between(from_ms, to_ms)
        .map_err(|e| e.to_string())?;
    let worklogs = store
        .tracker_worklogs(date, date)
        .map_err(|e| e.to_string())?;
    let ledger = DayLedger::new(date, &timeline, &proposals, &worklogs);
    Ok(json!({
        "version": OUTPUT_VERSION,
        "date": date,
        "activeMinutes": minutes(timeline.totals.active_ms),
        "awayMinutes": minutes(timeline.totals.away_ms),
        "agentMinutes": minutes(timeline.totals.agent_ms),
        "workMinutes": minutes(timeline.totals.split.work_ms),
        "uncountedAgentMinutes": minutes(timeline.totals.split.uncounted_ms),
        "unloggedMinutes": minutes(ledger.unlogged_ms()),
        "logged": by_issue(&ledger.logged),
        "proposed": by_issue(&ledger.proposed),
        "work": timeline.work.iter().map(work_view).collect::<Vec<_>>(),
        "blocks": timeline.blocks.iter().map(block_view).collect::<Vec<_>>(),
        "agents": timeline.agents.iter().map(|agent| json!({
            "source": agent.source,
            "start": local_iso(agent.start_ms),
            "end": local_iso(agent.end_ms),
            "minutes": minutes(agent.end_ms - agent.start_ms),
            "title": agent.title,
            "branch": agent.branch,
            "prompts": agent.prompts,
            "issue": agent.issue,
            "commits": agent.commits,
            "private": agent.private,
        })).collect::<Vec<_>>(),
        "commits": timeline.commits.iter().map(|commit| json!({
            "at": local_iso(commit.at_ms),
            "subject": commit.subject,
            "repo": commit.repo,
            "branch": commit.branch,
            "hash": commit.hash,
        })).collect::<Vec<_>>(),
        "proposals": proposals.iter().map(proposal_view).collect::<Vec<_>>(),
    }))
}

/// Days with at least 5 minutes of the user's work that no tracker work
/// item or proposal covers: what is logged and proposed per issue, and the
/// spans and app blocks as evidence.
fn unlogged(store: &Store, from: &str, to: &str) -> Result<Value, String> {
    let mut git = GitHistories::default();
    let worklogs = store
        .tracker_worklogs(from, to)
        .map_err(|e| e.to_string())?;
    let mut days = Vec::new();
    for date in dates(from, to)? {
        let (from_ms, to_ms) = day_bounds(&date).map_err(|e| e.to_string())?;
        let timeline = day_timeline(store, &mut git, from_ms, to_ms, DEFAULT_SLOT_MS)
            .map_err(|e| e.to_string())?;
        let proposals = store
            .proposals_between(from_ms, to_ms)
            .map_err(|e| e.to_string())?;
        let ledger = DayLedger::new(&date, &timeline, &proposals, &worklogs);
        if ledger.unlogged_ms() < MIN_UNLOGGED_MS {
            continue;
        }
        days.push(json!({
            "date": date,
            "workMinutes": minutes(ledger.work_ms),
            "unloggedMinutes": minutes(ledger.unlogged_ms()),
            "logged": by_issue(&ledger.logged),
            "proposed": by_issue(&ledger.proposed),
            "work": timeline.work.iter().map(work_view).collect::<Vec<_>>(),
            "blocks": timeline
                .blocks
                .iter()
                .filter(|block| matches!(block, Block::Apps { .. }))
                .map(block_view)
                .collect::<Vec<_>>(),
        }));
    }
    Ok(json!({ "version": OUTPUT_VERSION, "from": from, "to": to, "days": days }))
}

fn by_issue(minutes_by_issue: &std::collections::BTreeMap<String, i64>) -> Value {
    minutes_by_issue
        .iter()
        .map(|(issue, ms)| json!({ "issue": issue, "minutes": minutes(*ms) }))
        .collect()
}

fn week_view(store: &Store, week: &str) -> Result<Value, String> {
    let summary =
        week_summary(store, &mut GitHistories::default(), week).map_err(|e| e.to_string())?;
    let dates: Vec<&str> = summary.days.iter().map(|day| day.date.as_str()).collect();
    Ok(json!({
        "version": OUTPUT_VERSION,
        "days": summary.days.iter().map(|day| json!({
            "date": day.date,
            "activeMinutes": minutes(day.active_ms),
            "loggedMinutes": minutes(day.logged_ms),
            "proposedMinutes": minutes(day.proposed_ms),
            "unloggedMinutes": minutes(day.unlogged_ms),
        })).collect::<Vec<_>>(),
        "issues": summary.issues.iter().map(|issue| json!({
            "issue": issue.issue,
            "days": issue.cells.iter().zip(&dates).filter_map(|(cell, date)| {
                let (state, minutes) = match cell {
                    WeekCell::Empty => return None,
                    WeekCell::Logged { minutes } => ("logged", minutes),
                    WeekCell::Proposed { minutes } => ("proposed", minutes),
                    WeekCell::Detected { minutes } => ("detected", minutes),
                };
                Some(json!({ "date": date, "state": state, "minutes": minutes }))
            }).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    }))
}

fn block_view(block: &Block) -> Value {
    match block {
        Block::Apps {
            start_ms,
            end_ms,
            dominant,
            title,
            items,
        } => json!({
            "kind": "apps",
            "start": local_iso(*start_ms),
            "end": local_iso(*end_ms),
            "minutes": minutes(items.iter().map(|item| item.duration_ms).sum()),
            "app": dominant.name,
            "title": title_text(title),
            "windows": items.iter().take(WINDOWS_PER_BLOCK).map(|item| json!({
                "app": item.app.name,
                "title": title_text(&item.title),
                "url": item.url,
                "minutes": minutes(item.duration_ms),
            })).collect::<Vec<_>>(),
        }),
        Block::Away { start_ms, end_ms } | Block::Locked { start_ms, end_ms } => json!({
            "kind": if matches!(block, Block::Away { .. }) { "away" } else { "locked" },
            "start": local_iso(*start_ms),
            "end": local_iso(*end_ms),
            "minutes": minutes(end_ms - start_ms),
        }),
    }
}

/// A work span for agents. `issue` is only Memorius's suggestion from the
/// evidence; the agent decides. Minutes are the user's work, with the agent
/// time that counted on its own.
fn work_view(span: &WorkSpan) -> Value {
    let TimeSplit {
        work_ms, agent_ms, ..
    } = span.split;
    json!({
        "issue": &span.issue,
        "support": span.support,
        "otherIssues": span.candidates,
        "start": local_iso(span.start_ms),
        "end": local_iso(span.end_ms),
        "minutes": minutes(work_ms),
        "agentMinutes": minutes(agent_ms),
        "evidence": span.evidence,
    })
}

fn proposal_view(proposal: &Proposal) -> Value {
    let work_item_id = match &proposal.state {
        ProposalState::Logged { work_item_id, .. } => Some(work_item_id.as_str()),
        ProposalState::Proposed
        | ProposalState::Approved { .. }
        | ProposalState::Rejected { .. } => None,
    };
    json!({
        "id": proposal.id,
        "issue": proposal.issue,
        "date": proposal.date,
        "start": local_iso(proposal.start_ms),
        "end": local_iso(proposal.end_ms),
        "minutes": proposal.minutes,
        "workType": proposal.work_type,
        "text": proposal.text,
        "agent": proposal.agent,
        "state": state_name(&proposal.state),
        "workItemId": work_item_id,
        "lastError": proposal.last_error,
        "proposedAt": local_iso(proposal.created_ms),
    })
}

fn state_name(state: &ProposalState) -> &'static str {
    match state {
        ProposalState::Proposed => "proposed",
        ProposalState::Approved { .. } => "approved",
        ProposalState::Rejected { .. } => "rejected",
        ProposalState::Logged { .. } => "logged",
    }
}

fn title_text(title: &Title) -> Option<&str> {
    match title {
        Title::Known { text } => Some(text),
        Title::Unavailable { .. } => None,
    }
}

fn minutes(ms: i64) -> i64 {
    ms / 60_000
}

/// Every date from `from` to `to`, both included, at most [`MAX_DAYS`].
fn dates(from: &str, to: &str) -> Result<Vec<String>, String> {
    let first: Date = from.parse().map_err(|e: jiff::Error| e.to_string())?;
    let last: Date = to.parse().map_err(|e: jiff::Error| e.to_string())?;
    let days = (last - first).get_days();
    if !(0..i32::try_from(MAX_DAYS).unwrap_or(i32::MAX)).contains(&days) {
        return Err(format!(
            "the range must go forward and cover at most {MAX_DAYS} days"
        ));
    }
    (0..=i64::from(days))
        .map(|offset| {
            first
                .checked_add(offset.days())
                .map(|day| day.to_string())
                .map_err(|e| e.to_string())
        })
        .collect()
}

/// This week's Monday, for default ranges.
pub fn week_start(today: &str) -> String {
    week_dates(today, WeekStart::Monday)
        .ok()
        .and_then(|dates| dates.into_iter().next())
        .unwrap_or_else(|| today.to_owned())
}
