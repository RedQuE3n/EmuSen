//! Times as the platform stores and shows them, on the `time` crate: 100 ns ticks from 1970 in UTC (the stored unit),
//! the round-trip text written with seven fraction digits (`2026-10-18T14:03:11.2510000+02:00`) and read back, and the
//! system zone's offset at an instant (EmuSen_RustPlatform.md §47.20).

use time::format_description::BorrowedFormatItem;
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;
use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, UtcOffset};

pub const TICKS_PER_SECOND: i64 = 10_000_000;
pub const TICKS_PER_MILLISECOND: i64 = 10_000;
pub const TICKS_PER_DAY: i64 = 86_400 * TICKS_PER_SECOND;

/// `yyyy-MM-dd`.
pub const DAY: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]");
/// `yyyyMMdd`, a day's file name.
pub const DAY_NAME: &[BorrowedFormatItem<'_>] = format_description!("[year][month][day]");
/// `yyyy-MM-dd HH:mm:ss`.
pub const STAMP: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
/// `yyyy-MM-dd_HHmmss`, a stamp in a file name.
pub const STAMP_NAME: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]_[hour][minute][second]");
/// `yyyy-MM-ddTHH:mm:ssZ`, a manifest's `mtime`.
pub const MTIME: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");
/// `HH:mm`.
pub const CLOCK: &[BorrowedFormatItem<'_>] = format_description!("[hour]:[minute]");

const ROUND_TRIP: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:7]");
const MILLIS: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z");
const UNMARKED: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second][optional [.[subsecond]]]");

/// Ticks as an instant in UTC, held to the years `time` can name.
pub fn utc(ticks: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp_nanos(ticks as i128 * 100).unwrap_or(if ticks < 0 { PrimitiveDateTime::MIN.assume_utc() } else { PrimitiveDateTime::MAX.assume_utc() })
}

/// An instant as ticks, a part of a tick cut toward zero.
pub fn ticks_at(instant: OffsetDateTime) -> i64 {
    (instant.unix_timestamp_nanos() / 100) as i64
}

/// Ticks of a system time.
pub fn ticks_of(time: std::time::SystemTime) -> i64 {
    ticks_at(OffsetDateTime::from(time))
}

/// The system zone's offset from UTC, in seconds, at a Unix time; 0 where the system does not say.
pub fn local_offset(unix_seconds: i64) -> i64 {
    OffsetDateTime::from_unix_timestamp(unix_seconds).ok().and_then(|at| UtcOffset::local_offset_at(at).ok()).map_or(0, |offset| offset.whole_seconds() as i64)
}

/// Ticks counted as if in UTC, written by a format: a local time's fields when the ticks are local.
pub fn text(ticks: i64, format: &[BorrowedFormatItem<'_>]) -> String {
    utc(ticks).format(format).unwrap_or_default()
}

/// The round-trip text, seven fraction digits, of ticks counted as if in UTC, then `suffix`.
pub fn round_trip_text(ticks: i64, suffix: &str) -> String {
    text(ticks, ROUND_TRIP) + suffix
}

/// An offset in seconds as `+hh:mm` or `-hh:mm`.
pub fn offset_text(offset: i64) -> String {
    let minutes = offset.abs() / 60;
    format!("{}{:02}:{:02}", if offset < 0 { '-' } else { '+' }, minutes / 60, minutes % 60)
}

/// A UTC instant as the round-trip text of the system's local time, with its offset.
pub fn local_of_utc(utc_ticks: i64) -> String {
    let offset = local_offset(utc_ticks.div_euclid(TICKS_PER_SECOND));
    round_trip_text(utc_ticks + offset * TICKS_PER_SECOND, &offset_text(offset))
}

/// Ticks in UTC as `yyyy-MM-ddTHH:mm:ss.fffZ`, the fraction cut to milliseconds.
pub fn utc_millis(ticks: i64) -> String {
    text(ticks, MILLIS)
}

/// Now as local ticks, the system's offset added.
pub fn local_ticks_now() -> i64 {
    let utc = ticks_of(std::time::SystemTime::now());
    utc + local_offset(utc.div_euclid(TICKS_PER_SECOND)) * TICKS_PER_SECOND
}

/// Now as the round-trip text of the local time.
pub fn now() -> String {
    local_of_utc(ticks_of(std::time::SystemTime::now()))
}

/// Now as the round-trip text of UTC, ending in `Z`.
pub fn utc_now() -> String {
    round_trip_text(ticks_of(std::time::SystemTime::now()), "Z")
}

/// A file's last write as the round-trip text of the local time; none where the file cannot be read.
pub fn last_write(path: &str) -> Option<String> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok().map(|t| local_of_utc(ticks_of(t)))
}

