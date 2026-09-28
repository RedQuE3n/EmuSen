//! MercuryRT, the Game Boy and Game Boy Color core in Rust, called through a C ABI. See Mercury_Native.md.

pub mod apu;
pub mod cpu;
pub mod debug;
pub mod ffi;
pub mod machine;
pub mod memory;
#[cfg(test)]
mod naming;
pub mod ppu;
pub mod state;

use std::ffi::{CStr, c_char};
use std::sync::OnceLock;

/// The interface version; C# refuses a library whose number is not the one it was written against.
pub const INTERFACE_VERSION: u32 = 3;

/// A field C# marks `[SkipInState]`; see emusen-native.
pub use emusen_native::Skip;

#[unsafe(no_mangle)]
pub extern "C" fn mercury_interface_version() -> u32 {
    INTERFACE_VERSION
}

static CRASH_LOG: OnceLock<String> = OnceLock::new();

/// Where a panic is written before the process aborts, since a panic must never cross into C#. See Mars_Native.md §2.
///
/// # Safety
/// `path` must be a valid NUL-terminated UTF-8 string, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mercury_set_crash_log(path: *const c_char) {
    if path.is_null() {
        return;
    }
    let path = unsafe { CStr::from_ptr(path) }.to_string_lossy().into_owned();
    if CRASH_LOG.set(path).is_ok() {
        std::panic::set_hook(Box::new(|info| {
            let text = format!("MercuryRT panic: {info}\n{}\n", std::backtrace::Backtrace::force_capture());
            if let Some(path) = CRASH_LOG.get() {
                let _ = std::fs::write(path, &text);
            }
            eprintln!("{text}");
        }));
    }
}
