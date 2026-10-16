//! A DianaSh command module's side of the module ABI (`dianash_module.h`): the ABI's types, a safe view of a call,
//! the `Module` trait every module implements, and `module_exports!`, which turns one into a library exporting
//! `dianash_module_*` with the fault boundary of §6.7. A module linked into a host, as `dianash.text` is into the
//! shell's program and the platform library, is called through the same call by `run_module`. See
//! EmuSen_DianaSh.md §6 and §6.10.

pub mod abi;

use abi::*;
use std::ffi::c_void;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// A module: its manifest, and the one function that runs every command it names.
pub trait Module: Sync {
    /// The manifest of §5.3, as JSON.
    fn manifest(&self) -> &'static str;

    /// Runs the command `call` names; the exit status, `$?`. What it prints goes through the call.
    fn run(&self, call: &Call) -> i32;
}

/// A call as a module sees it, good until `run` returns.
pub struct Call<'a> {
    raw: &'a dianash_call,
}

unsafe fn text<'a>(bytes: dianash_bytes) -> Option<&'a [u8]> {
    unsafe { bytes.slice() }
}

/// Reads a length-query answer: a first try in 256 bytes, then once more in what was needed.
fn query(ask: impl Fn(*mut u8, usize, *mut i64) -> i32) -> Result<String, (i32, String)> {
    let mut buffer = vec![0u8; 256];
    let mut needed = 0i64;
    let mut status = ask(buffer.as_mut_ptr(), buffer.len(), &mut needed);
    if status == status::BUFFER_TOO_SMALL && needed > buffer.len() as i64 {
        buffer = vec![0u8; needed as usize];
        status = ask(buffer.as_mut_ptr(), buffer.len(), &mut needed);
    }
    let length = (needed.max(0) as usize).min(buffer.len());
    let answer = String::from_utf8_lossy(&buffer[..length]).into_owned();
    if status == status::OK { Ok(answer) } else { Err((status, answer)) }
}

