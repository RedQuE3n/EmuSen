//! Endymion's C layer: the resampler, the rate control, the router, the seats' rules and the pad's rules.
//!
//! A resampler, a rate control and a router are handles, used by one thread at a time. A call that changes one and
//! makes output takes the caller's buffer; output that does not fit waits in the handle for `*_take`, so that a
//! stateful call is never made twice to learn its length.

use crate::{fail, status};
use emusen_endymion::pad;
use emusen_endymion::rate_control::DynamicRateControl;
use emusen_endymion::resampler::{LinearResampler, RatioError};
use emusen_endymion::router::{Change, Inputs, PortRouter};
use emusen_endymion::slots::{self, MAX_PLAYERS, PlayerSlots};

/// A value and the output its last call made that the caller has not taken.
pub struct Held<T, O> {
    value: T,
    pending: Vec<O>,
}

/// The output handed over: into the caller's buffer when it holds all of it, else kept for `take`. Its whole count.
///
/// # Safety
/// `out` must be valid for `cap` elements, or null.
unsafe fn hand_over<O: Copy>(pending: &mut Vec<O>, made: Vec<O>, out: *mut O, cap: usize) -> i64 {
    let count = made.len() as i64;
    if !out.is_null() && made.len() <= cap {
        unsafe { std::ptr::copy_nonoverlapping(made.as_ptr(), out, made.len()) };
        pending.clear();
    } else {
        *pending = made;
    }
    count
}

/// What is waiting, copied into the caller's buffer when it holds all of it and then forgotten. Its whole count.
///
/// # Safety
/// `out` must be valid for `cap` elements, or null.
unsafe fn take<O: Copy>(pending: &mut Vec<O>, out: *mut O, cap: usize) -> i64 {
    let count = pending.len() as i64;
    if !out.is_null() && pending.len() <= cap {
        unsafe { std::ptr::copy_nonoverlapping(pending.as_ptr(), out, pending.len()) };
        pending.clear();
    }
    count
}

/// The samples a caller passed.
///
/// # Safety
/// `data` must be valid for `len` elements, or null.
unsafe fn samples<'a>(data: *const i16, len: usize) -> &'a [i16] {
    if data.is_null() || len == 0 { &[] } else { unsafe { std::slice::from_raw_parts(data, len) } }
}

fn ratio_fail(error: RatioError) -> i64 {
    fail(status::BAD_ARGUMENT, error.words()) as i64
}

macro_rules! handle {
    ($h:expr) => {
        match unsafe { $h.as_mut() } {
            Some(held) => held,
            None => return fail(status::NULL, "no handle").into(),
        }
    };
}

pub type Resampler = Held<LinearResampler, i16>;

#[unsafe(no_mangle)]
pub extern "C" fn emusen_endymion_resampler_new() -> *mut Resampler {
    Box::into_raw(Box::new(Held { value: LinearResampler::new(), pending: Vec::new() }))
}

/// # Safety
/// `h` must come from `emusen_endymion_resampler_new` and not have been freed, or be null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_resampler_free(h: *mut Resampler) -> i32 {
    if !h.is_null() {
        drop(unsafe { Box::from_raw(h) });
    }
    0
}

/// # Safety
/// As `emusen_endymion_resampler_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_resampler_reset(h: *mut Resampler) -> i32 {
    let held = handle!(h);
    held.value.reset();
    held.pending.clear();
    0
}

/// Resamples interleaved stereo by `ratio` output frames per input frame: the count of samples made.
///
/// # Safety
/// `h` as above; `input` valid for `len` samples and `out` for `cap`, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_resampler_run(h: *mut Resampler, input: *const i16, len: usize, ratio: f64, out: *mut i16, cap: usize) -> i64 {
    let held = handle!(h);
    let mut made = Vec::new();
    match held.value.resample(unsafe { samples(input, len) }, ratio, &mut made) {
        Ok(()) => unsafe { hand_over(&mut held.pending, made, out, cap) },
        Err(error) => ratio_fail(error),
    }
}

/// # Safety
/// `h` as above; `out` valid for `cap` samples, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_resampler_take(h: *mut Resampler, out: *mut i16, cap: usize) -> i64 {
    let held = handle!(h);
    unsafe { take(&mut held.pending, out, cap) }
}

pub type Rate = Held<DynamicRateControl, i16>;

