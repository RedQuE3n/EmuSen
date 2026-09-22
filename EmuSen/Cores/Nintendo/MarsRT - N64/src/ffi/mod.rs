//! MarsRT's C ABI: the machine, its state, and a frame at a time. A negative return is a status. See Mars_Native.md §5.1, §5.2 and §5.5.

pub mod rdp;
pub mod threads;
pub mod vi;

use std::ptr;
use std::sync::Arc;

use crate::cpu::cop0::ENTRY_HI;
use crate::cpu::segments::{self, Mode, Segment};
use crate::cpu::tlb::TlbResult;
use crate::machine::Machine;
use crate::memory::bus_access::map;
use crate::rom::RomImage;
use crate::state::{State, StateResult, StateWriter};
use crate::vi::scan::Scanout;

/// A null handle or buffer.
pub const STATUS_NULL: i32 = -1;
/// An image that is not a Nintendo 64 cartridge.
pub const STATUS_NOT_A_ROM: i32 = -9;
/// A memory this library has no number for.
pub const STATUS_NO_SUCH_SPACE: i32 = -10;
/// A memory the host may read and not write.
pub const STATUS_READ_ONLY: i32 = -11;

/// The memories a host names, as C#'s `MarsDebugSpaces` names them; `CPU` is the processor's kernel view of a virtual address.
pub mod space {
    pub const RDRAM: u32 = 0;
    pub const DMEM: u32 = 1;
    pub const IMEM: u32 = 2;
    pub const PIF_RAM: u32 = 3;
    pub const ROM: u32 = 4;
    pub const CPU: u32 = 5;
}

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
    pub fn new(machine: Machine) -> Core {
        Core { machine, scanout: Scanout::default(), frame_serial: 0, skip_rendering: false, shown: false }
    }

    /// `Present`: the VI's scan of what the machine left, at once or deferred as the machine is set; the serial moves when the shown frame does.
    pub fn present(&mut self) {
        let presented = self.machine.present(&mut self.scanout);
        self.shown = presented.walked;
        self.frame_serial += presented.replaced as i64;
    }

    pub fn run_frame(&mut self) {
        self.machine.run_frame();
        if !self.skip_rendering {
            self.present();
        }
    }

    /// `MarsDebugSpaces.TryPhysical`: kernel mode's view of a 32-bit virtual address, the TLB consulted and never faulted.
    pub fn physical(&self, address: u32) -> Option<u32> {
        let wide = address as i32 as i64 as u64;
        match segments::decode(wide, Mode::Kernel, false) {
            Segment::Direct(physical) => Some(physical),
            Segment::Mapped => match self.machine.cpu.tlb.try_translate(wide, self.machine.cpu.cop0[ENTRY_HI], false) {
                TlbResult::Mapped(physical) => Some(physical),
                _ => None,
            },
            Segment::Illegal => None,
        }
    }

    /// `MarsDebugSpaces.Window`: which memory a physical address names and where, for memories only, since a register can answer a read by changing.
    fn window(&self, physical: u32) -> Option<(u32, usize)> {
        let bus = &self.machine.bus;
        if (physical as usize) < bus.rdram.len() {
            return Some((space::RDRAM, physical as usize));
        }
        if (map::SP_DMEM_BASE..map::SP_REGISTERS_BASE).contains(&physical) {
            let local = (physical - map::SP_DMEM_BASE) % (2 * map::SP_MEM_SIZE);
            return Some(if local < map::SP_MEM_SIZE { (space::DMEM, local as usize) } else { (space::IMEM, (local - map::SP_MEM_SIZE) as usize) });
        }
        if (map::PIF_RAM_BASE..map::PIF_RAM_BASE + map::PIF_RAM_SIZE).contains(&physical) {
            return Some((space::PIF_RAM, (physical - map::PIF_RAM_BASE) as usize));
        }
        if (map::CART_DOMAIN1_ADDRESS2..map::PIF_ROM_BASE).contains(&physical) {
            let offset = (physical - map::CART_DOMAIN1_ADDRESS2) as usize;
            if offset < self.memory(space::ROM).map_or(0, <[u8]>::len) {
                return Some((space::ROM, offset));
            }
        }
        None
    }

    /// A named memory's bytes, or none for a number that names no memory.
    pub fn memory(&self, space: u32) -> Option<&[u8]> {
        let bus = &self.machine.bus;
        match space {
            space::RDRAM => Some(&bus.rdram),
            space::DMEM => Some(&bus.sp_dmem[..]),
            space::IMEM => Some(&bus.sp_imem[..]),
            space::PIF_RAM => Some(&bus.pif_ram),
            space::ROM => Some(bus.cart.as_ref().map_or(&[][..], |rom| &rom.rom[..])),
            _ => None,
        }
    }

    fn memory_mut(&mut self, space: u32) -> Option<&mut [u8]> {
        let bus = &mut self.machine.bus;
        match space {
            space::RDRAM => Some(&mut bus.rdram),
            space::DMEM => Some(&mut bus.sp_dmem[..]),
            space::IMEM => Some(&mut bus.sp_imem[..]),
            space::PIF_RAM => Some(&mut bus.pif_ram),
            _ => None,
        }
    }

    /// Bytes from a memory into `out`, zero wherever the address names nothing, and nothing disturbed; the host reads between frames.
    pub fn read_memory(&self, space: u32, address: u32, out: &mut [u8]) -> Result<(), i32> {
        // C# waits at site 9 per byte; RDRAM's whole slice is formed below, so the drain is waited for whole (Mars_Native.md §5.6.4).
        if space == space::RDRAM || space == space::CPU {
            self.machine.bus.dp.wait_all();
        }
        if space == space::CPU {
            for (i, byte) in out.iter_mut().enumerate() {
                *byte = self
                    .physical(address.wrapping_add(i as u32))
                    .and_then(|physical| self.window(physical))
                    .and_then(|(space, offset)| self.memory(space).map(|bytes| bytes[offset]))
                    .unwrap_or(0);
            }
            return Ok(());
        }
        let bytes = self.memory(space).ok_or(STATUS_NO_SUCH_SPACE)?;
        out.fill(0);
        let start = address as usize;
        if start < bytes.len() {
            let n = out.len().min(bytes.len() - start);
            out[..n].copy_from_slice(&bytes[start..start + n]);
        }
        Ok(())
    }

    /// Bytes into a memory, those past its end dropped, as a store to memory not installed is; returns how many landed, and any lands as a write the idle loop sees.
    pub fn write_memory(&mut self, space: u32, address: u32, data: &[u8]) -> Result<usize, i32> {
        if space == space::RDRAM || space == space::CPU {
            self.machine.join_rdp();
        }
        let mut landed = 0;
        if space == space::CPU {
            for (i, &value) in data.iter().enumerate() {
                let Some((space, offset)) = self.physical(address.wrapping_add(i as u32)).and_then(|physical| self.window(physical)) else { continue };
                if let Some(bytes) = self.memory_mut(space) {
                    bytes[offset] = value;
                    landed += 1;
                }
            }
        } else {
            if space == space::ROM {
                return Err(STATUS_READ_ONLY);
            }
            let bytes = self.memory_mut(space).ok_or(STATUS_NO_SUCH_SPACE)?;
            let start = address as usize;
            if start < bytes.len() {
                landed = data.len().min(bytes.len() - start);
                bytes[start..start + landed].copy_from_slice(&data[..landed]);
            }
        }
        if landed > 0 {
            *self.machine.bus.written += 1;
        }
        Ok(landed)
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
    let rdram = c.machine.rdram_bytes();
    match c.machine.restore_state(unsafe { input(data, len) }) {
        Ok(()) => {
            // A state of the other size rebuilds the C# machine, and its raster with it.
            if c.machine.rdram_bytes() != rdram {
                let repeat_rows = c.scanout.repeat_rows;
                c.scanout = Scanout::default();
                c.scanout.repeat_rows = repeat_rows;
            }
            c.scanout.forget();
            if !c.skip_rendering {
                let presented = c.machine.present_now(&mut c.scanout);
                c.shown = presented.walked;
                c.frame_serial += 1;
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

/// One frame and no picture, so the host can do what `MarsCore.RunFrame` does between the frame's end and `Present`: cheats, then the save.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_advance(core: *mut Core) {
    if let Some(c) = unsafe { core.as_mut() } {
        c.machine.run_frame();
    }
}

/// `Present`: the VI's scan of what the machine left, whatever the skip option says.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_present(core: *mut Core) {
    if let Some(c) = unsafe { core.as_mut() } {
        c.present();
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

/// Bit 0 skips rendering; bit 1 turns the idle skip off; bit 2 steps the signal processor through the idle loop; bit 4 is C#'s `RepeatRows`.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_set_options(core: *mut Core, flags: u32) {
    if let Some(c) = unsafe { core.as_mut() } {
        c.skip_rendering = flags & 1 != 0;
        c.machine.options.idle_skip = flags & 2 == 0;
        c.machine.options.rsp_whole = flags & 4 == 0;
        c.scanout.repeat_rows = flags & 16 != 0;
    }
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
    unsafe { core.as_ref() }.map_or(crate::memory::ai::DEFAULT_SAMPLE_RATE, |c| c.machine.bus.ai.sample_rate(c.machine.bus.vi.video_clock()))
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

/// A memory's length in bytes, or a negative status; the processor's view is the whole 32-bit space.
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_memory_size(core: *const Core, space: u32) -> i64 {
    let Some(c) = (unsafe { core.as_ref() }) else { return STATUS_NULL as i64 };
    if space == space::CPU {
        return 1 << 32;
    }
    c.memory(space).map_or(STATUS_NO_SUCH_SPACE as i64, |bytes| bytes.len() as i64)
}

/// `len` bytes of a memory from `address`, zero wherever it names nothing; returns `len`, or a negative status.
///
/// # Safety
/// `core` must be live or null; `out` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_read_memory(core: *const Core, space: u32, address: u32, out: *mut u8, len: usize) -> i64 {
    let Some(c) = (unsafe { core.as_ref() }) else { return STATUS_NULL as i64 };
    if out.is_null() {
        return STATUS_NULL as i64;
    }
    match c.read_memory(space, address, unsafe { std::slice::from_raw_parts_mut(out, len) }) {
        Ok(()) => len as i64,
        Err(status) => status as i64,
    }
}

/// `len` bytes into a memory at `address`, those it cannot hold dropped; returns how many landed, or a negative status.
///
/// # Safety
/// `core` must be live or null; `data` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_write_memory(core: *mut Core, space: u32, address: u32, data: *const u8, len: usize) -> i64 {
    let Some(c) = (unsafe { core.as_mut() }) else { return STATUS_NULL as i64 };
    if data.is_null() {
        return STATUS_NULL as i64;
    }
    match c.write_memory(space, address, unsafe { input(data, len) }) {
        Ok(landed) => landed as i64,
        Err(status) => status as i64,
    }
}

/// One COP0 register, `Cop0[register]`; the cheat gate reads Status (12).
///
/// # Safety
/// `core` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_cop0(core: *const Core, register: u32) -> u64 {
    unsafe { core.as_ref() }.map_or(0, |c| c.machine.cpu.cop0[(register & 31) as usize])
}

/// The CPU's registers for a debugger: PC, the 32 GPRs, HI, LO and the bus's cycles, up to `len` of them; returns 36.
///
/// # Safety
/// `core` must be live or null; `out` valid for `len` values, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_cpu_registers(core: *const Core, out: *mut u64, len: usize) -> i64 {
    let Some(c) = (unsafe { core.as_ref() }) else { return STATUS_NULL as i64 };
    let cpu = &c.machine.cpu;
    let mut values = Vec::with_capacity(36);
    values.push(cpu.pc);
    values.extend_from_slice(&cpu.gpr);
    values.extend_from_slice(&[cpu.hi, cpu.lo, c.machine.bus.cycles as u64]);
    if !out.is_null() {
        unsafe { ptr::copy_nonoverlapping(values.as_ptr(), out, values.len().min(len)) };
    }
    values.len() as i64
}

