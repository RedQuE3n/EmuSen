//! A DianaSh command module's side of the module ABI (`dianash_module.h`): the ABI's types, a safe view of a call,
//! the `Module` trait every module implements, and `module_exports!`, which turns one into a library exporting
//! `dianash_module_*` with the fault boundary of §6.7. A module linked into a host, as `dianash.text` is into the
//! shell's program and the platform library, is called through the same call by `run_module`. See
//! EmuSen_DianaSh.md §6 and §6.10.

pub mod abi;

use abi::*;
use std::ffi::c_void;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// What a command gives back: its standard output, its standard error and its exit status. A C# command's returned
/// text is `ok`, its `DianaShResult.Fail` is `fail`, and an exception the dispatcher caught is `fault`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Answer {
    pub out: String,
    pub err: String,
    pub status: i32,
}

impl Answer {
    pub fn ok(out: impl Into<String>) -> Answer {
        Answer { out: out.into(), err: String::new(), status: 0 }
    }

    pub fn with(out: impl Into<String>, status: i32) -> Answer {
        Answer { out: out.into(), err: String::new(), status }
    }

    pub fn fail(err: impl Into<String>) -> Answer {
        Answer { out: String::new(), err: err.into(), status: 1 }
    }

    /// An exception the C# dispatcher caught, in its words: `Error: <message>`, status 1.
    pub fn fault(words: impl std::fmt::Display) -> Answer {
        Answer::fail(format!("Error: {words}"))
    }
}

/// A string as a JSON value, for a manifest.
fn json_string(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// What a module's manifest says of it (§5.3): each command as `(name, read_only, usage)`, the services it asks for as
/// `(name, major, least minor, required)`, its aliases and the job prefixes it claims.
pub struct Manifest<'a> {
    pub id: &'a str,
    pub version: &'a str,
    pub services: &'a [(&'a str, u32, u32, bool)],
    pub commands: &'a [(&'a str, bool, &'a str)],
    pub aliases: &'a [(&'a str, &'a str)],
    pub jobs: &'a [&'a str],
}

impl Manifest<'_> {
    /// The manifest as JSON, each command's page `man/<name>.1`.
    pub fn json(&self) -> String {
        let services: Vec<String> = self.services.iter().map(|(name, major, minor, required)| format!("{{\"name\":{},\"major\":{major},\"minor\":{minor},\"required\":{required}}}", json_string(name))).collect();
        let commands: Vec<String> = self.commands.iter().map(|(name, read_only, usage)| format!("{{\"name\":{},\"usage\":{},\"read_only\":{read_only},\"man\":{}}}", json_string(name), json_string(usage), json_string(&format!("man/{name}.1")))).collect();
        let aliases: Vec<String> = self.aliases.iter().map(|(alias, command)| format!("{}:{}", json_string(alias), json_string(command))).collect();
        let jobs: Vec<String> = self.jobs.iter().map(|prefix| json_string(prefix)).collect();
        format!(
            "{{\"manifest\":1,\"id\":{},\"version\":{},\"abi\":\"1.0\",\"kind\":\"module\",\"services\":[{}],\"commands\":[{}],\"aliases\":{{{}}},\"jobs\":[{}]}}",
            json_string(self.id),
            json_string(self.version),
            services.join(","),
            commands.join(","),
            aliases.join(","),
            jobs.join(",")
        )
    }
}

/// A module: its manifest, and the one function that runs every command it names.
pub trait Module: Sync {
    /// The manifest of §5.3, as JSON.
    fn manifest(&self) -> &'static str;

    /// Runs the command `call` names; the exit status, `$?`. What it prints goes through the call.
    fn run(&self, call: &Call) -> i32;

    /// `ps`'s rows for the id prefixes the manifest's `jobs` claims, each line ended by a newline, through the call.
    fn jobs(&self, _call: &Call) {}

    /// Removes the job `id` names: its exit status, or none when the id is not one of this module's.
    fn kill(&self, _call: &Call, _id: &str) -> Option<i32> {
        None
    }
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

    /// Prints an answer, its output then its errors, and gives its exit status.
    pub fn answer(&self, answer: Answer) -> i32 {
        if !answer.out.is_empty() {
            self.out(&answer.out);
        }
        if !answer.err.is_empty() {
            self.err(&answer.err);
        }
        answer.status
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

    /// The table of a service the host provides under `name` at major `major`, which the shell copied in unread (§6.3).
    pub fn service(&self, name: &str, major: u32) -> Option<*const c_void> {
        if self.raw.services.is_null() {
            return None;
        }
        let services = unsafe { std::slice::from_raw_parts(self.raw.services, self.raw.service_count) };
        services.iter().find(|s| s.major == major && unsafe { s.name.slice() } == Some(name.as_bytes())).map(|s| s.table).filter(|t| !t.is_null())
    }

    /// Whether the call runs where a change must not be made: a read-only line on a thread that may not wait (§6.5).
    pub fn read_only_context(&self) -> bool {
        self.raw.flags & CALL_READ_ONLY_CONTEXT != 0
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
            view.err(&fault_words(fault));
            status::FAULT
        }
    }
}

fn fault_words(fault: Box<dyn std::any::Any + Send>) -> String {
    fault.downcast_ref::<&str>().map(|s| s.to_string()).or_else(|| fault.downcast_ref::<String>().cloned()).unwrap_or_else(|| "the command faulted".to_string())
}

/// A module's `ps` rows, with the same fault boundary.
///
/// # Safety
/// `call` must be what the shell passed.
pub unsafe fn jobs_module(module: &dyn Module, call: *const dianash_call) -> i32 {
    let Some(view) = (unsafe { Call::from_raw(call) }) else { return status::BAD_STRUCT };
    match catch_unwind(AssertUnwindSafe(|| module.jobs(&view))) {
        Ok(()) => status::OK,
        Err(fault) => {
            view.err(&fault_words(fault));
            status::FAULT
        }
    }
}

/// A module's `kill` of one job: `ABSENT` for an id not its own, with the same fault boundary.
///
/// # Safety
/// `call`, `id` and `result` must be what the shell passed.
pub unsafe fn kill_module(module: &dyn Module, call: *const dianash_call, id: *const u8, id_len: usize, result: *mut dianash_result) -> i32 {
    if result.is_null() || id.is_null() {
        return status::NULL;
    }
    let Some(view) = (unsafe { Call::from_raw(call) }) else { return status::BAD_STRUCT };
    let id = String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(id, id_len) }).into_owned();
    match catch_unwind(AssertUnwindSafe(|| module.kill(&view, &id))) {
        Ok(Some(exit)) => {
            unsafe { (*result).exit_status = exit };
            status::OK
        }
        Ok(None) => status::ABSENT,
        Err(fault) => {
            view.err(&fault_words(fault));
            status::FAULT
        }
    }
}

/// Exports a module as a library: `dianash_module_abi_version`, `dianash_module_manifest`, `dianash_module_run`,
/// and the optional `dianash_module_jobs` and `dianash_module_kill`.
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

        /// # Safety
        /// `call` must be the shell's.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn dianash_module_jobs(call: *const $crate::abi::dianash_call) -> i32 {
            unsafe { $crate::jobs_module(&$module, call) }
        }

        /// # Safety
        /// `call`, `id` and `result` must be the shell's.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn dianash_module_kill(call: *const $crate::abi::dianash_call, id: *const u8, id_len: usize, result: *mut $crate::abi::dianash_result) -> i32 {
            unsafe { $crate::kill_module(&$module, call, id, id_len, result) }
        }
    };
}
