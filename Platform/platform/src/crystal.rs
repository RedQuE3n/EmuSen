//! The frontend API's exports, `emusen_crystal.h`, over `emusen-crystal`: each converts its arguments, calls the
//! runtime, and copies the answer out by the length query. Exported as part B7 builds them; the binding reaches them
//! only once every export of major 1 is here (EmuSen_Backend.md §29.3).

use std::cell::RefCell;
use emusen_crystal::abi::{self, emusen_crystal_config, emusen_crystal_frame_info, emusen_crystal_open_params, emusen_crystal_raw_info, emusen_crystal_runtime, emusen_crystal_subscription};
use emusen_crystal::runtime::{Config, Runtime};
use emusen_session_service::abi::status as svc;

thread_local! {
    static LAST_ERROR: RefCell<String> = const { RefCell::new(String::new()) };
}

pub(crate) fn fail(status: i32, words: impl Into<String>) -> i32 {
    LAST_ERROR.with_borrow_mut(|last| *last = words.into());
    status
}

/// The length-query idiom.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
pub(crate) unsafe fn put(bytes: &[u8], out: *mut u8, len: usize) -> i64 {
    if !out.is_null() && bytes.len() <= len {
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, bytes.len()) };
    }
    bytes.len() as i64
}

/// # Safety
/// `data` valid for `len` bytes, or null.
unsafe fn text<'a>(data: *const u8, len: usize) -> Result<Option<&'a str>, i32> {
    if data.is_null() {
        return if len == 0 { Ok(None) } else { Err(fail(svc::NULL, "a null string with a length")) };
    }
    let bytes = if len == 0 { &[][..] } else { unsafe { std::slice::from_raw_parts(data, len) } };
    std::str::from_utf8(bytes).map(Some).map_err(|_| fail(svc::BAD_STRING, "text that is not UTF-8"))
}

/// # Safety
/// `rt` a runtime this library made and has not freed, or null.
pub(crate) unsafe fn runtime<'a>(rt: *mut emusen_crystal_runtime) -> Result<&'a Runtime, i32> {
    if rt.is_null() {
        return Err(fail(svc::NULL, "a null runtime"));
    }
    Ok(unsafe { &*(rt as *const Runtime) })
}

const BUILD: &str = env!("CARGO_PKG_VERSION");

#[unsafe(no_mangle)]
pub extern "C" fn emusen_crystal_abi_version() -> u32 {
    abi::VERSION
}

/// # Safety
/// `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_version_text(out: *mut u8, len: usize) -> i64 {
    let words = format!("EmuSen's Rust implementation of the frontend API {}.{}, emusen-crystal {BUILD}", abi::MAJOR, abi::MINOR);
    unsafe { put(words.as_bytes(), out, len) }
}

/// # Safety
/// `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_last_error(_rt: *mut emusen_crystal_runtime, out: *mut u8, len: usize) -> i64 {
    LAST_ERROR.with_borrow(|words| unsafe { put(words.as_bytes(), out, len) })
}

/// The config's directories, a JSON array of paths.
fn cores_of(text: &str) -> Result<Vec<String>, i32> {
    let refused = || fail(svc::BAD_ARGUMENT, "config's cores must be a JSON array of directories");
    match emusen_dotnet::json::parse(text) {
        Ok(emusen_dotnet::json::Value::Array(items)) => items.iter().map(|v| v.as_str().map(str::to_string).ok_or_else(refused)).collect(),
        _ => Err(refused()),
    }
}

/// A URL's body through the platform's one HTTP client, within `CheatDatabaseInstaller.FetchTimeout`'s five minutes.
fn fetch(url: &str) -> Result<Vec<u8>, String> {
    let request = emusen_net::Request::new(url, std::time::Duration::from_secs(300));
    emusen_net::fetch(&request, &emusen_net::Progress::default()).map_err(|f| f.to_string())
}

