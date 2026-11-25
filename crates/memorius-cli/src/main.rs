//! `memorius`: activity and worklog proposals for coding agents. Every
//! command prints JSON. `memorius mcp` serves the same operations over MCP.

mod mcp;
mod ops;
mod time;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use crate::ops::{Op, run, week_start};
use crate::time::today;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Database file (default: the Memorius desktop app's database, or `MEMORIUS_DB`).
    #[arg(long, global = true)]
    db: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// One day's activity blocks with detected issues and proposals.
    Day {
        /// YYYY-MM-DD (default: today).
        date: Option<String>,
    },
    /// The week timesheet: per issue and day, logged, proposed or detected.
    Week {
        /// ISO week (2026-W41) or a date in it (default: this week).
        week: Option<String>,
    },
    /// Activity blocks with at least 5 minutes no proposal covers.
    Unlogged {
        /// YYYY-MM-DD (default: this week's Monday).
        #[arg(long)]
        from: Option<String>,
        /// YYYY-MM-DD (default: today).
        #[arg(long)]
        to: Option<String>,
    },
    /// Proposals that overlap a range of days.
    Proposals {
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
    },
    /// Proposes a worklog for the user to approve in Memorius.
    Propose {
        #[arg(long)]
        issue: String,
        /// The day it is logged on, 2026-10-09.
        #[arg(long)]
        date: String,
        /// First moment of the work, 2026-10-09T13:31 (local) or with an
        /// offset; the day's start when left out.
        #[arg(long)]
        start: Option<String>,
        /// Last moment of the work; the day's end when left out.
        #[arg(long)]
        end: Option<String>,
        #[arg(long)]
        minutes: u32,
        /// The worklog description.
        #[arg(long)]
        text: String,
        /// YouTrack work type, for example Development.
        #[arg(long = "type")]
        work_type: Option<String>,
        /// Who proposes, for example "Claude Code".
        #[arg(long)]
        agent: String,
    },
    /// The build, output and schema versions, and the newest imported session.
    Status,
    /// Serves these operations as an MCP server over stdio.
    Mcp,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let Some(database) = cli.db.or_else(memorius_sources::views::default_database) else {
        eprintln!("memorius: cannot find the Memorius database; pass --db");
        return ExitCode::FAILURE;
    };
    if !database.exists() {
        eprintln!(
            "memorius: no database at {}; start the Memorius app first",
            database.display()
        );
        return ExitCode::FAILURE;
    }
    let today = today();
    let op = match cli.command {
        Command::Mcp => return mcp::serve(&database),
        Command::Day { date } => Op::Day {
            date: date.unwrap_or(today),
        },
        Command::Week { week } => Op::Week {
            week: week.unwrap_or(today),
        },
        Command::Unlogged { from, to } => Op::Unlogged {
            from: from.unwrap_or_else(|| week_start(&today)),
            to: to.unwrap_or(today),
        },
        Command::Proposals { from, to } => Op::Proposals {
            from: from.unwrap_or_else(|| week_start(&today)),
            to: to.unwrap_or(today),
        },
        Command::Propose {
            issue,
            date,
            start,
            end,
            minutes,
            text,
            work_type,
            agent,
        } => Op::Propose {
            issue,
            date,
            start,
            end,
            minutes,
            text,
            work_type,
            agent,
        },
        Command::Status => Op::Status,
    };
    match run(&op, &database, now_ms()) {
        Ok(value) => {
            println!("{value:#}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("memorius: {error}");
            ExitCode::FAILURE
        }
    }
}

pub fn now_ms() -> i64 {
    jiff::Timestamp::now().as_millisecond()
}
