//! The session service's twins, `emusen_crystal.h`'s functions after `session_service`, and the session's
//! `emusen_session_v1` table built from them, over `emusen-crystal`'s `service`: a twin names its session through the
//! runtime, a table entry through the table it was handed, and both make one call (EmuSen_Backend.md §29.3, §29.21).

use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::Mutex;

use emusen_crystal::abi::{self, emusen_crystal_runtime};
use emusen_crystal::runtime::Runtime;
use emusen_crystal::service::{Answer, Call, Raw, machine, registries, views};
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

fn number(result: Result<i64, i32>) -> i64 {
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
    number(unsafe { twin(rt, session, |c| machine::run_control(c, op, processor, argument)) })
}

unsafe extern "C" fn run_control(ctx: *mut c_void, session: u32, op: u32, processor: u32, argument: i64) -> i64 {
    number(unsafe { table(ctx, session, |c| machine::run_control(c, op, processor, argument)) })
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
    number(unsafe { twin(rt, session, |c| space_read_into(c, space, address, o, len)) })
}

unsafe extern "C" fn space_read(ctx: *mut c_void, session: u32, space: *const u8, space_len: usize, address: u32, o: *mut u8, len: usize) -> i64 {
    if o.is_null() {
        return len as i64;
    }
    let space = unsafe { Raw::new(space, space_len) };
    number(unsafe { table(ctx, session, |c| space_read_into(c, space, address, o, len)) })
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
    number(unsafe { twin(rt, session, |c| drain_into(c, kinds, o, len)) })
}

unsafe extern "C" fn events_drain(ctx: *mut c_void, session: u32, kinds: u32, o: *mut u8, len: usize) -> i64 {
    number(unsafe { table(ctx, session, |c| drain_into(c, kinds, o, len)) })
}

// ---- Games ----------------------------------------------------------------------------------------------------------

/// # Safety
/// `rt` a live runtime; each text valid for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_open_game(rt: *mut emusen_crystal_runtime, session: u32, path: *const u8, path_len: usize, engine: *const u8, engine_len: usize) -> i64 {
    let (path, engine) = unsafe { (Raw::new(path, path_len), Raw::new(engine, engine_len)) };
    number(unsafe { twin(rt, session, |c| machine::open(c, path, Raw::none(), engine, true)) })
}

unsafe extern "C" fn open_game(ctx: *mut c_void, session: u32, path: *const u8, path_len: usize, engine: *const u8, engine_len: usize) -> i64 {
    let (path, engine) = unsafe { (Raw::new(path, path_len), Raw::new(engine, engine_len)) };
    number(unsafe { table(ctx, session, |c| machine::open(c, path, Raw::none(), engine, true)) })
}

/// # Safety
/// `rt` a live runtime; each text valid for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_open_console(rt: *mut emusen_crystal_runtime, session: u32, console: *const u8, console_len: usize, engine: *const u8, engine_len: usize) -> i64 {
    let (console, engine) = unsafe { (Raw::new(console, console_len), Raw::new(engine, engine_len)) };
    number(unsafe { twin(rt, session, |c| machine::open(c, Raw::none(), console, engine, false)) })
}

unsafe extern "C" fn open_console(ctx: *mut c_void, session: u32, console: *const u8, console_len: usize, engine: *const u8, engine_len: usize) -> i64 {
    let (console, engine) = unsafe { (Raw::new(console, console_len), Raw::new(engine, engine_len)) };
    number(unsafe { table(ctx, session, |c| machine::open(c, Raw::none(), console, engine, false)) })
}

// ---- The registries: breakpoints, watches, frame logs, coverage, profile, counters, call stack, freezes, labels ----

/// The room a change that answers with a record has, -1 for a length query, as `SessionExports.LabelLoad` passes it.
fn room(o: *mut u8, len: usize) -> i64 {
    if o.is_null() { -1 } else { len.min(i64::MAX as usize) as i64 }
}

/// # Safety
/// `rt` a live runtime; each text valid for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_bp_add(rt: *mut emusen_crystal_runtime, session: u32, processor: u32, spec: *const u8, spec_len: usize) -> i64 {
    let spec = unsafe { Raw::new(spec, spec_len) };
    number(unsafe { twin(rt, session, |c| registries::bp_add(c, processor, spec)) })
}

