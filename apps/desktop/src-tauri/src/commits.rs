//! Reads the user's own commits from the repositories they work in, every
//! five minutes, for the Day view's Commits lane.
#![allow(
    clippy::needless_pass_by_value,
    reason = "Tauri injects command state by value"
)]

use std::path::{Path, PathBuf};
use std::time::Duration;

use memorius_core::store::Store;
use memorius_sources::commits;
use memorius_sources::import::expand_home;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use ts_rs::TS;

use crate::commands::Database;
use crate::recording::now_ms;

const SETTINGS: &str = "git";
const SCAN_STATE: &str = "gitScan";
const SCAN_EVERY: Duration = Duration::from_mins(5);
/// Folders seen in the last 90 days point to the repositories to read.
const FOLDERS_DAYS_MS: i64 = 90 * 24 * 3_600_000;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct GitSettings {
    /// Folders searched for repositories (a few levels deep); `~` is the
    /// home folder. Repositories in folders the user worked in count too.
    pub roots: Vec<String>,
}

/// What the last scan found.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GitScan {
    #[ts(type = "number")]
    pub at_ms: i64,
    pub repositories: u32,
    pub commits: u32,
    /// The e-mail addresses from `git config` whose commits count.
    pub emails: Vec<String>,
    /// Folders with several of the found repositories that no root covers,
    /// written with `~`.
    #[serde(default)]
    pub suggestions: Vec<String>,
}

pub struct Commits {
    database: PathBuf,
}

impl Commits {
    pub fn new(database: &Path) -> Self {
        Self {
            database: database.to_owned(),
        }
    }

    fn scan(&self) -> Result<GitScan, String> {
        let store = Store::open(&self.database).map_err(|error| error.to_string())?;
        let settings: GitSettings = store
            .setting(SETTINGS)
            .map_err(|error| error.to_string())?
            .unwrap_or_default();
        let folders: Vec<PathBuf> = store
            .work_folders(now_ms() - FOLDERS_DAYS_MS)
            .map_err(|error| error.to_string())?
            .into_iter()
            .map(PathBuf::from)
            .collect();
        let roots: Vec<PathBuf> = settings
            .roots
            .iter()
            .filter_map(|root| expand_home(root))
            .collect();
        let repositories = commits::repositories(&roots, &folders);
        let emails = commits::user_emails(&repositories);
        let mut found: Vec<_> = repositories
            .iter()
            .flat_map(|repository| {
                let git_dir = repository.display().to_string();
                commits::read_commits(repository, &emails)
                    .into_iter()
                    .map(move |commit| (commit, git_dir.clone()))
            })
            .collect();
        found.sort_by_key(|(commit, _)| commit.at_ms);
        store
            .replace_git_commits(&found)
            .map_err(|error| error.to_string())?;
        let mut emails: Vec<String> = emails.into_iter().collect();
        emails.sort();
        let scan = GitScan {
            at_ms: now_ms(),
            repositories: u32::try_from(repositories.len()).unwrap_or(u32::MAX),
            commits: u32::try_from(found.len()).unwrap_or(u32::MAX),
            emails,
            suggestions: commits::suggested_roots(&repositories, &roots)
                .iter()
                .map(|folder| with_tilde(folder))
                .collect(),
        };
        store
            .set_setting(SCAN_STATE, &scan)
            .map_err(|error| error.to_string())?;
        Ok(scan)
    }
}

/// `~/x` for a folder in the home folder, as the user would type it.
fn with_tilde(path: &Path) -> String {
    std::env::home_dir()
        .and_then(|home| path.strip_prefix(home).ok().map(Path::to_owned))
        .map_or_else(
            || path.display().to_string(),
            |rest| format!("~/{}", rest.display()),
        )
}

/// Scans now and then every five minutes.
pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        loop {
            if let Err(error) = app.state::<Commits>().scan() {
                eprintln!("memorius: reading git commits failed: {error}");
            }
            std::thread::sleep(SCAN_EVERY);
        }
    });
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GitStatus {
    pub settings: GitSettings,
    pub scan: Option<GitScan>,
}

#[tauri::command]
pub fn git_status(database: State<'_, Database>) -> Result<GitStatus, String> {
    let store = Store::open(&database.0).map_err(|error| error.to_string())?;
    Ok(GitStatus {
        settings: store
            .setting(SETTINGS)
            .map_err(|error| error.to_string())?
            .unwrap_or_default(),
        scan: store
            .setting(SCAN_STATE)
            .map_err(|error| error.to_string())?,
    })
}

/// Saves the folders and reads the commits again.
/// Off the main thread: it walks folders and reads reflogs.
#[tauri::command(async)]
pub fn save_git_settings(
    database: State<'_, Database>,
    commits: State<'_, Commits>,
    settings: GitSettings,
) -> Result<GitScan, String> {
    Store::open(&database.0)
        .and_then(|store| store.set_setting(SETTINGS, &settings))
        .map_err(|error| error.to_string())?;
    commits.scan()
}

/// A commit's message, changed files and GitHub link, read on hover.
/// Off the main thread: it runs git.
#[tauri::command(async)]
pub fn commit_details(
    database: State<'_, Database>,
    hash: String,
) -> Result<commits::CommitDetails, String> {
    let git_dir = Store::open(&database.0)
        .and_then(|store| store.git_commit_dir(&hash))
        .map_err(|error| error.to_string())?
        .ok_or("Memorius does not know this commit's repository")?;
    commits::commit_details(Path::new(&git_dir), &hash)
}

/// Opens a web link in the default browser: only a well-formed https URL,
/// handed to the OS opener as one argument, never through a shell.
#[tauri::command]
pub fn open_link(url: String) -> Result<(), String> {
    let url = url::Url::parse(&url).map_err(|_| "Not a valid link".to_owned())?;
    if url.scheme() != "https" || url.host_str().is_none() {
        return Err("Only https links open".into());
    }
    let opener = if cfg!(windows) { "explorer" } else { "open" };
    std::process::Command::new(opener)
        .arg(url.as_str())
        .spawn()
        .map(drop)
        .map_err(|error| error.to_string())
}
