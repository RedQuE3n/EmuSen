//! DianaSh's host API, `dianash.h`: the C layer over the shell core's `Shell`. Unlike the rest of this library it
//! is a stable surface, versioned as the core ABI is, with its baseline in `Platform/dianash/abi/v1/`. See
//! EmuSen_DianaSh.md §10 and §11.
//!
//! A shell is used by one thread at a time; another thread's call waits for it, and a call made from inside one of
//! the shell's own commands, on the thread running it, is refused with `BUSY` rather than deadlocking.

use crate::{fail, put, LAST_ERROR};
use emusen_dianash::abi::{dianash_config, dianash_host_commands, dianash_service, status, CONFIG_INTERACTIVE, CONFIG_SESSIONS, DONE, EXIT, NEEDS_MORE};
use emusen_dianash::shell::{self, Config, Shell, Submitted};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

/// The opaque `dianash_shell`.
pub struct DianashShell {
    shell: Mutex<Shell>,
    cancel: Arc<AtomicBool>,
}

thread_local! {
    /// The shells this thread is running a line of, so that a call back into one is refused, not deadlocked.
    static RUNNING: RefCell<Vec<usize>> = const { RefCell::new(Vec::new()) };
}

fn running(sh: *const DianashShell) -> bool {
    RUNNING.with_borrow(|r| r.contains(&(sh as usize)))
}

/// The shell behind a handle, locked; `BUSY` when this thread is inside one of its lines.
unsafe fn lock<'a>(sh: *const DianashShell) -> Result<MutexGuard<'a, Shell>, i32> {
    if sh.is_null() {
        return Err(fail(status::NULL, "the shell is null"));
    }
    if running(sh) {
        return Err(fail(status::BUSY, "the shell is running a line on this thread"));
    }
    Ok(unsafe { &*sh }.shell.lock().unwrap_or_else(|poisoned| poisoned.into_inner()))
}

unsafe fn bytes<'a>(data: *const u8, len: usize) -> Result<Option<&'a str>, i32> {
    unsafe { crate::text(data, len) }
}

fn answer(result: Result<String, i32>, out: *mut u8, len: usize) -> i64 {
    match result {
        Ok(text) => unsafe { put(text.as_bytes(), out, len) },
        Err(code) => code as i64,
    }
}

fn session_fail(code: i32) -> i32 {
    match code {
        status::NO_SUCH_SESSION => fail(code, "no such session"),
        status::NOT_SUPPORTED => fail(code, "this shell keeps no sessions"),
        _ => code,
    }
}

/// The host API's version, `(MAJOR << 16) | MINOR`.
#[unsafe(no_mangle)]
pub extern "C" fn dianash_abi_version() -> u32 {
    emusen_dianash::HOST_ABI_VERSION
}

/// The shell's version and the versions of its two interfaces, as text.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_version_text(out: *mut u8, len: usize) -> i64 {
    unsafe { put(shell::version_text().as_bytes(), out, len) }
}

/// The words for the last failing call on the calling thread.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_last_error(out: *mut u8, len: usize) -> i64 {
    LAST_ERROR.with_borrow(|last| unsafe { put(last.as_bytes(), out, len) })
}

/// A shell, or null with the words in `dianash_last_error`.
///
/// # Safety
/// `config` must be null or point at a `dianash_config` of at least its stated size.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_new(config: *const dianash_config) -> *mut DianashShell {
    let mut settings = Config::default();
    if !config.is_null() {
        let given = unsafe { &*config };
        if (given.size as usize) < std::mem::size_of::<dianash_config>() {
            fail(status::BAD_STRUCT, "the config is smaller than version 1.0's");
            return std::ptr::null_mut();
        }
        settings.sessions = given.flags & CONFIG_SESSIONS != 0;
        settings.interactive = given.flags & CONFIG_INTERACTIVE != 0;
        match unsafe { given.root.slice() }.map(std::str::from_utf8) {
            Some(Ok(root)) => settings.root = Some(root.to_string()),
            Some(Err(_)) => {
                fail(status::BAD_STRING, "the root is not UTF-8");
                return std::ptr::null_mut();
            }
            None => {}
        }
        if given.budget_ms != 0 {
            settings.budget = Duration::from_millis(u64::from(given.budget_ms));
        }
    }
    let mut shell = Shell::new(settings);
    let modules: [&'static dyn dianash_module::Module; 6] = [&dianash_text::MODULE, &emusen_debug::MODULE, &emusen_trace::MODULE, &emusen_session::MODULE, &emusen_cheat::MODULE, &emusen_library_module::MODULE];
    for module in modules {
        if let Err(words) = shell.add_module(module) {
            fail(status::BAD_MANIFEST, words);
            return std::ptr::null_mut();
        }
    }
    if let Err(code) = shell.set_services(&with_library(&[])) {
        fail(code, "the library tool's service could not be handed to the shell");
        return std::ptr::null_mut();
    }
    let cancel = shell.cancel_flag();
    Box::into_raw(Box::new(DianashShell { shell: Mutex::new(shell), cancel }))
}

