//! The session service's twins, `emusen_crystal.h`'s functions after `session_service`, and the session's
//! `emusen_session_v1` table built from them, over `emusen-crystal`'s `service`: a twin names its session through the
//! runtime, a table entry through the table it was handed, and both make one call (EmuSen_Backend.md §29.3, §29.21).

use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::Mutex;

use emusen_crystal::abi::{self, emusen_crystal_runtime};
use emusen_crystal::runtime::Runtime;
use emusen_crystal::service::{Answer, Call, Raw, machine};
use emusen_session_service::abi::{emusen_session_v1, status as svc};

use crate::crystal::{fail, put, runtime};

thread_local! {
    /// The table's own last words, `SessionExports`' `t_lastError`.
    static SERVICE_ERROR: RefCell<String> = const { RefCell::new(String::new()) };
}

fn service_fail(status: i32, words: &str) -> i32 {
    SERVICE_ERROR.with_borrow_mut(|last| *last = words.to_string());
    status
}

/// What a table's ctx points at: the runtime, the session it serves, and whether it may change it.
struct Binding {
    rt: *const Runtime,
    id: u32,
    read_only: bool,
}

/// One table and the binding its ctx points at, kept together at a fixed address.
struct Slot {
    table: emusen_session_v1,
    binding: Binding,
}

/// The tables made for one session: the full one and the read-only one.
struct Tables {
    rt: usize,
    id: u32,
    full: Box<Slot>,
    read_only: Box<Slot>,
}

/// SAFETY: a table's pointers are the runtime's and the binding's own; a caller reaches them only through the table.
unsafe impl Send for Tables {}

/// Every table handed out, freed with its runtime.
static TABLES: Mutex<Vec<Tables>> = Mutex::new(Vec::new());

/// The runtime's tables let go as it is freed.
pub(crate) fn forget(rt: *mut emusen_crystal_runtime) {
    TABLES.lock().unwrap_or_else(|p| p.into_inner()).retain(|t| t.rt != rt as usize);
}

fn slot(rt: *const Runtime, id: u32, read_only: bool) -> Box<Slot> {
    let mut s = Box::new(Slot { table: template(), binding: Binding { rt, id, read_only } });
    s.table.ctx = &mut s.binding as *mut Binding as *mut c_void;
    s
}

/// A twin's call: the session the runtime names, its words in the frontend API's `last_error` and the table's.
///
/// # Safety
/// `rt` a runtime this library made and has not freed, or null.
unsafe fn twin<T>(rt: *mut emusen_crystal_runtime, session: u32, call: impl FnOnce(&Call) -> Answer<T>) -> Result<T, i32> {
    let r = unsafe { runtime(rt) }?;
    let s = r.sessions.get(session).map_err(|(st, w)| fail(st, w))?;
    let c = Call { rt: r, s: &s, asked: s.id, read_only: false };
    call(&c).map_err(|(st, w)| {
        service_fail(st, &w);
        fail(st, w)
    })
}

/// A table's call: the session it serves, its words in the table's `last_error`.
///
/// # Safety
/// `ctx` a table's ctx this library made, or null.
unsafe fn table<T>(ctx: *mut c_void, session: u32, call: impl FnOnce(&Call) -> Answer<T>) -> Result<T, i32> {
    if ctx.is_null() {
        return Err(service_fail(svc::NULL, "a null ctx"));
    }
    let b = unsafe { &*(ctx as *const Binding) };
    let r = unsafe { &*b.rt };
    let s = r.sessions.get(b.id).map_err(|(st, w)| service_fail(st, &w))?;
    let c = Call { rt: r, s: &s, asked: session, read_only: b.read_only };
    call(&c).map_err(|(st, w)| service_fail(st, &w))
}

fn out(result: Result<Vec<u8>, i32>, o: *mut u8, len: usize) -> i64 {
    match result {
        Ok(bytes) => unsafe { put(&bytes, o, len) },
        Err(status) => status as i64,
    }
}

