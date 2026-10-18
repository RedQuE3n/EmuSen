//! MercuryRT's C ABI: the core ABI v1 (`v1`) and the Game Boy's test extensions `mercuryrt_core_*` beside it. A negative
//! return is a status. See EmuSen_CoreAPI.md, Mercury_Native.md §8.7 and EmuSen_Backend.md §17.

use std::collections::HashMap;
use std::ptr;

use crate::machine::Machine;

mod v1;

/// A null handle or buffer.
pub const STATUS_NULL: i32 = emusen_native::ffi::status::NULL;
/// An image shorter than the 336-byte header.
pub const STATUS_TOO_SHORT: i32 = -9;
/// A cartridge type no board here implements.
pub const STATUS_UNSUPPORTED_BOARD: i32 = -10;
/// A Model setting that is not Auto, Game Boy or Game Boy Color.
pub const STATUS_UNKNOWN_MODEL: i32 = -11;
/// An opcode no SM83 has; the frame's `detail` carries `(opcode << 16) | pc`.
pub const STATUS_ILLEGAL_OPCODE: i32 = -20;

/// The create-time setting that picks the console: Auto, Game Boy or Game Boy Color.
pub const MODEL_KEY: &str = "Model";

/// The battery RAM's space, `CARTRAM`.
pub(crate) const BATTERY_SPACE: u32 = 2;

pub const SCREEN_WIDTH: i32 = 160;
pub const SCREEN_HEIGHT: i32 = 144;

impl Machine {
    /// Port 0 alone, bit order right, left, up, down, A, B, select, start; another port is ignored, as C#'s `SetButton` ignores it.
    pub(crate) fn press(&mut self, port: u32, mask: u32, changed: u32) -> Result<(), i32> {
        if port != 0 {
            return Ok(());
        }
        let before = self.bus.read_joypad();
        let j = &mut *self.bus.joypad;
        let buttons = [&mut j.right, &mut j.left, &mut j.up, &mut j.down, &mut j.a, &mut j.b, &mut j.select, &mut j.start];
        for (n, button) in buttons.into_iter().enumerate() {
            if changed & (1 << n) != 0 {
                *button = mask & (1 << n) != 0;
            }
        }
        self.bus.request_joypad_on_fall(before);
        Ok(())
    }
}

/// `CheatRegistry.TryPatchRom`'s answer for every ROM address and original byte, from `ResolveRomPatches`' order.
pub fn build_rom_patches(triples: &[u32]) -> Option<Box<HashMap<u16, [u16; 256]>>> {
    let mut map: HashMap<u16, [u16; 256]> = HashMap::new();
    for t in triples.chunks_exact(3) {
        let (address, value, compare) = (t[0], t[1], t[2]);
        let Ok(address) = u16::try_from(address) else { continue };
        let entry = 0x100 | (value as u16 & 0xFF);
        let table = map.entry(address).or_insert([0u16; 256]);
        if compare == u32::MAX {
            for slot in table.iter_mut().filter(|s| **s == 0) {
                *slot = entry;
            }
        } else if let Some(slot) = table.get_mut(compare as usize)
            && *slot == 0
        {
            *slot = entry;
        }
    }
    if map.is_empty() { None } else { Some(Box::new(map)) }
}

/// The serial log copied up to `len`.
///
/// # Safety
/// `out` valid for `len` bytes, or null.
pub(crate) unsafe fn serial(m: &Machine, out: *mut u8, len: usize) -> i64 {
    let log = &m.bus.serial_log;
    if !out.is_null() {
        unsafe { ptr::copy_nonoverlapping(log.as_ptr(), out, log.len().min(len)) };
    }
    log.len() as i64
}

/// One instruction with the frame loop's acknowledge and stall: the T-cycles, or the illegal-opcode status.
pub(crate) fn step(m: &mut Machine) -> i32 {
    let (ie, iflags) = (m.bus.interrupt_enable, m.bus.interrupt_flags);
    match m.cpu.step(&mut m.bus, ie, iflags) {
        Ok((cycles, serviced)) => {
            if serviced >= 0 {
                m.bus.interrupt_flags &= !(1u8 << serviced);
            }
            let stall = m.bus.take_pending_stall();
            if stall > 0 {
                m.bus.tick(stall);
            }
            cycles + stall
        }
        Err(_) => STATUS_ILLEGAL_OPCODE,
    }
}

/// The patch table's answer for one read, or -1 for none.
pub(crate) fn rom_patch(m: &Machine, address: u32, original: u32) -> i32 {
    let (Ok(address), Ok(original)) = (u16::try_from(address), u8::try_from(original)) else { return -1 };
    // A scan, not `get`: a second call site of the map's `get` stops it being inlined into the plain bus read (Mercury_Native.md §8.7).
    let found = m.bus.rom_patches.as_ref().and_then(|map| map.iter().find(|(a, _)| **a == address).map(|(_, t)| t[original as usize]));
    match found {
        Some(entry) if entry != 0 => (entry & 0xFF) as i32,
        _ => -1,
    }
}
