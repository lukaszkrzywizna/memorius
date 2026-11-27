//! Read models shared by the desktop app and the `memorius` CLI: the day
//! timeline with detected work, and the week summary.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use jiff::Timestamp;
use jiff::ToSpan;
use jiff::civil::{Date, ISOWeekDate, Weekday};
use jiff::tz::TimeZone;
use memorius_core::context::{SessionContext, join_session_spans};
use memorius_core::logged::logged_lane;
use memorius_core::model::RuleKind;
use memorius_core::store::{Store, StoreError};
use memorius_core::time_rules::{TIME_RULES, TimeRules, WeekStart};
use memorius_core::timeline::{DEFAULT_SLOT_MS, DayInputs, DayTimeline, build_day};
use memorius_core::tracker::PROJECTS;
use memorius_core::week::{DayLedger, DaySummary, RuleMatch, WeekSummary, rule_matches, summarize};

use crate::git::GitHistories;

/// Where the desktop app keeps its database (Tauri's app data folder for
/// `app.memorius.desktop`); `MEMORIUS_DB` overrides it.
pub fn default_database() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("MEMORIUS_DB") {
        return Some(PathBuf::from(path));
    }
    let base = if cfg!(windows) {
        PathBuf::from(std::env::var_os("APPDATA")?)
    } else {
        std::env::home_dir()?
            .join("Library")
            .join("Application Support")
    };
    Some(base.join("app.memorius.desktop").join("memorius.db"))
}

/// Start and end of a local day (`YYYY-MM-DD`) in Unix milliseconds; DST
/// days have 23 or 25 hours.
pub fn day_bounds(date: &str) -> Result<(i64, i64), jiff::Error> {
    let date: Date = date.parse()?;
    let zone = TimeZone::system();
    let start = date.to_zoned(zone.clone())?.timestamp().as_millisecond();
    let end = date
        .tomorrow()?
        .to_zoned(zone)?
        .timestamp()
        .as_millisecond();
    Ok((start, end))
}

/// The seven days of an ISO week (`2026-W41`) or of the week holding a
/// date (`2026-10-09`), from the user's first day of the week.
pub fn week_dates(week: &str, start: WeekStart) -> Result<Vec<String>, jiff::Error> {
    let first_day = match start {
        WeekStart::Monday => Weekday::Monday,
        WeekStart::Sunday => Weekday::Sunday,
    };
    let date: Date = if let Some((year, number)) = week.split_once("-W") {
        let year: i16 = year
            .parse()
            .map_err(|_| jiff::Error::from_args(format_args!("bad year in {week}")))?;
        let number: i8 = number
            .parse()
            .map_err(|_| jiff::Error::from_args(format_args!("bad week in {week}")))?;
        ISOWeekDate::new(year, number, Weekday::Monday)?.date()
    } else {
        week.parse()?
    };
    let first = date.checked_sub(i64::from(date.weekday().since(first_day)).days())?;
    (0..7_i64)
        .map(|offset| first.checked_add(offset.days()).map(|day| day.to_string()))
        .collect()
}

/// A day's blocks in slots of `slot_ms`, and what the time was about.
pub fn day_timeline(
    store: &Store,
    git: &mut GitHistories,
    from_ms: i64,
    to_ms: i64,
    slot_ms: i64,
) -> Result<DayTimeline, StoreError> {
    let rows = store.rows_between(from_ms, to_ms)?;
    let rules: Vec<RuleKind> = store.rules()?.into_iter().map(|rule| rule.rule).collect();
    let spans = join_session_spans(store.context_spans_between(from_ms, to_ms)?);
    // Each span gets the commits of its whole session that day.
    let mut session_ranges: HashMap<&str, (i64, i64)> = HashMap::new();
    for span in &spans {
        let range = session_ranges
            .entry(&span.session_id)
            .or_insert((span.start_ms, span.end_ms));
        *range = (range.0.min(span.start_ms), range.1.max(span.end_ms));
    }
    let sessions: Vec<SessionContext> = spans
        .iter()
        .map(|span| {
            let (start_ms, end_ms) = session_ranges[span.session_id.as_str()];
            let commits = span
                .cwd
                .as_deref()
                .and_then(|cwd| git.get(Path::new(cwd)))
                .map(|history| {
                    history
                        .commits_between(start_ms, end_ms)
                        .map(|(at, subject)| (at, subject.to_owned()))
                        .collect()
                })
                .unwrap_or_default();
            SessionContext {
                span: span.clone(),
                commits,
            }
        })
        .collect();
    let private = store.private_sessions()?;
    let mut timeline = build_day(&DayInputs {
        rows: &rows,
        from_ms,
        to_ms,
        slot_ms,
        sessions: &sessions,
        rules: &rules,
        private: &private,
        time: store
            .setting::<TimeRules>(TIME_RULES)?
            .unwrap_or_default()
            .clamped(),
        projects: &store.setting::<Vec<String>>(PROJECTS)?.unwrap_or_default(),
    });
    timeline.commits = store.git_commits_between(from_ms, to_ms)?;
    let date = Timestamp::from_millisecond(from_ms)
        .map(|at| at.to_zoned(TimeZone::system()).date().to_string())
        .unwrap_or_default();
    timeline.logged = logged_lane(
        &date,
        &timeline,
        &store.proposals_between(from_ms, to_ms)?,
        &store.tracker_worklogs(&date, &date)?,
    );
    Ok(timeline)
}

