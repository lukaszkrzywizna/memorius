use std::collections::HashSet;
use std::path::Path;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use memorius_core::model::{RuleKind, Window, redact_private};
use memorius_core::recorder::{Change, Recorder};
use memorius_core::store::{Store, StoreError};
use memorius_core::time_rules::{TIME_RULES, TimeRules, minutes_ms};
use memorius_sources::{Probe, TitleAccess, observe};
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;
use ts_rs::TS;

use crate::shell;

const TICK: Duration = Duration::from_secs(1);

/// Sent to the UI when a new activity row starts or recording state changes.
pub const ACTIVITY_CHANGED: &str = "activity-changed";

#[derive(Clone, Debug, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum RecorderState {
    Recording {
        current: Option<Window>,
    },
    #[serde(rename_all = "camelCase")]
    Paused {
        #[ts(type = "number")]
        until_ms: i64,
    },
}

#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecorderStatus {
    pub state: RecorderState,
    pub title_access: TitleAccess,
    /// When macOS took title access away while Memorius ran; `None` while it
    /// is granted or was never granted in this run.
    #[ts(type = "number | null")]
    pub titles_lost_ms: Option<i64>,
}

enum Control {
    Pause { until_ms: i64 },
    Resume,
    RequestTitleAccess,
    PrivateApps(HashSet<String>),
    AwayAfter(i64),
    Stop,
}

/// App ids the user marked private.
pub fn private_apps(store: &Store) -> Result<HashSet<String>, StoreError> {
    Ok(store
        .rules()?
        .into_iter()
        .filter_map(|rule| match rule.rule {
            RuleKind::PrivateApp { app_id } => Some(app_id),
            RuleKind::AppIssue { .. }
            | RuleKind::HostIssue { .. }
            | RuleKind::TitleIssue { .. } => None,
        })
        .collect())
}

/// The recorder thread and its controls. The thread owns the writing
/// database connection and the platform probe.
pub struct Recording {
    status: Arc<Mutex<RecorderStatus>>,
    control: Sender<Control>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl Recording {
    pub fn start(app: AppHandle, database: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let store = Store::open(database)?;
        let private = private_apps(&store)?;
        let away_after_ms = minutes_ms(
            store
                .setting::<TimeRules>(TIME_RULES)?
                .unwrap_or_default()
                .clamped()
                .away_after_minutes,
        );
        let recorder = Recorder::new(store);
        let status = Arc::new(Mutex::new(RecorderStatus {
            state: RecorderState::Recording { current: None },
            title_access: TitleAccess::Granted,
            titles_lost_ms: None,
        }));
        let (control, inbox) = mpsc::channel();
        let shared = Arc::clone(&status);
        let thread = std::thread::Builder::new()
            .name("memorius-recorder".into())
            .spawn(move || run(&app, recorder, private, away_after_ms, &shared, &inbox))?;
        Ok(Self {
            status,
            control,
            thread: Mutex::new(Some(thread)),
        })
    }

    pub fn status(&self) -> RecorderStatus {
        self.status
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn pause(&self, minutes: u32) {
        let until_ms = now_ms() + i64::from(minutes) * 60_000;
        let _ = self.control.send(Control::Pause { until_ms });
    }

    pub fn resume(&self) {
        let _ = self.control.send(Control::Resume);
    }

    pub fn request_title_access(&self) {
        let _ = self.control.send(Control::RequestTitleAccess);
    }

    pub fn set_private_apps(&self, apps: HashSet<String>) {
        let _ = self.control.send(Control::PrivateApps(apps));
    }

    pub fn set_away_after(&self, away_after_ms: i64) {
        let _ = self.control.send(Control::AwayAfter(away_after_ms));
    }

    /// Ends the open row and waits for the thread, so quitting loses nothing.
    pub fn stop(&self) {
        let _ = self.control.send(Control::Stop);
        let thread = self
            .thread
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(thread) = thread {
            let _ = thread.join();
        }
    }
}

fn run(
    app: &AppHandle,
    mut recorder: Recorder,
    mut private: HashSet<String>,
    mut away_after_ms: i64,
    status: &Mutex<RecorderStatus>,
    inbox: &mpsc::Receiver<Control>,
) {
    let mut probe = Probe::new();
    let mut paused_until_ms: Option<i64> = None;
    let mut last_access: Option<TitleAccess> = None;
    loop {
        match inbox.recv_timeout(TICK) {
            Ok(Control::Stop) | Err(RecvTimeoutError::Disconnected) => {
                report(recorder.stop(now_ms()));
                return;
            }
            Ok(Control::Pause { until_ms }) => {
                paused_until_ms = Some(until_ms);
                report(recorder.stop(now_ms()));
            }
            Ok(Control::Resume) => paused_until_ms = None,
            Ok(Control::RequestTitleAccess) => probe.request_title_access(),
            Ok(Control::PrivateApps(apps)) => private = apps,
            Ok(Control::AwayAfter(ms)) => away_after_ms = ms,
            Err(RecvTimeoutError::Timeout) => {}
        }

        let now = now_ms();
        paused_until_ms = paused_until_ms.filter(|until| *until > now);
        let title_access = probe.title_access();
        let lost = last_access == Some(TitleAccess::Granted) && title_access == TitleAccess::Denied;
        if last_access.is_some_and(|last| last != title_access) {
            shell::set_tray_alert(app, title_access == TitleAccess::Denied);
        }
        if lost {
            titles_lost(app);
        }
        last_access = Some(title_access);
        let titles_lost_ms = match title_access {
            TitleAccess::Granted => None,
            TitleAccess::Denied => status
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .titles_lost_ms
                .or(lost.then_some(now)),
        };
        let mut next = RecorderStatus {
            state: RecorderState::Recording { current: None },
            title_access,
            titles_lost_ms,
        };
        let mut changed = false;
        if let Some(until_ms) = paused_until_ms {
            next.state = RecorderState::Paused { until_ms };
        } else {
            // A bug in one platform call must cost one reading, not the
            // whole recorder (this happened with a browser URL script).
            let Ok(mut reading) =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| probe.read()))
            else {
                eprintln!("memorius: a reading failed and was skipped");
                continue;
            };
            reading.window = reading.window.map(|window| {
                if private.contains(&window.app.id) {
                    redact_private(window)
                } else {
                    window
                }
            });
            next.state = RecorderState::Recording {
                current: reading.window.clone(),
            };
            if let Some(observation) = observe(reading, now, away_after_ms) {
                match recorder.observe(observation, now) {
                    Ok(change) => changed = change == Change::Started,
                    Err(error) => report(Err(error)),
                }
            }
        }

        let mut shared = status.lock().unwrap_or_else(PoisonError::into_inner);
        changed |= std::mem::discriminant(&shared.state) != std::mem::discriminant(&next.state)
            || shared.title_access != next.title_access;
        *shared = next;
        drop(shared);
        if changed {
            let _ = app.emit(ACTIVITY_CHANGED, ());
        }
    }
}

/// Tells the user once when macOS takes title access away while recording.
fn titles_lost(app: &AppHandle) {
    let shown = app
        .notification()
        .builder()
        .title("Memorius can't read window titles")
        .body("Accessibility was switched off. Open Memorius to fix it.")
        .show();
    if let Err(error) = shown {
        eprintln!("memorius: cannot show the lost-permission notification: {error}");
    }
}

fn report(result: Result<(), StoreError>) {
    if let Err(error) = result {
        eprintln!("memorius: recording failed to store activity: {error}");
    }
}

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
        })
}
