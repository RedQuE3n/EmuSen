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
