//! MercuryRT's C ABI. A negative return is a status. See Mercury_Native.md §2.3.

pub mod debug;

use std::ptr;

use crate::machine::Machine;
use crate::memory::cartridge::RomError;

/// A null handle or buffer.
pub const STATUS_NULL: i32 = -1;
/// An image shorter than the 336-byte header.
pub const STATUS_TOO_SHORT: i32 = -9;
/// A cartridge type no board here implements.
pub const STATUS_UNSUPPORTED_BOARD: i32 = -10;

unsafe fn input<'a>(data: *const u8, len: usize) -> &'a [u8] {
    if data.is_null() || len == 0 { &[] } else { unsafe { std::slice::from_raw_parts(data, len) } }
}

fn status(result: Result<usize, crate::state::StateError>) -> i64 {
    match result {
        Ok(n) => n as i64,
        Err(e) => e.status() as i64,
    }
}

/// `MercuryCore.LoadRom` from an image; `save_path_len` negative is C#'s null. Null on refusal, with the reason in `status`.
///
/// # Safety
/// `rom` valid for `len` bytes; `save_path` valid for `save_path_len` bytes when that is not negative; `status` writable or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_new(rom: *const u8, len: usize, save_path: *const u8, save_path_len: isize, status: *mut i32) -> *mut Machine {
    let image = unsafe { input(rom, len) }.to_vec();
    let path = (save_path_len >= 0).then(|| String::from_utf8_lossy(unsafe { input(save_path, save_path_len as usize) }).into_owned());
    let (machine, code) = match Machine::load_rom(image, path) {
        Ok(m) => (Box::into_raw(Box::new(m)), 0),
        Err(RomError::TooShort(_)) => (ptr::null_mut(), STATUS_TOO_SHORT),
        Err(RomError::UnsupportedType(_)) => (ptr::null_mut(), STATUS_UNSUPPORTED_BOARD),
    };
    if let Some(s) = unsafe { status.as_mut() } {
        *s = code;
    }
    machine
}

/// # Safety
/// `machine` must come from `mercury_machine_new` and not be used again, or be null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_free(machine: *mut Machine) {
    if !machine.is_null() {
        drop(unsafe { Box::from_raw(machine) });
    }
}

/// The state's fields; zero, or a negative status, and a failed load changes nothing.
///
/// # Safety
/// `machine` must be live or null; `data` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_load_state(machine: *mut Machine, data: *const u8, len: usize) -> i32 {
    let Some(m) = (unsafe { machine.as_mut() }) else { return STATUS_NULL };
    match m.load_state(unsafe { input(data, len) }) {
        Ok(()) => 0,
        Err(e) => e.status(),
    }
}

/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_save_state_size(machine: *const Machine) -> i64 {
    unsafe { machine.as_ref() }.map_or(STATUS_NULL as i64, |m| m.state_size() as i64)
}

/// The bytes written, or a negative status.
///
/// # Safety
/// `machine` must be live or null; `out` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_save_state(machine: *const Machine, out: *mut u8, len: usize) -> i64 {
    let Some(m) = (unsafe { machine.as_ref() }) else { return STATUS_NULL as i64 };
    if out.is_null() {
        return STATUS_NULL as i64;
    }
    status(m.save_state(unsafe { std::slice::from_raw_parts_mut(out, len) }))
}

/// An opcode no SM83 has; the run's `detail` carries it and its address.
pub const STATUS_ILLEGAL_OPCODE: i32 = -20;

/// `MercuryCore.RunFrame`'s machine: zero, or `STATUS_ILLEGAL_OPCODE` with `(opcode << 16) | pc` in `detail`.
///
/// # Safety
/// `machine` must be live or null; `detail` writable or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_run_frame(machine: *mut Machine, detail: *mut u32) -> i32 {
    let Some(m) = (unsafe { machine.as_mut() }) else { return STATUS_NULL };
    match m.run_frame() {
        Ok(()) => 0,
        Err(e) => {
            if let Some(d) = unsafe { detail.as_mut() } {
                *d = ((e.opcode as u32) << 16) | e.pc as u32;
            }
            STATUS_ILLEGAL_OPCODE
        }
    }
}

