//! Times as the records hold them: the round-trip text of a local time, for a call that gives none; and the instant
//! such a text names, for ordering. See EmuSen_Backend.md §28.4.

use emusen_galaxia::clock::{self, TICKS_PER_SECOND, local_offset};

/// Now as the round-trip text of the local time: `2026-10-18T14:03:11.2510000+02:00`.
pub fn now() -> String {
    clock::now()
}

/// The instant a round-trip text names, in 100 ns ticks from 1970 in UTC; a text with no offset is a local time
/// (`wall_offset`). None for a text that is not one.
pub fn instant(text: &str) -> Option<i64> {
    let (wall, offset) = clock::wall(text)?;
    let offset = offset.unwrap_or_else(|| wall_offset(wall.div_euclid(TICKS_PER_SECOND)));
    Some(wall - offset * TICKS_PER_SECOND)
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
