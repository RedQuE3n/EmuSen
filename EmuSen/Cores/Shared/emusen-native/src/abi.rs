//! What the pre-stable common native interface left to the core ABI v1, which `core` re-exports: the capability bits
//! 0 to 13, the status codes and the fault band, `Region`, `File`, `Settings` and the crash log. The interface itself,
//! its trait, its `emusen_native_*` exports and their macro, was deleted with B4 on 2026-10-18 (EmuSen_Backend.md §26).

use std::ffi::{CStr, c_char};
use std::io::Write;
use std::sync::{Mutex, OnceLock};

/// The capability bits 0 to 13, which the core ABI v1 kept from the pre-stable numbering (`core::caps`).
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

/// A file the host read for this game, as the core sees it: `which` 0 is the battery save.
#[derive(Clone, Copy, Debug)]
pub struct File<'a> {
    pub which: u32,
    pub data: &'a [u8],
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

    /// Every line as (key, value), in the order given.
    pub fn pairs(&self) -> impl Iterator<Item = (&str, &str)> {
        self.pairs.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    pub fn from_pairs(pairs: Vec<(String, String)>) -> Settings {
        Settings { pairs }
    }
}

static CRASH_LOG: OnceLock<String> = OnceLock::new();

/// Held while a report is written, so a second thread's panic waits for the first's report rather than cutting it off.
static CRASH_WRITE: Mutex<()> = Mutex::new(());

/// Where a panic is written before the process aborts; the first path given stands, and each report is appended to it
/// whole (`Diamond_Players.md` §12.7.5).
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
            let _one = CRASH_WRITE.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(path) = CRASH_LOG.get() {
                let _ = std::fs::OpenOptions::new().create(true).append(true).open(path).and_then(|mut f| f.write_all(text.as_bytes()));
            }
            eprintln!("{text}");
        }));
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fault_kind_counts_down_from_the_band_s_base() {
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
