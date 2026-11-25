use crate::model::Observation;
use crate::store::{RowKind, Store, StoreError};

/// The open row's end is written at most this often while nothing changes,
/// so a crash loses at most this much time.
pub const FLUSH_INTERVAL_MS: i64 = 5_000;

/// A longer pause between observations means the machine slept or Memorius
/// was not running; the open row then ends at the last observation.
pub const MAX_GAP_MS: i64 = 15_000;

/// What an observation did to the stored activity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    Extended,
    Started,
}

struct OpenRow {
    id: i64,
    kind: RowKind,
    start_ms: i64,
    last_seen_ms: i64,
    stored_end_ms: i64,
}

/// Turns a stream of observations into activity rows (heartbeat model: an
/// unchanged observation extends the open row, a change starts a new one).
pub struct Recorder {
    store: Store,
    open: Option<OpenRow>,
}

impl Recorder {
    pub fn new(store: Store) -> Self {
        Self { store, open: None }
    }

    pub fn observe(&mut self, observation: Observation, now_ms: i64) -> Result<Change, StoreError> {
        if let Some(open) = self
            .open
            .take_if(|open| now_ms - open.last_seen_ms > MAX_GAP_MS)
        {
            self.store.set_end(open.id, open.last_seen_ms)?;
        }

        let (kind, wanted_start_ms) = match observation {
            Observation::Active(window) => (RowKind::App(window), now_ms),
            Observation::Away { since_ms } => (RowKind::Away, since_ms),
            Observation::Locked => (RowKind::Locked, now_ms),
        };

        if let Some(open) = self.open.as_mut().filter(|open| open.kind == kind) {
            open.last_seen_ms = now_ms;
            if now_ms - open.stored_end_ms >= FLUSH_INTERVAL_MS {
                self.store.set_end(open.id, now_ms)?;
                open.stored_end_ms = now_ms;
            }
            return Ok(Change::Extended);
        }

        let start_ms = match self.open.take() {
            Some(open) => {
                let boundary = wanted_start_ms.clamp(open.start_ms, now_ms);
                self.store.set_end(open.id, boundary)?;
                boundary
            }
            None => now_ms,
        };
        let id = self.store.insert(start_ms, now_ms, &kind)?;
        self.open = Some(OpenRow {
            id,
            kind,
            start_ms,
            last_seen_ms: now_ms,
            stored_end_ms: now_ms,
        });
        Ok(Change::Started)
    }

    /// Ends the open row (pause or quit).
    pub fn stop(&mut self, now_ms: i64) -> Result<(), StoreError> {
        if let Some(open) = self.open.take() {
            let end_ms = if now_ms - open.last_seen_ms > MAX_GAP_MS {
                open.last_seen_ms
            } else {
                now_ms
            };
            self.store.set_end(open.id, end_ms)?;
        }
        Ok(())
    }

    pub fn store(&self) -> &Store {
        &self.store
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AppRef, Title, Window};
    use crate::store::ActivityRow;

    fn active(app: &str, title: &str) -> Observation {
        Observation::Active(Window {
            app: AppRef {
                id: app.to_owned(),
                name: app.to_owned(),
            },
            title: Title::Known {
                text: title.to_owned(),
            },
            document: None,
            url: None,
        })
    }

    fn recorder() -> (tempfile::TempDir, Recorder) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("m.db")).unwrap();
        (dir, Recorder::new(store))
    }

    fn spans(recorder: &Recorder) -> Vec<(i64, i64, String)> {
        recorder
            .store()
            .rows_between(i64::MIN, i64::MAX)
            .unwrap()
            .into_iter()
            .map(
                |ActivityRow {
                     start_ms,
                     end_ms,
                     kind,
                     ..
                 }| {
                    let label = match kind {
                        RowKind::App(w) => w.app.name,
                        RowKind::Away => "away".to_owned(),
                        RowKind::Locked => "locked".to_owned(),
                    };
                    (start_ms, end_ms, label)
                },
            )
            .collect()
    }

    #[test]
    fn unchanged_observations_extend_one_row_and_flush_every_interval() {
        let (_dir, mut r) = recorder();
        assert_eq!(r.observe(active("Rider", "a"), 0).unwrap(), Change::Started);
        for t in (1_000..=7_000).step_by(1_000) {
            assert_eq!(
                r.observe(active("Rider", "a"), t).unwrap(),
                Change::Extended
            );
        }

        assert_eq!(spans(&r), vec![(0, 5_000, "Rider".to_owned())]);
    }

    #[test]
    fn a_change_closes_the_open_row_at_the_change() {
        let (_dir, mut r) = recorder();
        r.observe(active("Rider", "a"), 0).unwrap();
        r.observe(active("Rider", "a"), 1_000).unwrap();
        r.observe(active("Rider", "b"), 2_000).unwrap();
        r.observe(active("Slack", "x"), 3_000).unwrap();

        assert_eq!(
            spans(&r),
            vec![
                (0, 2_000, "Rider".to_owned()),
                (2_000, 3_000, "Rider".to_owned()),
                (3_000, 3_000, "Slack".to_owned()),
            ]
        );
    }

    #[test]
    fn a_long_gap_ends_the_row_at_the_last_observation() {
        let (_dir, mut r) = recorder();
        r.observe(active("Rider", "a"), 0).unwrap();
        r.observe(active("Rider", "a"), 2_000).unwrap();
        r.observe(active("Rider", "a"), 2_000 + MAX_GAP_MS + 1)
            .unwrap();

        assert_eq!(
            spans(&r),
            vec![
                (0, 2_000, "Rider".to_owned()),
                (
                    2_000 + MAX_GAP_MS + 1,
                    2_000 + MAX_GAP_MS + 1,
                    "Rider".to_owned()
                ),
            ]
        );
    }

    #[test]
    fn away_starts_at_the_last_input_but_not_before_the_open_row() {
        let (_dir, mut r) = recorder();
        r.observe(active("Rider", "a"), 0).unwrap();
        r.observe(active("Slack", "x"), 10_000).unwrap();
        r.observe(active("Slack", "x"), 14_000).unwrap();
        r.observe(Observation::Away { since_ms: 12_000 }, 15_000)
            .unwrap();
        r.observe(Observation::Away { since_ms: 1_000 }, 16_000)
            .unwrap();
        r.observe(active("Rider", "a"), 17_000).unwrap();

        assert_eq!(
            spans(&r),
            vec![
                (0, 10_000, "Rider".to_owned()),
                (10_000, 12_000, "Slack".to_owned()),
                (12_000, 17_000, "away".to_owned()),
                (17_000, 17_000, "Rider".to_owned()),
            ]
        );
    }

    #[test]
    fn stop_ends_the_open_row_now() {
        let (_dir, mut r) = recorder();
        r.observe(Observation::Locked, 0).unwrap();
        r.observe(Observation::Locked, 1_000).unwrap();
        r.stop(2_000).unwrap();

        assert_eq!(spans(&r), vec![(0, 2_000, "locked".to_owned())]);
    }
}
