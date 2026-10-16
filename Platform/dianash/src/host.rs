//! Calling a host's command: the call and its io table built for it, the host's functions called on this thread,
//! and what they wrote gathered back. A host's command is called exactly as a module's `run` is (§10.3), with the
//! io table's rules of §6.0: the pointers it is handed are good only until it returns.

use crate::abi::*;
use crate::dotnet;
use crate::interp::{Interp, Outcome};
use crate::sandbox::Sandbox;
use crate::shell::{Request, Slot, World};
use crate::table::Kind;
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// One call's context, which every io function is handed as its `ctx`.
struct CallContext {
    /// What the call wrote, in order, each piece marked standard error or not.
    written: Vec<(bool, Vec<u8>)>,
    sandbox: Sandbox,
    slots: *mut HashMap<usize, Slot>,
    registration: usize,
    cancel: *const AtomicBool,
    started: Option<Instant>,
    budget: Duration,
    environment: HashMap<String, String>,
    interactive: bool,
}

impl CallContext {
    fn output(&self, include_err: bool) -> String {
        let mut bytes = Vec::new();
        for (is_err, piece) in &self.written {
            if include_err || !is_err {
                bytes.extend_from_slice(piece);
            }
        }
        String::from_utf8_lossy(&bytes).into_owned()
    }

    /// One stream alone: standard error, or standard output.
    fn stream(&self, err: bool) -> String {
        let bytes: Vec<u8> = self.written.iter().filter(|(is_err, _)| *is_err == err).flat_map(|(_, piece)| piece.iter().copied()).collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

unsafe fn context<'a>(ctx: *mut c_void) -> &'a mut CallContext {
    unsafe { &mut *(ctx as *mut CallContext) }
}

/// Copies `bytes` out by the length-query idiom: `needed` always, the bytes only when they fit.
unsafe fn give(bytes: &[u8], out: *mut u8, cap: usize, needed: *mut i64) -> i32 {
    if !needed.is_null() {
        unsafe { *needed = bytes.len() as i64 };
    }
    if out.is_null() || bytes.len() > cap {
        return status::BUFFER_TOO_SMALL;
    }
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, bytes.len()) };
    status::OK
}

unsafe fn text<'a>(data: *const u8, len: usize) -> Option<&'a str> {
    let bytes = unsafe { dianash_bytes { data, len }.slice() }?;
    std::str::from_utf8(bytes).ok()
}

unsafe extern "C" fn io_out(ctx: *mut c_void, data: *const u8, len: usize) -> i32 {
    write(ctx, data, len, false)
}

unsafe extern "C" fn io_err(ctx: *mut c_void, data: *const u8, len: usize) -> i32 {
    write(ctx, data, len, true)
}

fn write(ctx: *mut c_void, data: *const u8, len: usize, is_err: bool) -> i32 {
    let Some(bytes) = (unsafe { dianash_bytes { data, len }.slice() }) else { return status::NULL };
    let call = unsafe { context(ctx) };
    match call.written.last_mut() {
        Some((last_err, piece)) if *last_err == is_err => piece.extend_from_slice(bytes),
        _ => call.written.push((is_err, bytes.to_vec())),
    }
    status::OK
}

unsafe extern "C" fn io_resolve(ctx: *mut c_void, path: *const u8, len: usize, _flags: u32, out: *mut u8, cap: usize, needed: *mut i64) -> i32 {
    let Some(path) = (unsafe { text(path, len) }) else { return status::BAD_STRING };
    let call = unsafe { context(ctx) };
    match call.sandbox.resolve(path) {
        (true, real) => unsafe { give(real.as_bytes(), out, cap, needed) },
        (false, _) => {
            let words = call.sandbox.outside(path);
            let given = unsafe { give(words.as_bytes(), out, cap, needed) };
            if given == status::OK { status::OUTSIDE } else { given }
        }
    }
}

unsafe extern "C" fn io_display(ctx: *mut c_void, real: *const u8, len: usize, out: *mut u8, cap: usize, needed: *mut i64) -> i32 {
    let Some(real) = (unsafe { text(real, len) }) else { return status::BAD_STRING };
    let shown = unsafe { context(ctx) }.sandbox.display(real);
    unsafe { give(shown.as_bytes(), out, cap, needed) }
}

unsafe extern "C" fn io_cancelled(ctx: *mut c_void) -> i32 {
    let call = unsafe { context(ctx) };
    let cancelled = unsafe { &*call.cancel }.load(Ordering::Relaxed) || call.started.is_some_and(|s| s.elapsed() > call.budget);
    i32::from(cancelled)
}

