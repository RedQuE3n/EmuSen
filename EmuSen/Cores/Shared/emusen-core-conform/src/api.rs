//! A library loaded through the core ABI v1 and nothing else: every export by name, typed as `emusen_core.h` types it.

use std::ffi::c_char;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use emusen_native::core::sys::{CreateParams, Event, FileEntry, FrameInfo, Machine as RawMachine, caps, status};

type M = *mut RawMachine;
type C = *const RawMachine;

macro_rules! table {
    (required { $($r:ident : fn($($ra:ty),*) -> $rr:ty),* $(,)? } optional { $($o:ident : fn($($oa:ty),*) -> $or:ty),* $(,)? }) => {
        /// Every export, the optional ones `None` where the library lacks them.
        #[allow(non_snake_case)]
        pub struct Fns {
            $( pub $r: unsafe extern "C" fn($($ra),*) -> $rr, )*
            $( pub $o: Option<unsafe extern "C" fn($($oa),*) -> $or>, )*
        }

        impl Fns {
            /// The table, or the name of the first required export missing.
            unsafe fn resolve(lib: &libloading::Library) -> Result<Fns, String> {
                Ok(Fns {
                    $( $r: unsafe { *lib.get::<unsafe extern "C" fn($($ra),*) -> $rr>(concat!("emusen_core_", stringify!($r), "\0").as_bytes()).map_err(|_| concat!("emusen_core_", stringify!($r)).to_owned())? }, )*
                    $( $o: unsafe { lib.get::<unsafe extern "C" fn($($oa),*) -> $or>(concat!("emusen_core_", stringify!($o), "\0").as_bytes()).ok().map(|s| *s) }, )*
                })
            }
        }
    };
}

table! {
    required {
        abi_version: fn() -> u32,
        capabilities: fn() -> u64,
        info: fn(*mut u8, usize) -> i64,
        settings_schema: fn(*mut u8, usize) -> i64,
        firmware_for: fn(*const u8, usize, *mut u8, usize) -> i64,
        status_text: fn(i32, *mut u8, usize) -> i64,
        set_crash_log: fn(*const c_char) -> i32,
        log_drain: fn(M, *mut u8, usize) -> i64,
        create: fn(*const CreateParams, *mut i32) -> M,
        free: fn(M) -> i32,
        machine_info: fn(C, *mut u8, usize) -> i64,
        last_error: fn(C, *mut u8, usize) -> i64,
        advance: fn(M, *mut u64) -> i32,
        set_options: fn(M, u32) -> i32,
        frame_count: fn(C) -> i64,
        events: fn(M, *mut Event, usize, usize) -> i64,
        frame_info: fn(C, *mut FrameInfo) -> i32,
        frame_copy: fn(C, *mut u8, usize) -> i64,
        audio_rate: fn(C) -> i32,
        audio_buffered: fn(C) -> i64,
        audio_drain: fn(M, *mut i16, usize, i64, *mut i32) -> i64,
        set_audio_limit: fn(M, u64) -> i32,
        set_buttons: fn(M, u32, u32, u32) -> i32,
        state_size: fn(C, u32) -> i64,
        state_save: fn(M, u32, *mut u8, usize) -> i64,
        state_load: fn(M, *const u8, usize) -> i32,
        state_layout: fn(C, u32, *mut u8, usize) -> i64,
        space_size: fn(C, u32) -> i64,
        space_read: fn(M, u32, u32, *mut u8, usize) -> i64,
        space_write: fn(M, u32, u32, *const u8, usize) -> i64,
        battery: fn(C, u32, *mut u8, usize, *mut u32) -> i64,
        battery_saved: fn(M, u32) -> i32,
    }
    optional {
        reset: fn(M) -> i32,
        present: fn(M) -> i32,
        phases: fn(C, *mut i64, usize) -> i64,
        audio_peek: fn(C, *mut i16, usize) -> i64,
        set_mutes: fn(M, u32) -> i32,
        set_axis: fn(M, u32, u32, f64) -> i32,
        set_rom_patches: fn(M, *const u32, usize) -> i64,
        set_cheat_pokes: fn(M, *const u32, usize) -> i64,
        set_settings: fn(M, *const u8, usize) -> i32,
        setting_notes: fn(C, *mut u8, usize) -> i64,
        debug_set: fn(M, u32, i32, i32) -> i32,
        debug_set_stack: fn(M, *const u32, usize) -> i32,
        debug_set_breakpoints: fn(M, u32, *const i32, usize) -> i32,
        debug_set_ranges: fn(M, u32, *const u32, usize) -> i32,
        debug_run_frame: fn(M, u32, *mut u32, *mut u64, *mut u64) -> i32,
        debug_writes: fn(M, *mut u32, usize) -> i64,
        debug_calls: fn(M, *mut u32, usize) -> i64,
        debug_profile: fn(M, *mut i64, usize) -> i64,
        debug_coverage: fn(M, u32, *mut u8, usize, *mut i64) -> i64,
        debug_counters: fn(M, *mut i64, usize) -> i64,
        debug_pc: fn(C, u32, *mut u64) -> i32,
        debug_registers: fn(C, u32, *mut i64, usize) -> i64,
        debug_disassemble: fn(M, u32, u32, u32, u32, *mut u8, usize) -> i64,
    }
}

