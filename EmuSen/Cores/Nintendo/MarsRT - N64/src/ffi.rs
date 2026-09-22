//! MarsRT's C ABI: the machine, its state, and a frame at a time. A negative return is a status. See Mars_Native.md §5.1 and §5.2.

use std::ptr;
use std::sync::Arc;

use crate::machine::Machine;
use crate::rom::RomImage;
use crate::state::{State, StateResult, StateWriter};
use crate::vi_scan::{self, Scanout};

/// A null handle or buffer.
pub const STATUS_NULL: i32 = -1;
/// An image that is not a Nintendo 64 cartridge.
pub const STATUS_NOT_A_ROM: i32 = -9;

/// The machine and its host: the VI's scan-out, one for the machine's life as C#'s raster is, and whether a frame is scanned at all.
pub struct Core {
    pub machine: Machine,
    pub scanout: Scanout,
    /// Advanced wherever the shown frame is replaced, as C#'s `_frameSerial`.
    pub frame_serial: i64,
    pub skip_rendering: bool,
    pub shown: bool,
}

impl Core {
    fn new(machine: Machine) -> Core {
        Core { machine, scanout: Scanout::default(), frame_serial: 0, skip_rendering: false, shown: false }
    }

    /// `Present`: the VI's scan of what the machine left, which writes the VI's held lines; the frame is composed on every call.
    pub fn present(&mut self) {
        let bus = &mut self.machine.bus;
        self.shown = vi_scan::scan(&mut bus.vi, &bus.rdram, &bus.rdram_hidden, &mut self.scanout);
        self.frame_serial += 1;
    }

    pub fn run_frame(&mut self) {
        self.machine.run_frame();
        if !self.skip_rendering {
            self.present();
        }
    }
}

fn status(result: StateResult<usize>) -> i64 {
    match result {
        Ok(n) => n as i64,
        Err(e) => e.status() as i64,
    }
}

/// # Safety
/// `data` must be valid for `len` bytes, or null with `len` zero.
unsafe fn input<'a>(data: *const u8, len: usize) -> &'a [u8] {
    if data.is_null() { &[] } else { unsafe { std::slice::from_raw_parts(data, len) } }
}

/// # Safety
/// `data` must be valid for `len` bytes, or null.
unsafe fn file(data: *const u8, len: usize) -> Option<Vec<u8>> {
    if data.is_null() { None } else { Some(unsafe { input(data, len) }.to_vec()) }
}

/// A machine with 4 MB or 8 MB of RDRAM, or null for any other size.
#[unsafe(no_mangle)]
pub extern "C" fn mars_machine_new(rdram_bytes: u32) -> *mut Core {
    Machine::new(rdram_bytes as usize).map_or(ptr::null_mut(), |m| Box::into_raw(Box::new(Core::new(m))))
}

/// # Safety
/// `core` must come from this library and not be used again, or be null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_free(core: *mut Core) {
    if !core.is_null() {
        drop(unsafe { Box::from_raw(core) });
    }
}

/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_rdram_bytes(core: *const Core) -> u32 {
    unsafe { core.as_ref() }.map_or(0, |c| c.machine.rdram_bytes() as u32)
}

/// The state's fields alone, nothing derived; zero, or a negative status, and a failed load changes nothing.
///
/// # Safety
/// `core` must be live or null; `data` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_load_state(core: *mut Core, data: *const u8, len: usize) -> i32 {
    let Some(c) = (unsafe { core.as_mut() }) else { return STATUS_NULL };
    match c.machine.load_state(unsafe { input(data, len) }) {
        Ok(()) => 0,
        Err(e) => e.status(),
    }
}

/// `MarsCore.LoadState`: the fields, then everything a C# load derives from them, then the picture.
///
/// # Safety
/// `core` must be live or null; `data` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_restore_state(core: *mut Core, data: *const u8, len: usize) -> i32 {
    let Some(c) = (unsafe { core.as_mut() }) else { return STATUS_NULL };
    match c.machine.restore_state(unsafe { input(data, len) }) {
        Ok(()) => {
            if !c.skip_rendering {
                c.present();
            }
            0
        }
        Err(e) => e.status(),
    }
}