unsafe extern "C" fn bp_add(ctx: *mut c_void, session: u32, processor: u32, spec: *const u8, spec_len: usize) -> i64 {
    let spec = unsafe { Raw::new(spec, spec_len) };
    number(unsafe { table(ctx, session, |c| registries::bp_add(c, processor, spec)) })
}

/// # Safety
/// `rt` a live runtime; each text valid for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_bp_add_data(rt: *mut emusen_crystal_runtime, session: u32, processor: u32, spec: *const u8, spec_len: usize) -> i64 {
    let spec = unsafe { Raw::new(spec, spec_len) };
    number(unsafe { twin(rt, session, |c| registries::bp_add_data(c, processor, spec)) })
}

unsafe extern "C" fn bp_add_data(ctx: *mut c_void, session: u32, processor: u32, spec: *const u8, spec_len: usize) -> i64 {
    let spec = unsafe { Raw::new(spec, spec_len) };
    number(unsafe { table(ctx, session, |c| registries::bp_add_data(c, processor, spec)) })
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_bp_set_enabled(rt: *mut emusen_crystal_runtime, session: u32, processor: u32, id: u32, enabled: u32) -> i32 {
    code(unsafe { twin(rt, session, |c| registries::bp_set_enabled(c, processor, id, enabled != 0)) })
}

unsafe extern "C" fn bp_set_enabled(ctx: *mut c_void, session: u32, processor: u32, id: u32, enabled: u32) -> i32 {
    code(unsafe { table(ctx, session, |c| registries::bp_set_enabled(c, processor, id, enabled != 0)) })
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_bp_remove(rt: *mut emusen_crystal_runtime, session: u32, processor: u32, id: u32) -> i32 {
    code(unsafe { twin(rt, session, |c| registries::bp_remove(c, processor, id)) })
}

unsafe extern "C" fn bp_remove(ctx: *mut c_void, session: u32, processor: u32, id: u32) -> i32 {
    code(unsafe { table(ctx, session, |c| registries::bp_remove(c, processor, id)) })
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_bp_list(rt: *mut emusen_crystal_runtime, session: u32, processor: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| registries::bp_list(c, processor)) }, o, len)
}

unsafe extern "C" fn bp_list(ctx: *mut c_void, session: u32, processor: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| registries::bp_list(c, processor)) }, o, len)
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_bp_log(rt: *mut emusen_crystal_runtime, session: u32, processor: u32, count: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| registries::bp_log(c, processor, count)) }, o, len)
}

unsafe extern "C" fn bp_log(ctx: *mut c_void, session: u32, processor: u32, count: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| registries::bp_log(c, processor, count)) }, o, len)
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_bp_log_clear(rt: *mut emusen_crystal_runtime, session: u32, processor: u32) -> i32 {
    code(unsafe { twin(rt, session, |c| registries::bp_log_clear(c, processor)) })
}

unsafe extern "C" fn bp_log_clear(ctx: *mut c_void, session: u32, processor: u32) -> i32 {
    code(unsafe { table(ctx, session, |c| registries::bp_log_clear(c, processor)) })
}

/// # Safety
/// `rt` a live runtime; each text valid for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_bp_arm_condition(rt: *mut emusen_crystal_runtime, session: u32, processor: u32, name: *const u8, name_len: usize, flags: u32) -> i32 {
    let name = unsafe { Raw::new(name, name_len) };
    code(unsafe { twin(rt, session, |c| registries::bp_arm_condition(c, processor, name, flags)) })
}

unsafe extern "C" fn bp_arm_condition(ctx: *mut c_void, session: u32, processor: u32, name: *const u8, name_len: usize, flags: u32) -> i32 {
    let name = unsafe { Raw::new(name, name_len) };
    code(unsafe { table(ctx, session, |c| registries::bp_arm_condition(c, processor, name, flags)) })
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_bp_conditions(rt: *mut emusen_crystal_runtime, session: u32, processor: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| registries::bp_conditions(c, processor)) }, o, len)
}

