use std::collections::{BTreeMap, HashMap, HashSet};

use serde::Serialize;
use ts_rs::TS;

use crate::context::{
    Confidence, Evidence, MAX_EVIDENCE, SessionContext, WindowTime, confidence, detect_work,
    session_end_ms,
};
use crate::model::{
    AgentSource, AppRef, AppUsage, ContextSpan, RuleKind, SessionRef, Title, Window,
};
use crate::store::{ActivityRow, RowKind};
use crate::time_rules::{TimeRules, minutes_ms};

/// The day is cut into slots. Each slot shows its dominant
/// window; neighbour slots with the same one form a block. The UI's zoom
/// picks the slot length; agents and the week view use the default.
pub const DEFAULT_SLOT_MS: i64 = 15 * 60 * 1_000;

/// Slot lengths the zoom offers, in minutes.
pub const ZOOM_MINUTES: [u32; 6] = [1, 5, 10, 15, 30, 60];

/// The slot length for a zoom level, or `None` when the zoom is not offered.
pub fn slot_ms(zoom_minutes: u32) -> Option<i64> {
    ZOOM_MINUTES
        .contains(&zoom_minutes)
        .then(|| i64::from(zoom_minutes) * 60_000)
}

/// Offline time weighs less than app time when a slot picks its dominant
/// activity. Idle time starts at the last input, so it already holds the
/// minutes before Memorius noticed the idle.
const APP_WEIGHT: i64 = 3;
const OFFLINE_WEIGHT: i64 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DayTimeline {
    /// Local midnight that starts the day, and the next one.
    #[ts(type = "number")]
    pub from_ms: i64,
    #[ts(type = "number")]
    pub to_ms: i64,
    pub blocks: Vec<Block>,
    /// When coding agents worked, apart from the user's own app time.
    pub agents: Vec<AgentBlock>,
    /// What the time was about, from app time and agent time.
    pub work: Vec<WorkSpan>,
    /// The user's own git commits that day, oldest first.
    pub commits: Vec<DayCommit>,
    pub totals: Totals,
    /// The user's work as sorted disjoint `[start, end)` ranges, as counted
    /// in [`TimeSplit::work_ms`]. Worklogs cover it.
    #[serde(skip)]
    pub activity: Vec<(i64, i64)>,
}

/// One of the user's own git commits on this Mac, for the Commits lane.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DayCommit {
    pub hash: String,
    #[ts(type = "number")]
    pub at_ms: i64,
    pub subject: String,
    /// The repository's folder name, for example `Shop`.
    pub repo: String,
    /// The branch checked out when the commit was made.
    pub branch: Option<String>,
}

/// How the time in a range splits by the user's part. It is measured on
/// exact ranges, so the display zoom never changes it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TimeSplit {
    /// The user's work, counted once: app time on the Mac, each prompt's
    /// minute (also from a phone), breaks of up to 15 minutes between them,
    /// and agent time up to 30 minutes after the session's last prompt.
    #[ts(type = "number")]
    pub work_ms: i64,
    /// The part of `work_ms` that is agent time while the user was away.
    #[ts(type = "number")]
    pub agent_ms: i64,
    /// Agent time that does not count: later than 30 minutes after a
    /// prompt, a session with no prompt (heartbeats), or a private session.
    #[ts(type = "number")]
    pub uncounted_ms: i64,
}

/// Another issue that named part of a work span.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct IssueSupport {
    pub issue: String,
    /// Evidence support, 0–100, as in [`WorkSpan::support`].
    pub support: u8,
}

/// Everything one day's view is built from.
pub struct DayInputs<'a> {
    pub rows: &'a [ActivityRow],
    pub from_ms: i64,
    pub to_ms: i64,
    pub slot_ms: i64,
    pub sessions: &'a [SessionContext],
    pub rules: &'a [RuleKind],
    /// Sessions the user marked private: their prompts and agent time
    /// never count, and they name no issue.
    pub private: &'a [SessionRef],
    pub time: TimeRules,
    /// Project keys the tracker knows; empty: any key counts.
    pub projects: &'a [String],
}

/// One agent session's work, on the slot grid. Sessions run in parallel, so
/// agent blocks can overlap.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AgentBlock {
    #[ts(type = "number")]
    pub start_ms: i64,
    #[ts(type = "number")]
    pub end_ms: i64,
    pub source: AgentSource,
    pub title: Option<String>,
    pub branch: Option<String>,
    /// Messages the user sent in this block.
    pub prompts: u32,
    /// When each of those messages was sent: sorted, without duplicates,
    /// within `[start_ms, end_ms)`. Times only, never the text.
    #[ts(type = "Array<number>")]
    pub prompt_times_ms: Vec<i64>,
    pub issue: Option<String>,
    /// Commit subjects made in the session's folder during the block.
    pub commits: Vec<String>,
    /// This session's own part of the block: its counted and uncounted
    /// agent time; `work_ms` is all of the user's work in the range.
    pub split: TimeSplit,
    /// The user marked the block's sessions private.
    pub private: bool,
    /// The logged sessions behind the block (a resumed conversation has
    /// several); answers about the block go to these.
    pub sessions: Vec<SessionRef>,
}