/// A round-trip text read: its ticks counted as if in UTC, and its offset in seconds where it has one (`Z` is 0).
fn read(text: &str) -> Option<(i64, Option<i64>)> {
    // The round-trip form puts a `T` between date and time; RFC 3339 would also take a space or a `t`.
    if text.as_bytes().get(10) != Some(&b'T') {
        return None;
    }
    if let Ok(at) = OffsetDateTime::parse(text, &Rfc3339) {
        let offset = at.offset().whole_seconds() as i64;
        return Some((ticks_at(at) + offset * TICKS_PER_SECOND, Some(offset)));
    }
    PrimitiveDateTime::parse(text, UNMARKED).ok().map(|at| (ticks_at(at.assume_utc()), None))
}

/// The instant a round-trip text names, in ticks in UTC: a text with `Z` or an offset as it says, one with neither
/// taken as UTC. None for a text not of that form.
pub fn instant(text: &str) -> Option<i64> {
    read(text).map(|(wall, offset)| wall - offset.unwrap_or(0) * TICKS_PER_SECOND)
}

/// A wall time's ticks and offset as a round-trip text gives them, for a reader that places an unmarked time itself.
pub fn wall(text: &str) -> Option<(i64, Option<i64>)> {
    read(text)
}

/// A round-trip text read and written again: an unmarked time kept, `Z` kept, an offset's time turned into the
/// system's local time.
pub fn round_trip(value: &str) -> Option<String> {
    let (wall, offset) = read(value)?;
    Some(match offset {
        None => round_trip_text(wall, ""),
        Some(_) if value.ends_with('Z') => round_trip_text(wall, "Z"),
        Some(offset) => local_of_utc(wall - offset * TICKS_PER_SECOND),
    })
}

/// The date of a day counted from 1970-01-01.
pub fn civil(days: i64) -> (i64, u32, u32) {
    let date = utc(days * TICKS_PER_DAY).date();
    (date.year() as i64, date.month() as u32, date.day() as u32)
}

/// The day, counted from 1970-01-01, of a date; the day is held to its month's last.
pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let month = Month::try_from(m.clamp(1, 12) as u8).unwrap_or(Month::January);
    let first = Date::from_calendar_date(y as i32, month, 1).unwrap_or(Date::MIN);
    (first.to_julian_day() as i64 - Date::from_calendar_date(1970, Month::January, 1).map_or(0, |e| e.to_julian_day() as i64)) + d - 1
}

/// The day of the week, Sunday 0, of a day counted from 1970-01-01.
pub fn weekday(days: i64) -> u32 {
    utc(days * TICKS_PER_DAY).weekday().number_days_from_sunday() as u32
}

