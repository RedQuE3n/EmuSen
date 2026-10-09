//! Endymion's device half in C: SDL lent by the host, simulated pads and their sets, the pads' bookkeeping, the
//! game's audio player and the interface's sounds.
//!
//! A simulated pad, a set, a pads handle and an audio player may be used from any thread; each is behind its own
//! lock, as the C# reads a manager and submits samples on the emulation thread while the window polls the one and
//! sets the volume of the other. An interface player is used by one thread at a time. Freeing a pads handle or a player that was never disposed lets its device go to the process's
//! end, as the C# did; disposing lets it go now. See EmuSen_RustPlatform.md §14.

use crate::endymion::{Rate, hand_over, take};
use crate::{fail, put, required, status, text};
use emusen_endymion::audio::{self, AudioPlayer, UiSounds};
use emusen_endymion::devices::{self, PadDevices, SdlPads, SharedPad, SharedSet, SimulatedPad, SimulatedPads, lock, lock_set};
use emusen_endymion::pads::{self, PadEvent, Pads};
use emusen_endymion::sdl::Sdl;
use std::cell::RefCell;
use std::collections::HashSet;
use std::ffi::c_void;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

static SDL: OnceLock<Arc<Sdl>> = OnceLock::new();

fn sdl() -> Result<Arc<Sdl>, i32> {
    SDL.get().cloned().ok_or_else(|| fail(status::NOT_SUPPORTED, "SDL3 has not been lent to the library"))
}

/// SDL3 as the host has it loaded, by its loader's handle; the first lent is kept, and the library never closes it.
///
/// # Safety
/// `handle` must be a live handle to SDL3 from the platform's loader, kept loaded for the process.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_sdl_lend(handle: *mut c_void) -> i32 {
    if SDL.get().is_some() {
        return 0;
    }
    match unsafe { Sdl::lent(handle) } {
        Ok(lent) => {
            let _ = SDL.set(Arc::new(lent));
            0
        }
        Err(words) => fail(if handle.is_null() { status::NULL } else { status::NOT_SUPPORTED }, words),
    }
}

/// A hint as the lent SDL has it: how a test sees that both sides share one SDL.
///
/// # Safety
/// `name` valid for `len` bytes; `out` for `cap`, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_sdl_hint(name: *const u8, len: usize, out: *mut u8, cap: usize) -> i64 {
    let name = match unsafe { required(name, len) } {
        Ok(name) => name,
        Err(status) => return status as i64,
    };
    let sdl = match sdl() {
        Ok(sdl) => sdl,
        Err(status) => return status as i64,
    };
    match sdl.hint(name) {
        Some(value) => unsafe { put(value.as_bytes(), out, cap) },
        None => status::ABSENT as i64,
    }
}

/// An optional text out: ABSENT for none.
unsafe fn put_text(value: Option<String>, out: *mut u8, cap: usize) -> i64 {
    match value {
        Some(value) => unsafe { put(value.as_bytes(), out, cap) },
        None => status::ABSENT as i64,
    }
}

// Simulated pads.

pub type SimPad = SharedPad;

macro_rules! get {
    ($h:expr) => {
        match unsafe { $h.as_ref() } {
            Some(held) => held,
            None => return fail(status::NULL, "no handle").into(),
        }
    };
}

/// # Safety
/// `path` valid for `len` bytes, or null for none.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_sim_pad_new(path: *const u8, len: usize) -> *mut SimPad {
    match unsafe { text(path, len) } {
        Ok(path) => Box::into_raw(Box::new(Arc::new(Mutex::new(SimulatedPad::new(path.map(str::to_string)))))),
        Err(_) => std::ptr::null_mut(),
    }
}

/// # Safety
/// `pad` from `emusen_endymion_sim_pad_new`, not freed, or null; a set it is plugged into keeps its own reference.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_sim_pad_free(pad: *mut SimPad) -> i32 {
    if !pad.is_null() {
        drop(unsafe { Box::from_raw(pad) });
    }
    0
}

pub const PAD_NAME: u32 = 0;
pub const PAD_GUID: u32 = 1;
pub const PAD_PATH: u32 = 2;

/// Sets the name (null is refused), the GUID or the path (null for none).
///
/// # Safety
/// `pad` live; `value` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_sim_pad_set_text(pad: *const SimPad, which: u32, value: *const u8, len: usize) -> i32 {
    let pad = get!(pad);
    let value = match unsafe { text(value, len) } {
        Ok(value) => value.map(str::to_string),
        Err(status) => return status,
    };
    let mut pad = lock(pad);
    match which {
        PAD_NAME => match value {
            Some(name) => pad.name = name,
            None => return fail(status::NULL, "a pad's name is never null"),
        },
        PAD_GUID => pad.guid = value,
        PAD_PATH => pad.path = value,
        _ => return fail(status::BAD_ARGUMENT, "no such text"),
    }
    0
}

