//! Starting the user's agent CLI for a run: the range, the prompt, the
//! command line with only the tools the run needs, and the agent's answer.

use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use jiff::tz::TimeZone;
use jiff::{Timestamp, Zoned};
use memorius_core::agent::{AgentKind, AgentSettings, RunRange};
use serde_json::{Value, json};

const HOUR_MS: i64 = 3_600_000;
/// `memorius_unlogged` accepts at most 31 days.
const MONTH_MS: i64 = 30 * 24 * HOUR_MS;
const FIRST_RUN_MS: i64 = 48 * HOUR_MS;

/// The Memorius tools a run may call.
const PROPOSAL_TOOLS: [&str; 4] = [
    "memorius_unlogged",
    "memorius_day",
    "memorius_proposals",
    "memorius_propose_worklog",
];

/// The range a proposals run covers, ending at `to_ms`.
pub fn run_range(range: RunRange, to_ms: i64, last_end_ms: Option<i64>) -> (i64, i64) {
    let back = match range {
        RunRange::SinceLastRun => {
            last_end_ms.map_or(FIRST_RUN_MS, |end| (to_ms - end).clamp(0, MONTH_MS))
        }
        RunRange::Day => 24 * HOUR_MS,
        RunRange::TwoDays => FIRST_RUN_MS,
        RunRange::Week => 7 * 24 * HOUR_MS,
        RunRange::Month => MONTH_MS,
    };
    (to_ms - back, to_ms)
}

fn local_date(ms: i64) -> String {
    Timestamp::from_millisecond(ms)
        .map(|at| at.to_zoned(TimeZone::system()).date().to_string())
        .unwrap_or_default()
}

/// Whether a daily job at `time` (`HH:MM`) is due: enabled, past its time
/// today, and not done on `last_date` (the local date it last ran).
pub fn daily_due(enabled: bool, time: &str, now: &Zoned, last_date: Option<&str>) -> bool {
    let today = now.date().to_string();
    let Some((hour, minute)) = time
        .split_once(':')
        .and_then(|(h, m)| Some((h.parse::<i8>().ok()?, m.parse::<i8>().ok()?)))
    else {
        return false;
    };
    let due = now
        .with()
        .hour(hour)
        .minute(minute)
        .second(0)
        .subsec_nanosecond(0)
        .build()
        .is_ok_and(|at| *now >= at);
    enabled && due && last_date != Some(today.as_str())
}

/// What the agent is told: judge the evidence and propose worklogs for the
/// range, not copy spans. Every proposal waits for the user's approval.
pub fn prompt(
    settings: &AgentSettings,
    (from_ms, to_ms): (i64, i64),
    work_types: &[String],
) -> String {
    let name = match settings.agent {
        AgentKind::ClaudeCode => "Claude Code",
        AgentKind::Codex => "Codex",
        AgentKind::Custom => "Memorius agent",
    };
    let lookup = tracker_tools(&settings.lookup_tools);
    let task = format!(
        "You prepare the user's worklogs in Memorius from their recorded activity \
             from {from} to {to} (local dates). The tracker logs time per issue and day, \
             with no start or end time.\n\
             1. Call memorius_unlogged for that range. Per day it gives the unlogged \
             minutes, what is already logged in the tracker and proposed per issue, and the \
             evidence: work spans with a suggested issue and other possible issues, and \
             app blocks.\n\
             2. Look closer where you need to: memorius_day shows a day's windows, \
             addresses, agent sessions (titles, branches, prompts) and commits.{lookup}\n\
             3. Decide which issues each day's work belongs to and how many minutes \
             each. The suggested issues are only Memorius's first guess. Leave out what \
             was not work on an issue and what is already logged.\n\
             4. For each issue and day call memorius_propose_worklog once with the \
             issue, the date, start and end (the first and last moment of that work), \
             the minutes to log, a one-line description of what was done that day, \
             written as a worklog, {types}and agent \"{name}\". The minutes of one day \
             must not exceed its unlogged minutes.\n\
             Never log anything: the user approves each proposal in Memorius. End with \
             one short line of plain text (no Markdown): how many proposals you made and \
             what you left out and why.",
        types = if work_types.is_empty() {
            String::new()
        } else {
            format!(
                "a workType from the tracker's work types ({}) or none when none fits, ",
                work_types.join(", ")
            )
        },
        from = local_date(from_ms),
        to = local_date(to_ms - 1),
        lookup = if lookup.is_empty() {
            String::new()
        } else {
            format!(
                " Use {} to read an issue's summary or to find the issue for work \
                     with no key.",
                lookup
                    .iter()
                    .map(|(_, tool)| *tool)
                    .collect::<Vec<_>>()
                    .join(" and ")
            )
        },
    );
    let instructions = settings.instructions.trim();
    if instructions.is_empty() {
        task
    } else {
        format!("{task}\n\nThe user's instructions:\n{instructions}")
    }
}