/// A loaded library. It is never unloaded while a machine of it may exist, and the kit holds it to the end.
pub struct Lib {
    lib: libloading::Library,
    pub path: PathBuf,
    pub f: Fns,
    /// The host this kit plays (C12): the minor it says it was built with, and the bytes its structs carry past
    /// version 1.0's, each filled with `CANARY`.
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

/// Reads a text the length-query way.
pub fn text(call: impl Fn(*mut u8, usize) -> i64) -> Result<String, i64> {
    let n = call(std::ptr::null_mut(), 0);
    if n < 0 {
        return Err(n);
    }
    let mut buf = vec![0u8; n as usize];
    let m = call(buf.as_mut_ptr(), buf.len());
    if m != n {
        return Err(m.min(-1));
    }
    String::from_utf8(buf).map_err(|_| status::BAD_STRING as i64)
}

impl Lib {
    /// The library at `path` with every required export, or why not.
    pub fn open(path: &Path) -> Result<Lib, String> {
        let lib = unsafe { libloading::Library::new(path) }.map_err(|e| format!("could not be loaded: {e}"))?;
        let f = unsafe { Fns::resolve(&lib) }.map_err(|name| format!("lacks the required export {name}"))?;
        Ok(Lib { lib, path: path.to_owned(), f, host_minor: AtomicU32::new(0), pad: AtomicUsize::new(0), canaries_broken: AtomicUsize::new(0) })
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
        let mut n = name.as_bytes().to_vec();
        n.push(0);
        unsafe { self.lib.get::<*const ()>(&n) }.is_ok()
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
        let (minor, pad) = self.host();
        let element = std::mem::size_of::<FileEntry>() + pad;
        let mut entries = vec![CANARY; element * files.len()];
        for (i, (w, d)) in files.iter().enumerate() {
            let e = FileEntry { size: element as u32, which: *w, data: d.as_ptr(), len: d.len() };
            unsafe { std::ptr::write_unaligned(entries.as_mut_ptr().add(i * element) as *mut FileEntry, e) };
        }
        let mut error = vec![0u8; 512];
        let mut params = vec![CANARY; std::mem::size_of::<CreateParams>() + pad];
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
        };
        unsafe { std::ptr::write_unaligned(params.as_mut_ptr() as *mut CreateParams, p) };
        let made = self.create_raw(params.as_ptr() as *const CreateParams, &mut error);
        let damaged = broken(&params, std::mem::size_of::<CreateParams>()) + (0..files.len()).map(|i| broken(&entries[i * element..(i + 1) * element], std::mem::size_of::<FileEntry>())).sum::<usize>();
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
        Ok(Machine { lib: self, h })
    }

    pub fn create_with(&self, p: &CreateParams, error: &mut [u8]) -> Result<Machine<'_>, (i32, String)> {
        let mut code = 0;
        let h = unsafe { (self.f.create)(p, &mut code) };
        if h.is_null() {
            let end = error.iter().position(|&b| b == 0).unwrap_or(0);
            return Err((code, String::from_utf8_lossy(&error[..end]).into_owned()));
        }
        Ok(Machine { lib: self, h })
    }
}

/// One machine, freed when dropped.
pub struct Machine<'a> {
    pub lib: &'a Lib,
    pub h: M,
}

// SAFETY: a machine is used by one thread at a time, which the ABI permits (§6.16); C10 moves one to another thread.
unsafe impl Send for Machine<'_> {}

impl Machine<'_> {
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
}

impl Drop for Machine<'_> {
    fn drop(&mut self) {
        unsafe { (self.lib.f.free)(self.h) };
    }
}