/// # Safety
/// `pad` live; `out` valid for `cap` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_sim_pad_text(pad: *const SimPad, which: u32, out: *mut u8, cap: usize) -> i64 {
    let pad = lock(get!(pad));
    let value = match which {
        PAD_NAME => Some(pad.name.clone()),
        PAD_GUID => pad.guid.clone(),
        PAD_PATH => pad.path.clone(),
        _ => return fail(status::BAD_ARGUMENT, "no such text") as i64,
    };
    unsafe { put_text(value, out, cap) }
}

pub const PAD_PLAYER_INDEX: u32 = 0;
pub const PAD_KIND: u32 = 1;

/// # Safety
/// `pad` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_sim_pad_set(pad: *const SimPad, which: u32, value: i32) -> i32 {
    let mut pad = lock(get!(pad));
    match which {
        PAD_PLAYER_INDEX => pad.player_index = value,
        PAD_KIND => pad.kind = value,
        _ => return fail(status::BAD_ARGUMENT, "no such number"),
    }
    0
}

/// The player index or the kind.
///
/// # Safety
/// `pad` live; `out` valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_sim_pad_get(pad: *const SimPad, which: u32, out: *mut i32) -> i32 {
    let pad = lock(get!(pad));
    let value = match which {
        PAD_PLAYER_INDEX => pad.player_index,
        PAD_KIND => pad.kind,
        _ => return fail(status::BAD_ARGUMENT, "no such number"),
    };
    match unsafe { out.as_mut() } {
        Some(out) => {
            *out = value;
            0
        }
        None => fail(status::NULL, "no output"),
    }
}

/// Presses (`down` 1) or releases (0) an SDL button; any other `down` lets go of every button and axis.
///
/// # Safety
/// `pad` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_sim_pad_press(pad: *const SimPad, button: i32, down: u32) -> i32 {
    let mut pad = lock(get!(pad));
    match down {
        1 => pad.press(button),
        0 => pad.release(button),
        _ => pad.release_all(),
    }
    0
}

/// -1 to 1 for a stick, 0 to 1 for a trigger, kept as SDL's sixteen bits.
///
/// # Safety
/// `pad` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_sim_pad_set_axis(pad: *const SimPad, axis: i32, value: f64) -> i32 {
    lock(get!(pad)).set_axis(axis, value);
    0
}

/// 1 when the button is held, 0 when not.
///
/// # Safety
/// `pad` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_sim_pad_held(pad: *const SimPad, button: i32) -> i32 {
    lock(get!(pad)).is_held(button) as i32
}

/// The axis as SDL's sixteen bits.
///
/// # Safety
/// `pad` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_sim_pad_axis(pad: *const SimPad, axis: i32) -> i32 {
    lock(get!(pad)).axis(axis) as i32
}

// Sets of simulated pads.

pub type SimSet = SharedSet;

#[unsafe(no_mangle)]
pub extern "C" fn emusen_endymion_sim_set_new() -> *mut SimSet {
    Box::into_raw(Box::new(Arc::new(Mutex::new(SimulatedPads::new()))))
}

/// # Safety
/// `set` from `emusen_endymion_sim_set_new`, not freed, or null; a pads handle using it keeps its own reference.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_sim_set_free(set: *mut SimSet) -> i32 {
    if !set.is_null() {
        drop(unsafe { Box::from_raw(set) });
    }
    0
}

/// Plugs a pad in; its id.
///
/// # Safety
/// `set` and `pad` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_sim_set_connect(set: *const SimSet, pad: *const SimPad) -> i64 {
    let pad = get!(pad).clone();
    lock_set(get!(set)).connect(pad) as i64
}

/// Pulls a pad out from under every id it was plugged in as.
///
/// # Safety
/// `set` and `pad` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_sim_set_disconnect(set: *const SimSet, pad: *const SimPad) -> i32 {
    let pad = get!(pad);
    lock_set(get!(set)).disconnect(pad);
    0
}

pub const SET_FIRST: u32 = 0;
pub const SET_OPEN_HANDLES: u32 = 1;
pub const SET_OPENS: u32 = 2;
pub const SET_CLOSES: u32 = 3;
pub const SET_INITIALIZED: u32 = 4;