/// The eight buttons as a mask, bit order right, left, up, down, A, B, select, start.
///
/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_set_buttons(machine: *mut Machine, mask: u32) {
    let Some(m) = (unsafe { machine.as_mut() }) else { return };
    let j = &mut *m.bus.joypad;
    let bit = |n: u32| mask & (1 << n) != 0;
    (j.right, j.left, j.up, j.down, j.a, j.b, j.select, j.start) = (bit(0), bit(1), bit(2), bit(3), bit(4), bit(5), bit(6), bit(7));
}

/// Bit 0 skips the line renderer, as C#'s `SkipRendering`.
///
/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_set_options(machine: *mut Machine, flags: u32) {
    let Some(m) = (unsafe { machine.as_mut() }) else { return };
    *m.bus.ppu.skip_rendering = flags & 1 != 0;
}

/// The mixer's rate, as C#'s `Apu.SetSampleRate` computed it, `Math.Pow` included.
///
/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_set_sample_rate(machine: *mut Machine, cycles_per_sample: f64, charge_factor: f64) {
    if let Some(m) = unsafe { machine.as_mut() } {
        m.bus.apu.set_sample_rate(cycles_per_sample, charge_factor);
    }
}

/// Channel mutes, bit n for channel n.
///
/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_set_mutes(machine: *mut Machine, mask: u32) {
    if let Some(m) = unsafe { machine.as_mut() } {
        for (i, muted) in m.bus.apu.mixer.channel_muted.iter_mut().enumerate() {
            *muted = mask & (1 << i) != 0;
        }
    }
}

/// The 160x144 RGBA picture, copied; returns its length.
///
/// # Safety
/// `machine` must be live or null; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_frame(machine: *const Machine, out: *mut u8, len: usize) -> i64 {
    let Some(m) = (unsafe { machine.as_ref() }) else { return STATUS_NULL as i64 };
    let frame = &m.bus.ppu.frame_rgba;
    if !out.is_null() {
        unsafe { ptr::copy_nonoverlapping(frame.as_ptr(), out, frame.len().min(len)) };
    }
    frame.len() as i64
}

/// Samples buffered, counted as C#'s `BufferedSamples` counts them: two a stereo frame.
///
/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_audio_buffered(machine: *const Machine) -> i64 {
    unsafe { machine.as_ref() }.map_or(STATUS_NULL as i64, |m| m.bus.apu.mixer.buffer.len() as i64)
}

/// `Apu.Drain`: interleaved stereo into `out`; returns the samples written.
///
/// # Safety
/// `machine` must be live or null; `out` valid for `len` samples.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_drain_audio(machine: *mut Machine, out: *mut i16, len: usize, max_frames: i64) -> i64 {
    let Some(m) = (unsafe { machine.as_mut() }) else { return STATUS_NULL as i64 };
    if out.is_null() || max_frames <= 0 {
        return 0;
    }
    let out = unsafe { std::slice::from_raw_parts_mut(out, len) };
    m.bus.apu.drain(out, max_frames as usize) as i64
}

/// The bytes the ROM has clocked out of the link port, oldest first; returns their count.
///
/// # Safety
/// `machine` must be live or null; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_serial(machine: *const Machine, out: *mut u8, len: usize) -> i64 {
    let Some(m) = (unsafe { machine.as_ref() }) else { return STATUS_NULL as i64 };
    let log = &m.bus.serial_log;
    if !out.is_null() {
        unsafe { ptr::copy_nonoverlapping(log.as_ptr(), out, log.len().min(len)) };
    }
    log.len() as i64
}