/// A rate control's whole state, as one struct a host reads its properties from.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RateState {
    pub size: u32,
    pub target_queued_frames: i32,
    pub shedding_events: i32,
    pub is_shedding: u32,
    pub total_input_frames: i64,
    pub total_output_frames: i64,
    pub max_deviation: f64,
    pub shedding_entry_factor: f64,
    pub shedding_exit_factor: f64,
    pub nominal_ratio: f64,
    pub last_ratio: f64,
}

/// What `emusen_endymion_rate_set` sets.
pub mod rate_setting {
    pub const TARGET_QUEUED_FRAMES: u32 = 0;
    pub const MAX_DEVIATION: u32 = 1;
    pub const SHEDDING_ENTRY_FACTOR: u32 = 2;
    pub const SHEDDING_EXIT_FACTOR: u32 = 3;
    pub const NOMINAL_RATIO: u32 = 4;
}

#[unsafe(no_mangle)]
pub extern "C" fn emusen_endymion_rate_new(target_queued_frames: i32) -> *mut Rate {
    Box::into_raw(Box::new(Held { value: DynamicRateControl::new(target_queued_frames), pending: Vec::new() }))
}

/// # Safety
/// `h` must come from `emusen_endymion_rate_new` and not have been freed, or be null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_rate_free(h: *mut Rate) -> i32 {
    if !h.is_null() {
        drop(unsafe { Box::from_raw(h) });
    }
    0
}

/// Sets one of the rate control's settings; the target is a whole number of frames given as a double.
///
/// # Safety
/// As `emusen_endymion_rate_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_rate_set(h: *mut Rate, which: u32, value: f64) -> i32 {
    let control = &mut handle!(h).value;
    match which {
        rate_setting::TARGET_QUEUED_FRAMES if value.fract() == 0.0 && value >= i32::MIN as f64 && value <= i32::MAX as f64 => control.target_queued_frames = value as i32,
        rate_setting::MAX_DEVIATION => control.max_deviation = value,
        rate_setting::SHEDDING_ENTRY_FACTOR => control.shedding_entry_factor = value,
        rate_setting::SHEDDING_EXIT_FACTOR => control.shedding_exit_factor = value,
        rate_setting::NOMINAL_RATIO => control.nominal_ratio = value,
        _ => return fail(status::BAD_ARGUMENT, format!("no setting {which} for {value}")),
    }
    0
}

/// The rate control's state, into a struct whose `size` the caller set.
///
/// # Safety
/// As `emusen_endymion_rate_free`; `out` valid for its `size` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_rate_read(h: *mut Rate, out: *mut RateState) -> i32 {
    let control = &handle!(h).value;
    let Some(out) = (unsafe { out.as_mut() }) else {
        return fail(status::NULL, "no state to fill");
    };
    if (out.size as usize) < std::mem::size_of::<RateState>() {
        return fail(status::BAD_ARGUMENT, "the state struct is smaller than this build's");
    }
    *out = RateState {
        size: out.size,
        target_queued_frames: control.target_queued_frames,
        shedding_events: control.shedding_events,
        is_shedding: control.is_shedding() as u32,
        total_input_frames: control.total_input_frames,
        total_output_frames: control.total_output_frames,
        max_deviation: control.max_deviation,
        shedding_entry_factor: control.shedding_entry_factor,
        shedding_exit_factor: control.shedding_exit_factor,
        nominal_ratio: control.nominal_ratio,
        last_ratio: control.last_ratio,
    };
    0
}

/// The ratio a queue of `queued_frames` calls for.
///
/// # Safety
/// As `emusen_endymion_rate_free`; `out` valid, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_rate_compute(h: *mut Rate, queued_frames: i32, out: *mut f64) -> i32 {
    let control = &handle!(h).value;
    match unsafe { out.as_mut() } {
        Some(out) => {
            *out = control.compute_ratio(queued_frames);
            0
        }
        None => fail(status::NULL, "nowhere to put the ratio"),
    }
}