/// The first attached pad's id (0 for none), the handles open, the opens and closes made, or whether it is initialised.
///
/// # Safety
/// `set` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_sim_set_get(set: *const SimSet, which: u32) -> i64 {
    let set = lock_set(get!(set));
    match which {
        SET_FIRST => set.attached_ids().first().copied().unwrap_or(0) as i64,
        SET_OPEN_HANDLES => set.open_handles() as i64,
        SET_OPENS => set.opens as i64,
        SET_CLOSES => set.closes as i64,
        SET_INITIALIZED => set.initialized as i64,
        _ => fail(status::BAD_ARGUMENT, "no such count") as i64,
    }
}

/// The attached ids, in order; the length-query idiom.
///
/// # Safety
/// `set` live; `out` valid for `cap` ids, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_sim_set_attached(set: *const SimSet, out: *mut u32, cap: usize) -> i64 {
    let ids = lock_set(get!(set)).attached_ids();
    if !out.is_null() && ids.len() <= cap {
        unsafe { std::ptr::copy_nonoverlapping(ids.as_ptr(), out, ids.len()) };
    }
    ids.len() as i64
}

pub const DEVICE_INIT: u32 = 0;
pub const DEVICE_QUIT: u32 = 1;
pub const DEVICE_OPEN: u32 = 2;
pub const DEVICE_CLOSE: u32 = 3;
pub const DEVICE_IS_ATTACHED: u32 = 4;
pub const DEVICE_CHANGED: u32 = 5;
pub const DEVICE_UPDATE: u32 = 6;
pub const DEVICE_BUTTON: u32 = 7;
pub const DEVICE_AXIS: u32 = 8;
pub const DEVICE_KIND: u32 = 9;
pub const DEVICE_LABEL: u32 = 10;
pub const DEVICE_SET_PLAYER_INDEX: u32 = 11;

/// One of the set's device calls, as a manager makes it: `handle` is the opened pad's, `arg` the id, button, axis or index.
///
/// # Safety
/// `set` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_sim_set_call(set: *const SimSet, op: u32, handle: u64, arg: i32) -> i64 {
    let mut set = lock_set(get!(set));
    let pad = handle as devices::Handle;
    match op {
        DEVICE_INIT => set.init() as i64,
        DEVICE_QUIT => {
            set.quit();
            0
        }
        DEVICE_OPEN => set.open(arg as u32) as i64,
        DEVICE_CLOSE => {
            set.close(pad);
            0
        }
        DEVICE_IS_ATTACHED => set.is_attached(pad) as i64,
        DEVICE_CHANGED => set.devices_changed() as i64,
        DEVICE_UPDATE => 0,
        DEVICE_BUTTON => set.button(pad, arg) as i64,
        DEVICE_AXIS => set.axis(pad, arg) as i64,
        DEVICE_KIND => set.kind(pad) as i64,
        DEVICE_LABEL => set.label(pad, arg) as i64,
        DEVICE_SET_PLAYER_INDEX => {
            set.set_player_index(pad, arg);
            0
        }
        _ => fail(status::BAD_ARGUMENT, "no such device call") as i64,
    }
}

pub const DEVICE_NAME: u32 = 0;
pub const DEVICE_GUID: u32 = 1;
pub const DEVICE_PATH: u32 = 2;

/// An opened pad's name, GUID or path as the set reports it; ABSENT for none.
///
/// # Safety
/// `set` live; `out` valid for `cap` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_sim_set_text(set: *const SimSet, op: u32, handle: u64, out: *mut u8, cap: usize) -> i64 {
    let mut set = lock_set(get!(set));
    let pad = handle as devices::Handle;
    let value = match op {
        DEVICE_NAME => set.name(pad),
        DEVICE_GUID => Some(set.guid(pad)),
        DEVICE_PATH => set.path(pad),
        _ => return fail(status::BAD_ARGUMENT, "no such text") as i64,
    };
    unsafe { put_text(value, out, cap) }
}

// The pads.

/// A pad opened or closed, as `emusen_endymion_pad_event`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PadEventC {
    /// 0 opened, 1 closed.
    pub kind: u32,
    pub announce: u32,
    pub key: u64,
}

impl From<PadEvent> for PadEventC {
    fn from(event: PadEvent) -> PadEventC {
        match event {
            PadEvent::Opened { key, announce } => PadEventC { kind: 0, announce: announce as u32, key },
            PadEvent::Closed { key } => PadEventC { kind: 1, announce: 0, key },
        }
    }
}

pub struct PadsHandle {
    inner: Mutex<(Pads, Vec<PadEventC>)>,
}