unsafe extern "C" fn bp_conditions(ctx: *mut c_void, session: u32, processor: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| registries::bp_conditions(c, processor)) }, o, len)
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_bp_depth_guard(rt: *mut emusen_crystal_runtime, session: u32, processor: u32, depth: i32) -> i32 {
    code(unsafe { twin(rt, session, |c| registries::bp_depth_guard(c, processor, depth)) })
}

unsafe extern "C" fn bp_depth_guard(ctx: *mut c_void, session: u32, processor: u32, depth: i32) -> i32 {
    code(unsafe { table(ctx, session, |c| registries::bp_depth_guard(c, processor, depth)) })
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_bp_forbid_range(rt: *mut emusen_crystal_runtime, session: u32, processor: u32, address: u32, end: u32) -> i64 {
    number(unsafe { twin(rt, session, |c| registries::bp_forbid_range(c, processor, address, end)) })
}

unsafe extern "C" fn bp_forbid_range(ctx: *mut c_void, session: u32, processor: u32, address: u32, end: u32) -> i64 {
    number(unsafe { table(ctx, session, |c| registries::bp_forbid_range(c, processor, address, end)) })
}

/// # Safety
/// `rt` a live runtime; each text valid for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_bp_uninitialized(rt: *mut emusen_crystal_runtime, session: u32, processor: u32, space: *const u8, space_len: usize, size: u32) -> i32 {
    let space = unsafe { Raw::new(space, space_len) };
    code(unsafe { twin(rt, session, |c| registries::bp_uninitialized(c, processor, space, size)) })
}

unsafe extern "C" fn bp_uninitialized(ctx: *mut c_void, session: u32, processor: u32, space: *const u8, space_len: usize, size: u32) -> i32 {
    let space = unsafe { Raw::new(space, space_len) };
    code(unsafe { table(ctx, session, |c| registries::bp_uninitialized(c, processor, space, size)) })
}

/// # Safety
/// `rt` a live runtime; each text valid for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_watch_add(rt: *mut emusen_crystal_runtime, session: u32, space: *const u8, space_len: usize, address: u32, length: u32, kind: u32) -> i64 {
    let space = unsafe { Raw::new(space, space_len) };
    number(unsafe { twin(rt, session, |c| registries::watch_add(c, space, address, length, kind)) })
}

unsafe extern "C" fn watch_add(ctx: *mut c_void, session: u32, space: *const u8, space_len: usize, address: u32, length: u32, kind: u32) -> i64 {
    let space = unsafe { Raw::new(space, space_len) };
    number(unsafe { table(ctx, session, |c| registries::watch_add(c, space, address, length, kind)) })
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_watch_remove(rt: *mut emusen_crystal_runtime, session: u32, id: u32) -> i32 {
    code(unsafe { twin(rt, session, |c| registries::watch_remove(c, id)) })
}

unsafe extern "C" fn watch_remove(ctx: *mut c_void, session: u32, id: u32) -> i32 {
    code(unsafe { table(ctx, session, |c| registries::watch_remove(c, id)) })
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_watch_list(rt: *mut emusen_crystal_runtime, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| registries::watch_list(c)) }, o, len)
}

unsafe extern "C" fn watch_list(ctx: *mut c_void, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| registries::watch_list(c)) }, o, len)
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_watch_events(rt: *mut emusen_crystal_runtime, session: u32, id: u32, count: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| registries::watch_events(c, id, count)) }, o, len)
}

unsafe extern "C" fn watch_events(ctx: *mut c_void, session: u32, id: u32, count: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| registries::watch_events(c, id, count)) }, o, len)
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_watch_summary(rt: *mut emusen_crystal_runtime, session: u32, id: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| registries::watch_summary(c, id)) }, o, len)
}

unsafe extern "C" fn watch_summary(ctx: *mut c_void, session: u32, id: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| registries::watch_summary(c, id)) }, o, len)
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_watch_clear(rt: *mut emusen_crystal_runtime, session: u32, id: u32) -> i32 {
    code(unsafe { twin(rt, session, |c| registries::watch_clear(c, id)) })
}

unsafe extern "C" fn watch_clear(ctx: *mut c_void, session: u32, id: u32) -> i32 {
    code(unsafe { table(ctx, session, |c| registries::watch_clear(c, id)) })
}

