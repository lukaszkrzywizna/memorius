//! The user's rules for counting time, from Settings → Advanced.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The setting that holds them.
pub const TIME_RULES: &str = "timeRules";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export)]
pub struct TimeRules {
    /// No keyboard or mouse input this long means the user is away.
    pub away_after_minutes: u32,
    /// Pauses up to this long between moments at the computer are reading or
    /// thinking, and count as work.
    pub break_minutes: u32,
    /// An agent's time counts as the user's work this long after a prompt.
    pub agent_after_prompt_minutes: u32,
    /// The first day of Week and of "this week" in reminders.
    pub week_starts_on: WeekStart,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum WeekStart {
    #[default]
    Monday,
    Sunday,
}

impl Default for TimeRules {
    fn default() -> Self {
        Self {
            away_after_minutes: 5,
            break_minutes: 15,
            agent_after_prompt_minutes: 30,
            week_starts_on: WeekStart::Monday,
        }
    }
}

impl TimeRules {
    /// The rules within sensible bounds, so a typo cannot stop counting.
    #[must_use]
    pub fn clamped(self) -> Self {
        Self {
            away_after_minutes: self.away_after_minutes.clamp(1, 60),
            break_minutes: self.break_minutes.min(120),
            agent_after_prompt_minutes: self.agent_after_prompt_minutes.min(240),
            ..self
        }
    }
}

pub const fn minutes_ms(minutes: u32) -> i64 {
    minutes as i64 * 60_000
}