fn locked(h: &PadsHandle) -> MutexGuard<'_, (Pads, Vec<PadEventC>)> {
    h.inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub const DEVICES_SDL: u32 = 0;
pub const DEVICES_SIMULATED: u32 = 1;

/// SDL's pads (all, or only `only`'s ids when it is not null) or a simulated set's.
///
/// # Safety
/// `set` live when `kind` is simulated; `only` valid for `only_len` ids, or null.
unsafe fn devices_of(kind: u32, set: *const SimSet, only: *const u32, only_len: usize) -> Result<Box<dyn PadDevices>, i32> {
    match kind {
        DEVICES_SDL => {
            let mut pads = SdlPads::new(sdl()?);
            if !only.is_null() {
                let ids = if only_len == 0 { &[][..] } else { unsafe { std::slice::from_raw_parts(only, only_len) } };
                pads.only = Some(ids.iter().copied().collect::<HashSet<u32>>());
            }
            Ok(Box::new(pads))
        }
        DEVICES_SIMULATED => match unsafe { set.as_ref() } {
            Some(set) => Ok(Box::new(set.clone())),
            None => Err(fail(status::NULL, "no simulated set")),
        },
        _ => Err(fail(status::BAD_ARGUMENT, "no such devices")),
    }
}

/// A manager's pads on the devices given, not yet started.
///
/// # Safety
/// As `devices_of`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_pads_new(kind: u32, set: *const SimSet, only: *const u32, only_len: usize) -> *mut PadsHandle {
    match unsafe { devices_of(kind, set, only, only_len) } {
        Ok(devices) => Box::into_raw(Box::new(PadsHandle { inner: Mutex::new((Pads::new(devices), Vec::new())) })),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Frees the handle; a pads handle not closed first keeps its devices open, as a C# manager never disposed did.
///
/// # Safety
/// `h` from `emusen_endymion_pads_new`, not freed, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_pads_free(h: *mut PadsHandle) -> i32 {
    if !h.is_null() {
        let handle = unsafe { Box::from_raw(h) };
        let (pads, _) = handle.inner.into_inner().unwrap_or_else(|poisoned| poisoned.into_inner());
        pads.abandon();
    }
    0
}

/// The events made, the length-query idiom over the handle's pending list.
unsafe fn events_out(pending: &mut Vec<PadEventC>, made: Vec<PadEvent>, out: *mut PadEventC, cap: usize) -> i64 {
    unsafe { hand_over(pending, made.into_iter().map(PadEventC::from).collect(), out, cap) }
}

/// # Safety
/// `h` live; `out` valid for `cap` events, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_pads_start(h: *const PadsHandle, out: *mut PadEventC, cap: usize) -> i64 {
    let mut guard = locked(get!(h));
    let (pads, pending) = &mut *guard;
    let made = pads.start();
    unsafe { events_out(pending, made, out, cap) }
}

/// The pads closed and opened since the last poll; ABSENT while the devices have not started, when the C# poll does nothing.
///
/// # Safety
/// `h` live; `out` valid for `cap` events, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_pads_poll(h: *const PadsHandle, out: *mut PadEventC, cap: usize) -> i64 {
    let mut guard = locked(get!(h));
    let (pads, pending) = &mut *guard;
    if !pads.initialized() {
        return status::ABSENT as i64;
    }
    let made = pads.poll();
    unsafe { events_out(pending, made, out, cap) }
}

/// Shows the devices' pads of these ids alone from the next start or poll, or all of them for a null `only`; a test's, so that it opens SDL's virtual pads and no pad on the desk.
///
/// # Safety
/// `h` live; `only` valid for `only_len` ids, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_pads_show_only(h: *const PadsHandle, only: *const u32, only_len: usize) -> i32 {
    let ids = if only.is_null() {
        None
    } else if only_len == 0 {
        Some(HashSet::new())
    } else {
        Some(unsafe { std::slice::from_raw_parts(only, only_len) }.iter().copied().collect())
    };
    locked(get!(h)).0.show_only(ids);
    0
}

/// The devices brought up to date, after the host has seated a poll's pads.
///
/// # Safety
/// `h` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_pads_update(h: *const PadsHandle) -> i32 {
    locked(get!(h)).0.update();
    0
}

/// Lets go of every pad and starts on other devices.
///
/// # Safety
/// `h` live; the devices as `devices_of`; `out` valid for `cap` events, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_pads_use(h: *const PadsHandle, kind: u32, set: *const SimSet, only: *const u32, only_len: usize, out: *mut PadEventC, cap: usize) -> i64 {
    let handle = get!(h);
    let devices = match unsafe { devices_of(kind, set, only, only_len) } {
        Ok(devices) => devices,
        Err(status) => return status as i64,
    };
    let mut guard = locked(handle);
    let (pads, pending) = &mut *guard;
    let made = pads.use_devices(devices);
    unsafe { events_out(pending, made, out, cap) }
}

