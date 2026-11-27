//! The frontend API's exports, `emusen_crystal.h`, over `emusen-crystal`: each converts its arguments, calls the
//! runtime, and copies the answer out by the length query. Exported as part B7 builds them; the binding reaches them
//! only once every export of major 1 is here (EmuSen_Backend.md §29.3).

use emusen_crystal::abi::{self, emusen_crystal_config, emusen_crystal_frame_info, emusen_crystal_open_params, emusen_crystal_raw_info, emusen_crystal_runtime, emusen_crystal_subscription};
use emusen_crystal::runtime::{Config, Runtime};
use emusen_session_service::abi::status as svc;

pub(crate) fn fail(status: i32, words: impl Into<String>) -> i32 {
    crate::last_error::fail(crate::last_error::Slot::Crystal, status, words.into())
}

thread_local! {
    static CRYSTAL_WAITING: crate::reply::Waiting = const { std::cell::RefCell::new(None) };
}

fn panic_words(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unexpected panic in crystal".to_string()
    }
}

fn guard_i32(f: impl FnOnce() -> i32) -> i32 {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or_else(|p| fail(svc::FAULT, panic_words(p)))
}

fn guard_i64(f: impl FnOnce() -> i64) -> i64 {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or_else(|p| fail(svc::FAULT, panic_words(p)) as i64)
}

fn guard_ptr<T>(f: impl FnOnce() -> *mut T) -> *mut T {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or_else(|p| {
        fail(svc::FAULT, panic_words(p));
        std::ptr::null_mut()
    })
}

fn guard_const_ptr<T>(f: impl FnOnce() -> *const T) -> *const T {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or_else(|p| {
        fail(svc::FAULT, panic_words(p));
        std::ptr::null()
    })
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
    guard_i64(|| {
        let words = format!("EmuSen's Rust implementation of the frontend API {}.{}, emusen-crystal {BUILD}", abi::MAJOR, abi::MINOR);
        unsafe { put(words.as_bytes(), out, len) }
    })
}

/// # Safety
/// `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_last_error(_rt: *mut emusen_crystal_runtime, out: *mut u8, len: usize) -> i64 {
    guard_i64(|| unsafe { crate::last_error::copy_out(crate::last_error::Slot::Crystal, out, len) })
}

/// The config's directories, a JSON array of paths.
fn cores_of(text: &str) -> Result<Vec<String>, i32> {
    let refused = || fail(svc::BAD_ARGUMENT, "config's cores must be a JSON array of directories");
    match serde_json::from_str::<serde_json::Value>(text) {
        Ok(serde_json::Value::Array(items)) => items.iter().map(|v| v.as_str().map(str::to_string).ok_or_else(refused)).collect(),
        _ => Err(refused()),
    }
}

/// A URL's body through the platform's one HTTP client, within the cheat database's fetch timeout.
fn fetch(url: &str) -> Result<Vec<u8>, String> {
    let request = emusen_net::Request::new(url, emusen_cheats::install::FETCH_TIMEOUT);
    emusen_net::fetch(&request, &emusen_net::Progress::default()).map_err(|f| f.to_string())
}

/// A runtime; null on failure, the words in `last_error`.
///
/// # Safety
/// `config` null or valid for its `size` bytes, its texts valid for their lengths.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_new(config: *const emusen_crystal_config) -> *mut emusen_crystal_runtime {
    guard_ptr(|| {
        let made = || -> Result<Runtime, i32> {
            let mut c = Config { lender: Some(std::sync::Arc::new(SerenityLender)), fetch: Some(std::sync::Arc::new(fetch)), ..Config::default() };
            if !config.is_null() {
                let size = unsafe { (*config).size } as usize;
                if size < 8 {
                    return Err(fail(abi::status::BAD_STRUCT, format!("emusen_crystal_config of {size} bytes; version 1.0's is 8")));
                }
                let flags = unsafe { (*config).flags };
                if flags & !abi::CONFIG_DEVELOPMENT != 0 {
                    return Err(fail(svc::NOT_SUPPORTED, format!("config flags 0x{flags:X} are not known to {}.{}", abi::MAJOR, abi::MINOR)));
                }
                if flags & abi::CONFIG_DEVELOPMENT != 0 {
                    c.development = Some(true);
                }
                if size >= abi::CONFIG_SIZE_12 {
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
                if size >= std::mem::size_of::<emusen_crystal_config>() {
                    let (player, player_len) = unsafe { ((*config).player_cores, (*config).player_cores_len) };
                    if let Some(t) = unsafe { text(player, player_len) }? {
                        c.player_cores = Some(cores_of(t)?);
                    }
                }
            }
            Ok(Runtime::new(&c))
        };
        match made() {
            Ok(rt) => Box::into_raw(Box::new(rt)) as *mut emusen_crystal_runtime,
            Err(_) => std::ptr::null_mut(),
        }
    })
}

/// # Safety
/// `rt` a runtime this library made and has not freed, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_free(rt: *mut emusen_crystal_runtime) -> i32 {
    guard_i32(|| {
        if rt.is_null() {
            return fail(svc::NULL, "a null runtime");
        }
        crate::session_table::forget(rt);
        let owned = unsafe { Box::from_raw(rt as *mut Runtime) };
        let events = owned.events.clone();
        drop(owned);
        HANDLES.lock().unwrap_or_else(|p| p.into_inner()).retain(|_, h| !std::sync::Arc::ptr_eq(&h.events, &events));
        0
    })
}

