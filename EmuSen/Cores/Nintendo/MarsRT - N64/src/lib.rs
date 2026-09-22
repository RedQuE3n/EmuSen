//! MarsRT, the N64 core in Rust, and the C# Mars's native components, called through a C ABI. See Mars_Native.md.

pub mod cpu;
pub mod ffi;
pub mod machine;
pub mod memory;
#[cfg(test)]
mod naming;
pub mod rdp;
pub mod rom;
pub mod rsp;
pub mod state;
#[cfg(test)]
mod tests;
pub mod vi;

use std::ffi::{CStr, c_char};
use std::sync::OnceLock;

/// The interface version; C# refuses a library whose number is not the one it was written against.
pub const INTERFACE_VERSION: u32 = 6;

/// A field C# marks `[SkipInState]`: derived or host state, in no state and no part of the machine's identity.
#[derive(Clone, Copy, Debug, Default)]
pub struct Skip<T>(pub T);

impl<T> PartialEq for Skip<T> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<T> Eq for Skip<T> {}

impl<T> std::ops::Deref for Skip<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Skip<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

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
