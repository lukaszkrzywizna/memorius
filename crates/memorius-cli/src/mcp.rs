//! A small MCP server over stdio (newline-delimited JSON-RPC 2.0). It needs
//! only `initialize`, `ping`, `tools/list` and `tools/call`, so it is written
//! here instead of pulling in an SDK.

use std::io::{BufRead, Read, Write};
use std::path::Path;
use std::process::ExitCode;
use std::time::SystemTime;

use serde_json::{Value, json};

use crate::ops::{BUILD, Op, run, week_start};
use crate::time::today;

const SUPPORTED_VERSIONS: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];

const INSTRUCTIONS: &str = "Memorius records which apps and windows the user worked in and which \
issue each block of time was most likely about. To prepare worklogs: call memorius_unlogged for \
the period, propose one worklog per issue and day with memorius_propose_worklog, and stop. The \
user approves or rejects proposals in the Memorius app, and Memorius logs the approved ones in \
the tracker. Never log time yourself.";

/// When the `memorius` file on disk changed; a different value later means
/// it was updated while this server kept running the old code.
fn installed_at() -> Option<SystemTime> {
    std::env::current_exe()
        .and_then(std::fs::metadata)
        .and_then(|meta| meta.modified())
        .ok()
}

/// The longest JSON-RPC message accepted; real ones are a few kilobytes.
const MAX_MESSAGE: usize = 1 << 20;

