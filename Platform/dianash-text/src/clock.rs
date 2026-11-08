//! A file's modified time in the machine's own time zone, as `ls -l` shows it.

use std::time::SystemTime;
use time::macros::format_description;
use time::{OffsetDateTime, UtcOffset};

/// `yyyy-MM-dd HH:mm:ss`, local.
pub fn local(time: SystemTime) -> String {
    let utc = OffsetDateTime::from(time);
    let at = UtcOffset::local_offset_at(utc).map_or(utc, |offset| utc.to_offset(offset));
    at.format(format_description!("[year]-[month]-[day] [hour]:[minute]:[second]")).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_time_is_written_in_the_local_zone_to_the_second() {
        let at = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_791_936_000 + 3661);
        let offset = UtcOffset::local_offset_at(OffsetDateTime::from(at)).unwrap_or(UtcOffset::UTC);
        let expected = OffsetDateTime::from(at).to_offset(offset);
        assert_eq!(local(at), format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", expected.year(), expected.month() as u8, expected.day(), expected.hour(), expected.minute(), expected.second()));
        assert_eq!(local(std::time::UNIX_EPOCH + std::time::Duration::from_secs(951_782_400)).len(), 19);
    }
}