/// A runtime; null on failure, the words in `last_error`.
///
/// # Safety
/// `config` null or valid for its `size` bytes, its texts valid for their lengths.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_new(config: *const emusen_crystal_config) -> *mut emusen_crystal_runtime {
    let made = || -> Result<Runtime, i32> {
        let mut c = Config { lender: Some(std::sync::Arc::new(SerenityLender)), fetch: Some(std::sync::Arc::new(fetch)), ..Config::default() };
        if !config.is_null() {
            let size = unsafe { (*config).size } as usize;
            if size < 8 {
                return Err(fail(abi::status::BAD_STRUCT, format!("emusen_crystal_config of {size} bytes; version 1.0's is 8")));
            }
            let flags = unsafe { (*config).flags };
            if flags & !abi::CONFIG_DEVELOPMENT != 0 {
                return Err(fail(svc::NOT_SUPPORTED, format!("config flags 0x{flags:X} are not known to 1.{}", abi::MINOR)));
            }
            if flags & abi::CONFIG_DEVELOPMENT != 0 {
                c.development = Some(true);
            }
            if size >= std::mem::size_of::<emusen_crystal_config>() {
                let lent = unsafe { (*config).sqlite };
                if !lent.is_null() {
                    let sqlite = unsafe { emusen_sqlite::Sqlite::lent(lent) }.map_err(|why| fail(svc::BAD_ARGUMENT, format!("the SQLite lent could not be used: {why}")))?;
                    c.sqlite = Some(std::sync::Arc::new(sqlite));
                }
                let (cores, cores_len) = unsafe { ((*config).cores, (*config).cores_len) };
                if let Some(t) = unsafe { text(cores, cores_len) }? {
                    c.cores = Some(cores_of(t)?);
                }
            }
        }
        Ok(Runtime::new(&c))
    };
    match made() {
        Ok(rt) => Box::into_raw(Box::new(rt)) as *mut emusen_crystal_runtime,
        Err(_) => std::ptr::null_mut(),
    }
}

/// # Safety
/// `rt` a runtime this library made and has not freed, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_free(rt: *mut emusen_crystal_runtime) -> i32 {
    if rt.is_null() {
        return fail(svc::NULL, "a null runtime");
    }
    crate::crystal_twins::forget(rt);
    drop(unsafe { Box::from_raw(rt as *mut Runtime) });
    0
}

macro_rules! listing {
    ($name:ident, $method:ident) => {
        /// # Safety
        /// `rt` a live runtime; `out` valid for `len` bytes, or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(rt: *mut emusen_crystal_runtime, out: *mut u8, len: usize) -> i64 {
            match unsafe { runtime(rt) } {
                Ok(r) => unsafe { put(r.$method().as_bytes(), out, len) },
                Err(status) => status as i64,
            }
        }
    };
}

listing!(emusen_crystal_systems, systems_json);
listing!(emusen_crystal_shelves, shelves_json);
listing!(emusen_crystal_engines, engines_json);
listing!(emusen_crystal_cores, cores_json);
listing!(emusen_crystal_sessions, sessions_json);

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_rescan(rt: *mut emusen_crystal_runtime) -> i32 {
    match unsafe { runtime(rt) } {
        Ok(r) => {
            r.rescan();
            0
        }
        Err(status) => status,
    }
}

/// # Safety
/// `rt` a live runtime; `path` valid for `path_len` bytes; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_route(rt: *mut emusen_crystal_runtime, path: *const u8, path_len: usize, out: *mut u8, len: usize) -> i64 {
    let answer = || -> Result<String, i32> {
        let r = unsafe { runtime(rt) }?;
        let at = unsafe { text(path, path_len) }?.filter(|p| !p.is_empty()).ok_or_else(|| fail(svc::BAD_ARGUMENT, "the path must be given"))?;
        Ok(r.route_json(at))
    };
    match answer() {
        Ok(json) => unsafe { put(json.as_bytes(), out, len) },
        Err(status) => status as i64,
    }
}

/// # Safety
/// `rt` a live runtime; each text valid for its length; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_firmware_needed(rt: *mut emusen_crystal_runtime, path: *const u8, path_len: usize, engine: *const u8, engine_len: usize, out: *mut u8, len: usize) -> i64 {
    let answer = || -> Result<String, i32> {
        let r = unsafe { runtime(rt) }?;
        let at = unsafe { text(path, path_len) }?.filter(|p| !p.is_empty()).ok_or_else(|| fail(svc::BAD_ARGUMENT, "the path must be given"))?;
        let engine = unsafe { text(engine, engine_len) }?;
        Ok(emusen_crystal::firmware::needed_json(r, at, engine))
    };
    match answer() {
        Ok(json) => unsafe { put(json.as_bytes(), out, len) },
        Err(status) => status as i64,
    }
}

/// # Safety
/// `rt` a live runtime; each text valid for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_firmware_install(rt: *mut emusen_crystal_runtime, request: *const u8, request_len: usize, source: *const u8, source_len: usize) -> i32 {
    let done = || -> Result<(), i32> {
        let r = unsafe { runtime(rt) }?;
        if request.is_null() && request_len > 0 {
            return Err(fail(svc::NULL, "a null buffer with a length"));
        }
        let bytes = if request.is_null() { &[][..] } else { unsafe { std::slice::from_raw_parts(request, request_len) } };
        let request = std::str::from_utf8(bytes).map_err(|_| fail(svc::BAD_ARGUMENT, "the argument is not JSON: it is not UTF-8"))?;
        let source = unsafe { text(source, source_len) }?.filter(|p| !p.is_empty()).ok_or_else(|| fail(svc::BAD_ARGUMENT, "the source file must be given"))?;
        let tree = emusen_galaxia::tree::tree();
        emusen_crystal::firmware::install(&tree, request, source, &|line| r.say(line)).map_err(|(status, words)| fail(status, words))
    };
    match done() {
        Ok(()) => 0,
        Err(status) => status,
    }
}