/// The version of the state last loaded: 1 a state, 2 a snapshot, 0 none.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_state_kind(core: *const Core) -> i32 {
    unsafe { core.as_ref() }.map_or(0, |c| c.machine.loaded_version)
}

/// The bytes a save of this kind takes, or a negative status. `snapshot` nonzero asks for a snapshot.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_save_state_size(core: *const Core, snapshot: u32) -> i64 {
    let Some(c) = (unsafe { core.as_ref() }) else { return STATUS_NULL as i64 };
    status(c.machine.state_size(snapshot != 0))
}

/// The bytes written, or a negative status; the clocks are settled first, as a C# save settles them.
///
/// # Safety
/// `core` must be live or null; `out` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_save_state(core: *mut Core, out: *mut u8, len: usize, snapshot: u32) -> i64 {
    let Some(c) = (unsafe { core.as_mut() }) else { return STATUS_NULL as i64 };
    if out.is_null() {
        return STATUS_NULL as i64;
    }
    c.machine.settle();
    status(c.machine.save_state(unsafe { std::slice::from_raw_parts_mut(out, len) }, snapshot != 0))
}

/// The state's layout as UTF-8 text, copied up to `len` bytes; returns its whole length, or a negative status.
///
/// # Safety
/// `core` must be live or null; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_state_layout(core: *const Core, snapshot: u32, out: *mut u8, len: usize) -> i64 {
    let Some(c) = (unsafe { core.as_ref() }) else { return STATUS_NULL as i64 };
    status(c.machine.layout(snapshot != 0).map(|text| {
        if !out.is_null() {
            let n = text.len().min(len);
            unsafe { ptr::copy_nonoverlapping(text.as_ptr(), out, n) };
        }
        text.len()
    }))
}

/// A machine booted from a cartridge image, as `MarsCore.LoadRom` builds it; the save and pak files as the host read them, or null.
///
/// # Safety
/// `rom` valid for `rom_len` bytes; `saved` and `pak` valid for their lengths, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_load_rom(
    rom: *const u8,
    rom_len: usize,
    expansion_pak: u32,
    saved: *const u8,
    saved_len: usize,
    pak: *const u8,
    pak_len: usize,
) -> *mut Core {
    let Ok(image) = RomImage::from_image(unsafe { input(rom, rom_len) }) else { return ptr::null_mut() };
    let (saved, pak) = unsafe { (file(saved, saved_len), file(pak, pak_len)) };
    let machine = Machine::load_rom(Arc::new(image), expansion_pak != 0, saved, pak);
    Box::into_raw(Box::new(Core::new(machine)))
}

/// A machine as the corpus boots one: `Boot.HandOff` and nothing loaded beside it.
///
/// # Safety
/// `rom` valid for `rom_len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_boot(rom: *const u8, rom_len: usize, expansion_pak: u32) -> *mut Core {
    let Ok(image) = RomImage::from_image(unsafe { input(rom, rom_len) }) else { return ptr::null_mut() };
    Box::into_raw(Box::new(Core::new(Machine::boot(Arc::new(image), expansion_pak != 0))))
}

/// One frame, and its picture unless rendering is skipped.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_run_frame(core: *mut Core) {
    if let Some(c) = unsafe { core.as_mut() } {
        c.run_frame();
    }
}

/// The interpreter alone for `steps` instructions, as the corpus runs.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_run_steps(core: *mut Core, steps: u64) {
    if let Some(c) = unsafe { core.as_mut() } {
        c.machine.run_steps(steps);
    }
}

/// Bit 0 skips rendering; bit 1 turns the idle skip off; bit 2 steps the signal processor through the idle loop; bit 3 frames the RDP's words;
/// bit 4 is C#'s `RepeatRows`.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_set_options(core: *mut Core, flags: u32) {
    if let Some(c) = unsafe { core.as_mut() } {
        c.skip_rendering = flags & 1 != 0;
        c.machine.options.idle_skip = flags & 2 == 0;
        c.machine.options.rsp_whole = flags & 4 == 0;
        c.machine.bus.dp.framer.on = flags & 8 != 0;
        c.scanout.repeat_rows = flags & 16 != 0;
    }
}

