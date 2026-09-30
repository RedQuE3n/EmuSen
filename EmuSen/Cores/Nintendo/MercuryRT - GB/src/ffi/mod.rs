//! MercuryRT's C ABI: the common native interface, and the Game Boy's extensions `mercuryrt_*`. A negative return is a status.
//! See EmuSen_NativeCores.md §3 and Mercury_Native.md §8.7.

use std::collections::HashMap;
use std::ptr;

use emusen_native::abi::{self, File, FrameInfo, NativeCore, Settings, caps};

use crate::debug::Hooks;
use crate::machine::{Machine, Model};
use crate::memory::cartridge::RomError;

/// MercuryRT's half of the interface version: 5, the first on the common interface (1 to 4 were `mercury_interface_version`).
pub const CORE_VERSION: u16 = 5;

/// A null handle or buffer.
pub const STATUS_NULL: i32 = emusen_native::ffi::status::NULL;
/// An image shorter than the 336-byte header.
pub const STATUS_TOO_SHORT: i32 = -9;
/// A cartridge type no board here implements.
pub const STATUS_UNSUPPORTED_BOARD: i32 = -10;
/// A model number that is not Auto, Game Boy or Game Boy Color.
pub const STATUS_UNKNOWN_MODEL: i32 = -11;
/// An opcode no SM83 has; the frame's `detail` carries `(opcode << 16) | pc`.
pub const STATUS_ILLEGAL_OPCODE: i32 = -20;

/// The create-time setting that picks the console, `GbModel`'s number: 0 Auto, 1 Game Boy, 2 Game Boy Color.
pub const MODEL_KEY: &str = "Model";

/// The battery RAM's space, `CARTRAM`.
const BATTERY_SPACE: u32 = 2;

pub const SCREEN_WIDTH: i32 = 160;
pub const SCREEN_HEIGHT: i32 = 144;

impl NativeCore for Machine {
    const CORE_VERSION: u16 = CORE_VERSION;
    const CAPABILITIES: u64 = caps::MUTES | caps::ROM_PATCHES | caps::DEBUG | caps::DEBUG_STACK;
    const ENGINE: &'static str = "MercuryRT";