/// # Safety
/// `rt` a live runtime; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_firmware_overview(rt: *mut emusen_crystal_runtime, out: *mut u8, len: usize) -> i64 {
    match unsafe { runtime(rt) } {
        Ok(r) => unsafe { put(emusen_crystal::firmware::overview_json(r).as_bytes(), out, len) },
        Err(status) => status as i64,
    }
}

/// # Safety
/// `rt` a live runtime; `args` valid for `args_len` bytes; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_cards_op(rt: *mut emusen_crystal_runtime, op: u32, args: *const u8, args_len: usize, out: *mut u8, len: usize) -> i64 {
    let answer = || -> Result<String, i32> {
        let r = unsafe { runtime(rt) }?;
        if args.is_null() && args_len > 0 {
            return Err(fail(svc::NULL, "a null buffer with a length"));
        }
        let bytes = if args.is_null() { &[][..] } else { unsafe { std::slice::from_raw_parts(args, args_len) } };
        let args = std::str::from_utf8(bytes).map_err(|_| fail(svc::BAD_ARGUMENT, "the argument is not JSON: it is not UTF-8"))?;
        let discovery = r.discovery();
        let catalog = emusen_crystal::catalog::Catalog::new(&discovery);
        let tree = emusen_galaxia::tree::tree();
        emusen_crystal::media::cards_op(&catalog, &tree, op, args).map_err(|(status, words)| fail(status, words))
    };
    match answer() {
        Ok(json) => unsafe { put(json.as_bytes(), out, len) },
        Err(status) => status as i64,
    }
}

fn status_of(result: Result<(), (i32, String)>) -> i32 {
    match result {
        Ok(()) => 0,
        Err((status, words)) => fail(status, words),
    }
}

fn copy_of(result: Result<String, (i32, String)>, out: *mut u8, len: usize) -> i64 {
    match result {
        Ok(json) => unsafe { put(json.as_bytes(), out, len) },
        Err((status, words)) => fail(status, words) as i64,
    }
}

/// # Safety
/// `rt` a live runtime; `params` valid for its `size` bytes, its texts for their lengths.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_open(rt: *mut emusen_crystal_runtime, params: *const emusen_crystal_open_params) -> i64 {
    let opened = || -> Result<u32, i32> {
        let r = unsafe { runtime(rt) }?;
        if params.is_null() {
            return Err(fail(svc::NULL, "null open parameters"));
        }
        let p = unsafe { &*params };
        if p.size < 64 {
            return Err(fail(abi::status::BAD_STRUCT, format!("emusen_crystal_open_params of {} bytes; version 1.0's is 64", p.size)));
        }
        let known = abi::OPEN_NO_BATTERY | abi::OPEN_SINGLE_ROWS;
        if p.flags & !known != 0 {
            return Err(fail(svc::NOT_SUPPORTED, format!("open flags 0x{:X} are not known to 1.{}", p.flags & !known, abi::MINOR)));
        }
        let asked = emusen_crystal::open::Asked {
            path: unsafe { text(p.path, p.path_len) }?,
            console: unsafe { text(p.console, p.console_len) }?,
            engine: unsafe { text(p.engine, p.engine_len) }?,
            opened: emusen_crystal::session::OpenedWith {
                flags: p.flags,
                audio_buffer_max_samples: p.audio_buffer_max_samples,
                device: if p.size as usize >= std::mem::size_of::<emusen_crystal_open_params>() { p.device as usize } else { 0 },
            },
        };
        r.open(&asked).map_err(|(status, words)| fail(status, words))
    };
    match opened() {
        Ok(id) => id as i64,
        Err(status) => status as i64,
    }
}

macro_rules! session_call {
    ($name:ident, $method:ident) => {
        /// # Safety
        /// `rt` a live runtime.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(rt: *mut emusen_crystal_runtime, session: u32) -> i32 {
            match unsafe { runtime(rt) } {
                Ok(r) => status_of(r.$method(session)),
                Err(status) => status,
            }
        }
    };
}

session_call!(emusen_crystal_close, close);
session_call!(emusen_crystal_reset, reset);
session_call!(emusen_crystal_disc_eject, disc_eject);

macro_rules! session_read {
    ($name:ident, $method:ident) => {
        /// # Safety
        /// `rt` a live runtime; `out` valid for `len` bytes, or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(rt: *mut emusen_crystal_runtime, session: u32, out: *mut u8, len: usize) -> i64 {
            match unsafe { runtime(rt) } {
                Ok(r) => copy_of(r.$method(session), out, len),
                Err(status) => status as i64,
            }
        }
    };
}

