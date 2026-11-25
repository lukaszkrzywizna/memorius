//! Work context: agent session spans and the issue each timeline block is
//! most likely about. Pure logic; the desktop host reads the files.

use std::collections::HashMap;

use serde::Serialize;
use ts_rs::TS;

use crate::issue::{branch_issue_keys, issue_keys};
use crate::model::{AgentSource, ContextSpan, RuleKind, Title};

/// A longer pause in a session starts a new span.
pub const SESSION_GAP_MS: i64 = 15 * 60_000;

/// Evidence lists stay short enough to read.
pub(crate) const MAX_EVIDENCE: usize = 4;

/// One session event, as the importer reads it.
pub struct SessionEvent<'a> {
    pub source: AgentSource,
    pub session_id: &'a str,
    pub cwd: Option<&'a str>,
    pub branch: Option<&'a str>,
    pub at_ms: i64,
    pub prompt: bool,
}

/// Adds an event to the spans of one session (oldest first): extends the
/// last span when the session was busy recently on the same branch, else
/// starts a new span.
pub fn add_event(spans: &mut Vec<ContextSpan>, event: &SessionEvent<'_>) {
    if let Some(last) = spans.last_mut()
        && event.at_ms - last.end_ms <= SESSION_GAP_MS
        && event.at_ms >= last.start_ms - SESSION_GAP_MS
        && last.branch.as_deref() == event.branch
    {
        last.start_ms = last.start_ms.min(event.at_ms);
        last.end_ms = last.end_ms.max(event.at_ms);
        if event.prompt {
            last.prompt_times.push(event.at_ms);
        }
        return;
    }
    spans.push(ContextSpan {
        source: event.source,
        session_id: event.session_id.to_owned(),
        title: None,
        cwd: event.cwd.map(str::to_owned),
        branch: event.branch.map(str::to_owned),
        start_ms: event.at_ms,
        end_ms: event.at_ms,
        prompt_times: if event.prompt {
            vec![event.at_ms]
        } else {
            Vec::new()
        },
    });
}

/// Joins overlapping spans of one session. A session's helper agents write
/// their own files with the parent's session id, so their spans repeat the
/// parent's time; joined, the time counts once.
pub fn join_session_spans(mut spans: Vec<ContextSpan>) -> Vec<ContextSpan> {
    spans.sort_by(|a, b| {
        (a.source, &a.session_id, a.start_ms).cmp(&(b.source, &b.session_id, b.start_ms))
    });
    let mut joined: Vec<ContextSpan> = Vec::with_capacity(spans.len());
    for span in spans {
        match joined.last_mut() {
            Some(last)
                if last.source == span.source
                    && last.session_id == span.session_id
                    && span.start_ms <= last.end_ms =>
            {
                last.end_ms = last.end_ms.max(span.end_ms);
                last.prompt_times.extend(span.prompt_times);
                last.prompt_times.sort_unstable();
                last.prompt_times.dedup();
                last.title = last.title.take().or(span.title);
                last.branch = last.branch.take().or(span.branch);
                last.cwd = last.cwd.take().or(span.cwd);
            }
            _ => joined.push(span),
        }
    }
    joined
}

/// A session span plus the commits made in its folder while the session
/// ran that day (all its spans), so a key in any commit names all of it.
#[derive(Clone)]
pub struct SessionContext {
    pub span: ContextSpan,
    /// `(time_ms, subject)`.
    pub commits: Vec<(i64, String)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Confidence {
    /// An issue key in a window title, or a session the user typed in.
    High,
    /// Only a session that ran in the background at the same time.
    Medium,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum Evidence {
    WindowTitle {
        app: String,
        title: String,
    },
    Url {
        app: String,
        url: String,
    },
    Session {
        source: AgentSource,
        title: Option<String>,
        branch: Option<String>,
        prompts: u32,
    },
    Commit {
        subject: String,
    },
    /// One of the user's issue rules matched.
    Rule {
        rule: RuleKind,
    },
}

/// The issue of one slot; neighbour slots join into a `WorkSpan`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetectedWork {
    pub issue: String,
    pub confidence: Confidence,
    pub evidence: Vec<Evidence>,
    /// Whether a title, URL, rule or typed session named the issue.
    pub explicit: bool,
    /// The issue's weight, and the weight of everything in the time, named
    /// or not: `score / total` is the issue's support.
    pub score: i64,
    pub total: i64,
    /// Other named issues with their weight, heaviest first.
    pub others: Vec<(String, i64)>,
}

/// A window shown during the block, with its time.
pub struct WindowTime<'a> {
    pub app_id: &'a str,
    pub app: &'a str,
    pub title: &'a Title,
    pub url: Option<&'a str>,
    pub duration_ms: i64,
}