fn value(result: Result<i64, i32>) -> i64 {
    result.unwrap_or_else(|status| status as i64)
}

fn code(result: Result<i64, i32>) -> i32 {
    match result {
        Ok(_) => 0,
        Err(status) => status,
    }
}

// ---- The table's own: last_error, sessions, games ------------------------------------------------------------------

unsafe extern "C" fn last_error(_ctx: *mut c_void, o: *mut u8, len: usize) -> i64 {
    SERVICE_ERROR.with_borrow(|words| unsafe { put(words.as_bytes(), o, len) })
}

unsafe extern "C" fn sessions(ctx: *mut c_void, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, 0, machine::sessions) }, o, len)
}

unsafe extern "C" fn games(ctx: *mut c_void, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, 0, machine::games) }, o, len)
}

// ---- Session --------------------------------------------------------------------------------------------------------

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_info(rt: *mut emusen_crystal_runtime, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, machine::info) }, o, len)
}

unsafe extern "C" fn info(ctx: *mut c_void, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, machine::info) }, o, len)
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_summary(rt: *mut emusen_crystal_runtime, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, machine::summary) }, o, len)
}

unsafe extern "C" fn summary(ctx: *mut c_void, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, machine::summary) }, o, len)
}

// ---- Run control ----------------------------------------------------------------------------------------------------

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_run_control(rt: *mut emusen_crystal_runtime, session: u32, op: u32, processor: u32, argument: i64) -> i64 {
    value(unsafe { twin(rt, session, |c| machine::run_control(c, op, processor, argument)) })
}

unsafe extern "C" fn run_control(ctx: *mut c_void, session: u32, op: u32, processor: u32, argument: i64) -> i64 {
    value(unsafe { table(ctx, session, |c| machine::run_control(c, op, processor, argument)) })
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_halt_state(rt: *mut emusen_crystal_runtime, session: u32, processor: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| machine::halt_state(c, processor)) }, o, len)
}

unsafe extern "C" fn halt_state(ctx: *mut c_void, session: u32, processor: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| machine::halt_state(c, processor)) }, o, len)
}

// ---- Memory ---------------------------------------------------------------------------------------------------------

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_spaces(rt: *mut emusen_crystal_runtime, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, machine::spaces) }, o, len)
}

unsafe extern "C" fn spaces(ctx: *mut c_void, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, machine::spaces) }, o, len)
}

/// `len` bytes read into `o`, the whole length answered; a null `o` asks the length and reads nothing.
fn space_read_into(c: &Call, space: Raw, address: u32, o: *mut u8, len: usize) -> Answer<i64> {
    if o.is_null() {
        return Ok(len as i64);
    }
    if len > i32::MAX as usize {
        return Err((svc::FAULT, "Arithmetic operation resulted in an overflow.".to_string()));
    }
    let into = unsafe { std::slice::from_raw_parts_mut(o, len) };
    machine::space_read(c, space, address, into)?;
    Ok(len as i64)
}

/// # Safety
/// `rt` a live runtime; `space` valid for `space_len` bytes; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_space_read(rt: *mut emusen_crystal_runtime, session: u32, space: *const u8, space_len: usize, address: u32, o: *mut u8, len: usize) -> i64 {
    let space = unsafe { Raw::new(space, space_len) };
    value(unsafe { twin(rt, session, |c| space_read_into(c, space, address, o, len)) })
}

unsafe extern "C" fn space_read(ctx: *mut c_void, session: u32, space: *const u8, space_len: usize, address: u32, o: *mut u8, len: usize) -> i64 {
    if o.is_null() {
        return len as i64;
    }
    let space = unsafe { Raw::new(space, space_len) };
    value(unsafe { table(ctx, session, |c| space_read_into(c, space, address, o, len)) })
}

