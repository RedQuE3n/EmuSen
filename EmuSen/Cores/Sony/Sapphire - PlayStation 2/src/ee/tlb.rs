//! The TLB: 48 fully associative entries of even and odd page pairs, seven page sizes, the ASID and its G bit, and
//! the S bit that maps the scratchpad (EE Core User's Manual §5.2; EE Core Instruction Set Manual, TLBP, TLBR,
//! TLBWI, TLBWR). Each entry is held as the manual's 128 bits in four words; Sapphire_Native.md §17.2 has the readings.

pub const ENTRIES: usize = 48;

/// One entry as Figure 5-8 lays it out: bits 127..96 (PageMask), 95..64 (VPN2, G at bit 76, ASID), 63..32 (S and
/// the even page's PFN, C, D and V) and 31..0 (the odd page's).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Entry {
    pub mask: u32,
    pub hi: u32,
    pub lo0: u32,
    pub lo1: u32,
}

const G: u32 = 1 << 12;
const S: u32 = 1 << 31;
const VPN2: u32 = 0xFFFF_E000;

impl Entry {
    pub fn global(&self) -> bool {
        self.hi & G != 0
    }

    pub fn scratchpad(&self) -> bool {
        self.lo0 & S != 0
    }

    /// The bits of the virtual address the entry compares: VPN2 above the page pair, and above 16 KiB for the scratchpad.
    fn compared(&self) -> u32 {
        if self.scratchpad() { 0xFFFF_C000 } else { VPN2 & !self.mask }
    }

    fn matches(&self, vaddr: u32, asid: u32) -> bool {
        let c = self.compared();
        vaddr & c == self.hi & c && (self.global() || self.hi & 0xFF == asid)
    }
}

/// What a matching entry says of one address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hit {
    pub index: usize,
    pub valid: bool,
    pub dirty: bool,
    pub target: Target,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// The scratchpad, at this byte offset.
    Scratchpad(u32),
    /// A physical address with its page's C field.
    Physical { paddr: u32, c: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tlb {
    pub entries: [Entry; ENTRIES],
}

impl Default for Tlb {
    fn default() -> Self {
        Tlb { entries: [Entry::default(); ENTRIES] }
    }
}

impl Tlb {
    /// TLBWI's and TLBWR's Operation: G is the AND of both G bits, VPN2 masked by PageMask, bit 0 of each low half zero.
    pub fn write(&mut self, index: usize, pagemask: u32, entryhi: u32, lo0: u32, lo1: u32) {
        let g = lo0 & lo1 & 1;
        self.entries[index] =
            Entry { mask: pagemask, hi: (entryhi & VPN2 | g << 12 | entryhi & 0xFFF) & !pagemask, lo0: lo0 & !1, lo1: lo1 & !1 };
    }

    /// TLBR's Operation: (PageMask, EntryHi, EntryLo0, EntryLo1); a scratchpad entry's C fields read 2 (§5.2.5).
    pub fn read(&self, index: usize) -> (u32, u32, u32, u32) {
        let e = &self.entries[index];
        let g = u32::from(e.global());
        let c = |lo: u32| if e.scratchpad() { lo & !(7 << 3) | 2 << 3 } else { lo };
        (e.mask, e.hi & 0xFFFF_E0FF & !e.mask, c(e.lo0) | g, c(e.lo1) | g)
    }

    /// TLBP's comparison: the lowest index whose VPN2 equals EntryHi's under the entry's mask, with G or the ASID.
    pub fn probe(&self, entryhi: u32) -> Option<usize> {
        self.entries.iter().position(|e| e.hi & VPN2 == entryhi & VPN2 & !e.mask && (e.global() || e.hi & 0xFF == entryhi & 0xFF))
    }

    /// The entry translating `vaddr` under `asid`, the lowest index where several match (undefined, §5.2.2).
    pub fn lookup(&self, vaddr: u32, asid: u32) -> Option<Hit> {
        let index = self.entries.iter().position(|e| e.matches(vaddr, asid))?;
        let e = &self.entries[index];
        if e.scratchpad() {
            return Some(Hit { index, valid: e.lo0 & 2 != 0, dirty: e.lo0 & 4 != 0, target: Target::Scratchpad(vaddr & 0x3FFF) });
        }
        let offset = e.mask >> 1 | 0xFFF;
        let odd = vaddr & (offset + 1) != 0;
        let lo = if odd { e.lo1 } else { e.lo0 };
        let pfn = lo >> 6 & 0xF_FFFF;
        let paddr = (pfn << 12) & !offset | vaddr & offset;
        Some(Hit { index, valid: lo & 2 != 0, dirty: lo & 4 != 0, target: Target::Physical { paddr, c: lo >> 3 & 7 } })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_16_kib_pair_selects_its_odd_page_by_bit_14() {
        let mut t = Tlb::default();
        t.write(3, 0x6000, 0x0004_0000, 0x10 << 6 | 3 << 3 | 7, 0x20 << 6 | 2 << 3 | 3);
        let even = t.lookup(0x0004_1234, 0).unwrap();
        assert_eq!(even.target, Target::Physical { paddr: 0x0001_1234, c: 3 });
        let odd = t.lookup(0x0004_5678, 0).unwrap();
        assert_eq!((odd.target, odd.dirty), (Target::Physical { paddr: 0x0002_1678, c: 2 }, false));
        assert!(t.lookup(0x0004_8000, 0).is_none());
    }
}