/// `TimeSpan.FromSeconds` in ticks: the product cut toward zero.
pub fn ticks_from_seconds(seconds: f64) -> i64 {
    (seconds * TICKS_PER_SECOND as f64) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_round_trip_text_keeps_its_kind_and_its_fraction_s_seven_digits() {
        assert_eq!(round_trip("2026-09-20T10:00:00").as_deref(), Some("2026-09-20T10:00:00.0000000"));
        assert_eq!(round_trip("2026-09-20T10:00:00.25Z").as_deref(), Some("2026-09-20T10:00:00.2500000Z"));
        let local = round_trip("2026-09-20T10:00:00.1234567+00:00").unwrap();
        assert_eq!(round_trip(&local).as_deref(), Some(local.as_str()), "a local time read back is the same time");
        assert!(round_trip("yesterday").is_none());
        assert_eq!((instant("2026-10-18 14:03:11Z"), instant("2026-10-18t14:03:11Z")), (None, None));
    }

    #[test]
    fn a_stamp_is_written_cut_to_milliseconds_and_read_back() {
        let t = instant("2026-07-14T10:00:00.1239999Z").unwrap();
        assert_eq!(utc_millis(t), "2026-07-14T10:00:00.123Z");
        assert_eq!(instant(&utc_millis(t)), Some(t - 9999));
        assert_eq!(instant("1970-01-01T01:00:00+01:00"), Some(0));
        assert_eq!(instant("2026-10-18T14:03:11.251+02:00"), Some(17_923_249_912_510_000));
        assert_eq!(civil(0), (1970, 1, 1));
        assert_eq!(days_from_civil(2026, 10, 18), 20_744);
        assert_eq!(weekday(days_from_civil(2026, 10, 18)), 0);
        assert_eq!(text(instant("2026-10-18T14:03:09Z").unwrap(), STAMP_NAME), "2026-10-18_140309");
        assert_eq!(ticks_from_seconds(60.0 / (60.0 * 0.9)), 11_111_111);
    }

    #[test]
    fn known_instants_are_written_as_the_same_text_and_ticks() {
        assert_eq!(round_trip_text(0, "Z"), "1970-01-01T00:00:00.0000000Z");
        assert_eq!(round_trip_text(-1, "Z"), "1969-12-31T23:59:59.9999999Z");
        assert_eq!(round_trip_text(951_782_400 * TICKS_PER_SECOND + 12_345_678, ""), "2000-02-29T00:00:01.2345678");
        assert_eq!(round_trip_text(1_791_936_000 * TICKS_PER_SECOND + 3661 * TICKS_PER_SECOND + 5, "+02:00"), "2026-10-14T01:01:01.0000005+02:00");
        let last_day = (days_from_civil(9999, 12, 31) + 1) * TICKS_PER_DAY - 1;
        assert_eq!(round_trip_text(last_day, "Z"), "9999-12-31T23:59:59.9999999Z");
        assert_eq!(utc_millis(last_day), "9999-12-31T23:59:59.999Z");
        assert_eq!(civil(days_from_civil(2026, 2, 28) + 1), (2026, 3, 1));
        assert_eq!(civil(days_from_civil(2024, 2, 28) + 1), (2024, 2, 29));
        assert_eq!(civil(days_from_civil(2025, 12, 31) + 1), (2026, 1, 1));
        assert_eq!(offset_text(7200), "+02:00");
        assert_eq!(offset_text(-12_600), "-03:30");
        assert_eq!(offset_text(0), "+00:00");
        let t = std::time::UNIX_EPOCH + std::time::Duration::new(1_000, 123_456_789);
        assert_eq!(ticks_of(t), 1_000 * TICKS_PER_SECOND + 1_234_567);
        assert_eq!(ticks_of(std::time::UNIX_EPOCH - std::time::Duration::from_millis(1)), -TICKS_PER_MILLISECOND);
    }

    #[test]
    fn the_local_text_is_the_utc_instant_plus_the_systems_offset() {
        let utc = 1_791_936_000 * TICKS_PER_SECOND;
        let offset = local_offset(1_791_936_000);
        assert_eq!(local_of_utc(utc), round_trip_text(utc + offset * TICKS_PER_SECOND, &offset_text(offset)));
        assert_eq!(instant(&local_of_utc(utc)), Some(utc));
    }
}
