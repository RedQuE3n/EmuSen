//! The bodies of `core_exports!`, generic over the core so that dispatch stays static. Each takes the raw C
//! arguments; the safety contract of every function is the header's for its export.
#![allow(clippy::missing_safety_doc)]

use std::ffi::c_char;
use std::mem::size_of;

use super::outbox::{Inner, Outbox, Route, library_log};
use super::sys::{CreateParams, Event, FileEntry, FrameInfo, Machine, caps, event, flags, kind, status};
use super::{Core, Create, File, Settings, SettingKind, desc};
use crate::debug;
use crate::ffi::{Status, copy_text, input};

/// One machine: the core, its queues, and what the crate last saw of it to raise events on change.
pub struct Instance<T> {
    pub core: T,
    pub outbox: Outbox,
    seen: Seen,
}

#[derive(Clone, Copy, Default, PartialEq)]
struct Seen {
    width: i32,
    height: i32,
    rate: i32,
    sizes: [i64; 2],
}

const NULL: i32 = status::NULL;

unsafe fn mutable<'a, T>(m: *mut Machine) -> Option<&'a mut Instance<T>> {
    unsafe { (m as *mut Instance<T>).as_mut() }
}

unsafe fn shared<'a, T>(m: *const Machine) -> Option<&'a Instance<T>> {
    unsafe { (m as *const Instance<T>).as_ref() }
}

unsafe fn slice_mut<'a, U>(p: *mut U, len: usize) -> &'a mut [U] {
    if p.is_null() || len == 0 { &mut [] } else { unsafe { std::slice::from_raw_parts_mut(p, len) } }
}

unsafe fn slice<'a, U>(p: *const U, len: usize) -> &'a [U] {
    if p.is_null() || len == 0 { &[] } else { unsafe { std::slice::from_raw_parts(p, len) } }
}

/// Copies `values` up to `len` into `out`; their whole count.
unsafe fn copy_values<U: Copy>(values: &[U], out: *mut U, len: usize) -> i64 {
    let out = unsafe { slice_mut(out, len) };
    let n = values.len().min(out.len());
    out[..n].copy_from_slice(&values[..n]);
    values.len() as i64
}

/// The words for a code shared by every core, or `None` for one of the core's band.
pub fn shared_words(code: i32) -> Option<String> {
    let text = match code {
        0 => "success",
        status::NULL => "a null machine or buffer",
        status::TRUNCATED => "the state ended early",
        status::FOREIGN => "not this core's state",
        status::STATE_VERSION => "a state version this core does not read",
        status::BAD_STRING => "a string length the state format refuses",
        status::BUFFER_TOO_SMALL => "the buffer is too small",
        status::NOT_SUPPORTED => "not supported",
        status::NO_SUCH_SPACE => "no such memory space",
        status::READ_ONLY => "the memory space is read-only",
        status::UNKNOWN_SETTING => "a setting this core does not have",
        status::BAD_SETTING => "a setting value outside its domain",
        status::NO_SUCH_PORT => "no such controller port",
        status::BAD_FILE => "a file this core cannot use",
        status::BAD_STRUCT => "a structure smaller than its version 1.0 size",
        status::BAD_IMAGE => "an image this core cannot run",
        c if (status::FAULT_BASE - 63..=status::FAULT_BASE - 1).contains(&c) => return Some(format!("a reproduced .NET exception, kind {}", status::FAULT_BASE - c)),
        _ => return None,
    };
    Some(text.to_owned())
}

fn words<T: Core>(code: i32) -> String {
    T::status_text(code).or_else(|| shared_words(code)).unwrap_or_else(|| format!("status {code}"))
}

/// Records the outcome of a call: a failure's words as the last error, and a `LOG` event for new records.
fn settle_call<T: Core>(o: &mut Inner, r: Option<i32>) {
    let detail = o.detail.take();
    if let Some(code) = r {
        o.last_error = detail.unwrap_or_else(|| words::<T>(code));
    }
    o.announce();
}

/// A call that may change the machine: routed to its outbox, `end_call` after, its failure recorded.
fn call<T: Core, R>(inst: &mut Instance<T>, f: impl FnOnce(&mut T) -> Result<R, i32>) -> Result<R, i32> {
    let Instance { core, outbox, .. } = inst;
    let r = {
        let _route = Route::to(outbox);
        let r = f(core);
        T::end_call();
        r
    };
    settle_call::<T>(&mut outbox.borrow_mut(), r.as_ref().err().copied());
    r
}

