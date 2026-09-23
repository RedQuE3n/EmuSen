//! `CheatRomPatcher` beside the cartridge: the registry's ROM patches resolved to bytes by the host, consulted on every cartridge read (Mars_Native.md §6.6.1).

/// One patched byte: the ROM offset, the value, and the byte the cartridge must hold for it to apply, or none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RomPatch {
    pub address: u32,
    pub value: u8,
    pub compare: Option<u8>,
}

/// The patches sorted by address, each address's entries kept in the registry's order, so the first that applies wins as `TryPatchRom`'s does.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RomPatches {
    entries: Vec<RomPatch>,
    lowest: u32,
    highest: u32,
}

impl RomPatches {
    /// None when there is nothing to patch, so the cartridge's read stays one branch.
    pub fn new(mut entries: Vec<RomPatch>) -> Option<RomPatches> {
        if entries.is_empty() {
            return None;
        }
        entries.sort_by_key(|p| p.address);
        let (lowest, highest) = (entries[0].address, entries[entries.len() - 1].address);
        Some(RomPatches { entries, lowest, highest })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// `TryPatchRom` for one byte: the first entry at the address whose compare is none or the cartridge's byte.
    #[inline]
    pub fn byte(&self, address: u32, original: u8) -> u8 {
        if address < self.lowest || address > self.highest {
            return original;
        }
        let first = self.entries.partition_point(|p| p.address < address);
        self.entries[first..]
            .iter()
            .take_while(|p| p.address == address)
            .find(|p| p.compare.is_none_or(|c| c == original))
            .map_or(original, |p| p.value)
    }

    /// `Patched`: each byte of a big-endian word asked for by its own offset, so a patch lands whichever lane or transfer reaches it.
    #[inline]
    pub fn word(&self, offset: u32, word: u32) -> u32 {
        if offset.saturating_add(3) < self.lowest || offset > self.highest {
            return word;
        }
        let mut bytes = word.to_be_bytes();
        for (lane, byte) in bytes.iter_mut().enumerate() {
            *byte = self.byte(offset.wrapping_add(lane as u32), *byte);
        }
        u32::from_be_bytes(bytes)
    }
}
