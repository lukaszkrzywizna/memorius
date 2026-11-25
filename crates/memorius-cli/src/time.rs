//! Agents read and write local times as ISO 8601 strings.

use jiff::Timestamp;
use jiff::civil::DateTime;
use jiff::tz::TimeZone;

/// `2026-10-09T13:31:00+02:00` in the computer's time zone.
pub fn local_iso(ms: i64) -> String {
    Timestamp::from_millisecond(ms).map_or_else(
        |_| ms.to_string(),
        |time| {
            time.to_zoned(TimeZone::system())
                .strftime("%Y-%m-%dT%H:%M:%S%:z")
                .to_string()
        },
    )
}

/// Accepts a time with an offset (`2026-10-09T13:31:00+02:00`, `...Z`) or a
/// local one (`2026-10-09T13:31`).
pub fn parse_time(text: &str) -> Result<i64, String> {
    if let Ok(time) = text.parse::<Timestamp>() {
        return Ok(time.as_millisecond());
    }
    text.parse::<DateTime>()
        .and_then(|local| local.to_zoned(TimeZone::system()))
        .map(|zoned| zoned.timestamp().as_millisecond())
        .map_err(|_| {
            format!("{text:?} is not a time such as 2026-10-09T13:31 or 2026-10-09T13:31:00+02:00")
        })
}

pub fn today() -> String {
    jiff::Zoned::now().date().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_and_offset_times_round_trip() {
        let ms = parse_time("2026-10-09T13:31").unwrap();

        assert_eq!(parse_time(&local_iso(ms)).unwrap(), ms);
        assert_eq!(
            parse_time("2026-10-09T11:31:00Z").unwrap(),
            parse_time("2026-10-09T13:31:00+02:00").unwrap()
        );
        assert!(parse_time("yesterday").is_err());
    }
}
