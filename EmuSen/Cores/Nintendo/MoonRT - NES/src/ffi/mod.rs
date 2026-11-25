//! What MoonRT's exports on the core ABI v1 share: the status codes, the battery's space and the patch table. A negative
//! return is a status. See EmuSen_CoreAPI.md §26 and EmuSen_Backend.md §23.

use emusen_native::abi;

use crate::Fault;
use crate::memory::bus::RomPatches;

mod units;
mod v1;

/// A null handle or buffer.
pub const STATUS_NULL: i32 = emusen_native::ffi::status::NULL;
/// No "NES\x1A" magic.
pub const STATUS_NOT_INES: i32 = -9;
/// A mapper number no board here implements.
pub const STATUS_UNSUPPORTED_BOARD: i32 = -10;
/// A header claiming more PRG than the file holds.
pub const STATUS_PRG_TRUNCATED: i32 = -11;

/// The C# exception a fault reproduces, in the interface's band: -320 less its kind.
pub fn fault_status(fault: Fault) -> i32 {
    abi::fault::status(fault as i32)
}

/// The battery RAM's space, `PRGRAM`.
pub(crate) const BATTERY_SPACE: u32 = 2;

/// `CheatRegistry.TryPatchRom`'s answer for every CPU address and original byte, from `ResolveRomPatches`' order.
pub fn build_rom_patches(triples: &[u32]) -> Option<Box<RomPatches>> {
    let mut map = RomPatches::new();
    for t in triples.chunks_exact(3) {
        let (address, value, compare) = (t[0], t[1], t[2]);
        let Ok(address) = u16::try_from(address) else { continue };
        let entry = 0x100 | (value as u16 & 0xFF);
        let table = map.entry(address).or_insert_with(|| Box::new([0u16; 256]));
        if compare == u32::MAX {
            for slot in table.iter_mut().filter(|s| **s == 0) {
                *slot = entry;
            }
        } else if let Some(slot) = table.get_mut(compare as usize) {
            if *slot == 0 {
                *slot = entry;
            }
        }
    }
    if map.is_empty() { None } else { Some(Box::new(map)) }
}
