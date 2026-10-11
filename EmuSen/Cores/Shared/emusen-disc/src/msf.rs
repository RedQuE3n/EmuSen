//! A disc's geometry (ECMA-130 §21, §22.3.3): minutes, seconds and 1/75 s frames, the absolute sector number they
//! name, counted from 00:00:00, and the binary-coded decimal the subchannel and the sector headers carry.

/// Sectors a second at the standard speed, and a minute.
pub const FRAMES: u32 = 75;
pub const MINUTE: u32 = 60 * FRAMES;

/// Where the first track's user data begins, 00:02:00, after the two-second pause every disc opens with.
pub const PROGRAM_START: u32 = 2 * FRAMES;

/// An absolute time on the disc, each field binary.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Msf {
    pub m: u8,
    pub s: u8,
    pub f: u8,
}

impl Msf {
    pub const fn new(m: u8, s: u8, f: u8) -> Msf {
        Msf { m, s, f }
    }

    /// The time of absolute sector `n`; minutes past 99 wrap, as two BCD digits do.
    pub const fn of(n: u32) -> Msf {
        Msf { m: (n / MINUTE % 100) as u8, s: (n / FRAMES % 60) as u8, f: (n % FRAMES) as u8 }
    }

    pub const fn sector(self) -> u32 {
        self.m as u32 * MINUTE + self.s as u32 * FRAMES + self.f as u32
    }

    /// From three BCD bytes, refused unless each is valid BCD with seconds below 60 and frames below 75.
    pub fn from_bcd(b: [u8; 3]) -> Option<Msf> {
        let (m, s, f) = (from_bcd(b[0])?, from_bcd(b[1])?, from_bcd(b[2])?);
        (s < 60 && f < 75).then_some(Msf { m, s, f })
    }

    pub const fn bcd(self) -> [u8; 3] {
        [bcd(self.m), bcd(self.s), bcd(self.f)]
    }

    /// From `mm:ss:ff` as a cue sheet writes it.
    pub fn parse(text: &str) -> Option<Msf> {
        let mut p = text.trim().split(':').map(|x| x.parse::<u32>().ok());
        let (m, s, f) = (p.next()??, p.next()??, p.next()??);
        (p.next().is_none() && m < 100 && s < 60 && f < 75).then_some(Msf { m: m as u8, s: s as u8, f: f as u8 })
    }
}

/// Two decimal digits as packed BCD.
pub const fn bcd(v: u8) -> u8 {
    (v / 10) << 4 | v % 10
}

/// A packed BCD byte's value, if both digits are decimal.
pub const fn from_bcd(b: u8) -> Option<u8> {
    if b >> 4 > 9 || b & 0xF > 9 { None } else { Some((b >> 4) * 10 + (b & 0xF)) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn times_and_sectors_convert_both_ways() {
        assert_eq!(Msf::of(150), Msf::new(0, 2, 0));
        assert_eq!(Msf::new(74, 59, 74).sector(), 74 * 4500 + 59 * 75 + 74);
        for n in [0, 1, 74, 75, 4499, 4500, 333_000] {
            assert_eq!(Msf::of(n).sector(), n);
        }
        assert_eq!(Msf::new(12, 34, 56).bcd(), [0x12, 0x34, 0x56]);
        assert_eq!(Msf::from_bcd([0x12, 0x34, 0x56]), Some(Msf::new(12, 34, 56)));
        assert_eq!(Msf::from_bcd([0x00, 0x60, 0x00]), None);
        assert_eq!(Msf::from_bcd([0x00, 0x00, 0x75]), None);
        assert_eq!(Msf::from_bcd([0x0A, 0x00, 0x00]), None);
        assert_eq!(Msf::parse("08:09:29"), Some(Msf::new(8, 9, 29)));
        assert_eq!(Msf::parse("08:60:29"), None);
    }
}