/// # Safety
/// `h` live; `out` valid for `cap` events, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_pads_take(h: *const PadsHandle, out: *mut PadEventC, cap: usize) -> i64 {
    let mut guard = locked(get!(h));
    unsafe { take(&mut guard.1, out, cap) }
}

/// Lets go of every pad and the devices now.
///
/// # Safety
/// `h` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_pads_close_all(h: *const PadsHandle) -> i32 {
    locked(get!(h)).0.close_all();
    0
}

/// The open pads' keys in the order they were opened; the length-query idiom.
///
/// # Safety
/// `h` live; `out` valid for `cap` keys, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_pads_open(h: *const PadsHandle, out: *mut u64, cap: usize) -> i64 {
    let guard = locked(get!(h));
    let keys = guard.0.open_pads();
    if !out.is_null() && keys.len() <= cap {
        unsafe { std::ptr::copy_nonoverlapping(keys.as_ptr(), out, keys.len()) };
    }
    keys.len() as i64
}

pub const PADS_ID: u32 = 0;
pub const PADS_IS_OPEN: u32 = 1;
pub const PADS_KIND: u32 = 2;
pub const PADS_RAW_PRESSED: u32 = 3;
pub const PADS_AXIS_VALUE: u32 = 4;
pub const PADS_LABEL: u32 = 5;
pub const PADS_FRONTEND_COUNT: u32 = 6;
pub const PADS_STARTED: u32 = 7;
pub const PADS_ANY_PRESSED: u32 = 8;
pub const PADS_LAST_RESCAN: u32 = 9;

/// A number about a pad (`key`, with `arg` its button or axis) or about them all: the label is 0 for none, and the
/// button held first is -1 for none.
///
/// # Safety
/// `h` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_pads_get(h: *const PadsHandle, key: u64, which: u32, arg: i32) -> i64 {
    let mut guard = locked(get!(h));
    let pads = &mut guard.0;
    match which {
        PADS_ID => pads.pad(key).map_or(fail(status::ABSENT, "no such pad") as i64, |pad| pad.id as i64),
        PADS_IS_OPEN => pads.is_open(key) as i64,
        PADS_KIND => pads.kind(key) as i64,
        PADS_RAW_PRESSED => pads.raw_pressed(key, arg) as i64,
        PADS_AXIS_VALUE => pads.axis_value(key, arg) as i64,
        PADS_LABEL => pads.label(key, arg).unwrap_or(0) as i64,
        PADS_FRONTEND_COUNT => pads.frontend_pad_count() as i64,
        PADS_STARTED => pads.started() as i64,
        PADS_ANY_PRESSED => pads.any_pressed().unwrap_or(-1) as i64,
        PADS_LAST_RESCAN => pads.last_rescan(),
        _ => fail(status::BAD_ARGUMENT, "no such number") as i64,
    }
}

pub const PADS_GUID: u32 = 0;
pub const PADS_PATH: u32 = 1;
pub const PADS_NAME: u32 = 2;

/// A pad's GUID, path (ABSENT for none) or name.
///
/// # Safety
/// `h` live; `out` valid for `cap` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_pads_text(h: *const PadsHandle, key: u64, which: u32, out: *mut u8, cap: usize) -> i64 {
    let mut guard = locked(get!(h));
    let pads = &mut guard.0;
    let value = match which {
        PADS_GUID => pads.pad(key).map(|pad| pad.guid.clone()),
        PADS_PATH => pads.pad(key).and_then(|pad| pad.path.clone()),
        PADS_NAME => Some(pads.name(key)),
        _ => return fail(status::BAD_ARGUMENT, "no such text") as i64,
    };
    unsafe { put_text(value, out, cap) }
}

pub const AXIS_RAW: u32 = 0;
pub const AXIS_PAD: u32 = 1;

/// A pad's axis: raw by SDL's number (-1 to 1), or the pad's axis through the deadzone.
///
/// # Safety
/// `h` live; `out` valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_pads_axis(h: *const PadsHandle, key: u64, which: u32, axis: u32, out: *mut f64) -> i32 {
    let mut guard = locked(get!(h));
    let pads = &mut guard.0;
    let value = match which {
        AXIS_RAW => pads.raw_axis(key, axis as i32),
        AXIS_PAD => pads.axis(key, axis),
        _ => return fail(status::BAD_ARGUMENT, "no such axis reading"),
    };
    match unsafe { out.as_mut() } {
        Some(out) => {
            *out = value;
            0
        }
        None => fail(status::NULL, "no output"),
    }
}

