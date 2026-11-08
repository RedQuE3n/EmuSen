//! A DianaSh command module's side of the module ABI (`dianash_module.h`): the ABI's types, a safe view of a call,
//! the `Module` trait every module implements, and `module_exports!`, which turns one into a library exporting
//! `dianash_module_*` with the fault boundary of §6.7. A module linked into a host, as `dianash.text` is into the
//! shell's program and the platform library, is called through the same call by `run_module`. See
//! EmuSen_DianaSh.md §6 and §6.10.

pub mod abi;

use abi::*;
use std::any::TypeId;
use std::ffi::c_void;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Mutex, MutexGuard, TryLockError};

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
    /// Since 1.1: the settings keys the module may get and set through the settings service, and whether it may approve
    /// and revoke a core library there (EmuSen_DianaSh.md §9.9); none and false for a module that asks for neither.
    pub settings: &'a [&'a str],
    pub approvals: bool,
}

impl Manifest<'_> {
    /// The manifest as JSON, each command's page `man/<name>.1`.
    pub fn json(&self) -> String {
        let services: Vec<String> = self.services.iter().map(|(name, major, minor, required)| format!("{{\"name\":{},\"major\":{major},\"minor\":{minor},\"required\":{required}}}", json_string(name))).collect();
        let commands: Vec<String> = self.commands.iter().map(|(name, read_only, usage)| format!("{{\"name\":{},\"usage\":{},\"read_only\":{read_only},\"man\":{}}}", json_string(name), json_string(usage), json_string(&format!("man/{name}.1")))).collect();
        let aliases: Vec<String> = self.aliases.iter().map(|(alias, command)| format!("{}:{}", json_string(alias), json_string(command))).collect();
        let jobs: Vec<String> = self.jobs.iter().map(|prefix| json_string(prefix)).collect();
        let asks = !self.settings.is_empty() || self.approvals;
        let settings = if asks { format!(",\"settings\":[{}],\"approvals\":{}", self.settings.iter().map(|k| json_string(k)).collect::<Vec<_>>().join(","), self.approvals) } else { String::new() };
        format!(
            "{{\"manifest\":1,\"id\":{},\"version\":{},\"abi\":\"{}\",\"kind\":\"module\",\"services\":[{}],\"commands\":[{}],\"aliases\":{{{}}},\"jobs\":[{}]{settings}}}",
            json_string(self.id),
            json_string(self.version),
            if asks { "1.1" } else { "1.0" },
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

    /// Since 1.1: whether the command was typed at the prompt, not run by a script, a substitution or a pipe.
    pub fn at_prompt(&self) -> bool {
        self.raw.flags & CALL_AT_PROMPT != 0
    }

    /// Since 1.1: the settings service as far as the manifest names it, or none where the program provides none.
    pub fn settings(&self) -> Option<Settings<'_>> {
        let table = self.service(SETTINGS_SERVICE, SETTINGS_MAJOR)? as *const dianash_settings_v1;
        let table = unsafe { &*table };
        (table.size as usize >= std::mem::size_of::<dianash_settings_v1>()).then_some(Settings { table })
    }

    /// An exported variable, or none.
    pub fn environment(&self, name: &str) -> Option<String> {
        let io = self.io();
        query(|out, cap, needed| unsafe { (io.environment)(io.ctx, name.as_ptr(), name.len(), out, cap, needed) }).ok()
    }

    /// This module's per-shell state, made by `make` on the shell's first call, behind a lock a second hold faults on (EmuSen_DianaSh.md §6.3).
    pub fn slot<T: Send + 'static>(&self, make: impl FnOnce() -> T) -> MutexGuard<'_, T> {
        unsafe extern "C" fn release<T>(state: *mut c_void) {
            drop(unsafe { Box::from_raw(state as *mut Kept<T>) });
        }
        let io = self.io();
        let mut state: *mut c_void = std::ptr::null_mut();
        let mut releaser: Option<ReleaseFn> = None;
        unsafe { (io.slot)(io.ctx, &mut state, &mut releaser) };
        if state.is_null() {
            state = Box::into_raw(Box::new(Kept { kind: TypeId::of::<T>(), value: Mutex::new(make()) })) as *mut c_void;
            releaser = Some(release::<T>);
            unsafe { (io.slot)(io.ctx, &mut state, &mut releaser) };
        }
        // `kind` sits first in a repr(C) Kept of any T, so it is read before the cast is trusted.
        assert!(unsafe { *(state as *const TypeId) } == TypeId::of::<T>(), "{}: the per-shell slot holds another type", self.command());
        let kept = unsafe { &*(state as *const Kept<T>) };
        match kept.value.try_lock() {
            Ok(held) => held,
            Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
            Err(TryLockError::WouldBlock) => panic!("{}: the per-shell slot is already held", self.command()),
        }
    }
}

/// What a slot keeps: the type it was made as, then the state behind its lock.
#[repr(C)]
struct Kept<T> {
    kind: TypeId,
    value: Mutex<T>,
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

/// The settings service a call was handed (since 1.1), each answer the setting or the service's words.
pub struct Settings<'a> {
    table: &'a dianash_settings_v1,
}

