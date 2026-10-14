//! Word RAM: 256 KiB shared between the two CPUs, whole in 2M mode and as two 128 KiB banks in 1M mode, with the
//! memory mode register's swap rules and the views each CPU has of it. The Mega-CD Hardware Manual §1-1 to §1-3, §3-1
//! (`$FF8002`) and §4-1 (`$A12002`) are the sources; the swap rules the manual leaves open are mcd-verificator's
//! (Nephrite_Native.md §50).

/// The RAM's bytes in 2M order: bank 0's word n is word 2n, bank 1's word 2n + 1 (the manual's §1-1 "data arrangement").
pub const SIZE: usize = 0x4_0000;

#[derive(Clone)]
pub struct WordRam {
    pub bytes: Vec<u8>,
    /// MODE: 1M when true.
    pub one_m: bool,
    /// In 2M, the RAM is the sub CPU's: DMNA as the main CPU sets it, cleared by the sub CPU's RET.
    pub sub_owns: bool,
    /// In 1M, RET: bank 0 the sub CPU's and bank 1 the main CPU's when set.
    pub ret: bool,
    /// In 1M, DMNA: a swap the main CPU asked for and the sub CPU has not made by changing RET.
    pub request: bool,
    /// PM1-PM0: the priority mode of a write to the dot image.
    pub priority: u8,
}

impl Default for WordRam {
    fn default() -> WordRam {
        WordRam { bytes: vec![0; SIZE], one_m: false, sub_owns: false, ret: false, request: false, priority: 0 }
    }
}

/// Where an access lands in the RAM: a byte of the 2M array, or nowhere the CPU may reach now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lands {
    Byte(usize),
    /// A pixel of the 1M dot image: the byte, and whether the pixel is its high nibble.
    Dot(usize, bool),
    /// The RAM is the other CPU's: a main CPU's access sees the open bus, a sub CPU's waits.
    Elsewhere,
}

/// Bank `b`'s byte `j` in the 2M array.
pub fn bank_byte(b: usize, j: usize) -> usize {
    ((j >> 1) << 2) | (b << 1) | (j & 1)
}

/// The cell image's word `i` (0 to `$FFFF`) as the bank word it shows: five strips of cells, 32, 16, 8, 4 and 4 cells
/// high, each read cell by cell from its dot rows (the manual's §1-3; mcd-verificator's `testVramMapping`).
pub fn cell_word(i: usize) -> usize {
    let (slot, base) = match i {
        0x0000..=0x7FFF => (0, 0x0000),
        0x8000..=0xBFFF => (1, 0x8000),
        0xC000..=0xDFFF => (2, 0xC000),
        0xE000..=0xEFFF => (3, 0xE000),
        _ => (3, 0xF000),
    };
    let n = i - base;
    let hi = 0x1FE & (0x1FE >> slot);
    let lo = 0x7E00 >> slot;
    base + (((n & hi) << 6) | ((n & lo) >> (8 - slot)) | (n & 1))
}

impl WordRam {
    /// MODE, DMNA and RET as either CPU reads them.
    pub fn mode_bits(&self) -> u8 {
        let (dmna, ret) = if self.one_m { (self.request, self.ret) } else { (self.sub_owns, !self.sub_owns) };
        (self.one_m as u8) << 2 | (dmna as u8) << 1 | ret as u8
    }

    /// The main CPU's write of DMNA: 1 gives the 2M RAM to the sub CPU in either mode; 0 asks for a 1M swap.
    pub fn main_dmna(&mut self, dmna: bool) {
        if dmna {
            self.sub_owns = true;
        } else if self.one_m {
            self.request = true;
        }
    }

    /// The sub CPU's write of MODE and RET. Into 1M, RET picks the banks and a change of RET makes a swap the main CPU
    /// asked for; in 2M, RET 1 gives the RAM to the main CPU and 0 does nothing.
    pub fn sub_mode(&mut self, one_m: bool, ret: bool) {
        if one_m {
            if self.one_m && ret != self.ret {
                self.request = false;
            }
            if !self.one_m {
                self.request = false;
            }
            (self.one_m, self.ret) = (true, ret);
        } else {
            self.one_m = false;
            if ret {
                self.sub_owns = false;
            }
        }
    }

    /// The bank the main CPU holds in 1M, and the sub CPU's.
    pub fn main_bank(&self) -> usize {
        self.ret as usize
    }