/// A call that only reads the machine.
fn look<'a, T: Core, R>(inst: &'a Instance<T>, f: impl FnOnce(&'a T) -> Result<R, i32>) -> Result<R, i32> {
    let r = {
        let _route = Route::to(&inst.outbox);
        f(&inst.core)
    };
    settle_call::<T>(&mut inst.outbox.borrow_mut(), r.as_ref().err().copied());
    r
}

fn status_of(r: Result<(), i32>) -> i32 {
    r.map_or_else(|e| e, |()| 0)
}

fn measure<T: Core>(core: &T) -> Seen {
    let f = core.frame_info();
    let snapshot = if T::CAPABILITIES & caps::SNAPSHOT != 0 { core.snapshot_size() as i64 } else { 0 };
    Seen { width: f.width, height: f.height, rate: core.audio_rate(), sizes: [core.state_size() as i64, snapshot] }
}

/// After a call that may change what the host has read: an event for each change. State sizes are measured only
/// where they may change (§6.9).
fn observe<T: Core>(inst: &mut Instance<T>, state: bool) {
    let core = &inst.core;
    let f = core.frame_info();
    let rate = core.audio_rate();
    let mut o = inst.outbox.borrow_mut();
    if (f.width, f.height) != (inst.seen.width, inst.seen.height) {
        o.push_event(event::GEOMETRY, f.width as i64, f.height as i64);
        (inst.seen.width, inst.seen.height) = (f.width, f.height);
    }
    if rate != inst.seen.rate {
        o.push_event(event::AUDIO_RATE, rate as i64, 0);
        inst.seen.rate = rate;
    }
    if state {
        let now = measure(core).sizes;
        for (k, (&n, &was)) in now.iter().zip(&inst.seen.sizes).enumerate() {
            if n != was {
                o.push_event(event::STATE_SIZE, n, k as i64);
            }
        }
        inst.seen.sizes = now;
    }
    o.announce();
}

/// Settings text checked against the schema: every key known, every value in its domain, no key twice, and no
/// create-scope key unless `create`. With `fill`, every key absent takes its default.
pub fn resolve(schema: &[super::Setting], text: &[u8], create: bool, fill: bool) -> Result<Settings, (i32, String)> {
    let given = Settings::parse(text).map_err(|e| (e, "a settings line is not key=value UTF-8".to_owned()))?;
    let mut pairs: Vec<(String, String)> = Vec::new();
    for (key, value) in given.pairs() {
        let Some(s) = schema.iter().find(|s| s.key == key) else {
            return Err((status::UNKNOWN_SETTING, format!("no setting {key}")));
        };
        if pairs.iter().any(|(k, _)| k == key) {
            return Err((status::BAD_SETTING, format!("{key} given twice")));
        }
        if !create && s.scope == super::Scope::Create {
            return Err((status::BAD_SETTING, format!("{key} is read only when a game is loaded")));
        }
        if !s.accepts(value) {
            let domain = match &s.kind {
                SettingKind::Switch => "true or false".to_owned(),
                SettingKind::Count { min, max, .. } => format!("a whole number from {min} to {max}"),
                SettingKind::Choice(c) => c.iter().map(|c| c.value.as_str()).collect::<Vec<_>>().join(", "),
                SettingKind::Text => "one line of text".to_owned(),
            };
            return Err((status::BAD_SETTING, format!("{key}={value} is not {domain}")));
        }
        pairs.push((key.to_owned(), value.to_owned()));
    }
    if fill {
        for s in schema {
            if !pairs.iter().any(|(k, _)| *k == s.key) {
                pairs.push((s.key.clone(), s.default.clone()));
            }
        }
    }
    Ok(Settings::from_pairs(pairs))
}

/// A refusal's words into create's error buffer: at most `len - 1` bytes, cut at a character boundary, then NUL.
unsafe fn write_error(out: *mut u8, len: usize, text: &str) {
    if out.is_null() || len == 0 {
        return;
    }
    let mut n = text.len().min(len - 1);
    while !text.is_char_boundary(n) {
        n -= 1;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(text.as_ptr(), out, n);
        *out.add(n) = 0;
    }
}

