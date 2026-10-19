//! One machine: an owned handle on a library that lives for the process, one method per v1 call. Every call a frame
//! makes writes into the caller's buffer or one the machine keeps, so a frame allocates nothing (EmuSen_Backend.md
//! §27.3). The frame-end order between `advance` and `present` is the runtime's, not this crate's.

use std::ffi::CStr;
use std::ptr::NonNull;

use emusen_native::core::sys::{CreateParams, DeviceFrame, DeviceLend, Event, FileEntry, FrameInfo, Machine as RawMachine, Media, RawFrame, caps, status};

use crate::library::{Library, text};

/// A status the core answered, negative.
pub type Status = i32;

/// What a create is given.
#[derive(Clone, Copy)]
pub struct Create<'a> {
    pub image: &'a [u8],
    pub settings: &'a str,
    pub files: &'a [(u32, &'a [u8])],
    pub pixel_formats: u64,
    pub flags: u32,
    /// Passed only to a core that claims `DEVICE_PICTURE`.
    pub device: Option<&'a DeviceLend>,
    /// With an empty image, the system to start empty (since 1.11).
    pub system: Option<&'a CStr>,
}

impl<'a> Create<'a> {
    pub fn new(image: &'a [u8]) -> Create<'a> {
        Create { image, settings: "", files: &[], pixel_formats: 1, flags: 0, device: None, system: None }
    }
}

/// A create the core refused: its status and its words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal {
    pub status: Status,
    pub words: String,
}

/// One machine, freed when dropped. It may move between threads, used by one at a time (EmuSen_CoreAPI.md §6.16).
pub struct Machine {
    lib: &'static Library,
    h: NonNull<RawMachine>,
    events: Vec<Event>,
}

// SAFETY: the ABI lets a machine be used from any thread, one at a time; `&mut self` on every call that changes it.
unsafe impl Send for Machine {}

fn ok(code: i32) -> Result<(), Status> {
    if code < 0 { Err(code) } else { Ok(()) }
}

fn count(code: i64) -> Result<usize, Status> {
    if code < 0 { Err(code.max(i32::MIN as i64) as i32) } else { Ok(code as usize) }
}

impl Machine {
    /// A machine for `c`, at this host's minor with every struct its own size.
    pub fn create(lib: &'static Library, c: &Create<'_>) -> Result<Machine, Refusal> {
        let entries: Vec<FileEntry> = c.files.iter().map(|(w, d)| FileEntry { size: std::mem::size_of::<FileEntry>() as u32, which: *w, data: d.as_ptr(), len: d.len() }).collect();
        let mut error = [0u8; 1024];
        let device = c.device.filter(|_| lib.claims(caps::DEVICE_PICTURE)).map_or(std::ptr::null(), |d| d as *const DeviceLend);
        let p = CreateParams {
            size: std::mem::size_of::<CreateParams>() as u32,
            host_abi_version: emusen_native::core::sys::ABI_VERSION,
            image: c.image.as_ptr(),
            image_len: c.image.len(),
            settings: c.settings.as_ptr(),
            settings_len: c.settings.len(),
            files: entries.as_ptr(),
            file_count: entries.len(),
            file_size: std::mem::size_of::<FileEntry>(),
            pixel_formats: c.pixel_formats,
            error: error.as_mut_ptr(),
            error_len: error.len(),
            flags: c.flags,
            device,
            system: c.system.map_or(std::ptr::null(), CStr::as_ptr),
        };
        let mut code = 0;
        // SAFETY: every pointer in `p` outlives the call.
        let h = unsafe { (lib.exports().create)(&p, &mut code) };
        match NonNull::new(h) {
            Some(h) => Ok(Machine { lib, h, events: Vec::new() }),
            None => {
                let end = error.iter().position(|&b| b == 0).unwrap_or(0);
                let words = if end > 0 { String::from_utf8_lossy(&error[..end]).into_owned() } else { lib.words(code as i64) };
                Err(Refusal { status: code, words })
            }
        }
    }

    /// Takes ownership of a handle a raw create of `lib` returned, as the kit's skewed host makes them.
    ///
    /// # Safety
    /// `h` is a live machine of `lib` that nothing else frees.
    pub unsafe fn adopt(lib: &'static Library, h: *mut RawMachine) -> Option<Machine> {
        NonNull::new(h).map(|h| Machine { lib, h, events: Vec::new() })
    }

