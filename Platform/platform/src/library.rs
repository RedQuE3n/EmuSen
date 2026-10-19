//! The C layer over emusen-library, `emusen_library.h`: the library's stable header, exported from the platform library
//! beside its unstable one, as `dianash.h` is. Each call converts its arguments, asks the crate and copies the answer
//! out; nothing is decided here. See EmuSen_Backend.md §28.

use emusen_library::{Error, Library, abi};
use emusen_sqlite::Sqlite;
use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::Arc;

thread_local! {
    static LAST_ERROR: RefCell<String> = const { RefCell::new(String::new()) };
    /// An answer that did not fit, by the export and the request that made it, taken by the same call asked again.
    static WAITING: RefCell<Option<(u8, Vec<u8>, Vec<u8>)>> = const { RefCell::new(None) };
}

fn fail(status: i32, words: impl Into<String>) -> i32 {
    LAST_ERROR.with_borrow_mut(|last| *last = words.into());
    status
}

fn failed(error: Error) -> i32 {
    fail(error.status(), error.words())
}

/// The text a caller passed; an empty one may come as a null pointer with no length.
unsafe fn text<'a>(data: *const u8, len: usize) -> Result<&'a str, i32> {
    if data.is_null() {
        return if len == 0 { Ok("") } else { Err(fail(abi::NULL, "a text with a length is null")) };
    }
    let bytes = if len == 0 { &[][..] } else { unsafe { std::slice::from_raw_parts(data, len) } };
    std::str::from_utf8(bytes).map_err(|_| fail(abi::BAD_STRING, "the text is not UTF-8"))
}

unsafe fn handle<'a>(lib: *mut Library) -> Result<&'a Library, i32> {
    if lib.is_null() { Err(fail(abi::NULL, "the library handle is null")) } else { Ok(unsafe { &*lib }) }
}

/// The length-query idiom over an answer made once: copied when it fits, else kept for the same call asked again.
unsafe fn answer(export: u8, request: &str, out: *mut u8, len: usize, make: impl FnOnce() -> Result<String, Error>) -> i64 {
    let waiting = WAITING.with_borrow_mut(|w| match w {
        Some((e, r, _)) if *e == export && r.as_slice() == request.as_bytes() => w.take().map(|(_, _, a)| a),
        _ => None,
    });
    let bytes = match waiting {
        Some(bytes) => bytes,
        None => match make() {
            Ok(text) => text.into_bytes(),
            Err(e) => return failed(e) as i64,
        },
    };
    if !out.is_null() && bytes.len() <= len {
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, bytes.len()) };
    } else {
        let length = bytes.len();
        WAITING.with_borrow_mut(|w| *w = Some((export, request.as_bytes().to_vec(), bytes)));
        return length as i64;
    }
    bytes.len() as i64
}

/// The header's version, `(MAJOR << 16) | MINOR`.
#[unsafe(no_mangle)]
pub extern "C" fn emusen_library_abi_version() -> u32 {
    abi::VERSION
}

/// The words of this thread's last failing call, by the length-query idiom.
///
/// # Safety
/// `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_library_last_error(out: *mut u8, len: usize) -> i64 {
    LAST_ERROR.with_borrow(|words| {
        if !out.is_null() && words.len() <= len {
            unsafe { std::ptr::copy_nonoverlapping(words.as_ptr(), out, words.len()) };
        }
        words.len() as i64
    })
}

/// Opens `games.db` with the host's SQLite, migrating it; the handle in `out`.
///
/// # Safety
/// `config` valid for its `size` bytes; its texts valid for their lengths; `sqlite` the platform loader's live handle
/// of SQLite, kept loaded while the handle lives; `out` valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_library_open(config: *const abi::Config, out: *mut *mut Library) -> i32 {
    if config.is_null() || out.is_null() {
        return fail(abi::NULL, "the config or the handle's place is null");
    }
    let config = unsafe { &*config };
    if (config.size as usize) < std::mem::size_of::<abi::Config>() {
        return fail(abi::NULL, "the config is shorter than version 1.0's");
    }
    let path = match unsafe { text(config.database, config.database_len) } {
        Ok(p) if !p.is_empty() => p,
        Ok(_) => return fail(abi::NULL, "no database was named"),
        Err(s) => return s,
    };
    if config.sqlite.is_null() {
        return fail(abi::NO_SQLITE, "no SQLite was lent to the library");
    }
    let sqlite = match unsafe { Sqlite::lent(config.sqlite as *mut c_void) } {
        Ok(s) => Arc::new(s),
        Err(words) => return fail(abi::NO_SQLITE, words),
    };
    match Library::open(sqlite, path) {
        Ok(lib) => {
            unsafe { *out = Box::into_raw(Box::new(lib)) };
            0
        }
        Err(e) => failed(e),
    }
}

