//! What is logged, proposed, detected and not logged, per issue and day:
//! trackers log an issue on a day, with no start or end time.

use std::collections::{BTreeMap, HashSet};

use serde::Serialize;
use ts_rs::TS;

use crate::context::Evidence;
use crate::model::Rule;
use crate::proposal::{Proposal, ProposalState};
use crate::timeline::{DayTimeline, overlap_ms};
use crate::tracker::TrackerWorklog;

/// Leftovers shorter than this are not worth a worklog.
pub const MIN_UNLOGGED_MS: i64 = 5 * 60_000;

/// One day's time per issue: logged in the tracker (or by an agent and not
/// synced yet), proposed and waiting, and detected in activity (the issue
/// is a suggestion).
#[derive(Default)]
pub struct DayLedger {
    /// The user's work that day, as counted on the Day view.
    pub work_ms: i64,
    pub logged: BTreeMap<String, i64>,
    pub proposed: BTreeMap<String, i64>,
    pub detected: BTreeMap<String, i64>,
}

impl DayLedger {
    /// `proposals` and `worklogs` may hold other days; only `date` counts.
    pub fn new(
        date: &str,
        timeline: &DayTimeline,
        proposals: &[Proposal],
        worklogs: &[TrackerWorklog],
    ) -> Self {
        let mut ledger = Self {
            work_ms: timeline.totals.split.work_ms,
            ..Self::default()
        };
        let synced: HashSet<&str> = worklogs.iter().map(|item| item.id.as_str()).collect();
        let today: Vec<&TrackerWorklog> =
            worklogs.iter().filter(|item| item.date == date).collect();
        for item in &today {
            *ledger.logged.entry(item.issue.clone()).or_default() +=
                i64::from(item.minutes) * 60_000;
        }
        for proposal in proposals.iter().filter(|p| p.date == date) {
            let ms = i64::from(proposal.minutes) * 60_000;
            match &proposal.state {
                ProposalState::Rejected { .. } => {}
                // A logged proposal counts once: from the tracker after a sync
                // (by its work item id, or by issue and minutes when an agent
                // reported no id).
                ProposalState::Logged { work_item_id, .. } => {
                    let in_tracker = synced.contains(work_item_id.as_str())
                        || today.iter().any(|item| {
                            item.issue == proposal.issue && item.minutes == proposal.minutes
                        });
                    if !in_tracker {
                        *ledger.logged.entry(proposal.issue.clone()).or_default() += ms;
                    }
                }
                ProposalState::Proposed | ProposalState::Approved { .. } => {
                    *ledger.proposed.entry(proposal.issue.clone()).or_default() += ms;
                }
            }
        }
        for span in &timeline.work {
            *ledger.detected.entry(span.issue.clone()).or_default() += span.split.work_ms;
        }
        ledger
    }

    pub fn logged_ms(&self) -> i64 {
        self.logged.values().sum()
    }

    pub fn proposed_ms(&self) -> i64 {
        self.proposed.values().sum()
    }

