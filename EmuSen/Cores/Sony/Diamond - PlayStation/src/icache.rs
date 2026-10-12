//! The R3000A's 4 KiB instruction cache and the BIU/cache control register at FFFE0130h (psx-spx "i-Cache" and
//! "FFFE0130h - BCC"): 256 direct-mapped lines of four words, a tag of the physical address's bits 31-12 with a valid
//! bit per word, and the isolated stores the kernel's flush makes. It lives in the console's bus, not the processor's
//! crate (BlackMoon_R3000.md §2, decided at stage 3).

use emusen_mips::{Op, decode};

pub mod bcc {
    pub const TAG: u32 = 1 << 2;
    pub const IBLKSZ: u32 = 3 << 8;
    pub const IS1: u32 = 1 << 11;
    /// The bits that read back: 6 and 10 read zero.
    pub const WRITABLE: u32 = !(1 << 6 | 1 << 10);
}

/// The register's value as the kernel leaves it (psx-spx: RAM, DS, a four-word refill, IS1, RDPRI, NOPAD, BGNT, LDSCH).
pub const KERNEL_BCC: u32 = 0x0001_E988;

#[derive(Clone)]
pub struct ICache {
    pub bcc: u32,
    /// Per line: the tag (physical bits 31-12) with the valid bits in 3-0.
    pub tags: [u32; 256],
    pub words: [u32; 1024],
    /// Each word decoded when it entered the cache, which a hit runs without decoding (Diamond_Speed.md §5); not state.
    ops: [Op; 1024],
}

/// A fetch's result: the word, or a miss that refills `count` words from `first` (a word index in the line).
pub enum Lookup {
    Hit(u32),
    Miss { first: u32, count: u32 },
}

impl ICache {
    pub fn new(bcc: u32) -> ICache {
        ICache { bcc, tags: [0; 256], words: [0; 1024], ops: [decode(0); 1024] }
    }

    /// The decoded words made again from the words, as after a state load.
    pub fn redecode(&mut self) {
        for (op, &w) in self.ops.iter_mut().zip(&self.words) {
            *op = decode(w);
        }
    }

    /// A hit's word and its decoding, or `None` where `lookup` misses.
    #[inline(always)]
    pub fn hit(&self, physical: u32) -> Option<(u32, Op)> {
        let line = (physical >> 4 & 0xFF) as usize;
        let word = physical >> 2 & 3;
        let tag = self.tags[line];
        if tag & !0xFFF == physical & !0xFFF && tag & 1 << word != 0 {
            let i = line * 4 + word as usize;
            return Some((self.words[i], self.ops[i]));
        }
        None
    }

    pub fn enabled(&self) -> bool {
        self.bcc & bcc::IS1 != 0
    }

    pub fn lookup(&self, physical: u32) -> Lookup {
        let line = (physical >> 4 & 0xFF) as usize;
        let word = physical >> 2 & 3;
        let tag = self.tags[line];
        let matches = tag & !0xFFF == physical & !0xFFF;
        if matches && tag & 1 << word != 0 {
            return Lookup::Hit(self.words[line * 4 + word as usize]);
        }
        if matches {
            return Lookup::Miss { first: 0, count: 4 };
        }
        let count = if word == 0 && self.bcc & bcc::IBLKSZ == 0 { 2 } else { 4 - word };
        Lookup::Miss { first: word, count }
    }

    /// Words fetched from memory into `physical`'s line, from word `first`; the valid bits gather when the tag matches.
    pub fn fill(&mut self, physical: u32, first: u32, words: &[u32]) {
        let line = (physical >> 4 & 0xFF) as usize;
        let tag = physical & !0xFFF;
        let mut valid = if self.tags[line] & !0xFFF == tag { self.tags[line] & 0xF } else { 0 };
        for (i, &w) in words.iter().enumerate() {
            let at = first as usize + i;
            self.words[line * 4 + at] = w;
            self.ops[line * 4 + at] = decode(w);
            valid |= 1 << at;
        }
        self.tags[line] = tag | valid;
    }