/// # Safety
/// `rt` a live runtime; each text valid for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_framelog_add(rt: *mut emusen_crystal_runtime, session: u32, space: *const u8, space_len: usize, address: u32, width: u32) -> i64 {
    let space = unsafe { Raw::new(space, space_len) };
    number(unsafe { twin(rt, session, |c| registries::framelog_add(c, space, address, width)) })
}

unsafe extern "C" fn framelog_add(ctx: *mut c_void, session: u32, space: *const u8, space_len: usize, address: u32, width: u32) -> i64 {
    let space = unsafe { Raw::new(space, space_len) };
    number(unsafe { table(ctx, session, |c| registries::framelog_add(c, space, address, width)) })
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_framelog_remove(rt: *mut emusen_crystal_runtime, session: u32, id: u32) -> i32 {
    code(unsafe { twin(rt, session, |c| registries::framelog_remove(c, id)) })
}

unsafe extern "C" fn framelog_remove(ctx: *mut c_void, session: u32, id: u32) -> i32 {
    code(unsafe { table(ctx, session, |c| registries::framelog_remove(c, id)) })
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_framelog_list(rt: *mut emusen_crystal_runtime, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| registries::framelog_list(c)) }, o, len)
}

unsafe extern "C" fn framelog_list(ctx: *mut c_void, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| registries::framelog_list(c)) }, o, len)
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_framelog_samples(rt: *mut emusen_crystal_runtime, session: u32, id: u32, count: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| registries::framelog_samples(c, id, count)) }, o, len)
}

unsafe extern "C" fn framelog_samples(ctx: *mut c_void, session: u32, id: u32, count: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| registries::framelog_samples(c, id, count)) }, o, len)
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_framelog_clear(rt: *mut emusen_crystal_runtime, session: u32, id: u32) -> i32 {
    code(unsafe { twin(rt, session, |c| registries::framelog_clear(c, id)) })
}

unsafe extern "C" fn framelog_clear(ctx: *mut c_void, session: u32, id: u32) -> i32 {
    code(unsafe { table(ctx, session, |c| registries::framelog_clear(c, id)) })
}

/// # Safety
/// `rt` a live runtime; each text valid for its length; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_coverage_op(rt: *mut emusen_crystal_runtime, session: u32, processor: u32, op: u32, args: *const u8, args_len: usize, o: *mut u8, len: usize) -> i64 {
    let args = unsafe { Raw::new(args, args_len) };
    out(unsafe { twin(rt, session, |c| registries::coverage_op(c, processor, op, args)) }, o, len)
}

unsafe extern "C" fn coverage_op(ctx: *mut c_void, session: u32, processor: u32, op: u32, args: *const u8, args_len: usize, o: *mut u8, len: usize) -> i64 {
    let args = unsafe { Raw::new(args, args_len) };
    out(unsafe { table(ctx, session, |c| registries::coverage_op(c, processor, op, args)) }, o, len)
}

/// # Safety
/// `rt` a live runtime; each text valid for its length; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_profile_op(rt: *mut emusen_crystal_runtime, session: u32, processor: u32, op: u32, args: *const u8, args_len: usize, o: *mut u8, len: usize) -> i64 {
    let args = unsafe { Raw::new(args, args_len) };
    out(unsafe { twin(rt, session, |c| registries::profile_op(c, processor, op, args)) }, o, len)
}

unsafe extern "C" fn profile_op(ctx: *mut c_void, session: u32, processor: u32, op: u32, args: *const u8, args_len: usize, o: *mut u8, len: usize) -> i64 {
    let args = unsafe { Raw::new(args, args_len) };
    out(unsafe { table(ctx, session, |c| registries::profile_op(c, processor, op, args)) }, o, len)
}

/// # Safety
/// `rt` a live runtime; each text valid for its length; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_counters_op(rt: *mut emusen_crystal_runtime, session: u32, op: u32, args: *const u8, args_len: usize, o: *mut u8, len: usize) -> i64 {
    let args = unsafe { Raw::new(args, args_len) };
    out(unsafe { twin(rt, session, |c| registries::counters_op(c, { let _ = op; args })) }, o, len)
}