// ---- A core's own extensions ---------------------------------------------------------------------------------

/// The core behind a machine handle `create` returned, for a core's own extension exports (§4.7 of
/// EmuSen_CoreAPI.md); `None` for null. No route is set, so the extension raises no events and writes no last error.
///
/// # Safety
/// `machine` must be null or a live handle this library's `create` returned for a machine of type `T`.
pub unsafe fn core_of<'a, T: Core>(machine: *const Machine) -> Option<&'a T> {
    unsafe { shared::<T>(machine) }.map(|inst| &inst.core)
}

/// [`core_of`], mutable.
///
/// # Safety
/// As [`core_of`], and no other reference to the machine may be live.
pub unsafe fn core_of_mut<'a, T: Core>(machine: *mut Machine) -> Option<&'a mut T> {
    unsafe { mutable::<T>(machine) }.map(|inst| &mut inst.core)
}

// ---- Library-level -------------------------------------------------------------------------------------------

pub unsafe fn abi_version<T: Core>() -> u32 {
    super::sys::ABI_VERSION
}

pub unsafe fn capabilities<T: Core>() -> u64 {
    T::CAPABILITIES
}

pub unsafe fn info<T: Core>(out: *mut u8, len: usize) -> i64 {
    unsafe { copy_text(&desc::info_json(&T::info(), T::CAPABILITIES), out, len) }
}

pub unsafe fn settings_schema<T: Core>(out: *mut u8, len: usize) -> i64 {
    unsafe { copy_text(&desc::settings_json(&T::settings_schema()), out, len) }
}

pub unsafe fn firmware_for<T: Core>(image: *const u8, image_len: usize, out: *mut u8, len: usize) -> i64 {
    let list = T::firmware_for(unsafe { input(image, image_len) });
    unsafe { copy_text(&desc::firmware_json(&list), out, len) }
}

pub unsafe fn status_text<T: Core>(code: i32, out: *mut u8, len: usize) -> i64 {
    match T::status_text(code).or_else(|| shared_words(code)) {
        Some(text) => unsafe { copy_text(&text, out, len) },
        None => status::NOT_SUPPORTED as i64,
    }
}

pub unsafe fn set_crash_log<T: Core>(path: *const c_char) -> i32 {
    let engine: &'static str = Box::leak(T::info().name.into_boxed_str());
    unsafe { crate::abi::install_crash_log(engine, path) }
}

pub unsafe fn log_drain<T: Core>(machine: *mut Machine, out: *mut u8, len: usize) -> i64 {
    let drain = |q: &mut super::outbox::LogQueue| if out.is_null() { q.waiting() as i64 } else { q.drain(unsafe { slice_mut(out, len) }) as i64 };
    match unsafe { mutable::<T>(machine) } {
        None => drain(&mut library_log()),
        Some(inst) => {
            let mut o = inst.outbox.borrow_mut();
            let n = drain(&mut o.log);
            if o.log.is_empty() {
                o.announced = false;
            }
            n
        }
    }
}

// ---- Lifecycle ----------------------------------------------------------------------------------------------

