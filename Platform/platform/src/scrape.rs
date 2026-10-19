//! The C layer over emusen-scrape, `emusen_scrape.h`: the scraper's stable header, exported from the platform library
//! beside its unstable one, as `emusen_library.h` is. Each call converts its arguments, asks the crate and copies the
//! answer out; nothing is decided here. See EmuSen_Backend.md §30.

use emusen_scrape::{Error, Scraper, abi};
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
    LAST_ERROR.with_borrow_mut(|last| *last = emusen_scrape::redact::redact(&words.into()));
    status
}

fn failed(error: Error) -> i32 {
    fail(error.status(), error.words())
}

unsafe fn text<'a>(data: *const u8, len: usize) -> Result<&'a str, i32> {
    if data.is_null() {
        return if len == 0 { Ok("") } else { Err(fail(abi::NULL, "a text with a length is null")) };
    }
    let bytes = if len == 0 { &[][..] } else { unsafe { std::slice::from_raw_parts(data, len) } };
    std::str::from_utf8(bytes).map_err(|_| fail(abi::BAD_STRING, "the text is not UTF-8"))
}

unsafe fn bytes<'a>(data: *const u8, len: usize) -> &'a [u8] {
    if data.is_null() || len == 0 { &[] } else { unsafe { std::slice::from_raw_parts(data, len) } }
}

unsafe fn handle<'a>(s: *mut Scraper) -> Result<&'a Scraper, i32> {
    if s.is_null() { Err(fail(abi::NULL, "the scraper handle is null")) } else { Ok(unsafe { &*s }) }
}

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
pub extern "C" fn emusen_scrape_abi_version() -> u32 {
    abi::VERSION
}

/// The words of this thread's last failing call, by the length-query idiom; always redacted.
///
/// # Safety
/// `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_scrape_last_error(out: *mut u8, len: usize) -> i64 {
    LAST_ERROR.with_borrow(|words| {
        if !out.is_null() && words.len() <= len {
            unsafe { std::ptr::copy_nonoverlapping(words.as_ptr(), out, words.len()) };
        }
        words.len() as i64
    })
}

/// Opens the media store with the host's SQLite and, when given, its library; the handle in `out`.
///
/// # Safety
/// `config` valid for its `size` bytes and its texts for their lengths; `sqlite` the platform loader's live handle of
/// SQLite and `library` a live `emusen_library` handle or null, each kept alive while this handle lives; `out` valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_scrape_open(config: *const abi::Config, out: *mut *mut Scraper) -> i32 {
    if config.is_null() || out.is_null() {
        return fail(abi::NULL, "the config or the handle's place is null");
    }
    let config = unsafe { &*config };
    if (config.size as usize) < std::mem::size_of::<abi::Config>() {
        return fail(abi::NULL, "the config is shorter than version 1.0's");
    }
    let root = match unsafe { text(config.root, config.root_len) } {
        Ok(p) if !p.is_empty() => p,
        Ok(_) => return fail(abi::NULL, "no media store was named"),
        Err(s) => return s,
    };
    let settings = match unsafe { text(config.settings, config.settings_len) } {
        Ok(s) => s,
        Err(s) => return s,
    };
    if config.sqlite.is_null() {
        return fail(abi::NO_SQLITE, "no SQLite was lent to the scraper");
    }
    let sqlite = match unsafe { Sqlite::lent(config.sqlite as *mut c_void) } {
        Ok(s) => Arc::new(s),
        Err(words) => return fail(abi::NO_SQLITE, words),
    };
    let library: Option<&'static emusen_library::Library> = if config.library.is_null() { None } else { Some(unsafe { &*(config.library as *const emusen_library::Library) }) };
    match Scraper::open(sqlite, root, library, unsafe { bytes(config.embedded, config.embedded_len) }, settings) {
        Ok(s) => {
            unsafe { *out = Box::into_raw(Box::new(s)) };
            0
        }
        Err(e) => failed(e),
    }
}