impl<'a> Call<'a> {
    /// # Safety
    /// `raw` must be a call the shell made, valid for as long as the view is used.
    pub unsafe fn from_raw(raw: *const dianash_call) -> Option<Call<'a>> {
        if raw.is_null() {
            return None;
        }
        let raw = unsafe { &*raw };
        (raw.size as usize >= std::mem::size_of::<dianash_call>() && !raw.io.is_null()).then_some(Call { raw })
    }

    fn io(&self) -> &dianash_io {
        unsafe { &*self.raw.io }
    }

    /// The name it was called by, after alias mapping.
    pub fn command(&self) -> String {
        String::from_utf8_lossy(unsafe { text(self.raw.command) }.unwrap_or_default()).into_owned()
    }

    /// argv as text, `args()[0]` the name as typed.
    pub fn args(&self) -> Vec<String> {
        (0..self.raw.argc).map(|i| String::from_utf8_lossy(unsafe { text(*self.raw.argv.add(i)) }.unwrap_or_default()).into_owned()).collect()
    }

    /// Standard input: none when the command is first in its pipeline or alone, which is not the same as empty.
    pub fn stdin(&self) -> Option<String> {
        if self.raw.stdin_present == 0 {
            return None;
        }
        Some(String::from_utf8_lossy(unsafe { text(self.raw.stdin_data) }.unwrap_or_default()).into_owned())
    }

    pub fn cwd(&self) -> String {
        String::from_utf8_lossy(unsafe { text(self.raw.cwd) }.unwrap_or_default()).into_owned()
    }

    pub fn user(&self) -> String {
        String::from_utf8_lossy(unsafe { text(self.raw.user) }.unwrap_or_default()).into_owned()
    }

    /// Appends to standard output.
    pub fn out(&self, text: &str) {
        unsafe { (self.io().out)(self.io().ctx, text.as_ptr(), text.len()) };
    }

    /// Appends to standard error, which the shell prints where the output goes, in order (§4.3).
    pub fn err(&self, text: &str) {
        unsafe { (self.io().err)(self.io().ctx, text.as_ptr(), text.len()) };
    }

    /// A shell path to a real one through the sandbox; the words that refuse it when it lies above the root.
    pub fn resolve(&self, path: &str) -> Result<String, String> {
        let io = self.io();
        match query(|out, cap, needed| unsafe { (io.resolve)(io.ctx, path.as_ptr(), path.len(), RESOLVE_READ, out, cap, needed) }) {
            Ok(real) => Ok(real),
            Err((_, words)) => Err(words),
        }
    }

    /// A real path back to the shell's `/` form.
    pub fn display(&self, real: &str) -> String {
        let io = self.io();
        query(|out, cap, needed| unsafe { (io.display)(io.ctx, real.as_ptr(), real.len(), out, cap, needed) }).unwrap_or_else(|_| real.to_string())
    }

    /// Whether the player cancelled or the line's budget is spent, for a loop that may run long.
    pub fn cancelled(&self) -> bool {
        unsafe { (self.io().cancelled)(self.io().ctx) != 0 }
    }

    /// An exported variable, or none.
    pub fn environment(&self, name: &str) -> Option<String> {
        let io = self.io();
        query(|out, cap, needed| unsafe { (io.environment)(io.ctx, name.as_ptr(), name.len(), out, cap, needed) }).ok()
    }

    /// This module's per-shell state, made by `make` on the shell's first call and dropped when that shell's session ends.
    pub fn slot<T: Send + 'static>(&self, make: impl FnOnce() -> T) -> &mut T {
        unsafe extern "C" fn release<T>(state: *mut c_void) {
            drop(unsafe { Box::from_raw(state as *mut T) });
        }
        let io = self.io();
        let mut state: *mut c_void = std::ptr::null_mut();
        let mut releaser: Option<ReleaseFn> = None;
        unsafe { (io.slot)(io.ctx, &mut state, &mut releaser) };
        if state.is_null() {
            state = Box::into_raw(Box::new(make())) as *mut c_void;
            releaser = Some(release::<T>);
            unsafe { (io.slot)(io.ctx, &mut state, &mut releaser) };
        }
        unsafe { &mut *(state as *mut T) }
    }
}

/// Runs a module's command with the fault boundary every export has: a panic becomes `FAULT`, its words on standard error.
///
/// # Safety
/// `call` and `result` must be what the shell passed.
pub unsafe fn run_module(module: &dyn Module, call: *const dianash_call, result: *mut dianash_result) -> i32 {
    if result.is_null() {
        return status::NULL;
    }
    let Some(view) = (unsafe { Call::from_raw(call) }) else { return status::BAD_STRUCT };
    match catch_unwind(AssertUnwindSafe(|| module.run(&view))) {
        Ok(exit) => {
            unsafe { (*result).exit_status = exit };
            status::OK
        }
        Err(fault) => {
            let words = fault.downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| fault.downcast_ref::<String>().cloned()).unwrap_or_else(|| "the command faulted".to_string());
            view.err(&words);
            status::FAULT
        }
    }
}

/// Exports a module as a library: `dianash_module_abi_version`, `dianash_module_manifest` and `dianash_module_run`.
#[macro_export]
macro_rules! module_exports {
    ($module:expr) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn dianash_module_abi_version() -> u32 {
            $crate::abi::MODULE_ABI_VERSION
        }

        /// # Safety
        /// `out` must be valid for `len` bytes, or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn dianash_module_manifest(out: *mut u8, len: usize) -> i64 {
            let manifest = $crate::Module::manifest(&$module).as_bytes();
            if !out.is_null() && manifest.len() <= len {
                unsafe { std::ptr::copy_nonoverlapping(manifest.as_ptr(), out, manifest.len()) };
            }
            manifest.len() as i64
        }

        /// # Safety
        /// `call` and `result` must be the shell's.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn dianash_module_run(call: *const $crate::abi::dianash_call, result: *mut $crate::abi::dianash_result) -> i32 {
            unsafe { $crate::run_module(&$module, call, result) }
        }
    };
}