unsafe extern "C" fn counters_op(ctx: *mut c_void, session: u32, op: u32, args: *const u8, args_len: usize, o: *mut u8, len: usize) -> i64 {
    let args = unsafe { Raw::new(args, args_len) };
    out(unsafe { table(ctx, session, |c| registries::counters_op(c, { let _ = op; args })) }, o, len)
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_call_stack(rt: *mut emusen_crystal_runtime, session: u32, processor: u32, count: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| registries::call_stack(c, processor, count)) }, o, len)
}

unsafe extern "C" fn call_stack(ctx: *mut c_void, session: u32, processor: u32, count: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| registries::call_stack(c, processor, count)) }, o, len)
}

/// # Safety
/// `rt` a live runtime; each text valid for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_freeze_add(rt: *mut emusen_crystal_runtime, session: u32, space: *const u8, space_len: usize, address: u32, value: u32) -> i64 {
    let space = unsafe { Raw::new(space, space_len) };
    number(unsafe { twin(rt, session, |c| registries::freeze_add(c, space, address, value)) })
}

unsafe extern "C" fn freeze_add(ctx: *mut c_void, session: u32, space: *const u8, space_len: usize, address: u32, value: u32) -> i64 {
    let space = unsafe { Raw::new(space, space_len) };
    number(unsafe { table(ctx, session, |c| registries::freeze_add(c, space, address, value)) })
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_freeze_remove(rt: *mut emusen_crystal_runtime, session: u32, id: u32) -> i32 {
    code(unsafe { twin(rt, session, |c| registries::freeze_remove(c, id)) })
}

unsafe extern "C" fn freeze_remove(ctx: *mut c_void, session: u32, id: u32) -> i32 {
    code(unsafe { table(ctx, session, |c| registries::freeze_remove(c, id)) })
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_freeze_list(rt: *mut emusen_crystal_runtime, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| registries::freeze_list(c)) }, o, len)
}

unsafe extern "C" fn freeze_list(ctx: *mut c_void, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| registries::freeze_list(c)) }, o, len)
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_freeze_clear(rt: *mut emusen_crystal_runtime, session: u32) -> i32 {
    code(unsafe { twin(rt, session, |c| registries::freeze_clear(c)) })
}

unsafe extern "C" fn freeze_clear(ctx: *mut c_void, session: u32) -> i32 {
    code(unsafe { table(ctx, session, |c| registries::freeze_clear(c)) })
}

/// # Safety
/// `rt` a live runtime; each text valid for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_label_add(rt: *mut emusen_crystal_runtime, session: u32, address: u32, name: *const u8, name_len: usize, comment: *const u8, comment_len: usize) -> i32 {
    let (name, comment) = unsafe { (Raw::new(name, name_len), Raw::new(comment, comment_len)) };
    code(unsafe { twin(rt, session, |c| registries::label_add(c, address, name, comment)) })
}

unsafe extern "C" fn label_add(ctx: *mut c_void, session: u32, address: u32, name: *const u8, name_len: usize, comment: *const u8, comment_len: usize) -> i32 {
    let (name, comment) = unsafe { (Raw::new(name, name_len), Raw::new(comment, comment_len)) };
    code(unsafe { table(ctx, session, |c| registries::label_add(c, address, name, comment)) })
}

/// # Safety
/// `rt` a live runtime; each text valid for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_label_remove(rt: *mut emusen_crystal_runtime, session: u32, name: *const u8, name_len: usize, address: u32) -> i32 {
    let name = unsafe { Raw::new(name, name_len) };
    code(unsafe { twin(rt, session, |c| registries::label_remove(c, name, address)) })
}

unsafe extern "C" fn label_remove(ctx: *mut c_void, session: u32, name: *const u8, name_len: usize, address: u32) -> i32 {
    let name = unsafe { Raw::new(name, name_len) };
    code(unsafe { table(ctx, session, |c| registries::label_remove(c, name, address)) })
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_label_list(rt: *mut emusen_crystal_runtime, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| registries::label_list(c)) }, o, len)
}

unsafe extern "C" fn label_list(ctx: *mut c_void, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| registries::label_list(c)) }, o, len)
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_label_at(rt: *mut emusen_crystal_runtime, session: u32, address: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| registries::label_at(c, address)) }, o, len)
}

unsafe extern "C" fn label_at(ctx: *mut c_void, session: u32, address: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| registries::label_at(c, address)) }, o, len)
}

