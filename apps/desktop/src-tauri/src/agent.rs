//! Runs the user's agent CLI to propose and log worklogs, one run at a time,
//! and starts the daily proposals run.
#![allow(
    clippy::needless_pass_by_value,
    reason = "Tauri injects command state by value"
)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use jiff::Zoned;
use memorius_core::agent::{AgentKind, AgentRun, AgentSettings, RunRange};
use memorius_core::store::{Store, StoreError};
use memorius_core::tracker::WORK_TYPES;
use memorius_sources::{agent, import};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use ts_rs::TS;

use crate::commands::Database;
use crate::recording::now_ms;

const TIMEOUT: Duration = Duration::from_mins(15);
/// The local date of the last daily run, so it starts once a day.
const DAILY_DATE: &str = "agentDailyDate";

pub struct AgentRunner {
    database: PathBuf,
    stop: Arc<AtomicBool>,
    path: OnceLock<Option<String>>,
}

impl AgentRunner {
    pub fn new(database: &Path) -> Result<Self, StoreError> {
        Store::open(database)?.fail_interrupted_runs(now_ms())?;
        Ok(Self {
            database: database.to_owned(),
            stop: Arc::new(AtomicBool::new(false)),
            path: OnceLock::new(),
        })
    }

    fn path(&self) -> Option<&str> {
        self.path.get_or_init(login_path).as_deref()
    }

    /// Starts a run in the background that proposes worklogs for `range` up
    /// to `until_ms`.
    fn start(&self, range: RunRange, until_ms: i64) -> Result<(), String> {
        let store = Store::open(&self.database).map_err(|error| error.to_string())?;
        let settings = store.agent_settings().map_err(|error| error.to_string())?;
        let range = agent::run_range(
            range,
            until_ms,
            store
                .last_proposals_end()
                .map_err(|error| error.to_string())?,
        );
        let work_types = work_types(&store)?;
        let command = agent::command(
            &settings,
            &agent::prompt(&settings, range, &work_types),
            &bundled_cli(),
        );
        let id = store
            .start_agent_run(settings.agent, range, now_ms())
            .map_err(|error| error.to_string())?
            .ok_or("Another agent run is still going")?;
        let child = match agent::spawn(&command, self.path()) {
            Ok(child) => child,
            Err(error) => {
                let message = format!("Cannot start {}: {error}", command.program);
                finish(&self.database, id, Err(&message));
                return Err(message);
            }
        };
        self.stop.store(false, Ordering::Relaxed);
        let stop = Arc::clone(&self.stop);
        let database = self.database.clone();
        std::thread::spawn(move || {
            let outcome = agent::wait(child, TIMEOUT, &|| stop.load(Ordering::Relaxed))
                .and_then(|finished| agent::outcome(settings.agent, &finished));
            finish(&database, id, outcome.as_deref().map_err(String::as_str));
        });
        Ok(())
    }

    /// Starts the daily run when it is due; called every minute, so a run
    /// missed while the Mac slept starts after it wakes.
    fn tick(&self) -> Result<(), String> {
        let store = Store::open(&self.database).map_err(|error| error.to_string())?;
        let settings = store.agent_settings().map_err(|error| error.to_string())?;
        let now = Zoned::now();
        let last: Option<String> = store
            .setting(DAILY_DATE)
            .map_err(|error| error.to_string())?;
        if !agent::daily_due(settings.daily, &settings.daily_time, &now, last.as_deref()) {
            return Ok(());
        }
        store
            .set_setting(DAILY_DATE, &now.date().to_string())
            .map_err(|error| error.to_string())?;
        self.start(settings.default_range, now_ms())
    }
}

/// The tracker's work types from the last sync, so the agent picks one.
fn work_types(store: &Store) -> Result<Vec<String>, String> {
    Ok(store
        .setting(WORK_TYPES)
        .map_err(|error| error.to_string())?
        .unwrap_or_default())
}

fn finish(database: &Path, id: i64, outcome: Result<&str, &str>) {
    if let Err(error) =
        Store::open(database).and_then(|store| store.finish_agent_run(id, outcome, now_ms()))
    {
        eprintln!("memorius: cannot record the end of agent run {id}: {error}");
    }
}