/// `server/tool` names, comma separated, as `(server, tool)` pairs.
fn tracker_tools(list: &str) -> Vec<(&str, &str)> {
    list.split(',')
        .filter_map(|item| item.trim().split_once('/'))
        .filter(|(server, tool)| !server.is_empty() && !tool.is_empty())
        .collect()
}

/// A program and its arguments; the prompt is one argument, never parsed
/// by a shell (a custom command also gets it in `MEMORIUS_PROMPT`).
#[derive(Debug, PartialEq, Eq)]
pub struct AgentCommand {
    pub program: String,
    pub args: Vec<String>,
    pub prompt_env: Option<String>,
}

/// The command line of a run; `cli` is the `memorius` executable the agent
/// starts as its MCP server (the one inside the app, so PATH does not matter).
pub fn command(settings: &AgentSettings, prompt: &str, cli: &str) -> AgentCommand {
    // The read-only tracker lookups a run may call; their credentials stay
    // in the agent's config.
    let memorius = &PROPOSAL_TOOLS[..];
    let tracker = tracker_tools(&settings.lookup_tools);
    match settings.agent {
        AgentKind::ClaudeCode => claude(prompt, cli, memorius, &tracker),
        AgentKind::Codex => codex(prompt, cli, memorius, &tracker),
        AgentKind::Custom => {
            // No shell parses the line: the prompt holds text from the
            // tracker, so `{prompt}` becomes exactly one argument. A Windows
            // `.cmd` still runs through cmd.exe; the standard library escapes
            // its arguments and refuses ones it cannot (CVE-2024-24576).
            let mut words = split_command(&settings.custom_command)
                .into_iter()
                .map(|word| word.replace("{prompt}", prompt));
            AgentCommand {
                program: words.next().unwrap_or_default(),
                args: words.collect(),
                prompt_env: Some(prompt.into()),
            }
        }
    }
}

/// A command line split into words at spaces, with double quotes keeping
/// spaces inside a word.
fn split_command(line: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quoted = false;
    for character in line.chars() {
        match character {
            '"' => quoted = !quoted,
            ' ' | '\t' if !quoted => {
                if !word.is_empty() {
                    words.push(std::mem::take(&mut word));
                }
            }
            other => word.push(other),
        }
    }
    if !word.is_empty() {
        words.push(word);
    }
    words
}

fn claude(prompt: &str, cli: &str, memorius: &[&str], tracker: &[(&str, &str)]) -> AgentCommand {
    // Memorius is always passed, so the user need not configure it; with no
    // tracker tool the user's other MCP servers are not even started.
    let mut args = vec![
        "-p".into(),
        prompt.into(),
        "--output-format".into(),
        "json".into(),
        "--no-session-persistence".into(),
        "--tools".into(),
        String::new(),
        "--mcp-config".into(),
        json!({ "mcpServers": { "memorius": { "command": cli, "args": ["mcp"] } } }).to_string(),
    ];
    if tracker.is_empty() {
        args.push("--strict-mcp-config".into());
    }
    let allowed: Vec<String> = memorius
        .iter()
        .map(|tool| format!("mcp__memorius__{tool}"))
        .chain(
            tracker
                .iter()
                .map(|(server, tool)| format!("mcp__{server}__{tool}")),
        )
        .collect();
    args.extend(["--allowedTools".into(), allowed.join(",")]);
    AgentCommand {
        program: "claude".into(),
        args,
        prompt_env: None,
    }
}