/// # Safety
/// `rt` a live runtime; each text valid for its length; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_label_load(rt: *mut emusen_crystal_runtime, session: u32, text: *const u8, text_len: usize, o: *mut u8, len: usize) -> i64 {
    let text = unsafe { Raw::new(text, text_len) };
    out(unsafe { twin(rt, session, |c| registries::label_load(c, text, room(o, len))) }, o, len)
}

unsafe extern "C" fn label_load(ctx: *mut c_void, session: u32, text: *const u8, text_len: usize, o: *mut u8, len: usize) -> i64 {
    let text = unsafe { Raw::new(text, text_len) };
    out(unsafe { table(ctx, session, |c| registries::label_load(c, text, room(o, len))) }, o, len)
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_label_save(rt: *mut emusen_crystal_runtime, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| registries::label_save(c)) }, o, len)
}

unsafe extern "C" fn label_save(ctx: *mut c_void, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| registries::label_save(c)) }, o, len)
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_label_clear(rt: *mut emusen_crystal_runtime, session: u32) -> i32 {
    code(unsafe { twin(rt, session, |c| registries::label_clear(c)) })
}

unsafe extern "C" fn label_clear(ctx: *mut c_void, session: u32) -> i32 {
    code(unsafe { table(ctx, session, |c| registries::label_clear(c)) })
}

// ---- Chip views, expressions, trace and telemetry --------------------------------------------------------------

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_dma_channels(rt: *mut emusen_crystal_runtime, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| views::dma_channels(c)) }, o, len)
}

unsafe extern "C" fn dma_channels(ctx: *mut c_void, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| views::dma_channels(c)) }, o, len)
}

/// # Safety
/// `rt` a live runtime; each text valid for its length; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_dma_log_op(rt: *mut emusen_crystal_runtime, session: u32, op: u32, args: *const u8, args_len: usize, o: *mut u8, len: usize) -> i64 {
    let args = unsafe { Raw::new(args, args_len) };
    out(unsafe { twin(rt, session, |c| views::dma_log_op(c, { let _ = op; args })) }, o, len)
}

unsafe extern "C" fn dma_log_op(ctx: *mut c_void, session: u32, op: u32, args: *const u8, args_len: usize, o: *mut u8, len: usize) -> i64 {
    let args = unsafe { Raw::new(args, args_len) };
    out(unsafe { table(ctx, session, |c| views::dma_log_op(c, { let _ = op; args })) }, o, len)
}

/// # Safety
/// `rt` a live runtime; each text valid for its length; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_register_flow_op(rt: *mut emusen_crystal_runtime, session: u32, op: u32, args: *const u8, args_len: usize, o: *mut u8, len: usize) -> i64 {
    let args = unsafe { Raw::new(args, args_len) };
    out(unsafe { twin(rt, session, |c| views::register_flow_op(c, { let _ = op; args })) }, o, len)
}

unsafe extern "C" fn register_flow_op(ctx: *mut c_void, session: u32, op: u32, args: *const u8, args_len: usize, o: *mut u8, len: usize) -> i64 {
    let args = unsafe { Raw::new(args, args_len) };
    out(unsafe { table(ctx, session, |c| views::register_flow_op(c, { let _ = op; args })) }, o, len)
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_coprocessor_history(rt: *mut emusen_crystal_runtime, session: u32, count: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| views::coprocessor_history({ let _ = count; c })) }, o, len)
}

unsafe extern "C" fn coprocessor_history(ctx: *mut c_void, session: u32, count: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| views::coprocessor_history({ let _ = count; c })) }, o, len)
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_vectors(rt: *mut emusen_crystal_runtime, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| views::vectors(c)) }, o, len)
}

unsafe extern "C" fn vectors(ctx: *mut c_void, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| views::vectors(c)) }, o, len)
}

/// # Safety
/// `rt` a live runtime; each text valid for its length; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_tilemap_entry(rt: *mut emusen_crystal_runtime, session: u32, space: *const u8, space_len: usize, address: u32, o: *mut u8, len: usize) -> i64 {
    let space = unsafe { Raw::new(space, space_len) };
    out(unsafe { twin(rt, session, |c| views::tilemap_entry(c, space, address)) }, o, len)
}