impl Settings<'_> {
    fn words(&self, status: i32) -> String {
        let t = self.table;
        query(|out, cap, needed| unsafe { (t.last_error)(t.ctx, out, cap, needed) }).unwrap_or_else(|_| format!("the settings service refused ({status})"))
    }

    /// A stored setting, none where it has no stored value.
    pub fn get(&self, scope: u32, target: &str, key: &str) -> Result<Option<String>, String> {
        let t = self.table;
        match query(|out, cap, needed| unsafe { (t.get)(t.ctx, scope, dianash_bytes::of(target.as_bytes()), dianash_bytes::of(key.as_bytes()), out, cap, needed) }) {
            Ok(value) => Ok(Some(value)),
            Err((status::ABSENT, _)) => Ok(None),
            Err((code, _)) => Err(self.words(code)),
        }
    }

    /// A setting stored, or with none cleared.
    pub fn set(&self, scope: u32, target: &str, key: &str, value: Option<&str>) -> Result<(), String> {
        let t = self.table;
        let value = value.map_or(dianash_bytes::ABSENT, |v| dianash_bytes::of(v.as_bytes()));
        match unsafe { (t.set)(t.ctx, scope, dianash_bytes::of(target.as_bytes()), dianash_bytes::of(key.as_bytes()), value) } {
            status::OK => Ok(()),
            code => Err(self.words(code)),
        }
    }

    /// A core library's SHA-256 approved; the shell lets it through only from a command typed at the prompt.
    pub fn approve(&self, sha256: &str) -> Result<(), String> {
        let t = self.table;
        match unsafe { (t.approve)(t.ctx, dianash_bytes::of(sha256.as_bytes())) } {
            status::OK => Ok(()),
            code => Err(self.words(code)),
        }
    }

    /// Every approval of a SHA-256 revoked.
    pub fn revoke(&self, sha256: &str) -> Result<(), String> {
        let t = self.table;
        match unsafe { (t.revoke)(t.ctx, dianash_bytes::of(sha256.as_bytes())) } {
            status::OK => Ok(()),
            code => Err(self.words(code)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A shell's side of one slot, as `io_slot` keeps it: one pointer and its release.
    struct Shelf {
        state: *mut c_void,
        release: Option<ReleaseFn>,
    }

    unsafe extern "C" fn keep(ctx: *mut c_void, state: *mut *mut c_void, release: *mut Option<ReleaseFn>) -> i32 {
        let shelf = unsafe { &mut *(ctx as *mut Shelf) };
        unsafe {
            if (*state).is_null() {
                *state = shelf.state;
                *release = shelf.release;
            } else {
                shelf.state = *state;
                shelf.release = *release;
            }
        }
        status::OK
    }
    unsafe extern "C" fn none_out(_: *mut c_void, _: *const u8, _: usize) -> i32 {
        status::OK
    }
    unsafe extern "C" fn none_query(_: *mut c_void, _: *const u8, _: usize, _: *mut u8, _: usize, _: *mut i64) -> i32 {
        status::ABSENT
    }
    unsafe extern "C" fn none_resolve(_: *mut c_void, _: *const u8, _: usize, _: u32, _: *mut u8, _: usize, _: *mut i64) -> i32 {
        status::ABSENT
    }
    unsafe extern "C" fn none_cancelled(_: *mut c_void) -> i32 {
        0
    }
    unsafe extern "C" fn none_terminal(_: *mut c_void, _: *mut dianash_terminal) -> i32 {
        status::OK
    }

    /// Runs `body` on calls over one shelf, as two lines of one shell would, then releases the slot as the shell does.
    fn with_shell(body: impl Fn(&Call)) {
        let mut shelf = Shelf { state: std::ptr::null_mut(), release: None };
        let io = dianash_io {
            size: std::mem::size_of::<dianash_io>() as u32,
            ctx: &mut shelf as *mut Shelf as *mut c_void,
            out: none_out,
            err: none_out,
            resolve: none_resolve,
            display: none_query,
            cancelled: none_cancelled,
            slot: keep,
            environment: none_query,
            terminal: none_terminal,
        };
        let raw = dianash_call {
            size: std::mem::size_of::<dianash_call>() as u32,
            command: dianash_bytes::of(b"probe"),
            argv: std::ptr::null(),
            argc: 0,
            stdin_data: dianash_bytes::ABSENT,
            stdin_present: 0,
            flags: 0,
            cwd: dianash_bytes::ABSENT,
            user: dianash_bytes::ABSENT,
            io: &io,
            services: std::ptr::null(),
            service_count: 0,
        };
        for _ in 0..2 {
            body(&unsafe { Call::from_raw(&raw) }.unwrap());
        }
        if let Some(release) = shelf.release {
            unsafe { release(shelf.state) };
        }
    }

    #[test]
    fn a_slot_is_made_once_and_kept_between_calls() {
        let (made, calls) = (std::cell::Cell::new(0), std::cell::Cell::new(0));
        with_shell(|call| {
            calls.set(calls.get() + 1);
            let mut count = call.slot(|| {
                made.set(made.get() + 1);
                0u32
            });
            *count += 1;
            assert_eq!(*count, calls.get());
        });
        assert_eq!(made.get(), 1);
    }

    #[test]
    fn a_second_hold_in_one_call_faults_rather_than_aliases() {
        with_shell(|call| {
            let _first = call.slot(|| 0u32);
            let second = catch_unwind(AssertUnwindSafe(|| drop(call.slot(|| 0u32))));
            assert!(second.is_err());
        });
    }

    #[test]
    fn a_hold_released_lets_the_next_hold_in() {
        with_shell(|call| {
            drop(call.slot(|| 0u32));
            *call.slot(|| 0u32) += 1;
        });
    }

    #[test]
    fn a_slot_asked_for_as_another_type_faults() {
        let asked = std::cell::Cell::new(0);
        with_shell(|call| {
            asked.set(asked.get() + 1);
            if asked.get() == 1 {
                drop(call.slot(|| 0u32));
            } else {
                assert!(catch_unwind(AssertUnwindSafe(|| drop(call.slot(String::new)))).is_err());
            }
        });
    }
}