fn codex(prompt: &str, cli: &str, memorius: &[&str], tracker: &[(&str, &str)]) -> AgentCommand {
    let mut servers: Vec<(&str, Vec<&str>)> = vec![("memorius", memorius.to_vec())];
    for (server, tool) in tracker {
        match servers.iter_mut().find(|(name, _)| name == server) {
            Some((_, tools)) => tools.push(tool),
            None => servers.push((server, vec![tool])),
        }
    }
    let mut args: Vec<String> = [
        "exec",
        "--ephemeral",
        "--json",
        "--skip-git-repo-check",
        "--sandbox",
        "read-only",
        "-c",
        "mcp_servers.memorius.args=[\"mcp\"]",
    ]
    .map(String::from)
    .into();
    // A TOML literal string: Windows paths keep their backslashes.
    args.extend(["-c".into(), format!("mcp_servers.memorius.command='{cli}'")]);
    // Without tracker tools the user's other MCP servers stay out; with them
    // the tracker comes from the user's config, and tools that are not
    // approved here ask for approval, which `exec` refuses.
    if tracker.is_empty() {
        args.push("--ignore-user-config".into());
    }
    for (server, tools) in servers {
        let list = tools
            .iter()
            .map(|tool| format!("\"{tool}\""))
            .collect::<Vec<_>>()
            .join(",");
        args.extend([
            "-c".into(),
            format!("mcp_servers.{server}.enabled_tools=[{list}]"),
            "-c".into(),
            format!("mcp_servers.{server}.default_tools_approval_mode=\"approve\""),
        ]);
    }
    args.push(prompt.into());
    AgentCommand {
        program: "codex".into(),
        args,
        prompt_env: None,
    }
}

/// The agent CLI's own name, to check that it is installed.
pub fn program_name(settings: &AgentSettings) -> Option<&'static str> {
    match settings.agent {
        AgentKind::ClaudeCode => Some("claude"),
        AgentKind::Codex => Some("codex"),
        AgentKind::Custom => None,
    }
}

/// Starts the agent with `path` as its `PATH` (a GUI app has no login-shell
/// PATH, and the agent starts `memorius mcp` itself).
pub fn spawn(command: &AgentCommand, path: Option<&str>) -> std::io::Result<Child> {
    let mut process = Command::new(&command.program);
    process
        .args(&command.args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // The subscription login pays, never an API key from the environment.
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("OPENAI_API_KEY");
    if let Some(path) = path {
        process.env("PATH", path);
    }
    if let Some(prompt) = &command.prompt_env {
        process.env("MEMORIUS_PROMPT", prompt);
    }
    process.spawn()
}

/// What a finished agent process printed.
pub struct Finished {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

/// The most an agent's output keeps: the result is at its end.
const OUTPUT_TAIL: usize = 4 << 20;

/// Reads the pipe to its end and keeps only the last `OUTPUT_TAIL` bytes, so
/// a runaway command cannot fill the memory.
fn tail(pipe: Option<Box<dyn Read + Send>>) -> String {
    let mut kept = Vec::new();
    if let Some(mut pipe) = pipe {
        let mut chunk = vec![0u8; 64 * 1024];
        while let Ok(read @ 1..) = pipe.read(&mut chunk) {
            kept.extend_from_slice(&chunk[..read]);
            // Trim at twice the limit, so the copy happens rarely.
            if kept.len() > 2 * OUTPUT_TAIL {
                kept.drain(..kept.len() - OUTPUT_TAIL);
            }
        }
    }
    let from = kept.len().saturating_sub(OUTPUT_TAIL);
    String::from_utf8_lossy(&kept[from..]).into_owned()
}

/// Waits for `child`, killing it after `timeout` or when `stop` says so.
pub fn wait(
    mut child: Child,
    timeout: Duration,
    stop: &dyn Fn() -> bool,
) -> Result<Finished, String> {
    let read = |pipe: Option<Box<dyn Read + Send>>| std::thread::spawn(move || tail(pipe));
    let stdout = read(child.stdout.take().map(|pipe| Box::new(pipe) as _));
    let stderr = read(child.stderr.take().map(|pipe| Box::new(pipe) as _));
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
            break status;
        }
        let ended = if stop() {
            Some("Stopped")
        } else if started.elapsed() > timeout {
            Some("The agent did not finish in time")
        } else {
            None
        };
        if let Some(reason) = ended {
            let _ = child.kill();
            let _ = child.wait();
            return Err(reason.into());
        }
        std::thread::sleep(Duration::from_millis(250));
    };
    Ok(Finished {
        success: status.success(),
        stdout: stdout.join().unwrap_or_default(),
        stderr: stderr.join().unwrap_or_default(),
    })
}