    /// `MercuryCore.LoadRom` on the model the settings name, then the battery save written into cartridge RAM, clipped to it.
    fn create(image: &[u8], settings: &Settings, files: &[File<'_>]) -> Result<Self, i32> {
        if let Some(key) = settings.keys().find(|k| *k != MODEL_KEY) {
            let _ = key;
            return Err(abi::status::UNKNOWN_SETTING);
        }
        let model = match settings.get(MODEL_KEY) {
            None => Model::Auto,
            Some(v) => v.parse::<u32>().ok().and_then(Model::from_u32).ok_or(STATUS_UNKNOWN_MODEL)?,
        };
        let mut m = Machine::load_rom(image.to_vec(), model).map_err(|e| match e {
            RomError::TooShort(_) => STATUS_TOO_SHORT,
            RomError::UnsupportedType(_) => STATUS_UNSUPPORTED_BOARD,
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
        Ok(m)
    }

    fn advance(&mut self, detail: &mut u64) -> Result<(), i32> {
        self.run_frame().map_err(|e| {
            *detail = ((e.opcode as u64) << 16) | e.pc as u64;
            STATUS_ILLEGAL_OPCODE
        })
    }

    fn set_options(&mut self, flags: u32) {
        *self.bus.ppu.skip_rendering = flags & 1 != 0;
    }

    fn frame_count(&self) -> i64 {
        self.total_frames
    }

    fn frame_info(&self) -> FrameInfo {
        FrameInfo { width: SCREEN_WIDTH, height: SCREEN_HEIGHT, row_repeat: 1, flags: 0, serial: self.total_frames, bytes: self.bus.ppu.frame_rgba.len() as i64 }
    }

    fn frame(&self) -> &[u8] {
        &self.bus.ppu.frame_rgba
    }

    /// Port 0 alone, bit order right, left, up, down, A, B, select, start; another port is ignored, as C#'s `SetButton` ignores it.
    fn set_buttons(&mut self, port: u32, mask: u32, changed: u32) -> Result<(), i32> {
        if port != 0 {
            return Ok(());
        }
        let j = &mut *self.bus.joypad;
        let buttons = [&mut j.right, &mut j.left, &mut j.up, &mut j.down, &mut j.a, &mut j.b, &mut j.select, &mut j.start];
        for (n, button) in buttons.into_iter().enumerate() {
            if changed & (1 << n) != 0 {
                *button = mask & (1 << n) != 0;
            }
        }
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

    /// Each byte as `MercuryCore.ReadSpace` reads one; CPUBUS reads have their side effects.
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

    /// Cartridge RAM when the header has a battery, as C#'s `HasBattery`; changes are not tracked, and C# writes every 300th frame.
    fn battery(&self, which: u32) -> Result<(&[u8], u32), i32> {
        if which != 0 {
            return Err(abi::status::BAD_FILE);
        }
        let cart = &self.bus.cart;
        Ok((if *cart.has_battery { &cart.ram[..] } else { &[] }, 0))
    }

    /// Game Genie's table from the resolved list: for each original byte, the first entry whose compare is absent or equal.
    fn set_rom_patches(&mut self, triples: &[u32]) -> Result<(), i32> {
        *self.bus.rom_patches = build_rom_patches(triples);
        Ok(())
    }

    fn debug_hooks(&mut self) -> Option<&mut Hooks> {
        Some(&mut self.hooks)
    }

    fn debug_run_frame(&mut self, flags: u32, detail: &mut u64) -> Result<u32, i32> {
        self.run_frame_debug(flags).map_err(|e| {
            *detail = ((e.opcode as u64) << 16) | e.pc as u64;
            STATUS_ILLEGAL_OPCODE
        })
    }

    fn debug_pc(&self, processor: u32) -> Option<u64> {
        (processor == 0).then_some(self.cpu.pc as u64)
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

emusen_native::native_exports!(Machine; mutes, rom_patches, debug, debug_stack);

/// C#'s `Apu.SetSampleRate`, the division and `Math.Pow` evaluated by C# so the coefficients are its to the bit (Mercury_Native.md §3.3).
///
/// # Safety
/// `handle` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercuryrt_set_sample_rate(handle: *mut Machine, cycles_per_sample: f64, charge_factor: f64) -> i32 {
    let Some(m) = (unsafe { handle.as_mut() }) else { return STATUS_NULL };
    m.bus.apu.set_sample_rate(cycles_per_sample, charge_factor);
    0
}

/// The bytes the ROM has clocked out of the link port, oldest first, copied up to `len`; their count.
///
/// # Safety
/// `handle` must be live or null; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercuryrt_serial(handle: *const Machine, out: *mut u8, len: usize) -> i64 {
    let Some(m) = (unsafe { handle.as_ref() }) else { return STATUS_NULL as i64 };
    let log = &m.bus.serial_log;
    if !out.is_null() {
        unsafe { ptr::copy_nonoverlapping(log.as_ptr(), out, log.len().min(len)) };
    }
    log.len() as i64
}

/// 1 while the machine is a Game Boy Color, 0 for a Game Boy; a state from the other console changes it (Mercury_Model.md §5).
///
/// # Safety
/// `handle` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercuryrt_cgb_hardware(handle: *const Machine) -> i32 {
    unsafe { handle.as_ref() }.map_or(STATUS_NULL, |m| m.cgb_hardware() as i32)
}

/// One instruction or interrupt dispatch, with the frame loop's interrupt acknowledge and DMA stall: a test extension.
///
/// # Safety
/// `handle` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercuryrt_step(handle: *mut Machine) -> i32 {
    let Some(m) = (unsafe { handle.as_mut() }) else { return STATUS_NULL };
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

/// The byte a CPU read of `address` returns for `original` under the current patches, or -1 for none: the exhaustive patch test's view.
///
/// # Safety
/// `handle` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercuryrt_rom_patch(handle: *const Machine, address: u32, original: u32) -> i32 {
    let Some(m) = (unsafe { handle.as_ref() }) else { return STATUS_NULL };
    let (Ok(address), Ok(original)) = (u16::try_from(address), u8::try_from(original)) else { return -1 };
    match &*m.bus.rom_patches {
        Some(map) => match map.get(&address) {
            Some(entries) if entries[original as usize] != 0 => (entries[original as usize] & 0xFF) as i32,
            _ => -1,
        },
        None => -1,
    }
}

/// The cartridge RAM's length as the header gives it, battery or not; the C# `Cartridge.Ram.Length`, which a test pins.
///
/// # Safety
/// `handle` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercuryrt_ram_length(handle: *const Machine) -> i64 {
    unsafe { handle.as_ref() }.map_or(STATUS_NULL as i64, |m| m.bus.cart.ram.len() as i64)
}