/// One instruction or interrupt dispatch, with the frame loop's interrupt acknowledge and DMA stall: a test ABI.
///
/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_step(machine: *mut Machine) -> i32 {
    let Some(m) = (unsafe { machine.as_mut() }) else { return STATUS_NULL };
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

/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_total_frames(machine: *const Machine) -> i64 {
    unsafe { machine.as_ref() }.map_or(STATUS_NULL as i64, |m| m.total_frames)
}

/// # Safety
/// `machine` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_space_size(machine: *const Machine, space: u32) -> i64 {
    unsafe { machine.as_ref() }.map_or(STATUS_NULL as i64, |m| m.space_size(space) as i64)
}

/// `len` bytes of a space from `address` on, each read as `MercuryCore.ReadSpace` reads one; CPUBUS reads have their side effects.
///
/// # Safety
/// `machine` must be live or null; `out` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_read_space(machine: *mut Machine, space: u32, address: i32, out: *mut u8, len: usize) -> i64 {
    let Some(m) = (unsafe { machine.as_mut() }) else { return STATUS_NULL as i64 };
    if out.is_null() {
        return STATUS_NULL as i64;
    }
    let out = unsafe { std::slice::from_raw_parts_mut(out, len) };
    for (i, b) in out.iter_mut().enumerate() {
        *b = m.read_space(space, address.wrapping_add(i as i32));
    }
    len as i64
}

/// # Safety
/// `machine` must be live or null; `data` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_write_space(machine: *mut Machine, space: u32, address: i32, data: *const u8, len: usize) -> i64 {
    let Some(m) = (unsafe { machine.as_mut() }) else { return STATUS_NULL as i64 };
    for (i, &b) in unsafe { input(data, len) }.iter().enumerate() {
        m.write_space(space, address.wrapping_add(i as i32), b);
    }
    len as i64
}

/// The save path the state carries as UTF-8, copied up to `len`; its length, or -2 for C#'s null.
///
/// # Safety
/// `machine` must be live or null; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_save_path(machine: *const Machine, out: *mut u8, len: usize) -> i64 {
    let Some(m) = (unsafe { machine.as_ref() }) else { return STATUS_NULL as i64 };
    let Some(path) = &m.bus.cart.save_path else { return -2 };
    if !out.is_null() {
        unsafe { ptr::copy_nonoverlapping(path.as_ptr(), out, path.len().min(len)) };
    }
    path.len() as i64
}

/// Game Genie's table: `count` addresses, and 256 entries each of `0x100 | patched` or 0; a count of zero clears it.
///
/// # Safety
/// `machine` must be live or null; `addresses` valid for `count` and `tables` for `count * 256` entries.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_set_rom_patches(machine: *mut Machine, addresses: *const u16, tables: *const u16, count: usize) {
    let Some(m) = (unsafe { machine.as_mut() }) else { return };
    if count == 0 || addresses.is_null() || tables.is_null() {
        *m.bus.rom_patches = None;
        return;
    }
    let addresses = unsafe { std::slice::from_raw_parts(addresses, count) };
    let tables = unsafe { std::slice::from_raw_parts(tables, count * 256) };
    let map = addresses.iter().zip(tables.chunks_exact(256)).map(|(&a, t)| (a, t.try_into().expect("256 entries"))).collect();
    *m.bus.rom_patches = Some(Box::new(map));
}

/// The state's layout as UTF-8 text, copied up to `len` bytes; returns its whole length, or a negative status.
///
/// # Safety
/// `machine` must be live or null; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_machine_state_layout(machine: *const Machine, out: *mut u8, len: usize) -> i64 {
    let Some(m) = (unsafe { machine.as_ref() }) else { return STATUS_NULL as i64 };
    let text = m.layout();
    if !out.is_null() {
        let n = text.len().min(len);
        unsafe { ptr::copy_nonoverlapping(text.as_ptr(), out, n) };
    }
    text.len() as i64
}
