//! The EE core's caches as storage (EE Core User's Manual chapter 6): the 8 KiB data cache and the 16 KiB
//! instruction cache, two ways of 64-byte lines each with the tags of §6.2.3 and the LRF replacement of §6.3.1; the
//! uncached accelerated buffer (§6.5); and the BTAC as the CACHE instruction reads and writes it. What an access does
//! with them is `memory`'s; Sapphire_Native.md §17.3 has the readings.

/// A data cache tag (Figure 6-3): the physical frame (address bits 31..12) and D, V, R (LRF) and L.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DTag {
    pub pfn: u32,
    pub dirty: bool,
    pub valid: bool,
    pub lrf: bool,
    pub lock: bool,
}

/// An instruction cache tag (Figure 6-4): the frame, V and R.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ITag {
    pub pfn: u32,
    pub valid: bool,
    pub lrf: bool,
}

pub const LINE: usize = 64;
pub const DSETS: usize = 64;
pub const ISETS: usize = 128;

/// Table 6-4: the way a refill takes is the XOR of the two LRF bits, and the refill flips that way's bit.
pub fn lrf_way(r0: bool, r1: bool) -> usize {
    usize::from(r0 ^ r1)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DCache {
    pub tags: [[DTag; 2]; DSETS],
    pub data: Box<[[[u8; LINE]; 2]; DSETS]>,
}

impl Default for DCache {
    fn default() -> Self {
        DCache { tags: [[DTag::default(); 2]; DSETS], data: Box::new([[[0; LINE]; 2]; DSETS]) }
    }
}

impl DCache {
    /// The set an address indexes, virtual address bits 11..6 (Table 6-2).
    pub fn set(vaddr: u32) -> usize {
        (vaddr >> 6) as usize & (DSETS - 1)
    }

    pub fn hit(&self, set: usize, paddr: u32) -> Option<usize> {
        (0..2).find(|&w| self.tags[set][w].valid && self.tags[set][w].pfn == paddr >> 12)
    }

    /// The way a miss refills: the unlocked one if a way is locked, else the LRF way (§6.3.1, §6.3.7).
    pub fn victim(&self, set: usize) -> usize {
        let [a, b] = self.tags[set];
        if a.lock {
            1
        } else if b.lock {
            0
        } else {
            lrf_way(a.lrf, b.lrf)
        }
    }

    /// The physical address of a way's line, from its tag and the set.
    pub fn line_address(&self, set: usize, way: usize) -> u32 {
        self.tags[set][way].pfn << 12 | (set as u32) << 6
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ICache {
    pub tags: [[ITag; 2]; ISETS],
    pub data: Box<[[[u32; LINE / 4]; 2]; ISETS]>,
    /// The steering bits (3..0) and BHT (5..4) beside each instruction word, as CACHE IXLDT and IXSDT move them.
    pub side: Box<[[[u8; LINE / 4]; 2]; ISETS]>,
}

impl Default for ICache {
    fn default() -> Self {
        ICache { tags: [[ITag::default(); 2]; ISETS], data: Box::new([[[0; 16]; 2]; ISETS]), side: Box::new([[[0; 16]; 2]; ISETS]) }
    }
}

impl ICache {
    /// The set an address indexes, virtual address bits 12..6 (Table 6-2).
    pub fn set(vaddr: u32) -> usize {
        (vaddr >> 6) as usize & (ISETS - 1)
    }

    pub fn hit(&self, set: usize, paddr: u32) -> Option<usize> {
        (0..2).find(|&w| self.tags[set][w].valid && self.tags[set][w].pfn == paddr >> 12)
    }

    /// The way a miss refills: an invalid way if exactly one is, else the LRF way (§6.3.1's last sentence).
    pub fn victim(&self, set: usize) -> usize {
        let [a, b] = self.tags[set];
        match (a.valid, b.valid) {
            (false, true) => 0,
            (true, false) => 1,
            _ => lrf_way(a.lrf, b.lrf),
        }
    }
}

/// The uncached accelerated buffer: one direct-mapped line of 128 bytes tagged by physical address bits 31..7.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ucab {
    pub valid: bool,
    pub tag: u32,
    pub data: [u8; 128],
}

impl Default for Ucab {
    fn default() -> Self {
        Ucab { valid: false, tag: 0, data: [0; 128] }
    }
}

/// The BTAC as CACHE BXLBT and BXSBT see it: entries indexed by virtual address bits 5..0, TagLo's and TagHi's
/// words kept as written. Nothing predicts from it (Sapphire_Native.md §17.6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Btac {
    pub entries: [(u32, u32); 64],
}

impl Default for Btac {
    fn default() -> Self {
        Btac { entries: [(0, 0); 64] }
    }
}

impl Btac {
    pub fn flush(&mut self) {
        for e in &mut self.entries {
            e.0 &= !1;
        }
    }

    /// Invalidates the entries whose fetch address (TagLo bits 31..3) matches `vaddr` above `low` bits.
    pub fn invalidate(&mut self, vaddr: u32, low: u32) {
        let keep = !((1u32 << low) - 1);
        for e in &mut self.entries {
            if e.0 & keep == vaddr & keep {
                e.0 &= !1;
            }
        }
    }
}

/// A data tag's D, V, R and L as bits 0 to 3 of a byte, and an instruction tag's V and R as bits 0 and 1.
fn dbits(t: &DTag) -> u8 {
    u8::from(t.dirty) | u8::from(t.valid) << 1 | u8::from(t.lrf) << 2 | u8::from(t.lock) << 3
}

/// The caches' blocks of `SPH2`: every tag and line, the UCAB and the BTAC.
impl emusen_native::State for DCache {
    type Error = emusen_native::Truncated;

    fn write_state(&self, w: &mut emusen_native::StateWriter) {
        let tags = self.tags.as_flattened();
        w.u32s("Frames", &std::array::from_fn::<u32, { DSETS * 2 }, _>(|i| tags[i].pfn));
        w.bytes("Bits", &std::array::from_fn::<u8, { DSETS * 2 }, _>(|i| dbits(&tags[i])));
        w.bytes("Lines", self.data.as_flattened().as_flattened());
    }

    fn read_state(&mut self, r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        let mut frames = [0u32; DSETS * 2];
        let mut bits = [0u8; DSETS * 2];
        r.u32s(&mut frames)?;
        r.bytes(&mut bits)?;
        for (i, t) in self.tags.as_flattened_mut().iter_mut().enumerate() {
            let b = bits[i];
            *t = DTag { pfn: frames[i], dirty: b & 1 != 0, valid: b & 2 != 0, lrf: b & 4 != 0, lock: b & 8 != 0 };
        }
        r.bytes(self.data.as_flattened_mut().as_flattened_mut())
    }
}

impl emusen_native::State for ICache {
    type Error = emusen_native::Truncated;

    fn write_state(&self, w: &mut emusen_native::StateWriter) {
        let tags = self.tags.as_flattened();
        w.u32s("Frames", &std::array::from_fn::<u32, { ISETS * 2 }, _>(|i| tags[i].pfn));
        w.bytes("Bits", &std::array::from_fn::<u8, { ISETS * 2 }, _>(|i| u8::from(tags[i].valid) | u8::from(tags[i].lrf) << 1));
        w.u32s("Lines", self.data.as_flattened().as_flattened());
        w.bytes("Side", self.side.as_flattened().as_flattened());
    }

    fn read_state(&mut self, r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        let mut frames = [0u32; ISETS * 2];
        let mut bits = [0u8; ISETS * 2];
        r.u32s(&mut frames)?;
        r.bytes(&mut bits)?;
        for (i, t) in self.tags.as_flattened_mut().iter_mut().enumerate() {
            *t = ITag { pfn: frames[i], valid: bits[i] & 1 != 0, lrf: bits[i] & 2 != 0 };
        }
        r.u32s(self.data.as_flattened_mut().as_flattened_mut())?;
        r.bytes(self.side.as_flattened_mut().as_flattened_mut())
    }
}

impl emusen_native::State for Ucab {
    type Error = emusen_native::Truncated;

    fn write_state(&self, w: &mut emusen_native::StateWriter) {
        w.bool("Valid", self.valid);
        w.u32("Tag", self.tag);
        w.bytes("Line", &self.data);
    }

    fn read_state(&mut self, r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        self.valid = r.bool()?;
        self.tag = r.u32()?;
        r.bytes(&mut self.data)
    }
}

impl emusen_native::State for Btac {
    type Error = emusen_native::Truncated;

    fn write_state(&self, w: &mut emusen_native::StateWriter) {
        w.u32s("Entries", &std::array::from_fn::<u32, 128, _>(|i| if i % 2 == 0 { self.entries[i / 2].0 } else { self.entries[i / 2].1 }));
    }

    fn read_state(&mut self, r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        let mut words = [0u32; 128];
        r.u32s(&mut words)?;
        for (e, s) in self.entries.iter_mut().zip(words.chunks_exact(2)) {
            *e = (s[0], s[1]);
        }
        Ok(())
    }
}