/// # Safety
/// `rt` a live runtime; `space` valid for `space_len` bytes; `data` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_space_write(rt: *mut emusen_crystal_runtime, session: u32, space: *const u8, space_len: usize, address: u32, data: *const u8, len: usize) -> i32 {
    let (space, data) = unsafe { (Raw::new(space, space_len), Raw::new(data, len)) };
    code(unsafe { twin(rt, session, |c| machine::space_write(c, space, address, data)) })
}

unsafe extern "C" fn space_write(ctx: *mut c_void, session: u32, space: *const u8, space_len: usize, address: u32, data: *const u8, len: usize) -> i32 {
    let (space, data) = unsafe { (Raw::new(space, space_len), Raw::new(data, len)) };
    code(unsafe { table(ctx, session, |c| machine::space_write(c, space, address, data)) })
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_resolve_physical(rt: *mut emusen_crystal_runtime, session: u32, address: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| machine::resolve_physical(c, address)) }, o, len)
}

unsafe extern "C" fn resolve_physical(ctx: *mut c_void, session: u32, address: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| machine::resolve_physical(c, address)) }, o, len)
}

// ---- Processors and registers ---------------------------------------------------------------------------------------

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_processors(rt: *mut emusen_crystal_runtime, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, machine::processors) }, o, len)
}

unsafe extern "C" fn processors(ctx: *mut c_void, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, machine::processors) }, o, len)
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_registers(rt: *mut emusen_crystal_runtime, session: u32, processor: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| machine::registers(c, processor)) }, o, len)
}

unsafe extern "C" fn registers(ctx: *mut c_void, session: u32, processor: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| machine::registers(c, processor)) }, o, len)
}

/// # Safety
/// `rt` a live runtime; `name` valid for `name_len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_set_register(rt: *mut emusen_crystal_runtime, session: u32, processor: u32, name: *const u8, name_len: usize, value: u64) -> i32 {
    let name = unsafe { Raw::new(name, name_len) };
    code(unsafe { twin(rt, session, |c| machine::set_register(c, processor, name, value)) })
}

unsafe extern "C" fn set_register(ctx: *mut c_void, session: u32, processor: u32, name: *const u8, name_len: usize, value: u64) -> i32 {
    let name = unsafe { Raw::new(name, name_len) };
    code(unsafe { table(ctx, session, |c| machine::set_register(c, processor, name, value)) })
}

// ---- Disassembly ----------------------------------------------------------------------------------------------------

/// # Safety
/// `rt` a live runtime; each text valid for its length; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_disassemble(
    rt: *mut emusen_crystal_runtime,
    session: u32,
    processor: u32,
    space: *const u8,
    space_len: usize,
    address: u32,
    count: u32,
    hints: *const u8,
    hints_len: usize,
    o: *mut u8,
    len: usize,
) -> i64 {
    let (space, hints) = unsafe { (Raw::new(space, space_len), Raw::new(hints, hints_len)) };
    out(unsafe { twin(rt, session, |c| machine::disassemble(c, processor, space, address, count, hints)) }, o, len)
}

#[allow(clippy::too_many_arguments)]
unsafe extern "C" fn disassemble(ctx: *mut c_void, session: u32, processor: u32, space: *const u8, space_len: usize, address: u32, count: u32, hints: *const u8, hints_len: usize, o: *mut u8, len: usize) -> i64 {
    let (space, hints) = unsafe { (Raw::new(space, space_len), Raw::new(hints, hints_len)) };
    out(unsafe { table(ctx, session, |c| machine::disassemble(c, processor, space, address, count, hints)) }, o, len)
}

// ---- States ---------------------------------------------------------------------------------------------------------

/// # Safety
/// `rt` a live runtime; `path` null or valid for `path_len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_state_save(rt: *mut emusen_crystal_runtime, session: u32, path: *const u8, path_len: usize) -> i32 {
    let path = unsafe { Raw::new(path, path_len) };
    code(unsafe { twin(rt, session, |c| machine::state_save(c, path)) })
}

