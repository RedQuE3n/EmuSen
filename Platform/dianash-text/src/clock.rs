//! A file's modified time in the machine's own time zone, as `File.GetLastWriteTime` gives it to `ls -l`.

use std::time::{SystemTime, UNIX_EPOCH};

/// `yyyy-MM-dd HH:mm:ss`, local.
pub fn local(time: SystemTime) -> String {
    let seconds = match time.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs() as i64,
        Err(e) => -(e.duration().as_secs() as i64),
    };
    let (y, mo, d, h, mi, s) = civil(seconds + offset(seconds));
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{s:02}")
}

/// The zone's offset from UTC at that moment, in seconds.
#[cfg(unix)]
fn offset(seconds: i64) -> i64 {
    let time = seconds as libc::time_t;
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    if unsafe { libc::localtime_r(&time, &mut tm) }.is_null() {
        return 0;
    }
    tm.tm_gmtoff
}

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

/// Days since 1970 to a civil date, by Howard Hinnant's algorithm, and the time of day.
fn civil(seconds: i64) -> (i64, u32, u32, u32, u32, u32) {
    let days = seconds.div_euclid(86_400);
    let of_day = seconds.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d, (of_day / 3600) as u32, (of_day % 3600 / 60) as u32, (of_day % 60) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_fall_where_the_calendar_puts_them() {
        assert_eq!(civil(0), (1970, 1, 1, 0, 0, 0));
        assert_eq!(civil(951_782_400), (2000, 2, 29, 0, 0, 0));
        assert_eq!(civil(1_791_936_000 + 3661), (2026, 10, 14, 1, 1, 1));
    }
}
