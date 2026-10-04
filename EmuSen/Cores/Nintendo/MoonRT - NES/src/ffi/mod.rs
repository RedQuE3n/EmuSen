//! MoonRT's C ABI: the common native interface, and the one extension `moonrt_step`. A negative return is a status.
//! See EmuSen_NativeCores.md §3 and Moon_Native.md §8.3.

use emusen_native::abi::{self, File, FrameInfo, NativeCore, Settings, caps};

use crate::Fault;
use crate::machine::{LoadError, Machine};
use crate::memory::bus::RomPatches;
use crate::memory::cartridge::RomError;
use crate::ppu::{FRAME_BYTES, SCREEN_HEIGHT, SCREEN_WIDTH};

mod v1;

/// MoonRT's half of the interface version: 4 with the debug exports; 3 was the first on the common interface.
pub const CORE_VERSION: u16 = 4;

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

impl NativeCore for Machine {
    const CORE_VERSION: u16 = CORE_VERSION;
    const CAPABILITIES: u64 = caps::RESET | caps::MUTES | caps::ROM_PATCHES | caps::DEBUG | caps::DEBUG_STACK;
    const ENGINE: &'static str = "MoonRT";

    /// `MoonCore.LoadRom` from an image, then the battery save written into PRG RAM, clipped to it, before any frame.
    fn create(image: &[u8], settings: &Settings, files: &[File<'_>]) -> Result<Self, i32> {
        if settings.keys().next().is_some() {
            return Err(abi::status::UNKNOWN_SETTING);
        }
        let mut m = Machine::load_rom(image).map_err(|e| match e {
            LoadError::Rom(RomError::NotInes) => STATUS_NOT_INES,
            LoadError::Rom(RomError::PrgTruncated) => STATUS_PRG_TRUNCATED,
            LoadError::Rom(RomError::UnsupportedMapper(_)) => STATUS_UNSUPPORTED_BOARD,
            LoadError::Fault(f) => fault_status(f),
        })?;
        for file in files {
            if file.which != 0 {
                return Err(abi::status::BAD_FILE);
            }
            let length = m.space_size(BATTERY_SPACE).min(file.data.len());
            for (i, &b) in file.data[..length].iter().enumerate() {
                m.write_space(BATTERY_SPACE, i as i32, b);
            }
        }
        crate::take_fault();
        Ok(m)
    }

    fn reset(&mut self) -> Result<(), i32> {
        Machine::reset(self).map_err(fault_status)
    }

    fn advance(&mut self, _detail: &mut u64) -> Result<(), i32> {
        self.run_frame().map_err(fault_status)
    }

    fn set_options(&mut self, flags: u32) {
        *self.bus.ppu.skip_rendering = flags & 1 != 0;
    }

    fn frame_count(&self) -> i64 {
        self.total_frames
    }

    fn frame_info(&self) -> FrameInfo {
        FrameInfo { width: SCREEN_WIDTH as i32, height: SCREEN_HEIGHT as i32, row_repeat: 1, flags: 0, serial: self.total_frames, bytes: FRAME_BYTES as i64 }
    }

    fn frame(&self) -> &[u8] {
        &self.bus.ppu.frame_rgba
    }

    /// Bit n for C#'s `NesButton` n: A, B, Select, Start, Up, Down, Left, Right; port 0 is pad 1, any other pad 2.
    fn set_buttons(&mut self, port: u32, mask: u32, changed: u32) -> Result<(), i32> {
        let pad = if port == 0 { &mut *self.bus.controller1 } else { &mut *self.bus.controller2 };
        pad.state = (pad.state & !(changed as u8)) | (mask as u8 & changed as u8);
        Ok(())
    }

    fn audio_rate(&self) -> i32 {
        44100
    }

    fn audio_buffered(&self) -> usize {
        self.bus.apu.mixer.samples.len()
    }

    fn drain_audio(&mut self, out: &mut [i16], max_frames: usize) -> usize {
        self.bus.apu.drain(out, max_frames)
    }

    fn set_audio_limit(&mut self, samples: usize) {
        self.bus.apu.mixer.samples.set_limit(samples);
    }

    fn set_mutes(&mut self, mask: u32) -> Result<(), i32> {
        for (i, muted) in self.bus.apu.mixer.channel_muted.iter_mut().enumerate() {
            *muted = mask & (1 << i) != 0;
        }
        Ok(())
    }

    fn space_size(&self, space: u32) -> Result<i64, i32> {
        Ok(Machine::space_size(self, space) as i64)
    }

    /// Each byte as `MoonCore.ReadSpace` reads one; CPUBUS reads have their side effects.
    fn space_read(&mut self, space: u32, address: u32, out: &mut [u8]) -> Result<(), i32> {
        for (i, b) in out.iter_mut().enumerate() {
            *b = self.read_space(space, (address as i32).wrapping_add(i as i32));
        }
        Ok(())
    }

    fn space_write(&mut self, space: u32, address: u32, data: &[u8]) -> Result<(), i32> {
        for (i, &b) in data.iter().enumerate() {
            self.write_space(space, (address as i32).wrapping_add(i as i32), b);
        }
        Ok(())
    }

    /// PRG RAM when the header has a battery, as C#'s `HasBattery`; changes are not tracked, and C# writes every 300th frame.
    fn battery(&self, which: u32) -> Result<(&[u8], u32), i32> {
        if which != 0 {
            return Err(abi::status::BAD_FILE);
        }
        let cart = &self.bus.board.cart;
        Ok((if *cart.has_battery { &cart.prg_ram[..] } else { &[] }, 0))
    }

    /// Game Genie's table built from the resolved list: for each original byte, the first entry whose compare is absent or equal.
    fn set_rom_patches(&mut self, triples: &[u32]) -> Result<(), i32> {
        *self.bus.rom_patches = build_rom_patches(triples);
        Ok(())
    }

    fn debug_hooks(&mut self) -> Option<&mut emusen_native::debug::Hooks> {
        Some(&mut self.bus.hooks)
    }

    fn debug_run_frame(&mut self, flags: u32, _detail: &mut u64) -> Result<u32, i32> {
        self.run_frame_debug(flags).map_err(fault_status)
    }

    fn debug_pc(&self, processor: u32) -> Option<u64> {
        (processor == 0).then_some(self.cpu.pc as u64)
    }

    fn end_call() {
        crate::take_fault();
    }
}

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

emusen_native::native_exports!(Machine; reset, mutes, rom_patches, debug, debug_stack);

/// One instruction with the frame loop's DMA charge, NMI edge and stolen cycles: a test extension; the cycles, or a fault status.
///
/// # Safety
/// `handle` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moonrt_step(handle: *mut Machine) -> i32 {
    let Some(m) = (unsafe { handle.as_mut() }) else { return STATUS_NULL };
    crate::take_fault();
    let cycles = m.step();
    crate::take_fault().map_or(cycles, fault_status)
}

/// The byte a CPU read of `address` returns for `original` under the current patches, or -1 for none: the exhaustive patch test's view.
///
/// # Safety
/// `handle` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moonrt_rom_patch(handle: *const Machine, address: u32, original: u32) -> i32 {
    let Some(m) = (unsafe { handle.as_ref() }) else { return STATUS_NULL };
    rom_patch(m, address, original)
}

/// The patch table's answer for one read, which both interfaces' extensions give.
fn rom_patch(m: &Machine, address: u32, original: u32) -> i32 {
    let (Ok(address), Ok(original)) = (u16::try_from(address), u8::try_from(original)) else { return -1 };
    match &*m.bus.rom_patches {
        Some(map) => match map.get(&address) {
            Some(entries) if entries[original as usize] != 0 => (entries[original as usize] & 0xFF) as i32,
            _ => -1,
        },
        None => -1,
    }
}