/// The login shell's `PATH`: an app started from the Dock gets a short one,
/// without the folders where agent CLIs and `memorius` are installed.
fn login_path() -> Option<String> {
    if cfg!(windows) {
        return None;
    }
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into());
    let output = Command::new(shell)
        .args(["-ilc", "printf '<path>%s</path>' \"$PATH\""])
        .stdin(std::process::Stdio::null())
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let (_, rest) = text.rsplit_once("<path>")?;
    let (path, _) = rest.split_once("</path>")?;
    (!path.is_empty()).then(|| path.to_owned())
}

fn find(path: Option<&str>, program: &str) -> Option<String> {
    let path = path
        .map(str::to_owned)
        .or_else(|| std::env::var("PATH").ok())?;
    // Windows installs agent CLIs as `claude.cmd` or `codex.exe`.
    let names: &[String] = if cfg!(windows) {
        &[
            format!("{program}.exe"),
            format!("{program}.cmd"),
            program.to_owned(),
        ]
    } else {
        &[program.to_owned()]
    };
    std::env::split_paths(&path)
        .flat_map(|dir| names.iter().map(move |name| dir.join(name)))
        .find(|candidate| candidate.is_file())
        .map(|found| found.display().to_string())
}

/// Starts the minute check for the daily run.
pub fn start_schedule(app: AppHandle) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_mins(1));
            if let Err(error) = app.state::<AgentRunner>().tick() {
                eprintln!("memorius: the daily agent run did not start: {error}");
            }
        }
    });
}

/// Where the agent CLI and `memorius` (which the agent starts) were found.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AgentCheck {
    /// `None` for a custom command.
    pub program: Option<String>,
    pub program_path: Option<String>,
    pub memorius_path: Option<String>,
}