    /// A store while SR's IsC isolates the cache: with TAG set the line's tag takes the offset's address bits and the
    /// datum's low four bits as its valid bits; otherwise the word is written. Bytes and halfwords write the word.
    pub fn isolated_write(&mut self, address: u32, value: u32) {
        let line = (address >> 4 & 0xFF) as usize;
        if self.bcc & bcc::TAG != 0 {
            self.tags[line] = address & !0xFFF | value & 0xF;
        } else {
            self.words[line * 4 + (address >> 2 & 3) as usize] = value;
            self.ops[line * 4 + (address >> 2 & 3) as usize] = decode(value);
        }
    }

    /// A load while isolated: with TAG, the valid bits and whether the tag matches in bit 4, the line's word above
    /// them; otherwise the word.
    pub fn isolated_read(&self, address: u32) -> u32 {
        let line = (address >> 4 & 0xFF) as usize;
        let word = self.words[line * 4 + (address >> 2 & 3) as usize];
        if self.bcc & bcc::TAG == 0 {
            return word;
        }
        let tag = self.tags[line];
        let matches = (tag & !0xFFF == address & 0x1FFF_F000) as u32;
        word & !0x1F | matches << 4 | tag & 0xF
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every word the cache holds is held decoded too, after a refill, an isolated store and a state's words.
    #[test]
    fn each_word_is_decoded_as_it_enters() {
        let mut c = ICache::new(KERNEL_BCC);
        let mut words = [0u32; 4];
        for (i, w) in words.iter_mut().enumerate() {
            *w = 0x0C00_0000 | (i as u32) << 2;
        }
        c.fill(0x2000, 0, &words);
        c.isolated_write(0x2008, 0x2408_0001);
        for at in [0x2000, 0x2004, 0x2008, 0x200C] {
            let (w, op) = c.hit(at).expect("a hit");
            assert_eq!(op, decode(w), "{at:X}");
        }
        assert_eq!(c.hit(0x2008).unwrap().0, 0x2408_0001);
        let mut loaded = ICache::new(KERNEL_BCC);
        loaded.tags = c.tags;
        loaded.words = c.words;
        loaded.redecode();
        assert_eq!(loaded.hit(0x2008), c.hit(0x2008), "decoded again from a state's words");
        assert_eq!(c.hit(0x3000), None);
    }

    #[test]
    fn a_miss_fills_to_the_end_of_the_line_and_a_tag_flush_empties_it() {
        let mut c = ICache::new(KERNEL_BCC);
        let at = 0x0001_2348;
        let Lookup::Miss { first, count } = c.lookup(at) else { panic!() };
        assert_eq!((first, count), (2, 2));
        c.fill(at, first, &[0xAAAA, 0xBBBB]);
        assert!(matches!(c.lookup(at), Lookup::Hit(0xAAAA)));
        assert!(matches!(c.lookup(at + 4), Lookup::Hit(0xBBBB)));
        assert!(matches!(c.lookup(at - 8), Lookup::Miss { first: 0, count: 4 }), "the tag matches, word 0 is invalid");
        assert!(matches!(c.lookup(at + 0x1000), Lookup::Miss { first: 2, count: 2 }), "another tag");
        c.bcc |= bcc::TAG;
        c.isolated_write(at & 0xFF0, 0);
        c.bcc &= !bcc::TAG;
        assert!(matches!(c.lookup(at), Lookup::Miss { .. }));
        let mut two = ICache::new(KERNEL_BCC & !bcc::IBLKSZ);
        assert!(matches!(two.lookup(0x1100), Lookup::Miss { first: 0, count: 2 }), "IBLKSZ 0: two words from word 0");
        two.fill(0x1100, 0, &[1, 2]);
        assert_eq!(two.tags[0x10] & 0xF, 3);
        assert!(matches!(ICache::new(KERNEL_BCC).lookup(0x100), Lookup::Miss { first: 0, count: 4 }), "a zero tag matches the first 4 KiB");
    }
}
