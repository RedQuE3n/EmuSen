//! A memory card (SCPH-1020) on SIO0 after its address 81h (psx-spx "Memory Card Read/Write Commands"): Read, Write
//! and Get ID, the flag byte, the checksum, and the late /ACK of a read. Its 131,072 bytes are the raw card, frame 0
//! first, as a battery file holds it. Diamond_Pad.md §8.

use crate::pad::{HIGH_Z, Reply};

pub const CARD_BYTES: usize = 0x2_0000;
pub const SECTORS: u32 = 0x400;
/// The flag byte at power-on and on insertion: bit 3, the directory not yet read (psx-spx "FLAG Byte").
pub const FLAG_NEW: u8 = 0x08;
/// The extra delay of the /ACK after a read's 5Ch byte, an original Sony card's (psx-spx "Timings") [C on its byte].
pub const READ_ACK_EXTRA: u32 = 31_000;

pub const READ: u8 = 0x52;
pub const WRITE: u8 = 0x57;
pub const ID: u8 = 0x53;

#[derive(Clone, PartialEq, Eq)]
pub struct Card {
    pub data: Vec<u8>,
    pub flag: u8,
    /// Whether a write changed the data since the host last saved it.
    pub changed: bool,
    /// The command under way, the sector it names, the checksum so far, and the last byte received.
    pub command: u8,
    pub sector: u32,
    pub checksum: u8,
    pub previous: u8,
    pub buffer: [u8; 128],
}

impl std::fmt::Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Card {{ flag: {:02X}, changed: {} }}", self.flag, self.changed)
    }
}

impl Card {
    /// A card holding `data`, or an unformatted card of zeros when there is none [C].
    pub fn new(data: Option<&[u8]>) -> Card {
        let mut d = vec![0u8; CARD_BYTES];
        if let Some(src) = data {
            let n = src.len().min(CARD_BYTES);
            d[..n].copy_from_slice(&src[..n]);
        }
        Card { data: d, flag: FLAG_NEW, changed: false, command: 0, sector: 0, checksum: 0, previous: 0, buffer: [0; 128] }
    }

    fn valid(&self) -> bool {
        self.sector < SECTORS
    }

    fn at(&self, i: usize) -> u8 {
        self.data[(self.sector as usize & (SECTORS as usize - 1)) * 128 + i]
    }

    /// The reply to the byte at `step` after the address, 1 being the command; whether the transfer goes on.
    pub fn exchange(&mut self, step: u8, byte: u8) -> (Reply, bool) {
        let previous = self.previous;
        self.previous = byte;
        let s = step as u32;
        if s == 1 {
            self.command = byte;
            self.checksum = 0;
            let ok = matches!(byte, READ | WRITE | ID);
            return (if ok { Reply::more(self.flag) } else { Reply::last(self.flag) }, ok);
        }
        let go = |b: u8| (Reply::more(b), true);
        match (self.command, s) {
            (_, 2) => go(0x5A),
            (_, 3) => go(0x5D),
            (ID, 4) => go(0x5C),
            (ID, 5) => go(0x5D),
            (ID, 6) => go(0x04),
            (ID, 7) | (ID, 8) => go(0x00),
            (ID, 9) => (Reply::last(0x80), false),
            (READ | WRITE, 4) => {
                self.sector = (byte as u32) << 8;
                self.checksum = byte;
                go(0x00)
            }
            (READ | WRITE, 5) => {
                self.sector |= byte as u32;
                self.checksum ^= byte;
                go(previous)
            }
            (READ, 6) => (Reply { byte: 0x5C, ack: Some(READ_ACK_EXTRA) }, true),
            (READ, 7) => go(0x5D),
            (READ, 8) if !self.valid() => go(0xFF),
            (READ, 9) if !self.valid() => (Reply::last(0xFF), false),
            (READ, 8) => go((self.sector >> 8) as u8),
            (READ, 9) => go(self.sector as u8),
            (READ, 10..=137) => {
                let v = self.at(s as usize - 10);
                self.checksum ^= v;
                go(v)
            }
            (READ, 138) => go(self.checksum),
            (READ, 139) => (Reply::last(0x47), false),
            (WRITE, 6..=133) => {
                self.buffer[s as usize - 6] = byte;
                self.checksum ^= byte;
                go(previous)
            }
            (WRITE, 134) => {
                self.checksum ^= byte;
                go(previous)
            }
            (WRITE, 135) => go(0x5C),
            (WRITE, 136) => go(0x5D),
            (WRITE, 137) => (Reply::last(self.finish_write()), false),
            _ => (HIGH_Z, false),
        }
    }