/// A stretch of slots about one likely issue, shown in the Work context
/// lane.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WorkSpan {
    #[ts(type = "number")]
    pub start_ms: i64,
    #[ts(type = "number")]
    pub end_ms: i64,
    /// The most likely issue; agents may propose it only when `confidence`
    /// is high.
    pub issue: String,
    pub confidence: Confidence,
    /// Evidence support for `issue`, 0–100: its share of the span's
    /// detection weight (named or not). It is not a share of minutes; the
    /// minutes are in `split`.
    pub support: u8,
    /// Other issues named in the span, most support first.
    pub candidates: Vec<IssueSupport>,
    pub evidence: Vec<Evidence>,
    pub split: TimeSplit,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Totals {
    #[ts(type = "number")]
    pub active_ms: i64,
    #[ts(type = "number")]
    pub away_ms: i64,
    /// Time any agent session worked; parallel sessions count once.
    #[ts(type = "number")]
    pub agent_ms: i64,
    pub split: TimeSplit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum Block {
    #[serde(rename_all = "camelCase")]
    Apps {
        #[ts(type = "number")]
        start_ms: i64,
        #[ts(type = "number")]
        end_ms: i64,
        /// The app of the dominant window, which names the block.
        dominant: AppRef,
        /// The dominant window's title.
        title: Title,
        /// Windows in the block, longest first.
        items: Vec<BlockItem>,
    },
    #[serde(rename_all = "camelCase")]
    Away {
        #[ts(type = "number")]
        start_ms: i64,
        #[ts(type = "number")]
        end_ms: i64,
    },
    #[serde(rename_all = "camelCase")]
    Locked {
        #[ts(type = "number")]
        start_ms: i64,
        #[ts(type = "number")]
        end_ms: i64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BlockItem {
    pub app: AppRef,
    pub title: Title,
    /// Browser address, when the window was a browser tab.
    pub url: Option<String>,
    #[ts(type = "number")]
    pub duration_ms: i64,
}

/// What one slot holds.
#[derive(Default)]
struct Slot {
    pieces: Vec<(Window, i64)>,
    away_ms: i64,
    locked_ms: i64,
}

/// The dominant activity of a slot; equal neighbours merge.
#[derive(PartialEq)]
enum Dominant {
    Window { app: AppRef, title: Title },
    Offline,
}

/// Groups stored rows into the blocks shown on the day timeline.
pub fn group(rows: &[ActivityRow], from_ms: i64, to_ms: i64, slot_ms: i64) -> DayTimeline {
    let mut totals = Totals::default();
    for row in rows {
        let ms = row.end_ms.min(to_ms) - row.start_ms.max(from_ms);
        if ms > 0 {
            match row.kind {
                RowKind::App(_) => totals.active_ms += ms,
                RowKind::Away | RowKind::Locked => totals.away_ms += ms,
            }
        }
    }

    let slots = slots(rows, from_ms, to_ms, slot_ms);
    let mut runs: Vec<(i64, i64, Dominant, Slot)> = Vec::new();
    for (index, slot) in slots {
        let dominant = dominant(&slot);
        match runs.last_mut() {
            Some((_, last, current, merged)) if *last + 1 == index && *current == dominant => {
                *last = index;
                merged.pieces.extend(slot.pieces);
                merged.away_ms += slot.away_ms;
                merged.locked_ms += slot.locked_ms;
            }
            _ => runs.push((index, index, dominant, slot)),
        }
    }

    let blocks = runs
        .into_iter()
        .map(|(first, last, dominant, slot)| {
            let start_ms = from_ms + first * slot_ms;
            let end_ms = (from_ms + (last + 1) * slot_ms).min(to_ms);
            match dominant {
                Dominant::Offline if slot.locked_ms > slot.away_ms => {
                    Block::Locked { start_ms, end_ms }
                }
                Dominant::Offline => Block::Away { start_ms, end_ms },
                Dominant::Window { app, title } => Block::Apps {
                    start_ms,
                    end_ms,
                    dominant: app,
                    title,
                    items: items(slot.pieces),
                },
            }
        })
        .collect();

    DayTimeline {
        from_ms,
        to_ms,
        blocks,
        agents: Vec::new(),
        work: Vec::new(),
        commits: Vec::new(),
        totals,
        activity: Vec::new(),
    }
}

/// What each slot holds, by slot index from `from_ms`.
fn slots(rows: &[ActivityRow], from_ms: i64, to_ms: i64, slot_ms: i64) -> BTreeMap<i64, Slot> {
    let mut slots: BTreeMap<i64, Slot> = BTreeMap::new();
    for row in rows {
        let start_ms = row.start_ms.max(from_ms);
        let end_ms = row.end_ms.min(to_ms);
        if end_ms <= start_ms {
            continue;
        }
        for index in (start_ms - from_ms) / slot_ms..=(end_ms - 1 - from_ms) / slot_ms {
            let slot_start = from_ms + index * slot_ms;
            let overlap = end_ms.min(slot_start + slot_ms) - start_ms.max(slot_start);
            let slot = slots.entry(index).or_default();
            match &row.kind {
                RowKind::App(window) => slot.pieces.push((window.clone(), overlap)),
                RowKind::Away => slot.away_ms += overlap,
                RowKind::Locked => slot.locked_ms += overlap,
            }
        }
    }
    slots
}

/// The whole day view: app blocks, agent blocks, work spans and totals.
pub fn build_day(inputs: &DayInputs<'_>) -> DayTimeline {
    let DayInputs {
        rows,
        from_ms,
        to_ms,
        slot_ms,
        sessions,
        rules,
        ..
    } = *inputs;
    let mut timeline = group(rows, from_ms, to_ms, slot_ms);
    let private: HashSet<&SessionRef> = inputs.private.iter().collect();
    let accounting = account(inputs, &private);
    let public: Vec<SessionContext> = sessions
        .iter()
        .filter(|session| !private.contains(&session_ref(&session.span)))
        .cloned()
        .collect();
    timeline.work = work_spans(
        &DayInputs {
            sessions: &public,
            ..*inputs
        },
        &accounting,
    );
    timeline.agents = agent_blocks(from_ms, to_ms, slot_ms, sessions, rules, inputs.projects);
    for agent in &mut timeline.agents {
        agent.split = accounting.session_split(&agent.sessions, agent.start_ms, agent.end_ms);
        agent.private = agent
            .sessions
            .iter()
            .all(|session| private.contains(session));
    }
    timeline.totals.agent_ms = agent_ms(from_ms, to_ms, sessions);
    timeline.totals.split = accounting.split(from_ms, to_ms);
    timeline.activity = accounting.work;
    timeline
}

/// The day's time by the user's part, as exact sorted disjoint ranges, so
/// no display zoom changes it.
struct Accounting {
    work: Vec<(i64, i64)>,
    /// Agent time counted while the user was away.
    agent: Vec<(i64, i64)>,
    /// Agent time that does not count.
    uncounted: Vec<(i64, i64)>,
    sessions: HashMap<SessionRef, SessionTime>,
}

/// Sorted disjoint `[start, end)` ranges.
type Ranges = Vec<(i64, i64)>;

/// One session's counted agent time and its whole runtime.
struct SessionTime {
    counted: Ranges,
    runtime: Ranges,
}

impl Accounting {
    fn split(&self, start_ms: i64, end_ms: i64) -> TimeSplit {
        TimeSplit {
            work_ms: overlap_ms(&self.work, start_ms, end_ms),
            agent_ms: overlap_ms(&self.agent, start_ms, end_ms),
            uncounted_ms: overlap_ms(&self.uncounted, start_ms, end_ms),
        }
    }

    /// The part of `[start_ms, end_ms)` that belongs to these sessions.
    fn session_split(&self, sessions: &[SessionRef], start_ms: i64, end_ms: i64) -> TimeSplit {
        let (mut counted, mut runtime) = (Vec::new(), Vec::new());
        for time in sessions
            .iter()
            .filter_map(|session| self.sessions.get(session))
        {
            counted.extend_from_slice(&time.counted);
            runtime.extend_from_slice(&time.runtime);
        }
        TimeSplit {
            work_ms: overlap_ms(&self.work, start_ms, end_ms),
            agent_ms: overlap_ms(&union(counted), start_ms, end_ms),
            uncounted_ms: overlap_ms(&subtract(&union(runtime), &self.work), start_ms, end_ms),
        }
    }
}

/// A prompt proves the user took part in its own minute, nothing more.
const PROMPT_MS: i64 = 60_000;

/// The user's presence is app time on the computer and each prompt's minute
/// (prompts from a phone are in the same session logs), with breaks up to
/// the user's break rule filled. Agent time outside it counts for the
/// user's agent rule after a prompt of the same session. Private sessions
/// add nothing.
fn account(inputs: &DayInputs<'_>, private: &HashSet<&SessionRef>) -> Accounting {
    let DayInputs {
        rows,
        from_ms,
        to_ms,
        sessions,
        time,
        ..
    } = *inputs;
    let clip = |(start, end): (i64, i64)| (start.max(from_ms), end.min(to_ms));
    let ranges = |iter: &mut dyn Iterator<Item = (i64, i64)>| {
        union(iter.map(clip).filter(|(start, end)| end > start).collect())
    };
    let mut runs: HashMap<SessionRef, (Ranges, Vec<i64>)> = HashMap::new();
    for session in sessions {
        let (run, prompts) = runs.entry(session_ref(&session.span)).or_default();
        run.push((session.span.start_ms, session_end_ms(&session.span)));
        prompts.extend_from_slice(&session.span.prompt_times);
    }
    let presence = bridge(
        &ranges(
            &mut rows
                .iter()
                .filter(|row| matches!(row.kind, RowKind::App(_)))
                .map(|row| (row.start_ms, row.end_ms))
                .chain(
                    runs.iter()
                        .filter(|(key, _)| !private.contains(key))
                        .flat_map(|(_, (_, prompts))| {
                            prompts.iter().map(|at| (*at, at + PROMPT_MS))
                        }),
                ),
        ),
        minutes_ms(time.break_minutes),
    );
    let mut by_session = HashMap::new();
    let (mut all_agent, mut all_runtime) = (Vec::new(), Vec::new());
    for (key, (run, prompts)) in runs {
        let run = ranges(&mut run.into_iter());
        let counted = if private.contains(&key) {
            Vec::new()
        } else {
            let after_prompts = ranges(
                &mut prompts
                    .iter()
                    .map(|at| (*at, at + minutes_ms(time.agent_after_prompt_minutes))),
            );
            subtract(&intersect(&run, &after_prompts), &presence)
        };
        all_agent.extend_from_slice(&counted);
        all_runtime.extend_from_slice(&run);
        by_session.insert(
            key,
            SessionTime {
                counted,
                runtime: run,
            },
        );
    }
    let agent = union(all_agent);
    let mut work = presence;
    work.extend_from_slice(&agent);
    let work = union(work);
    Accounting {
        uncounted: subtract(&union(all_runtime), &work),
        work,
        agent,
        sessions: by_session,
    }
}

fn session_ref(span: &ContextSpan) -> SessionRef {
    SessionRef {
        source: span.source,
        session_id: span.session_id.clone(),
    }
}

/// Sorted disjoint `ranges` with the gaps of at most `gap_ms` filled.
fn bridge(ranges: &[(i64, i64)], gap_ms: i64) -> Vec<(i64, i64)> {
    let mut joined: Vec<(i64, i64)> = Vec::new();
    for &(start, end) in ranges {
        match joined.last_mut() {
            Some(last) if start - last.1 <= gap_ms => last.1 = last.1.max(end),
            _ => joined.push((start, end)),
        }
    }
    joined
}

/// `a` without the time in `b`; both sorted and disjoint.
fn subtract(a: &[(i64, i64)], b: &[(i64, i64)]) -> Vec<(i64, i64)> {
    let mut left = Vec::new();
    for &(start, end) in a {
        let mut from = start;
        for &(cut_start, cut_end) in b {
            if cut_end <= from || cut_start >= end {
                continue;
            }
            if cut_start > from {
                left.push((from, cut_start));
            }
            from = from.max(cut_end);
        }
        if from < end {
            left.push((from, end));
        }
    }
    left
}

/// The time in both `a` and `b`; both sorted and disjoint.
fn intersect(a: &[(i64, i64)], b: &[(i64, i64)]) -> Vec<(i64, i64)> {
    let mut both = Vec::new();
    for &(start, end) in a {
        for &(other_start, other_end) in b {
            let (from, to) = (start.max(other_start), end.min(other_end));
            if to > from {
                both.push((from, to));
            }
        }
    }
    union(both)
}

/// A span's issue weights while slots join: the issue's weight, all
/// weight, and the other issues' weights.
#[derive(Default)]
struct Weights {
    explicit: bool,
    score: i64,
    total: i64,
    others: HashMap<String, i64>,
}

/// The likely issue of each slot with app or agent time, from its windows,
/// the agent sessions that ran during it and the user's rules; neighbour
/// slots with the same issue form one span. Spans do not follow the app
/// blocks, and their time split comes from the exact ranges.
fn work_spans(inputs: &DayInputs<'_>, accounting: &Accounting) -> Vec<WorkSpan> {
    let DayInputs {
        rows,
        from_ms,
        to_ms,
        slot_ms,
        sessions,
        rules,
        projects,
        ..
    } = *inputs;
    let mut slots = slots(rows, from_ms, to_ms, slot_ms);
    for session in sessions {
        for index in slot_range(
            session.span.start_ms,
            session_end_ms(&session.span),
            from_ms,
            to_ms,
            slot_ms,
        ) {
            slots.entry(index).or_default();
        }
    }
    let mut spans: Vec<(WorkSpan, Weights)> = Vec::new();
    for (index, slot) in slots {
        let start_ms = from_ms + index * slot_ms;
        let end_ms = (start_ms + slot_ms).min(to_ms);
        let items = items(slot.pieces);
        let windows: Vec<WindowTime<'_>> = items
            .iter()
            .map(|item| WindowTime {
                app_id: &item.app.id,
                app: &item.app.name,
                title: &item.title,
                url: item.url.as_deref(),
                duration_ms: item.duration_ms,
            })
            .collect();
        let Some(work) = detect_work(start_ms, end_ms, &windows, sessions, rules, projects) else {
            continue;
        };
        let joins = spans
            .last()
            .is_some_and(|(last, _)| last.end_ms == start_ms && last.issue == work.issue);
        if !joins {
            spans.push((
                WorkSpan {
                    start_ms,
                    end_ms,
                    issue: work.issue.clone(),
                    confidence: work.confidence,
                    support: 0,
                    candidates: Vec::new(),
                    evidence: Vec::new(),
                    split: TimeSplit::default(),
                },
                Weights::default(),
            ));
        }
        let Some((span, weights)) = spans.last_mut() else {
            continue;
        };
        span.end_ms = end_ms;
        for evidence in work.evidence {
            if span.evidence.len() < MAX_EVIDENCE && !span.evidence.contains(&evidence) {
                span.evidence.push(evidence);
            }
        }
        weights.explicit |= work.explicit;
        weights.score += work.score;
        weights.total += work.total;
        for (other, score) in work.others {
            *weights.others.entry(other).or_default() += score;
        }
    }
    spans
        .into_iter()
        .map(|(mut span, weights)| {
            let mut others: Vec<(String, i64)> = weights.others.into_iter().collect();
            others.sort_by(|(a_key, a), (b_key, b)| b.cmp(a).then_with(|| a_key.cmp(b_key)));
            let runner_up = others.first().map_or(0, |(_, score)| *score);
            span.confidence = confidence(weights.explicit, weights.score, weights.total, runner_up);
            span.support = percent(weights.score, weights.total);
            span.candidates = others
                .into_iter()
                .take(MAX_CANDIDATES)
                .map(|(issue, score)| IssueSupport {
                    issue,
                    support: percent(score, weights.total),
                })
                .collect();
            span.split = accounting.split(span.start_ms, span.end_ms);
            span
        })
        .filter(|span| span.split.work_ms > 0)
        .collect()
}

/// Other issues listed with a work span.
const MAX_CANDIDATES: usize = 3;

fn percent(part: i64, whole: i64) -> u8 {
    if whole <= 0 {
        return 0;
    }
    u8::try_from((part * 100 + whole / 2) / whole).unwrap_or(100)
}

/// App time and agent time in the day as sorted disjoint ranges; parallel
/// sessions and app use at the same moment count once.
pub fn activity(
    rows: &[ActivityRow],
    from_ms: i64,
    to_ms: i64,
    sessions: &[SessionContext],
) -> Vec<(i64, i64)> {
    let apps = rows
        .iter()
        .filter(|row| matches!(row.kind, RowKind::App(_)))
        .map(|row| (row.start_ms, row.end_ms));
    let agents = sessions
        .iter()
        .map(|session| (session.span.start_ms, session_end_ms(&session.span)));
    union(
        apps.chain(agents)
            .map(|(start, end)| (start.max(from_ms), end.min(to_ms)))
            .filter(|(start, end)| end > start)
            .collect(),
    )
}

/// Sorted disjoint ranges covering the same time as `ranges`.
pub fn union(mut ranges: Vec<(i64, i64)>) -> Vec<(i64, i64)> {
    ranges.sort_unstable();
    let mut merged: Vec<(i64, i64)> = Vec::with_capacity(ranges.len());
    for (start, end) in ranges {
        match merged.last_mut() {
            Some(last) if start <= last.1 => last.1 = last.1.max(end),
            _ => merged.push((start, end)),
        }
    }
    merged
}

/// How much of `[start_ms, end_ms)` the sorted disjoint `ranges` cover.
pub fn overlap_ms(ranges: &[(i64, i64)], start_ms: i64, end_ms: i64) -> i64 {
    ranges
        .iter()
        .map(|(from, to)| (*to).min(end_ms) - (*from).max(start_ms))
        .filter(|ms| *ms > 0)
        .sum()
}

/// The window with the most time in the slot, unless offline time weighs more.
fn dominant(slot: &Slot) -> Dominant {
    let mut by_window: HashMap<(&AppRef, &Title), i64> = HashMap::new();
    for (window, duration) in &slot.pieces {
        *by_window.entry((&window.app, &window.title)).or_default() += duration;
    }
    let top = by_window.into_iter().max_by(|(a_key, a), (b_key, b)| {
        a.cmp(b)
            .then_with(|| b_key.0.id.cmp(&a_key.0.id))
            .then_with(|| format!("{:?}", b_key.1).cmp(&format!("{:?}", a_key.1)))
    });
    match top {
        Some(((app, title), ms))
            if ms * APP_WEIGHT >= (slot.away_ms + slot.locked_ms) * OFFLINE_WEIGHT =>
        {
            Dominant::Window {
                app: app.clone(),
                title: title.clone(),
            }
        }
        _ => Dominant::Offline,
    }
}

/// Time per window (and browser address), longest first.
fn items(pieces: Vec<(Window, i64)>) -> Vec<BlockItem> {
    let mut by_window: HashMap<(AppRef, Title, Option<String>), i64> = HashMap::new();
    for (window, duration) in pieces {
        *by_window
            .entry((window.app, window.title, window.url))
            .or_default() += duration;
    }
    let mut items: Vec<BlockItem> = by_window
        .into_iter()
        .map(|((app, title, url), duration_ms)| BlockItem {
            app,
            title,
            url,
            duration_ms,
        })
        .collect();
    items.sort_by(|a, b| {
        b.duration_ms
            .cmp(&a.duration_ms)
            .then_with(|| a.app.id.cmp(&b.app.id))
    });
    items
}

/// Slot indices that `[start_ms, end_ms)` touches within the day.
fn slot_range(start_ms: i64, end_ms: i64, from_ms: i64, to_ms: i64, slot_ms: i64) -> Vec<i64> {
    let start_ms = start_ms.max(from_ms);
    let end_ms = end_ms.min(to_ms);
    if end_ms <= start_ms {
        return Vec::new();
    }
    ((start_ms - from_ms) / slot_ms..=(end_ms - 1 - from_ms) / slot_ms).collect()
}

/// Agent sessions on the slot grid: each session's spans widen to whole
/// slots, and spans of one session that touch join into one block. A
/// resumed conversation gets a new session id but keeps its title, so the
/// title (or the id when there is none) names the session.
pub fn agent_blocks(
    from_ms: i64,
    to_ms: i64,
    slot_ms: i64,
    sessions: &[SessionContext],
    rules: &[RuleKind],
    projects: &[String],
) -> Vec<AgentBlock> {
    let mut by_session: BTreeMap<(&str, AgentSource), Vec<&SessionContext>> = BTreeMap::new();
    for session in sessions {
        by_session
            .entry((
                session
                    .span
                    .title
                    .as_deref()
                    .unwrap_or(&session.span.session_id),
                session.span.source,
            ))
            .or_default()
            .push(session);
    }
    let mut blocks = Vec::new();
    for mut spans in by_session.into_values() {
        spans.sort_by_key(|session| session.span.start_ms);
        let mut runs: Vec<(i64, i64, Vec<&SessionContext>)> = Vec::new();
        for session in spans {
            let slots = slot_range(
                session.span.start_ms,
                session_end_ms(&session.span),
                from_ms,
                to_ms,
                slot_ms,
            );
            let (Some(first), Some(last)) = (slots.first(), slots.last()) else {
                continue;
            };
            let start_ms = from_ms + first * slot_ms;
            let end_ms = (from_ms + (last + 1) * slot_ms).min(to_ms);
            match runs.last_mut() {
                Some((_, run_end, members)) if start_ms <= *run_end => {
                    *run_end = (*run_end).max(end_ms);
                    members.push(session);
                }
                _ => runs.push((start_ms, end_ms, vec![session])),
            }
        }
        for (start_ms, end_ms, members) in runs {
            blocks.push(agent_block(start_ms, end_ms, &members, rules, projects));
        }
    }
    blocks.sort_by_key(|block| (block.start_ms, block.end_ms));
    blocks
}

fn agent_block(
    start_ms: i64,
    end_ms: i64,
    members: &[&SessionContext],
    rules: &[RuleKind],
    projects: &[String],
) -> AgentBlock {
    let owned: Vec<SessionContext> = members.iter().map(|session| (*session).clone()).collect();
    let mut commits: Vec<String> = Vec::new();
    for (at, subject) in members.iter().flat_map(|session| &session.commits) {
        if (start_ms..end_ms).contains(at) && !commits.contains(subject) {
            commits.push(subject.clone());
        }
    }
    let mut prompt_times_ms: Vec<i64> = members
        .iter()
        .flat_map(|session| session.span.prompt_times.iter().copied())
        .filter(|at| (start_ms..end_ms).contains(at))
        .collect();
    prompt_times_ms.sort_unstable();
    prompt_times_ms.dedup();
    let mut sessions: Vec<SessionRef> = members
        .iter()
        .map(|session| session_ref(&session.span))
        .collect();
    sessions.sort();
    sessions.dedup();
    AgentBlock {
        start_ms,
        end_ms,
        source: members[0].span.source,
        title: members.iter().rev().find_map(|s| s.span.title.clone()),
        branch: members.iter().rev().find_map(|s| s.span.branch.clone()),
        prompts: u32::try_from(prompt_times_ms.len()).unwrap_or(u32::MAX),
        prompt_times_ms,
        issue: detect_work(start_ms, end_ms, &[], &owned, rules, projects).map(|work| work.issue),
        commits,
        split: TimeSplit::default(),
        private: false,
        sessions,
    }
}

/// Time any agent worked within the day; parallel sessions count once.
pub fn agent_ms(from_ms: i64, to_ms: i64, sessions: &[SessionContext]) -> i64 {
    activity(&[], from_ms, to_ms, sessions)
        .iter()
        .map(|(start, end)| end - start)
        .sum()
}

/// Time per app within `[from_ms, to_ms)`, most first: the evidence shown
/// next to a worklog proposal.
pub fn app_usage(rows: &[ActivityRow], from_ms: i64, to_ms: i64) -> Vec<AppUsage> {
    let mut by_app: HashMap<AppRef, i64> = HashMap::new();
    for row in rows {
        if let RowKind::App(window) = &row.kind {
            let overlap = row.end_ms.min(to_ms) - row.start_ms.max(from_ms);
            if overlap > 0 {
                *by_app.entry(window.app.clone()).or_default() += overlap;
            }
        }
    }
    let mut usage: Vec<AppUsage> = by_app
        .into_iter()
        .map(|(app, duration_ms)| AppUsage { app, duration_ms })
        .collect();
    usage.sort_by(|a, b| {
        b.duration_ms
            .cmp(&a.duration_ms)
            .then_with(|| a.app.id.cmp(&b.app.id))
    });
    usage
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIN: i64 = 60_000;

    fn app_row(app: &str, title: &str, start_min: i64, end_min: i64) -> ActivityRow {
        ActivityRow {
            id: 0,
            start_ms: start_min * MIN,
            end_ms: end_min * MIN,
            kind: RowKind::App(Window {
                app: AppRef {
                    id: app.to_owned(),
                    name: app.to_owned(),
                },
                title: Title::Known {
                    text: title.to_owned(),
                },
                document: None,
                url: None,
            }),
        }
    }

    fn summary(timeline: &DayTimeline) -> Vec<(i64, i64, String)> {
        timeline
            .blocks
            .iter()
            .map(|block| match block {
                Block::Apps {
                    start_ms,
                    end_ms,
                    dominant,
                    ..
                } => (*start_ms / MIN, *end_ms / MIN, dominant.name.clone()),
                Block::Away { start_ms, end_ms } => (*start_ms / MIN, *end_ms / MIN, "away".into()),
                Block::Locked { start_ms, end_ms } => {
                    (*start_ms / MIN, *end_ms / MIN, "locked".into())
                }
            })
            .collect()
    }

    fn offline_row(kind: RowKind, start_min: i64, end_min: i64) -> ActivityRow {
        ActivityRow {
            id: 0,
            start_ms: start_min * MIN,
            end_ms: end_min * MIN,
            kind,
        }
    }

    const DAY: i64 = 24 * 60 * MIN;

    #[test]
    fn a_slot_shows_its_dominant_window_and_keeps_the_others() {
        let rows = [
            app_row("Rider", "a.fs", 2, 12),
            app_row("Slack", "#dev", 12, 15),
        ];

        let timeline = group(&rows, 0, DAY, DEFAULT_SLOT_MS);

        assert_eq!(summary(&timeline), vec![(0, 15, "Rider".into())]);
        let Block::Apps { items, title, .. } = &timeline.blocks[0] else {
            panic!("expected an apps block")
        };
        assert_eq!(
            title,
            &Title::Known {
                text: "a.fs".into()
            }
        );
        assert_eq!(
            items
                .iter()
                .map(|i| i.duration_ms / MIN)
                .collect::<Vec<_>>(),
            vec![10, 3]
        );
    }

    #[test]
    fn neighbour_slots_with_the_same_window_merge() {
        let rows = [
            app_row("Rider", "a.fs", 0, 40),
            app_row("Teams", "Stand-up", 40, 44),
            app_row("Rider", "a.fs", 44, 50),
        ];

        let timeline = group(&rows, 0, DAY, DEFAULT_SLOT_MS);

        assert_eq!(summary(&timeline), vec![(0, 60, "Rider".into())]);
    }

    #[test]
    fn another_dominant_window_starts_a_new_block() {
        let rows = [
            app_row("Rider", "a.fs", 0, 15),
            app_row("Rider", "b.fs", 15, 30),
            app_row("Teams", "Stand-up", 30, 45),
        ];

        let timeline = group(&rows, 0, DAY, DEFAULT_SLOT_MS);

        assert_eq!(
            summary(&timeline),
            vec![
                (0, 15, "Rider".into()),
                (15, 30, "Rider".into()),
                (30, 45, "Teams".into())
            ]
        );
    }

    #[test]
    fn offline_time_weighs_less_than_app_time() {
        let rows = [
            app_row("Rider", "a.fs", 0, 7),
            offline_row(RowKind::Away, 7, 15),
            app_row("Rider", "a.fs", 15, 18),
            offline_row(RowKind::Away, 18, 25),
            offline_row(RowKind::Locked, 25, 45),
        ];

        let timeline = group(&rows, 0, DAY, DEFAULT_SLOT_MS);

        assert_eq!(
            summary(&timeline),
            vec![(0, 15, "Rider".into()), (15, 45, "locked".into())]
        );
        assert_eq!(
            timeline.totals,
            Totals {
                active_ms: 10 * MIN,
                away_ms: 35 * MIN,
                agent_ms: 0,
                split: TimeSplit::default(),
            }
        );
    }

    #[test]
    fn rows_are_clipped_to_the_day_and_empty_slots_split_blocks() {
        let rows = [
            app_row("Rider", "a.fs", -10, 10),
            app_row("Rider", "a.fs", 35, 40),
        ];

        let timeline = group(&rows, 0, DAY, DEFAULT_SLOT_MS);

        assert_eq!(
            summary(&timeline),
            vec![(0, 15, "Rider".into()), (30, 45, "Rider".into())]
        );
        assert_eq!(timeline.totals.active_ms, 15 * MIN);
    }

    #[test]
    fn work_spans_follow_the_issue_not_the_app_blocks() {
        let rows = [
            app_row("Rider", "INT-290 a.fs", 0, 20),
            app_row("Rider", "b.fs", 20, 45),
            app_row("Dia", "IS-61 · YouTrack", 45, 60),
        ];

        let spans = day(&rows, &[], &[]).work;

        assert_eq!(
            spans
                .iter()
                .map(|span| (span.start_ms / MIN, span.end_ms / MIN, span.issue.as_str()))
                .collect::<Vec<_>>(),
            vec![(0, 30, "INT-290"), (45, 60, "IS-61")]
        );
        assert_eq!(spans[0].confidence, Confidence::High);
    }

    fn session(id: &str, branch: &str, start: i64, end: i64, prompts: &[i64]) -> SessionContext {
        SessionContext {
            span: crate::model::ContextSpan {
                source: AgentSource::ClaudeCode,
                session_id: id.into(),
                title: Some(format!("{id} title")),
                cwd: None,
                branch: Some(branch.into()),
                start_ms: start * MIN,
                end_ms: end * MIN,
                prompt_times: prompts.iter().map(|at| at * MIN).collect(),
            },
            commits: Vec::new(),
        }
    }

    #[test]
    fn agent_sessions_become_slot_blocks_that_may_overlap() {
        let sessions = [
            session("one", "claude/int-290-push", 0, 10, &[2]),
            session("one", "claude/int-290-push", 12, 20, &[13]),
            session("two", "main", 5, 6, &[]),
        ];

        let blocks = agent_blocks(0, DAY, DEFAULT_SLOT_MS, &sessions, &[], &[]);

        assert_eq!(
            blocks
                .iter()
                .map(|b| (b.start_ms / MIN, b.end_ms / MIN, b.prompts, b.issue.clone()))
                .collect::<Vec<_>>(),
            vec![(0, 15, 0, None), (0, 30, 2, Some("INT-290".to_owned()))]
        );
        assert_eq!(agent_ms(0, DAY, &sessions), 18 * MIN);
    }

    #[test]
    fn prompt_times_are_sorted_unique_and_inside_the_block() {
        let sessions = [
            // Resumed: both spans carry the 13th minute.
            session("one", "main", 0, 10, &[9, 2]),
            session("one", "main", 12, 20, &[13, 13]),
            // Crosses the day's end: the prompt at the end belongs to the next day.
            session("late", "main", 1_430, 1_445, &[1_435, 1_440]),
        ];

        for zoom in ZOOM_MINUTES {
            let blocks = agent_blocks(0, DAY, i64::from(zoom) * MIN, &sessions, &[], &[]);
            let mut all: Vec<i64> = Vec::new();
            for block in &blocks {
                assert!(
                    block
                        .prompt_times_ms
                        .windows(2)
                        .all(|pair| pair[0] < pair[1])
                );
                assert!(
                    block
                        .prompt_times_ms
                        .iter()
                        .all(|at| (block.start_ms..block.end_ms).contains(at))
                );
                assert_eq!(block.prompts as usize, block.prompt_times_ms.len());
                all.extend(block.prompt_times_ms.iter().map(|at| at / MIN));
            }
            all.sort_unstable();
            assert_eq!(all, vec![2, 9, 13, 1_435], "zoom {zoom}");
        }
    }

    fn private(id: &str) -> SessionRef {
        SessionRef {
            source: AgentSource::ClaudeCode,
            session_id: id.into(),
        }
    }

    #[test]
    fn agent_time_counts_up_to_30_minutes_after_the_sessions_prompt() {
        let rows = [offline_row(RowKind::Locked, 0, 200)];
        let sessions = [session("one", "claude/int-290-push", 0, 200, &[10])];

        let day = day(&rows, &sessions, &[]);

        // The prompt minute 10–11, then the agent alone 11–40.
        assert_eq!(day.totals.split, split(30, 29, 170));
        assert_eq!(day.activity, vec![(10 * MIN, 40 * MIN)]);
    }

    #[test]
    fn a_session_without_prompts_never_counts() {
        let rows = [
            app_row("Dia", "Shopping", 0, 10),
            offline_row(RowKind::Locked, 10, 300),
        ];
        let sessions = [session("heartbeats", "claude/int-290-push", 0, 300, &[])];

        let day = day(&rows, &sessions, &[]);

        assert_eq!(day.totals.split, split(10, 0, 290));
    }

    #[test]
    fn a_prompt_counts_only_its_own_sessions_agent_time() {
        let sessions = [
            session("a", "main", 0, 15, &[5]),
            session("b", "claude/int-290-push", 0, 15, &[]),
        ];

        let day = day(&[], &sessions, &[]);

        assert_eq!(day.totals.split, split(10, 9, 5));
        let own = |title: &str| {
            day.agents
                .iter()
                .find(|agent| agent.title.as_deref() == Some(title))
                .map(|agent| (agent.split.agent_ms / MIN, agent.split.uncounted_ms / MIN))
        };
        assert_eq!(own("a title"), Some((9, 5)));
        assert_eq!(own("b title"), Some((0, 5)));
    }

    #[test]
    fn breaks_of_up_to_15_minutes_count_and_longer_ones_do_not() {
        let rows = [
            app_row("Rider", "INT-290 a.fs", 0, 10),
            offline_row(RowKind::Away, 10, 25),
            app_row("Rider", "INT-290 a.fs", 25, 30),
            offline_row(RowKind::Locked, 30, 46),
            app_row("Rider", "INT-290 a.fs", 46, 50),
        ];

        let day = day(&rows, &[], &[]);

        assert_eq!(day.totals.split, split(34, 0, 0));
        assert_eq!(day.activity, vec![(0, 30 * MIN), (46 * MIN, 50 * MIN)]);
    }

    #[test]
    fn the_users_rules_set_which_breaks_and_agent_time_count() {
        let rows = [
            app_row("Rider", "INT-290 a.fs", 0, 10),
            app_row("Rider", "INT-290 a.fs", 25, 30),
        ];
        let sessions = [session("one", "claude/int-290-push", 30, 60, &[30])];
        let strict = TimeRules {
            break_minutes: 10,
            agent_after_prompt_minutes: 10,
            ..TimeRules::default()
        };

        let day = build_day(&DayInputs {
            rows: &rows,
            from_ms: 0,
            to_ms: DAY,
            slot_ms: DEFAULT_SLOT_MS,
            sessions: &sessions,
            rules: &[],
            private: &[],
            time: strict,
            projects: &[],
        });

        // The 15-minute pause no longer counts; the agent counts 10 minutes
        // after its prompt, minus the prompt's own minute.
        assert_eq!(day.totals.split.work_ms, (10 + 5 + 10) * MIN);
        assert_eq!(day.totals.split.agent_ms, 9 * MIN);
    }

    #[test]
    fn a_private_session_adds_nothing_and_names_no_issue() {
        let sessions = [session("one", "claude/int-290-push", 0, 30, &[5])];

        let counted = day(&[], &sessions, &[]);
        let hidden = day(&[], &sessions, &[private("one")]);

        assert_eq!(counted.totals.split, split(25, 24, 5));
        assert_eq!(counted.work[0].issue, "INT-290");
        assert!(!counted.agents[0].private);
        assert_eq!(hidden.totals.split, split(0, 0, 30));
        assert!(hidden.work.is_empty());
        assert!(hidden.agents[0].private);
    }

    #[test]
    fn the_time_split_does_not_depend_on_the_zoom() {
        let rows = [
            app_row("Rider", "INT-290 a.fs", 3, 4),
            offline_row(RowKind::Locked, 4, 200),
        ];
        let sessions = [
            session("one", "claude/int-290-push", 0, 200, &[2, 61, 140]),
            session("two", "main", 30, 90, &[]),
        ];

        let by_zoom: Vec<(TimeSplit, Vec<(i64, i64)>)> = ZOOM_MINUTES
            .iter()
            .map(|zoom| {
                let day = build_day(&DayInputs {
                    rows: &rows,
                    from_ms: 0,
                    to_ms: DAY,
                    slot_ms: i64::from(*zoom) * MIN,
                    sessions: &sessions,
                    rules: &[],
                    private: &[],
                    time: TimeRules::default(),
                    projects: &[],
                });
                (day.totals.split, day.activity)
            })
            .collect();

        assert!(by_zoom.windows(2).all(|pair| pair[0] == pair[1]));
        // Presence: 2–4, 61–62, 140–141. The agent alone after each prompt:
        // 4–32, 62–91, 141–170.
        assert_eq!(by_zoom[0].0, split(90, 86, 110));
    }

    #[test]
    fn a_short_app_visit_proves_only_itself() {
        let mut visit = app_row("Rider", "INT-290 a.fs", 0, 0);
        visit.end_ms = 10_000;
        let sessions = [session("one", "claude/int-290-push", 0, 15, &[])];

        let day = day(&[visit], &sessions, &[]);

        assert_eq!(
            (day.totals.split.work_ms, day.totals.split.uncounted_ms),
            (10_000, 15 * MIN - 10_000)
        );
    }

    #[test]
    fn a_phone_prompt_counts_while_the_mac_is_locked() {
        let rows = [offline_row(RowKind::Locked, 0, 60)];
        let sessions = [session("one", "codex/int-290-push", 15, 40, &[20])];

        let day = day(&rows, &sessions, &[]);

        assert_eq!(day.totals.split, split(20, 19, 5));
        assert_eq!(day.activity, vec![(20 * MIN, 40 * MIN)]);
    }

    #[test]
    fn app_time_and_parallel_agents_count_once() {
        let rows = [app_row("Rider", "INT-290 a.fs", 0, 15)];
        let sessions = [
            session("one", "claude/int-290-push", 0, 15, &[]),
            session("two", "claude/int-290-tests", 5, 15, &[]),
        ];

        let day = day(&rows, &sessions, &[]);

        assert_eq!(day.totals.split, split(15, 0, 0));
        assert_eq!(day.totals.agent_ms, 15 * MIN);
    }

    #[test]
    fn a_weakly_named_span_reports_its_support_and_other_issues() {
        let rows = [
            app_row("Dia", "Auth0 sign-in test", 0, 14),
            app_row("Dia", "INT-122 ticket", 14, 15),
        ];

        let span = &day(&rows, &[], &[]).work[0];

        assert_eq!(
            (span.issue.as_str(), span.confidence, span.support),
            ("INT-122", Confidence::Medium, 7)
        );
    }

    fn day(
        rows: &[ActivityRow],
        sessions: &[SessionContext],
        private: &[SessionRef],
    ) -> DayTimeline {
        build_day(&DayInputs {
            rows,
            from_ms: 0,
            to_ms: DAY,
            slot_ms: DEFAULT_SLOT_MS,
            sessions,
            rules: &[],
            private,
            time: TimeRules::default(),
            projects: &[],
        })
    }

    /// Work, counted agent and uncounted agent minutes.
    fn split(work: i64, agent: i64, uncounted: i64) -> TimeSplit {
        TimeSplit {
            work_ms: work * MIN,
            agent_ms: agent * MIN,
            uncounted_ms: uncounted * MIN,
        }
    }

    #[test]
    fn a_minute_of_work_is_a_minute_not_a_slot() {
        let rows = [app_row("Rider", "INT-290 a.fs", 3, 4)];

        let spans = day(&rows, &[], &[]).work;

        assert_eq!(
            (spans[0].end_ms - spans[0].start_ms, spans[0].split.work_ms),
            (15 * MIN, MIN)
        );
    }
}