/// Processes one batch of samples against the queue's fill: the count of samples to hand the device.
///
/// # Safety
/// `h` as above; `input` valid for `len` samples and `out` for `cap`, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_rate_process(h: *mut Rate, input: *const i16, len: usize, queued_frames: i32, out: *mut i16, cap: usize) -> i64 {
    let held = handle!(h);
    if input.is_null() {
        // C# works out the ratio and then fails on the missing array.
        held.value.last_ratio = held.value.compute_ratio(queued_frames);
        return fail(status::NULL, "no samples") as i64;
    }
    let mut made = Vec::new();
    match held.value.process(unsafe { samples(input, len) }, queued_frames, &mut made) {
        Ok(()) => unsafe { hand_over(&mut held.pending, made, out, cap) },
        Err(error) => ratio_fail(error),
    }
}

/// # Safety
/// As `emusen_endymion_rate_free`; `out` valid for `cap` samples, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_rate_take(h: *mut Rate, out: *mut i16, cap: usize) -> i64 {
    let held = handle!(h);
    unsafe { take(&mut held.pending, out, cap) }
}

/// # Safety
/// As `emusen_endymion_rate_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_rate_reset(h: *mut Rate) -> i32 {
    let held = handle!(h);
    held.value.reset();
    held.pending.clear();
    0
}

/// One change the router asks the host to make, as `EMUSEN_ENDYMION_CHANGE_*` says.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RouterChange {
    pub kind: u32,
    pub port: i32,
    /// The button or the axis.
    pub which: u32,
    /// A button held, or a port holding a controller.
    pub on: u32,
    pub value: f64,
}

pub mod change {
    pub const CONNECTED: u32 = 0;
    pub const BUTTON: u32 = 1;
    pub const AXIS: u32 = 2;
}

impl From<Change> for RouterChange {
    fn from(c: Change) -> RouterChange {
        match c {
            Change::Connected { port, on } => RouterChange { kind: change::CONNECTED, port, which: 0, on: on as u32, value: 0.0 },
            Change::Button { port, button, held } => RouterChange { kind: change::BUTTON, port, which: button, on: held as u32, value: 0.0 },
            Change::Axis { port, axis, value } => RouterChange { kind: change::AXIS, port, which: axis, on: 0, value },
        }
    }
}

pub type Router = Held<PortRouter, RouterChange>;

/// What `emusen_endymion_router_get` and `_set` reach.
pub mod router_value {
    pub const KEYBOARD_PLAYER: u32 = 0;
    pub const MIRROR_PLAYER1_TO_PLAYER2: u32 = 1;
    pub const PORTS: u32 = 2;
    pub const PLAYERS_READ: u32 = 3;
    pub const AXES: u32 = 4;
}

#[unsafe(no_mangle)]
pub extern "C" fn emusen_endymion_router_new() -> *mut Router {
    Box::into_raw(Box::new(Held { value: PortRouter::new(), pending: Vec::new() }))
}

/// # Safety
/// `h` must come from `emusen_endymion_router_new` and not have been freed, or be null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_router_free(h: *mut Router) -> i32 {
    if !h.is_null() {
        drop(unsafe { Box::from_raw(h) });
    }
    0
}

/// # Safety
/// As `emusen_endymion_router_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_router_get(h: *mut Router, which: u32) -> i32 {
    let router = &handle!(h).value;
    match which {
        router_value::KEYBOARD_PLAYER => router.keyboard_player,
        router_value::MIRROR_PLAYER1_TO_PLAYER2 => router.mirror_player1_to_player2 as i32,
        router_value::PORTS => router.ports(),
        router_value::PLAYERS_READ => router.players_read() as i32,
        router_value::AXES => router.axes().len() as i32,
        _ => fail(status::BAD_ARGUMENT, format!("no router value {which}")),
    }
}

/// # Safety
/// As `emusen_endymion_router_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_router_set(h: *mut Router, which: u32, value: i32) -> i32 {
    let router = &mut handle!(h).value;
    match which {
        router_value::KEYBOARD_PLAYER => router.keyboard_player = value,
        router_value::MIRROR_PLAYER1_TO_PLAYER2 => router.mirror_player1_to_player2 = value != 0,
        _ => return fail(status::BAD_ARGUMENT, format!("no settable router value {which}")),
    }
    0
}

/// A game just started: its ports and the axes it reads, by `PadAxis`.
///
/// # Safety
/// As `emusen_endymion_router_free`; `axes` valid for `count` values, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_router_reset(h: *mut Router, ports: i32, axes: *const u32, count: usize) -> i32 {
    let held = handle!(h);
    let axes = if axes.is_null() || count == 0 { &[][..] } else { unsafe { std::slice::from_raw_parts(axes, count) } };
    held.value.reset(ports, axes);
    held.pending.clear();
    0
}

