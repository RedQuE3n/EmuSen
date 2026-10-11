//! Subchannel Q (ECMA-130 §22.3): the control and mode nibble, the twelve bytes of q-mode 1's position or of the
//! lead-in's table of contents, and the CRC of §22.3.6 over the first ten.

use crate::msf::{Msf, bcd};

/// The control field's values (§22.3.1, psx-spx "Subchannel Q"): audio, and digital data (copy prohibited).
pub const CONTROL_AUDIO: u8 = 0x0;
pub const CONTROL_DATA: u8 = 0x4;
/// The lead-out's track number (§22.3.3.1).
pub const LEAD_OUT: u8 = 0xAA;

/// G(x) = x¹⁶ + x¹² + x⁵ + 1, most significant bit first, the register starting at zero; the field holds it inverted.
pub fn crc16(data: &[u8]) -> u16 {
    let mut r: u16 = 0;
    for &b in data {
        r ^= (b as u16) << 8;
        for _ in 0..8 {
            r = if r & 0x8000 != 0 { r << 1 ^ 0x1021 } else { r << 1 };
        }
    }
    !r
}

/// One sector's Q: bytes 0–9 and the CRC, big-endian, in 10–11.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SubQ(pub [u8; 12]);

impl SubQ {
    /// Q-mode 1 in the user data and lead-out areas: track, index, the relative time, a zero, the absolute time.
    pub fn position(control: u8, track: u8, index: u8, relative: Msf, absolute: Msf) -> SubQ {
        let r = relative.bcd();
        let a = absolute.bcd();
        SubQ::sealed([control << 4 | 1, track, index, r[0], r[1], r[2], 0, a[0], a[1], a[2]])
    }

    /// Ten bytes with their CRC.
    pub fn sealed(q: [u8; 10]) -> SubQ {
        let c = crc16(&q);
        let mut b = [0u8; 12];
        b[..10].copy_from_slice(&q);
        b[10..].copy_from_slice(&c.to_be_bytes());
        SubQ(b)
    }

    /// Whether the CRC matches, which a drive needs before it trusts the position.
    pub fn valid(&self) -> bool {
        crc16(&self.0[..10]).to_be_bytes() == [self.0[10], self.0[11]]
    }

    pub fn control(&self) -> u8 {
        self.0[0] >> 4
    }

    pub fn track(&self) -> u8 {
        self.0[1]
    }

    pub fn index(&self) -> u8 {
        self.0[2]
    }

    /// Bytes 1–9 in GetlocP's order: track, index, the relative time, then the absolute time (its zero skipped).
    pub fn getloc(&self) -> [u8; 8] {
        let q = &self.0;
        [q[1], q[2], q[3], q[4], q[5], q[7], q[8], q[9]]
    }
}

/// A lead-in item (§22.3.4): track 0, the pointer and its time, the item's own running time.
pub fn toc_item(control: u8, pointer: u8, p: [u8; 3], running: Msf) -> SubQ {
    let r = running.bcd();
    SubQ::sealed([control << 4 | 1, 0, pointer, r[0], r[1], r[2], 0, p[0], p[1], p[2]])
}

/// A track number as the TOC's pointer writes it.
pub fn pointer(track: u8) -> u8 {
    bcd(track)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The CRC as a remainder: the ten bytes followed by the uninverted CRC divide by G(x).
    #[test]
    fn the_crc_is_the_standards_and_detects_a_flipped_bit() {
        let q = SubQ::position(CONTROL_DATA, 1, 1, Msf::new(0, 0, 0), Msf::new(0, 2, 0));
        assert!(q.valid());
        let mut all = q.0;
        all[10] ^= 0xFF;
        all[11] ^= 0xFF;
        let mut r: u32 = 0;
        for &b in &all {
            for k in (0..8).rev() {
                r = r << 1 | (b >> k & 1) as u32;
                if r & 0x1_0000 != 0 {
                    r ^= 0x1_1021;
                }
            }
        }
        assert_eq!(r, 0);
        let mut bad = q;
        bad.0[3] ^= 1;
        assert!(!bad.valid());
        assert_eq!(q.getloc(), [1, 1, 0, 0, 0, 0, 2, 0]);
        assert_eq!(q.0[0], 0x41);
    }
}