pub unsafe fn create<T: Core>(params: *const CreateParams, status_out: *mut i32) -> *mut Machine {
    let outbox = Outbox::default();
    let mut error: (*mut u8, usize) = (std::ptr::null_mut(), 0);
    let made = {
        let _route = Route::to(&outbox);
        let r = (|| {
            let p = unsafe { params.as_ref() }.ok_or(status::NULL)?;
            if (p.size as usize) < size_of::<CreateParams>() {
                return Err(status::BAD_STRUCT);
            }
            error = (p.error, p.error_len);
            if p.file_count > 0 && p.file_size < size_of::<FileEntry>() {
                return Err(status::BAD_STRUCT);
            }
            let mut files = Vec::with_capacity(p.file_count);
            for i in 0..p.file_count {
                // SAFETY: the host gave `file_count` elements of `file_size` bytes, each at least 1.0's.
                let f = unsafe { std::ptr::read_unaligned((p.files as *const u8).add(i * p.file_size) as *const FileEntry) };
                files.push(File { which: f.which, data: unsafe { input(f.data, f.len) } });
            }
            let settings = resolve(&T::settings_schema(), unsafe { input(p.settings, p.settings_len) }, true, true).map_err(|(code, why)| {
                super::detail(&why);
                code
            })?;
            let request = Create { image: unsafe { input(p.image, p.image_len) }, settings, files, pixel_formats: p.pixel_formats | 1, host_abi_version: p.host_abi_version };
            T::create(&request)
        })();
        T::end_call();
        r
    };
    let (machine, code) = match made {
        Ok(core) => {
            let seen = measure(&core);
            let inst = Box::new(Instance { core, outbox, seen });
            inst.outbox.borrow_mut().announce();
            (Box::into_raw(inst) as *mut Machine, 0)
        }
        Err(code) => {
            let mut o = outbox.into_inner();
            let text = o.detail.take().unwrap_or_else(|| words::<T>(code));
            unsafe { write_error(error.0, error.1, &text) };
            library_log().append(&mut o.log);
            (std::ptr::null_mut(), code)
        }
    };
    if let Some(s) = unsafe { status_out.as_mut() } {
        *s = code;
    }
    machine
}

pub unsafe fn free<T: Core>(machine: *mut Machine) -> i32 {
    if machine.is_null() {
        return NULL;
    }
    let inst = unsafe { Box::from_raw(machine as *mut Instance<T>) };
    let Instance { core, outbox, .. } = *inst;
    drop(core);
    library_log().append(&mut outbox.into_inner().log);
    0
}

pub unsafe fn reset<T: Core>(machine: *mut Machine) -> i32 {
    let Some(inst) = (unsafe { mutable::<T>(machine) }) else { return NULL };
    let r = call(inst, |c| c.reset());
    observe(inst, true);
    status_of(r)
}

pub unsafe fn machine_info<T: Core>(machine: *const Machine, out: *mut u8, len: usize) -> i64 {
    let Some(inst) = (unsafe { shared::<T>(machine) }) else { return NULL as i64 };
    let core = &inst.core;
    let measured = desc::Measured {
        space_size: &|id| core.space_size(id).ok(),
        battery_len: &|which| core.battery(which).ok().map(|(d, _)| d.len() as i64),
        capabilities: T::CAPABILITIES,
    };
    let text = desc::machine_info_json(&core.machine_info(), &measured);
    unsafe { copy_text(&text, out, len) }
}

pub unsafe fn last_error<T: Core>(machine: *const Machine, out: *mut u8, len: usize) -> i64 {
    let Some(inst) = (unsafe { shared::<T>(machine) }) else { return NULL as i64 };
    unsafe { copy_text(&inst.outbox.borrow().last_error, out, len) }
}

// ---- A frame -------------------------------------------------------------------------------------------------

pub unsafe fn advance<T: Core>(machine: *mut Machine, detail: *mut u64) -> i32 {
    let Some(inst) = (unsafe { mutable::<T>(machine) }) else { return NULL };
    let mut word = 0u64;
    let r = call(inst, |c| c.advance(&mut word));
    observe(inst, false);
    if let Some(d) = unsafe { detail.as_mut() } {
        *d = word;
    }
    status_of(r)
}

pub unsafe fn present<T: Core>(machine: *mut Machine) -> i32 {
    let Some(inst) = (unsafe { mutable::<T>(machine) }) else { return NULL };
    let r = call(inst, |c| c.present());
    observe(inst, false);
    status_of(r)
}

pub unsafe fn set_options<T: Core>(machine: *mut Machine, bits: u32) -> i32 {
    let Some(inst) = (unsafe { mutable::<T>(machine) }) else { return NULL };
    status_of(call(inst, |c| {
        c.set_options(bits & flags::OPTION_SKIP_RENDERING);
        Ok(())
    }))
}

pub unsafe fn frame_count<T: Core>(machine: *const Machine) -> i64 {
    unsafe { shared::<T>(machine) }.map_or(NULL as i64, |i| i.core.frame_count())
}

pub unsafe fn phases<T: Core>(machine: *const Machine, out: *mut i64, len: usize) -> i64 {
    let Some(inst) = (unsafe { shared::<T>(machine) }) else { return NULL as i64 };
    unsafe { copy_values(&inst.core.phases(), out, len) }
}