#[tauri::command]
pub fn agent_settings(database: State<'_, Database>) -> Result<AgentSettings, String> {
    Store::open(&database.0)
        .and_then(|store| store.agent_settings())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_agent_settings(
    database: State<'_, Database>,
    settings: AgentSettings,
) -> Result<(), String> {
    Store::open(&database.0)
        .and_then(|store| store.set_agent_settings(&settings))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn agent_runs(database: State<'_, Database>) -> Result<Vec<AgentRun>, String> {
    Store::open(&database.0)
        .and_then(|store| store.agent_runs(10))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn start_agent_run(runner: State<'_, AgentRunner>, range: RunRange) -> Result<(), String> {
    runner.start(range, now_ms())
}

/// The prompt a run would get now, shown in Settings.
#[tauri::command]
pub fn agent_prompt(database: State<'_, Database>) -> Result<String, String> {
    let store = Store::open(&database.0).map_err(|error| error.to_string())?;
    let settings = store.agent_settings().map_err(|error| error.to_string())?;
    let range = agent::run_range(
        settings.default_range,
        now_ms(),
        store
            .last_proposals_end()
            .map_err(|error| error.to_string())?,
    );
    Ok(agent::prompt(&settings, range, &work_types(&store)?))
}

#[tauri::command]
pub fn stop_agent_run(runner: State<'_, AgentRunner>) {
    runner.stop.store(true, Ordering::Relaxed);
}

/// Off the main thread: the first call asks the login shell for its PATH.
#[tauri::command(async)]
pub fn agent_check(
    runner: State<'_, AgentRunner>,
    database: State<'_, Database>,
) -> Result<AgentCheck, String> {
    let settings = Store::open(&database.0)
        .and_then(|store| store.agent_settings())
        .map_err(|error| error.to_string())?;
    let program = agent::program_name(&settings);
    let path = runner.path();
    Ok(AgentCheck {
        program: program.map(str::to_owned),
        program_path: program.and_then(|name| find(path, name)),
        memorius_path: find(path, "memorius"),
    })
}

/// Whether the agent CLI can run: found, and signed in to its account.
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum AgentAuth {
    /// A custom command: Memorius cannot tell.
    Custom,
    Missing,
    SignedOut,
    SignedIn,
}

/// The CLI's sign-in command and its status check.
fn auth_args(program: &str) -> (&'static [&'static str], &'static [&'static str]) {
    if program == "claude" {
        (&["auth", "login"], &["auth", "status", "--json"])
    } else {
        (&["login"], &["login", "status"])
    }
}

/// The agent program from Settings and where it was found.
fn agent_program(
    runner: &AgentRunner,
    database: &Database,
) -> Result<Option<(&'static str, Option<String>)>, String> {
    let settings = Store::open(&database.0)
        .and_then(|store| store.agent_settings())
        .map_err(|error| error.to_string())?;
    Ok(agent::program_name(&settings).map(|name| (name, find(runner.path(), name))))
}

/// Off the main thread: it starts the agent CLI.
#[tauri::command(async)]
pub fn agent_auth(
    runner: State<'_, AgentRunner>,
    database: State<'_, Database>,
) -> Result<AgentAuth, String> {
    let Some((name, found)) = agent_program(&runner, &database)? else {
        return Ok(AgentAuth::Custom);
    };
    let Some(program) = found else {
        return Ok(AgentAuth::Missing);
    };
    let mut status = Command::new(program);
    status
        .args(auth_args(name).1)
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    if let Some(path) = runner.path() {
        status.env("PATH", path);
    }
    let output = status.output().map_err(|error| error.to_string())?;
    let signed_in = if name == "claude" {
        serde_json::from_slice::<serde_json::Value>(&output.stdout)
            .ok()
            .and_then(|json| json.get("loggedIn")?.as_bool())
            .unwrap_or(false)
    } else {
        output.status.success()
    };
    Ok(if signed_in {
        AgentAuth::SignedIn
    } else {
        AgentAuth::SignedOut
    })
}

/// Opens a terminal with the agent's own sign-in, which needs the user's
/// browser and, for Claude Code, a pasted code.
#[tauri::command]
pub fn agent_sign_in(
    runner: State<'_, AgentRunner>,
    database: State<'_, Database>,
) -> Result<(), String> {
    let (name, found) =
        agent_program(&runner, &database)?.ok_or("A custom command signs in on its own")?;
    let program = found.ok_or_else(|| format!("{name} is not installed"))?;
    let login = auth_args(name).0;
    sign_in_window(&program, login, runner.path())
}

/// Windows: the program in its own console window; no shell parses its path.
#[cfg(windows)]
fn sign_in_window(program: &str, login: &[&str], path: Option<&str>) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NEW_CONSOLE: u32 = 0x10;
    let mut command = Command::new(program);
    if let Some(path) = path {
        command.env("PATH", path);
    }
    command
        .args(login)
        .creation_flags(CREATE_NEW_CONSOLE)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

/// macOS: a `.command` script that Terminal opens.
#[cfg(not(windows))]
fn sign_in_window(program: &str, login: &[&str], path: Option<&str>) -> Result<(), String> {
    let quote = |text: &str| format!("'{}'", text.replace('\'', r"'\''"));
    let script = std::env::temp_dir().join("memorius-sign-in.command");
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\n{}{} {}\n",
            path.map_or_else(String::new, |path| format!("export PATH={}\n", quote(path))),
            quote(program),
            login.join(" ")
        ),
    )
    .map_err(|error| error.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755))
            .map_err(|error| error.to_string())?;
    }
    Command::new("open")
        .arg(&script)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

/// The `memorius` CLI shipped next to the app's executable; `memorius` on
/// the PATH for builds without it (`pnpm dev`).
fn bundled_cli() -> String {
    let name = if cfg!(windows) {
        "memorius.exe"
    } else {
        "memorius"
    };
    std::env::current_exe()
        .ok()
        .and_then(|exe| Some(exe.parent()?.join(name)))
        .filter(|cli| cli.is_file())
        .map_or_else(|| "memorius".to_owned(), |cli| cli.display().to_string())
}