macro_rules! listing {
    ($name:ident, $method:ident) => {
        /// # Safety
        /// `rt` a live runtime; `out` valid for `len` bytes, or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(rt: *mut emusen_crystal_runtime, out: *mut u8, len: usize) -> i64 {
            guard_i64(|| match unsafe { runtime(rt) } {
                Ok(r) => unsafe { put(r.$method().as_bytes(), out, len) },
                Err(status) => status as i64,
            })
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
    guard_i32(|| match unsafe { runtime(rt) } {
        Ok(r) => {
            r.rescan();
            0
        }
        Err(status) => status,
    })
}

/// `core_approve` and `core_revoke`: the hash as text, then the runtime's change, its refusal as a status with words.
unsafe fn trust_change(rt: *mut emusen_crystal_runtime, sha256: *const u8, sha256_len: usize, change: fn(&Runtime, &str) -> Result<(), (i32, String)>) -> i32 {
    guard_i32(|| {
        let done = || -> Result<(), i32> {
            let r = unsafe { runtime(rt) }?;
            let hash = unsafe { text(sha256, sha256_len) }?.unwrap_or_default();
            change(r, hash).map_err(|(status, words)| fail(status, words))
        };
        match done() {
            Ok(()) => 0,
            Err(status) => status,
        }
    })
}

/// # Safety
/// `rt` a live runtime; `sha256` valid for `sha256_len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_core_approve(rt: *mut emusen_crystal_runtime, sha256: *const u8, sha256_len: usize) -> i32 {
    unsafe { trust_change(rt, sha256, sha256_len, Runtime::approve) }
}

/// # Safety
/// `rt` a live runtime; `sha256` valid for `sha256_len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_core_revoke(rt: *mut emusen_crystal_runtime, sha256: *const u8, sha256_len: usize) -> i32 {
    unsafe { trust_change(rt, sha256, sha256_len, Runtime::revoke) }
}

/// # Safety
/// `rt` a live runtime; `path` valid for `path_len` bytes; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_route(rt: *mut emusen_crystal_runtime, path: *const u8, path_len: usize, out: *mut u8, len: usize) -> i64 {
    guard_i64(|| {
        let answer = || -> Result<String, i32> {
            let r = unsafe { runtime(rt) }?;
            let at = unsafe { text(path, path_len) }?.filter(|p| !p.is_empty()).ok_or_else(|| fail(svc::BAD_ARGUMENT, "the path must be given"))?;
            Ok(r.route_json(at))
        };
        match answer() {
            Ok(json) => unsafe { put(json.as_bytes(), out, len) },
            Err(status) => status as i64,
        }
    })
}

/// # Safety
/// `rt` a live runtime; each text valid for its length; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_firmware_needed(rt: *mut emusen_crystal_runtime, path: *const u8, path_len: usize, engine: *const u8, engine_len: usize, out: *mut u8, len: usize) -> i64 {
    guard_i64(|| {
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
    })
}

/// # Safety
/// `rt` a live runtime; each text valid for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_firmware_install(rt: *mut emusen_crystal_runtime, request: *const u8, request_len: usize, source: *const u8, source_len: usize) -> i32 {
    guard_i32(|| {
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
    })
}

/// # Safety
/// `rt` a live runtime; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_firmware_overview(rt: *mut emusen_crystal_runtime, out: *mut u8, len: usize) -> i64 {
    guard_i64(|| match unsafe { runtime(rt) } {
        Ok(r) => unsafe { put(emusen_crystal::firmware::overview_json(r).as_bytes(), out, len) },
        Err(status) => status as i64,
    })
}

/// # Safety
/// `rt` a live runtime; `system` valid for its length; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_firmware_status(rt: *mut emusen_crystal_runtime, system: *const u8, system_len: usize, out: *mut u8, len: usize) -> i64 {
    guard_i64(|| {
        let answer = || -> Result<String, i32> {
            let r = unsafe { runtime(rt) }?;
            let system = unsafe { text(system, system_len) }?.filter(|s| !s.is_empty()).ok_or_else(|| fail(svc::BAD_ARGUMENT, "the system must be given"))?;
            emusen_crystal::firmware::status_json(r, system).ok_or_else(|| fail(svc::ABSENT, format!("no system is named {system}")))
        };
        match answer() {
            Ok(json) => unsafe { put(json.as_bytes(), out, len) },
            Err(status) => status as i64,
        }
    })
}

/// # Safety
/// `rt` a live runtime; `args` valid for `args_len` bytes; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_cards_op(rt: *mut emusen_crystal_runtime, op: u32, args: *const u8, args_len: usize, out: *mut u8, len: usize) -> i64 {
    guard_i64(|| {
        let r = match unsafe { runtime(rt) } {
            Ok(r) => r,
            Err(status) => return status as i64,
        };
        if args.is_null() && args_len > 0 {
            return fail(svc::NULL, "a null buffer with a length") as i64;
        }
        let bytes = if args.is_null() { &[][..] } else { unsafe { std::slice::from_raw_parts(args, args_len) } };
        let args = match std::str::from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => return fail(svc::BAD_ARGUMENT, "the argument is not JSON: it is not UTF-8") as i64,
        };
        let run = || {
            let discovery = r.discovery();
            let catalog = emusen_crystal::catalog::Catalog::new(&discovery);
            let tree = emusen_galaxia::tree::tree();
            emusen_crystal::media::cards_op(&catalog, &tree, op, args)
        };
        if matches!(op, emusen_crystal::abi::CARDS_LIST | emusen_crystal::abi::CARDS_OPEN) {
            return copy_of(run(), out, len);
        }
        let key = format!("{}:{op}:{args}", rt as usize);
        unsafe { crate::reply::answer(&CRYSTAL_WAITING, 3, &key, out, len, || run().map_err(|(status, words)| fail(status, words)), |status| status) }
    })
}

/// A codec's id as text, which must be given.
unsafe fn codec_named<'a>(codec: *const u8, codec_len: usize) -> Result<&'a str, i32> {
    unsafe { text(codec, codec_len) }?.filter(|c| !c.is_empty()).ok_or_else(|| fail(svc::BAD_ARGUMENT, "the codec must be given"))
}

