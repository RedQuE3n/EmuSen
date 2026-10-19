//! A library loaded through the core host's loader and export table (`emusen-core-host`), and the host the kit plays
//! around it: a minor it claims and the bytes its structs carry past that minor, each a canary (C12). The skew is the
//! kit's own, a test of a core rather than a host's behaviour (EmuSen_Backend.md §27.3).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use emusen_core_host::Library;
use emusen_native::core::sys::{CreateParams, DeviceFrame, DeviceLend, Event, FileEntry, FrameInfo, Media, RawFrame, caps, status};

pub use emusen_core_host::exports::{Exports, M};
pub use emusen_core_host::library::text;

/// A loaded library. It is never unloaded while a machine of it may exist, and the kit holds it to the end.
pub struct Lib {
    host: &'static Library,
    pub path: PathBuf,
    pub f: &'static Exports,
    /// The host this kit plays (C12): the minor it says it was built with, and the bytes its structs carry past
    /// the size that minor knows, each filled with `CANARY`. A host before 1.3 passes the 88-byte create params of 1.0.
    host_minor: AtomicU32,
    pad: AtomicUsize,
    /// Canary bytes a core changed, which must stay zero.
    pub canaries_broken: AtomicUsize,
}

/// The byte a skewed host's structs carry past their 1.0 size.
pub const CANARY: u8 = 0xA5;

/// Bytes `buf` holds past `from` that are no longer the canary.
fn broken(buf: &[u8], from: usize) -> usize {
    buf[from..].iter().filter(|&&b| b != CANARY).count()
}

/// Files as the slices a call passes.
pub fn slices(files: &[(u32, Vec<u8>)]) -> Vec<(u32, &[u8])> {
    files.iter().map(|(w, d)| (*w, &d[..])).collect()
}

impl Lib {
    /// The library at `path` with every required export, or why not.
    pub fn open(path: &Path) -> Result<Lib, String> {
        let host = Library::load(path).map_err(|e| e.to_string())?;
        Ok(Lib { host, path: path.to_owned(), f: host.exports(), host_minor: AtomicU32::new(0), pad: AtomicUsize::new(0), canaries_broken: AtomicUsize::new(0) })
    }

    /// Plays a host of minor `minor` whose structs are `pad` bytes longer than version 1.0's (a multiple of 8).
    pub fn set_host(&self, minor: u32, pad: usize) {
        self.host_minor.store(minor, Ordering::Relaxed);
        self.pad.store(pad, Ordering::Relaxed);
    }

    pub fn host(&self) -> (u32, usize) {
        (self.host_minor.load(Ordering::Relaxed), self.pad.load(Ordering::Relaxed))
    }

    /// Whether the library exports `name`.
    pub fn has(&self, name: &str) -> bool {
        self.host.has(name)
    }