/// Closes a handle; nothing may be running on it.
///
/// # Safety
/// `lib` from `emusen_library_open`, not closed before, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_library_close(lib: *mut Library) -> i32 {
    if lib.is_null() {
        return fail(abi::NULL, "the library handle is null");
    }
    drop(unsafe { Box::from_raw(lib) });
    0
}

/// Scans the request's roots for its systems, keeping the result; the games found, or a status.
///
/// # Safety
/// `lib` a live handle; `request` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_library_scan(lib: *mut Library, request: *const u8, len: usize) -> i64 {
    let run = || -> Result<i64, i32> {
        let lib = unsafe { handle(lib) }?;
        let request = unsafe { text(request, len) }?;
        lib.scan_request(request).map(|n| n as i64).map_err(failed)
    };
    run().unwrap_or_else(|s| s as i64)
}

/// What a running scan or reclaim has done, from any thread, never waiting.
///
/// # Safety
/// `lib` a live handle; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_library_progress(lib: *mut Library, out: *mut u8, len: usize) -> i64 {
    match unsafe { handle(lib) } {
        Ok(lib) => {
            let text = lib.progress.to_json();
            if !out.is_null() && text.len() <= len {
                unsafe { std::ptr::copy_nonoverlapping(text.as_ptr(), out, text.len()) };
            }
            text.len() as i64
        }
        Err(s) => s as i64,
    }
}

/// Stops a running scan or reclaim at its next file.
///
/// # Safety
/// `lib` a live handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_library_cancel(lib: *mut Library) -> i32 {
    match unsafe { handle(lib) } {
        Ok(lib) => {
            lib.cancel();
            0
        }
        Err(s) => s,
    }
}

/// The last scan, narrowed by the filter.
///
/// # Safety
/// `lib` a live handle; `filter` valid for `len` bytes, or null with none; `out` valid for `out_len`, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_library_games(lib: *mut Library, filter: *const u8, len: usize, out: *mut u8, out_len: usize) -> i64 {
    let (lib, filter) = match unsafe { handle(lib) }.and_then(|l| unsafe { text(filter, len) }.map(|f| (l, f))) {
        Ok(pair) => pair,
        Err(s) => return s as i64,
    };
    unsafe { answer(1, filter, out, out_len, || lib.games_request(filter)) }
}

/// What a dump is.
///
/// # Safety
/// As `emusen_library_games`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_library_identity(lib: *mut Library, request: *const u8, len: usize, out: *mut u8, out_len: usize) -> i64 {
    let (lib, request) = match unsafe { handle(lib) }.and_then(|l| unsafe { text(request, len) }.map(|r| (l, r))) {
        Ok(pair) => pair,
        Err(s) => return s as i64,
    };
    unsafe { answer(2, request, out, out_len, || lib.identity_request(request)) }
}

/// One read of the player's records.
///
/// # Safety
/// As `emusen_library_games`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_library_get(lib: *mut Library, request: *const u8, len: usize, out: *mut u8, out_len: usize) -> i64 {
    let (lib, request) = match unsafe { handle(lib) }.and_then(|l| unsafe { text(request, len) }.map(|r| (l, r))) {
        Ok(pair) => pair,
        Err(s) => return s as i64,
    };
    unsafe { answer(3, request, out, out_len, || lib.get_request(request)) }
}

/// One change of the player's records, made once whatever the buffer.
///
/// # Safety
/// As `emusen_library_games`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_library_change(lib: *mut Library, request: *const u8, len: usize, out: *mut u8, out_len: usize) -> i64 {
    let (lib, request) = match unsafe { handle(lib) }.and_then(|l| unsafe { text(request, len) }.map(|r| (l, r))) {
        Ok(pair) => pair,
        Err(s) => return s as i64,
    };
    unsafe { answer(4, request, out, out_len, || lib.change_request(request)) }
}
