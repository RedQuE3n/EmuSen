//! MoonRT's C ABI. A negative return is a status. See Moon_Native.md §2.3.

use std::ptr;

use crate::Fault;
use emusen_native::ffi::input;

use crate::machine::{LoadError, Machine};
use crate::memory::cartridge::RomError;

/// A null handle or buffer.
pub const STATUS_NULL: i32 = emusen_native::ffi::status::NULL;
/// No "NES\x1A" magic.
pub const STATUS_NOT_INES: i32 = -9;
/// A mapper number no board here implements.
pub const STATUS_UNSUPPORTED_BOARD: i32 = -10;
/// A header claiming more PRG than the file holds.
pub const STATUS_PRG_TRUNCATED: i32 = -11;
/// A C# exception the machine would have thrown: -30 less the `Fault`'s number.
pub const STATUS_FAULT_BASE: i32 = -30;

fn fault_status(fault: Fault) -> i32 {
    STATUS_FAULT_BASE - fault as i32
}

/// `MoonCore.LoadRom` from an image. Null on refusal, with the reason in `status`.
///
/// # Safety
/// `image` valid for `len` bytes; `status` writable or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moon_machine_new(image: *const u8, len: usize, status: *mut i32) -> *mut Machine {
    let (machine, code) = match Machine::load_rom(unsafe { input(image, len) }) {
        Ok(m) => (Box::into_raw(Box::new(m)), 0),
        Err(LoadError::Rom(RomError::NotInes)) => (ptr::null_mut(), STATUS_NOT_INES),
        Err(LoadError::Rom(RomError::PrgTruncated)) => (ptr::null_mut(), STATUS_PRG_TRUNCATED),
        Err(LoadError::Rom(RomError::UnsupportedMapper(_))) => (ptr::null_mut(), STATUS_UNSUPPORTED_BOARD),
        Err(LoadError::Fault(f)) => (ptr::null_mut(), fault_status(f)),
    };
    if let Some(s) = unsafe { status.as_mut() } {
        *s = code;
    }
    machine
}

/// # Safety
/// `machine` must come from `moon_machine_new` and not be used again, or be null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moon_machine_free(machine: *mut Machine) {
    if !machine.is_null() {
        drop(unsafe { Box::from_raw(machine) });
    }
}

emusen_native::state_exports!(Machine, moon_machine_load_state, moon_machine_save_state_size, moon_machine_save_state, moon_machine_state_layout);





/// `MoonCore.RunFrame`'s machine: zero, or a fault status.
///
/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moon_machine_run_frame(machine: *mut Machine) -> i32 {
    let Some(m) = (unsafe { machine.as_mut() }) else { return STATUS_NULL };
    match m.run_frame() {
        Ok(()) => 0,
        Err(f) => fault_status(f),
    }
}

/// `MoonCore.Reset`, the RESET button.
///
/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moon_machine_reset(machine: *mut Machine) -> i32 {
    let Some(m) = (unsafe { machine.as_mut() }) else { return STATUS_NULL };
    match m.reset() {
        Ok(()) => 0,
        Err(f) => fault_status(f),
    }
}

/// One instruction with the frame loop's DMA charge, NMI edge and stolen cycles: a test ABI; the cycles, or a fault status.
///
/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moon_machine_step(machine: *mut Machine) -> i32 {
    let Some(m) = (unsafe { machine.as_mut() }) else { return STATUS_NULL };
    crate::take_fault();
    let cycles = m.step();
    crate::take_fault().map_or(cycles, fault_status)
}

/// A pad's eight buttons, bit n for C#'s `NesButton` n: A, B, Select, Start, Up, Down, Left, Right.
///
/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moon_machine_set_buttons(machine: *mut Machine, port: i32, mask: u32) {
    let Some(m) = (unsafe { machine.as_mut() }) else { return };
    let pad = if port == 0 { &mut *m.bus.controller1 } else { &mut *m.bus.controller2 };
    pad.state = mask as u8;
}

/// Bit 0 skips the pixel writes, as C#'s `SkipRendering`.
///
/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moon_machine_set_options(machine: *mut Machine, flags: u32) {
    if let Some(m) = unsafe { machine.as_mut() } {
        *m.bus.ppu.skip_rendering = flags & 1 != 0;
    }
}

