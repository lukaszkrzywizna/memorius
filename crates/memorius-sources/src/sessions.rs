//! Claude Code and Codex session files. Each file is read from where the last
//! read stopped, one line at a time, so even multi-gigabyte histories are
//! never held in memory.

use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;

use memorius_core::model::{AgentSource, SessionCursor};
use serde::Deserialize;
use serde::de::IgnoredAny;

/// Replayed history in a forked Codex thread keeps its original completion
/// time; lines whose completion time is further than this from the line time
/// are copies, not new work.
const REPLAY_TOLERANCE_MS: i64 = 60_000;

/// A line longer than this keeps the read buffer large; shrink it afterwards.
const BUFFER_KEEP_BYTES: usize = 1 << 20;

/// One moment of session activity. `prompt` marks a message the user typed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionEvent {
    pub at_ms: i64,
    pub prompt: bool,
    /// Where the event's line starts in the file, to read a prompt's text
    /// again on demand.
    pub line_offset: u64,
}

/// Reads the lines added since `state.offset`, updates `state`, and calls
/// `on_event` for each event with the file state at that line. A partially
/// written last line is left for the next read.
pub fn read_new_events(
    path: &Path,
    source: AgentSource,
    state: &mut SessionCursor,
    mut on_event: impl FnMut(SessionEvent, &SessionCursor),
) -> std::io::Result<()> {
    let mut file = File::open(path)?;
    if file.metadata()?.len() < state.offset {
        // The file was replaced; read it again from the start.
        *state = SessionCursor::default();
    }
    file.seek(SeekFrom::Start(state.offset))?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    loop {
        line.clear();
        let read = reader.read_line(&mut line)?;
        if read == 0 || !line.ends_with('\n') {
            break;
        }
        let line_offset = state.offset;
        state.offset += read as u64;
        let event = match source {
            AgentSource::ClaudeCode => claude_line(&line, state),
            AgentSource::Codex => codex_line(&line, state),
        };
        if let Some(event) = event {
            on_event(
                SessionEvent {
                    line_offset,
                    ..event
                },
                state,
            );
        }
        if line.capacity() > BUFFER_KEEP_BYTES {
            line = String::new();
        }
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClaudeLine {
    #[serde(rename = "type")]
    kind: Option<String>,
    timestamp: Option<String>,
    session_id: Option<String>,
    cwd: Option<String>,
    git_branch: Option<String>,
    custom_title: Option<String>,
    tool_use_result: Option<IgnoredAny>,
    #[serde(default)]
    is_sidechain: bool,
    #[serde(default)]
    is_meta: bool,
    message: Option<ClaudeMessage>,
}

#[derive(Deserialize)]
struct ClaudeMessage {
    content: Option<serde_json::Value>,
}

fn claude_line(line: &str, state: &mut SessionCursor) -> Option<SessionEvent> {
    let parsed: ClaudeLine = serde_json::from_str(line).ok()?;
    if parsed.kind.as_deref() == Some("custom-title") {
        state.title = parsed.custom_title.or(state.title.take());
    }
    state.session_id = parsed.session_id.or(state.session_id.take());
    state.cwd = parsed.cwd.or(state.cwd.take());
    state.branch = parsed
        .git_branch
        .filter(|branch| !branch.is_empty() && branch != "HEAD")
        .or(state.branch.take());
    let at_ms = timestamp_ms(parsed.timestamp.as_deref()?)?;
    let prompt = parsed.kind.as_deref() == Some("user")
        && parsed.tool_use_result.is_none()
        && !parsed.is_sidechain
        && !parsed.is_meta
        && parsed
            .message
            .and_then(|message| message.content)
            .is_some_and(|content| from_user(&content));
    Some(SessionEvent {
        at_ms,
        prompt,
        line_offset: 0,
    })
}

#[derive(Deserialize)]
struct CodexLine {
    #[serde(rename = "type")]
    kind: Option<String>,
    timestamp: Option<String>,
    payload: Option<CodexPayload>,
}

#[derive(Deserialize)]
struct CodexPayload {
    #[serde(rename = "type")]
    kind: Option<String>,
    id: Option<String>,
    cwd: Option<String>,
    item: Option<CodexItem>,
    thread_source: Option<String>,
    source: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct CodexItem {
    #[serde(rename = "type")]
    kind: Option<String>,
    completed_at_ms: Option<i64>,
    content: Option<serde_json::Value>,
}

fn codex_line(line: &str, state: &mut SessionCursor) -> Option<SessionEvent> {
    let parsed: CodexLine = serde_json::from_str(line).ok()?;
    let at_ms = timestamp_ms(parsed.timestamp.as_deref()?)?;
    let payload = parsed.payload?;
    match parsed.kind.as_deref() {
        Some("session_meta") => {
            let parent = payload
                .source
                .as_ref()
                .and_then(|source| source.pointer("/subagent/thread_spawn/parent_thread_id"))
                .and_then(serde_json::Value::as_str);
            match payload.thread_source.as_deref() {
                // Automatic review threads are not the user's work.
                Some("guardian_review") => return None,
                Some("subagent") => {
                    state.subagent = true;
                    state.session_id = parent.map(str::to_owned).or(payload.id);
                }
                _ => state.session_id = payload.id.or(state.session_id.take()),
            }
            state.cwd = payload.cwd.or(state.cwd.take());
        }
        Some("turn_context") => state.cwd = payload.cwd.or(state.cwd.take()),
        _ => {}
    }
    let item = payload
        .item
        .filter(|_| payload.kind.as_deref() == Some("item_completed"));
    if let Some(completed) = item.as_ref().and_then(|item| item.completed_at_ms)
        && (completed - at_ms).abs() > REPLAY_TOLERANCE_MS
    {
        return None;
    }
    let prompt = !state.subagent
        && item.is_some_and(|item| {
            item.kind.as_deref() == Some("UserMessage")
                && item.content.as_ref().is_some_and(from_user)
        });
    Some(SessionEvent {
        at_ms,
        prompt,
        line_offset: 0,
    })
}

/// The text of the prompt whose line starts at `offset`, without the blocks
/// tools inject; `None` when the line is gone or holds no typed text.
/// Read on demand and never stored.
pub fn prompt_text(
    path: &Path,
    source: AgentSource,
    offset: u64,
) -> std::io::Result<Option<String>> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(offset))?;
    let mut line = String::new();
    BufReader::new(file).read_line(&mut line)?;
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else {
        return Ok(None);
    };
    let content = match source {
        AgentSource::ClaudeCode => value.pointer("/message/content"),
        AgentSource::Codex => value.pointer("/payload/item/content"),
    };
    let texts: Vec<&str> = match content {
        Some(serde_json::Value::String(text)) => vec![text.as_str()],
        Some(serde_json::Value::Array(parts)) => parts
            .iter()
            .filter_map(|part| part.get("text").and_then(serde_json::Value::as_str))
            .collect(),
        _ => Vec::new(),
    };
    let text = texts
        .iter()
        .map(|text| strip_injected(text))
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    Ok((!text.is_empty()).then_some(text))
}

/// The text after the leading `<tag>…</tag>` blocks that tools inject.
fn strip_injected(text: &str) -> &str {
    let mut rest = text.trim_start();
    while let Some(tag) = rest.strip_prefix('<') {
        let name: String = tag
            .chars()
            .take_while(|char| char.is_ascii_alphanumeric() || matches!(char, '-' | '_'))
            .collect();
        if name.is_empty() {
            break;
        }
        let close = format!("</{name}>");
        let Some(end) = rest.find(&close) else {
            return "";
        };
        rest = rest[end + close.len()..].trim_start();
    }
    rest.trim_end()
}

/// Whether a message holds something the user typed or attached, apart
/// from blocks that tools inject (`<heartbeat>`, `<task-notification>`,
/// `<environment_context>`, …). Content is a string or a list of parts.
fn from_user(content: &serde_json::Value) -> bool {
    match content {
        serde_json::Value::String(text) => typed(text),
        serde_json::Value::Array(parts) => {
            parts.iter().any(
                |part| match part.get("type").and_then(serde_json::Value::as_str) {
                    Some("text" | "input_text") => part
                        .get("text")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(typed),
                    Some("image" | "input_image" | "localImage") => true,
                    _ => false,
                },
            )
        }
        _ => false,
    }
}

/// Whether any text is left after the leading injected blocks.
fn typed(text: &str) -> bool {
    !strip_injected(text).is_empty()
}

/// Codex thread names by session id, from `~/.codex/session_index.jsonl`
/// (later lines rename earlier ones).
pub fn codex_thread_names(index: &str) -> std::collections::HashMap<String, String> {
    #[derive(Deserialize)]
    struct Entry {
        id: String,
        thread_name: Option<String>,
    }
    index
        .lines()
        .filter_map(|line| serde_json::from_str::<Entry>(line).ok())
        .filter_map(|entry| Some((entry.id, entry.thread_name?)))
        .collect()
}

fn timestamp_ms(text: &str) -> Option<i64> {
    text.parse::<jiff::Timestamp>()
        .ok()
        .map(jiff::Timestamp::as_millisecond)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn collect(path: &Path, source: AgentSource, state: &mut SessionCursor) -> Vec<SessionEvent> {
        let mut events = Vec::new();
        read_new_events(path, source, state, |event, _| events.push(event)).unwrap();
        events
    }

    fn write(dir: &tempfile::TempDir, lines: &[&str]) -> std::path::PathBuf {
        let path = dir.path().join("session.jsonl");
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .unwrap();
        for line in lines {
            writeln!(file, "{line}").unwrap();
        }
        path
    }

    #[test]
    fn reads_claude_code_activity_prompts_title_and_branch() {
        let dir = tempfile::tempdir().unwrap();
        let path = write(
            &dir,
            &[
                r#"{"type":"user","timestamp":"2026-10-09T09:00:00Z","sessionId":"s1","cwd":"/r","gitBranch":"claude/int-290-x","message":{"content":"fix it"}}"#,
                r#"{"type":"assistant","timestamp":"2026-10-09T09:00:05Z","sessionId":"s1","cwd":"/r","gitBranch":"claude/int-290-x"}"#,
                r#"{"type":"user","timestamp":"2026-10-09T09:00:06Z","sessionId":"s1","toolUseResult":{"ok":true}}"#,
                r#"{"type":"custom-title","customTitle":"Order sync","sessionId":"s1"}"#,
            ],
        );
        let mut state = SessionCursor::default();

        let events = collect(&path, AgentSource::ClaudeCode, &mut state);

        assert_eq!(
            events.iter().map(|e| e.prompt).collect::<Vec<_>>(),
            [true, false, false]
        );
        assert_eq!(state.session_id.as_deref(), Some("s1"));
        assert_eq!(state.branch.as_deref(), Some("claude/int-290-x"));
        assert_eq!(state.title.as_deref(), Some("Order sync"));
    }

    #[test]
    fn continues_from_the_last_offset_and_waits_for_whole_lines() {
        let dir = tempfile::tempdir().unwrap();
        let path = write(
            &dir,
            &[r#"{"type":"assistant","timestamp":"2026-10-09T09:00:00Z"}"#],
        );
        let mut state = SessionCursor::default();
        collect(&path, AgentSource::ClaudeCode, &mut state);
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        write!(
            file,
            r#"{{"type":"assistant","timestamp":"2026-10-09T09:01:00Z"}}"#
        )
        .unwrap();

        let partial = collect(&path, AgentSource::ClaudeCode, &mut state);
        writeln!(file).unwrap();
        let complete = collect(&path, AgentSource::ClaudeCode, &mut state);

        assert!(partial.is_empty());
        assert_eq!(complete.len(), 1);
    }

    #[test]
    fn reads_codex_sessions_and_skips_replayed_history() {
        let dir = tempfile::tempdir().unwrap();
        let path = write(
            &dir,
            &[
                r#"{"timestamp":"2026-10-09T11:30:00Z","type":"session_meta","payload":{"id":"t1","cwd":"/w"}}"#,
                r#"{"timestamp":"2026-10-09T11:30:01Z","type":"event_msg","payload":{"type":"item_completed","item":{"type":"UserMessage","content":[{"type":"text","text":"go on"}],"completed_at_ms":1760002201000}}}"#,
                r#"{"timestamp":"2026-10-09T11:31:00Z","type":"event_msg","payload":{"type":"item_completed","item":{"type":"UserMessage","content":[{"type":"text","text":"go on"}],"completed_at_ms":1791545460000}}}"#,
            ],
        );
        let mut state = SessionCursor::default();

        let events = collect(&path, AgentSource::Codex, &mut state);

        assert_eq!(state.session_id.as_deref(), Some("t1"));
        assert_eq!(state.cwd.as_deref(), Some("/w"));
        assert_eq!(
            events
                .iter()
                .map(|e| (e.at_ms, e.prompt))
                .collect::<Vec<_>>(),
            [(1_791_545_400_000, false), (1_791_545_460_000, true)]
        );
    }

    #[test]
    fn injected_messages_are_not_prompts() {
        let dir = tempfile::tempdir().unwrap();
        let path = write(
            &dir,
            &[
                r#"{"type":"user","timestamp":"2026-10-09T09:00:00Z","sessionId":"s1","message":{"content":"<task-notification>\n<task-id>a1</task-id>\n</task-notification>"}}"#,
                r#"{"type":"user","timestamp":"2026-10-09T09:01:00Z","sessionId":"s1","message":{"content":[{"type":"text","text":"<artifact-view-context>{}</artifact-view-context>"},{"type":"text","text":"why is it dark?"}]}}"#,
                r#"{"type":"user","timestamp":"2026-10-09T09:02:00Z","sessionId":"s1","message":{"content":[{"type":"image","source":{}}]}}"#,
                r#"{"type":"user","timestamp":"2026-10-09T09:03:00Z","sessionId":"s1","message":{"content":"<system-reminder>x</system-reminder>  "}}"#,
            ],
        );
        let mut state = SessionCursor::default();

        let prompts: Vec<bool> = collect(&path, AgentSource::ClaudeCode, &mut state)
            .iter()
            .map(|event| event.prompt)
            .collect();

        assert_eq!(prompts, [false, true, true, false]);
        assert!(!typed("<heartbeat> <automation-id>x</automation-id>"));
        assert!(!typed("<environment_context>cwd</environment_context>"));
        assert!(typed("odpowiadaj mu"));
        assert!(typed("< 5 minutes please"));
    }

    #[test]
    fn reads_a_prompt_back_from_its_line_without_injected_blocks() {
        let dir = tempfile::tempdir().unwrap();
        let path = write(
            &dir,
            &[
                r#"{"type":"user","timestamp":"2026-10-09T09:00:00Z","sessionId":"s1","message":{"content":"<task-notification>x</task-notification>"}}"#,
                r#"{"type":"user","timestamp":"2026-10-09T09:01:00Z","sessionId":"s1","message":{"content":[{"type":"text","text":"<artifact-view-context>{}</artifact-view-context>"},{"type":"text","text":"why is it dark?"}]}}"#,
            ],
        );
        let mut state = SessionCursor::default();
        let prompt = collect(&path, AgentSource::ClaudeCode, &mut state)
            .into_iter()
            .find(|event| event.prompt)
            .unwrap();

        assert_eq!(
            prompt_text(&path, AgentSource::ClaudeCode, prompt.line_offset)
                .unwrap()
                .as_deref(),
            Some("why is it dark?")
        );
        assert_eq!(
            prompt_text(&path, AgentSource::ClaudeCode, 0).unwrap(),
            None
        );

        let codex = dir.path().join("codex.jsonl");
        std::fs::write(
            &codex,
            "{\"timestamp\":\"2026-10-09T11:30:01Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"item_completed\",\"item\":{\"type\":\"UserMessage\",\"content\":[{\"type\":\"text\",\"text\":\"keep the watcher\\n\"}]}}}\n",
        )
        .unwrap();
        assert_eq!(
            prompt_text(&codex, AgentSource::Codex, 0)
                .unwrap()
                .as_deref(),
            Some("keep the watcher")
        );
    }

    #[test]
    fn skips_review_threads_and_folds_helpers_into_the_parent() {
        let dir = tempfile::tempdir().unwrap();
        let review = write(
            &dir,
            &[
                r#"{"timestamp":"2026-10-09T11:30:00Z","type":"session_meta","payload":{"id":"r1","thread_source":"guardian_review"}}"#,
                r#"{"timestamp":"2026-10-09T11:30:01Z","type":"event_msg","payload":{"type":"item_completed","item":{"type":"UserMessage"}}}"#,
            ],
        );
        let mut review_state = SessionCursor::default();
        collect(&review, AgentSource::Codex, &mut review_state);
        assert_eq!(review_state.session_id, None);

        let helper_dir = tempfile::tempdir().unwrap();
        let helper = write(
            &helper_dir,
            &[
                r#"{"timestamp":"2026-10-09T11:30:00Z","type":"session_meta","payload":{"id":"h1","thread_source":"subagent","source":{"subagent":{"thread_spawn":{"parent_thread_id":"p1"}}}}}"#,
                r#"{"timestamp":"2026-10-09T11:30:01Z","type":"event_msg","payload":{"type":"item_completed","item":{"type":"UserMessage"}}}"#,
            ],
        );
        let mut helper_state = SessionCursor::default();
        let events = collect(&helper, AgentSource::Codex, &mut helper_state);

        assert_eq!(helper_state.session_id.as_deref(), Some("p1"));
        assert!(events.iter().all(|event| !event.prompt));
    }

    #[test]
    fn later_index_lines_rename_codex_threads() {
        let names = codex_thread_names(
            "{\"id\":\"t1\",\"thread_name\":\"Main\"}\n{\"id\":\"t1\",\"thread_name\":\"INT-177 Job runner batch 2\"}\n",
        );

        assert_eq!(names["t1"], "INT-177 Job runner batch 2");
    }
}