/// Exchanges the per-shell state: a null `*state` reads what is kept, a non-null one keeps it, releasing what was kept before.
unsafe extern "C" fn io_slot(ctx: *mut c_void, state: *mut *mut c_void, release: *mut Option<ReleaseFn>) -> i32 {
    if state.is_null() || release.is_null() {
        return status::NULL;
    }
    let call = unsafe { context(ctx) };
    let slots = unsafe { &mut *call.slots };
    unsafe {
        if (*state).is_null() {
            match slots.get(&call.registration) {
                Some(slot) => {
                    *state = slot.state;
                    *release = slot.release;
                }
                None => *release = None,
            }
        } else {
            slots.insert(call.registration, Slot { state: *state, release: *release });
        }
    }
    status::OK
}

unsafe extern "C" fn io_environment(ctx: *mut c_void, name: *const u8, len: usize, out: *mut u8, cap: usize, needed: *mut i64) -> i32 {
    let Some(name) = (unsafe { text(name, len) }) else { return status::BAD_STRING };
    match unsafe { context(ctx) }.environment.get(name) {
        Some(value) => unsafe { give(value.as_bytes(), out, cap, needed) },
        None => status::ABSENT,
    }
}

unsafe extern "C" fn io_terminal(ctx: *mut c_void, out: *mut dianash_terminal) -> i32 {
    if out.is_null() {
        return status::NULL;
    }
    let interactive = unsafe { context(ctx) }.interactive;
    unsafe { *out = dianash_terminal { size: std::mem::size_of::<dianash_terminal>() as u32, columns: 0, rows: 0, raw_capable: u32::from(interactive) } };
    status::OK
}

/// Whom a call is for: a host's registration, or a linked module. Their per-shell slots are kept apart by their keys.
#[derive(Clone, Copy)]
enum Callee {
    Host(usize),
    Module(usize),
}

impl Callee {
    fn slot_key(self) -> usize {
        match self {
            Callee::Host(index) => index,
            Callee::Module(index) => (1 << 32) + index,
        }
    }
}

/// Builds a call over `args` for one callee and hands it to `with`, which calls the callee's function.
fn call<R>(interp: &Interp, world: &mut World, callee: Callee, command: &str, args: &[String], stdin: Option<&str>, with: impl FnOnce(*mut c_void, &dianash_call) -> R) -> (R, CallContext) {
    let session = interp.session;
    let cwd = match &world.shown_cwd {
        Some(shown) => shown.clone(),
        None => {
            let shown = world.sandbox.display(&dotnet::current_directory());
            world.shown_cwd = Some(shown.clone());
            shown
        }
    };
    let user = world.session(session).map(|s| s.user.clone()).unwrap_or_default();
    let environment = interp.exported_variables();
    let slots: *mut HashMap<usize, Slot> = match world.session_mut(session) {
        Some(s) => &mut s.slots,
        None => std::ptr::null_mut(),
    };
    let mut context = CallContext {
        written: Vec::new(),
        sandbox: world.sandbox.clone(),
        slots,
        registration: callee.slot_key(),
        cancel: &*world.cancel,
        started: interp.started(),
        budget: world.config.budget,
        environment,
        interactive: world.config.interactive,
    };
    let host_ctx = match callee {
        Callee::Host(index) => world.hosts[index].ctx,
        Callee::Module(_) => std::ptr::null_mut(),
    };
    let ctx = &mut context as *mut CallContext as *mut c_void;
    let io = dianash_io {
        size: std::mem::size_of::<dianash_io>() as u32,
        ctx,
        out: io_out,
        err: io_err,
        resolve: io_resolve,
        display: io_display,
        cancelled: io_cancelled,
        slot: io_slot,
        environment: io_environment,
        terminal: io_terminal,
    };
    let argv: Vec<dianash_bytes> = args.iter().map(|a| dianash_bytes::of(a.as_bytes())).collect();
    let services = world.services_for(session);
    let call = dianash_call {
        size: std::mem::size_of::<dianash_call>() as u32,
        command: dianash_bytes::of(command.as_bytes()),
        argv: argv.as_ptr(),
        argc: argv.len(),
        stdin_data: stdin.map_or(dianash_bytes::ABSENT, |s| dianash_bytes::of(s.as_bytes())),
        stdin_present: u32::from(stdin.is_some()),
        flags: if world.config.interactive { CALL_INTERACTIVE } else { 0 },
        cwd: dianash_bytes::of(cwd.as_bytes()),
        user: dianash_bytes::of(user.as_bytes()),
        io: &io,
        services: services.as_ptr(),
        service_count: services.len(),
    };
    let answer = with(host_ctx, &call);
    (answer, context)
}