/// Channel mutes, bit n for channel n.
///
/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moon_machine_set_mutes(machine: *mut Machine, mask: u32) {
    if let Some(m) = unsafe { machine.as_mut() } {
        for (i, muted) in m.bus.apu.mixer.channel_muted.iter_mut().enumerate() {
            *muted = mask & (1 << i) != 0;
        }
    }
}

/// The 256x240 RGBA picture, copied; returns its length.
///
/// # Safety
/// `machine` must be live or null; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moon_machine_frame(machine: *const Machine, out: *mut u8, len: usize) -> i64 {
    let Some(m) = (unsafe { machine.as_ref() }) else { return STATUS_NULL as i64 };
    let frame = &m.bus.ppu.frame_rgba;
    if !out.is_null() {
        unsafe { ptr::copy_nonoverlapping(frame.as_ptr(), out, frame.len().min(len)) };
    }
    frame.len() as i64
}

/// Samples buffered, two a stereo frame.
///
/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moon_machine_audio_buffered(machine: *const Machine) -> i64 {
    unsafe { machine.as_ref() }.map_or(STATUS_NULL as i64, |m| m.bus.apu.mixer.buffer.len() as i64)
}

/// `Apu.Drain`: interleaved stereo into `out`; returns the samples written.
///
/// # Safety
/// `machine` must be live or null; `out` valid for `len` samples.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moon_machine_drain_audio(machine: *mut Machine, out: *mut i16, len: usize, max_frames: i64) -> i64 {
    let Some(m) = (unsafe { machine.as_mut() }) else { return STATUS_NULL as i64 };
    if out.is_null() || max_frames <= 0 {
        return 0;
    }
    let out = unsafe { std::slice::from_raw_parts_mut(out, len) };
    m.bus.apu.drain(out, max_frames as usize) as i64
}

/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moon_machine_total_frames(machine: *const Machine) -> i64 {
    unsafe { machine.as_ref() }.map_or(STATUS_NULL as i64, |m| m.total_frames)
}

/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moon_machine_space_size(machine: *const Machine, space: u32) -> i64 {
    unsafe { machine.as_ref() }.map_or(STATUS_NULL as i64, |m| m.space_size(space) as i64)
}

/// `len` bytes of a space from `address` on, each read as `MoonCore.ReadSpace` reads one; CPUBUS reads have their side effects.
///
/// # Safety
/// `machine` must be live or null; `out` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moon_machine_read_space(machine: *mut Machine, space: u32, address: i32, out: *mut u8, len: usize) -> i64 {
    let Some(m) = (unsafe { machine.as_mut() }) else { return STATUS_NULL as i64 };
    if out.is_null() {
        return STATUS_NULL as i64;
    }
    let out = unsafe { std::slice::from_raw_parts_mut(out, len) };
    for (i, b) in out.iter_mut().enumerate() {
        *b = m.read_space(space, address.wrapping_add(i as i32));
    }
    crate::take_fault();
    len as i64
}

/// # Safety
/// `machine` must be live or null; `data` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moon_machine_write_space(machine: *mut Machine, space: u32, address: i32, data: *const u8, len: usize) -> i64 {
    let Some(m) = (unsafe { machine.as_mut() }) else { return STATUS_NULL as i64 };
    for (i, &b) in unsafe { input(data, len) }.iter().enumerate() {
        m.write_space(space, address.wrapping_add(i as i32), b);
    }
    crate::take_fault();
    len as i64
}

/// Game Genie's table: `count` CPU addresses, and 256 entries each of `0x100 | patched` or 0; a count of zero clears it.
///
/// # Safety
/// `machine` must be live or null; `addresses` valid for `count` and `tables` for `count * 256` entries.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moon_machine_set_rom_patches(machine: *mut Machine, addresses: *const u16, tables: *const u16, count: usize) {
    let Some(m) = (unsafe { machine.as_mut() }) else { return };
    if count == 0 || addresses.is_null() || tables.is_null() {
        *m.bus.rom_patches = None;
        return;
    }
    let addresses = unsafe { std::slice::from_raw_parts(addresses, count) };
    let tables = unsafe { std::slice::from_raw_parts(tables, count * 256) };
    let map = addresses.iter().zip(tables.chunks_exact(256)).map(|(&a, t)| (a, Box::new(<[u16; 256]>::try_from(t).expect("256 entries")))).collect();
    *m.bus.rom_patches = Some(Box::new(map));
}