fn changes(made: Vec<Change>) -> Vec<RouterChange> {
    made.into_iter().map(RouterChange::from).collect()
}

/// The game's ports changed in number: the buttons let go on the ports taken away.
///
/// # Safety
/// As `emusen_endymion_router_free`; `out` valid for `cap` changes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_router_resize(h: *mut Router, ports: i32, out: *mut RouterChange, cap: usize) -> i64 {
    let held = handle!(h);
    let made = changes(held.value.resize(ports));
    unsafe { hand_over(&mut held.pending, made, out, cap) }
}

/// The pads as read for each player a port hears, from player 1: `held` a mask by `PadButton` for each, `axes` the read axes' values for each in the order the reset gave them. Then every change.
///
/// # Safety
/// As `emusen_endymion_router_free`; `held` valid for `players` masks, `axes` for `axes_len` values, `out` for `cap` changes, each or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_router_poll(h: *mut Router, held: *const u16, players: usize, axes: *const f64, axes_len: usize, keyboard: u32, seated: u32, out: *mut RouterChange, cap: usize) -> i64 {
    let router = handle!(h);
    let masks = if held.is_null() || players == 0 { &[][..] } else { unsafe { std::slice::from_raw_parts(held, players) } };
    let values = if axes.is_null() || axes_len == 0 { &[][..] } else { unsafe { std::slice::from_raw_parts(axes, axes_len) } };
    let made = changes(router.value.poll(masks, values, Inputs { keyboard, seated }));
    unsafe { hand_over(&mut router.pending, made, out, cap) }
}

/// Every change with the pads as last read: after a key went down or up.
///
/// # Safety
/// As `emusen_endymion_router_free`; `out` valid for `cap` changes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_router_send(h: *mut Router, keyboard: u32, seated: u32, out: *mut RouterChange, cap: usize) -> i64 {
    let router = handle!(h);
    let made = changes(router.value.send(Inputs { keyboard, seated }));
    unsafe { hand_over(&mut router.pending, made, out, cap) }
}

/// # Safety
/// As `emusen_endymion_router_free`; `out` valid for `cap` changes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_router_take(h: *mut Router, out: *mut RouterChange, cap: usize) -> i64 {
    let held = handle!(h);
    unsafe { take(&mut held.pending, out, cap) }
}

/// Whether a player's pad held a button at the last poll: 1 or 0.
///
/// # Safety
/// As `emusen_endymion_router_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_router_pad_held(h: *mut Router, player: i32, button: u32) -> i32 {
    handle!(h).value.pad_held(player, button) as i32
}

/// Whether a port holds a controller, given which players have a pad seated: 1 or 0.
///
/// # Safety
/// As `emusen_endymion_router_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_router_connected(h: *mut Router, port: i32, seated: u32) -> i32 {
    handle!(h).value.connected(port, Inputs { keyboard: 0, seated }) as i32
}

/// A seat as a host describes it: empty, or a pad connected or gone, with its GUID and its path, the path null when the pad has none.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Seat {
    pub state: u32,
    pub guid: *const u8,
    pub guid_len: usize,
    pub path: *const u8,
    pub path_len: usize,
}

pub mod seat_state {
    pub const EMPTY: u32 = 0;
    pub const OPEN: u32 = 1;
    pub const CLOSED: u32 = 2;
}

#[derive(Clone, Debug)]
struct Described {
    open: bool,
    guid: String,
    path: Option<String>,
    /// Which seat the pad came from, or `MAX_PLAYERS` for the pad a rule is given.
    from: usize,
}

impl slots::Pad for Described {
    fn is_open(&self) -> bool {
        self.open
    }
    fn guid(&self) -> &str {
        &self.guid
    }
    fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }
}

/// # Safety
/// `data` valid for `len` bytes, or null.
unsafe fn string(data: *const u8, len: usize) -> Result<Option<String>, i32> {
    unsafe { crate::text(data, len) }.map(|text| text.map(str::to_string))
}

