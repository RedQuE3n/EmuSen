//! The common native interface: the trait a Rust core implements, and the macro that exports it under the fixed
//! `emusen_native_*` names. See EmuSen_NativeCores.md §3.
//!
//! Only as much is built as MoonRT needs (§7 step 2). The optional groups present, phases, audio peek, axes, settings
//! and the debug exports are not generated yet. They come with the cores that need them.

use std::ffi::{CStr, c_char};
use std::sync::OnceLock;

use crate::ffi::StateMachine;

/// This interface's own number, the high half of `emusen_native_interface_version`.
pub const COMMON_VERSION: u16 = 1;

/// `(COMMON << 16) | CORE`, which the host matches exactly on both halves (§3.2, §9 Q3).
pub const fn version(core: u16) -> u32 {
    ((COMMON_VERSION as u32) << 16) | core as u32
}

/// The capability bits (§3.2). A library claims exactly the optional exports it has.
pub mod caps {
    pub const RESET: u64 = 1 << 0;
    pub const PRESENT: u64 = 1 << 1;
    pub const SNAPSHOT: u64 = 1 << 2;
    pub const AXES: u64 = 1 << 3;
    pub const AUDIO_PEEK: u64 = 1 << 4;
    pub const MUTES: u64 = 1 << 5;
    pub const SETTINGS: u64 = 1 << 6;
    pub const PHASES: u64 = 1 << 7;
    pub const FRAME_SERIAL: u64 = 1 << 8;
    pub const ROW_REPEAT: u64 = 1 << 9;
    pub const BATTERY_DIRTY: u64 = 1 << 10;
    pub const ROM_PATCHES: u64 = 1 << 11;
    pub const DEBUG: u64 = 1 << 12;
    pub const DEBUG_STACK: u64 = 1 << 13;
}

/// The interface's own status codes, -256 to -319, and the band of reproduced C# exceptions below them (§3.3).
pub mod status {
    pub const NOT_SUPPORTED: i32 = -256;
    pub const NO_SUCH_SPACE: i32 = -257;
    pub const READ_ONLY: i32 = -258;
    pub const UNKNOWN_SETTING: i32 = -259;
    pub const BAD_SETTING: i32 = -260;
    pub const NO_SUCH_PORT: i32 = -261;
    pub const BAD_FILE: i32 = -262;
    /// A C# exception reproduced: this less its kind.
    pub const FAULT_BASE: i32 = -320;
}

/// The .NET exception types a port reproduces, as kinds; the status is `status::FAULT_BASE - kind`.
pub mod fault {
    pub const INDEX_OUT_OF_RANGE: i32 = 1;
    pub const DIVIDE_BY_ZERO: i32 = 2;
    pub const ARGUMENT_OUT_OF_RANGE: i32 = 3;
    pub const INVALID_OPERATION: i32 = 4;
    pub const OVERFLOW: i32 = 5;

    pub const fn status(kind: i32) -> i32 {
        super::status::FAULT_BASE - kind
    }
}

/// The television standard a machine runs to; for a later libretro adapter's AV info (§9 Q8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Region {
    Ntsc = 0,
    Pal = 1,
}

/// A file the host read for this game, as `emusen_native_create` receives it: `which` 0 is the battery save.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NativeFile {
    pub which: u32,
    pub data: *const u8,
    pub len: usize,
}

/// One of those files, as the core sees it.
#[derive(Clone, Copy, Debug)]
pub struct File<'a> {
    pub which: u32,
    pub data: &'a [u8],
}

/// The picture's shape: RGBA8888, `bytes` long; without `FRAME_SERIAL` the serial is the frame count (§3.6).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameInfo {
    pub width: i32,
    pub height: i32,
    pub row_repeat: i32,
    pub flags: u32,
    pub serial: i64,
    pub bytes: i64,
}

/// The create-time settings, one `key=value` line each (§3.4).
#[derive(Clone, Debug, Default)]
pub struct Settings {
    pairs: Vec<(String, String)>,
}

