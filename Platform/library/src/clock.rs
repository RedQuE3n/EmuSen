//! Times as the C# `GameRecords` writes them: .NET's round-trip text of a local time, `DateTime.Now.ToString("o")`, for
//! a call that gives none; and the instant such a text names, for ordering. See EmuSen_Backend.md §28.4.

use emusen_dotnet::date::{self, days_from_civil, local_offset};

/// Now, as `DateTime.Now.ToString("o", CultureInfo.InvariantCulture)`: `2026-10-18T14:03:11.2510000+02:00`.
pub fn now() -> String {
    date::local_of_utc(date::ticks_of(std::time::SystemTime::now()))
}

/// The instant a round-trip text names, in 100 ns ticks from 1970 in UTC; a text with no offset is a local time, as
/// .NET's `ToUniversalTime` reads one (`wall_offset`). None for a text that is not one.
pub fn instant(text: &str) -> Option<i64> {
    let b = text.as_bytes();
    let num = |from: usize, len: usize| -> Option<i64> { text.get(from..from + len)?.parse::<i64>().ok() };
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
        let padded = format!("{:0<7}", &digits[..digits.len().min(7)]);
        fraction = padded.parse().ok()?;
    }
    let wall = days_from_civil(y, mo, d) * 86_400 + h * 3600 + mi * 60 + s;
    let offset = match b.get(at) {
        None => wall_offset(wall),
        Some(b'Z') => 0,
        Some(sign @ (b'+' | b'-')) => {
            let minutes = num(at + 1, 2)? * 60 + num(at + 4, 2)?;
            60 * if *sign == b'-' { -minutes } else { minutes }
        }
        _ => return None,
    };
    Some((wall - offset) * 10_000_000 + fraction)
}

/// The local zone's offset, in seconds, for a wall time counted as if in UTC: the one in force there, or the standard
/// (smaller) one where the clock skips or repeats that time, as .NET's `TimeZoneInfo.ConvertTimeToUtc` takes it.
fn wall_offset(wall: i64) -> i64 {
    let (before, after) = (local_offset(wall - 86_400), local_offset(wall + 86_400));
    let fits = |offset: i64| local_offset(wall - offset) == offset;
    match (fits(before), fits(after)) {
        (true, false) => before,
        (false, true) => after,
        _ if before == after => before,
        _ => before.min(after),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_round_trip_text_is_written_and_read_back() {
        let now = super::now();
        assert_eq!(now.len(), 33, "{now}");
        assert!(super::instant(&now).is_some());
        assert_eq!(super::instant("1970-01-01T01:00:00.0000000+01:00"), Some(0));
        assert_eq!(super::instant("2026-10-18T12:00:00.0000001Z"), Some(super::instant("2026-10-18T14:00:00.0000000+02:00").unwrap() + 1));
    }
}
