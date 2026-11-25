//! Imports session history into a database, prints timing, and with a date
//! prints that day's work spans:
//! `cargo run --release -p memorius-sources --features store --example import -- /tmp/m.db [2026-10-11]`

use std::path::{Path, PathBuf};
use std::time::Instant;

use memorius_core::agent::AgentSettings;
use memorius_core::store::Store;
use memorius_core::timeline::DEFAULT_SLOT_MS;
use memorius_sources::git::GitHistories;
use memorius_sources::import::{Roots, import_all};
use memorius_sources::views::{day_bounds, day_timeline};

fn main() {
    let mut args = std::env::args().skip(1);
    let database = PathBuf::from(args.next().expect("database path"));
    let roots = Roots::new(&AgentSettings::default()).expect("home folder");
    let mut git = GitHistories::default();
    for run in ["first", "second"] {
        let started = Instant::now();
        import_all(&database, &roots, &mut git).expect("import");
        println!("{run} import: {:.1} s", started.elapsed().as_secs_f64());
    }
    if let Some(date) = args.next() {
        print_day(&database, &date, &mut git);
    }
}

fn print_day(database: &Path, date: &str, git: &mut GitHistories) {
    let (from, to) = day_bounds(date).expect("YYYY-MM-DD");
    let store = Store::open_read_only(database).expect("open");
    let day = day_timeline(&store, git, from, to, DEFAULT_SLOT_MS).expect("day");
    let clock = |ms: i64| {
        jiff::Timestamp::from_millisecond(ms)
            .expect("time")
            .to_zoned(jiff::tz::TimeZone::system())
            .strftime("%H:%M")
            .to_string()
    };
    for span in day.work {
        println!(
            "{}-{} {} ({:?}, {} %, {:?}) {:?}",
            clock(span.start_ms),
            clock(span.end_ms),
            span.issue,
            span.confidence,
            span.support,
            span.split,
            span.evidence.first()
        );
    }
}
