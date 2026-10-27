//! The IOP's instruction cache and the cache control register at FFFE0130h, as the PlayStation's R3000A has them
//! (psx-spx "i-Cache" and "FFFE0130h - BCC"), 4 KiB to begin with: no public document gives the IOP's size, and Sony's
//! "minor enhancements to the cache" are unread (Sapphire_References.md §5.1; Sapphire_Disputes.md D-62). 256
//! direct-mapped lines of four words, a tag of the physical address's bits 31-12 with a valid bit per word.
//! Sapphire_Native.md §22.3; each word's decode kept beside it, §53.

use emusen_mips::{Op, decode};

pub mod bcc {
    pub const TAG: u32 = 1 << 2;
    pub const IBLKSZ_SHIFT: u32 = 8;
    pub const IS1: u32 = 1 << 11;
    /// The bits that read back: 6 and 10 read zero (psx-spx).
    pub const WRITABLE: u32 = !(1 << 6 | 1 << 10);
    /// The value the PlayStation's kernel writes (psx-spx), which the project's programs write too.
    pub const KERNEL: u32 = 0x0001_E988;
}

pub const LINES: usize = 256;

/// A fetch's result: the word, or a miss that refills `count` words from `first` (a word index in the line).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lookup {
    Hit(u32),
    Miss { first: u32, count: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ICache {
    pub bcc: u32,
    /// Per line: the tag (physical bits 31-12) with the valid bits in 3-0.
    pub tags: [u32; LINES],
    pub words: [u32; LINES * 4],
    /// Each word decoded as it entered (a refill or an isolated store), and again after a state load: never state.
    pub ops: [Op; LINES * 4],
}

impl Default for ICache {
    fn default() -> ICache {
        ICache { bcc: 0, tags: [0; LINES], words: [0; LINES * 4], ops: [decode(0); LINES * 4] }
    }
}

fn place(physical: u32) -> (usize, u32) {
    ((physical >> 4) as usize % LINES, physical >> 2 & 3)
}

impl ICache {
    pub fn enabled(&self) -> bool {
        self.bcc & bcc::IS1 != 0
    }

    /// A cached fetch at `physical`: a hit, or the words a miss refills (from the word to the line's end, two from word
    /// 0 with IBLKSZ 0; the whole line when the tag matches with the word invalid).
    pub fn lookup(&self, physical: u32) -> Lookup {
        let (line, word) = place(physical);
        let tag = self.tags[line];
        if tag & !0xFFF == physical & !0xFFF {
            if tag & 1 << word != 0 {
                return Lookup::Hit(self.words[line * 4 + word as usize]);
            }
            return Lookup::Miss { first: 0, count: 4 };
        }
        let count = if word == 0 && self.bcc >> bcc::IBLKSZ_SHIFT & 3 == 0 { 2 } else { 4 - word };
        Lookup::Miss { first: word, count }
    }

    /// A cached fetch's hit at `physical` with its word decoded, or none for a miss (`lookup` then says what it refills).
    #[inline(always)]
    pub fn hit(&self, physical: u32) -> Option<(u32, Op)> {
        let (line, word) = place(physical);
        let tag = self.tags[line];
        let k = line * 4 + word as usize;
        (tag & !0xFFF == physical & !0xFFF && tag & 1 << word != 0).then(|| (self.words[k], self.ops[k]))
    }

    /// Every word decoded again from the words, after they were restored from a state.
    pub fn redecode(&mut self) {
        for (op, &w) in self.ops.iter_mut().zip(&self.words) {
            *op = decode(w);
        }
    }

    /// The refilled words placed, from word `first` of `physical`'s line; their valid bits joined to the line's when the
    /// tag matched, replacing them when it did not.
    pub fn fill(&mut self, physical: u32, first: u32, words: &[u32]) {
        let (line, _) = place(physical);
        let mut valid = 0;
        for (i, &w) in words.iter().enumerate() {
            let k = first as usize + i;
            self.words[line * 4 + k] = w;
            self.ops[line * 4 + k] = decode(w);
            valid |= 1 << k;
        }
        let tag = self.tags[line];
        let keep = if tag & !0xFFF == physical & !0xFFF { tag & 0xF } else { 0 };
        self.tags[line] = physical & !0xFFF | keep | valid;
    }

    /// A store while SR's IsC isolates the cache: with BCC's TAG the line's tag (the offset's physical address bits, the datum's
    /// low four bits as the valid bits), else the word.
    pub fn isolated_write(&mut self, address: u32, value: u32) {
        let (line, word) = place(address);
        if self.bcc & bcc::TAG != 0 {
            self.tags[line] = value & 0xF | address & 0x1FFF_F000;
        } else {
            self.words[line * 4 + word as usize] = value;
            self.ops[line * 4 + word as usize] = decode(value);
        }
    }

    /// A load while isolated: with TAG the valid bits and the tag's match in bit 4 [C: the word's leaking upper bits read
    /// as zero], else the word.
    pub fn isolated_read(&self, address: u32) -> u32 {
        let (line, word) = place(address);
        if self.bcc & bcc::TAG != 0 {
            let tag = self.tags[line];
            return tag & 0xF | u32::from(tag & !0xFFF == address & 0x1FFF_F000) << 4;
        }
        self.words[line * 4 + word as usize]
    }
}
