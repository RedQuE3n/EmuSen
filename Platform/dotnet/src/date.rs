//! Dates and instants as .NET counts them: 100 ns ticks, here from 1970-01-01 UTC, the proleptic Gregorian calendar,
//! and the texts the scraper's C# writes and reads them in. See EmuSen_Backend.md §30.6.

pub const TICKS_PER_SECOND: i64 = 10_000_000;
pub const TICKS_PER_MILLISECOND: i64 = 10_000;
pub const TICKS_PER_DAY: i64 = 86_400 * TICKS_PER_SECOND;

/// The date of a day counted from 1970-01-01, by Howard Hinnant's civil-from-days.
pub fn civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + if m <= 2 { 1 } else { 0 }, m, d)
}

/// The day, counted from 1970-01-01, of a date.
pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The day of the week, Sunday 0, of a day counted from 1970-01-01 (a Thursday).
pub fn weekday(days: i64) -> u32 {
    (days + 4).rem_euclid(7) as u32
}

/// Ticks as `yyyy-MM-ddTHH:mm:ss.fffZ`, the fraction cut to milliseconds as .NET's `fff` cuts it.
pub fn utc_millis(ticks: i64) -> String {
    let days = ticks.div_euclid(TICKS_PER_DAY);
    let rest = ticks.rem_euclid(TICKS_PER_DAY);
    let (y, m, d) = civil(days);
    let seconds = rest / TICKS_PER_SECOND;
    let millis = rest % TICKS_PER_SECOND / TICKS_PER_MILLISECOND;
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}.{millis:03}Z", seconds / 3600, seconds / 60 % 60, seconds % 60)
}

/// Ticks of local time as `format`'s fields name them: `yyyy`, `MM`, `dd`, `HH`, `mm`, `ss`, and anything else as written.
pub fn local_text(local_ticks: i64, format: &str) -> String {
    let days = local_ticks.div_euclid(TICKS_PER_DAY);
    let rest = local_ticks.rem_euclid(TICKS_PER_DAY) / TICKS_PER_SECOND;
    let (y, m, d) = civil(days);
    format
        .replace("yyyy", &format!("{y:04}"))
        .replace("MM", &format!("{m:02}"))
        .replace("dd", &format!("{d:02}"))
        .replace("HH", &format!("{:02}", rest / 3600))
        .replace("mm", &format!("{:02}", rest / 60 % 60))
        .replace("ss", &format!("{:02}", rest % 60))
}

/// The instant an ISO 8601 text names, as `DateTimeOffset.Parse` and `DateTime.Parse(…, RoundtripKind)` read the texts
/// this code writes: a date, a time with an optional fraction, then `Z`, an offset or nothing (taken as UTC). None for
/// a text not of that form.
pub fn instant(text: &str) -> Option<i64> {
    let b = text.as_bytes();
    let num = |from: usize, len: usize| -> Option<i64> {
        let s = text.get(from..from + len)?;
        s.bytes().all(|c| c.is_ascii_digit()).then(|| s.parse().ok()).flatten()
    };
    if b.len() < 19 || b[4] != b'-' || b[7] != b'-' || b[10] != b'T' || b[13] != b':' || b[16] != b':' {
        return None;
    }
    let (y, mo, d, h, mi, s) = (num(0, 4)?, num(5, 2)?, num(8, 2)?, num(11, 2)?, num(14, 2)?, num(17, 2)?);
    let mut at = 19;
    let mut fraction = 0i64;
    if b.get(at) == Some(&b'.') {
        at += 1;
        let start = at;
        while b.get(at).is_some_and(u8::is_ascii_digit) {
            at += 1;
        }
        let digits = &text[start..at];
        if digits.is_empty() {
            return None;
        }
        fraction = format!("{:0<7}", &digits[..digits.len().min(7)]).parse().ok()?;
    }
    let offset = match b.get(at) {
        None | Some(b'Z') => 0,
        Some(sign @ (b'+' | b'-')) => {
            let minutes = num(at + 1, 2)? * 60 + num(at + 4, 2)?;
            if *sign == b'-' { -minutes } else { minutes }
        }
        _ => return None,
    };
    let seconds = days_from_civil(y, mo, d) * 86_400 + h * 3600 + mi * 60 + s - offset * 60;
    Some(seconds * TICKS_PER_SECOND + fraction)
}

/// `TimeSpan.FromSeconds(double)` in ticks: the product cut toward zero, as .NET Core 3.0 and later cut it.
pub fn ticks_from_seconds(seconds: f64) -> i64 {
    (seconds * TICKS_PER_SECOND as f64) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stamp_is_written_cut_to_milliseconds_and_read_back() {
        let t = instant("2026-07-14T10:00:00.1239999Z").unwrap();
        assert_eq!(utc_millis(t), "2026-07-14T10:00:00.123Z");
        assert_eq!(instant(&utc_millis(t)), Some(t - 9999));
        assert_eq!(instant("1970-01-01T01:00:00+01:00"), Some(0));
        assert_eq!(civil(0), (1970, 1, 1));
        assert_eq!(days_from_civil(2026, 10, 18), 20_744);
        assert_eq!(weekday(days_from_civil(2026, 10, 18)), 0);
        assert_eq!(local_text(instant("2026-10-18T14:03:09Z").unwrap(), "yyyy-MM-dd_HHmmss"), "2026-10-18_140309");
        assert_eq!(ticks_from_seconds(60.0 / (60.0 * 0.9)), 11_111_111);
    }
}
