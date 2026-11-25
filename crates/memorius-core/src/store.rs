use std::path::Path;

use rusqlite::{Connection, OpenFlags, OptionalExtension, Row, params};

use crate::agent::{AgentKind, AgentRun, AgentSettings, RunState};
use crate::model::{
    AgentSource, AppRef, AppUsage, ContextSpan, Rule, RuleKind, SessionCursor, SessionRef, Title,
    TitleUnavailable, Window,
};
use crate::proposal::{NewProposal, Proposal, ProposalEdit, ProposalError, ProposalState};
use crate::search::{
    IssueHit, IssueWorklog, MAX_HITS, MAX_ROWS, MAX_SPANS, SearchResults, SessionHit, WindowHit,
    like_pattern,
};
use crate::timeline::DayCommit;
use crate::tracker::{TrackerWorklog, WORK_TYPES};

/// Schema version this build reads and writes (`PRAGMA user_version`): one
/// version per migration, so the two cannot disagree.
#[expect(clippy::cast_possible_wrap, reason = "a handful of migrations")]
pub const SCHEMA_VERSION: i64 = MIGRATIONS.len() as i64;

const MIGRATIONS: [&str; 20] = [
    r"
CREATE TABLE activity (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('app', 'away', 'locked')),
    app_id TEXT,
    app_name TEXT,
    title TEXT,
    title_unavailable TEXT CHECK (title_unavailable IN ('permissionDenied', 'noWindow')),
    document TEXT,
    start_ms INTEGER NOT NULL,
    end_ms INTEGER NOT NULL CHECK (end_ms >= start_ms),
    CHECK ((kind = 'app') = (app_id IS NOT NULL AND app_name IS NOT NULL)),
    CHECK (kind <> 'app' OR ((title IS NULL) <> (title_unavailable IS NULL))),
    CHECK (kind = 'app' OR (title IS NULL AND title_unavailable IS NULL AND document IS NULL))
) STRICT;
CREATE INDEX activity_start ON activity (start_ms);
CREATE INDEX activity_end ON activity (end_ms);
",
    r"
CREATE TABLE session_file (
    path TEXT PRIMARY KEY,
    source TEXT NOT NULL CHECK (source IN ('claudeCode', 'codex')),
    modified_ms INTEGER NOT NULL,
    offset INTEGER NOT NULL CHECK (offset >= 0),
    session_id TEXT,
    cwd TEXT,
    branch TEXT,
    title TEXT
) STRICT;
CREATE TABLE context_span (
    id INTEGER PRIMARY KEY,
    source TEXT NOT NULL CHECK (source IN ('claudeCode', 'codex')),
    session_id TEXT NOT NULL,
    title TEXT,
    cwd TEXT,
    branch TEXT,
    start_ms INTEGER NOT NULL,
    end_ms INTEGER NOT NULL CHECK (end_ms >= start_ms),
    prompt_times TEXT NOT NULL CHECK (json_valid(prompt_times))
) STRICT;
CREATE INDEX context_span_end ON context_span (end_ms);
CREATE INDEX context_span_session ON context_span (source, session_id, start_ms);
ALTER TABLE activity ADD COLUMN url TEXT CHECK (url IS NULL OR kind = 'app');
",
    // Session data is rebuilt from the session files: clear what earlier
    // builds imported without the review-thread and helper-agent rules.
    r"
DELETE FROM context_span;
DELETE FROM session_file;
ALTER TABLE session_file ADD COLUMN subagent INTEGER NOT NULL DEFAULT 0 CHECK (subagent IN (0, 1));
",
    // Private apps add a title state; SQLite changes a CHECK only by
    // rebuilding the table.
    r"
CREATE TABLE activity_v4 (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('app', 'away', 'locked')),
    app_id TEXT,
    app_name TEXT,
    title TEXT,
    title_unavailable TEXT CHECK (title_unavailable IN ('permissionDenied', 'noWindow', 'private')),
    document TEXT,
    start_ms INTEGER NOT NULL,
    end_ms INTEGER NOT NULL CHECK (end_ms >= start_ms),
    url TEXT CHECK (url IS NULL OR kind = 'app'),
    CHECK ((kind = 'app') = (app_id IS NOT NULL AND app_name IS NOT NULL)),
    CHECK (kind <> 'app' OR ((title IS NULL) <> (title_unavailable IS NULL))),
    CHECK (kind = 'app' OR (title IS NULL AND title_unavailable IS NULL AND document IS NULL))
) STRICT;
INSERT INTO activity_v4 (id, kind, app_id, app_name, title, title_unavailable, document, start_ms, end_ms, url)
    SELECT id, kind, app_id, app_name, title, title_unavailable, document, start_ms, end_ms, url FROM activity;
DROP TABLE activity;
ALTER TABLE activity_v4 RENAME TO activity;
CREATE INDEX activity_start ON activity (start_ms);
CREATE INDEX activity_end ON activity (end_ms);
CREATE TABLE rule (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('privateApp', 'appIssue', 'hostIssue', 'titleIssue')),
    pattern TEXT NOT NULL CHECK (pattern <> ''),
    issue TEXT,
    CHECK ((kind = 'privateApp') = (issue IS NULL))
) STRICT;
",
    r"
CREATE TABLE proposal (
    id INTEGER PRIMARY KEY,
    issue TEXT NOT NULL,
    start_ms INTEGER NOT NULL,
    end_ms INTEGER NOT NULL CHECK (end_ms > start_ms),
    minutes INTEGER NOT NULL CHECK (minutes > 0),
    work_type TEXT,
    text TEXT NOT NULL CHECK (text <> ''),
    agent TEXT NOT NULL,
    created_ms INTEGER NOT NULL,
    state TEXT NOT NULL CHECK (state IN ('proposed', 'approved', 'rejected', 'logged')),
    decided_ms INTEGER,
    work_item_id TEXT,
    last_error TEXT,
    CHECK ((state = 'proposed') = (decided_ms IS NULL)),
    CHECK ((state = 'logged') = (work_item_id IS NOT NULL)),
    UNIQUE (issue, start_ms, end_ms)
) STRICT;
CREATE INDEX proposal_end ON proposal (end_ms);
",
    // Session files are known by name for Codex (archiving moves them) and
    // archived and subagent files are read too: import everything again.
    r"
DELETE FROM context_span;
DELETE FROM session_file;
",
    // The user's decisions on agent time that had no sign of supervision.
    r"
CREATE TABLE agent_decision (
    id INTEGER PRIMARY KEY,
    start_ms INTEGER NOT NULL,
    end_ms INTEGER NOT NULL CHECK (end_ms > start_ms),
    decision TEXT NOT NULL CHECK (decision IN ('work', 'notWork')),
    decided_ms INTEGER NOT NULL
) STRICT;
CREATE INDEX agent_decision_end ON agent_decision (end_ms);
",
    // Answers belong to one agent session; older rows (no session) still
    // cover every session in their range.
    r"
ALTER TABLE agent_decision ADD COLUMN source TEXT CHECK (source IS NULL OR source IN ('claudeCode', 'codex'));
ALTER TABLE agent_decision ADD COLUMN session_id TEXT;
",
    // Prompts leave out injected messages (heartbeats, task notifications):
    // session data is rebuilt from the session files.
    r"
DELETE FROM context_span;
DELETE FROM session_file;
",
    // Where each prompt's line is, to show its text on hover; the text
    // itself is never stored. Session data is imported again to fill it.
    r"
CREATE TABLE prompt_line (
    source TEXT NOT NULL CHECK (source IN ('claudeCode', 'codex')),
    session_id TEXT NOT NULL,
    at_ms INTEGER NOT NULL,
    path TEXT NOT NULL,
    offset INTEGER NOT NULL CHECK (offset >= 0),
    PRIMARY KEY (source, session_id, at_ms, offset)
) STRICT;
DELETE FROM context_span;
DELETE FROM session_file;
",
    // Agent time counts on its own now (up to 30 minutes after a prompt);
    // the user only marks whole sessions private. Rows in agent_decision
    // stay as history and are not read any more.
    r"
CREATE TABLE private_session (
    source TEXT NOT NULL CHECK (source IN ('claudeCode', 'codex')),
    session_id TEXT NOT NULL,
    decided_ms INTEGER NOT NULL,
    PRIMARY KEY (source, session_id)
) STRICT;
",
    // Memorius starts the user's agent: its settings and each run. One run
    // at a time; a finished run has its end and message.
    r"
CREATE TABLE setting (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT;
CREATE TABLE agent_run (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('proposals', 'log')),
    agent TEXT NOT NULL,
    from_ms INTEGER,
    to_ms INTEGER,
    started_ms INTEGER NOT NULL,
    state TEXT NOT NULL CHECK (state IN ('running', 'done', 'failed')),
    finished_ms INTEGER,
    message TEXT,
    CHECK ((state = 'running') = (finished_ms IS NULL)),
    CHECK ((state = 'running') = (message IS NULL))
) STRICT;
CREATE UNIQUE INDEX agent_run_one_running ON agent_run (state) WHERE state = 'running';
",
    // Work items read from the user's tracker; a sync replaces its range.
    r"