    /// The core host's library under the kit's skew.
    pub fn library(&self) -> &'static Library {
        self.host
    }

    pub fn capabilities(&self) -> u64 {
        unsafe { (self.f.capabilities)() }
    }

    pub fn claims(&self, bit: u64) -> bool {
        self.capabilities() & bit == bit
    }

    pub fn info(&self) -> Result<String, i64> {
        text(|o, l| unsafe { (self.f.info)(o, l) })
    }

    pub fn settings(&self) -> Result<String, i64> {
        text(|o, l| unsafe { (self.f.settings_schema)(o, l) })
    }

    pub fn words(&self, code: i32) -> Option<String> {
        text(|o, l| unsafe { (self.f.status_text)(code, o, l) }).ok()
    }

    /// A machine, or the refusal's status and its error text.
    pub fn create(&self, image: &[u8], settings: &str, files: &[(u32, Vec<u8>)], pixel_formats: u64) -> Result<Machine<'_>, (i32, String)> {
        self.create_from(image, settings, &slices(files), pixel_formats, 0)
    }

    /// `create` from slices, with `emusen_create_params.flags` where the host played is of 1.3 or later; an earlier
    /// one has no such field, and passes the struct of 1.0.
    pub fn create_from(&self, image: &[u8], settings: &str, files: &[(u32, &[u8])], pixel_formats: u64, flags: u32) -> Result<Machine<'_>, (i32, String)> {
        self.create_lending(image, settings, files, pixel_formats, flags, std::ptr::null())
    }

    /// `create_from` with a device lent (since 1.9, `DEVICE_PICTURE`), passed by a host of 1.9 or later; an earlier one
    /// has no such field and passes the struct it knew.
    pub fn create_lending(&self, image: &[u8], settings: &str, files: &[(u32, &[u8])], pixel_formats: u64, flags: u32, device: *const DeviceLend) -> Result<Machine<'_>, (i32, String)> {
        self.create_full(image, settings, files, pixel_formats, flags, device, None)
    }

    /// The console `system` started with nothing in it (since 1.11, core info's `starts_empty`): an image of no bytes
    /// and the system named, passed by a host of 1.11 or later; an earlier one has no such field.
    pub fn create_empty(&self, system: &std::ffi::CStr, settings: &str, files: &[(u32, &[u8])]) -> Result<Machine<'_>, (i32, String)> {
        self.create_full(&[], settings, files, 1, 0, std::ptr::null(), Some(system))
    }

    /// A machine of `system`, the system a host routed `image` to (since 1.15), passed by a host of 1.15 or later.
    pub fn create_as(&self, system: &std::ffi::CStr, image: &[u8], settings: &str, files: &[(u32, Vec<u8>)]) -> Result<Machine<'_>, (i32, String)> {
        self.create_full(image, settings, &slices(files), 1, 0, std::ptr::null(), Some(system))
    }

    #[allow(clippy::too_many_arguments)]
    fn create_full(&self, image: &[u8], settings: &str, files: &[(u32, &[u8])], pixel_formats: u64, flags: u32, device: *const DeviceLend, system: Option<&std::ffi::CStr>) -> Result<Machine<'_>, (i32, String)> {
        // A host that names a system to start empty knows 1.11, and one that names it beside an image 1.15, whichever older host the kit otherwise plays.
        let (minor, pad) = self.host();
        let minor = match system {
            Some(_) if image.is_empty() => minor.max(11),
            Some(_) => minor.max(15),
            None => minor,
        };
        let known = if minor >= 11 {
            std::mem::size_of::<CreateParams>()
        } else if minor >= 9 {
            std::mem::offset_of!(CreateParams, system)
        } else if minor >= 3 {
            std::mem::offset_of!(CreateParams, device)
        } else {
            CreateParams::SIZE_1_0
        };
        let element = std::mem::size_of::<FileEntry>() + pad;
        let mut entries = vec![CANARY; element * files.len()];
        for (i, (w, d)) in files.iter().enumerate() {
            let e = FileEntry { size: element as u32, which: *w, data: d.as_ptr(), len: d.len() };
            unsafe { std::ptr::write_unaligned(entries.as_mut_ptr().add(i * element) as *mut FileEntry, e) };
        }
        let mut error = vec![0u8; 512];
        let mut params = vec![CANARY; known + pad];
        let p = CreateParams {
            size: params.len() as u32,
            host_abi_version: (1 << 16) | minor,
            image: image.as_ptr(),
            image_len: image.len(),
            settings: settings.as_ptr(),
            settings_len: settings.len(),
            files: entries.as_ptr() as *const FileEntry,
            file_count: files.len(),
            file_size: element,
            pixel_formats,
            error: error.as_mut_ptr(),
            error_len: error.len(),
            flags,
            device,
            system: system.map_or(std::ptr::null(), |s| s.as_ptr()),
        };
        // SAFETY: `params` holds at least `known` bytes, and `known` is no more than the struct.
        unsafe { std::ptr::copy_nonoverlapping(&p as *const CreateParams as *const u8, params.as_mut_ptr(), known) };
        let made = self.create_raw(params.as_ptr() as *const CreateParams, &mut error);
        let damaged = broken(&params, known) + (0..files.len()).map(|i| broken(&entries[i * element..(i + 1) * element], std::mem::size_of::<FileEntry>())).sum::<usize>();
        self.canaries_broken.fetch_add(damaged, Ordering::Relaxed);
        made
    }

    fn create_raw(&self, p: *const CreateParams, error: &mut [u8]) -> Result<Machine<'_>, (i32, String)> {
        let mut code = 0;
        let h = unsafe { (self.f.create)(p, &mut code) };
        if h.is_null() {
            let end = error.iter().position(|&b| b == 0).unwrap_or(0);
            return Err((code, String::from_utf8_lossy(&error[..end]).into_owned()));
        }
        Ok(Machine::adopt(self, h))
    }

    pub fn create_with(&self, p: &CreateParams, error: &mut [u8]) -> Result<Machine<'_>, (i32, String)> {
        let mut code = 0;
        let h = unsafe { (self.f.create)(p, &mut code) };
        if h.is_null() {
            let end = error.iter().position(|&b| b == 0).unwrap_or(0);
            return Err((code, String::from_utf8_lossy(&error[..end]).into_owned()));
        }
        Ok(Machine::adopt(self, h))
    }
}