pub unsafe fn events<T: Core>(machine: *mut Machine, out: *mut Event, count: usize, event_size: usize) -> i64 {
    let Some(inst) = (unsafe { mutable::<T>(machine) }) else { return NULL as i64 };
    let mut o = inst.outbox.borrow_mut();
    if out.is_null() {
        return o.events.len() as i64;
    }
    if event_size < size_of::<Event>() {
        return status::BAD_STRUCT as i64;
    }
    let n = count.min(o.events.len());
    for (i, e) in o.events.drain(..n).enumerate() {
        // SAFETY: element i of `count`, `event_size` bytes each; `size` is the host's and is not written.
        unsafe {
            let at = (out as *mut u8).add(i * event_size);
            std::ptr::copy_nonoverlapping((&e as *const Event as *const u8).add(4), at.add(4), size_of::<Event>() - 4);
        }
    }
    n as i64
}

// ---- Picture -------------------------------------------------------------------------------------------------

pub unsafe fn frame_info<T: Core>(machine: *const Machine, out: *mut FrameInfo) -> i32 {
    let Some(inst) = (unsafe { shared::<T>(machine) }) else { return NULL };
    if out.is_null() {
        return NULL;
    }
    if (unsafe { std::ptr::read_unaligned(out as *const u32) } as usize) < size_of::<FrameInfo>() {
        return status::BAD_STRUCT;
    }
    let mut f = inst.core.frame_info();
    if T::CAPABILITIES & caps::FRAME_SERIAL == 0 {
        f.serial = inst.core.frame_count();
    }
    if T::CAPABILITIES & caps::ROW_REPEAT == 0 {
        f.row_repeat = 1;
    }
    // Everything after `size`, which stays the host's.
    unsafe { std::ptr::copy_nonoverlapping((&f as *const FrameInfo as *const u8).add(4), (out as *mut u8).add(4), size_of::<FrameInfo>() - 4) };
    0
}

pub unsafe fn frame_copy<T: Core>(machine: *const Machine, out: *mut u8, len: usize) -> i64 {
    let Some(inst) = (unsafe { shared::<T>(machine) }) else { return NULL as i64 };
    unsafe { copy_values(inst.core.frame(), out, len) }
}

// ---- Sound ---------------------------------------------------------------------------------------------------

pub unsafe fn audio_rate<T: Core>(machine: *const Machine) -> i32 {
    unsafe { shared::<T>(machine) }.map_or(NULL, |i| i.core.audio_rate())
}

pub unsafe fn audio_buffered<T: Core>(machine: *const Machine) -> i64 {
    unsafe { shared::<T>(machine) }.map_or(NULL as i64, |i| i.core.audio_buffered() as i64)
}

pub unsafe fn audio_drain<T: Core>(machine: *mut Machine, out: *mut i16, len: usize, max_frames: i64, rate: *mut i32) -> i64 {
    let Some(inst) = (unsafe { mutable::<T>(machine) }) else { return NULL as i64 };
    let now = inst.core.audio_rate();
    if let Some(r) = unsafe { rate.as_mut() } {
        *r = now;
    }
    if out.is_null() {
        return inst.core.audio_buffered() as i64;
    }
    if max_frames <= 0 {
        return 0;
    }
    let n = inst.core.drain_audio(unsafe { slice_mut(out, len) }, max_frames as usize);
    let after = inst.core.audio_rate();
    if after != inst.seen.rate {
        inst.outbox.borrow_mut().push_event(event::AUDIO_RATE, after as i64, 0);
        inst.seen.rate = after;
    }
    n as i64
}

pub unsafe fn set_audio_limit<T: Core>(machine: *mut Machine, samples: u64) -> i32 {
    let Some(inst) = (unsafe { mutable::<T>(machine) }) else { return NULL };
    inst.core.set_audio_limit(samples as usize);
    0
}

pub unsafe fn audio_peek<T: Core>(machine: *const Machine, out: *mut i16, len: usize) -> i64 {
    let Some(inst) = (unsafe { shared::<T>(machine) }) else { return NULL as i64 };
    inst.core.peek_audio(unsafe { slice_mut(out, len) }) as i64
}