/// Ends a shell, releasing every session's state.
///
/// # Safety
/// `sh` must be a shell from `dianash_new`, not used again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_free(sh: *mut DianashShell) -> i32 {
    if sh.is_null() {
        return fail(status::NULL, "the shell is null");
    }
    if running(sh) {
        return fail(status::BUSY, "the shell is running a line on this thread");
    }
    drop(unsafe { Box::from_raw(sh) });
    status::OK
}

/// A host's commands for a session (0: the current one): a manifest and the functions that serve it.
///
/// # Safety
/// `sh` must be a live shell; `commands` must point at a `dianash_host_commands` whose pointers stay valid as long as the shell.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_add_commands(sh: *mut DianashShell, session: u32, commands: *const dianash_host_commands) -> i32 {
    let mut shell = match unsafe { lock(sh) } {
        Ok(shell) => shell,
        Err(code) => return code,
    };
    if commands.is_null() {
        return fail(status::NULL, "the commands are null");
    }
    let commands = unsafe { &*commands };
    if (commands.size as usize) < std::mem::size_of::<dianash_host_commands>() {
        return fail(status::BAD_STRUCT, "the commands' struct is smaller than version 1.0's");
    }
    match shell.add_commands(session, commands) {
        Ok(()) => status::OK,
        Err((code, words)) => fail(code, words),
    }
}

/// The host's services and, unless it names its own, the library tool's, which this library hands every shell so that
/// `emusen.library` runs without the host's help (EmuSen_GamePackages.md §12.3).
fn with_library(given: &[dianash_service]) -> Vec<dianash_service> {
    let name = crate::library_tool::SERVICE.as_bytes();
    let mut all = given.to_vec();
    if !given.iter().any(|s| unsafe { s.name.slice() } == Some(name)) {
        all.push(dianash_service {
            size: std::mem::size_of::<dianash_service>() as u32,
            name: emusen_dianash::abi::dianash_bytes::of(name),
            major: crate::library_tool::MAJOR,
            minor: crate::library_tool::MINOR,
            table: &crate::library_tool::TABLE as *const crate::library_tool::LibraryToolV1 as *const std::ffi::c_void,
        });
    }
    all
}

/// The services the host provides, copied into every call.
///
/// # Safety
/// `services` must be valid for `count` entries, or null with zero; each table must outlive the shell.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_set_services(sh: *mut DianashShell, services: *const dianash_service, count: usize) -> i32 {
    let mut shell = match unsafe { lock(sh) } {
        Ok(shell) => shell,
        Err(code) => return code,
    };
    let given = if count == 0 {
        &[][..]
    } else if services.is_null() {
        return fail(status::NULL, "the services are null");
    } else {
        unsafe { std::slice::from_raw_parts(services, count) }
    };
    match shell.set_services(&with_library(given)) {
        Ok(()) => status::OK,
        Err(code) => fail(code, "a service has no name"),
    }
}

/// The services one session's calls are handed in place of the shell's of the same name (since 1.1).
///
/// # Safety
/// `services` must be valid for `count` entries, or null with zero; each table must outlive the session.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_session_set_services(sh: *mut DianashShell, session: u32, services: *const dianash_service, count: usize) -> i32 {
    let mut shell = match unsafe { lock(sh) } {
        Ok(shell) => shell,
        Err(code) => return code,
    };
    let given = if count == 0 {
        &[][..]
    } else if services.is_null() {
        return fail(status::NULL, "the services are null");
    } else {
        unsafe { std::slice::from_raw_parts(services, count) }
    };
    match shell.set_session_services(session, given) {
        Ok(()) => status::OK,
        Err(status::NO_SUCH_SESSION) => session_fail(status::NO_SUCH_SESSION),
        Err(code) => fail(code, "a service has no name"),
    }
}