#[derive(Debug, thiserror::Error)]
pub enum ViewError {
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Date(#[from] jiff::Error),
}

/// Each date's totals (`YYYY-MM-DD`), for the Log queue's day rows.
pub fn day_summaries(
    store: &Store,
    git: &mut GitHistories,
    dates: &[String],
) -> Result<Vec<DaySummary>, ViewError> {
    dates
        .iter()
        .map(|date| {
            let (from_ms, to_ms) = day_bounds(date)?;
            let timeline = day_timeline(store, git, from_ms, to_ms, DEFAULT_SLOT_MS)?;
            let proposals = store.proposals_between(from_ms, to_ms)?;
            let worklogs = store.tracker_worklogs(date, date)?;
            Ok(DayLedger::new(date, &timeline, &proposals, &worklogs).summary(date))
        })
        .collect()
}

/// The week timesheet for an ISO week or a date in it.
pub fn week_summary(
    store: &Store,
    git: &mut GitHistories,
    week: &str,
) -> Result<WeekSummary, ViewError> {
    let mut days = Vec::with_capacity(7);
    let start = store
        .setting::<TimeRules>(TIME_RULES)?
        .unwrap_or_default()
        .week_starts_on;
    for date in week_dates(week, start)? {
        let (from_ms, to_ms) = day_bounds(&date)?;
        days.push((
            date,
            day_timeline(store, git, from_ms, to_ms, DEFAULT_SLOT_MS)?,
        ));
    }
    let (from_ms, to_ms) = (
        days.first().map_or(0, |(_, day)| day.from_ms),
        days.last().map_or(0, |(_, day)| day.to_ms),
    );
    let proposals = store.proposals_between(from_ms, to_ms)?;
    let worklogs = match (days.first(), days.last()) {
        (Some((first, _)), Some((last, _))) => store.tracker_worklogs(first, last)?,
        _ => Vec::new(),
    };
    Ok(summarize(&days, &proposals, &worklogs))
}

/// How much time each rule named over the 7 days that end on `today`.
pub fn rule_matches_last_week(
    store: &Store,
    git: &mut GitHistories,
    today: &str,
) -> Result<Vec<RuleMatch>, ViewError> {
    let today: Date = today.parse()?;
    let mut days = Vec::with_capacity(7);
    for back in (0..7).rev() {
        let date = today.checked_sub(back.days())?.to_string();
        let (from_ms, to_ms) = day_bounds(&date)?;
        days.push(day_timeline(store, git, from_ms, to_ms, DEFAULT_SLOT_MS)?);
    }
    Ok(rule_matches(&store.rules()?, &days))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_week_runs_from_the_users_first_day() {
        let by_number = week_dates("2026-W41", WeekStart::Monday).unwrap();
        let by_date = week_dates("2026-10-09", WeekStart::Monday).unwrap();
        let sunday = week_dates("2026-10-09", WeekStart::Sunday).unwrap();

        assert_eq!(by_number.first().map(String::as_str), Some("2026-10-05"));
        assert_eq!(by_number.last().map(String::as_str), Some("2026-10-11"));
        assert_eq!(by_number, by_date);
        assert_eq!(sunday.first().map(String::as_str), Some("2026-10-04"));
        assert_eq!(sunday.last().map(String::as_str), Some("2026-10-10"));
    }

    #[test]
    fn a_day_spans_from_local_midnight_to_the_next() {
        let (start, end) = day_bounds("2026-10-09").unwrap();

        assert!((23..=25).contains(&((end - start) / 3_600_000)));
        assert!(day_bounds("09.10.2026").is_err());
    }
}