/// The RDRAM ranges the framer saw primitives drawn into since the last call, as start and end pairs, up to `max` pairs; returns the pairs.
///
/// # Safety
/// `core` must be live or null; `out` valid for `2 * max` values.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_take_rdp_regions(core: *mut Core, out: *mut u32, max: u64) -> u64 {
    let Some(c) = (unsafe { core.as_mut() }) else { return 0 };
    let regions = std::mem::take(&mut c.machine.bus.dp.framer.regions);
    let n = regions.len().min(max as usize);
    if !out.is_null() {
        for (i, &(start, end)) in regions.iter().take(n).enumerate() {
            unsafe {
                *out.add(2 * i) = start;
                *out.add(2 * i + 1) = end;
            }
        }
    }
    n as u64
}

/// `Press`: one mask of the joybus's button bits on a port.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_press(core: *mut Core, port: u32, mask: u32, pressed: u32) {
    if let Some(c) = unsafe { core.as_mut() } {
        c.machine.press(port as usize, mask as u16, pressed != 0);
    }
}

/// The stick's reach on one axis, 0 X and 1 Y.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_set_stick(core: *mut Core, port: u32, axis: u32, value: i32) {
    if let Some(c) = unsafe { core.as_mut() } {
        c.machine.set_stick(port as usize, axis, value as i8);
    }
}

/// Samples waiting, interleaved left then right.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_audio_buffered(core: *const Core) -> u64 {
    unsafe { core.as_ref() }.map_or(0, |c| c.machine.bus.ai.samples.len() as u64)
}

/// `Drain`: up to `max_frames` stereo pairs into `out`, which holds twice that; returns the samples written.
///
/// # Safety
/// `core` must be live or null; `out` valid for `2 * max_frames` samples.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_drain_audio(core: *mut Core, out: *mut i16, max_frames: u64) -> u64 {
    let Some(c) = (unsafe { core.as_mut() }) else { return 0 };
    if out.is_null() {
        return 0;
    }
    let samples = c.machine.bus.ai.drain(max_frames as usize);
    unsafe { ptr::copy_nonoverlapping(samples.as_ptr(), out, samples.len()) };
    samples.len() as u64
}

/// `AudioSampleRate`: the rate the game set the DAC to.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_audio_sample_rate(core: *const Core) -> i32 {
    unsafe { core.as_ref() }.map_or(crate::ai::DEFAULT_SAMPLE_RATE, |c| c.machine.bus.ai.sample_rate(c.machine.bus.vi.video_clock()))
}

/// The picture the last scan composed: width, height, rows shown per row, and the serial; the bytes are `mars_machine_frame_bytes`.
/// Returns whether that scan walked, as C#'s `Scan()` does.
///
/// # Safety
/// `core` must be live or null; `out` valid for four values.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_frame_info(core: *const Core, out: *mut i64) -> u32 {
    let Some(c) = (unsafe { core.as_ref() }) else { return 0 };
    if !out.is_null() {
        let values = [c.scanout.width as i64, c.scanout.height as i64, c.scanout.row_repeat as i64, c.frame_serial];
        unsafe { ptr::copy_nonoverlapping(values.as_ptr(), out, 4) };
    }
    c.shown as u32
}

/// The last scan's RGBA bytes, copied up to `len`; returns their whole length.
///
/// # Safety
/// `core` must be live or null; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_frame_bytes(core: *const Core, out: *mut u8, len: usize) -> u64 {
    let Some(c) = (unsafe { core.as_ref() }) else { return 0 };
    let frame = &c.scanout.frame;
    if !out.is_null() {
        unsafe { ptr::copy_nonoverlapping(frame.as_ptr(), out, frame.len().min(len)) };
    }
    frame.len() as u64
}