/// Runs a line in a session (0: the current one): `DONE`, `NEEDS_MORE` while a block is open, `EXIT` when it ended the shell.
///
/// # Safety
/// `sh` must be a live shell; `line` must be valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_submit(sh: *mut DianashShell, session: u32, line: *const u8, len: usize) -> i32 {
    let text = match unsafe { bytes(line, len) } {
        Ok(Some(text)) => text,
        Ok(None) => return fail(status::NULL, "the line is null"),
        Err(code) => return code,
    };
    let mut shell = match unsafe { lock(sh) } {
        Ok(shell) => shell,
        Err(code) => return code,
    };
    RUNNING.with_borrow_mut(|r| r.push(sh as usize));
    let submitted = shell.submit(session, text);
    RUNNING.with_borrow_mut(|r| r.retain(|s| *s != sh as usize));
    match submitted {
        Ok(Submitted::Done) => DONE,
        Ok(Submitted::NeedsMore) => NEEDS_MORE,
        Ok(Submitted::Exit) => EXIT,
        Err(code) => session_fail(code),
    }
}

/// What the last line printed, kept until a buffer takes all of it.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_output_take(sh: *mut DianashShell, out: *mut u8, len: usize) -> i64 {
    let mut shell = match unsafe { lock(sh) } {
        Ok(shell) => shell,
        Err(code) => return code as i64,
    };
    let length = unsafe { put(shell.output().as_bytes(), out, len) };
    if !out.is_null() && length as usize <= len {
        shell.take_output();
    }
    length
}

/// What the lines since the last drain asked of the host, as JSON, removed once a buffer takes all of it.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_requests_drain(sh: *mut DianashShell, out: *mut u8, len: usize) -> i64 {
    let mut shell = match unsafe { lock(sh) } {
        Ok(shell) => shell,
        Err(code) => return code as i64,
    };
    let requests = shell.drain_requests();
    let text = Shell::requests_json(&requests);
    let length = unsafe { put(text.as_bytes(), out, len) };
    if out.is_null() || length as usize > len {
        shell.restore_requests(requests);
    }
    length
}

/// `$?` of a session's last line, written to `status_out`.
///
/// # Safety
/// `status_out` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_last_status(sh: *mut DianashShell, session: u32, status_out: *mut i32) -> i32 {
    if status_out.is_null() {
        return fail(status::NULL, "nowhere to write the status");
    }
    let mut shell = match unsafe { lock(sh) } {
        Ok(shell) => shell,
        Err(code) => return code,
    };
    match shell.last_status(session) {
        Ok(code) => {
            unsafe { *status_out = code };
            status::OK
        }
        Err(code) => session_fail(code),
    }
}

/// 1 when a line is one read-only command a host may run on a thread that must not wait, else 0 (§6.5).
///
/// # Safety
/// `line` must be valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_is_read_only(sh: *mut DianashShell, session: u32, line: *const u8, len: usize) -> i32 {
    let text = match unsafe { bytes(line, len) } {
        Ok(Some(text)) => text,
        Ok(None) => return fail(status::NULL, "the line is null"),
        Err(code) => return code,
    };
    let mut shell = match unsafe { lock(sh) } {
        Ok(shell) => shell,
        Err(code) => return code,
    };
    match shell.is_read_only(session, text) {
        Ok(read_only) => i32::from(read_only),
        Err(code) => session_fail(code),
    }
}

/// 1 while a session holds an open block, a quote or a `$(...)`, else 0.
///
/// # Safety
/// `sh` must be a live shell.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_awaiting(sh: *mut DianashShell, session: u32) -> i32 {
    let mut shell = match unsafe { lock(sh) } {
        Ok(shell) => shell,
        Err(code) => return code,
    };
    match shell.awaiting(session) {
        Ok(awaiting) => i32::from(awaiting),
        Err(code) => session_fail(code),
    }
}

/// Stops the running line at its next check, from any thread.
///
/// # Safety
/// `sh` must be a live shell.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_cancel(sh: *mut DianashShell) -> i32 {
    if sh.is_null() {
        return fail(status::NULL, "the shell is null");
    }
    unsafe { &*sh }.cancel.store(true, Ordering::Relaxed);
    status::OK
}

/// The prompt the shell's own program shows for a session.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_prompt(sh: *mut DianashShell, session: u32, out: *mut u8, len: usize) -> i64 {
    let mut shell = match unsafe { lock(sh) } {
        Ok(shell) => shell,
        Err(code) => return code as i64,
    };
    answer(shell.prompt(session).map_err(session_fail), out, len)
}

/// A session's history, each entry ended by a NUL.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_history(sh: *mut DianashShell, session: u32, out: *mut u8, len: usize) -> i64 {
    let shell = match unsafe { lock(sh) } {
        Ok(shell) => shell,
        Err(code) => return code as i64,
    };
    match shell.history(session) {
        Ok(entries) => unsafe { put(&crate::join_nul(&entries), out, len) },
        Err(code) => session_fail(code) as i64,
    }
}