unsafe extern "C" fn state_save(ctx: *mut c_void, session: u32, path: *const u8, path_len: usize) -> i32 {
    let path = unsafe { Raw::new(path, path_len) };
    code(unsafe { table(ctx, session, |c| machine::state_save(c, path)) })
}

/// # Safety
/// `rt` a live runtime; `path` null or valid for `path_len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_state_load(rt: *mut emusen_crystal_runtime, session: u32, path: *const u8, path_len: usize) -> i32 {
    let path = unsafe { Raw::new(path, path_len) };
    code(unsafe { twin(rt, session, |c| machine::state_load(c, path)) })
}

unsafe extern "C" fn state_load(ctx: *mut c_void, session: u32, path: *const u8, path_len: usize) -> i32 {
    let path = unsafe { Raw::new(path, path_len) };
    code(unsafe { table(ctx, session, |c| machine::state_load(c, path)) })
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_state_default_path(rt: *mut emusen_crystal_runtime, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, machine::state_default_path) }, o, len)
}

unsafe extern "C" fn state_default_path(ctx: *mut c_void, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, machine::state_default_path) }, o, len)
}

// ---- Logs: the process's switch -------------------------------------------------------------------------------------

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_log_switch(rt: *mut emusen_crystal_runtime, on: u32) -> i32 {
    match unsafe { runtime(rt) } {
        Ok(_) => {
            machine::set_log(on != 0);
            0
        }
        Err(status) => status,
    }
}

unsafe extern "C" fn log_switch(ctx: *mut c_void, on: u32) -> i32 {
    code(unsafe { table(ctx, 0, |c| machine::log_switch(c, on != 0)) })
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_log_status(rt: *mut emusen_crystal_runtime) -> i32 {
    match unsafe { runtime(rt) } {
        Ok(_) => machine::log_status() as i32,
        Err(status) => status,
    }
}

unsafe extern "C" fn log_status(ctx: *mut c_void) -> i32 {
    if ctx.is_null() {
        return service_fail(svc::NULL, "a null ctx");
    }
    machine::log_status() as i32
}

// ---- Events ---------------------------------------------------------------------------------------------------------

/// Run control's requests copied out and taken only when the buffer holds them all.
fn drain_into(c: &Call, kinds: u32, o: *mut u8, len: usize) -> Answer<i64> {
    let (bytes, taken) = machine::events_peek(c, kinds)?;
    if o.is_null() || len < bytes.len() {
        return Ok(bytes.len() as i64);
    }
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), o, bytes.len()) };
    machine::events_take(c, taken);
    Ok(bytes.len() as i64)
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_events_drain(rt: *mut emusen_crystal_runtime, session: u32, kinds: u32, o: *mut u8, len: usize) -> i64 {
    value(unsafe { twin(rt, session, |c| drain_into(c, kinds, o, len)) })
}

unsafe extern "C" fn events_drain(ctx: *mut c_void, session: u32, kinds: u32, o: *mut u8, len: usize) -> i64 {
    value(unsafe { table(ctx, session, |c| drain_into(c, kinds, o, len)) })
}

// ---- Games ----------------------------------------------------------------------------------------------------------

/// # Safety
/// `rt` a live runtime; each text valid for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_open_game(rt: *mut emusen_crystal_runtime, session: u32, path: *const u8, path_len: usize, engine: *const u8, engine_len: usize) -> i64 {
    let (path, engine) = unsafe { (Raw::new(path, path_len), Raw::new(engine, engine_len)) };
    value(unsafe { twin(rt, session, |c| machine::open(c, path, Raw::none(), engine, true)) })
}