/// One machine, owned by the core host's `Machine`, which frees it when dropped; `h` is its handle for the cases' raw calls.
pub struct Machine<'a> {
    pub lib: &'a Lib,
    pub h: M,
    _owner: emusen_core_host::Machine,
}

// SAFETY: a machine is used by one thread at a time, which the ABI permits (§6.16); C10 moves one to another thread.
unsafe impl Send for Machine<'_> {}

impl<'a> Machine<'a> {
    fn adopt(lib: &'a Lib, h: M) -> Machine<'a> {
        // SAFETY: `h` is a live machine `lib`'s create just returned, and nothing else frees it.
        let owner = unsafe { emusen_core_host::Machine::adopt(lib.host, h) }.expect("a create that returned a handle");
        Machine { lib, h, _owner: owner }
    }

    pub fn advance(&self) -> Result<(), i32> {
        let mut detail = 0u64;
        match unsafe { (self.lib.f.advance)(self.h, &mut detail) } {
            0 => Ok(()),
            e => Err(e),
        }
    }

    pub fn present(&self) {
        if let Some(p) = self.lib.f.present {
            unsafe { p(self.h) };
        }
    }

    pub fn set_options(&self, flags: u32) -> i32 {
        unsafe { (self.lib.f.set_options)(self.h, flags) }
    }

    pub fn frame_info(&self) -> Result<FrameInfo, i32> {
        let (_, pad) = self.lib.host();
        let known = std::mem::size_of::<FrameInfo>();
        let mut words = vec![u64::from_ne_bytes([CANARY; 8]); (known + pad).div_ceil(8)];
        let buf = unsafe { std::slice::from_raw_parts_mut(words.as_mut_ptr() as *mut u8, known + pad) };
        buf[..known].fill(0);
        buf[..4].copy_from_slice(&((known + pad) as u32).to_ne_bytes());
        let r = unsafe { (self.lib.f.frame_info)(self.h, buf.as_mut_ptr() as *mut FrameInfo) };
        self.lib.canaries_broken.fetch_add(broken(buf, known), Ordering::Relaxed);
        match r {
            0 => Ok(unsafe { std::ptr::read(buf.as_ptr() as *const FrameInfo) }),
            e => Err(e),
        }
    }

    /// `frame_device` (since 1.9): `None` where the export is absent; the picture's place when it is on the device.
    pub fn frame_device(&self) -> Option<Result<Option<DeviceFrame>, i32>> {
        let f = self.lib.f.frame_device?;
        let mut out = DeviceFrame { size: std::mem::size_of::<DeviceFrame>() as u32, ..DeviceFrame::default() };
        Some(match unsafe { f(self.h, &mut out) } {
            1 => Ok(Some(out)),
            0 => Ok(None),
            e => Err(e),
        })
    }

    /// `raw_select` (since 1.10): `NOT_SUPPORTED` where the library lacks it.
    pub fn raw_select(&self, mask: u64) -> i32 {
        self.lib.f.raw_select.map_or(status::NOT_SUPPORTED, |f| unsafe { f(self.h, mask) })
    }

    /// `raw_info` with a struct of `size` bytes: its answer and the shape, the struct's fields past `size` zeroed first.
    pub fn raw_info_sized(&self, raw: u32, size: u32) -> (i32, RawFrame) {
        let Some(f) = self.lib.f.raw_info else { return (status::NOT_SUPPORTED, RawFrame::default()) };
        let mut out = RawFrame { size, ..RawFrame::default() };
        let r = unsafe { f(self.h, raw, &mut out) };
        (r, out)
    }

    pub fn raw_info(&self, raw: u32) -> (i32, RawFrame) {
        self.raw_info_sized(raw, std::mem::size_of::<RawFrame>() as u32)
    }

    /// `raw_copy`'s values by the length-query idiom, or its status.
    pub fn raw_copy(&self, raw: u32) -> Result<Vec<u8>, i64> {
        let Some(f) = self.lib.f.raw_copy else { return Err(status::NOT_SUPPORTED as i64) };
        let n = unsafe { f(self.h, raw, std::ptr::null_mut(), 0) };
        if n < 0 {
            return Err(n);
        }
        let mut buf = vec![0u8; n as usize];
        let m = unsafe { f(self.h, raw, buf.as_mut_ptr(), buf.len()) };
        if m != n {
            return Err(m.min(-1));
        }
        Ok(buf)
    }

    pub fn frame(&self) -> Vec<u8> {
        let n = unsafe { (self.lib.f.frame_copy)(self.h, std::ptr::null_mut(), 0) }.max(0) as usize;
        let mut buf = vec![0u8; n];
        unsafe { (self.lib.f.frame_copy)(self.h, buf.as_mut_ptr(), n) };
        buf
    }

    /// Every buffered sample, drain by drain, each with the rate the drain reported.
    pub fn drain_audio(&self) -> Vec<(Vec<i16>, i32)> {
        let mut out = Vec::new();
        for _ in 0..64 {
            let n = unsafe { (self.lib.f.audio_buffered)(self.h) }.max(0) as usize;
            if n < 2 {
                break;
            }
            let mut buf = vec![0i16; n];
            let mut rate = 0;
            let got = unsafe { (self.lib.f.audio_drain)(self.h, buf.as_mut_ptr(), n, (n / 2) as i64, &mut rate) };
            if got <= 0 {
                break;
            }
            buf.truncate(got as usize);
            out.push((buf, rate));
        }
        out
    }

    pub fn events(&self) -> Vec<Event> {
        let (_, pad) = self.lib.host();
        let known = std::mem::size_of::<Event>();
        let element = known + pad;
        let n = unsafe { (self.lib.f.events)(self.h, std::ptr::null_mut(), 0, 0) }.max(0) as usize;
        let mut words = vec![u64::from_ne_bytes([CANARY; 8]); (element * n).div_ceil(8)];
        let buf = unsafe { std::slice::from_raw_parts_mut(words.as_mut_ptr() as *mut u8, element * n) };
        for i in 0..n {
            buf[i * element..i * element + known].fill(0);
            buf[i * element..i * element + 4].copy_from_slice(&(element as u32).to_ne_bytes());
        }
        let got = unsafe { (self.lib.f.events)(self.h, buf.as_mut_ptr() as *mut Event, n, element) }.max(0) as usize;
        let damaged: usize = (0..n).map(|i| broken(&buf[i * element..(i + 1) * element], known)).sum();
        self.lib.canaries_broken.fetch_add(damaged, Ordering::Relaxed);
        (0..got.min(n)).map(|i| unsafe { std::ptr::read_unaligned(buf.as_ptr().add(i * element) as *const Event) }).collect()
    }

    /// `debug_registers` by the length-query idiom (a processor's or, since 1.13, a unit's), or its status.
    pub fn registers(&self, id: u32) -> Result<Vec<i64>, i64> {
        let Some(f) = self.lib.f.debug_registers else { return Err(status::NOT_SUPPORTED as i64) };
        let n = unsafe { f(self.h, id, std::ptr::null_mut(), 0) };
        if n < 0 {
            return Err(n);
        }
        let mut out = vec![0i64; n as usize];
        let m = unsafe { f(self.h, id, out.as_mut_ptr(), out.len()) };
        if m != n {
            return Err(m.min(-1));
        }
        Ok(out)
    }

    /// `telemetry` (since 1.13): the count it answers to a null buffer, and the values it writes into one that holds them.
    pub fn telemetry(&self) -> Result<(i64, Vec<i64>), i64> {
        let Some(f) = self.lib.f.telemetry else { return Err(status::NOT_SUPPORTED as i64) };
        let n = unsafe { f(self.h, std::ptr::null_mut(), 0) };
        if n < 0 {
            return Err(n);
        }
        let mut out = vec![0i64; n as usize];
        let m = unsafe { f(self.h, out.as_mut_ptr(), out.len()) };
        if m < 0 {
            return Err(m);
        }
        Ok((m, out))
    }

    /// `set_present` (since 1.13): `NOT_SUPPORTED` where the library lacks it.
    pub fn set_present(&self, port: u32, present: u32) -> i32 {
        self.lib.f.set_present.map_or(status::NOT_SUPPORTED, |f| unsafe { f(self.h, port, present) })
    }

    /// `debug_translate` (since 1.13): its answer and the bus address it wrote.
    pub fn translate(&self, processor: u32, address: u32) -> (i32, u32) {
        let Some(f) = self.lib.f.debug_translate else { return (status::NOT_SUPPORTED, 0) };
        let mut bus = 0u32;
        let r = unsafe { f(self.h, processor, address, &mut bus) };
        (r, bus)
    }

    pub fn space_write(&self, space: u32, address: u32, value: u8) -> i64 {
        unsafe { (self.lib.f.space_write)(self.h, space, address, &value, 1) }
    }

    pub fn space_byte(&self, space: u32, address: u32) -> u8 {
        let mut b = 0u8;
        unsafe { (self.lib.f.space_read)(self.h, space, address, &mut b, 1) };
        b
    }

    pub fn set_buttons(&self, port: u32, mask: u32, changed: u32) -> i32 {
        unsafe { (self.lib.f.set_buttons)(self.h, port, mask, changed) }
    }

    pub fn state_size(&self, kind: u32) -> i64 {
        unsafe { (self.lib.f.state_size)(self.h, kind) }
    }

    pub fn save(&self, kind: u32) -> Result<Vec<u8>, i64> {
        let n = self.state_size(kind);
        if n < 0 {
            return Err(n);
        }
        let mut buf = vec![0u8; n as usize];
        let got = unsafe { (self.lib.f.state_save)(self.h, kind, buf.as_mut_ptr(), buf.len()) };
        if got < 0 {
            return Err(got);
        }
        buf.truncate(got as usize);
        Ok(buf)
    }

    pub fn load(&self, state: &[u8]) -> i32 {
        unsafe { (self.lib.f.state_load)(self.h, state.as_ptr(), state.len()) }
    }

    pub fn machine_info(&self) -> Result<String, i64> {
        text(|o, l| unsafe { (self.lib.f.machine_info)(self.h, o, l) })
    }

    pub fn last_error(&self) -> String {
        text(|o, l| unsafe { (self.lib.f.last_error)(self.h, o, l) }).unwrap_or_default()
    }

    pub fn has_snapshot(&self) -> bool {
        self.lib.claims(caps::SNAPSHOT)
    }

    /// MEDIA's lid, open or closed; `NOT_SUPPORTED` where the library lacks it.
    pub fn media_lid(&self, drive: u32, open: bool) -> i32 {
        self.lib.f.media_lid.map_or(status::NOT_SUPPORTED, |f| unsafe { f(self.h, drive, open as u32) })
    }

    /// MEDIA's insert of `image` with its files, or of nothing for `None`.
    pub fn media_insert(&self, drive: u32, image: Option<&[u8]>, files: &[(u32, Vec<u8>)], flags: u32) -> i32 {
        self.media_insert_from(drive, image, &slices(files), flags)
    }

    /// `media_insert` from slices.
    pub fn media_insert_from(&self, drive: u32, image: Option<&[u8]>, files: &[(u32, &[u8])], flags: u32) -> i32 {
        let Some(f) = self.lib.f.media_insert else { return status::NOT_SUPPORTED };
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
        unsafe { f(self.h, drive, &m) }
    }
}