    /// The user's work that nothing logged or proposed covers yet.
    pub fn unlogged_ms(&self) -> i64 {
        (self.work_ms - self.logged_ms() - self.proposed_ms()).max(0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum WeekCell {
    Empty,
    Logged {
        minutes: u32,
    },
    /// Proposed or approved, not logged yet.
    Proposed {
        minutes: u32,
    },
    /// Detected in activity (a suggested issue), with nothing logged or proposed.
    Detected {
        minutes: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct IssueWeek {
    pub issue: String,
    pub cells: Vec<WeekCell>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WeekDay {
    pub date: String,
    /// The user's work, as on the Day view.
    #[ts(type = "number")]
    pub active_ms: i64,
    #[ts(type = "number")]
    pub logged_ms: i64,
    #[ts(type = "number")]
    pub proposed_ms: i64,
    /// The user's work that no tracker work item or proposal covers.
    #[ts(type = "number")]
    pub unlogged_ms: i64,
    /// Active time in each hour from local midnight (23 to 25 hours when
    /// the clocks change), for the day's sparkline.
    #[ts(type = "Array<number>")]
    pub hourly_active_ms: Vec<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WeekSummary {
    pub days: Vec<WeekDay>,
    pub issues: Vec<IssueWeek>,
}

/// Milliseconds per state for one issue on one day.
#[derive(Default, Clone, Copy)]
struct Tally {
    logged: i64,
    proposed: i64,
    detected: i64,
}

/// One day's totals: the user's work, what YouTrack holds, what waits as
/// proposals, and what nothing covers yet.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DaySummary {
    pub date: String,
    #[ts(type = "number")]
    pub work_ms: i64,
    #[ts(type = "number")]
    pub logged_ms: i64,
    #[ts(type = "number")]
    pub proposed_ms: i64,
    #[ts(type = "number")]
    pub unlogged_ms: i64,
}

impl DayLedger {
    pub fn summary(&self, date: &str) -> DaySummary {
        DaySummary {
            date: date.to_owned(),
            work_ms: self.work_ms,
            logged_ms: self.logged_ms(),
            proposed_ms: self.proposed_ms(),
            unlogged_ms: self.unlogged_ms(),
        }
    }
}

/// One summary over consecutive days; `days` holds each date with its
/// timeline, `proposals` and `worklogs` everything in the week.
pub fn summarize(
    days: &[(String, DayTimeline)],
    proposals: &[Proposal],
    worklogs: &[TrackerWorklog],
) -> WeekSummary {
    let mut tallies: BTreeMap<String, Vec<Tally>> = BTreeMap::new();
    let mut week_days = Vec::with_capacity(days.len());
    for (index, (date, timeline)) in days.iter().enumerate() {
        let (day_start, day_end) = day_span(timeline);
        let ledger = DayLedger::new(date, timeline, proposals, worklogs);
        let states = [
            (&ledger.logged, 0usize),
            (&ledger.proposed, 1),
            (&ledger.detected, 2),
        ];
        for (by_issue, state) in states {
            for (issue, ms) in by_issue {
                let tally = &mut tallies
                    .entry(issue.clone())
                    .or_insert_with(|| vec![Tally::default(); days.len()])[index];
                match state {
                    0 => tally.logged += ms,
                    1 => tally.proposed += ms,
                    _ => tally.detected += ms,
                }
            }
        }
        week_days.push(WeekDay {
            date: date.clone(),
            active_ms: ledger.work_ms,
            logged_ms: ledger.logged_ms(),
            proposed_ms: ledger.proposed_ms(),
            unlogged_ms: ledger.unlogged_ms(),
            hourly_active_ms: hourly(&timeline.activity, day_start, day_end),
        });
    }
    let issues = tallies
        .into_iter()
        .map(|(issue, tallies)| IssueWeek {
            issue,
            cells: tallies.into_iter().map(cell).collect(),
        })
        .filter(|week| week.cells.iter().any(|cell| *cell != WeekCell::Empty))
        .collect();
    WeekSummary {
        days: week_days,
        issues,
    }
}

/// How much time one rule named in a stretch of days.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RuleMatch {
    #[ts(type = "number")]
    pub id: i64,
    #[ts(type = "number")]
    pub matched_ms: i64,
}

/// Time of the work spans each rule named across `days`, for the rules list.
pub fn rule_matches(rules: &[Rule], days: &[DayTimeline]) -> Vec<RuleMatch> {
    rules
        .iter()
        .map(|rule| RuleMatch {
            id: rule.id,
            matched_ms: days
                .iter()
                .flat_map(|day| &day.work)
                .filter(|span| {
                    span.evidence
                        .iter()
                        .any(|evidence| matches!(evidence, Evidence::Rule { rule: named } if *named == rule.rule))
                })
                .map(|span| span.end_ms - span.start_ms)
                .sum(),
        })
        .collect()
}

const HOUR_MS: i64 = 3_600_000;

/// Active time per hour of `[day_start, day_end)`.
fn hourly(activity: &[(i64, i64)], day_start: i64, day_end: i64) -> Vec<i64> {
    let mut hours = Vec::new();
    let mut from = day_start;
    while from < day_end {
        let to = (from + HOUR_MS).min(day_end);
        hours.push(overlap_ms(activity, from, to));
        from = to;
    }
    hours
}

fn day_span(timeline: &DayTimeline) -> (i64, i64) {
    (timeline.from_ms, timeline.to_ms)
}

fn minutes(ms: i64) -> u32 {
    u32::try_from(ms / 60_000).unwrap_or(u32::MAX)
}

fn cell(tally: Tally) -> WeekCell {
    if tally.logged > 0 {
        WeekCell::Logged {
            minutes: minutes(tally.logged),
        }
    } else if tally.proposed > 0 {
        WeekCell::Proposed {
            minutes: minutes(tally.proposed),
        }
    } else if tally.detected >= MIN_UNLOGGED_MS {
        WeekCell::Detected {
            minutes: minutes(tally.detected),
        }
    } else {
        WeekCell::Empty
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AppRef, Title};
    use crate::timeline::{Block, TimeSplit, Totals, WorkSpan};

    const MIN: i64 = 60_000;
    const DAY: i64 = 24 * 60 * MIN;

    fn proposal(issue: &str, start: i64, end: i64, state: ProposalState) -> Proposal {
        Proposal {
            id: 1,
            issue: issue.into(),
            date: if start < 1440 {
                "2026-10-05"
            } else {
                "2026-10-06"
            }
            .into(),
            start_ms: start * MIN,
            end_ms: end * MIN,
            minutes: u32::try_from(end - start).unwrap(),
            work_type: None,
            text: "x".into(),
            agent: "Codex".into(),
            created_ms: 0,
            state,
            last_error: None,
            revision: 1,
        }
    }

    fn worklog(id: &str, issue: &str, date: &str, minutes: u32) -> TrackerWorklog {
        TrackerWorklog {
            id: id.into(),
            issue: issue.into(),
            summary: None,
            date: date.into(),
            minutes,
            text: None,
            work_type: None,
        }
    }

    fn block(start: i64, end: i64) -> Block {
        Block::Apps {
            start_ms: start * MIN,
            end_ms: end * MIN,
            dominant: AppRef {
                id: "rider".into(),
                name: "Rider".into(),
            },
            title: Title::Known {
                text: "a.fs".into(),
            },
            items: Vec::new(),
        }
    }

    fn work(start: i64, end: i64, issue: &str) -> WorkSpan {
        WorkSpan {
            start_ms: start * MIN,
            end_ms: end * MIN,
            issue: issue.into(),
            confidence: crate::context::Confidence::High,
            evidence: Vec::new(),
            support: 100,
            candidates: Vec::new(),
            split: TimeSplit {
                work_ms: (end - start) * MIN,
                ..TimeSplit::default()
            },
        }
    }

    #[test]
    fn a_rule_counts_the_spans_it_named_on_every_day() {
        use crate::model::RuleKind;
        let host = RuleKind::HostIssue {
            host: "orders.example.com".into(),
            issue: "SEL-54".into(),
        };
        let other = RuleKind::TitleIssue {
            text: "Daily stand-up".into(),
            issue: "SEL-21".into(),
        };
        let mut named = work(540, 600, "SEL-54");
        named.evidence = vec![Evidence::Rule { rule: host.clone() }];
        let day = |work: Vec<WorkSpan>| DayTimeline {
            from_ms: 0,
            to_ms: DAY,
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
            activity: Vec::new(),
        };
        let rules = [Rule { id: 1, rule: host }, Rule { id: 2, rule: other }];

        let matches = rule_matches(
            &rules,
            &[
                day(vec![named.clone(), work(600, 660, "IS-61")]),
                day(vec![named]),
            ],
        );

        assert_eq!(
            matches,
            vec![
                RuleMatch {
                    id: 1,
                    matched_ms: 120 * MIN
                },
                RuleMatch {
                    id: 2,
                    matched_ms: 0
                },
            ]
        );
    }

    #[test]
    fn hours_split_active_time_from_midnight_and_follow_short_days() {
        let activity = [(9 * 60 * MIN + 30 * MIN, 11 * 60 * MIN)];

        let full = hourly(&activity, 0, DAY);
        assert_eq!(full.len(), 24);
        assert_eq!(&full[9..12], &[30 * MIN, 60 * MIN, 0]);
        assert_eq!(full.iter().sum::<i64>(), 90 * MIN);

        // A 23-hour day when the clocks go forward.
        assert_eq!(hourly(&activity, 0, DAY - 60 * MIN).len(), 23);
    }

    #[test]
    fn the_week_marks_each_issue_logged_proposed_or_detected() {
        let monday = DayTimeline {
            from_ms: 0,
            to_ms: DAY,
            blocks: vec![block(540, 660)],
            agents: Vec::new(),
            work: vec![work(540, 600, "INT-290"), work(600, 660, "IS-61")],
            commits: Vec::new(),
            totals: Totals {
                active_ms: 120 * MIN,
                away_ms: 0,
                agent_ms: 0,
                split: TimeSplit {
                    work_ms: 120 * MIN,
                    ..TimeSplit::default()
                },
            },
            activity: vec![(540 * MIN, 660 * MIN)],
        };
        let tuesday = DayTimeline {
            from_ms: DAY,
            to_ms: 2 * DAY,
            blocks: vec![block(1440 + 540, 1440 + 560)],
            agents: Vec::new(),
            work: vec![work(1440 + 540, 1440 + 560, "IS-61")],
            commits: Vec::new(),
            totals: Totals {
                active_ms: 20 * MIN,
                away_ms: 0,
                agent_ms: 0,
                split: TimeSplit {
                    work_ms: 20 * MIN,
                    ..TimeSplit::default()
                },
            },
            activity: vec![((1440 + 540) * MIN, (1440 + 560) * MIN)],
        };
        let proposals = [
            proposal(
                "INT-290",
                540,
                600,
                ProposalState::Logged {
                    at_ms: 0,
                    work_item_id: "2-1".into(),
                },
            ),
            proposal("IS-61", 1440 + 540, 1440 + 560, ProposalState::Proposed),
        ];

        let week = summarize(
            &[
                ("2026-10-05".into(), monday),
                ("2026-10-06".into(), tuesday),
            ],
            &proposals,
            &[worklog("9-1", "SEL-21", "2026-10-06", 10)],
        );

        let cells: Vec<(String, Vec<WeekCell>)> = week
            .issues
            .into_iter()
            .map(|issue| (issue.issue, issue.cells))
            .collect();
        assert_eq!(
            cells,
            [
                (
                    "INT-290".to_owned(),
                    vec![WeekCell::Logged { minutes: 60 }, WeekCell::Empty]
                ),
                (
                    "IS-61".to_owned(),
                    vec![
                        WeekCell::Detected { minutes: 60 },
                        WeekCell::Proposed { minutes: 20 }
                    ]
                ),
                (
                    "SEL-21".to_owned(),
                    vec![WeekCell::Empty, WeekCell::Logged { minutes: 10 }]
                ),
            ]
        );
        assert_eq!(
            (week.days[0].logged_ms, week.days[0].unlogged_ms),
            (60 * MIN, 60 * MIN)
        );
        // Tuesday: 20 minutes of work, 20 proposed and 10 logged by hand.
        assert_eq!(
            (
                week.days[1].proposed_ms,
                week.days[1].logged_ms,
                week.days[1].unlogged_ms
            ),
            (20 * MIN, 10 * MIN, 0)
        );
    }

    #[test]
    fn a_logged_proposal_without_a_work_item_id_counts_once() {
        let day = DayTimeline {
            from_ms: 0,
            to_ms: DAY,
            blocks: Vec::new(),
            agents: Vec::new(),
            work: Vec::new(),
            commits: Vec::new(),
            totals: Totals {
                active_ms: 0,
                away_ms: 0,
                agent_ms: 0,
                split: TimeSplit {
                    work_ms: 120 * MIN,
                    ..TimeSplit::default()
                },
            },
            activity: Vec::new(),
        };
        let logged = proposal(
            "SEL-82",
            540,
            600,
            ProposalState::Logged {
                at_ms: 0,
                work_item_id: String::new(),
            },
        );
        let synced = worklog("196-2997", "SEL-82", "2026-10-05", 60);

        let ledger = DayLedger::new("2026-10-05", &day, &[logged], &[synced]);

        assert_eq!(
            (ledger.logged_ms(), ledger.unlogged_ms()),
            (60 * MIN, 60 * MIN)
        );
    }
}
