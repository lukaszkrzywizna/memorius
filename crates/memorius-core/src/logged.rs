//! The Day view's YouTrack lane: what is logged or on its way, placed on
//! the day, and the user's work that nothing covers yet.
//!
//! Trackers keep a day and minutes, no time of day. A proposal knows its
//! range, so it sits there; a work item logged anywhere else sits where
//! Memorius saw work on its issue, then in the nearest uncovered work.

use std::collections::HashSet;

use serde::Serialize;
use ts_rs::TS;

use crate::proposal::{Proposal, ProposalState};
use crate::timeline::{DayTimeline, union};
use crate::tracker::TrackerWorklog;
use crate::week::MIN_UNLOGGED_MS;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LoggedLane {
    /// Oldest first; one worklog may take several pieces around breaks.
    pub blocks: Vec<LoggedBlock>,
    /// The user's work, 5 minutes or longer, that no block covers.
    pub gaps: Vec<Gap>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum LoggedState {
    Logged,
    Approved,
    Proposed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LoggedBlock {
    #[ts(type = "number")]
    pub start_ms: i64,
    #[ts(type = "number")]
    pub end_ms: i64,
    /// The same for every piece of one worklog or proposal.
    pub key: String,
    pub issue: String,
    pub summary: Option<String>,
    /// The whole worklog's minutes, not this piece's.
    pub minutes: u32,
    pub work_type: Option<String>,
    pub text: Option<String>,
    pub state: LoggedState,
    /// The tracker gave no time of day: Memorius chose where it sits.
    pub estimated: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Gap {
    #[ts(type = "number")]
    pub start_ms: i64,
    #[ts(type = "number")]
    pub end_ms: i64,
}

type Range = (i64, i64);

/// The lane for `date` (`YYYY-MM-DD`, the day of `timeline`); `proposals`
/// and `worklogs` may hold other days.
pub fn logged_lane(
    date: &str,
    timeline: &DayTimeline,
    proposals: &[Proposal],
    worklogs: &[TrackerWorklog],
) -> LoggedLane {
    let day = [(timeline.from_ms, timeline.to_ms)];
    let today: Vec<&TrackerWorklog> = worklogs.iter().filter(|w| w.date == date).collect();
    let mut placed: Vec<Range> = Vec::new();
    let mut blocks = Vec::new();
    let synced = place_proposals(date, timeline, proposals, &today, &mut placed, &mut blocks);

    // Work items with detected work on their issue go first, so they get it.
    let mut rest: Vec<&TrackerWorklog> = today
        .into_iter()
        .filter(|w| !synced.contains(w.id.as_str()))
        .collect();
    let spans = |issue: &str| -> Vec<Range> {
        union(
            timeline
                .work
                .iter()
                .filter(|span| span.issue == issue)
                .map(|span| (span.start_ms, span.end_ms))
                .collect(),
        )
    };
    rest.sort_by_key(|w| (spans(&w.issue).is_empty(), std::cmp::Reverse(w.minutes)));
    for item in rest {
        let own = spans(&item.issue);
        let anchor = own.first().map_or(timeline.from_ms, |range| range.0);
        let mut near = timeline.activity.clone();
        near.sort_by_key(|range| distance(*range, anchor));
        // More logged than worked: after everything else.
        let after = placed
            .last()
            .map_or(anchor, |range| range.1)
            .max(timeline.activity.last().map_or(anchor, |range| range.1));
        let pieces = take(
            &mut placed,
            &[
                intersect(&timeline.activity, &own),
                near,
                vec![(after, timeline.to_ms)],
                day.to_vec(),
            ],
            minutes_ms(item.minutes),
        );
        blocks.extend(pieces.into_iter().map(|(start_ms, end_ms)| LoggedBlock {
            start_ms,
            end_ms,
            key: item.id.clone(),
            issue: item.issue.clone(),
            summary: item.summary.clone(),
            minutes: item.minutes,
            work_type: item.work_type.clone(),
            text: item.text.clone(),
            state: LoggedState::Logged,
            estimated: true,
        }));
    }

    blocks.sort_by_key(|block| block.start_ms);
    let gaps = subtract(&timeline.activity, &placed)
        .into_iter()
        .filter(|(start, end)| end - start >= MIN_UNLOGGED_MS)
        .map(|(start_ms, end_ms)| Gap { start_ms, end_ms })
        .collect();
    LoggedLane { blocks, gaps }
}

/// Proposals sit in their own range; returns the work items they logged.
fn place_proposals<'a>(
    date: &str,
    timeline: &DayTimeline,
    proposals: &[Proposal],
    today: &[&'a TrackerWorklog],
    placed: &mut Vec<Range>,
    blocks: &mut Vec<LoggedBlock>,
) -> HashSet<&'a str> {
    let mut synced: HashSet<&'a str> = HashSet::new();
    let mut proposals: Vec<&Proposal> = proposals.iter().filter(|p| p.date == date).collect();
    proposals.sort_by_key(|p| p.start_ms);
    for proposal in proposals {
        let state = match &proposal.state {
            ProposalState::Rejected { .. } => continue,
            ProposalState::Proposed => LoggedState::Proposed,
            ProposalState::Approved { .. } => LoggedState::Approved,
            ProposalState::Logged { work_item_id, .. } => {
                // The same work item counts once, as in the week ledger.
                let item = today.iter().find(|w| w.id == *work_item_id).or_else(|| {
                    today.iter().find(|w| {
                        !synced.contains(w.id.as_str())
                            && w.issue == proposal.issue
                            && w.minutes == proposal.minutes
                    })
                });
                if let Some(item) = item {
                    synced.insert(&item.id);
                }
                LoggedState::Logged
            }
        };
        let range = [(proposal.start_ms, proposal.end_ms)];
        let pieces = take(
            placed,
            &[intersect(&timeline.activity, &range), range.to_vec()],
            minutes_ms(proposal.minutes),
        );
        let summary = today
            .iter()
            .find(|w| w.issue == proposal.issue)
            .and_then(|w| w.summary.clone());
        blocks.extend(pieces.into_iter().map(|(start_ms, end_ms)| LoggedBlock {
            start_ms,
            end_ms,
            key: format!("proposal-{}", proposal.id),
            issue: proposal.issue.clone(),
            summary: summary.clone(),
            minutes: proposal.minutes,
            work_type: proposal.work_type.clone(),
            text: Some(proposal.text.clone()),
            state,
            estimated: false,
        }));
    }
    synced
}

fn minutes_ms(minutes: u32) -> i64 {
    i64::from(minutes) * 60_000
}

/// Takes `ms` of room that `placed` does not hold yet: from the first
/// choice of ranges, in its order, then the next. The pieces join `placed`.
fn take(placed: &mut Vec<Range>, choices: &[Vec<Range>], mut ms: i64) -> Vec<Range> {
    let mut pieces = Vec::new();
    for choice in choices {
        for range in choice {
            if ms <= 0 {
                break;
            }
            for (start, end) in subtract(&[*range], placed) {
                let end = end.min(start + ms);
                ms -= end - start;
                pieces.push((start, end));
                *placed = union([placed.as_slice(), &[(start, end)]].concat());
                if ms <= 0 {
                    break;
                }
            }
        }
    }
    union(pieces)
}

fn distance((start, end): Range, at: i64) -> i64 {
    if at < start {
        start - at
    } else if at > end {
        at - end
    } else {
        0
    }
}

/// Sorted disjoint `a` ∩ `b`.
fn intersect(a: &[Range], b: &[Range]) -> Vec<Range> {
    let mut out = Vec::new();
    for &(a0, a1) in a {
        for &(b0, b1) in b {
            let (start, end) = (a0.max(b0), a1.min(b1));
            if start < end {
                out.push((start, end));
            }
        }
    }
    union(out)
}

/// `ranges` without what sorted disjoint `cut` covers.
fn subtract(ranges: &[Range], cut: &[Range]) -> Vec<Range> {
    let mut out = Vec::new();
    for &(mut start, end) in ranges {
        for &(c0, c1) in cut {
            if c1 <= start || c0 >= end {
                continue;
            }
            if c0 > start {
                out.push((start, c0));
            }
            start = start.max(c1);
        }
        if start < end {
            out.push((start, end));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{Confidence, Evidence};
    use crate::timeline::{TimeSplit, Totals, WorkSpan};

    const MIN: i64 = 60_000;

    fn day(activity: Vec<Range>, work: Vec<WorkSpan>) -> DayTimeline {
        DayTimeline {
            from_ms: 0,
            to_ms: 24 * 60 * MIN,
            blocks: Vec::new(),
            agents: Vec::new(),
            work,
            commits: Vec::new(),
            totals: Totals {
                active_ms: 0,
                away_ms: 0,
                agent_ms: 0,
                split: TimeSplit::default(),
            },
            activity,
            logged: LoggedLane::default(),
        }
    }

    fn span(from: i64, to: i64, issue: &str) -> WorkSpan {
        WorkSpan {
            start_ms: from * MIN,
            end_ms: to * MIN,
            issue: issue.into(),
            confidence: Confidence::High,
            support: 100,
            candidates: Vec::new(),
            evidence: Vec::<Evidence>::new(),
            split: TimeSplit::default(),
        }
    }

    fn item(id: &str, issue: &str, minutes: u32) -> TrackerWorklog {
        TrackerWorklog {
            id: id.into(),
            issue: issue.into(),
            summary: None,
            date: "2026-11-27".into(),
            minutes,
            text: None,
            work_type: None,
        }
    }

    fn proposal(id: i64, from: i64, to: i64, minutes: u32, state: ProposalState) -> Proposal {
        Proposal {
            id,
            issue: "SEL-1".into(),
            date: "2026-11-27".into(),
            start_ms: from * MIN,
            end_ms: to * MIN,
            minutes,
            work_type: None,
            text: "Work".into(),
            agent: "Claude Code".into(),
            created_ms: 0,
            state,
            last_error: None,
            revision: 0,
        }
    }

    fn ranges(lane: &LoggedLane) -> Vec<(String, i64, i64, bool)> {
        lane.blocks
            .iter()
            .map(|b| (b.key.clone(), b.start_ms / MIN, b.end_ms / MIN, b.estimated))
            .collect()
    }

    #[test]
    fn a_work_item_without_a_time_sits_on_its_issue_and_the_rest_is_a_gap() {
        // Work 9:00–12:00: INT-1 detected 9:00–10:00, 60 minutes logged.
        let timeline = day(vec![(540 * MIN, 720 * MIN)], vec![span(540, 600, "INT-1")]);
        let lane = logged_lane("2026-11-27", &timeline, &[], &[item("w1", "INT-1", 60)]);
        assert_eq!(ranges(&lane), vec![("w1".into(), 540, 600, true)]);
        assert_eq!(
            lane.gaps,
            vec![Gap {
                start_ms: 600 * MIN,
                end_ms: 720 * MIN
            }]
        );
    }

    #[test]
    fn a_proposal_sits_in_its_range_and_its_work_item_counts_once() {
        let timeline = day(vec![(540 * MIN, 660 * MIN)], Vec::new());
        let logged = ProposalState::Logged {
            at_ms: 0,
            work_item_id: "w1".into(),
        };
        let lane = logged_lane(
            "2026-11-27",
            &timeline,
            &[
                proposal(1, 540, 600, 30, logged),
                proposal(2, 600, 660, 60, ProposalState::Proposed),
                proposal(3, 600, 660, 60, ProposalState::Rejected { at_ms: 0 }),
            ],
            &[item("w1", "SEL-1", 30)],
        );
        assert_eq!(
            ranges(&lane),
            vec![
                ("proposal-1".into(), 540, 570, false),
                ("proposal-2".into(), 600, 660, false),
            ]
        );
        assert_eq!(lane.blocks[1].state, LoggedState::Proposed);
        assert_eq!(
            lane.gaps,
            vec![Gap {
                start_ms: 570 * MIN,
                end_ms: 600 * MIN
            }]
        );
    }

    #[test]
    fn more_logged_than_worked_goes_after_the_work_and_short_leftovers_are_no_gap() {
        // Work 9:00–9:33 and 10:00–10:30; 60 minutes on an issue never seen.
        let timeline = day(
            vec![(540 * MIN, 573 * MIN), (600 * MIN, 630 * MIN)],
            Vec::new(),
        );
        let lane = logged_lane("2026-11-27", &timeline, &[], &[item("w1", "X-1", 60)]);
        assert_eq!(
            ranges(&lane),
            vec![("w1".into(), 540, 573, true), ("w1".into(), 600, 627, true),]
        );
        assert!(lane.gaps.is_empty(), "3 minutes is no gap");
        let lane = logged_lane("2026-11-27", &timeline, &[], &[item("w1", "X-1", 70)]);
        assert_eq!(
            lane.blocks
                .last()
                .map(|b| (b.start_ms / MIN, b.end_ms / MIN)),
            Some((600, 637))
        );
    }
}