    pub fn library(&self) -> &'static Library {
        self.lib
    }

    /// The handle, for a call this type does not wrap; it stays this machine's.
    pub fn raw(&self) -> *mut RawMachine {
        self.h.as_ptr()
    }

    fn h(&self) -> *mut RawMachine {
        self.h.as_ptr()
    }

    /// The words for a status: the machine's `last_error`, else `status_text`.
    pub fn words(&self, code: i64) -> String {
        let detail = self.last_error();
        if detail.is_empty() { self.lib.words(code) } else { detail }
    }

    pub fn last_error(&self) -> String {
        text(|o, l| unsafe { (self.lib.exports().last_error)(self.h(), o, l) }).unwrap_or_default()
    }

    pub fn machine_info(&self) -> Result<String, Status> {
        text(|o, l| unsafe { (self.lib.exports().machine_info)(self.h(), o, l) }).map_err(|e| e as i32)
    }

    /// One frame; the detail word the core gave with a fault.
    pub fn advance(&mut self) -> Result<(), (Status, u64)> {
        let mut detail = 0u64;
        match unsafe { (self.lib.exports().advance)(self.h(), &mut detail) } {
            0 => Ok(()),
            e => Err((e, detail)),
        }
    }

    /// `present` where the core claims it.
    pub fn present(&mut self) -> Result<(), Status> {
        self.lib.exports().present.map_or(Ok(()), |f| ok(unsafe { f(self.h()) }))
    }

    pub fn reset(&mut self) -> Result<(), Status> {
        self.lib.exports().reset.map_or(Err(status::NOT_SUPPORTED), |f| ok(unsafe { f(self.h()) }))
    }

    pub fn set_options(&mut self, flags: u32) -> Result<(), Status> {
        ok(unsafe { (self.lib.exports().set_options)(self.h(), flags) })
    }

    pub fn frame_count(&self) -> i64 {
        unsafe { (self.lib.exports().frame_count)(self.h()) }
    }

    pub fn frame_info(&self) -> Result<FrameInfo, Status> {
        let mut fi = FrameInfo { size: std::mem::size_of::<FrameInfo>() as u32, ..FrameInfo::default() };
        ok(unsafe { (self.lib.exports().frame_info)(self.h(), &mut fi) }).map(|_| fi)
    }

    /// The picture's bytes into `into`; the bytes the picture holds, which may be more than `into` takes.
    pub fn frame_copy(&self, into: &mut [u8]) -> Result<usize, Status> {
        count(unsafe { (self.lib.exports().frame_copy)(self.h(), into.as_mut_ptr(), into.len()) })
    }

    /// The bytes the picture holds now.
    pub fn frame_bytes(&self) -> Result<usize, Status> {
        count(unsafe { (self.lib.exports().frame_copy)(self.h(), std::ptr::null_mut(), 0) })
    }

    /// `frame_device` (since 1.9): `None` where the core lacks it or the picture is not on the device.
    pub fn frame_device(&mut self) -> Result<Option<DeviceFrame>, Status> {
        let Some(f) = self.lib.exports().frame_device else { return Ok(None) };
        let mut out = DeviceFrame { size: std::mem::size_of::<DeviceFrame>() as u32, ..DeviceFrame::default() };
        match unsafe { f(self.h(), &mut out) } {
            1 => Ok(Some(out)),
            0 => Ok(None),
            e => Err(e),
        }
    }

    pub fn audio_rate(&self) -> i32 {
        unsafe { (self.lib.exports().audio_rate)(self.h()) }
    }

    pub fn audio_buffered(&self) -> i64 {
        unsafe { (self.lib.exports().audio_buffered)(self.h()) }
    }

    /// Interleaved stereo samples into `into`, at most `into.len() / 2` frames; the samples written and their rate.
    pub fn audio_drain(&mut self, into: &mut [i16]) -> Result<(usize, i32), Status> {
        let mut rate = 0;
        let n = unsafe { (self.lib.exports().audio_drain)(self.h(), into.as_mut_ptr(), into.len(), (into.len() / 2) as i64, &mut rate) };
        count(n).map(|n| (n, rate))
    }

    pub fn audio_peek(&self, into: &mut [i16]) -> Result<usize, Status> {
        self.lib.exports().audio_peek.map_or(Err(status::NOT_SUPPORTED), |f| count(unsafe { f(self.h(), into.as_mut_ptr(), into.len()) }))
    }

    pub fn set_audio_limit(&mut self, samples: u64) -> Result<(), Status> {
        ok(unsafe { (self.lib.exports().set_audio_limit)(self.h(), samples) })
    }

    pub fn set_mutes(&mut self, mask: u32) -> Result<(), Status> {
        self.lib.exports().set_mutes.map_or(Err(status::NOT_SUPPORTED), |f| ok(unsafe { f(self.h(), mask) }))
    }

    /// The events since the last drain, in a buffer the machine keeps and grows only when more arrive than it held.
    pub fn events(&mut self) -> Result<&[Event], Status> {
        let f = self.lib.exports().events;
        let n = count(unsafe { f(self.h(), std::ptr::null_mut(), 0, 0) })?;
        if self.events.len() < n {
            self.events.resize(n, Event::default());
        }
        for e in &mut self.events[..n] {
            *e = Event { size: std::mem::size_of::<Event>() as u32, ..Event::default() };
        }
        let got = count(unsafe { f(self.h(), self.events.as_mut_ptr(), n, std::mem::size_of::<Event>()) })?;
        Ok(&self.events[..got.min(n)])
    }

    pub fn set_buttons(&mut self, port: u32, mask: u32, changed: u32) -> Result<(), Status> {
        ok(unsafe { (self.lib.exports().set_buttons)(self.h(), port, mask, changed) })
    }

    pub fn set_axis(&mut self, port: u32, axis: u32, value: f64) -> Result<(), Status> {
        self.lib.exports().set_axis.map_or(Err(status::NOT_SUPPORTED), |f| ok(unsafe { f(self.h(), port, axis, value) }))
    }

    /// `set_present` (since 1.13).
    pub fn set_present(&mut self, port: u32, present: bool) -> Result<(), Status> {
        self.lib.exports().set_present.map_or(Err(status::NOT_SUPPORTED), |f| ok(unsafe { f(self.h(), port, present as u32) }))
    }

    pub fn state_size(&self, kind: u32) -> Result<usize, Status> {
        count(unsafe { (self.lib.exports().state_size)(self.h(), kind) })
    }

    /// The state of `kind` into `into`, which holds at least `state_size`; the bytes written.
    pub fn state_save(&mut self, kind: u32, into: &mut [u8]) -> Result<usize, Status> {
        count(unsafe { (self.lib.exports().state_save)(self.h(), kind, into.as_mut_ptr(), into.len()) })
    }

    /// A failed load changes nothing (EmuSen_CoreAPI.md §6.9).
    pub fn state_load(&mut self, state: &[u8]) -> Result<(), Status> {
        ok(unsafe { (self.lib.exports().state_load)(self.h(), state.as_ptr(), state.len()) })
    }

    /// `state_layout`'s listing, one line per field.
    pub fn layout(&self, kind: u32) -> Result<String, Status> {
        text(|o, l| unsafe { (self.lib.exports().state_layout)(self.h(), kind, o, l) }).map_err(|e| e as i32)
    }

    pub fn space_size(&self, space: u32) -> Result<u64, Status> {
        count(unsafe { (self.lib.exports().space_size)(self.h(), space) }).map(|n| n as u64)
    }

    pub fn space_read(&mut self, space: u32, address: u32, into: &mut [u8]) -> Result<usize, Status> {
        count(unsafe { (self.lib.exports().space_read)(self.h(), space, address, into.as_mut_ptr(), into.len()) })
    }

    pub fn space_write(&mut self, space: u32, address: u32, from: &[u8]) -> Result<usize, Status> {
        count(unsafe { (self.lib.exports().space_write)(self.h(), space, address, from.as_ptr(), from.len()) })
    }

    /// Battery file `which` into `into`, resized to it; its `EMUSEN_BATTERY_*` flags.
    pub fn battery(&self, which: u32, into: &mut Vec<u8>) -> Result<u32, Status> {
        let f = self.lib.exports().battery;
        let mut flags = 0u32;
        let n = count(unsafe { f(self.h(), which, std::ptr::null_mut(), 0, &mut flags) })?;
        into.resize(n, 0);
        let m = count(unsafe { f(self.h(), which, into.as_mut_ptr(), into.len(), &mut flags) })?;
        into.truncate(m);
        Ok(flags)
    }

    pub fn battery_saved(&mut self, which: u32) -> Result<(), Status> {
        ok(unsafe { (self.lib.exports().battery_saved)(self.h(), which) })
    }

    pub fn set_rom_patches(&mut self, triples: &[u32]) -> Result<usize, Status> {
        self.lib.exports().set_rom_patches.map_or(Err(status::NOT_SUPPORTED), |f| count(unsafe { f(self.h(), triples.as_ptr(), triples.len()) }))
    }

    pub fn set_cheat_pokes(&mut self, quads: &[u32]) -> Result<usize, Status> {
        self.lib.exports().set_cheat_pokes.map_or(Err(status::NOT_SUPPORTED), |f| count(unsafe { f(self.h(), quads.as_ptr(), quads.len()) }))
    }

    pub fn set_settings(&mut self, settings: &str) -> Result<(), Status> {
        self.lib.exports().set_settings.map_or(Err(status::NOT_SUPPORTED), |f| ok(unsafe { f(self.h(), settings.as_ptr(), settings.len()) }))
    }

    pub fn setting_notes(&self) -> Result<String, Status> {
        let f = self.lib.exports().setting_notes.ok_or(status::NOT_SUPPORTED)?;
        text(|o, l| unsafe { f(self.h(), o, l) }).map_err(|e| e as i32)
    }

    /// The phases' times into `into`; how many the core has.
    pub fn phases(&self, into: &mut [i64]) -> Result<usize, Status> {
        self.lib.exports().phases.map_or(Err(status::NOT_SUPPORTED), |f| count(unsafe { f(self.h(), into.as_mut_ptr(), into.len()) }))
    }

    /// `telemetry` (since 1.13) into `into`; how many counters it wrote.
    pub fn telemetry(&self, into: &mut [i64]) -> Result<usize, Status> {
        self.lib.exports().telemetry.map_or(Err(status::NOT_SUPPORTED), |f| count(unsafe { f(self.h(), into.as_mut_ptr(), into.len()) }))
    }

    pub fn media_lid(&mut self, drive: u32, open: bool) -> Result<(), Status> {
        self.lib.exports().media_lid.map_or(Err(status::NOT_SUPPORTED), |f| ok(unsafe { f(self.h(), drive, open as u32) }))
    }

    /// A disc and its companions into `drive`, or none for `None`.
    pub fn media_insert(&mut self, drive: u32, image: Option<&[u8]>, files: &[(u32, &[u8])], flags: u32) -> Result<(), Status> {
        let f = self.lib.exports().media_insert.ok_or(status::NOT_SUPPORTED)?;
        let entries: Vec<FileEntry> = files.iter().map(|(w, d)| FileEntry { size: std::mem::size_of::<FileEntry>() as u32, which: *w, data: d.as_ptr(), len: d.len() }).collect();
        let m = Media {
            size: std::mem::size_of::<Media>() as u32,
            flags,
            image: image.map_or(std::ptr::null(), <[u8]>::as_ptr),
            image_len: image.map_or(0, <[u8]>::len),
            files: entries.as_ptr(),
            file_count: entries.len(),
            file_size: std::mem::size_of::<FileEntry>(),
        };
        ok(unsafe { f(self.h(), drive, &m) })
    }

    pub fn raw_select(&mut self, mask: u64) -> Result<(), Status> {
        self.lib.exports().raw_select.map_or(Err(status::NOT_SUPPORTED), |f| ok(unsafe { f(self.h(), mask) }))
    }

    pub fn raw_info(&self, raw: u32) -> Result<RawFrame, Status> {
        let f = self.lib.exports().raw_info.ok_or(status::NOT_SUPPORTED)?;
        let mut out = RawFrame { size: std::mem::size_of::<RawFrame>() as u32, ..RawFrame::default() };
        ok(unsafe { f(self.h(), raw, &mut out) }).map(|_| out)
    }

    pub fn raw_copy(&self, raw: u32, into: &mut [u8]) -> Result<usize, Status> {
        self.lib.exports().raw_copy.map_or(Err(status::NOT_SUPPORTED), |f| count(unsafe { f(self.h(), raw, into.as_mut_ptr(), into.len()) }))
    }

    pub fn debug_set(&mut self, what: u32, a: i32, b: i32) -> Result<(), Status> {
        self.lib.exports().debug_set.map_or(Err(status::NOT_SUPPORTED), |f| ok(unsafe { f(self.h(), what, a, b) }))
    }

    pub fn debug_set_stack(&mut self, depths: &[u32]) -> Result<(), Status> {
        self.lib.exports().debug_set_stack.map_or(Err(status::NOT_SUPPORTED), |f| ok(unsafe { f(self.h(), depths.as_ptr(), depths.len()) }))
    }

    pub fn debug_set_breakpoints(&mut self, processor: u32, table: &[i32]) -> Result<(), Status> {
        self.lib.exports().debug_set_breakpoints.map_or(Err(status::NOT_SUPPORTED), |f| ok(unsafe { f(self.h(), processor, table.as_ptr(), table.len()) }))
    }

    pub fn debug_set_ranges(&mut self, what: u32, ranges: &[u32]) -> Result<(), Status> {
        self.lib.exports().debug_set_ranges.map_or(Err(status::NOT_SUPPORTED), |f| ok(unsafe { f(self.h(), what, ranges.as_ptr(), ranges.len()) }))
    }

    /// `debug_run_frame`: its answer, the processor that stopped, the address and the detail.
    pub fn debug_run_frame(&mut self, flags: u32) -> Result<(i32, u32, u64, u64), Status> {
        let f = self.lib.exports().debug_run_frame.ok_or(status::NOT_SUPPORTED)?;
        let (mut processor, mut address, mut detail) = (0u32, 0u64, 0u64);
        let r = unsafe { f(self.h(), flags, &mut processor, &mut address, &mut detail) };
        if r < 0 { Err(r) } else { Ok((r, processor, address, detail)) }
    }

    pub fn debug_writes(&mut self, into: &mut [u32]) -> Result<usize, Status> {
        self.lib.exports().debug_writes.map_or(Err(status::NOT_SUPPORTED), |f| count(unsafe { f(self.h(), into.as_mut_ptr(), into.len()) }))
    }

    pub fn debug_reads(&mut self, into: &mut [u32]) -> Result<usize, Status> {
        self.lib.exports().debug_reads.map_or(Err(status::NOT_SUPPORTED), |f| count(unsafe { f(self.h(), into.as_mut_ptr(), into.len()) }))
    }

    pub fn debug_calls(&mut self, into: &mut [u32]) -> Result<usize, Status> {
        self.lib.exports().debug_calls.map_or(Err(status::NOT_SUPPORTED), |f| count(unsafe { f(self.h(), into.as_mut_ptr(), into.len()) }))
    }

    pub fn debug_profile(&mut self, into: &mut [i64]) -> Result<usize, Status> {
        self.lib.exports().debug_profile.map_or(Err(status::NOT_SUPPORTED), |f| count(unsafe { f(self.h(), into.as_mut_ptr(), into.len()) }))
    }

    pub fn debug_counters(&mut self, into: &mut [i64]) -> Result<usize, Status> {
        self.lib.exports().debug_counters.map_or(Err(status::NOT_SUPPORTED), |f| count(unsafe { f(self.h(), into.as_mut_ptr(), into.len()) }))
    }

    /// `debug_coverage` of a processor into `into`; the bytes written and the base address.
    pub fn debug_coverage(&mut self, processor: u32, into: &mut [u8]) -> Result<(usize, i64), Status> {
        let f = self.lib.exports().debug_coverage.ok_or(status::NOT_SUPPORTED)?;
        let mut base = 0i64;
        count(unsafe { f(self.h(), processor, into.as_mut_ptr(), into.len(), &mut base) }).map(|n| (n, base))
    }

    pub fn debug_pc(&self, processor: u32) -> Result<u64, Status> {
        let f = self.lib.exports().debug_pc.ok_or(status::NOT_SUPPORTED)?;
        let mut pc = 0u64;
        ok(unsafe { f(self.h(), processor, &mut pc) }).map(|_| pc)
    }

    /// A processor's or a unit's registers into `into`; how many it has.
    pub fn registers(&self, id: u32, into: &mut [i64]) -> Result<usize, Status> {
        self.lib.exports().debug_registers.map_or(Err(status::NOT_SUPPORTED), |f| count(unsafe { f(self.h(), id, into.as_mut_ptr(), into.len()) }))
    }

    pub fn disassemble(&mut self, processor: u32, space: u32, address: u32, lines: u32) -> Result<String, Status> {
        let f = self.lib.exports().debug_disassemble.ok_or(status::NOT_SUPPORTED)?;
        let h = self.h();
        text(|o, l| unsafe { f(h, processor, space, address, lines, o, l) }).map_err(|e| e as i32)
    }

    /// `debug_translate` (since 1.13): the processor's address on its bus.
    pub fn translate(&self, processor: u32, address: u32) -> Result<u32, Status> {
        let f = self.lib.exports().debug_translate.ok_or(status::NOT_SUPPORTED)?;
        let mut bus = 0u32;
        ok(unsafe { f(self.h(), processor, address, &mut bus) }).map(|_| bus)
    }

    /// The machine's log since the last drain, appended to `into`; nothing is allocated when the core logged nothing.
    pub fn log_drain(&mut self, into: &mut String) -> Result<usize, Status> {
        let f = self.lib.exports().log_drain;
        let n = count(unsafe { f(self.h(), std::ptr::null_mut(), 0) })?;
        if n == 0 {
            return Ok(0);
        }
        let mut buf = vec![0u8; n];
        let m = count(unsafe { f(self.h(), buf.as_mut_ptr(), n) })?;
        buf.truncate(m);
        into.push_str(&String::from_utf8_lossy(&buf));
        Ok(m)
    }
}

impl Drop for Machine {
    fn drop(&mut self) {
        unsafe { (self.lib.exports().free)(self.h()) };
    }
}
