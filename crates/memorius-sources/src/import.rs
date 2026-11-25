//! Imports Claude Code and Codex session activity into the store every
//! minute. Each file is read from its saved offset, so after the first run
//! only new lines are read.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::git::GitHistories;
use crate::sessions::{codex_thread_names, read_new_events};
use memorius_core::agent::AgentSettings;
use memorius_core::context::{SessionEvent, add_event};
use memorius_core::model::{AgentSource, ContextSpan};
use memorius_core::store::{SessionImport, Store, StoreError};

const INTERVAL: Duration = Duration::from_secs(60);

/// Session files untouched for longer than this are not imported.
const HISTORY: Duration = Duration::from_secs(90 * 24 * 60 * 60);

/// Codex keeps sessions in `YYYY/MM/DD/` folders.
const CODEX_DEPTH: usize = 3;
const CLAUDE_DEPTH: usize = 3;

pub struct Roots {
    claude_projects: PathBuf,
    codex: PathBuf,
}

impl Roots {
    /// The folders from Settings, else `CLAUDE_CONFIG_DIR` / `CODEX_HOME`,
    /// else `~/.claude` and `~/.codex`.
    pub fn new(settings: &AgentSettings) -> Option<Self> {
        let home = std::env::home_dir()?;
        let folder = |setting: &str, variable: &str, default: &str| {
            expand_home(setting)
                .or_else(|| std::env::var_os(variable).map(PathBuf::from))
                .unwrap_or_else(|| home.join(default))
        };
        Some(Self {
            claude_projects: folder(&settings.claude_home, "CLAUDE_CONFIG_DIR", ".claude")
                .join("projects"),
            codex: folder(&settings.codex_home, "CODEX_HOME", ".codex"),
        })
    }
}

/// A folder the user typed, with `~/` for the home folder; `None` when empty.
pub fn expand_home(path: &str) -> Option<PathBuf> {
    let path = path.trim();
    if path.is_empty() {
        return None;
    }
    match path.strip_prefix("~/") {
        Some(rest) => std::env::home_dir().map(|home| home.join(rest)),
        None => Some(PathBuf::from(path)),
    }
}

pub fn start(database: PathBuf) {
    let spawned = std::thread::Builder::new()
        .name("memorius-context".into())
        .spawn(move || {
            let mut git = GitHistories::default();
            loop {
                // The folders are read each time, so a change in Settings
                // applies within a minute.
                let roots = Store::open(&database)
                    .and_then(|store| store.agent_settings())
                    .map(|settings| Roots::new(&settings));
                match roots {
                    Ok(Some(roots)) => {
                        if let Err(error) = import_all(&database, &roots, &mut git) {
                            eprintln!("memorius: session import failed: {error}");
                        }
                    }
                    Ok(None) => eprintln!("memorius: no home folder for agent sessions"),
                    Err(error) => eprintln!("memorius: session import failed: {error}"),
                }
                std::thread::sleep(INTERVAL);
            }
        });
    if let Err(error) = spawned {
        eprintln!("memorius: cannot start the session importer: {error}");
    }
}

/// Imports every session file changed since the last run.
pub fn import_all(
    database: &Path,
    roots: &Roots,
    git: &mut GitHistories,
) -> Result<(), StoreError> {
    let store = Store::open(database)?;
    let oldest = SystemTime::now() - HISTORY;
    let titles = fs::read_to_string(roots.codex.join("session_index.jsonl"))
        .map(|index| codex_thread_names(&index))
        .unwrap_or_default();
    let mut files = Vec::new();
    // <project>/<session>.jsonl, and <project>/<session>/subagents/*.jsonl
    // for helper agents (their lines carry the parent session id).
    collect(&roots.claude_projects, CLAUDE_DEPTH, &mut |path| {
        files.push((path, AgentSource::ClaudeCode));
    });
    collect(&roots.codex.join("sessions"), CODEX_DEPTH, &mut |path| {
        files.push((path, AgentSource::Codex));
    });
    // Archiving a Codex thread moves its file here.
    collect(&roots.codex.join("archived_sessions"), 0, &mut |path| {
        files.push((path, AgentSource::Codex));
    });
    for (path, source) in files {
        let Some(modified) = fs::metadata(&path).and_then(|meta| meta.modified()).ok() else {
            continue;
        };
        if modified < oldest {
            continue;
        }
        if let Err(error) = import_file(&store, &path, source, unix_ms(modified), &titles, git) {
            eprintln!("memorius: cannot import {}: {error}", path.display());
        }
    }
    Ok(())
}

/// Calls `found` for every `*.jsonl` file up to `depth` folders below `root`.
fn collect(root: &Path, depth: usize, found: &mut impl FnMut(PathBuf)) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        match entry.file_type() {
            Ok(kind) if kind.is_dir() && depth > 0 => collect(&path, depth - 1, found),
            Ok(kind) if kind.is_file() && path.extension().is_some_and(|ext| ext == "jsonl") => {
                found(path);
            }
            _ => {}
        }
    }
}

fn import_file(
    store: &Store,
    path: &Path,
    source: AgentSource,
    modified_ms: i64,
    titles: &HashMap<String, String>,
    git: &mut GitHistories,
) -> Result<(), Box<dyn std::error::Error>> {
    // Codex moves a file when its thread is archived; its unique name keeps
    // the read position, so the file is not read twice.
    let key = match source {
        AgentSource::Codex => path
            .file_name()
            .map_or_else(|| path.to_string_lossy(), |name| name.to_string_lossy()),
        AgentSource::ClaudeCode => path.to_string_lossy(),
    };
    let (stored_modified, mut cursor) = store.session_cursor(&key)?.unwrap_or_default();
    if stored_modified == modified_ms {
        return Ok(());
    }
    let mut spans: Vec<ContextSpan> = Vec::new();
    let mut continues_span = None;
    let mut looked_up = false;
    let mut prompt_lines: Vec<(String, i64, u64)> = Vec::new();
    read_new_events(path, source, &mut cursor, |event, state| {
        let Some(session_id) = state.session_id.as_deref() else {
            return;
        };
        if !looked_up {
            looked_up = true;
            if let Ok(Some((id, last))) = store.last_context_span(source, session_id) {
                continues_span = Some(id);
                spans.push(last);
            }
        }
        // Claude Code writes the branch on every line; for Codex it comes
        // from the Git history of the session folder.
        let branch = match source {
            AgentSource::ClaudeCode => state.branch.clone(),
            AgentSource::Codex => state
                .cwd
                .as_deref()
                .and_then(|cwd| git.get(Path::new(cwd)))
                .and_then(|history| history.branch_at(event.at_ms))
                .map(str::to_owned),
        };
        if event.prompt {
            prompt_lines.push((session_id.to_owned(), event.at_ms, event.line_offset));
        }
        add_event(
            &mut spans,
            &SessionEvent {
                source,
                session_id,
                cwd: state.cwd.as_deref(),
                branch: branch.as_deref(),
                at_ms: event.at_ms,
                prompt: event.prompt,
            },
        );
    })?;
    if let Some(name) = cursor.session_id.as_ref().and_then(|id| titles.get(id)) {
        cursor.title = Some(name.clone());
    }
    store.save_session_import(&SessionImport {
        path: &key,
        source,
        modified_ms,
        cursor: &cursor,
        continues_span,
        spans: &spans,
        file: &path.to_string_lossy(),
        prompt_lines: &prompt_lines,
    })?;
    Ok(())
}

fn unix_ms(time: SystemTime) -> i64 {
    time.duration_since(UNIX_EPOCH).map_or(0, |elapsed| {
        i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
    })
}