session_read!(emusen_crystal_session_info, session_info);
session_read!(emusen_crystal_save_media, save_media);

/// # Safety
/// `rt` a live runtime; `path` valid for `path_len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_disc_insert(rt: *mut emusen_crystal_runtime, session: u32, path: *const u8, path_len: usize) -> i32 {
    let done = || -> Result<(), i32> {
        let r = unsafe { runtime(rt) }?;
        r.sessions.get(session).map_err(|(s, w)| fail(s, w))?;
        let at = unsafe { text(path, path_len) }?.filter(|p| !p.is_empty()).ok_or_else(|| fail(svc::BAD_ARGUMENT, "the disc's path must be given"))?;
        r.disc_insert(session, at).map_err(|(s, w)| fail(s, w))
    };
    match done() {
        Ok(()) => 0,
        Err(status) => status,
    }
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_disc_change(rt: *mut emusen_crystal_runtime, session: u32, index: u32) -> i32 {
    match unsafe { runtime(rt) } {
        Ok(r) => status_of(r.disc_change(session, index)),
        Err(status) => status,
    }
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_disc_lid(rt: *mut emusen_crystal_runtime, session: u32, open: u32) -> i32 {
    match unsafe { runtime(rt) } {
        Ok(r) => status_of(r.disc_lid(session, open != 0)),
        Err(status) => status,
    }
}

/// The session a call names, the runtime's current one for 0.
unsafe fn session_of<'a>(rt: *mut emusen_crystal_runtime, session: u32) -> Result<std::sync::Arc<emusen_crystal::session::Session>, i32> {
    let r: &'a Runtime = unsafe { runtime(rt) }?;
    r.sessions.get(session).map_err(|(s, w)| fail(s, w))
}

fn done(result: Result<(), (i32, String)>) -> Result<(), i32> {
    result.map_err(|(s, w)| fail(s, w))
}