/// The eight seats as a host describes them.
///
/// # Safety
/// `seats` valid for eight seats, each string valid for its length.
unsafe fn described(seats: *const Seat) -> Result<PlayerSlots<Described>, i32> {
    if seats.is_null() {
        return Err(fail(status::NULL, "no seats"));
    }
    let given = unsafe { std::slice::from_raw_parts(seats, MAX_PLAYERS) };
    let mut out: [Option<Described>; MAX_PLAYERS] = Default::default();
    for (i, seat) in given.iter().enumerate() {
        if seat.state == seat_state::EMPTY {
            continue;
        }
        let guid = unsafe { string(seat.guid, seat.guid_len) }?.unwrap_or_default();
        let path = unsafe { string(seat.path, seat.path_len) }?;
        out[i] = Some(Described { open: seat.state == seat_state::OPEN, guid, path, from: i });
    }
    Ok(PlayerSlots::new(out))
}

/// The seat a pad just connected takes, by the rule: the player it becomes, or 0 when every seat holds a connected pad.
///
/// # Safety
/// `seats` valid for eight seats; each string valid for its length, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_slots_seat(seats: *const Seat, guid: *const u8, guid_len: usize, path: *const u8, path_len: usize) -> i32 {
    let result = || -> Result<i32, i32> {
        let mut slots = unsafe { described(seats) }?;
        let pad = Described { open: true, guid: unsafe { string(guid, guid_len) }?.unwrap_or_default(), path: unsafe { string(path, path_len) }?, from: MAX_PLAYERS };
        Ok(slots.seat(pad) as i32)
    };
    result().unwrap_or_else(|status| status)
}

/// A pad, seated as `from` (0 for none), moved to `player` (0 for none): 1 when the seats changed and `out` holds the new ones, 0 when they did not. Each seat of `out` is -1 for empty, 0 to 7 for the pad that was in that seat, 8 for the pad moved.
///
/// # Safety
/// `seats` valid for eight seats and `out` for eight values.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_slots_move(seats: *const Seat, mover_open: u32, from: i32, player: i32, out: *mut i32) -> i32 {
    let result = || -> Result<i32, i32> {
        if out.is_null() || !(0..=MAX_PLAYERS as i32).contains(&from) {
            return Err(fail(status::BAD_ARGUMENT, "no seats to fill, or no such seat to move from"));
        }
        let mut slots = unsafe { described(seats) }?;
        let mover = Described { open: mover_open != 0, guid: String::new(), path: None, from: MAX_PLAYERS };
        let changed = slots.move_pad(mover, from as usize, player).map_err(|_| fail(status::BAD_ARGUMENT, "Specified argument was out of the range of valid values."))?;
        let out = unsafe { std::slice::from_raw_parts_mut(out, MAX_PLAYERS) };
        for (i, seat) in slots.seats.iter().enumerate() {
            out[i] = seat.as_ref().map_or(-1, |pad| pad.from as i32);
        }
        Ok(changed as i32)
    };
    result().unwrap_or_else(|status| status)
}

/// Lets go of a reservation: 1 when there was one to let go of, 0 otherwise.
///
/// # Safety
/// `seats` valid for eight seats.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_slots_forget(seats: *const Seat, player: i32) -> i32 {
    match unsafe { described(seats) } {
        Ok(mut slots) => slots.forget(player) as i32,
        Err(status) => status,
    }
}

/// The highest player with a pad seated, connected or reserved; 0 for none.
///
/// # Safety
/// `seats` valid for eight seats.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_slots_highest(seats: *const Seat) -> i32 {
    match unsafe { described(seats) } {
        Ok(slots) => slots.highest() as i32,
        Err(status) => status,
    }
}

/// `PadControls.Resolve`, the controls held given as a mask by `PadControl`.
///
/// # Safety
/// `out` valid, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_pad_resolve(axis: u32, analog: f64, held: u32, out: *mut f64) -> i32 {
    match unsafe { out.as_mut() } {
        Some(out) => {
            *out = pad::resolve(axis, analog, |control| control < 32 && held & (1 << control) != 0);
            0
        }
        None => fail(status::NULL, "nowhere to put the value"),
    }
}

/// `PadControls.Combine`.
///
/// # Safety
/// `out` valid, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_pad_combine(analog: f64, negative: u32, positive: u32, out: *mut f64) -> i32 {
    match unsafe { out.as_mut() } {
        Some(out) => {
            *out = pad::combine(analog, negative != 0, positive != 0);
            0
        }
        None => fail(status::NULL, "nowhere to put the value"),
    }
}