pub unsafe fn set_mutes<T: Core>(machine: *mut Machine, mask: u32) -> i32 {
    let Some(inst) = (unsafe { mutable::<T>(machine) }) else { return NULL };
    status_of(call(inst, |c| c.set_mutes(mask)))
}

// ---- Input ---------------------------------------------------------------------------------------------------

pub unsafe fn set_buttons<T: Core>(machine: *mut Machine, port: u32, mask: u32, changed: u32) -> i32 {
    let Some(inst) = (unsafe { mutable::<T>(machine) }) else { return NULL };
    status_of(inst.core.set_buttons(port, mask, changed))
}

pub unsafe fn set_axis<T: Core>(machine: *mut Machine, port: u32, axis: u32, value: f64) -> i32 {
    let Some(inst) = (unsafe { mutable::<T>(machine) }) else { return NULL };
    status_of(call(inst, |c| c.set_axis(port, axis, value)))
}

// ---- State ---------------------------------------------------------------------------------------------------

fn kind_size<T: Core>(core: &T, k: u32) -> Result<usize, i32> {
    match k {
        kind::FULL => Ok(core.state_size()),
        kind::SNAPSHOT if T::CAPABILITIES & caps::SNAPSHOT != 0 => Ok(core.snapshot_size()),
        _ => Err(status::NOT_SUPPORTED),
    }
}

pub unsafe fn state_size<T: Core>(machine: *const Machine, k: u32) -> i64 {
    let Some(inst) = (unsafe { shared::<T>(machine) }) else { return NULL as i64 };
    look(inst, |c| kind_size(c, k)).map_or_else(|e| e as i64, |n| n as i64)
}

pub unsafe fn state_save<T: Core>(machine: *mut Machine, k: u32, out: *mut u8, len: usize) -> i64 {
    let Some(inst) = (unsafe { mutable::<T>(machine) }) else { return NULL as i64 };
    let query = out.is_null();
    let out = unsafe { slice_mut(out, len) };
    call(inst, |c| {
        let size = kind_size(c, k)?;
        if query {
            return Ok(size);
        }
        c.settle();
        match k {
            kind::FULL => c.save_state(out).map_err(|e| e.status()),
            _ => c.save_snapshot(out),
        }
    })
    .map_or_else(|e| e as i64, |n| n as i64)
}

pub unsafe fn state_load<T: Core>(machine: *mut Machine, data: *const u8, len: usize) -> i32 {
    let Some(inst) = (unsafe { mutable::<T>(machine) }) else { return NULL };
    let r = call(inst, |c| c.load_state(unsafe { input(data, len) }).map_err(|e| e.status()));
    observe(inst, true);
    status_of(r)
}

pub unsafe fn state_layout<T: Core>(machine: *const Machine, k: u32, out: *mut u8, len: usize) -> i64 {
    let Some(inst) = (unsafe { shared::<T>(machine) }) else { return NULL as i64 };
    let text = look(inst, |c| {
        kind_size(c, k)?;
        Ok(if k == kind::FULL { c.layout() } else { c.snapshot_layout() })
    });
    match text {
        Ok(t) => unsafe { copy_text(&t, out, len) },
        Err(e) => e as i64,
    }
}

// ---- Memory and battery --------------------------------------------------------------------------------------

pub unsafe fn space_size<T: Core>(machine: *const Machine, space: u32) -> i64 {
    let Some(inst) = (unsafe { shared::<T>(machine) }) else { return NULL as i64 };
    look(inst, |c| c.space_size(space)).unwrap_or_else(|e| e as i64)
}

pub unsafe fn space_read<T: Core>(machine: *mut Machine, space: u32, address: u32, out: *mut u8, len: usize) -> i64 {
    let Some(inst) = (unsafe { mutable::<T>(machine) }) else { return NULL as i64 };
    call(inst, |c| {
        c.space_size(space)?;
        if !out.is_null() {
            c.space_read(space, address, unsafe { slice_mut(out, len) })?;
        }
        Ok(len as i64)
    })
    .unwrap_or_else(|e| e as i64)
}

pub unsafe fn space_write<T: Core>(machine: *mut Machine, space: u32, address: u32, data: *const u8, len: usize) -> i64 {
    let Some(inst) = (unsafe { mutable::<T>(machine) }) else { return NULL as i64 };
    call(inst, |c| c.space_write(space, address, unsafe { input(data, len) }).map(|()| len as i64)).unwrap_or_else(|e| e as i64)
}