unsafe extern "C" fn open_game(ctx: *mut c_void, session: u32, path: *const u8, path_len: usize, engine: *const u8, engine_len: usize) -> i64 {
    let (path, engine) = unsafe { (Raw::new(path, path_len), Raw::new(engine, engine_len)) };
    value(unsafe { table(ctx, session, |c| machine::open(c, path, Raw::none(), engine, true)) })
}

/// # Safety
/// `rt` a live runtime; each text valid for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_open_console(rt: *mut emusen_crystal_runtime, session: u32, console: *const u8, console_len: usize, engine: *const u8, engine_len: usize) -> i64 {
    let (console, engine) = unsafe { (Raw::new(console, console_len), Raw::new(engine, engine_len)) };
    value(unsafe { twin(rt, session, |c| machine::open(c, Raw::none(), console, engine, false)) })
}

unsafe extern "C" fn open_console(ctx: *mut c_void, session: u32, console: *const u8, console_len: usize, engine: *const u8, engine_len: usize) -> i64 {
    let (console, engine) = unsafe { (Raw::new(console, console_len), Raw::new(engine, engine_len)) };
    value(unsafe { table(ctx, session, |c| machine::open(c, Raw::none(), console, engine, false)) })
}

// ---- The table ------------------------------------------------------------------------------------------------------

/// A table with no entries, the start of one filled as the groups are built.
fn unbuilt() -> emusen_session_v1 {
    // SAFETY: every field is an integer, a pointer or an optional function pointer, for which zero is valid.
    unsafe { std::mem::zeroed() }
}

/// Every entry of the table, its ctx set by the slot that holds it.
fn template() -> emusen_session_v1 {
    emusen_session_v1 {
        size: std::mem::size_of::<emusen_session_v1>() as u32,
        version: emusen_session_service::abi::VERSION,
        ctx: std::ptr::null_mut(),
        last_error: Some(last_error),
        sessions: Some(sessions),
        info: Some(info),
        summary: Some(summary),
        run_control: Some(run_control),
        halt_state: Some(halt_state),
        spaces: Some(spaces),
        space_read: Some(space_read),
        space_write: Some(space_write),
        resolve_physical: Some(resolve_physical),
        processors: Some(processors),
        registers: Some(registers),
        set_register: Some(set_register),
        disassemble: Some(disassemble),
        state_save: Some(state_save),
        state_load: Some(state_load),
        state_default_path: Some(state_default_path),
        log_switch: Some(log_switch),
        log_status: Some(log_status),
        events_drain: Some(events_drain),
        games: Some(games),
        open_game: Some(open_game),
        open_console: Some(open_console),
        ..unbuilt()
    }
}

/// The session's `emusen.session@1` table, read-only where the flag asks; null with the words in `last_error`.
///
/// # Safety
/// `rt` a runtime this library made and has not freed, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_session_service(rt: *mut emusen_crystal_runtime, session: u32, flags: u32) -> *const emusen_session_v1 {
    if flags & !abi::SERVICE_READ_ONLY != 0 {
        fail(svc::NOT_SUPPORTED, format!("service flags 0x{flags:X} are not known to 1.{}", abi::MINOR));
        return std::ptr::null();
    }
    let Ok(r) = (unsafe { runtime(rt) }) else { return std::ptr::null() };
    let s = match r.sessions.get(session) {
        Ok(s) => s,
        Err((st, w)) => {
            fail(st, w);
            return std::ptr::null();
        }
    };
    let mut all = TABLES.lock().unwrap_or_else(|p| p.into_inner());
    let at = match all.iter().position(|t| t.rt == rt as usize && t.id == s.id) {
        Some(at) => at,
        None => {
            let runtime = rt as *const Runtime;
            all.push(Tables { rt: rt as usize, id: s.id, full: slot(runtime, s.id, false), read_only: slot(runtime, s.id, true) });
            all.len() - 1
        }
    };
    let t = &all[at];
    if flags & abi::SERVICE_READ_ONLY != 0 { &t.read_only.table } else { &t.full.table }
}