#[derive(Default)]
struct Candidate {
    score: i64,
    explicit: bool,
    evidence: Vec<Evidence>,
}

impl Candidate {
    fn add(&mut self, score: i64, explicit: bool, evidence: Evidence) {
        self.score += score;
        self.explicit |= explicit;
        if self.evidence.len() < MAX_EVIDENCE && !self.evidence.contains(&evidence) {
            self.evidence.push(evidence);
        }
    }
}

/// The issue a block of time was most likely about, or `None` when nothing
/// names one. Window titles and sessions the user typed in count fully;
/// background sessions count a quarter.
pub fn detect_work(
    start_ms: i64,
    end_ms: i64,
    windows: &[WindowTime<'_>],
    sessions: &[SessionContext],
    rules: &[RuleKind],
    projects: &[String],
) -> Option<DetectedWork> {
    let mut candidates: HashMap<String, Candidate> = HashMap::new();
    for window in windows {
        add_window_evidence(&mut candidates, window, rules);
    }
    for session in sessions {
        add_session_evidence(&mut candidates, start_ms, end_ms, session, rules);
    }
    // With the tracker's projects known, `GPT-5` or `ISO-8601` is no issue;
    // the user's own rules name issues as they are.
    if !projects.is_empty() {
        candidates.retain(|key, candidate| {
            candidate
                .evidence
                .iter()
                .any(|evidence| matches!(evidence, Evidence::Rule { .. }))
                || key
                    .split_once('-')
                    .is_some_and(|(project, _)| projects.iter().any(|known| known == project))
        });
    }
    // Everything the slot holds, named or not: a short URL among long
    // unnamed work is a hint, not an answer.
    let total: i64 = windows.iter().map(|window| window.duration_ms).sum::<i64>()
        + sessions
            .iter()
            .filter_map(|session| session_score(start_ms, end_ms, session))
            .map(|(score, _)| score)
            .sum::<i64>();
    let mut ranked: Vec<(String, Candidate)> = candidates.into_iter().collect();
    ranked.sort_by(|(a_key, a), (b_key, b)| b.score.cmp(&a.score).then_with(|| a_key.cmp(b_key)));
    let mut ranked = ranked.into_iter();
    let (issue, best) = ranked.next()?;
    let others: Vec<(String, i64)> = ranked.map(|(key, other)| (key, other.score)).collect();
    let runner_up = others.first().map_or(0, |(_, score)| *score);
    Some(DetectedWork {
        issue,
        confidence: confidence(best.explicit, best.score, total, runner_up),
        evidence: best.evidence,
        explicit: best.explicit,
        score: best.score,
        total,
        others,
    })
}

fn add_window_evidence(
    candidates: &mut HashMap<String, Candidate>,
    window: &WindowTime<'_>,
    rules: &[RuleKind],
) {
    for rule in rules {
        if let Some(issue) = rule_issue(rule, window) {
            candidates.entry(issue.to_owned()).or_default().add(
                window.duration_ms,
                true,
                Evidence::Rule { rule: rule.clone() },
            );
        }
    }
    if let Title::Known { text } = window.title {
        for key in issue_keys(text) {
            candidates.entry(key).or_default().add(
                window.duration_ms,
                true,
                Evidence::WindowTitle {
                    app: window.app.to_owned(),
                    title: text.clone(),
                },
            );
        }
    }
    for key in window.url.map(issue_keys).unwrap_or_default() {
        candidates.entry(key).or_default().add(
            window.duration_ms,
            true,
            Evidence::Url {
                app: window.app.to_owned(),
                url: window.url.unwrap_or_default().to_owned(),
            },
        );
    }
}

fn add_session_evidence(
    candidates: &mut HashMap<String, Candidate>,
    start_ms: i64,
    end_ms: i64,
    session: &SessionContext,
    rules: &[RuleKind],
) {
    let span = &session.span;
    let Some((score, prompts)) = session_score(start_ms, end_ms, session) else {
        return;
    };
    let typed = prompts > 0;
    let evidence = Evidence::Session {
        source: span.source,
        title: span.title.clone(),
        branch: span.branch.clone(),
        prompts: u32::try_from(prompts).unwrap_or(u32::MAX),
    };
    for rule in rules {
        if let RuleKind::TitleIssue { text, issue } = rule {
            let text = text.to_lowercase();
            let named = [span.title.as_deref(), span.branch.as_deref()]
                .into_iter()
                .flatten()
                .any(|value| value.to_lowercase().contains(&text));
            if named {
                let candidate = candidates.entry(issue.clone()).or_default();
                candidate.add(score, true, Evidence::Rule { rule: rule.clone() });
                candidate.add(0, true, evidence.clone());
            }
        }
    }
    let mut keys: Vec<String> = span
        .branch
        .as_deref()
        .map(branch_issue_keys)
        .unwrap_or_default();
    keys.extend(span.title.as_deref().map(issue_keys).unwrap_or_default());
    // Commits in this stretch break ties between keys of the same session.
    let commit_keys: Vec<(String, &String, bool)> = session
        .commits
        .iter()
        .flat_map(|(at, subject)| {
            let here = (start_ms..=end_ms).contains(at);
            issue_keys(subject)
                .into_iter()
                .map(move |key| (key, subject, here))
        })
        .collect();
    let mut session_keys: Vec<(&String, bool)> = keys.iter().map(|key| (key, false)).collect();
    for (key, _, here) in &commit_keys {
        match session_keys.iter_mut().find(|(known, _)| *known == key) {
            Some(known) => known.1 |= here,
            None => session_keys.push((key, *here)),
        }
    }
    for (key, here) in session_keys {
        let candidate = candidates.entry(key.clone()).or_default();
        candidate.add(score + i64::from(here), typed, evidence.clone());
        for (_, subject, _) in commit_keys
            .iter()
            .filter(|(commit_key, _, _)| commit_key == key)
        {
            candidate.add(
                0,
                false,
                Evidence::Commit {
                    subject: (*subject).clone(),
                },
            );
        }
    }
}

/// The issue a rule names for this window, if it matches.
/// `High` only when something explicit names the issue, the issue holds at
/// least half of the weight, and no other issue has half of its weight.
pub fn confidence(explicit: bool, score: i64, total: i64, runner_up: i64) -> Confidence {
    if explicit && score * 2 >= total && runner_up * 2 < score {
        Confidence::High
    } else {
        Confidence::Medium
    }
}

/// A session's weight in `[start_ms, end_ms)` and the prompts typed there:
/// its overlap when the user typed, a quarter when it ran on its own.
fn session_score(start_ms: i64, end_ms: i64, session: &SessionContext) -> Option<(i64, usize)> {
    let span = &session.span;
    let overlap = end_ms.min(session_end_ms(span)) - start_ms.max(span.start_ms);
    if overlap <= 0 {
        return None;
    }
    let prompts = span
        .prompt_times
        .iter()
        .filter(|at| (start_ms..=end_ms).contains(*at))
        .count();
    Some((if prompts > 0 { overlap } else { overlap / 4 }, prompts))
}

/// A stretch with one event still means a minute of work.
const MIN_SESSION_MS: i64 = 60_000;

/// The end of a session span, at least a minute after its start.
pub fn session_end_ms(span: &ContextSpan) -> i64 {
    span.end_ms.max(span.start_ms + MIN_SESSION_MS)
}

fn rule_issue<'r>(rule: &'r RuleKind, window: &WindowTime<'_>) -> Option<&'r str> {
    match rule {
        RuleKind::PrivateApp { .. } => None,
        RuleKind::AppIssue { app_id, issue } => (app_id == window.app_id).then_some(issue),
        RuleKind::HostIssue { host, issue } => {
            let url_host = window.url?.split("://").nth(1)?.split('/').next()?;
            let host = host.to_ascii_lowercase();
            let url_host = url_host.to_ascii_lowercase();
            (url_host == host || url_host.ends_with(&format!(".{host}"))).then_some(issue)
        }
        RuleKind::TitleIssue { text, issue } => {
            let Title::Known { text: title } = window.title else {
                return None;
            };
            title
                .to_lowercase()
                .contains(&text.to_lowercase())
                .then_some(issue)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIN: i64 = 60_000;

    fn span(branch: &str, title: &str, start: i64, end: i64, prompts: &[i64]) -> ContextSpan {
        ContextSpan {
            source: AgentSource::Codex,
            session_id: "t1".into(),
            title: Some(title.into()),
            cwd: None,
            branch: Some(branch.into()),
            start_ms: start * MIN,
            end_ms: end * MIN,
            prompt_times: prompts.iter().map(|p| p * MIN).collect(),
        }
    }

    fn event(at: i64, branch: &str, prompt: bool) -> SessionEvent<'static> {
        SessionEvent {
            source: AgentSource::ClaudeCode,
            session_id: "s1",
            cwd: None,
            branch: Some(if branch == "a" { "a" } else { "b" }),
            at_ms: at * MIN,
            prompt,
        }
    }

    #[test]
    fn events_join_into_spans_until_a_long_pause_or_a_branch_change() {
        let mut spans = Vec::new();
        for (at, branch, prompt) in [
            (0, "a", true),
            (10, "a", false),
            (40, "a", true),
            (41, "b", false),
        ] {
            add_event(&mut spans, &event(at, branch, prompt));
        }

        let summary: Vec<_> = spans
            .iter()
            .map(|s| (s.start_ms / MIN, s.end_ms / MIN, s.prompt_times.len()))
            .collect();
        assert_eq!(summary, [(0, 10, 1), (40, 40, 1), (41, 41, 0)]);
    }

    #[test]
    fn a_session_the_user_typed_in_names_the_block_from_its_branch() {
        let sessions = [SessionContext {
            span: span("codex/int-177-batch-2", "Job runner batch 2", 0, 60, &[10]),
            commits: vec![(20 * MIN, "[INT-177] Move the stock sync".into())],
        }];

        let work = detect_work(0, 30 * MIN, &[], &sessions, &[], &[]).unwrap();

        assert_eq!(work.issue, "INT-177");
        assert_eq!(work.confidence, Confidence::High);
        assert!(work.evidence.contains(&Evidence::Commit {
            subject: "[INT-177] Move the stock sync".into()
        }));
    }

    #[test]
    fn a_commit_names_the_whole_session_not_only_its_own_stretch() {
        let sessions = [SessionContext {
            span: span(
                "claude/is61-uuid-keys-pg",
                "Order Search readiness",
                0,
                90,
                &[5],
            ),
            commits: vec![(80 * MIN, "[IS-61] UUID keys on PostgreSQL".into())],
        }];

        let work = detect_work(0, 15 * MIN, &[], &sessions, &[], &[]).unwrap();

        assert_eq!(work.issue, "IS-61");
        assert_eq!(work.confidence, Confidence::High);
    }

    #[test]
    fn a_window_title_beats_a_background_session() {
        let title = Title::Known {
            text: "IS-61 · YouTrack".into(),
        };
        let windows = [WindowTime {
            app_id: "app.id",
            app: "Dia",
            title: &title,
            url: None,
            duration_ms: 10 * MIN,
        }];
        let sessions = [SessionContext {
            span: span("claude/sel-54-history", "History", 0, 60, &[]),
            commits: Vec::new(),
        }];

        let work = detect_work(0, 30 * MIN, &windows, &sessions, &[], &[]).unwrap();

        // The other issue is close behind, so this is a suggestion.
        assert_eq!(work.issue, "IS-61");
        assert_eq!(work.confidence, Confidence::Medium);
    }

    #[test]
    fn a_short_address_among_unnamed_work_is_only_a_suggestion() {
        let testing = Title::Known {
            text: "Auth0 sign-in test".into(),
        };
        let ticket = Title::Known {
            text: "Ticket".into(),
        };
        let windows = [
            WindowTime {
                app_id: "app.id",
                app: "Dia",
                title: &testing,
                url: Some("https://acme.auth0.com/admin"),
                duration_ms: 14 * MIN,
            },
            WindowTime {
                app_id: "app.id",
                app: "Dia",
                title: &ticket,
                url: Some("https://youtrack.example/issue/INT-122"),
                duration_ms: MIN,
            },
        ];

        let work = detect_work(0, 15 * MIN, &windows, &[], &[], &[]).unwrap();

        assert_eq!(
            (work.issue.as_str(), work.confidence),
            ("INT-122", Confidence::Medium)
        );
    }

    #[test]
    fn a_browser_url_names_the_issue() {
        let title = Title::Known {
            text: "Pull request · GitHub".into(),
        };
        let windows = [WindowTime {
            app_id: "app.id",
            app: "Dia",
            title: &title,
            url: Some("https://acme.youtrack.cloud/issue/PD-260/one-sql"),
            duration_ms: 5 * MIN,
        }];

        let work = detect_work(0, 30 * MIN, &windows, &[], &[], &[]).unwrap();

        assert_eq!(work.issue, "PD-260");
        assert!(matches!(work.evidence[0], Evidence::Url { .. }));
    }

    #[test]
    fn only_a_background_session_gives_medium_confidence() {
        let sessions = [SessionContext {
            span: span("claude/sel-54-history", "History", 0, 60, &[]),
            commits: Vec::new(),
        }];

        let work = detect_work(0, 30 * MIN, &[], &sessions, &[], &[]).unwrap();

        assert_eq!(
            (work.issue.as_str(), work.confidence),
            ("SEL-54", Confidence::Medium)
        );
    }

    #[test]
    fn rules_match_hosts_with_subdomains_and_titles_without_case() {
        let title = Title::Known {
            text: "Weekly SYNC with Ann".into(),
        };
        let windows = [WindowTime {
            app_id: "com.microsoft.teams2",
            app: "Teams",
            title: &title,
            url: Some("https://app.orders.example.com/history"),
            duration_ms: 30 * MIN,
        }];
        let host = RuleKind::HostIssue {
            host: "orders.example.com".into(),
            issue: "SEL-54".into(),
        };
        let meeting = RuleKind::TitleIssue {
            text: "weekly sync".into(),
            issue: "SEL-21".into(),
        };

        let by_host =
            detect_work(0, 30 * MIN, &windows, &[], std::slice::from_ref(&host), &[]).unwrap();
        let by_title = detect_work(
            0,
            30 * MIN,
            &windows,
            &[],
            std::slice::from_ref(&meeting),
            &[],
        )
        .unwrap();

        assert_eq!(
            (by_host.issue.as_str(), by_host.confidence),
            ("SEL-54", Confidence::High)
        );
        assert_eq!(by_host.evidence, [Evidence::Rule { rule: host }]);
        assert_eq!(by_title.issue, "SEL-21");
    }

    #[test]
    fn title_rules_also_name_agent_sessions() {
        let sessions = [SessionContext {
            span: span(
                "claude/interesting-turing-a1e2c9",
                "Research Auth0 plans",
                0,
                30,
                &[5],
            ),
            commits: Vec::new(),
        }];
        let rule = RuleKind::TitleIssue {
            text: "auth0".into(),
            issue: "PD-275".into(),
        };

        let work = detect_work(0, 15 * MIN, &[], &sessions, &[rule], &[]).unwrap();

        assert_eq!(
            (work.issue.as_str(), work.confidence),
            ("PD-275", Confidence::High)
        );
    }

    #[test]
    fn nothing_named_means_no_detected_work() {
        let title = Title::Known {
            text: "Inbox".into(),
        };
        let windows = [WindowTime {
            app_id: "app.id",
            app: "Outlook",
            title: &title,
            url: None,
            duration_ms: MIN,
        }];

        assert_eq!(detect_work(0, MIN, &windows, &[], &[], &[]), None);
    }

    #[test]
    fn only_the_trackers_projects_count_as_issue_keys() {
        let windows = [WindowTime {
            app_id: "com.apple.Safari",
            app: "Safari",
            title: &Title::Known {
                text: "Review PR-12 for INT-290".into(),
            },
            url: None,
            duration_ms: 20 * MIN,
        }];
        let projects = ["INT".to_owned()];

        let work = detect_work(0, 30 * MIN, &windows, &[], &[], &projects).unwrap();

        assert_eq!(work.issue, "INT-290");
        assert!(work.others.is_empty());
    }

    #[test]
    fn helper_agent_spans_join_their_session_so_time_counts_once() {
        let parent = span("claude/int-290-x", "Push", 0, 30, &[1]);
        let mut helper = span("claude/int-290-x", "Push", 10, 40, &[]);
        helper.title = None;
        let mut other = span("main", "Other", 5, 6, &[]);
        other.session_id = "other".into();

        let joined = join_session_spans(vec![helper, parent, other]);

        assert_eq!(
            joined
                .iter()
                .map(|s| (s.session_id.as_str(), s.start_ms / MIN, s.end_ms / MIN))
                .collect::<Vec<_>>(),
            [("other", 5, 6), ("t1", 0, 40)]
        );
        assert_eq!(joined[1].title.as_deref(), Some("Push"));
    }
}
