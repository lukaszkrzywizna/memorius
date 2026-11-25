//! Platform readers. The desktop host calls [`Probe::read`] about once per
//! second on its recorder thread; every call is an in-process API call, never
//! a spawned process.

use memorius_core::model::{Observation, Window};
use serde::Serialize;
use ts_rs::TS;

pub mod agent;
#[cfg(feature = "store")]
pub mod commits;
pub mod git;
#[cfg(feature = "store")]
pub mod import;
pub mod sessions;
#[cfg(feature = "store")]
pub mod views;
pub mod youtrack;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{Probe, app_icon_png};

#[cfg(windows)]
mod win;
#[cfg(windows)]
pub use win::{Probe, app_icon_png};

/// One raw reading of the foreground state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    pub idle_ms: i64,
    pub locked: bool,
    /// `None` when no app is in the foreground.
    pub window: Option<Window>,
}

/// Whether the OS lets Memorius read window titles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum TitleAccess {
    Granted,
    /// macOS: Accessibility permission is missing.
    Denied,
}

/// Turns a reading into what the recorder stores. `None` means nothing to
/// record (no foreground app), which the recorder treats as a gap.
pub fn observe(reading: Reading, now_ms: i64, away_after_ms: i64) -> Option<Observation> {
    if reading.locked {
        Some(Observation::Locked)
    } else if reading.idle_ms >= away_after_ms {
        Some(Observation::Away {
            since_ms: now_ms - reading.idle_ms,
        })
    } else {
        reading.window.map(Observation::Active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDLE_THRESHOLD_MS: i64 = 5 * 60 * 1_000;
    use memorius_core::model::{AppRef, Title};

    fn window() -> Window {
        Window {
            app: AppRef {
                id: "com.jetbrains.rider".into(),
                name: "Rider".into(),
            },
            title: Title::Known {
                text: "a.fs".into(),
            },
            document: None,
            url: None,
        }
    }

    #[test]
    fn lock_wins_over_idle_and_window() {
        let reading = Reading {
            idle_ms: IDLE_THRESHOLD_MS,
            locked: true,
            window: Some(window()),
        };

        assert_eq!(
            observe(reading, 1_000_000, IDLE_THRESHOLD_MS),
            Some(Observation::Locked)
        );
    }

    #[test]
    fn idle_past_the_threshold_is_away_since_the_last_input() {
        let reading = Reading {
            idle_ms: IDLE_THRESHOLD_MS,
            locked: false,
            window: Some(window()),
        };

        assert_eq!(
            observe(reading, 1_000_000, IDLE_THRESHOLD_MS),
            Some(Observation::Away {
                since_ms: 1_000_000 - IDLE_THRESHOLD_MS
            })
        );
    }

    #[test]
    fn active_window_otherwise_and_nothing_without_one() {
        let active = Reading {
            idle_ms: 0,
            locked: false,
            window: Some(window()),
        };
        let empty = Reading {
            window: None,
            ..active.clone()
        };

        assert_eq!(
            observe(active, 0, IDLE_THRESHOLD_MS),
            Some(Observation::Active(window()))
        );
        assert_eq!(observe(empty, 0, IDLE_THRESHOLD_MS), None);
    }
}