impl Settings {
    /// The lines of `text`; a line without `=` is `BAD_SETTING`.
    pub fn parse(text: &[u8]) -> Result<Settings, i32> {
        let text = std::str::from_utf8(text).map_err(|_| status::BAD_SETTING)?;
        let mut pairs = Vec::new();
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            let (key, value) = line.split_once('=').ok_or(status::BAD_SETTING)?;
            pairs.push((key.trim().to_owned(), value.trim().to_owned()));
        }
        Ok(Settings { pairs })
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.pairs.iter().map(|(k, _)| k.as_str())
    }
}

/// What a Rust core implements; `native_exports!` turns it into the C ABI. Every `Err` is a status (§3.3).
///
/// The state size for kind 0 must stay the same for a machine's whole life, as a libretro adapter's
/// `retro_serialize_size` requires (§9 Q8); `create` fixes it.
pub trait NativeCore: Sized + StateMachine {
    /// The core's half of the version: its extension exports, its status band and its settings keys.
    const CORE_VERSION: u16;
    const CAPABILITIES: u64;
    /// The engine's name in its crash log, "MoonRT".
    const ENGINE: &'static str;

    fn create(image: &[u8], settings: &Settings, files: &[File<'_>]) -> Result<Self, i32>;

    /// `RESET`: the console's reset button.
    fn reset(&mut self) -> Result<(), i32> {
        Err(status::NOT_SUPPORTED)
    }

    fn region(&self) -> Region {
        Region::Ntsc
    }

    /// The machine to the frame's end; `detail` is the core's word for a failure.
    fn advance(&mut self, detail: &mut u64) -> Result<(), i32>;

    /// Bit 0 skips rendering; bits 1-23 are the core's.
    fn set_options(&mut self, flags: u32);

    fn frame_count(&self) -> i64;
    fn frame_info(&self) -> FrameInfo;
    fn frame(&self) -> &[u8];

    /// Bits set in `changed` take `mask`'s values; a port the core lacks is ignored, as C#'s `SetButton` ignores it.
    fn set_buttons(&mut self, port: u32, mask: u32, changed: u32) -> Result<(), i32>;

    fn audio_rate(&self) -> i32;
    fn audio_buffered(&self) -> usize;
    fn drain_audio(&mut self, out: &mut [i16], max_frames: usize) -> usize;
    /// `AudioSettings.AudioBufferMaxSamples`, the most samples the queue holds.
    fn set_audio_limit(&mut self, samples: usize);

    /// `MUTES`: bit n mutes channel n.
    fn set_mutes(&mut self, _mask: u32) -> Result<(), i32> {
        Err(status::NOT_SUPPORTED)
    }

    fn space_size(&self, space: u32) -> Result<i64, i32>;
    fn space_read(&mut self, space: u32, address: u32, out: &mut [u8]) -> Result<(), i32>;
    fn space_write(&mut self, space: u32, address: u32, data: &[u8]) -> Result<(), i32>;

    /// A battery file's bytes and flags: bit 0 changed, bit 1 tracked. Empty for a cartridge without one.
    fn battery(&self, which: u32) -> Result<(&[u8], u32), i32>;

    fn battery_saved(&mut self, _which: u32) -> Result<(), i32> {
        Ok(())
    }

    /// `ROM_PATCHES`: `CheatRegistry.ResolveRomPatches`' list, as (address, value, compare) triples with `u32::MAX` for none.
    fn set_rom_patches(&mut self, _triples: &[u32]) -> Result<(), i32> {
        Err(status::NOT_SUPPORTED)
    }

    /// `DEBUG`: the hooks the host's tables are pushed into and its logs drained from.
    fn debug_hooks(&mut self) -> Option<&mut crate::debug::Hooks> {
        None
    }

    /// `DEBUG`: the frame through the core's observed loop; the stop reasons (`debug::stop`), zero at the frame's end.
    fn debug_run_frame(&mut self, _flags: u32, _detail: &mut u64) -> Result<u32, i32> {
        Err(status::NOT_SUPPORTED)
    }

    /// `DEBUG`: the address of the step processor `processor` stands in front of.
    fn debug_pc(&self, _processor: u32) -> Option<u64> {
        None
    }

    /// Clears whatever the core records between calls, so that nothing outlives the call that made it.
    fn end_call() {}
}

static CRASH_LOG: OnceLock<String> = OnceLock::new();

/// Where a panic is written before the process aborts; the first path given stands.
///
/// # Safety
/// `path` must be a valid NUL-terminated UTF-8 string, or null.
pub unsafe fn install_crash_log(engine: &'static str, path: *const c_char) -> i32 {
    if path.is_null() {
        return crate::ffi::status::NULL;
    }
    let path = unsafe { CStr::from_ptr(path) }.to_string_lossy().into_owned();
    if CRASH_LOG.set(path).is_ok() {
        std::panic::set_hook(Box::new(move |info| {
            let text = format!("{engine} panic: {info}\n{}\n", std::backtrace::Backtrace::force_capture());
            if let Some(path) = CRASH_LOG.get() {
                let _ = std::fs::write(path, &text);
            }
            eprintln!("{text}");
        }));
    }
    0
}

/// The capability bit an optional group of `native_exports!` claims.
#[macro_export]
#[doc(hidden)]
macro_rules! __native_capability {
    (reset) => {
        $crate::abi::caps::RESET
    };
    (mutes) => {
        $crate::abi::caps::MUTES
    };
    (rom_patches) => {
        $crate::abi::caps::ROM_PATCHES
    };
    (debug) => {
        $crate::abi::caps::DEBUG
    };
    (debug_stack) => {
        $crate::abi::caps::DEBUG_STACK
    };
}

/// One optional group's exports.
#[macro_export]
#[doc(hidden)]
macro_rules! __native_optional {
    ($t:ty, reset) => {
        /// # Safety
        /// `handle` must be live or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_reset(handle: *mut $t) -> i32 {
            let Some(m) = (unsafe { handle.as_mut() }) else { return $crate::ffi::status::NULL };
            let r = <$t as $crate::abi::NativeCore>::reset(m).map_or_else(|e| e, |()| 0);
            <$t as $crate::abi::NativeCore>::end_call();
            r
        }
    };
    ($t:ty, mutes) => {
        /// # Safety
        /// `handle` must be live or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_set_mutes(handle: *mut $t, mask: u32) -> i32 {
            let Some(m) = (unsafe { handle.as_mut() }) else { return $crate::ffi::status::NULL };
            <$t as $crate::abi::NativeCore>::set_mutes(m, mask).map_or_else(|e| e, |()| 0)
        }
    };
    ($t:ty, debug_stack) => {
        /// The registry's call stack as (source, target) pairs, innermost last (`DEBUG_STACK`).
        ///
        /// # Safety
        /// `handle` must be live or null; `pairs` valid for `2 * count` values, or null with `count` zero.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_debug_set_stack(handle: *mut $t, pairs: *const u32, count: usize) -> i32 {
            match $crate::abi::hooks_of(handle) {
                Ok(h) => {
                    let values = unsafe { $crate::abi::slice(pairs, 2 * count) };
                    let pairs: Vec<(u32, u32)> = values.chunks_exact(2).map(|p| (p[0], p[1])).collect();
                    h.set_stack(&pairs);
                    0
                }
                Err(e) => e,
            }
        }
    };
    ($t:ty, debug) => {
        /// The flags (`debug::flag`), the depth `step over`/`out` stop at (`i32::MIN` unarmed) and the guard (`-1` unarmed).
        ///
        /// # Safety
        /// `handle` must be live or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_debug_set(handle: *mut $t, flags: u32, depth_target: i32, depth_guard: i32) -> i32 {
            match $crate::abi::hooks_of(handle) {
                Ok(h) => {
                    h.configure(flags, depth_target, depth_guard);
                    0
                }
                Err(e) => e,
            }
        }

        /// The enabled breakpoints as `count` pairs of first and last address.
        ///
        /// # Safety
        /// `handle` must be live or null; `pairs` valid for `2 * count` values, or null with `count` zero.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_debug_set_breakpoints(handle: *mut $t, pairs: *const i32, count: usize) -> i32 {
            match $crate::abi::hooks_of(handle) {
                Ok(h) => {
                    h.set_breakpoints(unsafe { $crate::abi::slice(pairs, 2 * count) });
                    0
                }
                Err(e) => e,
            }
        }

        /// The stores to report: `count` triples of space, first and last offset; `kind` 0 the watches, 1 the data breakpoints.
        ///
        /// # Safety
        /// `handle` must be live or null; `triples` valid for `3 * count` values, or null with `count` zero.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_debug_set_ranges(handle: *mut $t, kind: u32, triples: *const u32, count: usize) -> i32 {
            match $crate::abi::hooks_of(handle) {
                Ok(h) => {
                    h.set_ranges(kind, unsafe { $crate::abi::slice(triples, 3 * count) });
                    0
                }
                Err(e) => e,
            }
        }

        /// The observed frame: its stop reasons, zero at the frame's end, with processor 0's program counter in `pc`.
        ///
        /// # Safety
        /// `handle` must be live or null; `pc` and `detail` writable or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_debug_run_frame(handle: *mut $t, flags: u32, pc: *mut u64, detail: *mut u64) -> i32 {
            let Some(m) = (unsafe { handle.as_mut() }) else { return $crate::ffi::status::NULL };
            let mut word = 0u64;
            let r = <$t as $crate::abi::NativeCore>::debug_run_frame(m, flags, &mut word);
            <$t as $crate::abi::NativeCore>::end_call();
            if let Some(p) = unsafe { pc.as_mut() } {
                *p = <$t as $crate::abi::NativeCore>::debug_pc(m, 0).unwrap_or(0);
            }
            if let Some(d) = unsafe { detail.as_mut() } {
                *d = word;
            }
            match r {
                Ok(why) => why as i32,
                Err(e) => e,
            }
        }

        /// The stores logged, four values each (space, offset, value, pc), copied and forgotten when they all fit; the count.
        ///
        /// # Safety
        /// `handle` must be live or null; `out` valid for `len` values, or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_debug_writes(handle: *mut $t, out: *mut u32, len: usize) -> i64 {
            match $crate::abi::hooks_of(handle) {
                Ok(h) => $crate::debug::drain(&mut h.writes_log, unsafe { $crate::abi::slice_mut(out, len) }, |w: &$crate::debug::Write| [w.space, w.address, w.value, w.pc]) as i64,
                Err(e) => e as i64,
            }
        }

        /// The calls, interrupts and returns logged, three values each (kind, source, target), drained as the writes are.
        ///
        /// # Safety
        /// `handle` must be live or null; `out` valid for `len` values, or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_debug_calls(handle: *mut $t, out: *mut u32, len: usize) -> i64 {
            match $crate::abi::hooks_of(handle) {
                Ok(h) => $crate::debug::drain(&mut h.calls_log, unsafe { $crate::abi::slice_mut(out, len) }, |c: &$crate::debug::Call| [c.kind, c.source, c.target]) as i64,
                Err(e) => e as i64,
            }
        }

        /// The profile's runs, (owner, instructions) pairs, drained as the logs are; the pair count.
        ///
        /// # Safety
        /// `handle` must be live or null; `out` valid for `len` values, or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_debug_profile(handle: *mut $t, out: *mut i64, len: usize) -> i64 {
            match $crate::abi::hooks_of(handle) {
                Ok(h) => $crate::debug::drain_profile(h, unsafe { $crate::abi::slice_mut(out, len) }) as i64,
                Err(e) => e as i64,
            }
        }

        /// A processor's coverage bitmap, copied and cleared when `len` holds it, with its recorded steps; its length, zero unarmed.
        ///
        /// # Safety
        /// `handle` must be live or null; `out` valid for `len` bytes, or null; `recorded` writable or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_debug_coverage(handle: *mut $t, processor: u32, out: *mut u8, len: usize, recorded: *mut i64) -> i64 {
            match $crate::abi::hooks_of(handle) {
                Ok(h) => {
                    let mut n = 0i64;
                    let r = $crate::debug::drain_coverage(h, processor as usize, unsafe { $crate::abi::slice_mut(out, len) }, &mut n);
                    if let Some(p) = unsafe { recorded.as_mut() } {
                        *p = n;
                    }
                    r as i64
                }
                Err(e) => e as i64,
            }
        }

        /// The depth and the unmatched returns, copied up to `len`; how many there are.
        ///
        /// # Safety
        /// `handle` must be live or null; `out` valid for `len` values, or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_debug_counters(handle: *mut $t, out: *mut i64, len: usize) -> i64 {
            match $crate::abi::hooks_of(handle) {
                Ok(h) => {
                    let values = h.counters();
                    let out = unsafe { $crate::abi::slice_mut(out, len) };
                    let n = values.len().min(out.len());
                    out[..n].copy_from_slice(&values[..n]);
                    values.len() as i64
                }
                Err(e) => e as i64,
            }
        }

        /// # Safety
        /// `handle` must be live or null; `pc` writable or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_debug_pc(handle: *const $t, processor: u32, pc: *mut u64) -> i32 {
            let Some(m) = (unsafe { handle.as_ref() }) else { return $crate::ffi::status::NULL };
            match <$t as $crate::abi::NativeCore>::debug_pc(m, processor) {
                Some(value) => {
                    if let Some(p) = unsafe { pc.as_mut() } {
                        *p = value;
                    }
                    0
                }
                None => $crate::abi::status::NOT_SUPPORTED,
            }
        }
    };
    ($t:ty, rom_patches) => {
        /// `count` triples of (address, value, compare), `u32::MAX` for no compare; zero clears them. Returns the triples taken.
        ///
        /// # Safety
        /// `handle` must be live or null; `words` valid for `count * 3` values, or null when `count` is zero.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_set_rom_patches(handle: *mut $t, words: *const u32, count: usize) -> i64 {
            let Some(m) = (unsafe { handle.as_mut() }) else { return $crate::ffi::status::NULL as i64 };
            let words: &[u32] = if count == 0 || words.is_null() { &[] } else { unsafe { std::slice::from_raw_parts(words, count * 3) } };
            match <$t as $crate::abi::NativeCore>::set_rom_patches(m, words) {
                Ok(()) => count as i64,
                Err(e) => e as i64,
            }
        }
    };
}