pub fn serve(database: &Path) -> ExitCode {
    let started = installed_at();
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout();
    let mut bytes = Vec::new();
    loop {
        bytes.clear();
        // A message longer than the limit ends the server instead of
        // filling the memory.
        match (&mut stdin)
            .take(MAX_MESSAGE as u64 + 1)
            .read_until(b'\n', &mut bytes)
        {
            Ok(0) => break,
            Ok(read) if read <= MAX_MESSAGE => {}
            _ => return ExitCode::FAILURE,
        }
        let Ok(line) = std::str::from_utf8(&bytes) else {
            return ExitCode::FAILURE;
        };
        if line.trim().is_empty() {
            continue;
        }
        let stale = started.is_some() && installed_at() != started;
        let Some(reply) = handle(line, database, stale) else {
            continue;
        };
        if writeln!(stdout, "{reply}")
            .and_then(|()| stdout.flush())
            .is_err()
        {
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}

/// The JSON-RPC reply to one message, or `None` for notifications.
fn handle(line: &str, database: &Path, stale: bool) -> Option<Value> {
    let Ok(message) = serde_json::from_str::<Value>(line) else {
        return Some(error(&Value::Null, -32700, "parse error"));
    };
    let id = message.get("id")?.clone();
    let method = message
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let params = message.get("params").cloned().unwrap_or(Value::Null);
    let result = match method {
        "initialize" => {
            let requested = params.get("protocolVersion").and_then(Value::as_str);
            let version = requested
                .filter(|version| SUPPORTED_VERSIONS.contains(version))
                .unwrap_or(SUPPORTED_VERSIONS[0]);
            json!({
                "protocolVersion": version,
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "memorius",
                    "version": format!("{}+{BUILD}", env!("CARGO_PKG_VERSION")),
                },
                "instructions": INSTRUCTIONS,
            })
        }
        "ping" => json!({}),
        "tools/list" => json!({ "tools": tools() }),
        "tools/call" => call(&params, database, stale),
        _ => return Some(error(&id, -32601, &format!("method not found: {method}"))),
    };
    Some(json!({ "jsonrpc": "2.0", "id": id, "result": result }))
}

fn error(id: &Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

/// Tool errors are results with `isError`, so the agent can read and fix them.
fn call(params: &Value, database: &Path, stale: bool) -> Value {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let outcome = op(name, &args).and_then(|op| run(&op, database, crate::now_ms()));
    let mut reply = match outcome {
        Ok(value) => json!({
            "content": [{ "type": "text", "text": value.to_string() }],
            "structuredContent": value,
            "isError": false,
        }),
        Err(message) => json!({
            "content": [{ "type": "text", "text": message }],
            "isError": true,
        }),
    };
    if stale && let Some(content) = reply["content"].as_array_mut() {
        content.insert(0, json!({ "type": "text", "text": STALE_WARNING }));
    }
    reply
}

const STALE_WARNING: &str = "Warning: memorius was updated after this MCP server started, so \
these results come from the old version. Tell the user to restart this agent session (or its \
MCP servers) before proposing or logging worklogs.";

fn op(name: &str, args: &Value) -> Result<Op, String> {
    let text = |key: &str| args.get(key).and_then(Value::as_str).map(str::to_owned);
    let required = |key: &str| text(key).ok_or_else(|| format!("missing argument {key}"));
    let number = |key: &str| {
        args.get(key)
            .and_then(Value::as_i64)
            .ok_or_else(|| format!("missing number {key}"))
    };
    let today = today();
    Ok(match name {
        "memorius_day" => Op::Day {
            date: text("date").unwrap_or(today),
        },
        "memorius_week" => Op::Week {
            week: text("week").unwrap_or(today),
        },
        "memorius_unlogged" => Op::Unlogged {
            from: text("from").unwrap_or_else(|| week_start(&today)),
            to: text("to").unwrap_or(today),
        },
        "memorius_proposals" => Op::Proposals {
            from: text("from").unwrap_or_else(|| week_start(&today)),
            to: text("to").unwrap_or(today),
        },
        "memorius_propose_worklog" => Op::Propose {
            issue: required("issue")?,
            date: required("date")?,
            start: text("start"),
            end: text("end"),
            minutes: u32::try_from(number("minutes")?)
                .map_err(|_| "minutes must be positive".to_owned())?,
            text: required("text")?,
            work_type: text("workType"),
            agent: required("agent")?,
        },
        "memorius_status" => Op::Status,
        other => return Err(format!("unknown tool {other}")),
    })
}

fn tools() -> Value {
    let date = json!({ "type": "string", "description": "YYYY-MM-DD" });
    let range = json!({
        "type": "object",
        "properties": {
            "from": { "type": "string", "description": "YYYY-MM-DD (default: this week's Monday)" },
            "to": { "type": "string", "description": "YYYY-MM-DD (default: today), at most 31 days after from" },
        },
    });
    json!([
        {
            "name": "memorius_status",
            "description": "Which memorius build answers, its output and schema versions, and when the newest imported agent session changed. Check it when results look old.",
            "inputSchema": { "type": "object", "properties": {} },
            "annotations": { "readOnlyHint": true },
        },
        {
            "name": "memorius_day",
            "description": "One day: work spans (suggested issue, evidence), activity blocks (apps, window titles, URLs), agent session blocks (title, branch, prompts, issue), the user's own git commits (time, subject, repository, branch), logged, proposed and unlogged minutes, and that day's proposals.",
            "inputSchema": { "type": "object", "properties": { "date": date } },
            "annotations": { "readOnlyHint": true },
        },
        {
            "name": "memorius_week",
            "description": "The week timesheet: per day active, logged, proposed and unlogged minutes; per issue and day logged, proposed or detected minutes.",
            "inputSchema": { "type": "object", "properties": { "week": { "type": "string", "description": "ISO week 2026-W41 or a date in it (default: this week)" } } },
            "annotations": { "readOnlyHint": true },
        },
        {
            "name": "memorius_unlogged",
            "description": "Days with at least 5 minutes of the user's work that no YouTrack work item or proposal covers. Per day: work and unlogged minutes, what is already logged (from YouTrack) and proposed per issue, and the evidence: work spans (a suggested issue with its support and other issues, minutes including agent time up to 30 minutes after the user's prompt) and app blocks. Trackers log per issue and day, so propose one worklog per issue and day. Judge the issues yourself; memorius_day has more detail. Every proposal waits for the user's approval. Start here to prepare worklogs.",
            "inputSchema": range,
            "annotations": { "readOnlyHint": true },
        },
        {
            "name": "memorius_proposals",
            "description": "Worklog proposals in a range of days, in every state.",
            "inputSchema": range,
            "annotations": { "readOnlyHint": true },
        },
        {
            "name": "memorius_propose_worklog",
            "description": "Proposes one worklog for one issue on one day for the user to approve in Memorius. Proposing the same issue and day again while it waits replaces its minutes and text. Do not log it in YouTrack until it is approved.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "issue": { "type": "string", "description": "YouTrack issue key, for example INT-290" },
                    "date": date,
                    "start": { "type": "string", "description": "Optional: the first moment of the work, local time 2026-10-09T13:31" },
                    "end": { "type": "string", "description": "Optional: the last moment of the work" },
                    "minutes": { "type": "integer", "minimum": 1, "description": "Time to log that day; at most the time between start and end" },
                    "text": { "type": "string", "description": "Worklog description: what was done" },
                    "workType": { "type": "string", "description": "One of the user's YouTrack work types (Memorius refuses others once it knows them); leave it out when none fits" },
                    "agent": { "type": "string", "description": "Who proposes, for example Claude Code" },
                },
                "required": ["issue", "date", "minutes", "text", "agent"],
            },
        },
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn database() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("m.db");
        memorius_core::store::Store::open(&path).unwrap();
        (dir, path)
    }

    fn reply(line: &str, path: &Path) -> Value {
        handle(line, path, false).unwrap()
    }

    #[test]
    fn initializes_with_a_supported_version_and_ignores_notifications() {
        let (_dir, path) = database();
        let init = reply(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26"}}"#,
            &path,
        );

        assert_eq!(init["result"]["protocolVersion"], "2025-03-26");
        assert!(
            handle(
                r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
                &path,
                false
            )
            .is_none()
        );
        assert_eq!(
            reply(r#"{"jsonrpc":"2.0","id":2,"method":"nope"}"#, &path)["error"]["code"],
            -32601
        );
    }

    #[test]
    fn proposes_a_worklog_for_the_users_approval() {
        let (_dir, path) = database();
        let propose = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"memorius_propose_worklog","arguments":{"issue":"int-290","date":"2026-10-09","start":"2026-10-09T09:00","end":"2026-10-09T10:00","minutes":50,"text":"Skip unchanged customers","agent":"Claude Code"}}}"#;

        let proposed = reply(propose, &path);
        let result = &proposed["result"]["structuredContent"];
        assert_eq!(result["created"], true);
        assert_eq!(result["proposal"]["issue"], "INT-290");
        assert_eq!(result["proposal"]["state"], "proposed");
    }

    #[test]
    fn lists_every_tool_with_a_schema() {
        let tools = tools();
        let names: Vec<&str> = tools
            .as_array()
            .unwrap()
            .iter()
            .map(|tool| tool["name"].as_str().unwrap())
            .collect();

        assert_eq!(names.len(), 6);
        for name in names {
            assert!(op(name, &json!({ "issue": "A-1", "date": "d", "start": "s", "end": "e", "minutes": 1, "text": "t", "agent": "a", "id": 1, "workItemId": "w", "error": "x" })).is_ok(), "{name}");
        }
    }

    #[test]
    fn a_stale_server_warns_before_its_results() {
        let (_dir, path) = database();
        let status = r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"memorius_status","arguments":{}}}"#;

        let fresh = handle(status, &path, false).unwrap();
        let stale = handle(status, &path, true).unwrap();

        assert_eq!(fresh["result"]["structuredContent"]["build"], BUILD);
        assert_eq!(fresh["result"]["content"].as_array().unwrap().len(), 1);
        assert_eq!(stale["result"]["content"][0]["text"], STALE_WARNING);
    }

    #[test]
    fn unlogged_counts_agent_time_up_to_30_minutes_after_a_phone_prompt() {
        use memorius_core::model::{AgentSource, ContextSpan, SessionCursor};
        use memorius_core::store::{SessionImport, Store};

        let (_dir, path) = database();
        let at = |clock: &str| crate::time::parse_time(&format!("2026-10-09T{clock}")).unwrap();
        // One phone prompt at 09:01, then the agent works alone until 10:00;
        // 09:01–09:31 counts.
        Store::open(&path)
            .unwrap()
            .save_session_import(&SessionImport {
                path: "s.jsonl",
                source: AgentSource::ClaudeCode,
                modified_ms: 1,
                cursor: &SessionCursor::default(),
                continues_span: None,
                file: "/tmp/s1.jsonl",
                prompt_lines: &[],
                spans: &[ContextSpan {
                    source: AgentSource::ClaudeCode,
                    session_id: "s".into(),
                    title: Some("Push".into()),
                    cwd: None,
                    branch: Some("claude/int-290-push".into()),
                    start_ms: at("09:00"),
                    end_ms: at("10:00"),
                    prompt_times: vec![at("09:01")],
                }],
            })
            .unwrap();
        let request = r#"{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"memorius_unlogged","arguments":{"from":"2026-10-09","to":"2026-10-09"}}}"#;

        let result = reply(request, &path)["result"]["structuredContent"].clone();

        assert_eq!(result["version"], 9);
        let day = &result["days"][0];
        let span = &day["work"][0];
        assert_eq!(
            (
                day["unloggedMinutes"].as_u64(),
                span["issue"].as_str(),
                span["minutes"].as_u64(),
                span["agentMinutes"].as_u64(),
            ),
            (Some(30), Some("INT-290"), Some(30), Some(29))
        );
    }
}
