//! MarsRT, the N64 core in Rust, and the C# Mars's native components, called through a C ABI. See Mars_Native.md.

pub mod rsp;

use std::ffi::{CStr, c_char};
use std::sync::OnceLock;

/// The interface version; C# refuses a library whose number is not the one it was written against.
pub const INTERFACE_VERSION: u32 = 1;

#[unsafe(no_mangle)]
pub extern "C" fn emusen_native_interface_version() -> u32 {
    INTERFACE_VERSION
}

static CRASH_LOG: OnceLock<String> = OnceLock::new();

/// Where a panic is written before the process aborts, since a panic must never cross into C#. See Mars_Native.md §2.
///
/// # Safety
/// `path` must be a valid NUL-terminated UTF-8 string, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_native_set_crash_log(path: *const c_char) {
    if path.is_null() {
        return;
    }
    let path = unsafe { CStr::from_ptr(path) }.to_string_lossy().into_owned();
    if CRASH_LOG.set(path).is_ok() {
        std::panic::set_hook(Box::new(|info| {
            let text = format!("EmuSen native panic: {info}\n{}\n", std::backtrace::Backtrace::force_capture());
            if let Some(path) = CRASH_LOG.get() {
                let _ = std::fs::write(path, &text);
            }
            eprintln!("{text}");
        }));
    }
}