fn new_result() -> dianash_host_result {
    dianash_host_result { size: std::mem::size_of::<dianash_host_result>() as u32, exit_status: 0, request: 0 }
}

fn outcome(context: &CallContext, result: &dianash_host_result) -> Outcome {
    Outcome { output: context.stream(false), errors: context.stream(true), status: result.exit_status, request: (result.request != 0).then_some(Request::Host(result.request)) }
}

fn failed(context: &CallContext, what: &str, code: i32) -> String {
    let words = context.output(true);
    if words.is_empty() { format!("the host's {what} failed with status {code}") } else { words }
}

/// Runs a host's command.
pub(crate) fn run(interp: &mut Interp, world: &mut World, registration: usize, command: &str, args: &[String], stdin: Option<&str>) -> Result<Outcome, String> {
    let run = world.hosts[registration].run;
    let mut result = new_result();
    let (code, context) = call(interp, world, Callee::Host(registration), command, args, stdin, |ctx, call| unsafe { run(ctx, call, &mut result as *mut dianash_host_result as *mut dianash_result) });
    if code < 0 {
        return Err(failed(&context, command, code));
    }
    Ok(outcome(&context, &result))
}

/// Whom a claim names, as a call is made to it.
fn callee(claimant: Kind) -> Callee {
    match claimant {
        Kind::Module(index) => Callee::Module(index),
        Kind::Host(registration) => Callee::Host(registration),
        Kind::Builtin(_) | Kind::Program(_) => unreachable!("only a host or a module claims jobs"),
    }
}

/// The jobs a host or a module lists for `ps`, one line each.
pub(crate) fn jobs(interp: &mut Interp, world: &mut World, claimant: Kind) -> Result<String, String> {
    let args = ["ps".to_string()];
    let code_and_context = match claimant {
        Kind::Module(index) => {
            let module = world.modules[index];
            call(interp, world, callee(claimant), "ps", &args, None, |_, call| unsafe { dianash_module::jobs_module(module, call) })
        }
        _ => {
            let Kind::Host(registration) = claimant else { return Ok(String::new()) };
            let Some(jobs) = world.hosts[registration].jobs else { return Ok(String::new()) };
            call(interp, world, callee(claimant), "ps", &args, None, |ctx, call| unsafe { jobs(ctx, call) })
        }
    };
    let (code, context) = code_and_context;
    if code < 0 {
        return Err(failed(&context, "jobs", code));
    }
    Ok(context.output(false))
}

/// A job removed by the host or module that claims its id; none when it says the id is not one of its.
pub(crate) fn kill(interp: &mut Interp, world: &mut World, claimant: Kind, id: &str) -> Result<Option<Outcome>, String> {
    let args = ["kill".to_string(), id.to_string()];
    let mut result = new_result();
    let (code, context) = match claimant {
        Kind::Module(index) => {
            let module = world.modules[index];
            call(interp, world, callee(claimant), "kill", &args, None, |_, call| unsafe {
                dianash_module::kill_module(module, call, id.as_ptr(), id.len(), &mut result as *mut dianash_host_result as *mut dianash_result)
            })
        }
        _ => {
            let Kind::Host(registration) = claimant else { return Ok(None) };
            let Some(kill) = world.hosts[registration].kill else { return Ok(None) };
            call(interp, world, callee(claimant), "kill", &args, None, |ctx, call| unsafe {
                kill(ctx, call, id.as_ptr(), id.len(), &mut result as *mut dianash_host_result as *mut dianash_result)
            })
        }
    };
    match code {
        status::ABSENT => Ok(None),
        code if code < 0 => Err(failed(&context, "kill", code)),
        _ => Ok(Some(outcome(&context, &result))),
    }
}

/// Runs a command of a module linked into the program, through the same call a module built as a library is given.
pub(crate) fn run_module(interp: &mut Interp, world: &mut World, index: usize, command: &str, args: &[String], stdin: Option<&str>) -> Result<Outcome, String> {
    let module = world.modules[index];
    let mut result = dianash_result { size: std::mem::size_of::<dianash_result>() as u32, exit_status: 0 };
    let (code, context) = call(interp, world, Callee::Module(index), command, args, stdin, |_, call| unsafe { dianash_module::run_module(module, call, &mut result) });
    if code < 0 {
        return Err(failed(&context, command, code));
    }
    Ok(Outcome { output: context.stream(false), errors: context.stream(true), status: result.exit_status, request: None })
}