/// # Safety
/// `rt` a live runtime; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_cheat_codec_list(rt: *mut emusen_crystal_runtime, out: *mut u8, len: usize) -> i64 {
    guard_i64(|| match unsafe { runtime(rt) } {
        Ok(_) => unsafe { put(emusen_systems::cheats::codecs_json().as_bytes(), out, len) },
        Err(status) => status as i64,
    })
}

/// # Safety
/// `rt` a live runtime; each text valid for its length; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_cheat_decode(rt: *mut emusen_crystal_runtime, codec: *const u8, codec_len: usize, code: *const u8, code_len: usize, out: *mut u8, len: usize) -> i64 {
    guard_i64(|| {
        let answer = || -> Result<String, i32> {
            unsafe { runtime(rt) }?;
            let codec = unsafe { codec_named(codec, codec_len) }?;
            let code = unsafe { text(code, code_len) }?.unwrap_or("");
            emusen_systems::cheats::decode(codec, code).map(|a| emusen_systems::cheats::answer_json(&a)).ok_or_else(|| fail(svc::ABSENT, format!("no codec is named {codec}")))
        };
        match answer() {
            Ok(json) => unsafe { put(json.as_bytes(), out, len) },
            Err(status) => status as i64,
        }
    })
}

/// # Safety
/// `rt` a live runtime; `codec` valid for `codec_len` bytes; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_cheat_encode(rt: *mut emusen_crystal_runtime, codec: *const u8, codec_len: usize, address: i32, value: u32, out: *mut u8, len: usize) -> i64 {
    guard_i64(|| {
        let answer = || -> Result<String, i32> {
            unsafe { runtime(rt) }?;
            let codec = unsafe { codec_named(codec, codec_len) }?;
            emusen_systems::cheats::encode(codec, address, value).ok_or_else(|| fail(svc::ABSENT, format!("the codec {codec} has no encoder")))
        };
        match answer() {
            Ok(code) => unsafe { put(code.as_bytes(), out, len) },
            Err(status) => status as i64,
        }
    })
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
    guard_i64(|| {
        let opened = || -> Result<u32, i32> {
            let r = unsafe { runtime(rt) }?;
            if params.is_null() {
                return Err(fail(svc::NULL, "null open parameters"));
            }
            let p = unsafe { &*params };
            if p.size < 64 {
                return Err(fail(abi::status::BAD_STRUCT, format!("emusen_crystal_open_params of {} bytes; version 1.0's is 64", p.size)));
            }
            let known = abi::OPEN_NO_BATTERY | abi::OPEN_SINGLE_ROWS | abi::OPEN_RING;
            if p.flags & !known != 0 {
                return Err(fail(svc::NOT_SUPPORTED, format!("open flags 0x{:X} are not known to {}.{}", p.flags & !known, abi::MAJOR, abi::MINOR)));
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
    })
}

macro_rules! session_call {
    ($name:ident, $method:ident) => {
        /// # Safety
        /// `rt` a live runtime.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(rt: *mut emusen_crystal_runtime, session: u32) -> i32 {
            guard_i32(|| match unsafe { runtime(rt) } {
                Ok(r) => status_of(r.$method(session)),
                Err(status) => status,
            })
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
            guard_i64(|| match unsafe { runtime(rt) } {
                Ok(r) => copy_of(r.$method(session), out, len),
                Err(status) => status as i64,
            })
        }
    };
}

session_read!(emusen_crystal_session_info, session_info);
session_read!(emusen_crystal_save_media, save_media);