/// The agent's last message, or why the run failed.
pub fn outcome(agent: AgentKind, finished: &Finished) -> Result<String, String> {
    let parsed = match agent {
        AgentKind::ClaudeCode => claude_outcome(&finished.stdout),
        AgentKind::Codex => codex_outcome(&finished.stdout),
        AgentKind::Custom => last_line(&finished.stdout).map(Ok),
    };
    match parsed {
        Some(Ok(message)) if finished.success => Ok(message),
        Some(Err(error)) => Err(error),
        _ => Err(last_line(&finished.stderr)
            .or_else(|| last_line(&finished.stdout))
            .unwrap_or_else(|| "The agent stopped without an answer".into())),
    }
}

fn claude_outcome(stdout: &str) -> Option<Result<String, String>> {
    let value: Value = serde_json::from_str(stdout.trim()).ok()?;
    let result = value["result"]
        .as_str()
        .unwrap_or_default()
        .trim()
        .to_owned();
    Some(if value["is_error"].as_bool() == Some(true) {
        Err(if result.is_empty() {
            value["subtype"].as_str().unwrap_or("error").to_owned()
        } else {
            result
        })
    } else {
        Ok(result)
    })
}

fn codex_outcome(stdout: &str) -> Option<Result<String, String>> {
    let mut answer = None;
    for value in stdout
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
    {
        match value["type"].as_str() {
            Some("item.completed") if value["item"]["type"] == "agent_message" => {
                answer = value["item"]["text"]
                    .as_str()
                    .map(|text| Ok(text.trim().to_owned()));
            }
            Some("turn.failed") => {
                return Some(Err(value["error"]["message"]
                    .as_str()
                    .unwrap_or("The agent failed")
                    .to_owned()));
            }
            Some("error") => {
                answer = Some(Err(value["message"]
                    .as_str()
                    .unwrap_or("The agent failed")
                    .to_owned()));
            }
            _ => {}
        }
    }
    answer
}

