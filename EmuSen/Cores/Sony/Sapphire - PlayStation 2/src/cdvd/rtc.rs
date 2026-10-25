//! The drive's real-time clock (ps2tek "CDVD S Commands", ReadRTC and WriteRTC): seconds, minutes, hours, day, month
//! and a two-digit year in BCD, running on the drive's time. The host sets it; it then counts IOP cycles.

use super::timing::IOP_HZ;

/// A moment as the clock holds it: seconds since 2000-01-01 00:00:00, which the two-digit year counts from [C].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rtc {
    base: u64,
    /// The drive's time at which `base` held.
    at: u64,
}

const fn bcd(v: u64) -> u8 {
    (((v / 10) << 4) | (v % 10)) as u8
}

fn from_bcd(b: u8) -> u64 {
    (b >> 4) as u64 * 10 + (b & 15) as u64
}

/// Days from 2000-01-01 to a date of the Gregorian calendar, 2000 to 2099 [A: every fourth year a leap year there].
fn days(year: u64, month: u64, day: u64) -> u64 {
    const BEFORE: [u64; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let m = month.clamp(1, 12) as usize - 1;
    year * 365 + year.div_ceil(4) + BEFORE[m] + (m >= 2 && year.is_multiple_of(4)) as u64 + day.max(1) - 1
}

impl Rtc {
    /// The clock showing `seconds` since 2000-01-01 at drive time `now`.
    pub fn new(seconds: u64, now: u64) -> Rtc {
        Rtc { base: seconds, at: now }
    }

    pub fn seconds(&self, now: u64) -> u64 {
        self.base + now.saturating_sub(self.at) / IOP_HZ
    }

    /// ReadRTC's eight bytes: 0, second, minute, hour, 0, day, month, year.
    pub fn read(&self, now: u64) -> [u8; 8] {
        let s = self.seconds(now);
        let (mut d, t) = (s / 86_400, s % 86_400);
        let mut year = 0;
        while d >= days(year + 1, 1, 1) {
            year += 1;
        }
        d -= days(year, 1, 1);
        let mut month = 12;
        while days(year, month, 1) - days(year, 1, 1) > d {
            month -= 1;
        }
        let day = d - (days(year, month, 1) - days(year, 1, 1)) + 1;
        [0, bcd(t % 60), bcd(t / 60 % 60), bcd(t / 3600), 0, bcd(day), bcd(month), bcd(year % 100)]
    }

    /// WriteRTC's parameters in the same order, the first and fifth ignored.
    pub fn write(&mut self, p: &[u8; 8], now: u64) {
        let (s, m, h) = (from_bcd(p[1]), from_bcd(p[2]), from_bcd(p[3]));
        let (day, month, year) = (from_bcd(p[5]), from_bcd(p[6]), from_bcd(p[7]));
        *self = Rtc::new(days(year, month, day) * 86_400 + h * 3600 + m * 60 + s, now);
    }
}

impl Rtc {
    /// The clock in `SPH2`: the moment it held and the drive time it held at (Sapphire_Native.md §38.2).
    pub fn saved(&self) -> [u64; 2] {
        [self.base, self.at]
    }

    pub fn restored(v: [u64; 2]) -> Rtc {
        Rtc { base: v[0], at: v[1] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clock_reads_back_what_was_written_and_runs() {
        let mut r = Rtc::default();
        assert_eq!(r.read(0), [0, 0, 0, 0, 0, 1, 1, 0], "2000-01-01 00:00:00");
        let p = [0, 0x59, 0x59, 0x23, 0, 0x31, 0x12, 0x99];
        r.write(&p, 1000);
        assert_eq!(r.read(1000), p);
        assert_eq!(r.read(1000 + IOP_HZ), [0, 0, 0, 0, 0, 1, 1, 0x00], "a second later the century turns, as the two digits do");
        for p in [[0, 0x30, 0x15, 0x08, 0, 0x29, 0x02, 0x24], [0, 0x00, 0x00, 0x00, 0, 0x01, 0x03, 0x23], [0, 0x01, 0x02, 0x03, 0, 0x25, 0x10, 0x26]] {
            r.write(&p, 0);
            assert_eq!(r.read(0), p);
        }
        r.write(&[0, 0x59, 0x59, 0x23, 0, 0x28, 0x02, 0x24], 0);
        assert_eq!(r.read(IOP_HZ), [0, 0, 0, 0, 0, 0x29, 0x02, 0x24], "2024 is a leap year");
    }
}