/// Closes a handle: a run is cancelled and its workers joined first.
///
/// # Safety
/// `s` from `emusen_scrape_open`, not closed before, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_scrape_close(s: *mut Scraper) -> i32 {
    if s.is_null() {
        return fail(abi::NULL, "the scraper handle is null");
    }
    drop(unsafe { Box::from_raw(s) });
    0
}

/// One read, `{"what": ...}`; the answer's length, or a status.
///
/// # Safety
/// `s` a live handle; `request` valid for `len` bytes; `out` valid for `out_len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_scrape_get(s: *mut Scraper, request: *const u8, len: usize, out: *mut u8, out_len: usize) -> i64 {
    let scraper = match unsafe { handle(s) } {
        Ok(h) => h,
        Err(st) => return st as i64,
    };
    let request = match unsafe { text(request, len) } {
        Ok(r) => r,
        Err(st) => return st as i64,
    };
    unsafe { answer(1, request, out, out_len, || scraper.get_request(request)) }
}

/// One change, `{"op": ...}`; the answer's length, or a status.
///
/// # Safety
/// As `emusen_scrape_get`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_scrape_change(s: *mut Scraper, request: *const u8, len: usize, out: *mut u8, out_len: usize) -> i64 {
    let scraper = match unsafe { handle(s) } {
        Ok(h) => h,
        Err(st) => return st as i64,
    };
    let request = match unsafe { text(request, len) } {
        Ok(r) => r,
        Err(st) => return st as i64,
    };
    unsafe { answer(2, request, out, out_len, || scraper.change_request(request)) }
}

/// Starts a run over the request's games; how many are queued, or a status. Zero starts nothing.
///
/// # Safety
/// `s` a live handle; `request` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_scrape_start(s: *mut Scraper, request: *const u8, len: usize) -> i64 {
    let scraper = match unsafe { handle(s) } {
        Ok(h) => h,
        Err(st) => return st as i64,
    };
    let request = match unsafe { text(request, len) } {
        Ok(r) => r,
        Err(st) => return st as i64,
    };
    match scraper.start_request(request) {
        Ok(n) => n as i64,
        Err(e) => failed(e) as i64,
    }
}

/// The run as it stands, `{"since"}` naming the last result already drained; the answer's length, or a status.
///
/// # Safety
/// As `emusen_scrape_get`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_scrape_progress(s: *mut Scraper, request: *const u8, len: usize, out: *mut u8, out_len: usize) -> i64 {
    let scraper = match unsafe { handle(s) } {
        Ok(h) => h,
        Err(st) => return st as i64,
    };
    let request = match unsafe { text(request, len) } {
        Ok(r) => r,
        Err(st) => return st as i64,
    };
    unsafe { answer(3, request, out, out_len, || scraper.progress_request(request)) }
}

/// Waits until the run's serial passes `serial`, or the timeout ends; the serial then.
///
/// # Safety
/// `s` a live handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_scrape_wait(s: *mut Scraper, serial: u64, timeout_ms: u32) -> i64 {
    match unsafe { handle(s) } {
        Ok(h) => h.wait(serial, timeout_ms) as i64,
        Err(st) => st as i64,
    }
}

/// Pauses or resumes the run's ScreenScraper requests: 1 when it changed, 0 when it did not.
///
/// # Safety
/// `s` a live handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_scrape_pause(s: *mut Scraper, paused: i32) -> i32 {
    match unsafe { handle(s) } {
        Ok(h) => h.pause(paused != 0) as i32,
        Err(st) => st,
    }
}

/// Cancels the run: 1 when one was going, 0 when none was.
///
/// # Safety
/// `s` a live handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_scrape_cancel(s: *mut Scraper) -> i32 {
    match unsafe { handle(s) } {
        Ok(h) => h.cancel() as i32,
        Err(st) => st,
    }
}