/// A pad's `PadButton` as the game hears it, `bound` its player's SDL button or -1 for none: 1 pressed, 0 not.
///
/// # Safety
/// `h` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_pads_pressed(h: *const PadsHandle, key: u64, button: u32, bound: i32) -> i32 {
    locked(get!(h)).0.is_pressed(key, button, (bound >= 0).then_some(bound)) as i32
}

/// # Safety
/// `h` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_pads_set_player_index(h: *const PadsHandle, key: u64, index: i32) -> i32 {
    locked(get!(h)).0.set_player_index(key, index);
    0
}

pub const SETTING_FIRST_ONLY: u32 = 0;
pub const SETTING_STICK_AS_DPAD: u32 = 1;
pub const SETTING_STICK_DEADZONE: u32 = 2;
pub const SETTING_LEFT_STICK_ANALOG: u32 = 3;
pub const SETTING_ANALOG_DEADZONE: u32 = 4;

/// Sets a reading rule's setting; a flag is nonzero for true.
///
/// # Safety
/// `h` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_pads_set_setting(h: *const PadsHandle, which: u32, value: f64) -> i32 {
    let mut guard = locked(get!(h));
    let s = &mut guard.0.settings;
    match which {
        SETTING_FIRST_ONLY => s.first_controller_only = value != 0.0,
        SETTING_STICK_AS_DPAD => s.analog_stick_as_dpad = value != 0.0,
        SETTING_STICK_DEADZONE => s.stick_deadzone = value,
        SETTING_LEFT_STICK_ANALOG => s.left_stick_is_analog = value != 0.0,
        SETTING_ANALOG_DEADZONE => s.analog_deadzone = value,
        _ => return fail(status::BAD_ARGUMENT, "no such setting"),
    }
    0
}

/// # Safety
/// `h` live; `out` valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_pads_setting(h: *const PadsHandle, which: u32, out: *mut f64) -> i32 {
    let s = locked(get!(h)).0.settings;
    let flag = |b: bool| if b { 1.0 } else { 0.0 };
    let value = match which {
        SETTING_FIRST_ONLY => flag(s.first_controller_only),
        SETTING_STICK_AS_DPAD => flag(s.analog_stick_as_dpad),
        SETTING_STICK_DEADZONE => s.stick_deadzone,
        SETTING_LEFT_STICK_ANALOG => flag(s.left_stick_is_analog),
        SETTING_ANALOG_DEADZONE => s.analog_deadzone,
        _ => return fail(status::BAD_ARGUMENT, "no such setting"),
    };
    match unsafe { out.as_mut() } {
        Some(out) => {
            *out = value;
            0
        }
        None => fail(status::NULL, "no output"),
    }
}

/// Whether a rescan is due at `now` after one at `last`, in .NET ticks.
#[unsafe(no_mangle)]
pub extern "C" fn emusen_endymion_pads_rescan_due(now: i64, last: i64) -> i32 {
    pads::rescan_due(now, last) as i32
}

// The game's audio player.

pub struct Audio {
    player: Mutex<AudioPlayer>,
}

fn audio(h: &Audio) -> MutexGuard<'_, AudioPlayer> {
    h.player.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Opens SDL's audio and the default device at `sample_rate`, steering `rate`, which the host keeps and lends to each submit.
///
/// # Safety
/// `rate` from `emusen_endymion_rate_new`, live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_audio_new(rate: *mut Rate, sample_rate: i32, target_latency_ms: i32, buffer_frames: i32) -> *mut Audio {
    let Some(rate) = (unsafe { rate.as_ref() }) else {
        fail(status::NULL, "no rate control");
        return std::ptr::null_mut();
    };
    match sdl() {
        Ok(sdl) => Box::into_raw(Box::new(Audio { player: Mutex::new(AudioPlayer::new(sdl, &mut rate.lock().value, sample_rate, target_latency_ms, buffer_frames)) })),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Frees the handle; a player not disposed first keeps its device open, as a C# player never disposed did.
///
/// # Safety
/// `h` from `emusen_endymion_audio_new`, not freed, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_audio_free(h: *mut Audio) -> i32 {
    if !h.is_null() {
        unsafe { Box::from_raw(h) }.player.into_inner().unwrap_or_else(|poisoned| poisoned.into_inner()).abandon();
    }
    0
}