/// Cycles, total frames, the last frame's cycles, instructions, and the idle loop's turns and instructions.
///
/// # Safety
/// `core` must be live or null; `out` valid for six values.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_counters(core: *const Core, out: *mut i64) {
    let Some(c) = (unsafe { core.as_ref() }) else { return };
    if out.is_null() {
        return;
    }
    let m = &c.machine;
    let values = [m.bus.cycles, m.total_frames, m.last_frame_cycles, m.cpu.instructions, m.cpu.run.idle_turns_passed, m.cpu.run.idle_instructions];
    unsafe { ptr::copy_nonoverlapping(values.as_ptr(), out, values.len()) };
}

/// The IS-Viewer's transcript, copied up to `len`; returns its whole length.
///
/// # Safety
/// `core` must be live or null; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_is_viewer_text(core: *const Core, out: *mut u8, len: usize) -> u64 {
    let Some(c) = (unsafe { core.as_ref() }) else { return 0 };
    let text = &c.machine.bus.is_viewer.text;
    if !out.is_null() {
        unsafe { ptr::copy_nonoverlapping(text.as_ptr(), out, text.len().min(len)) };
    }
    text.len() as u64
}

/// The save chip: its type, whether it changed since the last save, and its bytes copied up to `len`; returns their length.
///
/// # Safety
/// `core` must be live or null; `out` valid for `len` bytes or null; `info` valid for two values or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_save_data(core: *const Core, out: *mut u8, len: usize, info: *mut i32) -> i64 {
    let Some(c) = (unsafe { core.as_ref() }) else { return STATUS_NULL as i64 };
    let save = &c.machine.bus.save;
    if !info.is_null() {
        unsafe { ptr::copy_nonoverlapping([save.kind, save.dirty() as i32].as_ptr(), info, 2) };
    }
    let Some(contents) = save.contents() else { return 0 };
    if !out.is_null() {
        unsafe { ptr::copy_nonoverlapping(contents.as_ptr(), out, contents.len().min(len)) };
    }
    contents.len() as i64
}

/// The first port's pak: whether it changed, its bytes copied up to `len`; returns their length, or zero for no pak.
///
/// # Safety
/// `core` must be live or null; `out` valid for `len` bytes or null; `dirty` valid or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_pak_data(core: *const Core, out: *mut u8, len: usize, dirty: *mut i32) -> i64 {
    let Some(c) = (unsafe { core.as_ref() }) else { return STATUS_NULL as i64 };
    let Some(pak) = &c.machine.bus.si.controllers[0].pak else { return 0 };
    if !dirty.is_null() {
        unsafe { *dirty = pak.dirty as i32 };
    }
    if !out.is_null() {
        unsafe { ptr::copy_nonoverlapping(pak.data.as_ptr(), out, pak.data.len().min(len)) };
    }
    pak.data.len() as i64
}

/// `Saved`: the host wrote the chip's file (bit 0) or the pak's (bit 1).
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_mark_saved(core: *mut Core, which: u32) {
    let Some(c) = (unsafe { core.as_mut() }) else { return };
    if which & 1 != 0 {
        c.machine.bus.save.set_dirty(false);
    }
    if which & 2 != 0
        && let Some(pak) = &mut c.machine.bus.si.controllers[0].pak
    {
        pak.dirty = false;
    }
}

/// The VI's field count, which ends a frame.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_vi_fields(core: *const Core) -> i64 {
    unsafe { core.as_ref() }.map_or(0, |c| c.machine.bus.vi.fields)
}

/// The CPU's fields alone, as C#'s `StateSerializer.Write(w, cpu)` writes them; returns the bytes, or a negative status.
///
/// # Safety
/// `core` must be live or null; `out` valid for `len` bytes, or null to ask the length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_save_cpu(core: *const Core, out: *mut u8, len: usize) -> i64 {
    let Some(c) = (unsafe { core.as_ref() }) else { return STATUS_NULL as i64 };
    if out.is_null() {
        let mut w = StateWriter::counter();
        c.machine.cpu.write_state(&mut w);
        return w.len() as i64;
    }
    let mut w = StateWriter::new(unsafe { std::slice::from_raw_parts_mut(out, len) });
    c.machine.cpu.write_state(&mut w);
    if w.overflowed() { STATUS_NULL as i64 } else { w.len() as i64 }
}