unsafe extern "C" fn tilemap_entry(ctx: *mut c_void, session: u32, space: *const u8, space_len: usize, address: u32, o: *mut u8, len: usize) -> i64 {
    let space = unsafe { Raw::new(space, space_len) };
    out(unsafe { table(ctx, session, |c| views::tilemap_entry(c, space, address)) }, o, len)
}

/// # Safety
/// `rt` a live runtime; each text valid for its length; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_tile_pixels(rt: *mut emusen_crystal_runtime, session: u32, space: *const u8, space_len: usize, address: u32, bpp: u32, o: *mut u8, len: usize) -> i64 {
    let space = unsafe { Raw::new(space, space_len) };
    out(unsafe { twin(rt, session, |c| views::tile_pixels(c, space, address, bpp)) }, o, len)
}

unsafe extern "C" fn tile_pixels(ctx: *mut c_void, session: u32, space: *const u8, space_len: usize, address: u32, bpp: u32, o: *mut u8, len: usize) -> i64 {
    let space = unsafe { Raw::new(space, space_len) };
    out(unsafe { table(ctx, session, |c| views::tile_pixels(c, space, address, bpp)) }, o, len)
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_sprites(rt: *mut emusen_crystal_runtime, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| views::sprites(c)) }, o, len)
}

unsafe extern "C" fn sprites(ctx: *mut c_void, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| views::sprites(c)) }, o, len)
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_palettes(rt: *mut emusen_crystal_runtime, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| views::palettes(c)) }, o, len)
}

unsafe extern "C" fn palettes(ctx: *mut c_void, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| views::palettes(c)) }, o, len)
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_audio_channels(rt: *mut emusen_crystal_runtime, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| views::audio_channels(c)) }, o, len)
}

unsafe extern "C" fn audio_channels(ctx: *mut c_void, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| views::audio_channels(c)) }, o, len)
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_set_channel_muted(rt: *mut emusen_crystal_runtime, session: u32, channel: u32, muted: u32) -> i32 {
    code(unsafe { twin(rt, session, |c| views::set_channel_muted(c, channel, muted != 0)) })
}

unsafe extern "C" fn set_channel_muted(ctx: *mut c_void, session: u32, channel: u32, muted: u32) -> i32 {
    code(unsafe { table(ctx, session, |c| views::set_channel_muted(c, channel, muted != 0)) })
}

/// # Safety
/// `rt` a live runtime; each text valid for its length; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_evaluate(rt: *mut emusen_crystal_runtime, session: u32, processor: u32, text: *const u8, text_len: usize, o: *mut u8, len: usize) -> i64 {
    let text = unsafe { Raw::new(text, text_len) };
    out(unsafe { twin(rt, session, |c| views::evaluate(c, processor, text)) }, o, len)
}

unsafe extern "C" fn evaluate(ctx: *mut c_void, session: u32, processor: u32, text: *const u8, text_len: usize, o: *mut u8, len: usize) -> i64 {
    let text = unsafe { Raw::new(text, text_len) };
    out(unsafe { table(ctx, session, |c| views::evaluate(c, processor, text)) }, o, len)
}

/// # Safety
/// `rt` a live runtime; each text valid for its length; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_symbols(rt: *mut emusen_crystal_runtime, session: u32, processor: u32, filter: *const u8, filter_len: usize, o: *mut u8, len: usize) -> i64 {
    let filter = unsafe { Raw::new(filter, filter_len) };
    out(unsafe { twin(rt, session, |c| views::symbols(c, processor, filter)) }, o, len)
}

unsafe extern "C" fn symbols(ctx: *mut c_void, session: u32, processor: u32, filter: *const u8, filter_len: usize, o: *mut u8, len: usize) -> i64 {
    let filter = unsafe { Raw::new(filter, filter_len) };
    out(unsafe { table(ctx, session, |c| views::symbols(c, processor, filter)) }, o, len)
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_trace(rt: *mut emusen_crystal_runtime, session: u32, processor: u32, count: u32) -> i32 {
    code(unsafe { twin(rt, session, |c| views::trace(c, processor, count)) })
}

unsafe extern "C" fn trace(ctx: *mut c_void, session: u32, processor: u32, count: u32) -> i32 {
    code(unsafe { table(ctx, session, |c| views::trace(c, processor, count)) })
}

/// # Safety
/// `rt` a live runtime.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_trace_off(rt: *mut emusen_crystal_runtime, session: u32, processor: u32) -> i32 {
    code(unsafe { twin(rt, session, |c| views::trace_off(c, processor)) })
}