/// Closes the device and SDL's audio; again closes nothing more, but lets go of SDL's audio again, as the C#'s `Dispose` does.
///
/// # Safety
/// `h` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_audio_dispose(h: *mut Audio) -> i32 {
    audio(get!(h)).dispose();
    0
}

/// Samples to the device through `rate`; a null `samples` is the C#'s null, and does nothing.
///
/// # Safety
/// `h` and `rate` live; `samples` valid for `len` samples, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_audio_submit(h: *mut Audio, rate: *mut Rate, samples: *const i16, len: usize, sample_rate: i32) -> i32 {
    let (Some(handle), Some(rate)) = (unsafe { h.as_ref() }, unsafe { rate.as_ref() }) else {
        return fail(status::NULL, "no handle");
    };
    let samples = if samples.is_null() { None } else if len == 0 { Some(&[][..]) } else { Some(unsafe { std::slice::from_raw_parts(samples, len) }) };
    // The player's lock, then the rate control's: the one order in which the two are ever held together.
    let mut player = audio(handle);
    match player.submit(&mut rate.lock().value, samples, sample_rate) {
        Ok(()) => 0,
        Err(error) => fail(status::BAD_ARGUMENT, error.words()),
    }
}

pub const AUDIO_QUEUED_FRAMES: u32 = 0;
pub const AUDIO_AVAILABLE: u32 = 1;
pub const AUDIO_SAMPLE_RATE: u32 = 2;

/// # Safety
/// `h` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_audio_get(h: *const Audio, which: u32) -> i64 {
    let player = audio(get!(h));
    match which {
        AUDIO_QUEUED_FRAMES => player.queued_frames() as i64,
        AUDIO_AVAILABLE => player.is_available() as i64,
        AUDIO_SAMPLE_RATE => player.sample_rate() as i64,
        _ => fail(status::BAD_ARGUMENT, "no such number") as i64,
    }
}

/// # Safety
/// `h` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_audio_set_volume(h: *mut Audio, volume: f32) -> i32 {
    audio(get!(h)).set_volume(volume);
    0
}

/// # Safety
/// `h` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_audio_volume(h: *const Audio) -> f32 {
    unsafe { h.as_ref() }.map_or(f32::NAN, |handle| audio(handle).volume())
}

// The interface's sounds.

pub struct Ui {
    sounds: UiSounds,
}

#[unsafe(no_mangle)]
pub extern "C" fn emusen_endymion_ui_new() -> *mut Ui {
    match sdl() {
        Ok(sdl) => Box::into_raw(Box::new(Ui { sounds: UiSounds::new(sdl) })),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Frees the handle; a player not disposed first keeps its stream, as a C# player never disposed did.
///
/// # Safety
/// `h` from `emusen_endymion_ui_new`, not freed, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_ui_free(h: *mut Ui) -> i32 {
    if !h.is_null() {
        unsafe { Box::from_raw(h) }.sounds.abandon();
    }
    0
}

/// # Safety
/// `h` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_ui_dispose(h: *mut Ui) -> i32 {
    match unsafe { h.as_mut() } {
        Some(ui) => {
            ui.sounds.dispose();
            0
        }
        None => fail(status::NULL, "no handle"),
    }
}

pub const UI_PLAY: u32 = 0;
pub const UI_PRELOAD: u32 = 1;

/// Plays, or only decodes and keeps, the sound a path or a remembered name gives.
///
/// # Safety
/// `h` live; `key` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_ui_sound(h: *mut Ui, which: u32, key: *const u8, len: usize) -> i32 {
    let Some(ui) = (unsafe { h.as_mut() }) else { return fail(status::NULL, "no handle") };
    let key = match unsafe { required(key, len) } {
        Ok(key) => key,
        Err(status) => return status,
    };
    match which {
        UI_PLAY => ui.sounds.play(key),
        UI_PRELOAD => ui.sounds.preload(key),
        _ => return fail(status::BAD_ARGUMENT, "no such use of a sound"),
    }
    0
}

/// Keeps samples already in the interface's format under a name.
///
/// # Safety
/// `h` live; `key` valid for `len` bytes; `samples` for `samples_len`, or null when it is 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_ui_remember(h: *mut Ui, key: *const u8, len: usize, samples: *const u8, samples_len: usize) -> i32 {
    let Some(ui) = (unsafe { h.as_mut() }) else { return fail(status::NULL, "no handle") };
    let key = match unsafe { required(key, len) } {
        Ok(key) => key,
        Err(status) => return status,
    };
    let bytes = if samples.is_null() || samples_len == 0 { Vec::new() } else { unsafe { std::slice::from_raw_parts(samples, samples_len) }.to_vec() };
    ui.sounds.remember(key, bytes);
    0
}