CREATE TABLE tracker_worklog (
    id TEXT PRIMARY KEY,
    issue TEXT NOT NULL,
    summary TEXT,
    date TEXT NOT NULL,
    minutes INTEGER NOT NULL CHECK (minutes >= 0),
    text TEXT,
    work_type TEXT
) STRICT;
CREATE INDEX tracker_worklog_date ON tracker_worklog (date);
",
    // Trackers log per issue and day: a proposal gets its local day, and
    // waiting proposals of one issue and day merge into one (minutes summed,
    // texts joined in time order). Approved, logged and rejected ones stay.
    r"
ALTER TABLE proposal ADD COLUMN date TEXT NOT NULL DEFAULT '';
UPDATE proposal SET date = date(start_ms / 1000, 'unixepoch', 'localtime');
CREATE TEMP TABLE proposal_merge AS
    SELECT issue, date, min(id) AS keep, sum(minutes) AS minutes,
           min(start_ms) AS start_ms, max(end_ms) AS end_ms,
           (SELECT group_concat(text, '; ') FROM (
               SELECT text FROM proposal p
               WHERE p.state = 'proposed' AND p.issue = g.issue AND p.date = g.date
               ORDER BY p.start_ms)) AS text
    FROM proposal g WHERE state = 'proposed' GROUP BY issue, date HAVING count(*) > 1;
DELETE FROM proposal WHERE state = 'proposed' AND EXISTS (
    SELECT 1 FROM proposal_merge m
    WHERE m.issue = proposal.issue AND m.date = proposal.date AND m.keep <> proposal.id);
UPDATE proposal SET
    minutes = (SELECT minutes FROM proposal_merge WHERE keep = proposal.id),
    start_ms = (SELECT start_ms FROM proposal_merge WHERE keep = proposal.id),
    end_ms = (SELECT end_ms FROM proposal_merge WHERE keep = proposal.id),
    text = (SELECT text FROM proposal_merge WHERE keep = proposal.id)
WHERE id IN (SELECT keep FROM proposal_merge);
DROP TABLE proposal_merge;
CREATE UNIQUE INDEX proposal_waiting_day ON proposal (issue, date) WHERE state = 'proposed';
",
    // The vault: secrets encrypted with a key that lives in the system
    // keychain; this table never holds a readable secret.
    r"
CREATE TABLE secret (
    name TEXT PRIMARY KEY,
    nonce BLOB NOT NULL,
    ciphertext BLOB NOT NULL
) STRICT;
",
    // The user's own commits on this Mac, read from the repositories'
    // reflogs; a scan replaces them all.
    r"
CREATE TABLE git_commit (
    hash TEXT PRIMARY KEY,
    at_ms INTEGER NOT NULL,
    subject TEXT NOT NULL,
    repo TEXT NOT NULL,
    branch TEXT
) STRICT;
CREATE INDEX git_commit_at ON git_commit (at_ms);
",
    // Where a commit's repository is, to read its details on hover.
    r"
ALTER TABLE git_commit ADD COLUMN git_dir TEXT;
",
    // Memorius logs approved worklogs itself; agents only propose.
    r"
DELETE FROM agent_run WHERE kind = 'log';
ALTER TABLE agent_run DROP COLUMN kind;
",
    // One repositories folder became a list.
    r"
UPDATE setting
SET value = json_object('roots', json_array(json_extract(value, '$.root')))
WHERE key = 'git' AND coalesce(json_extract(value, '$.root'), '') != '';
",
    // An approval must name the revision of the proposal the user saw.
    r"