    pub fn sub_bank(&self) -> usize {
        !self.ret as usize
    }

    /// A main CPU address in `$200000`-`$3FFFFF`.
    pub fn main(&self, a: u32) -> Lands {
        let o = (a & 0x3_FFFF) as usize;
        if !self.one_m {
            return if self.sub_owns { Lands::Elsewhere } else { Lands::Byte(o) };
        }
        let b = self.main_bank();
        if o < 0x2_0000 {
            Lands::Byte(bank_byte(b, o))
        } else {
            let i = (o - 0x2_0000) >> 1;
            Lands::Byte(bank_byte(b, cell_word(i) << 1 | (o & 1)))
        }
    }

    /// A sub CPU address in `$080000`-`$0DFFFF`.
    pub fn sub(&self, a: u32) -> Lands {
        let a = a as usize;
        if !self.one_m {
            return if self.sub_owns && a < 0xC_0000 { Lands::Byte(a - 0x8_0000) } else { Lands::Elsewhere };
        }
        let b = self.sub_bank();
        if a >= 0xC_0000 {
            Lands::Byte(bank_byte(b, a - 0xC_0000))
        } else {
            let p = a - 0x8_0000;
            Lands::Dot(bank_byte(b, p >> 1), p & 1 == 0)
        }
    }

    pub fn read8(&self, at: Lands) -> Option<u8> {
        match at {
            Lands::Byte(i) => Some(self.bytes[i]),
            Lands::Dot(i, high) => Some(if high { self.bytes[i] >> 4 } else { self.bytes[i] & 15 }),
            Lands::Elsewhere => None,
        }
    }

    /// A byte written; a pixel by the priority mode: underwrite (PM0) only over 0, overwrite (PM1) only when not 0.
    pub fn write8(&mut self, at: Lands, v: u8) {
        match at {
            Lands::Byte(i) => self.bytes[i] = v,
            Lands::Dot(i, high) => {
                let (old, new) = (if high { self.bytes[i] >> 4 } else { self.bytes[i] & 15 }, v & 15);
                let keep = match self.priority {
                    1 => old != 0,
                    2 => new == 0,
                    _ => false,
                };
                if !keep {
                    self.bytes[i] = if high { (self.bytes[i] & 0x0F) | new << 4 } else { (self.bytes[i] & 0xF0) | new };
                }
            }
            Lands::Elsewhere => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_banks_interleave_by_words_in_2m() {
        assert_eq!((bank_byte(0, 0), bank_byte(0, 1), bank_byte(1, 0), bank_byte(1, 1), bank_byte(0, 2)), (0, 1, 2, 3, 4));
    }

    #[test]
    fn the_cell_image_reads_its_strips_cell_by_cell() {
        // Cell-map word 0 is dot-map word 0; word 2 (the cell's next row) is dot-map word $100 (a 512-dot row), as the
        // manual's §1-3 table has it, and the 16-cell strip begins at the 32-cell strip's end.
        assert_eq!((cell_word(0), cell_word(1), cell_word(2), cell_word(3)), (0, 1, 0x80, 0x81));
        assert_eq!((cell_word(0x200), cell_word(0x201)), (2, 3), "cell byte $400 shows dots 8 to 11");
        assert_eq!(cell_word(0x8000), 0x8000);
        let all: std::collections::BTreeSet<usize> = (0..0x1_0000).map(cell_word).collect();
        assert_eq!(all.len(), 0x1_0000, "every bank word is shown once");
    }

    #[test]
    fn the_swaps_follow_dmna_and_ret() {
        let mut w = WordRam::default();
        assert_eq!(w.mode_bits(), 0b001, "2M, the main CPU's");
        w.main_dmna(true);
        assert_eq!(w.mode_bits(), 0b010);
        w.sub_mode(false, false);
        assert_eq!(w.mode_bits(), 0b010, "RET 0 in 2M does nothing");
        w.sub_mode(false, true);
        assert_eq!(w.mode_bits(), 0b001);
        w.sub_mode(true, true);
        assert_eq!(w.mode_bits(), 0b101);
        w.main_dmna(false);
        assert_eq!(w.mode_bits(), 0b111, "a swap asked for");
        w.sub_mode(true, true);
        assert_eq!(w.mode_bits(), 0b111, "RET unchanged makes no swap");
        w.sub_mode(true, false);
        assert_eq!(w.mode_bits(), 0b100);
    }
}