/// The signal processor's registers for a debugger: PC, halted, broke, and the 32 scalar registers, up to `len`; returns 35.
///
/// # Safety
/// `core` must be live or null; `out` valid for `len` values, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_rsp_registers(core: *const Core, out: *mut u32, len: usize) -> i64 {
    let Some(c) = (unsafe { core.as_ref() }) else { return STATUS_NULL as i64 };
    let rsp = &c.machine.bus.sp.processor;
    let mut values = Vec::with_capacity(35);
    values.extend_from_slice(&[rsp.pc, rsp.halted as u32, rsp.broke as u32]);
    values.extend_from_slice(&rsp.gpr);
    if !out.is_null() {
        unsafe { ptr::copy_nonoverlapping(values.as_ptr(), out, values.len().min(len)) };
    }
    values.len() as i64
}

/// The VI's fourteen registers as stored, up to `len`; returns 14.
///
/// # Safety
/// `core` must be live or null; `out` valid for `len` values, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_machine_vi_registers(core: *const Core, out: *mut u32, len: usize) -> i64 {
    let Some(c) = (unsafe { core.as_ref() }) else { return STATUS_NULL as i64 };
    let values: Vec<u32> = (0..crate::vi::REGISTERS).map(|i| c.machine.bus.vi.read32(i * 4)).collect();
    if !out.is_null() {
        unsafe { ptr::copy_nonoverlapping(values.as_ptr(), out, values.len().min(len)) };
    }
    values.len() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::bus::RDRAM_SIZE;

    fn core() -> Core {
        Core::new(Machine::new(RDRAM_SIZE).unwrap())
    }

    #[test]
    fn a_read_past_a_memory_s_end_is_zero_and_a_write_there_is_dropped() {
        let mut c = core();
        c.machine.bus.rdram[RDRAM_SIZE - 1] = 0x5A;
        let mut out = [0xEEu8; 4];
        c.read_memory(space::RDRAM, (RDRAM_SIZE - 1) as u32, &mut out).unwrap();
        assert_eq!(out, [0x5A, 0, 0, 0]);
        assert_eq!(c.write_memory(space::RDRAM, (RDRAM_SIZE - 2) as u32, &[1, 2, 3]).unwrap(), 2);
        assert_eq!(&c.machine.bus.rdram[RDRAM_SIZE - 2..], &[1, 2]);
        assert_eq!(c.write_memory(space::PIF_RAM, 64, &[9]).unwrap(), 0);
    }

    #[test]
    fn a_write_that_lands_is_one_the_idle_loop_sees_and_one_that_does_not_is_not() {
        let mut c = core();
        let before = *c.machine.bus.written;
        c.write_memory(space::DMEM, 0x2000, &[1]).unwrap();
        assert_eq!(*c.machine.bus.written, before);
        c.write_memory(space::DMEM, 0xFFF, &[1]).unwrap();
        assert_eq!(*c.machine.bus.written, before + 1);
        assert_eq!(c.machine.bus.sp_dmem[0xFFF], 1);
    }

    #[test]
    fn the_rom_is_read_and_never_written_and_an_unknown_space_is_refused() {
        let mut c = core();
        assert_eq!(c.write_memory(space::ROM, 0, &[1]), Err(STATUS_READ_ONLY));
        assert_eq!(c.write_memory(9, 0, &[1]), Err(STATUS_NO_SUCH_SPACE));
        assert_eq!(c.read_memory(9, 0, &mut [0]), Err(STATUS_NO_SUCH_SPACE));
    }

    #[test]
    fn the_processor_s_view_reaches_rdram_through_both_direct_segments_and_imem_through_its_mirror() {
        let mut c = core();
        c.machine.bus.rdram[0x1234] = 0x11;
        c.machine.bus.sp_imem[0x10] = 0x22;
        let mut out = [0u8; 1];
        for address in [0x8000_1234u32, 0xA000_1234] {
            c.read_memory(space::CPU, address, &mut out).unwrap();
            assert_eq!(out[0], 0x11);
        }
        c.read_memory(space::CPU, 0xA400_1010 + 0x2000, &mut out).unwrap();
        assert_eq!(out[0], 0x22);
        c.write_memory(space::CPU, 0x8000_0010, &[0x33]).unwrap();
        assert_eq!(c.machine.bus.rdram[0x10], 0x33);
    }

    #[test]
    fn an_unmapped_address_in_the_processor_s_view_reads_zero_and_takes_no_write() {
        let mut c = core();
        let mut out = [0xEEu8; 2];
        c.read_memory(space::CPU, 0x0000_1000, &mut out).unwrap();
        assert_eq!(out, [0, 0]);
        assert_eq!(c.write_memory(space::CPU, 0x0000_1000, &[1, 2]).unwrap(), 0);
    }
}