pub unsafe fn battery<T: Core>(machine: *const Machine, which: u32, out: *mut u8, len: usize, bits: *mut u32) -> i64 {
    let Some(inst) = (unsafe { shared::<T>(machine) }) else { return NULL as i64 };
    match look(inst, |c| c.battery(which)) {
        Ok((data, f)) => {
            if let Some(b) = unsafe { bits.as_mut() } {
                *b = f;
            }
            unsafe { copy_values(data, out, len) }
        }
        Err(e) => e as i64,
    }
}

pub unsafe fn battery_saved<T: Core>(machine: *mut Machine, which: u32) -> i32 {
    let Some(inst) = (unsafe { mutable::<T>(machine) }) else { return NULL };
    status_of(call(inst, |c| c.battery_saved(which)))
}

// ---- Cheats and settings -------------------------------------------------------------------------------------

pub unsafe fn set_rom_patches<T: Core>(machine: *mut Machine, triples: *const u32, count: usize) -> i64 {
    let Some(inst) = (unsafe { mutable::<T>(machine) }) else { return NULL as i64 };
    let words = unsafe { slice(triples, count * 3) };
    call(inst, |c| c.set_rom_patches(words)).map_or_else(|e| e as i64, |()| count as i64)
}

pub unsafe fn set_cheat_pokes<T: Core>(machine: *mut Machine, quads: *const u32, count: usize) -> i64 {
    let Some(inst) = (unsafe { mutable::<T>(machine) }) else { return NULL as i64 };
    let words = unsafe { slice(quads, count * 4) };
    call(inst, |c| c.set_cheat_pokes(words)).map_or_else(|e| e as i64, |()| count as i64)
}

pub unsafe fn set_settings<T: Core>(machine: *mut Machine, text: *const u8, len: usize) -> i32 {
    let Some(inst) = (unsafe { mutable::<T>(machine) }) else { return NULL };
    let text = unsafe { input(text, len) };
    let r = call(inst, |c| {
        let settings = resolve(&T::settings_schema(), text, false, false).map_err(|(code, why)| {
            super::detail(&why);
            code
        })?;
        c.set_settings(&settings)
    });
    observe(inst, true);
    status_of(r)
}

pub unsafe fn setting_notes<T: Core>(machine: *const Machine, out: *mut u8, len: usize) -> i64 {
    let Some(inst) = (unsafe { shared::<T>(machine) }) else { return NULL as i64 };
    unsafe { copy_text(&desc::notes_json(&inst.core.setting_notes()), out, len) }
}

// ---- Debug ---------------------------------------------------------------------------------------------------

fn hooks<'a, T: Core + 'a>(machine: *mut Machine) -> Result<&'a mut debug::Hooks, i32> {
    let inst = unsafe { mutable::<T>(machine) }.ok_or(NULL)?;
    inst.core.debug_hooks().ok_or(status::NOT_SUPPORTED)
}

pub unsafe fn debug_set<T: Core>(machine: *mut Machine, bits: u32, depth_target: i32, depth_guard: i32) -> i32 {
    hooks::<T>(machine).map_or_else(|e| e, |h| {
        h.configure(bits, depth_target, depth_guard);
        0
    })
}

pub unsafe fn debug_set_stack<T: Core>(machine: *mut Machine, pairs: *const u32, count: usize) -> i32 {
    hooks::<T>(machine).map_or_else(|e| e, |h| {
        let values = unsafe { slice(pairs, 2 * count) };
        h.set_stack(&values.chunks_exact(2).map(|p| (p[0], p[1])).collect::<Vec<_>>());
        0
    })
}

pub unsafe fn debug_set_breakpoints<T: Core>(machine: *mut Machine, processor: u32, pairs: *const i32, count: usize) -> i32 {
    let Some(inst) = (unsafe { mutable::<T>(machine) }) else { return NULL };
    let values = unsafe { slice(pairs, 2 * count) };
    status_of(call(inst, |c| c.debug_breakpoints(processor, values)))
}