/// The hooks of a live handle, or the status to return: null, or a core without `DEBUG`.
#[doc(hidden)]
pub fn hooks_of<'a, T: NativeCore + 'a>(handle: *mut T) -> Result<&'a mut crate::debug::Hooks, i32> {
    // SAFETY: the export's caller promises `handle` is live or null.
    let m: &'a mut T = unsafe { handle.as_mut() }.ok_or(crate::ffi::status::NULL)?;
    m.debug_hooks().ok_or(status::NOT_SUPPORTED)
}

/// A caller's buffer as a slice, empty when null.
///
/// # Safety
/// `data` valid for `len` values, or null.
#[doc(hidden)]
pub unsafe fn slice_mut<'a, T>(data: *mut T, len: usize) -> &'a mut [T] {
    if data.is_null() || len == 0 { &mut [] } else { unsafe { std::slice::from_raw_parts_mut(data, len) } }
}

/// # Safety
/// `data` valid for `len` values, or null.
#[doc(hidden)]
pub unsafe fn slice<'a, T>(data: *const T, len: usize) -> &'a [T] {
    if data.is_null() || len == 0 { &[] } else { unsafe { std::slice::from_raw_parts(data, len) } }
}

/// The common exports over a `NativeCore`, and the optional groups named, whose bits must be exactly the optional bits
/// `CAPABILITIES` claims: `native_exports!(Machine; reset, mutes, rom_patches);`.
#[macro_export]
macro_rules! native_exports {
    ($t:ty; $($opt:ident),* $(,)?) => {
        const _: () = assert!(
            (<$t as $crate::abi::NativeCore>::CAPABILITIES
                & ($crate::abi::caps::RESET | $crate::abi::caps::MUTES | $crate::abi::caps::ROM_PATCHES | $crate::abi::caps::DEBUG | $crate::abi::caps::DEBUG_STACK))
                == (0 $(| $crate::__native_capability!($opt))*),
            "the optional exports and CAPABILITIES disagree"
        );
        $( $crate::__native_optional!($t, $opt); )*

        #[unsafe(no_mangle)]
        pub extern "C" fn emusen_native_interface_version() -> u32 {
            $crate::abi::version(<$t as $crate::abi::NativeCore>::CORE_VERSION)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn emusen_native_capabilities() -> u64 {
            <$t as $crate::abi::NativeCore>::CAPABILITIES
        }

        /// # Safety
        /// `path` must be a valid NUL-terminated UTF-8 string, or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_set_crash_log(path: *const std::ffi::c_char) -> i32 {
            unsafe { $crate::abi::install_crash_log(<$t as $crate::abi::NativeCore>::ENGINE, path) }
        }

        /// The machine for an image, with its create-time settings and files; null on refusal, the reason in `status`.
        ///
        /// # Safety
        /// `image` valid for `len` bytes, `settings` for `settings_len`, `files` for `file_count` entries each valid for
        /// its own length; `status` writable or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_create(
            image: *const u8,
            len: usize,
            settings: *const u8,
            settings_len: usize,
            files: *const $crate::abi::NativeFile,
            file_count: usize,
            status: *mut i32,
        ) -> *mut $t {
            let made = (|| {
                let settings = $crate::abi::Settings::parse(unsafe { $crate::ffi::input(settings, settings_len) })?;
                let raw: &[$crate::abi::NativeFile] = if files.is_null() || file_count == 0 { &[] } else { unsafe { std::slice::from_raw_parts(files, file_count) } };
                let files: Vec<$crate::abi::File<'_>> = raw.iter().map(|f| $crate::abi::File { which: f.which, data: unsafe { $crate::ffi::input(f.data, f.len) } }).collect();
                <$t as $crate::abi::NativeCore>::create(unsafe { $crate::ffi::input(image, len) }, &settings, &files)
            })();
            <$t as $crate::abi::NativeCore>::end_call();
            let (machine, code) = match made {
                Ok(m) => (Box::into_raw(Box::new(m)), 0),
                Err(e) => (std::ptr::null_mut(), e),
            };
            if let Some(s) = unsafe { status.as_mut() } {
                *s = code;
            }
            machine
        }

        /// # Safety
        /// `handle` must come from `emusen_native_create` and not be used again, or be null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_free(handle: *mut $t) -> i32 {
            if handle.is_null() {
                return $crate::ffi::status::NULL;
            }
            drop(unsafe { Box::from_raw(handle) });
            0
        }

        /// # Safety
        /// `handle` must be live or null; `detail` writable or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_advance(handle: *mut $t, detail: *mut u64) -> i32 {
            let Some(m) = (unsafe { handle.as_mut() }) else { return $crate::ffi::status::NULL };
            let mut word = 0u64;
            let r = <$t as $crate::abi::NativeCore>::advance(m, &mut word).map_or_else(|e| e, |()| 0);
            <$t as $crate::abi::NativeCore>::end_call();
            if let Some(d) = unsafe { detail.as_mut() } {
                *d = word;
            }
            r
        }

        /// # Safety
        /// `handle` must be live or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_set_options(handle: *mut $t, flags: u32) -> i32 {
            let Some(m) = (unsafe { handle.as_mut() }) else { return $crate::ffi::status::NULL };
            <$t as $crate::abi::NativeCore>::set_options(m, flags);
            0
        }

        /// # Safety
        /// `handle` must be live or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_frame_count(handle: *const $t) -> i64 {
            unsafe { handle.as_ref() }.map_or($crate::ffi::status::NULL as i64, |m| <$t as $crate::abi::NativeCore>::frame_count(m))
        }

        /// # Safety
        /// `handle` must be live or null; `out` writable or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_frame_info(handle: *const $t, out: *mut $crate::abi::FrameInfo) -> i32 {
            let Some(m) = (unsafe { handle.as_ref() }) else { return $crate::ffi::status::NULL };
            let Some(out) = (unsafe { out.as_mut() }) else { return $crate::ffi::status::NULL };
            *out = <$t as $crate::abi::NativeCore>::frame_info(m);
            0
        }

        /// The picture copied up to `len` bytes; its whole length, and a null `out` asks only that.
        ///
        /// # Safety
        /// `handle` must be live or null; `out` valid for `len` bytes, or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_frame_copy(handle: *const $t, out: *mut u8, len: usize) -> i64 {
            let Some(m) = (unsafe { handle.as_ref() }) else { return $crate::ffi::status::NULL as i64 };
            let frame = <$t as $crate::abi::NativeCore>::frame(m);
            if !out.is_null() {
                unsafe { std::ptr::copy_nonoverlapping(frame.as_ptr(), out, frame.len().min(len)) };
            }
            frame.len() as i64
        }

        /// # Safety
        /// `handle` must be live or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_audio_rate(handle: *const $t) -> i32 {
            unsafe { handle.as_ref() }.map_or($crate::ffi::status::NULL, |m| <$t as $crate::abi::NativeCore>::audio_rate(m))
        }

        /// Samples buffered, two a stereo frame.
        ///
        /// # Safety
        /// `handle` must be live or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_audio_buffered(handle: *const $t) -> i64 {
            unsafe { handle.as_ref() }.map_or($crate::ffi::status::NULL as i64, |m| <$t as $crate::abi::NativeCore>::audio_buffered(m) as i64)
        }

        /// Whole stereo pairs into `out`, at most `max_frames` of them; the samples written.
        ///
        /// # Safety
        /// `handle` must be live or null; `out` valid for `len` samples, or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_audio_drain(handle: *mut $t, out: *mut i16, len: usize, max_frames: i64) -> i64 {
            let Some(m) = (unsafe { handle.as_mut() }) else { return $crate::ffi::status::NULL as i64 };
            if out.is_null() || max_frames <= 0 {
                return 0;
            }
            let out = unsafe { std::slice::from_raw_parts_mut(out, len) };
            <$t as $crate::abi::NativeCore>::drain_audio(m, out, max_frames as usize) as i64
        }

        /// # Safety
        /// `handle` must be live or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_set_audio_limit(handle: *mut $t, samples: u64) -> i32 {
            let Some(m) = (unsafe { handle.as_mut() }) else { return $crate::ffi::status::NULL };
            <$t as $crate::abi::NativeCore>::set_audio_limit(m, samples as usize);
            0
        }

        /// # Safety
        /// `handle` must be live or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_set_buttons(handle: *mut $t, port: u32, mask: u32, changed: u32) -> i32 {
            let Some(m) = (unsafe { handle.as_mut() }) else { return $crate::ffi::status::NULL };
            <$t as $crate::abi::NativeCore>::set_buttons(m, port, mask, changed).map_or_else(|e| e, |()| 0)
        }

        /// Kind 0 is the state; kind 1, the snapshot, is not supported yet.
        ///
        /// # Safety
        /// `handle` must be live or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_state_size(handle: *const $t, kind: u32) -> i64 {
            let Some(m) = (unsafe { handle.as_ref() }) else { return $crate::ffi::status::NULL as i64 };
            if kind != 0 {
                return $crate::abi::status::NOT_SUPPORTED as i64;
            }
            $crate::ffi::StateMachine::state_size(m) as i64
        }

        /// # Safety
        /// `handle` must be live or null; `out` valid for `len` bytes.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_state_save(handle: *const $t, kind: u32, out: *mut u8, len: usize) -> i64 {
            let Some(m) = (unsafe { handle.as_ref() }) else { return $crate::ffi::status::NULL as i64 };
            if kind != 0 {
                return $crate::abi::status::NOT_SUPPORTED as i64;
            }
            if out.is_null() {
                return $crate::ffi::status::NULL as i64;
            }
            $crate::ffi::result($crate::ffi::StateMachine::save_state(m, unsafe { std::slice::from_raw_parts_mut(out, len) }))
        }

        /// C#'s whole `LoadState`; zero, or a status, and a failed load changes nothing.
        ///
        /// # Safety
        /// `handle` must be live or null; `data` valid for `len` bytes.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_state_load(handle: *mut $t, data: *const u8, len: usize) -> i32 {
            let Some(m) = (unsafe { handle.as_mut() }) else { return $crate::ffi::status::NULL };
            let r = match $crate::ffi::StateMachine::load_state(m, unsafe { $crate::ffi::input(data, len) }) {
                Ok(()) => 0,
                Err(e) => $crate::ffi::Status::status(&e),
            };
            <$t as $crate::abi::NativeCore>::end_call();
            r
        }

        /// # Safety
        /// `handle` must be live or null; `out` valid for `len` bytes, or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_state_layout(handle: *const $t, kind: u32, out: *mut u8, len: usize) -> i64 {
            let Some(m) = (unsafe { handle.as_ref() }) else { return $crate::ffi::status::NULL as i64 };
            if kind != 0 {
                return $crate::abi::status::NOT_SUPPORTED as i64;
            }
            unsafe { $crate::ffi::copy_text(&$crate::ffi::StateMachine::layout(m), out, len) }
        }

        /// # Safety
        /// `handle` must be live or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_space_size(handle: *const $t, space: u32) -> i64 {
            let Some(m) = (unsafe { handle.as_ref() }) else { return $crate::ffi::status::NULL as i64 };
            <$t as $crate::abi::NativeCore>::space_size(m, space).unwrap_or_else(|e| e as i64)
        }

        /// `len` bytes from `address` on, as the C# core's `ReadSpace` reads them; the bytes read, or a status.
        ///
        /// # Safety
        /// `handle` must be live or null; `out` valid for `len` bytes.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_space_read(handle: *mut $t, space: u32, address: u32, out: *mut u8, len: usize) -> i64 {
            let Some(m) = (unsafe { handle.as_mut() }) else { return $crate::ffi::status::NULL as i64 };
            if out.is_null() {
                return $crate::ffi::status::NULL as i64;
            }
            let out = unsafe { std::slice::from_raw_parts_mut(out, len) };
            let r = <$t as $crate::abi::NativeCore>::space_read(m, space, address, out).map_or_else(|e| e as i64, |()| len as i64);
            <$t as $crate::abi::NativeCore>::end_call();
            r
        }

        /// # Safety
        /// `handle` must be live or null; `data` valid for `len` bytes.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_space_write(handle: *mut $t, space: u32, address: u32, data: *const u8, len: usize) -> i64 {
            let Some(m) = (unsafe { handle.as_mut() }) else { return $crate::ffi::status::NULL as i64 };
            let r = <$t as $crate::abi::NativeCore>::space_write(m, space, address, unsafe { $crate::ffi::input(data, len) }).map_or_else(|e| e as i64, |()| len as i64);
            <$t as $crate::abi::NativeCore>::end_call();
            r
        }

        /// A battery file copied up to `len` bytes, its flags in `flags`; its whole length, and a null `out` asks only that.
        ///
        /// # Safety
        /// `handle` must be live or null; `out` valid for `len` bytes, or null; `flags` writable or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_battery(handle: *const $t, which: u32, out: *mut u8, len: usize, flags: *mut u32) -> i64 {
            let Some(m) = (unsafe { handle.as_ref() }) else { return $crate::ffi::status::NULL as i64 };
            match <$t as $crate::abi::NativeCore>::battery(m, which) {
                Ok((data, f)) => {
                    if !out.is_null() {
                        unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), out, data.len().min(len)) };
                    }
                    if let Some(fl) = unsafe { flags.as_mut() } {
                        *fl = f;
                    }
                    data.len() as i64
                }
                Err(e) => e as i64,
            }
        }

        /// # Safety
        /// `handle` must be live or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn emusen_native_battery_saved(handle: *mut $t, which: u32) -> i32 {
            let Some(m) = (unsafe { handle.as_mut() }) else { return $crate::ffi::status::NULL };
            <$t as $crate::abi::NativeCore>::battery_saved(m, which).map_or_else(|e| e, |()| 0)
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_version_puts_the_common_half_above_the_cores() {
        assert_eq!(version(3), 0x0001_0003);
        assert_eq!(fault::status(fault::DIVIDE_BY_ZERO), -322);
    }

    #[test]
    fn settings_are_key_value_lines() {
        let s = Settings::parse(b"Model=GBC\n\n ExpansionPak = 1 \n").unwrap();
        assert_eq!(s.get("Model"), Some("GBC"));
        assert_eq!(s.get("ExpansionPak"), Some("1"));
        assert_eq!(s.keys().count(), 2);
        assert_eq!(Settings::parse(b"nokey").unwrap_err(), status::BAD_SETTING);
        assert_eq!(Settings::parse(b"").unwrap().keys().count(), 0);
    }
}
