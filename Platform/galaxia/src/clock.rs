//! Local times as .NET's round-trip text gives them, `DateTime.ToString("o")`: now, a file's last write, and a text
//! read back as `DateTime.Parse(…, RoundtripKind)` reads it and written again. The local offset is the system's at that
//! instant (EmuSen_Backend.md §29.18).

pub use emusen_dotnet::date::{local_of_utc, local_offset, ticks_of};
use emusen_dotnet::date::{TICKS_PER_SECOND, days_from_civil, round_trip_text as text};

/// Now as local ticks from 1970, the system's offset added.
pub fn local_ticks_now() -> i64 {
    let utc = ticks_of(std::time::SystemTime::now());
    utc + local_offset(utc.div_euclid(TICKS_PER_SECOND)) * TICKS_PER_SECOND
}

/// `DateTime.Now.ToString("o")`.
pub fn now() -> String {
    local_of_utc(ticks_of(std::time::SystemTime::now()))
}

/// `DateTime.UtcNow.ToString("o")`.
pub fn utc_now() -> String {
    text(ticks_of(std::time::SystemTime::now()), "Z")
}

/// `File.GetLastWriteTime(path).ToString("o")`, none where the file cannot be read.
pub fn last_write(path: &str) -> Option<String> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok().map(|t| local_of_utc(ticks_of(t)))
}

/// A round-trip text read as `DateTime.Parse(…, RoundtripKind)` reads it and written with "o": an unmarked time kept,
/// `Z` kept as UTC, an offset's time turned into the system's local time.
pub fn round_trip(value: &str) -> Option<String> {
    let b = value.as_bytes();
    let num = |from: usize, len: usize| -> Option<i64> {
        let s = value.get(from..from + len)?;
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
        let digits = &value[start..at];
        if digits.is_empty() {
            return None;
        }
        fraction = format!("{:0<7}", &digits[..digits.len().min(7)]).parse().ok()?;
    }
    let ticks = (days_from_civil(y, mo, d) * 86_400 + h * 3600 + mi * 60 + s) * TICKS_PER_SECOND + fraction;
    match b.get(at) {
        None => Some(text(ticks, "")),
        Some(b'Z') if at + 1 == b.len() => Some(text(ticks, "Z")),
        Some(sign @ (b'+' | b'-')) if at + 6 == b.len() && b[at + 3] == b':' => {
            let minutes = num(at + 1, 2)? * 60 + num(at + 4, 2)?;
            let offset = if *sign == b'-' { -minutes } else { minutes } * 60;
            Some(local_of_utc(ticks - offset * TICKS_PER_SECOND))
        }
        _ => None,
    }
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
    }
}