pub unsafe fn debug_set_ranges<T: Core>(machine: *mut Machine, which: u32, triples: *const u32, count: usize) -> i32 {
    hooks::<T>(machine).map_or_else(|e| e, |h| {
        h.set_ranges(which, unsafe { slice(triples, 3 * count) });
        0
    })
}

pub unsafe fn debug_run_frame<T: Core>(machine: *mut Machine, bits: u32, processor: *mut u32, pc: *mut u64, detail: *mut u64) -> i32 {
    let Some(inst) = (unsafe { mutable::<T>(machine) }) else { return NULL };
    let mut word = 0u64;
    let r = call(inst, |c| c.debug_run_frame(bits, &mut word));
    observe(inst, false);
    let stopped = inst.core.debug_stopped();
    if let Some(p) = unsafe { processor.as_mut() } {
        *p = stopped;
    }
    if let Some(p) = unsafe { pc.as_mut() } {
        *p = inst.core.debug_pc(stopped).unwrap_or(0);
    }
    if let Some(d) = unsafe { detail.as_mut() } {
        *d = word;
    }
    r.map_or_else(|e| e, |why| why as i32)
}

pub unsafe fn debug_writes<T: Core>(machine: *mut Machine, out: *mut u32, len: usize) -> i64 {
    hooks::<T>(machine).map_or_else(|e| e as i64, |h| debug::drain(&mut h.writes_log, unsafe { slice_mut(out, len) }, |w: &debug::Write| [w.space, w.address, w.value, w.pc]) as i64)
}

pub unsafe fn debug_calls<T: Core>(machine: *mut Machine, out: *mut u32, len: usize) -> i64 {
    hooks::<T>(machine).map_or_else(|e| e as i64, |h| debug::drain(&mut h.calls_log, unsafe { slice_mut(out, len) }, |c: &debug::Call| [c.kind, c.source, c.target]) as i64)
}

pub unsafe fn debug_profile<T: Core>(machine: *mut Machine, out: *mut i64, len: usize) -> i64 {
    hooks::<T>(machine).map_or_else(|e| e as i64, |h| debug::drain_profile(h, unsafe { slice_mut(out, len) }) as i64)
}

pub unsafe fn debug_coverage<T: Core>(machine: *mut Machine, processor: u32, out: *mut u8, len: usize, recorded: *mut i64) -> i64 {
    hooks::<T>(machine).map_or_else(|e| e as i64, |h| {
        let mut n = 0i64;
        let r = debug::drain_coverage(h, processor as usize, unsafe { slice_mut(out, len) }, &mut n);
        if let Some(p) = unsafe { recorded.as_mut() } {
            *p = n;
        }
        r as i64
    })
}

pub unsafe fn debug_counters<T: Core>(machine: *mut Machine, out: *mut i64, len: usize) -> i64 {
    hooks::<T>(machine).map_or_else(|e| e as i64, |h| unsafe { copy_values(&h.counters(), out, len) })
}

pub unsafe fn debug_pc<T: Core>(machine: *const Machine, processor: u32, pc: *mut u64) -> i32 {
    let Some(inst) = (unsafe { shared::<T>(machine) }) else { return NULL };
    match inst.core.debug_pc(processor) {
        Some(v) => {
            if let Some(p) = unsafe { pc.as_mut() } {
                *p = v;
            }
            0
        }
        None => status::NOT_SUPPORTED,
    }
}

pub unsafe fn debug_registers<T: Core>(machine: *const Machine, processor: u32, out: *mut i64, len: usize) -> i64 {
    let Some(inst) = (unsafe { shared::<T>(machine) }) else { return NULL as i64 };
    match look(inst, |c| c.debug_registers(processor)) {
        Ok(values) => unsafe { copy_values(&values, out, len) },
        Err(e) => e as i64,
    }
}

pub unsafe fn debug_disassemble<T: Core>(machine: *mut Machine, processor: u32, space: u32, address: u32, count: u32, out: *mut u8, len: usize) -> i64 {
    let Some(inst) = (unsafe { mutable::<T>(machine) }) else { return NULL as i64 };
    match call(inst, |c| c.debug_disassemble(processor, space, address, count)) {
        Ok(list) => unsafe { copy_text(&desc::disassembly_json(&list), out, len) },
        Err(e) => e as i64,
    }
}