unsafe extern "C" fn trace_off(ctx: *mut c_void, session: u32, processor: u32) -> i32 {
    code(unsafe { table(ctx, session, |c| views::trace_off(c, processor)) })
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_telemetry(rt: *mut emusen_crystal_runtime, session: u32, kind: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| views::telemetry(c, kind)) }, o, len)
}

unsafe extern "C" fn telemetry(ctx: *mut c_void, session: u32, kind: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| views::telemetry(c, kind)) }, o, len)
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_tile_sheet(rt: *mut emusen_crystal_runtime, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| views::tile_sheet(c)) }, o, len)
}

unsafe extern "C" fn tile_sheet(ctx: *mut c_void, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| views::tile_sheet(c)) }, o, len)
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_palette_swatch(rt: *mut emusen_crystal_runtime, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| views::palette_swatch(c)) }, o, len)
}

unsafe extern "C" fn palette_swatch(ctx: *mut c_void, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| views::palette_swatch(c)) }, o, len)
}

/// # Safety
/// `rt` a live runtime; `out` null or valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_crystal_audio_peek(rt: *mut emusen_crystal_runtime, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { twin(rt, session, |c| views::audio_peek(c)) }, o, len)
}

unsafe extern "C" fn audio_peek(ctx: *mut c_void, session: u32, o: *mut u8, len: usize) -> i64 {
    out(unsafe { table(ctx, session, |c| views::audio_peek(c)) }, o, len)
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
        bp_add: Some(bp_add),
        bp_add_data: Some(bp_add_data),
        bp_set_enabled: Some(bp_set_enabled),
        bp_remove: Some(bp_remove),
        bp_list: Some(bp_list),
        bp_log: Some(bp_log),
        bp_log_clear: Some(bp_log_clear),
        bp_arm_condition: Some(bp_arm_condition),
        bp_conditions: Some(bp_conditions),
        bp_depth_guard: Some(bp_depth_guard),
        bp_forbid_range: Some(bp_forbid_range),
        bp_uninitialized: Some(bp_uninitialized),
        watch_add: Some(watch_add),
        watch_remove: Some(watch_remove),
        watch_list: Some(watch_list),
        watch_events: Some(watch_events),
        watch_summary: Some(watch_summary),
        watch_clear: Some(watch_clear),
        framelog_add: Some(framelog_add),
        framelog_remove: Some(framelog_remove),
        framelog_list: Some(framelog_list),
        framelog_samples: Some(framelog_samples),
        framelog_clear: Some(framelog_clear),
        coverage_op: Some(coverage_op),
        profile_op: Some(profile_op),
        counters_op: Some(counters_op),
        call_stack: Some(call_stack),
        freeze_add: Some(freeze_add),
        freeze_remove: Some(freeze_remove),
        freeze_list: Some(freeze_list),
        freeze_clear: Some(freeze_clear),
        label_add: Some(label_add),
        label_remove: Some(label_remove),
        label_list: Some(label_list),
        label_at: Some(label_at),
        label_load: Some(label_load),
        label_save: Some(label_save),
        label_clear: Some(label_clear),
        dma_channels: Some(dma_channels),
        dma_log_op: Some(dma_log_op),
        register_flow_op: Some(register_flow_op),
        coprocessor_history: Some(coprocessor_history),
        vectors: Some(vectors),
        tilemap_entry: Some(tilemap_entry),
        tile_pixels: Some(tile_pixels),
        sprites: Some(sprites),
        palettes: Some(palettes),
        audio_channels: Some(audio_channels),
        set_channel_muted: Some(set_channel_muted),
        evaluate: Some(evaluate),
        symbols: Some(symbols),
        trace: Some(trace),
        trace_off: Some(trace_off),
        telemetry: Some(telemetry),
        tile_sheet: Some(tile_sheet),
        palette_swatch: Some(palette_swatch),
        audio_peek: Some(audio_peek),
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