pub const UI_IS_OPEN: u32 = 0;
pub const UI_QUEUED: u32 = 1;

/// # Safety
/// `h` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_ui_get(h: *const Ui, which: u32) -> i64 {
    let ui = get!(h);
    match which {
        UI_IS_OPEN => ui.sounds.is_open() as i64,
        UI_QUEUED => ui.sounds.queued() as i64,
        _ => fail(status::BAD_ARGUMENT, "no such number") as i64,
    }
}

/// # Safety
/// `h` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_ui_set_volume(h: *mut Ui, volume: f32) -> i32 {
    match unsafe { h.as_mut() } {
        Some(ui) => {
            ui.sounds.set_volume(volume);
            0
        }
        None => fail(status::NULL, "no handle"),
    }
}

/// # Safety
/// `h` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_ui_volume(h: *const Ui) -> f32 {
    unsafe { h.as_ref() }.map_or(f32::NAN, |ui| ui.sounds.volume())
}

thread_local! {
    static DECODED: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// A WAV file in the interface's format: ABSENT when SDL cannot read it, else its length, the bytes copied when they fit
/// and otherwise kept on this thread for `emusen_endymion_ui_decode_take`.
///
/// # Safety
/// `path` valid for `len` bytes; `out` for `cap`, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_ui_decode(path: *const u8, len: usize, out: *mut u8, cap: usize) -> i64 {
    let path = match unsafe { required(path, len) } {
        Ok(path) => path,
        Err(status) => return status as i64,
    };
    let sdl = match sdl() {
        Ok(sdl) => sdl,
        Err(status) => return status as i64,
    };
    match audio::decode(&sdl, path) {
        Some(bytes) => DECODED.with_borrow_mut(|pending| unsafe { hand_over(pending, bytes, out, cap) }),
        None => status::ABSENT as i64,
    }
}

/// # Safety
/// `out` valid for `cap` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_ui_decode_take(out: *mut u8, cap: usize) -> i64 {
    DECODED.with_borrow_mut(|pending| unsafe { take(pending, out, cap) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_simulated_set_seen_through_the_c_layer_opens_reads_and_closes_its_pads() {
        let pad = unsafe { emusen_endymion_sim_pad_new(b"/dev/input/x".as_ptr(), 12) };
        unsafe { emusen_endymion_sim_pad_set_text(pad, PAD_NAME, b"Pad".as_ptr(), 3) };
        let set = emusen_endymion_sim_set_new();
        unsafe { emusen_endymion_sim_set_connect(set, pad) };
        let pads = unsafe { emusen_endymion_pads_new(DEVICES_SIMULATED, set, std::ptr::null(), 0) };
        let mut events = [PadEventC::default(); 1];
        assert_eq!(unsafe { emusen_endymion_pads_start(pads, events.as_mut_ptr(), 0) }, 1, "an event that does not fit waits");
        assert_eq!(unsafe { emusen_endymion_pads_take(pads, events.as_mut_ptr(), 1) }, 1);
        assert_eq!(events[0], PadEventC { kind: 0, announce: 0, key: 1 });
        unsafe { emusen_endymion_sim_pad_press(pad, 0, 1) };
        assert_eq!(unsafe { emusen_endymion_pads_get(pads, 1, PADS_RAW_PRESSED, 0) }, 1);
        let mut name = [0u8; 8];
        assert_eq!(unsafe { emusen_endymion_pads_text(pads, 1, PADS_NAME, name.as_mut_ptr(), 8) }, 3);
        assert_eq!(&name[..3], b"Pad");
        assert_eq!(unsafe { emusen_endymion_sim_set_get(set, SET_OPEN_HANDLES) }, 1);
        unsafe { emusen_endymion_pads_close_all(pads) };
        assert_eq!(unsafe { emusen_endymion_sim_set_get(set, SET_OPEN_HANDLES) }, 0);
        unsafe {
            emusen_endymion_pads_free(pads);
            emusen_endymion_sim_set_free(set);
            emusen_endymion_sim_pad_free(pad);
        }
    }

    #[test]
    fn nothing_that_needs_sdl_is_made_before_sdl_is_lent() {
        if SDL.get().is_none() {
            assert!(emusen_endymion_ui_new().is_null());
            assert!(unsafe { emusen_endymion_pads_new(DEVICES_SDL, std::ptr::null(), std::ptr::null(), 0) }.is_null());
        }
        assert_eq!(unsafe { emusen_endymion_sdl_lend(std::ptr::null_mut()) }, if SDL.get().is_some() { 0 } else { status::NULL });
    }
}
