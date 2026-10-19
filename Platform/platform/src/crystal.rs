//! The frontend API's exports, `emusen_crystal.h`, over `emusen-crystal`: each converts its arguments, calls the
//! runtime, and copies the answer out by the length query. Exported as part B7 builds them; the binding reaches them
//! only once every export of major 1 is here (EmuSen_Backend.md §29.3).

use std::cell::RefCell;
use emusen_crystal::abi::{self, emusen_crystal_config, emusen_crystal_open_params, emusen_crystal_runtime};
use emusen_crystal::runtime::{Config, Runtime};
use emusen_session_service::abi::status as svc;

thread_local! {
    static LAST_ERROR: RefCell<String> = const { RefCell::new(String::new()) };
}

fn fail(status: i32, words: impl Into<String>) -> i32 {
    LAST_ERROR.with_borrow_mut(|last| *last = words.into());
    status
}

/// The length-query idiom.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
unsafe fn put(bytes: &[u8], out: *mut u8, len: usize) -> i64 {
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
unsafe fn runtime<'a>(rt: *mut emusen_crystal_runtime) -> Result<&'a Runtime, i32> {
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

/// A runtime; null on failure, the words in `last_error`.
///
/// # Safety
/// `config` null or valid for its `size` bytes, its texts valid for their lengths.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_new(config: *const emusen_crystal_config) -> *mut emusen_crystal_runtime {
    let made = || -> Result<Runtime, i32> {
        let mut c = Config::default();
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
            opened: emusen_crystal::session::OpenedWith { flags: p.flags, audio_buffer_max_samples: p.audio_buffer_max_samples },
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