/// `PadControls.For`: a console's bindable controls from its buttons and the axes it reads; the whole count, `out` filled when it holds them all.
///
/// # Safety
/// Each pointer valid for its count, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_endymion_pad_controls_for(buttons: *const u32, buttons_len: usize, axes: *const u32, axes_len: usize, out: *mut u32, cap: usize) -> i64 {
    let slice = |data: *const u32, len: usize| if data.is_null() || len == 0 { &[][..] } else { unsafe { std::slice::from_raw_parts(data, len) } };
    let controls = pad::controls_for(slice(buttons, buttons_len), slice(axes, axes_len));
    if !out.is_null() && controls.len() <= cap {
        unsafe { std::ptr::copy_nonoverlapping(controls.as_ptr(), out, controls.len()) };
    }
    controls.len() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_that_does_not_fit_waits_to_be_taken() {
        let h = emusen_endymion_resampler_new();
        let input = [1i16, 1, 2, 2, 3, 3];
        let mut small = [0i16; 2];
        assert_eq!(unsafe { emusen_endymion_resampler_run(h, input.as_ptr(), 6, 1.0, small.as_mut_ptr(), 2) }, 4);
        assert_eq!(small, [0, 0]);
        let mut out = [0i16; 4];
        assert_eq!(unsafe { emusen_endymion_resampler_take(h, out.as_mut_ptr(), 4) }, 4);
        assert_eq!(out, [1, 1, 2, 2]);
        assert_eq!(unsafe { emusen_endymion_resampler_take(h, out.as_mut_ptr(), 4) }, 0);
        assert_eq!(unsafe { emusen_endymion_resampler_run(h, input.as_ptr(), 6, 0.0, out.as_mut_ptr(), 4) }, status::BAD_ARGUMENT as i64);
        unsafe { emusen_endymion_resampler_free(h) };
    }

    #[test]
    fn the_rate_state_is_read_whole_and_a_short_struct_is_refused() {
        let h = emusen_endymion_rate_new(1000);
        assert_eq!(unsafe { emusen_endymion_rate_set(h, rate_setting::NOMINAL_RATIO, 1.5) }, 0);
        assert_eq!(unsafe { emusen_endymion_rate_set(h, rate_setting::TARGET_QUEUED_FRAMES, 0.5) }, status::BAD_ARGUMENT);
        let mut state = RateState { size: std::mem::size_of::<RateState>() as u32, ..Default::default() };
        assert_eq!(unsafe { emusen_endymion_rate_read(h, &mut state) }, 0);
        assert_eq!((state.target_queued_frames, state.nominal_ratio, state.max_deviation), (1000, 1.5, 0.005));
        let mut short = RateState { size: 8, ..Default::default() };
        assert_eq!(unsafe { emusen_endymion_rate_read(h, &mut short) }, status::BAD_ARGUMENT);
        assert_eq!(std::mem::size_of::<RateState>(), 72);
        unsafe { emusen_endymion_rate_free(h) };
    }

    #[test]
    fn a_move_reports_where_each_pad_went() {
        let guid = b"g";
        let open = Seat { state: seat_state::OPEN, guid: guid.as_ptr(), guid_len: 1, path: std::ptr::null(), path_len: 0 };
        let empty = Seat { state: seat_state::EMPTY, guid: std::ptr::null(), guid_len: 0, path: std::ptr::null(), path_len: 0 };
        let mut seats = [empty; MAX_PLAYERS];
        seats[0] = open;
        seats[1] = open;
        let mut out = [0i32; MAX_PLAYERS];
        assert_eq!(unsafe { emusen_endymion_slots_move(seats.as_ptr(), 1, 1, 2, out.as_mut_ptr()) }, 1);
        assert_eq!(out[..3], [1, 8, -1]);
        assert_eq!(unsafe { emusen_endymion_slots_move(seats.as_ptr(), 1, 0, 9, out.as_mut_ptr()) }, status::BAD_ARGUMENT);
        assert_eq!(unsafe { emusen_endymion_slots_seat(seats.as_ptr(), guid.as_ptr(), 1, std::ptr::null(), 0) }, 3);
        assert_eq!(unsafe { emusen_endymion_slots_highest(seats.as_ptr()) }, 2);
        assert_eq!(std::mem::size_of::<RouterChange>(), 24);
    }
}