fn last_line(text: &str) -> Option<String> {
    text.lines()
        .rev()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(|line| line.chars().take(500).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY_MS: i64 = 24 * HOUR_MS;

    #[test]
    fn since_last_run_starts_where_the_last_run_ended_within_a_month() {
        let now = 100 * DAY_MS;
        assert_eq!(
            run_range(RunRange::SinceLastRun, now, None).0,
            now - 2 * DAY_MS
        );
        assert_eq!(
            run_range(RunRange::SinceLastRun, now, Some(now - 3 * DAY_MS)).0,
            now - 3 * DAY_MS
        );
        assert_eq!(
            run_range(RunRange::SinceLastRun, now, Some(now - 90 * DAY_MS)).0,
            now - 30 * DAY_MS
        );
        assert_eq!(
            run_range(RunRange::Week, now, None),
            (now - 7 * DAY_MS, now)
        );
    }

    #[test]
    fn the_daily_run_is_due_once_a_day_after_its_time() {
        let at = |clock: &str| {
            format!("2026-10-16T{clock}:00[Europe/Warsaw]")
                .parse::<Zoned>()
                .unwrap()
        };

        assert!(!daily_due(true, "08:00", &at("07:59"), None));
        assert!(daily_due(true, "08:00", &at("08:00"), Some("2026-10-15")));
        assert!(!daily_due(true, "08:00", &at("09:00"), Some("2026-10-16")));
        assert!(!daily_due(false, "08:00", &at("09:00"), None));
    }

    #[test]
    fn a_proposals_run_gives_claude_only_the_memorius_proposal_tools() {
        let settings = AgentSettings {
            lookup_tools: String::new(),
            ..AgentSettings::default()
        };
        let command = command(&settings, "go", "memorius");

        assert_eq!(command.program, "claude");
        assert!(command.args.contains(&"--strict-mcp-config".to_owned()));
        assert!(
            command
                .args
                .contains(&"--no-session-persistence".to_owned())
        );
        let allowed = command
            .args
            .iter()
            .skip_while(|arg| *arg != "--allowedTools")
            .nth(1)
            .unwrap();
        assert_eq!(
            allowed,
            "mcp__memorius__memorius_unlogged,mcp__memorius__memorius_day,\
             mcp__memorius__memorius_proposals,mcp__memorius__memorius_propose_worklog"
        );
    }

    #[test]
    fn a_proposals_run_may_read_issues_and_follows_the_users_instructions() {
        let settings = AgentSettings {
            instructions: "Round to 15 minutes.".into(),
            lookup_tools: "youtrack/get_issue,youtrack/search_issues".into(),
            ..AgentSettings::default()
        };

        let command = command(&settings, "go", "memorius");
        let text = prompt(&settings, (0, 86_400_000), &["Development".to_owned()]);

        assert!(!command.args.contains(&"--strict-mcp-config".to_owned()));
        assert!(command.args.iter().any(|arg| arg.ends_with(
            "mcp__memorius__memorius_propose_worklog,mcp__youtrack__get_issue,mcp__youtrack__search_issues"
        )));
        assert!(text.contains("Use get_issue and search_issues"));
        assert!(text.contains("the tracker's work types (Development)"));
        assert!(text.ends_with("The user's instructions:\nRound to 15 minutes."));
    }

    #[test]
    fn codex_gets_the_users_tracker_lookups_from_its_own_config() {
        let codex = AgentSettings {
            agent: AgentKind::Codex,
            lookup_tools: "youtrack/get_issue".into(),
            ..AgentSettings::default()
        };

        let command = command(&codex, "go", "memorius");

        assert!(!command.args.contains(&"--ignore-user-config".to_owned()));
        assert!(
            command
                .args
                .contains(&"mcp_servers.youtrack.enabled_tools=[\"get_issue\"]".to_owned())
        );
        assert_eq!(command.args.last().map(String::as_str), Some("go"));
    }

    #[test]
    fn a_custom_command_gets_the_prompt_as_one_argument_even_when_quoted() {
        let custom = AgentSettings {
            agent: AgentKind::Custom,
            custom_command: r#"printf %s "{prompt}""#.into(),
            ..AgentSettings::default()
        };
        let command = command(&custom, "it's $HOME; `x`", "memorius");

        let finished = wait(
            spawn(&command, None).unwrap(),
            Duration::from_secs(10),
            &|| false,
        )
        .unwrap();

        assert_eq!(
            outcome(AgentKind::Custom, &finished),
            Ok("it's $HOME; `x`".into())
        );
    }

    #[test]
    fn a_custom_command_splits_at_spaces_outside_double_quotes() {
        assert_eq!(
            split_command(r#""C:\Program Files\agent.exe" --prompt {prompt}  -q"#),
            ["C:\\Program Files\\agent.exe", "--prompt", "{prompt}", "-q"]
        );
    }

    #[test]
    fn a_run_is_stopped_on_request() {
        let custom = AgentSettings {
            agent: AgentKind::Custom,
            custom_command: "sleep 30".into(),
            ..AgentSettings::default()
        };
        let child = spawn(&command(&custom, "", "memorius"), None).unwrap();

        assert_eq!(
            wait(child, Duration::from_secs(10), &|| true).err(),
            Some("Stopped".into())
        );
    }

    #[test]
    fn answers_and_errors_are_read_from_each_agents_output() {
        let finished = |stdout: &str, success| Finished {
            success,
            stdout: stdout.into(),
            stderr: "boom".into(),
        };

        assert_eq!(
            outcome(
                AgentKind::ClaudeCode,
                &finished(r#"{"result":" 2 proposals ","is_error":false}"#, true)
            ),
            Ok("2 proposals".into())
        );
        assert_eq!(
            outcome(
                AgentKind::ClaudeCode,
                &finished(
                    r#"{"result":"","is_error":true,"subtype":"error_max_turns"}"#,
                    false
                )
            ),
            Err("error_max_turns".into())
        );
        let codex = "{\"type\":\"item.completed\",\"item\":{\"type\":\"agent_message\",\"text\":\"1 logged\"}}\n{\"type\":\"turn.completed\"}";
        assert_eq!(
            outcome(AgentKind::Codex, &finished(codex, true)),
            Ok("1 logged".into())
        );
        assert_eq!(
            outcome(AgentKind::Codex, &finished("", false)),
            Err("boom".into())
        );
    }
}