fn code(result: Result<(), i32>) -> i32 {
    match result {
        Ok(()) => 0,
        Err(status) => status,
    }
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for its `size` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_frame(rt: *mut emusen_crystal_runtime, session: u32, out: *mut emusen_crystal_frame_info) -> i32 {
    code((|| {
        let s = unsafe { session_of(rt, session) }?;
        done(s.frame(unsafe { out.as_mut() }))
    })())
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_picture_copy(rt: *mut emusen_crystal_runtime, session: u32, out: *mut u8, len: usize) -> i64 {
    let copied = || -> Result<i64, i32> {
        let s = unsafe { session_of(rt, session) }?;
        let into: &mut [u8] = if out.is_null() { &mut [] } else { unsafe { std::slice::from_raw_parts_mut(out, len.min(i32::MAX as usize)) } };
        s.copy_picture(into).map(|n| n as i64).map_err(|(st, w)| fail(st, w))
    };
    copied().unwrap_or_else(|status| status as i64)
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `frames` stereo frames; `rate` null or valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_audio_drain(rt: *mut emusen_crystal_runtime, session: u32, out: *mut i16, frames: usize, rate: *mut u32) -> i64 {
    let drained = || -> Result<i64, i32> {
        let s = unsafe { session_of(rt, session) }?;
        let into = if out.is_null() { None } else { Some(unsafe { std::slice::from_raw_parts_mut(out, frames.saturating_mul(2).min(i32::MAX as usize)) }) };
        let (n, r) = s.drain_audio(into).map_err(|(st, w)| fail(st, w))?;
        if !rate.is_null() {
            unsafe { *rate = r };
        }
        Ok(n)
    };
    drained().unwrap_or_else(|status| status as i64)
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_wait(rt: *mut emusen_crystal_runtime, session: u32, timeout_ms: u32) -> i32 {
    match unsafe { session_of(rt, session) } {
        Ok(s) => s.wait(timeout_ms) as i32,
        Err(status) => status,
    }
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_phase_names(rt: *mut emusen_crystal_runtime, session: u32, out: *mut u8, len: usize) -> i64 {
    match unsafe { session_of(rt, session) } {
        Ok(s) => copy_of(s.phase_names_json(), out, len),
        Err(status) => status as i64,
    }
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `count` values.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_phases(rt: *mut emusen_crystal_runtime, session: u32, out: *mut f64, count: usize) -> i64 {
    let timed = || -> Result<i64, i32> {
        let s = unsafe { session_of(rt, session) }?;
        let into: &mut [f64] = if out.is_null() { &mut [] } else { unsafe { std::slice::from_raw_parts_mut(out, count) } };
        s.phases(into).map(|n| n as i64).map_err(|(st, w)| fail(st, w))
    };
    timed().unwrap_or_else(|status| status as i64)
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_input_button(rt: *mut emusen_crystal_runtime, session: u32, port: u32, control: u32, pressed: u32) -> i32 {
    code((|| done(unsafe { session_of(rt, session) }?.button(port, control, pressed != 0)))())
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_input_axis(rt: *mut emusen_crystal_runtime, session: u32, port: u32, axis: u32, value: f64) -> i32 {
    code((|| done(unsafe { session_of(rt, session) }?.axis(port, axis, value)))())
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_input_connected(rt: *mut emusen_crystal_runtime, session: u32, port: u32, connected: u32) -> i32 {
    code((|| {
        unsafe { session_of(rt, session) }?.connected(port, connected != 0);
        Ok(())
    })())
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_speed_set(rt: *mut emusen_crystal_runtime, session: u32, percent: u32, turbo_percent: u32, slow_percent: u32, flags: u32) -> i32 {
    code((|| done(unsafe { session_of(rt, session) }?.speed_set(percent, turbo_percent, slow_percent, flags)))())
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_speed_mode(rt: *mut emusen_crystal_runtime, session: u32, mode: u32) -> i32 {
    code((|| done(unsafe { session_of(rt, session) }?.speed_mode(mode)))())
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_rewind_set(rt: *mut emusen_crystal_runtime, session: u32, enabled: u32, interval_frames: u32, budget_bytes: u64, thumbnail_width: u32) -> i32 {
    code((|| done(unsafe { session_of(rt, session) }?.rewind_set(enabled != 0, interval_frames, budget_bytes, thumbnail_width)))())
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for its `size` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_rewind_step(rt: *mut emusen_crystal_runtime, session: u32, count: u32, out: *mut emusen_crystal_frame_info) -> i64 {
    let stepped = || -> Result<i64, i32> {
        let r = unsafe { runtime(rt) }?;
        let s = r.sessions.get(session).map_err(|(st, w)| fail(st, w))?;
        r.with_ctx(|ctx| s.rewind_step(count, unsafe { out.as_mut() }, ctx)).map_err(|(st, w)| fail(st, w))
    };
    stepped().unwrap_or_else(|status| status as i64)
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_rewind_reel(rt: *mut emusen_crystal_runtime, session: u32, out: *mut u8, len: usize) -> i64 {
    match unsafe { session_of(rt, session) } {
        Ok(s) => copy_of(s.rewind_reel_json(), out, len),
        Err(status) => status as i64,
    }
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_rewind_picture(rt: *mut emusen_crystal_runtime, session: u32, frame: u64, out: *mut u8, len: usize) -> i64 {
    match unsafe { session_of(rt, session) }.and_then(|s| s.rewind_picture(frame).map_err(|(st, w)| fail(st, w))) {
        Ok(bytes) => unsafe { put(&bytes, out, len) },
        Err(status) => status as i64,
    }
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_rewind_to(rt: *mut emusen_crystal_runtime, session: u32, frame: u64) -> i32 {
    code((|| {
        let r = unsafe { runtime(rt) }?;
        let s = r.sessions.get(session).map_err(|(st, w)| fail(st, w))?;
        done(r.with_ctx(|ctx| s.rewind_to(frame, ctx)))
    })())
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_state_slots(rt: *mut emusen_crystal_runtime, session: u32, out: *mut u8, len: usize) -> i64 {
    match unsafe { session_of(rt, session) } {
        Ok(s) => {
            let json = s.slots_json(&emusen_galaxia::tree::tree());
            unsafe { put(json.as_bytes(), out, len) }
        }
        Err(status) => status as i64,
    }
}

macro_rules! state_call {
    ($name:ident, $call:expr) => {
        /// # Safety
        /// `rt` a live runtime.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(rt: *mut emusen_crystal_runtime, session: u32) -> i32 {
            code((|| {
                let r = unsafe { runtime(rt) }?;
                let s = r.sessions.get(session).map_err(|(st, w)| fail(st, w))?;
                done(r.with_records(|records| r.with_ctx(|ctx| $call(&s, records, ctx))))
            })())
        }
    };
}

state_call!(emusen_crystal_resume_save, |s: &emusen_crystal::session::Session, rec, ctx| s.save_resume(rec, ctx));
state_call!(emusen_crystal_resume_load, |s: &emusen_crystal::session::Session, rec, ctx| s.load_resume(rec, ctx));

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_state_slot_save(rt: *mut emusen_crystal_runtime, session: u32, slot: u32) -> i32 {
    code((|| {
        let r = unsafe { runtime(rt) }?;
        let s = r.sessions.get(session).map_err(|(st, w)| fail(st, w))?;
        done(r.with_records(|records| r.with_ctx(|ctx| s.save_slot(slot, records, ctx))))
    })())
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_state_slot_load(rt: *mut emusen_crystal_runtime, session: u32, slot: u32) -> i32 {
    code((|| {
        let r = unsafe { runtime(rt) }?;
        let s = r.sessions.get(session).map_err(|(st, w)| fail(st, w))?;
        done(r.with_records(|records| r.with_ctx(|ctx| s.load_slot(slot, records, ctx))))
    })())
}

/// # Safety
/// `rt` a live runtime; `path` valid for `path_len` bytes; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_state_record(rt: *mut emusen_crystal_runtime, path: *const u8, path_len: usize, out: *mut u8, len: usize) -> i64 {
    let read = || -> Result<String, i32> {
        let r = unsafe { runtime(rt) }?;
        let at = unsafe { text(path, path_len) }?.filter(|p| !p.is_empty()).ok_or_else(|| fail(svc::BAD_ARGUMENT, "the path must be given"))?;
        r.state_record_json(at).map_err(|(st, w)| fail(st, w))
    };
    match read() {
        Ok(json) => unsafe { put(json.as_bytes(), out, len) },
        Err(status) => status as i64,
    }
}

/// # Safety
/// `rt` a live runtime; `path` valid for `path_len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_state_forget(rt: *mut emusen_crystal_runtime, path: *const u8, path_len: usize) -> i32 {
    code((|| {
        let r = unsafe { runtime(rt) }?;
        let at = unsafe { text(path, path_len) }?.filter(|p| !p.is_empty()).ok_or_else(|| fail(svc::BAD_ARGUMENT, "the path must be given"))?;
        done(r.state_forget(at))
    })())
}

/// A subscription's handle: its subscription and the events it was taken from.
struct Held {
    sub: std::sync::Arc<emusen_crystal::events::Subscription>,
    events: std::sync::Arc<emusen_crystal::events::Events>,
}

/// The handles given out and not yet ended, so a handle is checked before it is read.
static HANDLES: std::sync::Mutex<Vec<usize>> = std::sync::Mutex::new(Vec::new());

/// # Safety
/// `sub` a handle this library gave, or anything: it is looked up before it is read.
unsafe fn subscription<'a>(sub: *mut emusen_crystal_subscription) -> Result<&'a Held, i32> {
    if sub.is_null() {
        return Err(fail(svc::NULL, "a null subscription"));
    }
    let known = HANDLES.lock().unwrap_or_else(|p| p.into_inner()).contains(&(sub as usize));
    let held = if known { Some(unsafe { &*(sub as *const Held) }) } else { None };
    match held {
        Some(h) if h.sub.live.load(std::sync::atomic::Ordering::Acquire) => Ok(h),
        _ => Err(fail(svc::NULL, "not a subscription")),
    }
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_subscribe(rt: *mut emusen_crystal_runtime, session: u32, kinds: u32) -> *mut emusen_crystal_subscription {
    let made = || -> Result<*mut emusen_crystal_subscription, i32> {
        let r = unsafe { runtime(rt) }?;
        if session != 0 {
            r.sessions.get(session).map_err(|(st, w)| fail(st, w))?;
        }
        let held = Box::new(Held { sub: r.events.subscribe(session, kinds), events: r.events.clone() });
        let handle = Box::into_raw(held) as *mut emusen_crystal_subscription;
        HANDLES.lock().unwrap_or_else(|p| p.into_inner()).push(handle as usize);
        Ok(handle)
    };
    made().unwrap_or(std::ptr::null_mut())
}

/// # Safety
/// `sub` a subscription's handle; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_subscription_drain(sub: *mut emusen_crystal_subscription, out: *mut u8, len: usize) -> i64 {
    match unsafe { subscription(sub) } {
        Ok(h) => {
            let into = if out.is_null() { None } else { Some(unsafe { std::slice::from_raw_parts_mut(out, len) }) };
            h.sub.drain(into) as i64
        }
        Err(status) => status as i64,
    }
}

/// # Safety
/// `sub` a subscription's handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_subscription_wait(sub: *mut emusen_crystal_subscription, timeout_ms: u32) -> i32 {
    match unsafe { subscription(sub) } {
        Ok(h) => h.sub.wait(timeout_ms) as i32,
        Err(status) => status,
    }
}

/// # Safety
/// `sub` a subscription's handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_unsubscribe(sub: *mut emusen_crystal_subscription) -> i32 {
    match unsafe { subscription(sub) } {
        Ok(h) => {
            h.events.remove(&h.sub);
            HANDLES.lock().unwrap_or_else(|p| p.into_inner()).retain(|x| *x != sub as usize);
            drop(unsafe { Box::from_raw(sub as *mut Held) });
            0
        }
        Err(status) => status,
    }
}

/// A required text, refused as the C#'s `Required` refuses it.
unsafe fn required<'a>(data: *const u8, len: usize, what: &str) -> Result<&'a str, i32> {
    unsafe { text(data, len) }?.filter(|t| !t.is_empty()).ok_or_else(|| fail(svc::BAD_ARGUMENT, format!("{what} must be given")))
}

/// # Safety
/// `rt` a live runtime; each text valid for its length; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_settings_schema(rt: *mut emusen_crystal_runtime, session: u32, console: *const u8, console_len: usize, engine: *const u8, engine_len: usize, out: *mut u8, len: usize) -> i64 {
    let read = || -> Result<String, i32> {
        let r = unsafe { runtime(rt) }?;
        let (c, e) = (unsafe { text(console, console_len) }?, unsafe { text(engine, engine_len) }?);
        r.settings_schema_json(session, c, e).map_err(|(st, w)| fail(st, w))
    };
    match read() {
        Ok(json) => unsafe { put(json.as_bytes(), out, len) },
        Err(status) => status as i64,
    }
}

/// # Safety
/// `rt` a live runtime; each text valid for its length; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_settings_get(rt: *mut emusen_crystal_runtime, session: u32, scope: u32, console: *const u8, console_len: usize, key: *const u8, key_len: usize, out: *mut u8, len: usize) -> i64 {
    let read = || -> Result<String, i32> {
        let k = unsafe { required(key, key_len, "the key") }?;
        let r = unsafe { runtime(rt) }?;
        let c = unsafe { text(console, console_len) }?;
        r.settings_get(session, scope, c, k).map_err(|(st, w)| fail(st, w))
    };
    match read() {
        Ok(value) => unsafe { put(value.as_bytes(), out, len) },
        Err(status) => status as i64,
    }
}

/// # Safety
/// `rt` a live runtime; each text valid for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_settings_set(rt: *mut emusen_crystal_runtime, session: u32, scope: u32, console: *const u8, console_len: usize, key: *const u8, key_len: usize, value: *const u8, value_len: usize) -> i32 {
    code((|| {
        let r = unsafe { runtime(rt) }?;
        let (c, v) = (unsafe { text(console, console_len) }?, unsafe { text(value, value_len) }?);
        let k = unsafe { required(key, key_len, "the key") }?;
        done(r.settings_set(session, scope, c, k, v))
    })())
}

/// # Safety
/// `rt` a live runtime; each text valid for its length; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_game_setting_get(rt: *mut emusen_crystal_runtime, path: *const u8, path_len: usize, key: *const u8, key_len: usize, out: *mut u8, len: usize) -> i64 {
    let read = || -> Result<String, i32> {
        let r = unsafe { runtime(rt) }?;
        let (p, k) = (unsafe { required(path, path_len, "the path") }?, unsafe { required(key, key_len, "the key") }?);
        r.game_setting_get(p, k).map_err(|(st, w)| fail(st, w))
    };
    match read() {
        Ok(value) => unsafe { put(value.as_bytes(), out, len) },
        Err(status) => status as i64,
    }
}

/// # Safety
/// `rt` a live runtime; each text valid for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_game_setting_set(rt: *mut emusen_crystal_runtime, path: *const u8, path_len: usize, key: *const u8, key_len: usize, value: *const u8, value_len: usize) -> i32 {
    code((|| {
        let r = unsafe { runtime(rt) }?;
        let (p, k) = (unsafe { required(path, path_len, "the path") }?, unsafe { required(key, key_len, "the key") }?);
        let v = unsafe { text(value, value_len) }?;
        done(r.game_setting_set(p, k, v))
    })())
}

/// The runtime's lender over Serenity's lend of the presenter's device, `PlatformLend`.
struct SerenityLender;

/// The platform library's last words, as `emusen_platform_last_error` gives them.
fn platform_words() -> String {
    let mut words = vec![0u8; 4096];
    let n = unsafe { crate::emusen_platform_last_error(words.as_mut_ptr(), words.len()) }.clamp(0, 4096) as usize;
    String::from_utf8_lossy(&words[..n]).into_owned()
}

impl emusen_crystal::lend::Lender for SerenityLender {
    fn make(&self, device: usize) -> Result<(usize, emusen_native::core::sys::DeviceLend), String> {
        let mut info = crate::serenity_device::LendInfo {
            size: std::mem::size_of::<crate::serenity_device::LendInfo>() as u32,
            api_version: 0,
            get_instance_proc_addr: 0,
            instance: 0,
            physical_device: 0,
            device: 0,
            queue_family: 0,
            queue_index: 0,
            host_queue_family: 0,
            reserved: 0,
            done: 0,
        };
        let made = unsafe { crate::serenity_device::emusen_serenity_lend_new(device as *const _, &mut info) };
        if made.is_null() {
            return Err(platform_words());
        }
        let proc_addr: Option<unsafe extern "C" fn()> = if info.get_instance_proc_addr == 0 { None } else { Some(unsafe { std::mem::transmute::<usize, unsafe extern "C" fn()>(info.get_instance_proc_addr as usize) }) };
        let lend = emusen_native::core::sys::DeviceLend {
            size: std::mem::size_of::<emusen_native::core::sys::DeviceLend>() as u32,
            api_version: info.api_version,
            get_instance_proc_addr: proc_addr,
            instance: info.instance,
            physical_device: info.physical_device,
            device: info.device,
            queue_family: info.queue_family,
            queue_index: info.queue_index,
            host_queue_family: info.host_queue_family,
            reserved: 0,
            done: info.done,
        };
        Ok((made as usize, lend))
    }

    fn take(&self, lend: usize, f: &emusen_native::core::sys::DeviceFrame) -> Result<bool, String> {
        match unsafe { crate::serenity_device::emusen_serenity_lend_take(lend as *const _, f.buffer, f.offset, f.width, f.height, f.stride, f.ready, f.value) } {
            status if status < 0 => Err(platform_words()),
            status => Ok(status == 1),
        }
    }

    fn settle(&self, lend: usize) -> Result<(), String> {
        if unsafe { crate::serenity_device::emusen_serenity_lend_settle(lend as *const _) } < 0 { Err(platform_words()) } else { Ok(()) }
    }

    fn free(&self, lend: usize) {
        unsafe { crate::serenity_device::emusen_serenity_lend_free(lend as *mut _) };
    }
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_picture_lend(rt: *mut emusen_crystal_runtime, session: u32) -> *const () {
    let lent = || -> Result<usize, i32> { unsafe { session_of(rt, session) }?.picture_lend().map_err(|(st, w)| fail(st, w)) };
    lent().unwrap_or(0) as *const ()
}

/// # Safety
/// `rt` a live runtime; `encoding` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_raw_select(rt: *mut emusen_crystal_runtime, session: u32, encoding: *const u8, len: usize) -> i32 {
    code((|| {
        let s = unsafe { session_of(rt, session) }?;
        let chosen = unsafe { text(encoding, len) }?;
        done(s.raw_select(chosen))
    })())
}

/// # Safety
/// `rt` a live runtime; `info` null or valid for its `size` bytes; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_raw_copy(rt: *mut emusen_crystal_runtime, session: u32, info: *mut emusen_crystal_raw_info, out: *mut u8, len: usize) -> i64 {
    let copied = || -> Result<i64, i32> {
        let s = unsafe { session_of(rt, session) }?;
        let info = unsafe { info.as_mut() };
        if let Some(i) = &info
            && (i.size as usize) < std::mem::size_of::<emusen_crystal_raw_info>()
        {
            return Err(fail(abi::status::BAD_STRUCT, format!("emusen_crystal_raw_info of {} bytes; version 1.0's is {}", i.size, std::mem::size_of::<emusen_crystal_raw_info>())));
        }
        let into: &mut [u8] = if out.is_null() { &mut [] } else { unsafe { std::slice::from_raw_parts_mut(out, len.min(i32::MAX as usize)) } };
        let (shape, bytes) = s.raw_copy(into).map_err(|(st, w)| fail(st, w))?;
        if let Some(i) = info {
            (i.element, i.width, i.height, i.stride, i.reserved) = (shape.element, shape.width, shape.height, shape.stride, 0);
        }
        Ok(bytes as i64)
    };
    copied().unwrap_or_else(|status| status as i64)
}

/// # Safety
/// `rt` a live runtime; `base` valid for `base_len` bytes; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_record_start(rt: *mut emusen_crystal_runtime, session: u32, base: *const u8, base_len: usize, stride: u32, out: *mut u8, len: usize) -> i64 {
    let started = || -> Result<String, i32> {
        let s = unsafe { session_of(rt, session) }?;
        let at = unsafe { required(base, base_len, "the folder") }?;
        s.start_recording(at, stride).map_err(|(st, w)| fail(st, w))
    };
    match started() {
        Ok(folder) => unsafe { put(folder.as_bytes(), out, len) },
        Err(status) => status as i64,
    }
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_record_stop(rt: *mut emusen_crystal_runtime, session: u32, out: *mut u8, len: usize) -> i64 {
    let stopped = || -> Result<String, i32> { unsafe { session_of(rt, session) }?.stop_recording().map_err(|(st, w)| fail(st, w)) };
    match stopped() {
        Ok(folder) => unsafe { put(folder.as_bytes(), out, len) },
        Err(status) => status as i64,
    }
}
