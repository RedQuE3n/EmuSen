//! A file's modified time in the machine's own time zone, as `File.GetLastWriteTime` gives it to `ls -l`.

use emusen_dotnet::date::civil_time;
use std::time::{SystemTime, UNIX_EPOCH};

/// `yyyy-MM-dd HH:mm:ss`, local.
pub fn local(time: SystemTime) -> String {
    let seconds = match time.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs() as i64,
        Err(e) => -(e.duration().as_secs() as i64),
    };
    let (y, mo, d, h, mi, s) = civil_time(seconds + offset(seconds));
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{s:02}")
}

#[cfg(unix)]
use emusen_dotnet::date::local_offset as offset;

#[cfg(windows)]
fn offset(_: i64) -> i64 {
    #[repr(C)]
    struct SystemTimeW([u16; 8]);
    #[repr(C)]
    struct TimeZoneInformation {
        bias: i32,
        standard_name: [u16; 32],
        standard_date: SystemTimeW,
        standard_bias: i32,
        daylight_name: [u16; 32],
        daylight_date: SystemTimeW,
        daylight_bias: i32,
    }
    unsafe extern "system" {
        fn GetTimeZoneInformation(info: *mut TimeZoneInformation) -> u32;
    }
    let mut info: TimeZoneInformation = unsafe { std::mem::zeroed() };
    let kind = unsafe { GetTimeZoneInformation(&mut info) };
    let bias = info.bias + if kind == 2 { info.daylight_bias } else { info.standard_bias };
    -(bias as i64) * 60
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_fall_where_the_calendar_puts_them() {
        assert_eq!(civil_time(0), (1970, 1, 1, 0, 0, 0));
        assert_eq!(civil_time(951_782_400), (2000, 2, 29, 0, 0, 0));
        assert_eq!(civil_time(1_791_936_000 + 3661), (2026, 10, 14, 1, 1, 1));
    }
}
