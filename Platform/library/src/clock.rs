//! Times as the C# `GameRecords` writes them: .NET's round-trip text of a local time, `DateTime.Now.ToString("o")`, for
//! a call that gives none; and the instant such a text names, for ordering. See EmuSen_Backend.md §28.4.

/// Now, as `DateTime.Now.ToString("o", CultureInfo.InvariantCulture)`: `2026-10-18T14:03:11.2510000+02:00`.
pub fn now() -> String {
    let since = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let seconds = since.as_secs() as i64;
    let offset = local_offset(seconds);
    let local = seconds + offset;
    let (days, rest) = (local.div_euclid(86_400), local.rem_euclid(86_400));
    let (y, m, d) = civil(days);
    let sign = if offset < 0 { '-' } else { '+' };
    let off = offset.abs() / 60;
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}.{:07}{sign}{:02}:{:02}", rest / 3600, rest / 60 % 60, rest % 60, since.subsec_nanos() / 100, off / 60, off % 60)
}

#[cfg(unix)]
fn local_offset(seconds: i64) -> i64 {
    let t = seconds as libc::time_t;
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    if unsafe { libc::localtime_r(&t, &mut tm) }.is_null() { 0 } else { tm.tm_gmtoff as i64 }
}

#[cfg(not(unix))]
fn local_offset(_: i64) -> i64 {
    0
}

/// The date of a day counted from 1970-01-01, by Howard Hinnant's civil-from-days.
fn civil(days: i64) -> (i64, u32, u32) {
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

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The instant a round-trip text names, in 100 ns ticks from 1970 in UTC; a text with no offset is taken as written.
/// None for a text that is not one.
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
    let offset = match b.get(at) {
        None => 0,
        Some(b'Z') => 0,
        Some(sign @ (b'+' | b'-')) => {
            let minutes = num(at + 1, 2)? * 60 + num(at + 4, 2)?;
            if *sign == b'-' { -minutes } else { minutes }
        }
        _ => return None,
    };
    let seconds = days_from_civil(y, mo, d) * 86_400 + h * 3600 + mi * 60 + s - offset * 60;
    Some(seconds * 10_000_000 + fraction)
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
        assert_eq!(super::civil(0), (1970, 1, 1));
        assert_eq!(super::days_from_civil(2026, 10, 18), 20_744);
    }
}