    /// The end byte of a write, and the sector stored when it is good: 47h, 4Eh for a bad checksum, FFh for a bad
    /// sector. A good write clears the flag's bit 3 (psx-spx "FLAG Byte").
    fn finish_write(&mut self) -> u8 {
        if !self.valid() {
            return 0xFF;
        }
        if self.checksum != 0 {
            return 0x4E;
        }
        let at = self.sector as usize * 128;
        self.data[at..at + 128].copy_from_slice(&self.buffer);
        self.changed = true;
        self.flag &= !FLAG_NEW;
        0x47
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transfer(c: &mut Card, bytes: &[u8]) -> Vec<Reply> {
        let mut out = Vec::new();
        for (i, &b) in bytes.iter().enumerate() {
            let (r, more) = c.exchange(i as u8 + 1, b);
            out.push(r);
            if !more {
                break;
            }
        }
        out
    }

    fn write(sector: u16, data: &[u8; 128], spoil: u8) -> Vec<u8> {
        let mut b = vec![WRITE, 0, 0, (sector >> 8) as u8, sector as u8];
        b.extend_from_slice(data);
        b.push(data.iter().fold((sector >> 8) as u8 ^ sector as u8, |c, &d| c ^ d) ^ spoil);
        b.extend_from_slice(&[0, 0, 0]);
        b
    }

    #[test]
    fn a_write_then_a_read_round_trip_a_sector_with_its_checksum() {
        let mut c = Card::new(None);
        let data: [u8; 128] = std::array::from_fn(|i| (i * 3) as u8);
        let w = transfer(&mut c, &write(0x3F, &data, 0));
        assert_eq!((w[0].byte, w.len(), w.last().unwrap().byte), (FLAG_NEW, 137, 0x47));
        assert_eq!((c.flag, c.changed), (0, true), "a write clears the new-card bit");
        let mut r = vec![READ, 0, 0, 0, 0x3F];
        r.resize(139, 0);
        let got = transfer(&mut c, &r);
        assert_eq!(got.iter().take(9).map(|r| r.byte).collect::<Vec<_>>(), [0, 0x5A, 0x5D, 0, 0, 0x5C, 0x5D, 0, 0x3F]);
        assert_eq!(got[5].ack, Some(READ_ACK_EXTRA), "the late /ACK after 5Ch");
        assert_eq!(got[9..137].iter().map(|r| r.byte).collect::<Vec<_>>(), data);
        assert_eq!((got[137].byte, got[138]), (data.iter().fold(0x3F, |c, &d| c ^ d), Reply::last(0x47)));
    }

    #[test]
    fn a_bad_checksum_a_bad_sector_an_id_and_an_unknown_command() {
        let mut c = Card::new(None);
        assert_eq!(transfer(&mut c, &write(5, &[1; 128], 1)).last().unwrap().byte, 0x4E);
        assert_eq!(transfer(&mut c, &write(0x400, &[1; 128], 0)).last().unwrap().byte, 0xFF);
        assert!(!c.changed && c.data.iter().all(|&b| b == 0));
        let mut r = vec![READ, 0, 0, 0x04, 0x00];
        r.resize(139, 0);
        assert_eq!(transfer(&mut c, &r).len(), 9, "FFFFh confirmed, then the transfer ends");
        let id: Vec<u8> = transfer(&mut c, &[ID, 0, 0, 0, 0, 0, 0, 0, 0]).iter().map(|r| r.byte).collect();
        assert_eq!(id, [8, 0x5A, 0x5D, 0x5C, 0x5D, 0x04, 0x00, 0x00, 0x80]);
        assert_eq!(transfer(&mut c, &[0x41, 0]), [Reply::last(8)]);
    }
}
