//! The VR4300's TLB, `Cpu.Tlb`: thirty-two entries as the C# `TlbEntry` struct holds them.

use crate::state::{State, StateReader, StateResult, StateWriter};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct TlbEntry {
    pub entry_hi: u64,
    pub entry_lo0: u64,
    pub entry_lo1: u64,
    pub page_mask: u64,
}

impl State for TlbEntry {
    fn write_state(&self, w: &mut StateWriter) {
        w.u64("EntryHi", self.entry_hi);
        w.u64("EntryLo0", self.entry_lo0);
        w.u64("EntryLo1", self.entry_lo1);
        w.u64("PageMask", self.page_mask);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.entry_hi = r.u64()?; // EntryHi
        self.entry_lo0 = r.u64()?; // EntryLo0
        self.entry_lo1 = r.u64()?; // EntryLo1
        self.page_mask = r.u64()?; // PageMask
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Tlb {
    pub entries: [TlbEntry; 32],
}

impl State for Tlb {
    fn write_state(&self, w: &mut StateWriter) {
        w.structures("Entries", &self.entries);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        r.structures(&mut self.entries)?; // Entries
        Ok(())
    }
}

pub const ENTRY_LO_GLOBAL: u64 = 1 << 0;
pub const ENTRY_LO_VALID: u64 = 1 << 1;
pub const ENTRY_LO_DIRTY: u64 = 1 << 2;
/// `EntryLoKept`: twenty frame bits and the cache, dirty and valid flags.
pub const ENTRY_LO_KEPT: u64 = 0x03FF_FFFE;
/// `MatchedBits`: the region bits and a forty-bit page number.
pub const MATCHED_BITS: u64 = 0xC000_00FF_FFFF_FFFF;

/// `TlbResult`, with the mapped address carried by its variant.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TlbResult {
    Missing,
    Invalid,
    NotWritable,
    Mapped(u32),
}

impl Tlb {
    /// `PairedPageMask`: each pair of mask bits is decided by its higher bit alone.
    pub fn paired_page_mask(raw: u64) -> u64 {
        let higher = raw & 0x0155_4000;
        higher | (higher >> 1)
    }

    /// `TryTranslate`, the entries searched in order.
    pub fn try_translate(&self, address: u64, asid: u64, store: bool) -> TlbResult {
        for entry in &self.entries {
            let pair_mask = entry.page_mask | 0x1FFF;
            let page_mask = pair_mask >> 1;
            if ((address ^ entry.entry_hi) & !pair_mask & MATCHED_BITS) != 0 {
                continue;
            }
            let global = (entry.entry_lo0 & entry.entry_lo1 & ENTRY_LO_GLOBAL) != 0;
            if !global && (entry.entry_hi & 0xFF) != (asid & 0xFF) {
                continue;
            }
            let half = if address & (page_mask + 1) != 0 { entry.entry_lo1 } else { entry.entry_lo0 };
            if half & ENTRY_LO_VALID == 0 {
                return TlbResult::Invalid;
            }
            if store && half & ENTRY_LO_DIRTY == 0 {
                return TlbResult::NotWritable;
            }
            let frame = ((half >> 6) & 0x00FF_FFFF) << 12;
            return TlbResult::Mapped(((frame & !page_mask) | (address & page_mask)) as u32);
        }
        TlbResult::Missing
    }

    /// `Probe`: only whether something matches, usable or not.
    pub fn probe(&self, entry_hi: u64) -> i32 {
        for (i, entry) in self.entries.iter().enumerate() {
            let pair_mask = entry.page_mask | 0x1FFF;
            if ((entry_hi ^ entry.entry_hi) & !pair_mask & MATCHED_BITS) != 0 {
                continue;
            }
            let global = (entry.entry_lo0 & entry.entry_lo1 & ENTRY_LO_GLOBAL) != 0;
            if !global && (entry.entry_hi & 0xFF) != (entry_hi & 0xFF) {
                continue;
            }
            return i as i32;
        }
        -1
    }
}
