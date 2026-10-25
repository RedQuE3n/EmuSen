//! The library tool's C layer (EmuSen_GamePackages.md §12.3): a job started from a request, its events taken one at a
//! time by the length-query idiom, stopped and freed. The same four calls are the `emusen.library` service every shell
//! this library makes is handed, so that the shell's module and Mistress's window run one tool.

use crate::{bytes, fail, put, status};
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::models::{APP_SETTINGS_ID, MODELS};
use emusen_galaxia::{config, setting_folders, tree};
use emusen_library_tool::{job, Context};
use std::time::Duration;

/// The opaque `emusen_tool_job`.
pub struct LibraryJob(job::Job);

/// `emusen_library_tool.h`'s table, laid out as the header lays it out.
#[repr(C)]
pub struct LibraryToolV1 {
    pub size: u32,
    pub start: unsafe extern "C" fn(*const u8, usize, *mut *mut LibraryJob) -> i32,
    pub next: unsafe extern "C" fn(*mut LibraryJob, u32, *mut u8, usize) -> i64,
    pub stop: unsafe extern "C" fn(*mut LibraryJob) -> i32,
    pub free: unsafe extern "C" fn(*mut LibraryJob) -> i32,
    pub last_error: unsafe extern "C" fn(*mut u8, usize) -> i64,
}

pub const SERVICE: &str = "emusen.library";
pub const MAJOR: u32 = 1;
pub const MINOR: u32 = 2;

/// The service's one table, for every shell.
pub static TABLE: LibraryToolV1 = LibraryToolV1 {
    size: std::mem::size_of::<LibraryToolV1>() as u32,
    start: emusen_tool_job_start,
    next: emusen_tool_job_next,
    stop: emusen_tool_job_stop,
    free: emusen_tool_job_free,
    last_error: crate::emusen_platform_last_error,
};

/// The chdman a job runs: `EMUSEN_CHDMAN` where set, for a build's tests, else the one beside this library; either is
/// refused unless its SHA-256 is the pin's.
fn chdman() -> String {
    std::env::var("EMUSEN_CHDMAN").ok().filter(|c| !c.is_empty()).unwrap_or_else(|| path::combine(Style::HOST, &tree::base_directory(), if cfg!(windows) { "chdman.exe" } else { "chdman" }))
}

/// The process's tree, `AppSettings.RomDirectory` and `StateDirectory` as the settings file holds them, and the
/// process's SQLite where it has one loaded or one ships beside it.
fn context() -> Context {
    let tree = tree::tree().clone();
    let (settings, _) = config::load_or_new(&tree, &MODELS[APP_SETTINGS_ID as usize], None, "appsettings.json");
    let library = setting_folders::checked(settings.get("RomDirectory").and_then(|v| v.as_str()), &tree::user_home()).path;
    let states = settings.get("StateDirectory").and_then(|v| v.as_str()).filter(|s| !s.trim().is_empty()).map(str::to_string);
    let beside = path::combine(Style::HOST, &tree::base_directory(), emusen_sqlite::SHIPPED_NAME);
    let sqlite = emusen_sqlite::Sqlite::of_process(&[beside]).ok().map(std::sync::Arc::new);
    Context { state_folder: states, sqlite, ..Context::pinned(tree, library, chdman()) }
}

/// Starts a job from a request (EmuSen_GamePackages.md §12.3); PARSE with words for one that is not a request.
///
/// # Safety
/// `request` must be valid for `len` bytes; `job` must be valid for a write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_tool_job_start(request: *const u8, len: usize, job: *mut *mut LibraryJob) -> i32 {
    if job.is_null() {
        return fail(status::NULL, "the job's place is null");
    }
    let text = match unsafe { bytes(request, len) }.map(std::str::from_utf8) {
        Ok(Ok(t)) => t,
        Ok(Err(_)) => return fail(status::BAD_STRING, "the request is not UTF-8"),
        Err(code) => return code,
    };
    match job::start(text, context()) {
        Ok(j) => {
            unsafe { *job = Box::into_raw(Box::new(LibraryJob(j))) };
            0
        }
        Err(words) => fail(status::PARSE, words),
    }
}

/// The next event, waiting up to `wait_ms` for one: its length, copied and taken only when `out` holds it; ABSENT when
/// none came in time; 0 once the last was taken.
///
/// # Safety
/// `job` must be a job from `emusen_tool_job_start`; `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_tool_job_next(job: *mut LibraryJob, wait_ms: u32, out: *mut u8, len: usize) -> i64 {
    let Some(job) = (unsafe { job.as_ref() }) else { return fail(status::NULL, "the job is null") as i64 };
    match job.0.peek(Duration::from_millis(wait_ms as u64)) {
        Some(event) => {
            let length = unsafe { put(event.as_bytes(), out, len) };
            if !out.is_null() && event.len() <= len {
                job.0.take();
            }
            length
        }
        None if job.0.finished() => 0,
        None => status::ABSENT as i64,
    }
}

/// Asks a job to stop; it ends with what it did, its last events still to be taken.
///
/// # Safety
/// `job` must be a job from `emusen_tool_job_start`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_tool_job_stop(job: *mut LibraryJob) -> i32 {
    let Some(job) = (unsafe { job.as_ref() }) else { return fail(status::NULL, "the job is null") };
    job.0.stop();
    0
}

/// Stops a job, waits for it to end and frees it.
///
/// # Safety
/// `job` must be a job from `emusen_tool_job_start`, not used again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_tool_job_free(job: *mut LibraryJob) -> i32 {
    if job.is_null() {
        return fail(status::NULL, "the job is null");
    }
    drop(unsafe { Box::from_raw(job) });
    0
}