ALTER TABLE proposal ADD COLUMN revision INTEGER NOT NULL DEFAULT 1;
",
];

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(
        "database schema version {found} is newer than this build supports ({supported}); update Memorius"
    )]
    SchemaTooNew { found: i64, supported: i64 },
    #[error(
        "database schema version {found} is older than {supported}; open Memorius once to migrate it"
    )]
    SchemaTooOld { found: i64, supported: i64 },
    #[error(transparent)]
    Proposal(#[from] ProposalError),
    #[error("there is no proposal {0}")]
    NoProposal(i64),
    #[error("stored JSON is not valid: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RowKind {
    App(Window),
    Away,
    Locked,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivityRow {
    pub id: i64,
    pub start_ms: i64,
    pub end_ms: i64,
    pub kind: RowKind,
}

pub struct Store {
    conn: Connection,
}

/// A vault secret as stored: its nonce and ciphertext, never readable.
pub type SealedSecret = (Vec<u8>, Vec<u8>);

impl Store {
    /// Opens the database for writing and migrates it to [`SCHEMA_VERSION`].
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    /// Opens the database read-only. The schema must already be current.
    pub fn open_read_only(path: &Path) -> Result<Self, StoreError> {
        Self::open_current(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
    }

    /// Opens for writing without migrating, for agents through the CLI: the
    /// desktop app owns the schema.
    pub fn open_existing(path: &Path) -> Result<Self, StoreError> {
        Self::open_current(path, OpenFlags::SQLITE_OPEN_READ_WRITE)
    }

    fn open_current(path: &Path, mode: OpenFlags) -> Result<Self, StoreError> {
        let conn = Connection::open_with_flags(path, mode | OpenFlags::SQLITE_OPEN_NO_MUTEX)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        let found = user_version(&conn)?;
        match found.cmp(&SCHEMA_VERSION) {
            std::cmp::Ordering::Equal => Ok(Self { conn }),
            std::cmp::Ordering::Greater => Err(StoreError::SchemaTooNew {
                found,
                supported: SCHEMA_VERSION,
            }),
            std::cmp::Ordering::Less => Err(StoreError::SchemaTooOld {
                found,
                supported: SCHEMA_VERSION,
            }),
        }
    }

    fn migrate(&self) -> Result<(), StoreError> {
        let found = user_version(&self.conn)?;
        if found > SCHEMA_VERSION {
            return Err(StoreError::SchemaTooNew {
                found,
                supported: SCHEMA_VERSION,
            });
        }
        for (version, sql) in (1_i64..)
            .zip(MIGRATIONS)
            .filter(|(version, _)| *version > found)
        {
            let tx = self.conn.unchecked_transaction()?;
            tx.execute_batch(sql)?;
            tx.pragma_update(None, "user_version", version)?;
            tx.commit()?;
        }
        Ok(())
    }

    pub fn insert(&self, start_ms: i64, end_ms: i64, kind: &RowKind) -> Result<i64, StoreError> {
        let (kind_text, app, title, unavailable, document, url) = match kind {
            RowKind::App(window) => {
                let (title, unavailable) = match &window.title {
                    Title::Known { text } => (Some(text.as_str()), None),
                    Title::Unavailable { reason } => (None, Some(unavailable_text(*reason))),
                };
                (
                    "app",
                    Some(&window.app),
                    title,
                    unavailable,
                    window.document.as_deref(),
                    window.url.as_deref(),
                )
            }
            RowKind::Away => ("away", None, None, None, None, None),
            RowKind::Locked => ("locked", None, None, None, None, None),
        };
        self.conn.execute(
            "INSERT INTO activity (kind, app_id, app_name, title, title_unavailable, document, start_ms, end_ms, url)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                kind_text,
                app.map(|a| a.id.as_str()),
                app.map(|a| a.name.as_str()),
                title,
                unavailable,
                document,
                start_ms,
                end_ms,
                url
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn set_end(&self, id: i64, end_ms: i64) -> Result<(), StoreError> {
        self.conn.execute(
            "UPDATE activity SET end_ms = ?2 WHERE id = ?1",
            params![id, end_ms],
        )?;
        Ok(())
    }

    /// Rows that overlap `[from_ms, to_ms)`, oldest first.
    pub fn rows_between(&self, from_ms: i64, to_ms: i64) -> Result<Vec<ActivityRow>, StoreError> {
        let mut statement = self.conn.prepare_cached(
            "SELECT id, kind, app_id, app_name, title, title_unavailable, document, start_ms, end_ms, url
             FROM activity WHERE end_ms > ?1 AND start_ms < ?2 ORDER BY start_ms, id",
        )?;
        let rows = statement
            .query_map(params![from_ms, to_ms], read_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn last_row(&self) -> Result<Option<ActivityRow>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT id, kind, app_id, app_name, title, title_unavailable, document, start_ms, end_ms, url
                 FROM activity ORDER BY start_ms DESC, id DESC LIMIT 1",
                [],
                read_row,
            )
            .optional()?)
    }
}

/// One session file's new spans and read position, saved together.
pub struct SessionImport<'a> {
    pub path: &'a str,
    pub source: AgentSource,
    pub modified_ms: i64,
    pub cursor: &'a SessionCursor,
    /// The stored span that `spans[0]` continues, if any.
    pub continues_span: Option<i64>,
    pub spans: &'a [ContextSpan],
    /// The file's full path now (`path` is only its name for Codex).
    pub file: &'a str,
    /// `(session_id, at_ms, line offset)` of each prompt read from the file.
    pub prompt_lines: &'a [(String, i64, u64)],
}

impl Store {
    /// When the newest imported session file last changed: shows whether
    /// the app's importer keeps up.
    pub fn newest_session_file_ms(&self) -> Result<Option<i64>, StoreError> {
        Ok(self
            .conn
            .query_row("SELECT MAX(modified_ms) FROM session_file", [], |row| {
                row.get(0)
            })?)
    }

    pub fn session_cursor(&self, path: &str) -> Result<Option<(i64, SessionCursor)>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT modified_ms, offset, session_id, cwd, branch, title, subagent FROM session_file WHERE path = ?1",
                [path],
                |row| {
                    Ok((
                        row.get(0)?,
                        SessionCursor {
                            offset: row.get::<_, i64>(1)?.unsigned_abs(),
                            session_id: row.get(2)?,
                            cwd: row.get(3)?,
                            branch: row.get(4)?,
                            title: row.get(5)?,
                            subagent: row.get(6)?,
                        },
                    ))
                },
            )
            .optional()?)
    }

    /// The newest stored span of a session, with its id.
    pub fn last_context_span(
        &self,
        source: AgentSource,
        session_id: &str,
    ) -> Result<Option<(i64, ContextSpan)>, StoreError> {
        let mut statement = self.conn.prepare_cached(
            "SELECT id, source, session_id, title, cwd, branch, start_ms, end_ms, prompt_times
             FROM context_span WHERE source = ?1 AND session_id = ?2 ORDER BY start_ms DESC LIMIT 1",
        )?;
        let found = statement
            .query_row(params![source_text(source), session_id], |row| {
                Ok((row.get::<_, i64>(0)?, read_span(row)?))
            })
            .optional()?;
        found
            .map(|(id, (span, prompts))| Ok((id, with_prompts(span, &prompts)?)))
            .transpose()
    }

    pub fn save_session_import(&self, import: &SessionImport<'_>) -> Result<(), StoreError> {
        let tx = self.conn.unchecked_transaction()?;
        let source = source_text(import.source);
        for (index, span) in import.spans.iter().enumerate() {
            let prompts = serde_json::to_string(&span.prompt_times)?;
            match import.continues_span.filter(|_| index == 0) {
                Some(id) => tx.execute(
                    "UPDATE context_span SET cwd = ?2, branch = ?3, start_ms = ?4, end_ms = ?5, prompt_times = ?6
                     WHERE id = ?1",
                    params![id, span.cwd, span.branch, span.start_ms, span.end_ms, prompts],
                )?,
                // A subagent file has no title; its session's other files do.
                None => tx.execute(
                    "INSERT INTO context_span (source, session_id, title, cwd, branch, start_ms, end_ms, prompt_times)
                     VALUES (?1, ?2, COALESCE(?3, (SELECT title FROM context_span
                         WHERE source = ?1 AND session_id = ?2 AND title IS NOT NULL LIMIT 1)),
                         ?4, ?5, ?6, ?7, ?8)",
                    params![
                        source,
                        span.session_id,
                        import.cursor.title,
                        span.cwd,
                        span.branch,
                        span.start_ms,
                        span.end_ms,
                        prompts
                    ],
                )?,
            };
        }
        for (session_id, at_ms, offset) in import.prompt_lines {
            tx.execute(
                "INSERT OR IGNORE INTO prompt_line (source, session_id, at_ms, path, offset)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    source,
                    session_id,
                    at_ms,
                    import.file,
                    i64::try_from(*offset).unwrap_or(i64::MAX)
                ],
            )?;
        }
        if let (Some(session_id), Some(title)) = (&import.cursor.session_id, &import.cursor.title) {
            tx.execute(
                "UPDATE context_span SET title = ?3 WHERE source = ?1 AND session_id = ?2",
                params![source, session_id, title],
            )?;
        }
        tx.execute(
            "INSERT INTO session_file (path, source, modified_ms, offset, session_id, cwd, branch, title, subagent)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT (path) DO UPDATE SET modified_ms = ?3, offset = ?4, session_id = ?5,
                 cwd = ?6, branch = ?7, title = ?8, subagent = ?9",
            params![
                import.path,
                source,
                import.modified_ms,
                i64::try_from(import.cursor.offset).unwrap_or(i64::MAX),
                import.cursor.session_id,
                import.cursor.cwd,
                import.cursor.branch,
                import.cursor.title,
                import.cursor.subagent
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Session spans that overlap `[from_ms, to_ms)`, oldest first.
    pub fn context_spans_between(
        &self,
        from_ms: i64,
        to_ms: i64,
    ) -> Result<Vec<ContextSpan>, StoreError> {
        let mut statement = self.conn.prepare_cached(
            "SELECT id, source, session_id, title, cwd, branch, start_ms, end_ms, prompt_times
             FROM context_span WHERE end_ms >= ?1 AND start_ms < ?2 ORDER BY start_ms, id",
        )?;
        let rows = statement
            .query_map(params![from_ms, to_ms], read_span)?
            .collect::<Result<Vec<_>, _>>()?;
        rows.into_iter()
            .map(|(span, prompts)| with_prompts(span, &prompts))
            .collect()
    }
}

impl Store {
    /// Windows, agent sessions and proposals since `since_ms` that match
    /// `query`. Titles of apps that are private now never come back, even
    /// when they were recorded before the app was made private.
    pub fn search(&self, query: &str, since_ms: i64) -> Result<SearchResults, StoreError> {
        let Some(pattern) = like_pattern(query) else {
            return Ok(SearchResults::default());
        };
        let private: Vec<String> = self
            .rules()?
            .into_iter()
            .filter_map(|rule| match rule.rule {
                RuleKind::PrivateApp { app_id } => Some(app_id),
                _ => None,
            })
            .collect();
        let (windows, windows_cut) = self.search_windows(&pattern, since_ms, &private)?;
        let (issues, issues_cut) = self.search_issues(&pattern, since_ms)?;
        Ok(SearchResults {
            issues,
            windows,
            sessions: self.search_sessions(&pattern, since_ms)?,
            truncated: windows_cut || issues_cut,
        })
    }

    fn search_windows(
        &self,
        pattern: &str,
        since_ms: i64,
        private: &[String],
    ) -> Result<(Vec<WindowHit>, bool), StoreError> {
        let limit = i64::try_from(MAX_ROWS + 1).unwrap_or(i64::MAX);
        let mut truncated = false;
        let mut statement = self.conn.prepare_cached(
            "SELECT app_id, app_name, title, url, start_ms, end_ms FROM activity
             WHERE kind = 'app' AND title IS NOT NULL AND end_ms > ?1
               AND (title LIKE ?2 ESCAPE '\\' OR url LIKE ?2 ESCAPE '\\' OR app_name LIKE ?2 ESCAPE '\\')
             ORDER BY start_ms DESC LIMIT ?3",
        )?;
        let rows = statement
            .query_map(params![since_ms, pattern, limit], |row| {
                Ok((
                    AppRef {
                        id: row.get(0)?,
                        name: row.get(1)?,
                    },
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        truncated |= rows.len() > MAX_ROWS;
        let mut windows: Vec<WindowHit> = Vec::new();
        for (app, title, url, start_ms, end_ms) in rows.into_iter().take(MAX_ROWS) {
            if private.contains(&app.id) {
                continue;
            }
            let same = windows
                .iter()
                .position(|hit| hit.app.id == app.id && hit.title == title);
            if let Some(hit) = same.and_then(|index| windows.get_mut(index)) {
                hit.total_ms += end_ms - start_ms;
                if hit.spans.len() < MAX_SPANS {
                    hit.spans.push((start_ms, end_ms));
                }
            } else if windows.len() < MAX_HITS {
                windows.push(WindowHit {
                    app,
                    title,
                    url,
                    total_ms: end_ms - start_ms,
                    spans: vec![(start_ms, end_ms)],
                });
            } else {
                truncated = true;
            }
        }

        Ok((windows, truncated))
    }

    fn search_sessions(&self, pattern: &str, since_ms: i64) -> Result<Vec<SessionHit>, StoreError> {
        let mut statement = self.conn.prepare_cached(
            "SELECT source, session_id, title, branch, min(start_ms), max(end_ms) FROM context_span
             WHERE end_ms > ?1 AND (title LIKE ?2 ESCAPE '\\' OR branch LIKE ?2 ESCAPE '\\')
             GROUP BY source, session_id ORDER BY max(end_ms) DESC LIMIT ?3",
        )?;
        let sessions = statement
            .query_map(
                params![
                    since_ms,
                    pattern,
                    i64::try_from(MAX_HITS).unwrap_or(i64::MAX)
                ],
                |row| {
                    let source: String = row.get(0)?;
                    Ok(SessionHit {
                        source: if source == "codex" {
                            AgentSource::Codex
                        } else {
                            AgentSource::ClaudeCode
                        },
                        title: row.get(2)?,
                        branch: row.get(3)?,
                        start_ms: row.get(4)?,
                        end_ms: row.get(5)?,
                    })
                },
            )?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    fn search_issues(
        &self,
        pattern: &str,
        since_ms: i64,
    ) -> Result<(Vec<IssueHit>, bool), StoreError> {
        let limit = i64::try_from(MAX_ROWS + 1).unwrap_or(i64::MAX);
        let mut truncated = false;
        let mut statement = self.conn.prepare_cached(
            "SELECT issue, start_ms, minutes, state FROM proposal
             WHERE end_ms > ?1 AND state <> 'rejected' AND issue LIKE ?2 ESCAPE '\\'
             ORDER BY start_ms DESC LIMIT ?3",
        )?;
        let worklogs = statement
            .query_map(params![since_ms, pattern, limit], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    IssueWorklog {
                        start_ms: row.get(1)?,
                        minutes: row.get(2)?,
                        logged: row.get::<_, String>(3)? == "logged",
                    },
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        truncated |= worklogs.len() > MAX_ROWS;
        let mut issues: Vec<IssueHit> = Vec::new();
        for (issue, worklog) in worklogs.into_iter().take(MAX_ROWS) {
            let same = issues.iter().position(|hit| hit.issue == issue);
            if let Some(hit) = same.and_then(|index| issues.get_mut(index)) {
                hit.worklogs.push(worklog);
            } else if issues.len() < MAX_HITS {
                issues.push(IssueHit {
                    issue,
                    worklogs: vec![worklog],
                });
            } else {
                truncated = true;
            }
        }

        Ok((issues, truncated))
    }

    pub fn rules(&self) -> Result<Vec<Rule>, StoreError> {
        let mut statement = self
            .conn
            .prepare_cached("SELECT id, kind, pattern, issue FROM rule ORDER BY id")?;
        let rules = statement
            .query_map([], |row| {
                let kind: String = row.get(1)?;
                let pattern: String = row.get(2)?;
                let issue: Option<String> = row.get(3)?;
                let issue = issue.unwrap_or_default();
                let rule = match kind.as_str() {
                    "privateApp" => RuleKind::PrivateApp { app_id: pattern },
                    "appIssue" => RuleKind::AppIssue {
                        app_id: pattern,
                        issue,
                    },
                    "hostIssue" => RuleKind::HostIssue {
                        host: pattern,
                        issue,
                    },
                    _ => RuleKind::TitleIssue {
                        text: pattern,
                        issue,
                    },
                };
                Ok(Rule {
                    id: row.get(0)?,
                    rule,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rules)
    }

    pub fn add_rule(&self, rule: &RuleKind) -> Result<i64, StoreError> {
        let (kind, pattern, issue) = match rule {
            RuleKind::PrivateApp { app_id } => ("privateApp", app_id, None),
            RuleKind::AppIssue { app_id, issue } => ("appIssue", app_id, Some(issue)),
            RuleKind::HostIssue { host, issue } => ("hostIssue", host, Some(issue)),
            RuleKind::TitleIssue { text, issue } => ("titleIssue", text, Some(issue)),
        };
        self.conn.execute(
            "INSERT INTO rule (kind, pattern, issue) VALUES (?1, ?2, ?3)",
            params![
                kind,
                pattern.trim(),
                issue.map(|issue| issue.trim().to_ascii_uppercase())
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Apps used since `since_ms`, most time first.
    pub fn recent_apps(&self, since_ms: i64) -> Result<Vec<AppUsage>, StoreError> {
        let mut statement = self.conn.prepare_cached(
            "SELECT app_id, max(app_name), sum(end_ms - start_ms) AS total FROM activity
             WHERE kind = 'app' AND end_ms > ?1 GROUP BY app_id ORDER BY total DESC LIMIT 40",
        )?;
        let apps = statement
            .query_map([since_ms], |row| {
                Ok(AppUsage {
                    app: AppRef {
                        id: row.get(0)?,
                        name: row.get(1)?,
                    },
                    duration_ms: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(apps)
    }

    pub fn remove_rule(&self, id: i64) -> Result<(), StoreError> {
        self.conn.execute("DELETE FROM rule WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Where the prompt sent at `at_ms` in one of `sessions` is: its file
    /// and the offset of its line.
    pub fn prompt_line(
        &self,
        sessions: &[SessionRef],
        at_ms: i64,
    ) -> Result<Option<(AgentSource, String, u64)>, StoreError> {
        let mut statement = self.conn.prepare_cached(
            "SELECT path, offset FROM prompt_line WHERE source = ?1 AND session_id = ?2 AND at_ms = ?3
             LIMIT 1",
        )?;
        for session in sessions {
            let found = statement
                .query_row(
                    params![source_text(session.source), session.session_id, at_ms],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
                )
                .optional()?;
            if let Some((path, offset)) = found {
                return Ok(Some((
                    session.source,
                    path,
                    u64::try_from(offset).unwrap_or(0),
                )));
            }
        }
        Ok(None)
    }

    /// The agent settings, or the defaults before the first save.
    pub fn agent_settings(&self) -> Result<AgentSettings, StoreError> {
        Ok(self.setting(AGENT_SETTINGS)?.unwrap_or_default())
    }

    pub fn set_agent_settings(&self, settings: &AgentSettings) -> Result<(), StoreError> {
        self.set_setting(AGENT_SETTINGS, settings)
    }

    /// A stored setting, `None` before the first save.
    pub fn setting<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, StoreError> {
        let value: Option<String> = self
            .conn
            .query_row("SELECT value FROM setting WHERE key = ?1", [key], |row| {
                row.get(0)
            })
            .optional()?;
        Ok(value.map(|text| serde_json::from_str(&text)).transpose()?)
    }

    pub fn set_setting<T: serde::Serialize>(&self, key: &str, value: &T) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO setting (key, value) VALUES (?1, ?2)
             ON CONFLICT (key) DO UPDATE SET value = excluded.value",
            params![key, serde_json::to_string(value)?],
        )?;
        Ok(())
    }

    /// Records a new running run; `None` when another run is still going.
    pub fn start_agent_run(
        &self,
        agent: AgentKind,
        (from_ms, to_ms): (i64, i64),
        now_ms: i64,
    ) -> Result<Option<i64>, StoreError> {
        let inserted = self.conn.execute(
            "INSERT OR IGNORE INTO agent_run (agent, from_ms, to_ms, started_ms, state)
             VALUES (?1, ?2, ?3, ?4, 'running')",
            params![agent.as_str(), from_ms, to_ms, now_ms],
        )?;
        Ok((inserted == 1).then(|| self.conn.last_insert_rowid()))
    }

    /// Ends a running run with the agent's last message or the error.
    pub fn finish_agent_run(
        &self,
        id: i64,
        outcome: Result<&str, &str>,
        now_ms: i64,
    ) -> Result<(), StoreError> {
        let (state, message) = match outcome {
            Ok(summary) => ("done", summary),
            Err(error) => ("failed", error),
        };
        self.conn.execute(
            "UPDATE agent_run SET state = ?2, finished_ms = ?3, message = ?4
             WHERE id = ?1 AND state = 'running'",
            params![id, state, now_ms, message],
        )?;
        Ok(())
    }

    /// Fails runs left running when Memorius stopped.
    pub fn fail_interrupted_runs(&self, now_ms: i64) -> Result<(), StoreError> {
        self.conn.execute(
            "UPDATE agent_run SET state = 'failed', finished_ms = ?1,
             message = 'Memorius stopped during the run'
             WHERE state = 'running'",
            [now_ms],
        )?;
        Ok(())
    }

    /// The newest runs first.
    pub fn agent_runs(&self, limit: u32) -> Result<Vec<AgentRun>, StoreError> {
        let mut statement = self.conn.prepare_cached(
            "SELECT id, agent, from_ms, to_ms, started_ms, state, finished_ms, message
             FROM agent_run ORDER BY id DESC LIMIT ?1",
        )?;
        let runs = statement
            .query_map([limit], |row| {
                let agent: String = row.get(1)?;
                let state: String = row.get(5)?;
                let finished_ms: Option<i64> = row.get(6)?;
                let message: Option<String> = row.get(7)?;
                Ok(AgentRun {
                    id: row.get(0)?,
                    agent: AgentKind::parse(&agent),
                    from_ms: row.get(2)?,
                    to_ms: row.get(3)?,
                    started_ms: row.get(4)?,
                    state: match (state.as_str(), finished_ms, message) {
                        ("done", Some(finished_ms), Some(summary)) => RunState::Done {
                            finished_ms,
                            summary,
                        },
                        ("failed", Some(finished_ms), Some(error)) => {
                            RunState::Failed { finished_ms, error }
                        }
                        _ => RunState::Running,
                    },
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(runs)
    }

    /// Where the last finished proposals run ended.
    pub fn last_proposals_end(&self) -> Result<Option<i64>, StoreError> {
        Ok(self.conn.query_row(
            "SELECT max(to_ms) FROM agent_run WHERE state = 'done'",
            [],
            |row| row.get(0),
        )?)
    }

    /// Replaces the stored work items dated `from_date..=to_date`.
    pub fn replace_tracker_worklogs(
        &self,
        from_date: &str,
        to_date: &str,
        worklogs: &[TrackerWorklog],
    ) -> Result<(), StoreError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "DELETE FROM tracker_worklog WHERE date BETWEEN ?1 AND ?2",
            params![from_date, to_date],
        )?;
        for item in worklogs {
            tx.execute(
                "INSERT OR REPLACE INTO tracker_worklog
                 (id, issue, summary, date, minutes, text, work_type)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    item.id,
                    item.issue,
                    item.summary,
                    item.date,
                    item.minutes,
                    item.text,
                    item.work_type
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Stored work items dated `from_date..=to_date`, oldest first.
    pub fn tracker_worklogs(
        &self,
        from_date: &str,
        to_date: &str,
    ) -> Result<Vec<TrackerWorklog>, StoreError> {
        let mut statement = self.conn.prepare_cached(
            "SELECT id, issue, summary, date, minutes, text, work_type FROM tracker_worklog
             WHERE date BETWEEN ?1 AND ?2 ORDER BY date, issue, id",
        )?;
        let items = statement
            .query_map(params![from_date, to_date], |row| {
                Ok(TrackerWorklog {
                    id: row.get(0)?,
                    issue: row.get(1)?,
                    summary: row.get(2)?,
                    date: row.get(3)?,
                    minutes: row.get(4)?,
                    text: row.get(5)?,
                    work_type: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(items)
    }

    /// An encrypted secret.
    pub fn secret(&self, name: &str) -> Result<Option<SealedSecret>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT nonce, ciphertext FROM secret WHERE name = ?1",
                [name],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?)
    }

    pub fn set_secret(
        &self,
        name: &str,
        nonce: &[u8],
        ciphertext: &[u8],
    ) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO secret (name, nonce, ciphertext) VALUES (?1, ?2, ?3)
             ON CONFLICT (name) DO UPDATE SET nonce = excluded.nonce, ciphertext = excluded.ciphertext",
            params![name, nonce, ciphertext],
        )?;
        Ok(())
    }

    pub fn delete_secret(&self, name: &str) -> Result<(), StoreError> {
        self.conn
            .execute("DELETE FROM secret WHERE name = ?1", [name])?;
        Ok(())
    }

    /// Replaces every stored commit with a fresh scan.
    /// `commits` pairs each commit with its repository's git directory.
    pub fn replace_git_commits(&self, commits: &[(DayCommit, String)]) -> Result<(), StoreError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM git_commit", [])?;
        for (commit, git_dir) in commits {
            tx.execute(
                "INSERT OR REPLACE INTO git_commit (hash, at_ms, subject, repo, branch, git_dir)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    commit.hash,
                    commit.at_ms,
                    commit.subject,
                    commit.repo,
                    commit.branch,
                    git_dir
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Commits in `[from_ms, to_ms)`, oldest first.
    pub fn git_commits_between(
        &self,
        from_ms: i64,
        to_ms: i64,
    ) -> Result<Vec<DayCommit>, StoreError> {
        let mut statement = self.conn.prepare_cached(
            "SELECT hash, at_ms, subject, repo, branch FROM git_commit
             WHERE at_ms >= ?1 AND at_ms < ?2 ORDER BY at_ms, hash",
        )?;
        let commits = statement
            .query_map(params![from_ms, to_ms], |row| {
                Ok(DayCommit {
                    hash: row.get(0)?,
                    at_ms: row.get(1)?,
                    subject: row.get(2)?,
                    repo: row.get(3)?,
                    branch: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(commits)
    }

    /// The git directory of a stored commit's repository.
    pub fn git_commit_dir(&self, hash: &str) -> Result<Option<String>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT git_dir FROM git_commit WHERE hash = ?1",
                [hash],
                |row| row.get(0),
            )
            .optional()?
            .flatten())
    }

    /// Folders the user worked in since `since_ms`: agent session folders
    /// and the documents open in windows, to find their repositories.
    pub fn work_folders(&self, since_ms: i64) -> Result<Vec<String>, StoreError> {
        let mut statement = self.conn.prepare_cached(
            "SELECT DISTINCT cwd FROM context_span WHERE cwd IS NOT NULL AND end_ms > ?1
             UNION SELECT DISTINCT document FROM activity WHERE document IS NOT NULL AND end_ms > ?1",
        )?;
        let folders = statement
            .query_map([since_ms], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(folders)
    }

    /// The sessions the user marked private.
    pub fn private_sessions(&self) -> Result<Vec<SessionRef>, StoreError> {
        let mut statement = self
            .conn
            .prepare_cached("SELECT source, session_id FROM private_session")?;
        let sessions = statement
            .query_map([], |row| {
                let source: String = row.get(0)?;
                Ok(SessionRef {
                    source: if source == "codex" {
                        AgentSource::Codex
                    } else {
                        AgentSource::ClaudeCode
                    },
                    session_id: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(sessions)
    }

    /// Marks `sessions` private, or not private again (undo).
    pub fn set_sessions_private(
        &self,
        sessions: &[SessionRef],
        private: bool,
        now_ms: i64,
    ) -> Result<(), StoreError> {
        let tx = self.conn.unchecked_transaction()?;
        for session in sessions {
            let source = source_text(session.source);
            if private {
                tx.execute(
                    "INSERT OR IGNORE INTO private_session (source, session_id, decided_ms)
                     VALUES (?1, ?2, ?3)",
                    params![source, session.session_id, now_ms],
                )?;
            } else {
                tx.execute(
                    "DELETE FROM private_session WHERE source = ?1 AND session_id = ?2",
                    params![source, session.session_id],
                )?;
            }
        }
        tx.commit()?;
        Ok(())
    }
}

const PROPOSAL_COLUMNS: &str =
    "id, issue, start_ms, end_ms, minutes, work_type, text, agent, created_ms,
     state, decided_ms, work_item_id, last_error, date, revision";

impl Store {
    /// Saves a proposal for an issue and day (`true`). Proposing the same
    /// issue and range again returns the earlier one, and a waiting
    /// proposal of the same issue and day takes the new minutes and text
    /// (`false` for both): agents may safely propose twice.
    pub fn propose(&self, new: NewProposal, now_ms: i64) -> Result<(Proposal, bool), StoreError> {
        let mut new = new.validated()?;
        // Once the tracker's work types are known, only they are accepted,
        // spelled as the tracker spells them.
        if let Some(name) = &new.work_type {
            let known: Vec<String> = self.setting(WORK_TYPES)?.unwrap_or_default();
            if !known.is_empty() {
                let canonical = known
                    .iter()
                    .find(|known| known.eq_ignore_ascii_case(name))
                    .ok_or_else(|| ProposalError::WorkType(name.clone(), known.join(", ")))?;
                new.work_type = Some(canonical.clone());
            }
        }
        let same_range: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM proposal WHERE issue = ?1 AND start_ms = ?2 AND end_ms = ?3",
                params![new.issue, new.start_ms, new.end_ms],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(id) = same_range {
            return Ok((self.proposal(id)?, false));
        }
        let waiting: Option<(i64, i64, i64, u32)> = self
            .conn
            .query_row(
                "SELECT id, start_ms, end_ms, revision FROM proposal
                 WHERE issue = ?1 AND date = ?2 AND state = 'proposed'",
                params![new.issue, new.date],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()?;
        if let Some((id, start_ms, end_ms, revision)) = waiting {
            let (start_ms, end_ms) = (start_ms.min(new.start_ms), end_ms.max(new.end_ms));
            crate::proposal::check_minutes(new.minutes, start_ms, end_ms)?;
            // Only the row read above: one the user approved since then stays as approved.
            let changed = self.conn.execute(
                "UPDATE proposal SET start_ms = ?2, end_ms = ?3, minutes = ?4, work_type = ?5,
                 text = ?6, agent = ?7, revision = revision + 1
                 WHERE id = ?1 AND state = 'proposed' AND revision = ?8",
                params![
                    id,
                    start_ms,
                    end_ms,
                    new.minutes,
                    new.work_type,
                    new.text,
                    new.agent,
                    revision
                ],
            )?;
            if changed == 0 {
                return Err(ProposalError::Changed.into());
            }
            return Ok((self.proposal(id)?, false));
        }
        self.conn.execute(
            "INSERT INTO proposal
             (issue, date, start_ms, end_ms, minutes, work_type, text, agent, created_ms, state)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'proposed')",
            params![
                new.issue,
                new.date,
                new.start_ms,
                new.end_ms,
                new.minutes,
                new.work_type,
                new.text,
                new.agent,
                now_ms
            ],
        )?;
        Ok((self.proposal(self.conn.last_insert_rowid())?, true))
    }

    pub fn proposal(&self, id: i64) -> Result<Proposal, StoreError> {
        self.conn
            .query_row(
                &format!("SELECT {PROPOSAL_COLUMNS} FROM proposal WHERE id = ?1"),
                [id],
                read_proposal,
            )
            .optional()?
            .ok_or(StoreError::NoProposal(id))
    }

    /// Proposals that overlap `[from_ms, to_ms)`, oldest first.
    pub fn proposals_between(&self, from_ms: i64, to_ms: i64) -> Result<Vec<Proposal>, StoreError> {
        let mut statement = self.conn.prepare_cached(&format!(
            "SELECT {PROPOSAL_COLUMNS} FROM proposal WHERE end_ms > ?1 AND start_ms < ?2 ORDER BY start_ms, id"
        ))?;
        let proposals = statement
            .query_map(params![from_ms, to_ms], read_proposal)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(proposals)
    }

    /// The log queue: proposals waiting for the user or the agent, and those
    /// decided since `decided_since_ms` (so a decision can be undone),
    /// newest first.
    pub fn queue_proposals(&self, decided_since_ms: i64) -> Result<Vec<Proposal>, StoreError> {
        let mut statement = self.conn.prepare_cached(&format!(
            "SELECT {PROPOSAL_COLUMNS} FROM proposal
             WHERE state IN ('proposed', 'approved') OR decided_ms > ?1
             ORDER BY start_ms DESC, id DESC"
        ))?;
        let proposals = statement
            .query_map([decided_since_ms], read_proposal)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(proposals)
    }

    /// Proposals waiting for the user or for the agent, newest first.
    pub fn open_proposals(&self) -> Result<Vec<Proposal>, StoreError> {
        let mut statement = self.conn.prepare_cached(&format!(
            "SELECT {PROPOSAL_COLUMNS} FROM proposal WHERE state IN ('proposed', 'approved')
             ORDER BY start_ms DESC, id DESC"
        ))?;
        let proposals = statement
            .query_map([], read_proposal)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(proposals)
    }

    /// Approves the revision of a proposal the user saw, with the user's
    /// edits when given; a proposal an agent changed since then is refused.
    pub fn approve_proposal(
        &self,
        id: i64,
        revision: u32,
        edit: Option<ProposalEdit>,
        now_ms: i64,
    ) -> Result<Proposal, StoreError> {
        // One transaction: no agent can change it between check and write.
        let transaction = self.conn.unchecked_transaction()?;
        let current = self.proposal(id)?;
        if current.revision != revision {
            return Err(ProposalError::Changed.into());
        }
        let state = crate::proposal::approve(&current.state, now_ms)?;
        let edit = edit
            .map(|edit| edit.validated(current.start_ms, current.end_ms))
            .transpose()?
            .unwrap_or(ProposalEdit {
                issue: current.issue,
                minutes: current.minutes,
                work_type: current.work_type,
                text: current.text,
            });
        self.conn.execute(
            "UPDATE proposal SET issue = ?2, minutes = ?3, work_type = ?4, text = ?5 WHERE id = ?1",
            params![id, edit.issue, edit.minutes, edit.work_type, edit.text],
        )?;
        let approved = self.save_state(id, &state, None)?;
        transaction.commit()?;
        Ok(approved)
    }

    pub fn reject_proposal(&self, id: i64, now_ms: i64) -> Result<Proposal, StoreError> {
        let state = crate::proposal::reject(&self.proposal(id)?.state, now_ms)?;
        self.save_state(id, &state, None)
    }

    pub fn undo_proposal(&self, id: i64) -> Result<Proposal, StoreError> {
        let state = crate::proposal::undo(&self.proposal(id)?.state)?;
        self.save_state(id, &state, None)
    }

    pub fn mark_logged(
        &self,
        id: i64,
        work_item_id: &str,
        now_ms: i64,
    ) -> Result<Proposal, StoreError> {
        let state = crate::proposal::mark_logged(
            id,
            &self.proposal(id)?.state,
            work_item_id.trim(),
            now_ms,
        )?;
        self.save_state(id, &state, None)
    }

    pub fn mark_failed(&self, id: i64, error: &str) -> Result<Proposal, StoreError> {
        let current = self.proposal(id)?;
        crate::proposal::check_can_fail(&current.state)?;
        self.save_state(id, &current.state, Some(error))
    }

    fn save_state(
        &self,
        id: i64,
        state: &ProposalState,
        error: Option<&str>,
    ) -> Result<Proposal, StoreError> {
        let (name, decided, work_item) = match state {
            ProposalState::Proposed => ("proposed", None, None),
            ProposalState::Approved { at_ms } => ("approved", Some(*at_ms), None),
            ProposalState::Rejected { at_ms } => ("rejected", Some(*at_ms), None),
            ProposalState::Logged {
                at_ms,
                work_item_id,
            } => ("logged", Some(*at_ms), Some(work_item_id.as_str())),
        };
        self.conn.execute(
            "UPDATE proposal SET state = ?2, decided_ms = ?3, work_item_id = ?4, last_error = ?5 WHERE id = ?1",
            params![id, name, decided, work_item, error],
        )?;
        self.proposal(id)
    }
}

fn read_proposal(row: &Row<'_>) -> rusqlite::Result<Proposal> {
    let state: String = row.get(9)?;
    let decided: Option<i64> = row.get(10)?;
    let at_ms = decided.unwrap_or_default();
    let state = match state.as_str() {
        "approved" => ProposalState::Approved { at_ms },
        "rejected" => ProposalState::Rejected { at_ms },
        "logged" => ProposalState::Logged {
            at_ms,
            work_item_id: row.get::<_, Option<String>>(11)?.unwrap_or_default(),
        },
        _ => ProposalState::Proposed,
    };
    Ok(Proposal {
        id: row.get(0)?,
        issue: row.get(1)?,
        start_ms: row.get(2)?,
        end_ms: row.get(3)?,
        minutes: row.get(4)?,
        work_type: row.get(5)?,
        text: row.get(6)?,
        agent: row.get(7)?,
        created_ms: row.get(8)?,
        state,
        last_error: row.get(12)?,
        date: row.get(13)?,
        revision: row.get(14)?,
    })
}

const AGENT_SETTINGS: &str = "agent";

fn source_text(source: AgentSource) -> &'static str {
    match source {
        AgentSource::ClaudeCode => "claudeCode",
        AgentSource::Codex => "codex",
    }
}

fn read_span(row: &Row<'_>) -> rusqlite::Result<(ContextSpan, String)> {
    let source: String = row.get(1)?;
    Ok((
        ContextSpan {
            source: if source == "codex" {
                AgentSource::Codex
            } else {
                AgentSource::ClaudeCode
            },
            session_id: row.get(2)?,
            title: row.get(3)?,
            cwd: row.get(4)?,
            branch: row.get(5)?,
            start_ms: row.get(6)?,
            end_ms: row.get(7)?,
            prompt_times: Vec::new(),
        },
        row.get(8)?,
    ))
}

fn with_prompts(mut span: ContextSpan, prompts: &str) -> Result<ContextSpan, StoreError> {
    span.prompt_times = serde_json::from_str(prompts)?;
    Ok(span)
}

fn user_version(conn: &Connection) -> Result<i64, rusqlite::Error> {
    conn.query_row("PRAGMA user_version", [], |row| row.get(0))
}

fn unavailable_text(reason: TitleUnavailable) -> &'static str {
    match reason {
        TitleUnavailable::PermissionDenied => "permissionDenied",
        TitleUnavailable::NoWindow => "noWindow",
        TitleUnavailable::Private => "private",
    }
}

fn read_row(row: &Row<'_>) -> rusqlite::Result<ActivityRow> {
    let kind: String = row.get(1)?;
    let kind = match kind.as_str() {
        "app" => {
            let title = match (
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
            ) {
                (Some(text), _) => Title::Known { text },
                (None, Some(reason)) if reason == "permissionDenied" => Title::Unavailable {
                    reason: TitleUnavailable::PermissionDenied,
                },
                (None, Some(reason)) if reason == "private" => Title::Unavailable {
                    reason: TitleUnavailable::Private,
                },
                _ => Title::Unavailable {
                    reason: TitleUnavailable::NoWindow,
                },
            };
            RowKind::App(Window {
                app: AppRef {
                    id: row.get(2)?,
                    name: row.get(3)?,
                },
                title,
                document: row.get(6)?,
                url: row.get(9)?,
            })
        }
        "away" => RowKind::Away,
        _ => RowKind::Locked,
    };
    Ok(ActivityRow {
        id: row.get(0)?,
        start_ms: row.get(7)?,
        end_ms: row.get(8)?,
        kind,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window(app: &str, title: Title) -> RowKind {
        RowKind::App(Window {
            app: AppRef {
                id: format!("com.example.{app}"),
                name: app.to_owned(),
            },
            title,
            document: Some("/tmp/a.fs".to_owned()),
            url: Some("https://youtrack.example/issue/IS-61".to_owned()),
        })
    }

    #[test]
    fn round_trips_every_row_kind() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("m.db")).unwrap();
        let kinds = [
            window(
                "Rider",
                Title::Known {
                    text: "a.fs".into(),
                },
            ),
            window(
                "Rider",
                Title::Unavailable {
                    reason: TitleUnavailable::PermissionDenied,
                },
            ),
            window(
                "Finder",
                Title::Unavailable {
                    reason: TitleUnavailable::NoWindow,
                },
            ),
            RowKind::Away,
            RowKind::Locked,
        ];
        for (i, kind) in kinds.iter().enumerate() {
            let start = i64::try_from(i).unwrap() * 10;
            store.insert(start, start + 10, kind).unwrap();
        }

        let rows = store.rows_between(0, 100).unwrap();

        assert_eq!(
            rows.iter().map(|r| r.kind.clone()).collect::<Vec<_>>(),
            kinds
        );
    }

    #[test]
    fn returns_rows_overlapping_the_range_only() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("m.db")).unwrap();
        store.insert(0, 10, &RowKind::Away).unwrap();
        store.insert(10, 20, &RowKind::Locked).unwrap();
        store.insert(20, 30, &RowKind::Away).unwrap();

        let rows = store.rows_between(10, 20).unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].kind, RowKind::Locked);
    }

    #[test]
    fn rejects_an_empty_bag_of_app_columns() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("m.db")).unwrap();

        let result = store.conn.execute(
            "INSERT INTO activity (kind, start_ms, end_ms) VALUES ('app', 0, 1)",
            [],
        );

        assert!(result.is_err());
    }

    #[test]
    fn saves_session_spans_continues_them_and_renames_the_session() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("m.db")).unwrap();
        let span = |start: i64, end: i64| ContextSpan {
            source: AgentSource::Codex,
            session_id: "t1".into(),
            title: None,
            cwd: Some("/w".into()),
            branch: Some("codex/int-177-batch-2".into()),
            start_ms: start,
            end_ms: end,
            prompt_times: vec![start],
        };
        let mut cursor = SessionCursor {
            offset: 10,
            session_id: Some("t1".into()),
            title: Some("Main".into()),
            ..SessionCursor::default()
        };
        let import = |cursor: &SessionCursor, continues: Option<i64>, spans: &[ContextSpan]| {
            store
                .save_session_import(&SessionImport {
                    path: "/s/rollout.jsonl",
                    source: AgentSource::Codex,
                    modified_ms: 1,
                    cursor,
                    continues_span: continues,
                    spans,
                    file: "/tmp/s.jsonl",
                    prompt_lines: &[],
                })
                .unwrap();
        };
        import(&cursor, None, &[span(0, 100)]);
        let (id, mut last) = store
            .last_context_span(AgentSource::Codex, "t1")
            .unwrap()
            .unwrap();
        last.end_ms = 500;
        cursor.offset = 20;
        cursor.title = Some("INT-177 Job runner batch 2".into());
        import(&cursor, Some(id), &[last, span(5_000, 6_000)]);

        let spans = store.context_spans_between(0, 10_000).unwrap();

        assert_eq!(
            spans
                .iter()
                .map(|s| (s.start_ms, s.end_ms))
                .collect::<Vec<_>>(),
            [(0, 500), (5_000, 6_000)]
        );
        assert!(
            spans
                .iter()
                .all(|s| s.title.as_deref() == Some("INT-177 Job runner batch 2"))
        );
        assert_eq!(spans[0].prompt_times, [0]);
        assert_eq!(
            store
                .session_cursor("/s/rollout.jsonl")
                .unwrap()
                .unwrap()
                .1
                .offset,
            20
        );
    }

    #[test]
    fn stores_rules_with_normalized_issue_keys() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("m.db")).unwrap();
        let private = store
            .add_rule(&RuleKind::PrivateApp {
                app_id: "net.whatsapp.WhatsApp".into(),
            })
            .unwrap();
        store
            .add_rule(&RuleKind::HostIssue {
                host: " orders.example.com ".into(),
                issue: "sel-54".into(),
            })
            .unwrap();
        store.remove_rule(private).unwrap();

        let rules: Vec<RuleKind> = store.rules().unwrap().into_iter().map(|r| r.rule).collect();

        assert_eq!(
            rules,
            [RuleKind::HostIssue {
                host: "orders.example.com".into(),
                issue: "SEL-54".into()
            }]
        );
    }

    #[test]
    fn keeps_activity_through_the_table_rebuild() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("m.db");
        let conn = Connection::open(&path).unwrap();
        for sql in &MIGRATIONS[..3] {
            conn.execute_batch(sql).unwrap();
        }
        conn.pragma_update(None, "user_version", 3).unwrap();
        conn.execute(
            "INSERT INTO activity (kind, start_ms, end_ms) VALUES ('away', 0, 10)",
            [],
        )
        .unwrap();
        drop(conn);

        let store = Store::open(&path).unwrap();
        store
            .insert(
                10,
                20,
                &window(
                    "WhatsApp",
                    Title::Unavailable {
                        reason: TitleUnavailable::Private,
                    },
                ),
            )
            .unwrap();

        assert_eq!(store.rows_between(0, 100).unwrap().len(), 2);
    }

    #[test]
    fn search_hides_private_apps_and_takes_wildcards_literally() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("m.db")).unwrap();
        let known = |text: &str| Title::Known { text: text.into() };
        store
            .insert(0, 10, &window("Rider", known("IS-61 CatalogSource.fs")))
            .unwrap();
        store
            .insert(10, 30, &window("Rider", known("IS-61 CatalogSource.fs")))
            .unwrap();
        store
            .insert(30, 40, &window("WhatsApp", known("IS-61 chat with Ann")))
            .unwrap();
        store
            .insert(40, 50, &window("Dia", known("Report 100% done")))
            .unwrap();
        store
            .add_rule(&RuleKind::PrivateApp {
                app_id: "com.example.WhatsApp".into(),
            })
            .unwrap();
        store
            .propose(
                NewProposal {
                    issue: "IS-61".into(),
                    date: "1970-01-01".into(),
                    start_ms: 0,
                    end_ms: 3_600_000,
                    minutes: 40,
                    work_type: None,
                    text: "Catalog read model".into(),
                    agent: "Codex".into(),
                },
                1,
            )
            .unwrap();

        let found = store.search("is-61", 0).unwrap();
        assert_eq!(
            found
                .windows
                .iter()
                .map(|hit| (hit.app.name.as_str(), hit.total_ms, hit.spans.len()))
                .collect::<Vec<_>>(),
            vec![("Dia", 10, 1), ("Rider", 30, 2)]
        );
        assert_eq!(found.issues.len(), 1);
        assert_eq!(found.issues[0].worklogs[0].minutes, 40);

        assert_eq!(store.search("0%", 0).unwrap().windows.len(), 1);
        assert_eq!(store.search("0_d", 0).unwrap().windows.len(), 0);
        assert_eq!(store.search("is-61", 60).unwrap().windows.len(), 0);
        assert_eq!(store.search("x", 0).unwrap(), SearchResults::default());
    }

    #[test]
    fn proposals_are_unique_per_range_and_follow_the_approval_flow() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("m.db")).unwrap();
        let new = || NewProposal {
            issue: "int-290".into(),
            date: "1970-01-01".into(),
            start_ms: 0,
            end_ms: 3_600_000,
            minutes: 50,
            work_type: Some("Development".into()),
            text: "Skip unchanged employees".into(),
            agent: "Codex".into(),
        };

        let (first, created) = store.propose(new(), 1).unwrap();
        let (again, created_again) = store.propose(new(), 2).unwrap();
        assert!(created && !created_again);
        assert_eq!(first.id, again.id);

        let edit = ProposalEdit {
            issue: "INT-290".into(),
            minutes: 45,
            work_type: None,
            text: "Skip unchanged employees and cost codes".into(),
        };
        let approved = store
            .approve_proposal(first.id, first.revision, Some(edit), 3)
            .unwrap();
        assert_eq!(
            (approved.minutes, approved.state.clone()),
            (45, ProposalState::Approved { at_ms: 3 })
        );
        assert_eq!(store.open_proposals().unwrap().len(), 1);

        let failed = store.mark_failed(first.id, "YouTrack said 403").unwrap();
        assert_eq!(failed.last_error.as_deref(), Some("YouTrack said 403"));
        let logged = store.mark_logged(first.id, "2-48211", 4).unwrap();
        assert_eq!(logged.last_error, None);
        assert!(matches!(logged.state, ProposalState::Logged { .. }));
        assert!(store.open_proposals().unwrap().is_empty());
        assert!(matches!(
            store.undo_proposal(first.id),
            Err(StoreError::Proposal(ProposalError::Transition { .. }))
        ));
    }

    #[test]
    fn read_only_open_refuses_a_newer_schema() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("m.db");
        let store = Store::open(&path).unwrap();
        store.conn.pragma_update(None, "user_version", 99).unwrap();
        drop(store);

        let result = Store::open_read_only(&path);

        assert!(matches!(
            result,
            Err(StoreError::SchemaTooNew { found: 99, .. })
        ));
    }

    #[test]
    fn marking_sessions_private_can_be_undone() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("m.db")).unwrap();
        let session = |id: &str| SessionRef {
            source: AgentSource::Codex,
            session_id: id.into(),
        };

        store
            .set_sessions_private(&[session("a"), session("b")], true, 1)
            .unwrap();
        store
            .set_sessions_private(&[session("a")], true, 2)
            .unwrap();
        store
            .set_sessions_private(&[session("b")], false, 3)
            .unwrap();

        assert_eq!(store.private_sessions().unwrap(), [session("a")]);
    }

    #[test]
    fn agent_settings_default_until_saved() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("m.db")).unwrap();
        assert_eq!(store.agent_settings().unwrap(), AgentSettings::default());

        let codex = AgentSettings {
            agent: AgentKind::Codex,
            daily: true,
            ..AgentSettings::default()
        };
        store.set_agent_settings(&codex).unwrap();

        assert_eq!(store.agent_settings().unwrap(), codex);
    }

    #[test]
    fn approving_refuses_a_proposal_an_agent_changed_after_the_user_saw_it() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("m.db")).unwrap();
        let new = |minutes: u32, start_ms: i64| NewProposal {
            issue: "INT-290".into(),
            date: "1970-01-01".into(),
            start_ms,
            end_ms: start_ms + 3 * 3_600_000,
            minutes,
            work_type: None,
            text: "Order sync".into(),
            agent: "Codex".into(),
        };
        let (seen, _) = store.propose(new(30, 0), 1).unwrap();
        let (changed, _) = store.propose(new(120, 60_000), 2).unwrap();

        assert_eq!(changed.revision, seen.revision + 1);
        assert!(matches!(
            store.approve_proposal(seen.id, seen.revision, None, 3),
            Err(StoreError::Proposal(ProposalError::Changed))
        ));
        let approved = store
            .approve_proposal(seen.id, changed.revision, None, 4)
            .unwrap();
        assert_eq!(approved.minutes, 120);
    }

    #[test]
    fn one_agent_run_at_a_time_and_its_end_is_kept() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("m.db")).unwrap();

        let first = store
            .start_agent_run(AgentKind::ClaudeCode, (10, 20), 1)
            .unwrap()
            .unwrap();
        let second = store
            .start_agent_run(AgentKind::ClaudeCode, (20, 30), 2)
            .unwrap();
        store.finish_agent_run(first, Ok("2 proposals"), 3).unwrap();
        let third = store
            .start_agent_run(AgentKind::Codex, (20, 30), 4)
            .unwrap()
            .unwrap();
        store.fail_interrupted_runs(5).unwrap();

        assert_eq!(second, None);
        assert_eq!(store.last_proposals_end().unwrap(), Some(20));
        let runs = store.agent_runs(10).unwrap();
        assert_eq!(
            runs.iter().map(|run| run.id).collect::<Vec<_>>(),
            [third, first]
        );
        assert_eq!(
            runs[0].state,
            RunState::Failed {
                finished_ms: 5,
                error: "Memorius stopped during the run".into()
            }
        );
        assert_eq!(
            runs[1].state,
            RunState::Done {
                finished_ms: 3,
                summary: "2 proposals".into()
            }
        );
    }

    #[test]
    fn a_tracker_sync_replaces_only_its_own_days() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("m.db")).unwrap();
        let item = |id: &str, date: &str| TrackerWorklog {
            id: id.into(),
            issue: "SEL-78".into(),
            summary: None,
            date: date.into(),
            minutes: 30,
            text: None,
            work_type: None,
        };
        store
            .replace_tracker_worklogs(
                "2026-10-01",
                "2026-10-31",
                &[item("a", "2026-10-01"), item("b", "2026-10-15")],
            )
            .unwrap();

        store
            .replace_tracker_worklogs("2026-10-15", "2026-10-16", &[item("c", "2026-10-16")])
            .unwrap();

        let ids: Vec<String> = store
            .tracker_worklogs("2026-10-01", "2026-10-31")
            .unwrap()
            .into_iter()
            .map(|item| item.id)
            .collect();
        assert_eq!(ids, ["a", "c"]);
    }

    #[test]
    fn a_waiting_proposal_takes_a_later_proposal_for_its_issue_and_day() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("m.db")).unwrap();
        let new = |start: i64, end: i64, minutes: u32, text: &str| NewProposal {
            issue: "SEL-78".into(),
            date: "2026-10-16".into(),
            start_ms: start * 60_000,
            end_ms: end * 60_000,
            minutes,
            work_type: None,
            text: text.into(),
            agent: "Claude Code".into(),
        };

        let (first, created) = store.propose(new(0, 60, 30, "Auth0 research"), 1).unwrap();
        let (merged, created_again) = store
            .propose(new(60, 180, 75, "Auth0 research and sign-in test"), 2)
            .unwrap();

        assert!(created && !created_again);
        assert_eq!(merged.id, first.id);
        assert_eq!(
            (
                merged.minutes,
                merged.start_ms,
                merged.end_ms,
                merged.text.as_str()
            ),
            (75, 0, 180 * 60_000, "Auth0 research and sign-in test")
        );
    }

    #[test]
    fn the_migration_merges_waiting_proposals_per_issue_and_day() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("m.db");
        let conn = Connection::open(&path).unwrap();
        for sql in &MIGRATIONS[..13] {
            conn.execute_batch(sql).unwrap();
        }
        conn.pragma_update(None, "user_version", 13).unwrap();
        // Noon UTC keeps the local day the same in any time zone.
        let noon: i64 = 1_792_065_600_000;
        let insert = |start: i64, minutes: i64, text: &str, state: &str| {
            conn.execute(
                "INSERT INTO proposal (issue, start_ms, end_ms, minutes, text, agent, created_ms,
                 state, decided_ms) VALUES ('SEL-78', ?1, ?2, ?3, ?4, 'a', 0, ?5,
                 CASE WHEN ?5 = 'proposed' THEN NULL ELSE 1 END)",
                params![start, start + 900_000, minutes, text, state],
            )
            .unwrap();
        };
        insert(noon + 3_600_000, 20, "Second", "proposed");
        insert(noon, 10, "First", "proposed");
        insert(noon + 7_200_000, 15, "Approved", "approved");
        drop(conn);

        let store = Store::open(&path).unwrap();

        let mut proposals = store.proposals_between(0, i64::MAX).unwrap();
        proposals.sort_by_key(|proposal| proposal.minutes);
        assert_eq!(
            proposals
                .iter()
                .map(|p| (p.minutes, p.text.as_str(), p.date.len()))
                .collect::<Vec<_>>(),
            [(15, "Approved", 10), (30, "First; Second", 10)]
        );
        assert_eq!(proposals[1].start_ms, noon);
    }

    #[test]
    fn a_proposal_takes_only_the_trackers_work_types_once_known() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("m.db")).unwrap();
        let new = |work_type: &str, date: &str| NewProposal {
            issue: "SEL-3".into(),
            date: date.into(),
            start_ms: if date.ends_with("20") { 86_400_000 } else { 0 },
            end_ms: if date.ends_with("20") {
                90_000_000
            } else {
                3_600_000
            },
            minutes: 30,
            work_type: Some(work_type.into()),
            text: "Planning".into(),
            agent: "Claude Code".into(),
        };
        store.propose(new("Meeting", "2026-10-19"), 1).unwrap();
        store
            .set_setting(WORK_TYPES, &["Development", "Investigation"])
            .unwrap();

        let refused = store.propose(new("Research", "2026-10-20"), 2);
        let (accepted, _) = store
            .propose(new("investigation", "2026-10-20"), 3)
            .unwrap();

        assert!(matches!(
            refused,
            Err(StoreError::Proposal(ProposalError::WorkType(..)))
        ));
        assert_eq!(accepted.work_type.as_deref(), Some("Investigation"));
    }

    #[test]
    fn a_commit_scan_replaces_the_stored_commits() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("m.db")).unwrap();
        let commit = |hash: &str, at_ms: i64| DayCommit {
            hash: hash.into(),
            at_ms,
            subject: "x".into(),
            repo: "memorius".into(),
            branch: None,
        };
        let dir_of = |commit: DayCommit| (commit, "/r/.git".to_owned());
        store
            .replace_git_commits(&[dir_of(commit("a", 10)), dir_of(commit("b", 20))])
            .unwrap();

        store
            .replace_git_commits(&[dir_of(commit("c", 15))])
            .unwrap();
        assert_eq!(
            store.git_commit_dir("c").unwrap().as_deref(),
            Some("/r/.git")
        );

        let hashes: Vec<String> = store
            .git_commits_between(0, 100)
            .unwrap()
            .into_iter()
            .map(|commit| commit.hash)
            .collect();
        assert_eq!(hashes, ["c"]);
    }
}