/// # Safety
/// `rt` a live runtime; `path` valid for `path_len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_disc_insert(rt: *mut emusen_crystal_runtime, session: u32, path: *const u8, path_len: usize) -> i32 {
    guard_i32(|| {
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
    })
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_disc_change(rt: *mut emusen_crystal_runtime, session: u32, index: u32) -> i32 {
    guard_i32(|| match unsafe { runtime(rt) } {
        Ok(r) => status_of(r.disc_change(session, index)),
        Err(status) => status,
    })
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_disc_lid(rt: *mut emusen_crystal_runtime, session: u32, open: u32) -> i32 {
    guard_i32(|| match unsafe { runtime(rt) } {
        Ok(r) => status_of(r.disc_lid(session, open != 0)),
        Err(status) => status,
    })
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
    guard_i32(|| {
        code((|| {
            let s = unsafe { session_of(rt, session) }?;
            done(s.frame(unsafe { out.as_mut() }))
        })())
    })
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_picture_copy(rt: *mut emusen_crystal_runtime, session: u32, out: *mut u8, len: usize) -> i64 {
    guard_i64(|| {
        let copied = || -> Result<i64, i32> {
            let s = unsafe { session_of(rt, session) }?;
            let into: &mut [u8] = if out.is_null() { &mut [] } else { unsafe { std::slice::from_raw_parts_mut(out, len.min(i32::MAX as usize)) } };
            s.copy_picture(into).map(|n| n as i64).map_err(|(st, w)| fail(st, w))
        };
        copied().unwrap_or_else(|status| status as i64)
    })
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `frames` stereo frames; `rate` null or valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_audio_drain(rt: *mut emusen_crystal_runtime, session: u32, out: *mut i16, frames: usize, rate: *mut u32) -> i64 {
    guard_i64(|| {
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
    })
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_wait(rt: *mut emusen_crystal_runtime, session: u32, timeout_ms: u32) -> i32 {
    guard_i32(|| match unsafe { session_of(rt, session) } {
        Ok(s) => s.wait(timeout_ms) as i32,
        Err(status) => status,
    })
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_phase_names(rt: *mut emusen_crystal_runtime, session: u32, out: *mut u8, len: usize) -> i64 {
    guard_i64(|| match unsafe { session_of(rt, session) } {
        Ok(s) => copy_of(s.phase_names_json(), out, len),
        Err(status) => status as i64,
    })
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `count` values.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_phases(rt: *mut emusen_crystal_runtime, session: u32, out: *mut f64, count: usize) -> i64 {
    guard_i64(|| {
        let timed = || -> Result<i64, i32> {
            let s = unsafe { session_of(rt, session) }?;
            let into: &mut [f64] = if out.is_null() { &mut [] } else { unsafe { std::slice::from_raw_parts_mut(out, count) } };
            s.phases(into).map(|n| n as i64).map_err(|(st, w)| fail(st, w))
        };
        timed().unwrap_or_else(|status| status as i64)
    })
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_input_button(rt: *mut emusen_crystal_runtime, session: u32, port: u32, control: u32, pressed: u32) -> i32 {
    guard_i32(|| code((|| done(unsafe { session_of(rt, session) }?.button(port, control, pressed != 0)))()))
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_input_axis(rt: *mut emusen_crystal_runtime, session: u32, port: u32, axis: u32, value: f64) -> i32 {
    guard_i32(|| code((|| done(unsafe { session_of(rt, session) }?.axis(port, axis, value)))()))
}

/// # Safety
/// `rt` a live runtime; `strong` and `weak` null or valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_input_rumble(rt: *mut emusen_crystal_runtime, session: u32, port: u32, strong: *mut u16, weak: *mut u16) -> i32 {
    guard_i32(|| {
        code((|| {
            let (s, w) = unsafe { session_of(rt, session) }?.rumble.level(port);
            unsafe {
                if let Some(out) = strong.as_mut() {
                    *out = s;
                }
                if let Some(out) = weak.as_mut() {
                    *out = w;
                }
            }
            Ok(())
        })())
    })
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_input_rumble_switch(rt: *mut emusen_crystal_runtime, session: u32, on: u32) -> i32 {
    guard_i32(|| {
        code((|| {
            unsafe { session_of(rt, session) }?.rumble.set_switch(on != 0);
            Ok(())
        })())
    })
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_input_connected(rt: *mut emusen_crystal_runtime, session: u32, port: u32, connected: u32) -> i32 {
    guard_i32(|| {
        code((|| {
            unsafe { session_of(rt, session) }?.connected(port, connected != 0);
            Ok(())
        })())
    })
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_speed_set(rt: *mut emusen_crystal_runtime, session: u32, percent: u32, turbo_percent: u32, slow_percent: u32, flags: u32) -> i32 {
    guard_i32(|| code((|| done(unsafe { session_of(rt, session) }?.speed_set(percent, turbo_percent, slow_percent, flags)))()))
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_speed_mode(rt: *mut emusen_crystal_runtime, session: u32, mode: u32) -> i32 {
    guard_i32(|| code((|| done(unsafe { session_of(rt, session) }?.speed_mode(mode)))()))
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_rewind_set(rt: *mut emusen_crystal_runtime, session: u32, enabled: u32, interval_frames: u32, budget_bytes: u64, thumbnail_width: u32) -> i32 {
    guard_i32(|| code((|| done(unsafe { session_of(rt, session) }?.rewind_set(enabled != 0, interval_frames, budget_bytes, thumbnail_width)))()))
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for its `size` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_rewind_step(rt: *mut emusen_crystal_runtime, session: u32, count: u32, out: *mut emusen_crystal_frame_info) -> i64 {
    guard_i64(|| {
        let stepped = || -> Result<i64, i32> {
            let r = unsafe { runtime(rt) }?;
            let s = r.sessions.get(session).map_err(|(st, w)| fail(st, w))?;
            r.with_ctx(|ctx| s.rewind_step(count, unsafe { out.as_mut() }, ctx)).map_err(|(st, w)| fail(st, w))
        };
        stepped().unwrap_or_else(|status| status as i64)
    })
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_rewind_reel(rt: *mut emusen_crystal_runtime, session: u32, out: *mut u8, len: usize) -> i64 {
    guard_i64(|| match unsafe { session_of(rt, session) } {
        Ok(s) => copy_of(s.rewind_reel_json(), out, len),
        Err(status) => status as i64,
    })
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_rewind_picture(rt: *mut emusen_crystal_runtime, session: u32, frame: u64, out: *mut u8, len: usize) -> i64 {
    guard_i64(|| match unsafe { session_of(rt, session) }.and_then(|s| s.rewind_picture(frame).map_err(|(st, w)| fail(st, w))) {
        Ok(bytes) => unsafe { put(&bytes, out, len) },
        Err(status) => status as i64,
    })
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_rewind_to(rt: *mut emusen_crystal_runtime, session: u32, frame: u64) -> i32 {
    guard_i32(|| {
        code((|| {
            let r = unsafe { runtime(rt) }?;
            let s = r.sessions.get(session).map_err(|(st, w)| fail(st, w))?;
            done(r.with_ctx(|ctx| s.rewind_to(frame, ctx)))
        })())
    })
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_state_slots(rt: *mut emusen_crystal_runtime, session: u32, out: *mut u8, len: usize) -> i64 {
    guard_i64(|| match unsafe { session_of(rt, session) } {
        Ok(s) => {
            let json = s.slots_json(&emusen_galaxia::tree::tree());
            unsafe { put(json.as_bytes(), out, len) }
        }
        Err(status) => status as i64,
    })
}

macro_rules! state_call {
    ($name:ident, $call:expr) => {
        /// # Safety
        /// `rt` a live runtime.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $name(rt: *mut emusen_crystal_runtime, session: u32) -> i32 {
            guard_i32(|| {
                code((|| {
                    let r = unsafe { runtime(rt) }?;
                    let s = r.sessions.get(session).map_err(|(st, w)| fail(st, w))?;
                    done(r.with_records(|records| r.with_ctx(|ctx| $call(&s, records, ctx))))
                })())
            })
        }
    };
}

state_call!(emusen_crystal_resume_save, |s: &emusen_crystal::session::Session, rec, ctx| s.save_resume(rec, ctx));
state_call!(emusen_crystal_resume_load, |s: &emusen_crystal::session::Session, rec, ctx| s.load_resume(rec, ctx));

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_state_slot_save(rt: *mut emusen_crystal_runtime, session: u32, slot: u32) -> i32 {
    guard_i32(|| {
        code((|| {
            let r = unsafe { runtime(rt) }?;
            let s = r.sessions.get(session).map_err(|(st, w)| fail(st, w))?;
            done(r.with_records(|records| r.with_ctx(|ctx| s.save_slot(slot, records, ctx))))
        })())
    })
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_state_slot_load(rt: *mut emusen_crystal_runtime, session: u32, slot: u32) -> i32 {
    guard_i32(|| {
        code((|| {
            let r = unsafe { runtime(rt) }?;
            let s = r.sessions.get(session).map_err(|(st, w)| fail(st, w))?;
            done(r.with_records(|records| r.with_ctx(|ctx| s.load_slot(slot, records, ctx))))
        })())
    })
}

/// # Safety
/// `rt` a live runtime; `path` valid for `path_len` bytes; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_state_record(rt: *mut emusen_crystal_runtime, path: *const u8, path_len: usize, out: *mut u8, len: usize) -> i64 {
    guard_i64(|| {
        let read = || -> Result<String, i32> {
            let r = unsafe { runtime(rt) }?;
            let at = unsafe { text(path, path_len) }?.filter(|p| !p.is_empty()).ok_or_else(|| fail(svc::BAD_ARGUMENT, "the path must be given"))?;
            r.state_record_json(at).map_err(|(st, w)| fail(st, w))
        };
        match read() {
            Ok(json) => unsafe { put(json.as_bytes(), out, len) },
            Err(status) => status as i64,
        }
    })
}

/// # Safety
/// `rt` a live runtime; `path` valid for `path_len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_state_forget(rt: *mut emusen_crystal_runtime, path: *const u8, path_len: usize) -> i32 {
    guard_i32(|| {
        code((|| {
            let r = unsafe { runtime(rt) }?;
            let at = unsafe { text(path, path_len) }?.filter(|p| !p.is_empty()).ok_or_else(|| fail(svc::BAD_ARGUMENT, "the path must be given"))?;
            done(r.state_forget(at))
        })())
    })
}

/// A subscription's handle: its subscription and the events it was taken from.
struct Held {
    sub: std::sync::Arc<emusen_crystal::events::Subscription>,
    events: std::sync::Arc<emusen_crystal::events::Events>,
}

/// The live subscriptions keyed by opaque non-zero tokens so callers hold strong Arcs across waits.
static HANDLES: std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<usize, std::sync::Arc<Held>>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));
static NEXT_HANDLE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(8);

/// Looks up a subscription handle, cloning its strong Arc under the table lock.
fn subscription(sub: *mut emusen_crystal_subscription) -> Result<std::sync::Arc<Held>, i32> {
    if sub.is_null() {
        return Err(fail(svc::NULL, "a null subscription"));
    }
    let held = HANDLES.lock().unwrap_or_else(|p| p.into_inner()).get(&(sub as usize)).cloned();
    match held {
        Some(h) if h.sub.live.load(std::sync::atomic::Ordering::Acquire) => Ok(h),
        _ => Err(fail(svc::NULL, "not a subscription")),
    }
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_subscribe(rt: *mut emusen_crystal_runtime, session: u32, kinds: u32) -> *mut emusen_crystal_subscription {
    guard_ptr(|| {
        let made = || -> Result<*mut emusen_crystal_subscription, i32> {
            let r = unsafe { runtime(rt) }?;
            if session != 0 {
                r.sessions.get(session).map_err(|(st, w)| fail(st, w))?;
            }
            let held = std::sync::Arc::new(Held { sub: r.events.subscribe(session, kinds), events: r.events.clone() });
            let mut map = HANDLES.lock().unwrap_or_else(|p| p.into_inner());
            let key = loop {
                let candidate = NEXT_HANDLE.fetch_add(8, std::sync::atomic::Ordering::Relaxed);
                if candidate != 0 && !map.contains_key(&candidate) {
                    break candidate;
                }
            };
            map.insert(key, held);
            Ok(std::ptr::without_provenance_mut(key))
        };
        made().unwrap_or(std::ptr::null_mut())
    })
}

/// # Safety
/// `sub` a subscription's handle; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_subscription_drain(sub: *mut emusen_crystal_subscription, out: *mut u8, len: usize) -> i64 {
    guard_i64(|| match subscription(sub) {
        Ok(h) => {
            let into = if out.is_null() { None } else { Some(unsafe { std::slice::from_raw_parts_mut(out, len) }) };
            h.sub.drain(into) as i64
        }
        Err(status) => status as i64,
    })
}

/// # Safety
/// `sub` a subscription's handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_subscription_wait(sub: *mut emusen_crystal_subscription, timeout_ms: u32) -> i32 {
    guard_i32(|| match subscription(sub) {
        Ok(h) => h.sub.wait(timeout_ms) as i32,
        Err(status) => status,
    })
}

/// # Safety
/// `sub` a subscription's handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_unsubscribe(sub: *mut emusen_crystal_subscription) -> i32 {
    guard_i32(|| {
        if sub.is_null() {
            return fail(svc::NULL, "a null subscription");
        }
        let removed = HANDLES.lock().unwrap_or_else(|p| p.into_inner()).remove(&(sub as usize));
        match removed {
            Some(h) if h.sub.live.load(std::sync::atomic::Ordering::Acquire) => {
                h.events.remove(&h.sub);
                0
            }
            _ => fail(svc::NULL, "not a subscription"),
        }
    })
}

/// A required text, refused as the C#'s `Required` refuses it.
unsafe fn required<'a>(data: *const u8, len: usize, what: &str) -> Result<&'a str, i32> {
    unsafe { text(data, len) }?.filter(|t| !t.is_empty()).ok_or_else(|| fail(svc::BAD_ARGUMENT, format!("{what} must be given")))
}

/// # Safety
/// `rt` a live runtime; each text valid for its length; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_settings_schema(rt: *mut emusen_crystal_runtime, session: u32, console: *const u8, console_len: usize, engine: *const u8, engine_len: usize, out: *mut u8, len: usize) -> i64 {
    guard_i64(|| {
        let read = || -> Result<String, i32> {
            let r = unsafe { runtime(rt) }?;
            let (c, e) = (unsafe { text(console, console_len) }?, unsafe { text(engine, engine_len) }?);
            r.settings_schema_json(session, c, e).map_err(|(st, w)| fail(st, w))
        };
        match read() {
            Ok(json) => unsafe { put(json.as_bytes(), out, len) },
            Err(status) => status as i64,
        }
    })
}

/// # Safety
/// `rt` a live runtime; each text valid for its length; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_settings_get(rt: *mut emusen_crystal_runtime, session: u32, scope: u32, console: *const u8, console_len: usize, key: *const u8, key_len: usize, out: *mut u8, len: usize) -> i64 {
    guard_i64(|| {
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
    })
}

/// # Safety
/// `rt` a live runtime; each text valid for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_settings_set(rt: *mut emusen_crystal_runtime, session: u32, scope: u32, console: *const u8, console_len: usize, key: *const u8, key_len: usize, value: *const u8, value_len: usize) -> i32 {
    guard_i32(|| {
        code((|| {
            let r = unsafe { runtime(rt) }?;
            let (c, v) = (unsafe { text(console, console_len) }?, unsafe { text(value, value_len) }?);
            let k = unsafe { required(key, key_len, "the key") }?;
            done(r.settings_set(session, scope, c, k, v))
        })())
    })
}

/// # Safety
/// `rt` a live runtime; each text valid for its length; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_game_setting_get(rt: *mut emusen_crystal_runtime, path: *const u8, path_len: usize, key: *const u8, key_len: usize, out: *mut u8, len: usize) -> i64 {
    guard_i64(|| {
        let read = || -> Result<String, i32> {
            let r = unsafe { runtime(rt) }?;
            let (p, k) = (unsafe { required(path, path_len, "the path") }?, unsafe { required(key, key_len, "the key") }?);
            r.game_setting_get(p, k).map_err(|(st, w)| fail(st, w))
        };
        match read() {
            Ok(value) => unsafe { put(value.as_bytes(), out, len) },
            Err(status) => status as i64,
        }
    })
}

/// # Safety
/// `rt` a live runtime; each text valid for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_game_setting_set(rt: *mut emusen_crystal_runtime, path: *const u8, path_len: usize, key: *const u8, key_len: usize, value: *const u8, value_len: usize) -> i32 {
    guard_i32(|| {
        code((|| {
            let r = unsafe { runtime(rt) }?;
            let (p, k) = (unsafe { required(path, path_len, "the path") }?, unsafe { required(key, key_len, "the key") }?);
            let v = unsafe { text(value, value_len) }?;
            done(r.game_setting_set(p, k, v))
        })())
    })
}

/// The runtime's lender over Serenity's lend of the presenter's device, `PlatformLend`.
struct SerenityLender;

impl emusen_crystal::lend::Lender for SerenityLender {
    fn make(&self, device: usize) -> Result<(usize, emusen_native::core::sys::DeviceLend), String> {
        let dev = unsafe { (device as *const crate::serenity_device::DeviceHandle).as_ref() };
        let handle = crate::serenity_device::lend_new(dev).map_err(|(_, w)| w)?;
        let l = handle.lend.lent();
        let lend = emusen_native::core::sys::DeviceLend {
            size: std::mem::size_of::<emusen_native::core::sys::DeviceLend>() as u32,
            api_version: l.api,
            get_instance_proc_addr: Some(handle.lend.proc_addr()),
            instance: l.instance,
            physical_device: l.physical,
            device: l.device,
            queue_family: l.family,
            queue_index: l.queue,
            host_queue_family: l.host_family,
            reserved: 0,
            done: l.done,
        };
        Ok((Box::into_raw(handle) as usize, lend))
    }

    fn take(&self, lend: usize, f: &emusen_native::core::sys::DeviceFrame) -> Result<bool, String> {
        let handle = unsafe { (lend as *const crate::serenity_device::LendHandle).as_ref() };
        crate::serenity_device::lend_take(handle, f.buffer, f.offset, f.width, f.height, f.stride, f.ready, f.value).map_err(|(_, w)| w)
    }

    fn settle(&self, lend: usize) -> Result<(), String> {
        let handle = unsafe { (lend as *const crate::serenity_device::LendHandle).as_ref() };
        crate::serenity_device::lend_settle(handle).map_err(|(_, w)| w)
    }

    fn free(&self, lend: usize) {
        let handle = lend as *mut crate::serenity_device::LendHandle;
        if !handle.is_null() {
            drop(unsafe { Box::from_raw(handle) });
        }
    }

    fn make_ring(&self, device: usize, width: u32, height: u32, count: u32) -> Result<emusen_crystal::lend::MachineRing, String> {
        let dev = unsafe { (device as *const crate::serenity_device::DeviceHandle).as_ref() };
        let handle = crate::serenity_device::ring_new(dev, width, height, count).map_err(|(_, w)| w)?;
        let images = handle
            .ring
            .images
            .iter()
            .map(|i| emusen_native::core::sys::RingImage {
                size: std::mem::size_of::<emusen_native::core::sys::RingImage>() as u32,
                memory_type: i.memory_type,
                handle: i.handle as u64,
                allocation_size: i.allocation_size,
                width: i.width,
                height: i.height,
                format: i.format,
                usage: i.usage,
            })
            .collect();
        let (ready, done, device_uuid, driver_uuid) = (handle.ring.ready_fd as u64, handle.ring.done_fd as u64, handle.ring.device_uuid, handle.ring.driver_uuid);
        Ok(emusen_crystal::lend::MachineRing { lender: None, handle: Box::into_raw(handle) as usize, images, ready, done, device_uuid, driver_uuid, width, height, last_value: 0 })
    }

    fn free_ring(&self, ring: usize, value: u64) {
        let handle = ring as *mut crate::serenity_device::RingHandle;
        if !handle.is_null() {
            let owned = unsafe { Box::from_raw(handle) };
            if let Err(words) = owned.ring.release_after_crash(value) {
                emusen_crystal::error_log::warning("picture", &format!("a ring's ready could not be raised before it was freed: {words}"));
            }
        }
    }
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_picture_lend(rt: *mut emusen_crystal_runtime, session: u32) -> *const () {
    guard_const_ptr(|| {
        let lent = || -> Result<usize, i32> { unsafe { session_of(rt, session) }?.picture_lend().map_err(|(st, w)| fail(st, w)) };
        lent().unwrap_or(0) as *const ()
    })
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_picture_ring(rt: *mut emusen_crystal_runtime, session: u32) -> *const () {
    guard_const_ptr(|| {
        let lent = || -> Result<usize, i32> { unsafe { session_of(rt, session) }?.picture_ring().map_err(|(st, w)| fail(st, w)) };
        lent().unwrap_or(0) as *const ()
    })
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_frame_advance(rt: *mut emusen_crystal_runtime, session: u32) -> i32 {
    guard_i32(|| code((|| done(unsafe { session_of(rt, session) }?.frame_advance()))()))
}

/// # Safety
/// `rt` a live runtime; `encoding` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_raw_select(rt: *mut emusen_crystal_runtime, session: u32, encoding: *const u8, len: usize) -> i32 {
    guard_i32(|| {
        code((|| {
            let s = unsafe { session_of(rt, session) }?;
            let chosen = unsafe { text(encoding, len) }?;
            done(s.raw_select(chosen))
        })())
    })
}

/// # Safety
/// `rt` a live runtime; `info` null or valid for its `size` bytes; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_raw_copy(rt: *mut emusen_crystal_runtime, session: u32, info: *mut emusen_crystal_raw_info, out: *mut u8, len: usize) -> i64 {
    guard_i64(|| {
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
    })
}

/// # Safety
/// `rt` a live runtime; `base` valid for `base_len` bytes; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_record_start(rt: *mut emusen_crystal_runtime, session: u32, base: *const u8, base_len: usize, stride: u32, out: *mut u8, len: usize) -> i64 {
    guard_i64(|| {
        let s = match unsafe { session_of(rt, session) } {
            Ok(s) => s,
            Err(status) => return status as i64,
        };
        let at = match unsafe { required(base, base_len, "the folder") } {
            Ok(at) => at,
            Err(status) => return status as i64,
        };
        let key = format!("{}:{}:{stride}:{at}", rt as usize, s.id);
        unsafe { crate::reply::answer(&CRYSTAL_WAITING, 1, &key, out, len, || s.start_recording(at, stride).map_err(|(st, w)| fail(st, w)), |st| st) }
    })
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_record_stop(rt: *mut emusen_crystal_runtime, session: u32, out: *mut u8, len: usize) -> i64 {
    guard_i64(|| {
        let s = match unsafe { session_of(rt, session) } {
            Ok(s) => s,
            Err(status) => return status as i64,
        };
        let key = format!("{}:{}", rt as usize, s.id);
        unsafe { crate::reply::answer(&CRYSTAL_WAITING, 2, &key, out, len, || s.stop_recording().map_err(|(st, w)| fail(st, w)), |st| st) }
    })
}

#[cfg(test)]
pub(crate) fn empty_runtime_for_tests() -> *mut emusen_crystal_runtime {
    let none = "[]";
    let config = emusen_crystal_config {
        size: std::mem::size_of::<emusen_crystal_config>() as u32,
        flags: 0,
        sqlite: std::ptr::null_mut(),
        cores: none.as_ptr(),
        cores_len: none.len(),
        player_cores: none.as_ptr(),
        player_cores_len: none.len(),
    };
    let rt = unsafe { emusen_crystal_new(&config) };
    assert!(!rt.is_null());
    rt
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::time::{Duration, Instant};

    #[test]
    fn subscription_wait_wakes_cleanly_when_unsubscribed_concurrently() {
        let rt = empty_runtime_for_tests();
        let sub = unsafe { emusen_crystal_subscribe(rt, 0, 0) };
        assert!(!sub.is_null());
        let addr = sub as usize;
        let ready = Arc::new(Barrier::new(2));
        let ready_wait = ready.clone();
        let waiter = std::thread::spawn(move || {
            ready_wait.wait();
            let started = Instant::now();
            let code = unsafe { emusen_crystal_subscription_wait(std::ptr::without_provenance_mut(addr), 10_000) };
            (code, started.elapsed())
        });
        ready.wait();
        std::thread::sleep(Duration::from_millis(25));
        assert_eq!(unsafe { emusen_crystal_unsubscribe(sub) }, 0);
        let (code, elapsed) = waiter.join().unwrap();
        assert_eq!(code, 0, "unsubscribed waiter returns 0 rather than hanging or faulting");
        assert!(elapsed < Duration::from_secs(2), "waiter woke promptly: {elapsed:?}");
        assert_eq!(unsafe { emusen_crystal_subscription_wait(sub, 10) }, svc::NULL);

        for _ in 0..64 {
            let sub = unsafe { emusen_crystal_subscribe(rt, 0, 0) };
            assert!(!sub.is_null());
            let addr = sub as usize;
            let gate = Arc::new(Barrier::new(2));
            let gate_w = gate.clone();
            let t = std::thread::spawn(move || {
                gate_w.wait();
                unsafe { emusen_crystal_subscription_wait(std::ptr::without_provenance_mut(addr), 2_000) }
            });
            gate.wait();
            assert_eq!(unsafe { emusen_crystal_unsubscribe(sub) }, 0);
            let res = t.join().unwrap();
            assert!(res == 0 || res == svc::NULL, "unexpected wait status: {res}");
        }
        assert_eq!(unsafe { emusen_crystal_free(rt) }, 0);
    }

    #[test]
    fn concurrent_double_unsubscribe_succeeds_exactly_once() {
        let rt = empty_runtime_for_tests();
        for _ in 0..100 {
            let sub = unsafe { emusen_crystal_subscribe(rt, 0, 0) };
            assert!(!sub.is_null());
            let addr = sub as usize;
            let gate = Arc::new(Barrier::new(2));
            let g1 = gate.clone();
            let t1 = std::thread::spawn(move || {
                g1.wait();
                unsafe { emusen_crystal_unsubscribe(std::ptr::without_provenance_mut(addr)) }
            });
            let t2 = std::thread::spawn(move || {
                gate.wait();
                unsafe { emusen_crystal_unsubscribe(std::ptr::without_provenance_mut(addr)) }
            });
            let mut results = [t1.join().unwrap(), t2.join().unwrap()];
            results.sort();
            assert_eq!(results, [svc::NULL, 0], "exactly one caller must win unsubscribe");
        }
        assert_eq!(unsafe { emusen_crystal_free(rt) }, 0);
    }

    #[test]
    fn runtime_free_evicts_unclosed_subscription_handles() {
        let rt = empty_runtime_for_tests();
        let sub = unsafe { emusen_crystal_subscribe(rt, 0, 0) };
        assert!(!sub.is_null());
        let key = sub as usize;
        assert!(HANDLES.lock().unwrap_or_else(|p| p.into_inner()).contains_key(&key));
        assert_eq!(unsafe { emusen_crystal_free(rt) }, 0);
        assert!(!HANDLES.lock().unwrap_or_else(|p| p.into_inner()).contains_key(&key));
        assert_eq!(unsafe { emusen_crystal_unsubscribe(sub) }, svc::NULL);
    }

    #[test]
    fn cards_op_two_pass_copy_out_executes_once_via_waiting_cache() {
        let dir = std::env::temp_dir().join(format!("platform-cards-2pass-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        emusen_galaxia::tree::set_thread_scope(Some(emusen_galaxia::tree::ThreadScope {
            config: Some(dir.join("etc")),
            data: Some(dir.join("data")),
            legacy: Some(dir.join("legacy")),
        }));
        let rt = empty_runtime_for_tests();
        let card = dir.join("data/Saves/PlayStation/Cards/Test.mcr");
        let args = serde_json::json!({ "path": card }).to_string();
        let needed = unsafe { emusen_crystal_cards_op(rt, emusen_crystal::abi::CARDS_NEW, args.as_ptr(), args.len(), std::ptr::null_mut(), 0) };
        assert!(needed > 0, "expected length, got {needed}");
        let mut buf = vec![0u8; needed as usize];
        let copied = unsafe { emusen_crystal_cards_op(rt, emusen_crystal::abi::CARDS_NEW, args.as_ptr(), args.len(), buf.as_mut_ptr(), buf.len()) };
        assert_eq!(copied, needed);
        let reply: serde_json::Value = serde_json::from_slice(&buf).unwrap();
        assert!(reply.get("done").is_some(), "valid json reply: {reply}");

        let runs = std::sync::atomic::AtomicUsize::new(0);
        for call_kind in [1u8, 2u8, 3u8] {
            runs.store(0, std::sync::atomic::Ordering::SeqCst);
            let make = || -> Result<String, i32> {
                runs.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok("recorded".to_string())
            };
            let n = unsafe { crate::reply::answer(&CRYSTAL_WAITING, call_kind, "k", std::ptr::null_mut(), 0, make, |s| s) };
            let mut out = vec![0u8; n as usize];
            let m = unsafe { crate::reply::answer(&CRYSTAL_WAITING, call_kind, "k", out.as_mut_ptr(), out.len(), || { runs.fetch_add(1, std::sync::atomic::Ordering::SeqCst); Ok("second".to_string()) }, |s| s) };
            assert_eq!((n, m, &out[..]), (8, 8, &b"recorded"[..]));
            assert_eq!(runs.load(std::sync::atomic::Ordering::SeqCst), 1);
        }

        assert_eq!(unsafe { emusen_crystal_free(rt) }, 0);
        emusen_galaxia::tree::set_thread_scope(None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn panic_in_crystal_guard_records_last_error_without_unwinding() {
        assert_eq!(guard_i32(|| panic!("i32 fault")), svc::FAULT);
        let mut buf = [0u8; 64];
        let n = unsafe { emusen_crystal_last_error(std::ptr::null_mut(), buf.as_mut_ptr(), buf.len()) };
        assert!(n > 0);
        assert!(std::str::from_utf8(&buf[..n as usize]).unwrap().contains("i32 fault"));
        assert_eq!(guard_i64(|| panic!("i64 fault")), svc::FAULT as i64);
        assert!(guard_ptr::<u8>(|| panic!("ptr fault")).is_null());
        assert!(guard_const_ptr::<u8>(|| panic!("cptr fault")).is_null());
    }
}