/// The shell's own banner, which names no core.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_banner(sh: *mut DianashShell, out: *mut u8, len: usize) -> i64 {
    if let Err(code) = unsafe { lock(sh) } {
        return code as i64;
    }
    unsafe { put(shell::banner().as_bytes(), out, len) }
}

/// A session's commands in `help`'s order, as JSON: name, kind, usage, read_only.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_commands(sh: *mut DianashShell, session: u32, out: *mut u8, len: usize) -> i64 {
    let shell = match unsafe { lock(sh) } {
        Ok(shell) => shell,
        Err(code) => return code as i64,
    };
    answer(shell.commands_json(session).map_err(session_fail), out, len)
}

/// The account a session runs as.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_user(sh: *mut DianashShell, session: u32, out: *mut u8, len: usize) -> i64 {
    let shell = match unsafe { lock(sh) } {
        Ok(shell) => shell,
        Err(code) => return code as i64,
    };
    answer(shell.user(session).map_err(session_fail), out, len)
}

/// Sets the account a session runs as, with no check that it exists, as a host's rebuild restores it.
///
/// # Safety
/// `name` must be valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_set_user(sh: *mut DianashShell, session: u32, name: *const u8, len: usize) -> i32 {
    let name = match unsafe { bytes(name, len) } {
        Ok(Some(name)) => name,
        Ok(None) => return fail(status::NULL, "the name is null"),
        Err(code) => return code,
    };
    let mut shell = match unsafe { lock(sh) } {
        Ok(shell) => shell,
        Err(code) => return code,
    };
    match shell.set_user(session, name) {
        Ok(()) => status::OK,
        Err(code) => session_fail(code),
    }
}

/// The listed sessions as JSON: id, name, user, current.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_sessions(sh: *mut DianashShell, out: *mut u8, len: usize) -> i64 {
    let shell = match unsafe { lock(sh) } {
        Ok(shell) => shell,
        Err(code) => return code as i64,
    };
    unsafe { put(shell.sessions_json().as_bytes(), out, len) }
}

/// A new session, unlisted until attached; its id.
///
/// # Safety
/// `sh` must be a live shell.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_session_new(sh: *mut DianashShell) -> i64 {
    let mut shell = match unsafe { lock(sh) } {
        Ok(shell) => shell,
        Err(code) => return code as i64,
    };
    match shell.session_new() {
        Ok(id) => i64::from(id),
        Err(code) => session_fail(code) as i64,
    }
}

/// Lists a session under a name, or the next default one when `name` is null, and makes it current.
///
/// # Safety
/// `name` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_session_attach(sh: *mut DianashShell, session: u32, name: *const u8, len: usize) -> i32 {
    let name = match unsafe { bytes(name, len) } {
        Ok(name) => name,
        Err(code) => return code,
    };
    let mut shell = match unsafe { lock(sh) } {
        Ok(shell) => shell,
        Err(code) => return code,
    };
    match shell.session_attach(session, name) {
        Ok(_) => status::OK,
        Err((code, words)) => fail(code, words),
    }
}

/// Makes a listed session current.
///
/// # Safety
/// `sh` must be a live shell.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_session_switch(sh: *mut DianashShell, session: u32) -> i32 {
    let mut shell = match unsafe { lock(sh) } {
        Ok(shell) => shell,
        Err(code) => return code,
    };
    match shell.session_switch(session) {
        Ok(()) => status::OK,
        Err(code) => session_fail(code),
    }
}

/// Ends a session: a listed one unless it is the last, an unlisted one always.
///
/// # Safety
/// `sh` must be a live shell.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_session_kill(sh: *mut DianashShell, session: u32) -> i32 {
    let mut shell = match unsafe { lock(sh) } {
        Ok(shell) => shell,
        Err(code) => return code,
    };
    match shell.session_kill(session) {
        Ok(()) => status::OK,
        Err((code, words)) => fail(code, words),
    }
}

/// An unlisted session takes a session's name, place and currency, and the old one ends.
///
/// # Safety
/// `sh` must be a live shell.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dianash_session_replace(sh: *mut DianashShell, session: u32, replacement: u32) -> i32 {
    let mut shell = match unsafe { lock(sh) } {
        Ok(shell) => shell,
        Err(code) => return code,
    };
    match shell.session_replace(session, replacement) {
        Ok(()) => status::OK,
        Err(code) => session_fail(code),
    }
}