/// Makes the bundled CLI the `memorius` command in a terminal: a link in
/// `~/.local/bin` (macOS), the app's folder on the user PATH (Windows).
/// Returns what the user still has to do, if anything.
#[tauri::command]
pub fn install_cli(runner: State<'_, AgentRunner>) -> Result<String, String> {
    let cli = PathBuf::from(bundled_cli());
    if !cli.is_absolute() {
        return Err("This build has no memorius CLI inside".into());
    }
    if cfg!(windows) {
        let folder = cli.parent().ok_or("No folder for the CLI")?;
        // The folder goes in as data, never as script text.
        let script = "$d = $env:MEMORIUS_DIR; $p = [Environment]::GetEnvironmentVariable('Path','User'); \
             if (-not ($p -split ';' -contains $d)) { \
             [Environment]::SetEnvironmentVariable('Path', ($p.TrimEnd(';') + ';' + $d), 'User') }";
        let status = Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .env("MEMORIUS_DIR", folder)
            .status()
            .map_err(|error| error.to_string())?;
        return if status.success() {
            Ok("Open a new terminal to use memorius.".into())
        } else {
            Err("Could not add Memorius to your PATH".into())
        };
    }
    let bin = std::env::home_dir()
        .ok_or("No home folder")?
        .join(".local")
        .join("bin");
    std::fs::create_dir_all(&bin).map_err(|error| error.to_string())?;
    let link = bin.join("memorius");
    // Replace an earlier link, never a file the user put there.
    if link.is_symlink() {
        std::fs::remove_file(&link).map_err(|error| error.to_string())?;
    } else if link.exists() {
        return Err(format!("{} already exists", link.display()));
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(&cli, &link).map_err(|error| error.to_string())?;
    let on_path = runner
        .path()
        .is_some_and(|path| std::env::split_paths(path).any(|dir| dir == bin));
    Ok(if on_path {
        "Installed in ~/.local/bin.".into()
    } else {
        "Installed. Add ~/.local/bin to your PATH: echo 'export PATH=\"$HOME/.local/bin:$PATH\"' >> ~/.zshrc"
            .into()
    })
}

/// The worklogs skill agents read; copied into their skill folders.
const SKILL: &str = include_str!("../../../../skills/memorius-worklogs/SKILL.md");

/// Registers the bundled CLI as the `memorius` MCP server of Claude Code or
/// Codex for the user's own sessions, and copies the worklogs skill.
#[tauri::command(async)]
pub fn add_to_agent(
    runner: State<'_, AgentRunner>,
    database: State<'_, Database>,
    agent: AgentKind,
) -> Result<(), String> {
    let settings = Store::open(&database.0)
        .and_then(|store| store.agent_settings())
        .map_err(|error| error.to_string())?;
    let (program, home, remove, add): (&str, PathBuf, &[&str], &[&str]) = match agent {
        AgentKind::ClaudeCode => (
            "claude",
            import::expand_home(&settings.claude_home)
                .or_else(|| std::env::home_dir().map(|home| home.join(".claude")))
                .ok_or("No home folder")?,
            &["mcp", "remove", "--scope", "user", "memorius"],
            &["mcp", "add", "--scope", "user", "memorius", "--"],
        ),
        AgentKind::Codex => (
            "codex",
            import::expand_home(&settings.codex_home)
                .or_else(|| std::env::home_dir().map(|home| home.join(".codex")))
                .ok_or("No home folder")?,
            &["mcp", "remove", "memorius"],
            &["mcp", "add", "memorius", "--"],
        ),
        AgentKind::Custom => return Err("A custom command has its own MCP setup".into()),
    };
    let found =
        find(runner.path(), program).ok_or_else(|| format!("{program} is not installed"))?;
    let run = |args: &[&str]| {
        let mut command = Command::new(&found);
        command.args(args).stdin(std::process::Stdio::null());
        if let Some(path) = runner.path() {
            command.env("PATH", path);
        }
        command.output().map_err(|error| error.to_string())
    };
    // An earlier registration may point to another memorius; replace it.
    run(remove)?;
    let cli = bundled_cli();
    let added = run(&[add, &[cli.as_str(), "mcp"]].concat())?;
    if !added.status.success() {
        return Err(String::from_utf8_lossy(&added.stderr).trim().to_owned());
    }
    let skill = home.join("skills").join("memorius-worklogs");
    std::fs::create_dir_all(&skill).map_err(|error| error.to_string())?;
    std::fs::write(skill.join("SKILL.md"), SKILL).map_err(|error| error.to_string())
}
