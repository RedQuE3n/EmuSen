//! Every connected pad, opened, hot-plugged and let go, and read through the stick and trigger rules: the C#
//! `GamepadManager`'s device bookkeeping and `ConnectedPad`. Who sits where is `slots`, which the host asks after each
//! pad opened; the bindings are the host's. See EmuSen_Input.md §4 and §8, EmuSen_Settings_Reference.md §4.4 and §4.61.

use crate::devices::{Handle, PadDevices, SDL_BUTTONS};
use crate::pad;
use std::collections::HashMap;
use std::time::Instant;

/// A rescan's interval, one second in .NET ticks.
pub const RESCAN_TICKS: i64 = 10_000_000;

/// A pad as this manager opened it; kept after it closes, so it is still known by name.
#[derive(Clone, Debug)]
pub struct Pad {
    pub id: u32,
    handle: Handle,
    /// Read once at opening, so a pad that has gone is still known by them.
    pub guid: String,
    pub path: Option<String>,
    closed_name: Option<String>,
    closed_kind: i32,
}

/// A pad opened or closed by a start, a poll or a change of devices, in the order it happened.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadEvent {
    /// The host seats it and lights the players; `announce` is false for the pads present at start.
    Opened { key: u64, announce: bool },
    Closed { key: u64 },
}

/// The reading rules' settings, as the C# has them by default.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub first_controller_only: bool,
    pub analog_stick_as_dpad: bool,
    pub stick_deadzone: f64,
    pub left_stick_is_analog: bool,
    pub analog_deadzone: f64,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings { first_controller_only: false, analog_stick_as_dpad: true, stick_deadzone: 0.5, left_stick_is_analog: false, analog_deadzone: 0.1 }
    }
}

pub struct Pads {
    devices: Box<dyn PadDevices>,
    /// The open pads in the order they were opened; the first is the first controller.
    open: Vec<u64>,
    known: HashMap<u64, Pad>,
    next_key: u64,
    initialized: bool,
    started: bool,
    clock: Instant,
    last_rescan: i64,
    pub settings: Settings,
}

/// Whether a rescan is due; a difference too large for a tick count is one.
pub fn rescan_due(now: i64, last: i64) -> bool {
    now.checked_sub(last).is_none_or(|elapsed| elapsed >= RESCAN_TICKS)
}

/// The SDL axis behind each of the pad's.
fn sdl_axis(axis: u32) -> i32 {
    match axis {
        pad::LEFT_X => 0,
        pad::LEFT_Y => 1,
        pad::RIGHT_X => 2,
        pad::RIGHT_Y => 3,
        pad::LEFT_TRIGGER => 4,
        _ => 5,
    }
}

impl Pads {
    /// Leaves the devices untouched until `start`.
    pub fn new(devices: Box<dyn PadDevices>) -> Pads {
        Pads {
            devices,
            open: Vec::new(),
            known: HashMap::new(),
            next_key: 1,
            initialized: false,
            started: false,
            clock: Instant::now(),
            last_rescan: -RESCAN_TICKS,
            settings: Settings::default(),
        }
    }

    pub fn started(&self) -> bool {
        self.started
    }

    /// Whether the devices started; until they do a poll does nothing.
    pub fn initialized(&self) -> bool {
        self.initialized
    }

    /// The last rescan, in ticks since this was made; a second before it at first.
    pub fn last_rescan(&self) -> i64 {
        self.last_rescan
    }

    pub fn devices(&mut self) -> &mut dyn PadDevices {
        self.devices.as_mut()
    }

    /// Idempotent; the pads present are opened and not announced.
    pub fn start(&mut self) -> Vec<PadEvent> {
        let mut events = Vec::new();
        if self.started {
            return events;
        }
        self.started = true;
        self.initialized = self.devices.init();
        if self.initialized {
            self.open_attached(false, &mut events);
        }
        events
    }

    fn open_attached(&mut self, announce: bool, events: &mut Vec<PadEvent>) {
        for id in self.devices.attached() {
            if self.open.iter().any(|key| self.known[key].id == id) {
                continue;
            }
            let handle = self.devices.open(id);
            if handle == 0 {
                continue;
            }
            let guid = self.devices.guid(handle);
            let path = self.devices.path(handle);
            let key = self.next_key;
            self.next_key += 1;
            self.known.insert(key, Pad { id, handle, guid, path, closed_name: None, closed_kind: 0 });
            self.open.push(key);
            events.push(PadEvent::Opened { key, announce });
        }
    }

    /// Once per frame tick: lets go of pads pulled out, and opens new ones on the devices' word or the one-a-second rescan.
    /// The host seats what opened and then calls `update`, as the C# updates the devices at the end of its poll.
    pub fn poll(&mut self) -> Vec<PadEvent> {
        let ticks = self.clock.elapsed().as_nanos() as i64 / 100;
        self.poll_at(ticks)
    }

    /// `poll` at a given time, in ticks since this was made.
    pub fn poll_at(&mut self, now: i64) -> Vec<PadEvent> {
        let mut events = Vec::new();
        if !self.initialized {
            return events;
        }
        let mut i = 0;
        while i < self.open.len() {
            let key = self.open[i];
            if self.devices.is_attached(self.known[&key].handle) {
                i += 1;
                continue;
            }
            self.open.remove(i);
            self.close(key);
            events.push(PadEvent::Closed { key });
        }
        // Both asked, as the C# asks both with `|`: a batch of the devices' changes is consumed even when a rescan is due.
        let changed = self.devices.devices_changed();
        if changed | rescan_due(now, self.last_rescan) {
            self.last_rescan = now;
            self.open_attached(true, &mut events);
        }
        events
    }

    /// The devices' state brought up to date, after a poll's pads are seated.
    pub fn update(&mut self) {
        if self.initialized {
            self.devices.update();
        }
    }

    fn close(&mut self, key: u64) {
        let handle = self.known[&key].handle;
        if handle == 0 {
            return;
        }
        let name = self.name(key);
        let kind = self.kind(key);
        self.devices.close(handle);
        let pad = self.known.get_mut(&key).expect("a known pad");
        pad.closed_name = Some(name);
        pad.closed_kind = kind;
        pad.handle = 0;
    }

    /// Lets go of every pad and the devices.
    pub fn close_all(&mut self) {
        for key in std::mem::take(&mut self.open) {
            self.close(key);
        }
        if self.initialized {
            self.devices.quit();
        }
        self.initialized = false;
    }

    /// Hides every pad of the devices but these from the next start, poll or rescan; a pad already open stays open.
    pub fn show_only(&mut self, ids: Option<std::collections::HashSet<u32>>) {
        self.devices.show_only(ids);
    }

    /// Forgets the open pads and the devices without closing them.
    pub fn abandon(mut self) {
        self.open.clear();
        self.initialized = false;
    }

    /// Lets go of every pad on the old devices and starts on these.
    pub fn use_devices(&mut self, devices: Box<dyn PadDevices>) -> Vec<PadEvent> {
        self.close_all();
        self.started = false;
        self.devices = devices;
        self.start()
    }

    /// The open pads' keys, in the order they were opened.
    pub fn open_pads(&self) -> &[u64] {
        &self.open
    }

    pub fn pad(&self, key: u64) -> Option<&Pad> {
        self.known.get(&key)
    }

    fn handle(&self, key: u64) -> Handle {
        self.known.get(&key).map_or(0, |pad| pad.handle)
    }

    pub fn is_open(&self, key: u64) -> bool {
        self.handle(key) != 0
    }

    pub fn name(&mut self, key: u64) -> String {
        let handle = self.handle(key);
        if handle == 0 {
            return self.known.get(&key).and_then(|pad| pad.closed_name.clone()).unwrap_or_else(|| "Unknown controller".to_string());
        }
        self.devices.name(handle).filter(|name| !name.is_empty()).unwrap_or_else(|| "Unknown controller".to_string())
    }

    /// SDL's own reading of what the pad is.
    pub fn kind(&mut self, key: u64) -> i32 {
        let handle = self.handle(key);
        if handle == 0 { self.known.get(&key).map_or(0, |pad| pad.closed_kind) } else { self.devices.kind(handle) }
    }

    pub fn raw_pressed(&mut self, key: u64, button: i32) -> bool {
        let handle = self.handle(key);
        handle != 0 && self.devices.button(handle, button)
    }

    pub fn axis_value(&mut self, key: u64, axis: i32) -> i16 {
        let handle = self.handle(key);
        if handle == 0 { 0 } else { self.devices.axis(handle, axis) }
    }

    /// -1 to 1.
    pub fn raw_axis(&mut self, key: u64, axis: i32) -> f64 {
        crate::clamp(self.axis_value(key, axis) as f64 / i16::MAX as f64, -1.0, 1.0)
    }

    /// The pad's own printed label for a button, as SDL numbers them; None when closed or not known.
    pub fn label(&mut self, key: u64, button: i32) -> Option<i32> {
        let handle = self.handle(key);
        if handle == 0 {
            return None;
        }
        Some(self.devices.label(handle, button)).filter(|&label| label != 0)
    }

    pub fn set_player_index(&mut self, key: u64, index: i32) {
        let handle = self.handle(key);
        if handle != 0 {
            self.devices.set_player_index(handle, index);
        }
    }

    /// How many open pads, from the first, the interface reads.
    pub fn frontend_pad_count(&self) -> usize {
        if self.settings.first_controller_only { self.open.len().min(1) } else { self.open.len() }
    }

    /// A pad's button as the game hears it: the stick standing in for the d-pad, a trigger past half its travel, else the bound SDL button.
    pub fn is_pressed(&mut self, key: u64, button: u32, bound: Option<i32>) -> bool {
        let s = self.settings;
        if s.analog_stick_as_dpad && !s.left_stick_is_analog && self.stick_direction_pressed(key, button) {
            return true;
        }
        if (button == pad::L2 || button == pad::R2) && self.axis(key, if button == pad::L2 { pad::LEFT_TRIGGER } else { pad::RIGHT_TRIGGER }) >= 0.5 {
            return true;
        }
        match bound {
            Some(sdl_button) => self.raw_pressed(key, sdl_button),
            None => false,
        }
    }

    /// Sticks -1 to 1, right and down positive, triggers 0 to 1, zero below the deadzone.
    pub fn axis(&mut self, key: u64, axis: u32) -> f64 {
        let value = self.raw_axis(key, sdl_axis(axis));
        if value.abs() < self.settings.analog_deadzone { 0.0 } else { value }
    }

    fn stick_direction_pressed(&mut self, key: u64, button: u32) -> bool {
        let threshold = (crate::clamp(self.settings.stick_deadzone, 0.05, 0.95) * i16::MAX as f64) as i16 as i32;
        match button {
            pad::LEFT => (self.axis_value(key, 0) as i32) < -threshold,
            pad::RIGHT => self.axis_value(key, 0) as i32 > threshold,
            pad::UP => (self.axis_value(key, 1) as i32) < -threshold,
            pad::DOWN => self.axis_value(key, 1) as i32 > threshold,
            _ => false,
        }
    }

    /// The first SDL button held on any pad the interface reads, the devices updated first.
    pub fn any_pressed(&mut self) -> Option<i32> {
        if self.open.is_empty() {
            return None;
        }
        self.devices.update();
        for i in 0..self.frontend_pad_count() {
            let key = self.open[i];
            for button in 0..SDL_BUTTONS {
                if self.raw_pressed(key, button) {
                    return Some(button);
                }
            }
        }
        None
    }
}

impl Drop for Pads {
    fn drop(&mut self) {
        self.close_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::devices::{SharedPad, SharedSet, SimulatedPad, SimulatedPads, lock, lock_set};
    use std::sync::{Arc, Mutex};

    fn pad(name: &str) -> SharedPad {
        let mut pad = SimulatedPad::new(Some(format!("/dev/input/{name}")));
        pad.name = name.into();
        Arc::new(Mutex::new(pad))
    }

    fn set(pads: &[&SharedPad]) -> SharedSet {
        let set = Arc::new(Mutex::new(SimulatedPads::new()));
        for pad in pads {
            lock_set(&set).connect((*pad).clone());
        }
        set
    }

    #[test]
    fn the_pads_at_start_open_unannounced_and_a_plug_and_a_pull_are_announced_in_order() {
        let (first, second) = (pad("First"), pad("Second"));
        let set = set(&[&first]);
        let mut pads = Pads::new(Box::new(set.clone()));
        assert_eq!(pads.start(), [PadEvent::Opened { key: 1, announce: false }]);
        assert!(pads.start().is_empty());
        lock_set(&set).connect(second.clone());
        assert_eq!(pads.poll_at(0), [PadEvent::Opened { key: 2, announce: true }]);
        lock(&second).kind = 6;
        lock_set(&set).disconnect(&second);
        assert_eq!(pads.poll_at(5), [PadEvent::Closed { key: 2 }]);
        assert_eq!((pads.name(2), pads.kind(2), pads.is_open(2)), ("Second".to_string(), 6, false));
        assert_eq!(lock_set(&set).open_handles(), 1);
        pads.close_all();
        let devices = lock_set(&set);
        assert_eq!((devices.opens, devices.closes, devices.initialized), (2, 2, false));
    }

    #[test]
    fn a_pad_pulled_and_plugged_back_waits_for_the_devices_word_or_the_rescan() {
        let only = pad("Only");
        let set = set(&[&only]);
        let mut pads = Pads::new(Box::new(set.clone()));
        pads.start();
        lock_set(&set).disconnect(&only);
        assert_eq!(pads.poll_at(0), [PadEvent::Closed { key: 1 }]);
        lock_set(&set).connect(only.clone());
        assert_eq!(pads.poll_at(1), [PadEvent::Opened { key: 2, announce: true }]);
        assert!(rescan_due(0, -RESCAN_TICKS) && !rescan_due(15_000_000, 6_000_000) && rescan_due(16_000_000, 6_000_000));
        assert!(rescan_due(i64::MAX, -RESCAN_TICKS), "a difference past a tick count is a rescan, not a fault");
    }

    #[test]
    fn the_stick_stands_in_for_the_dpad_a_trigger_is_l2_and_the_rest_is_the_bound_button() {
        let only = pad("Only");
        let set = set(&[&only]);
        let mut pads = Pads::new(Box::new(set));
        pads.start();
        lock(&only).set_axis(0, -0.6);
        assert!(pads.is_pressed(1, pad::LEFT, None));
        pads.settings.left_stick_is_analog = true;
        assert!(!pads.is_pressed(1, pad::LEFT, None));
        lock(&only).set_axis(4, 0.5);
        assert!(pads.is_pressed(1, pad::L2, None));
        lock(&only).set_axis(4, 0.05);
        assert_eq!(pads.axis(1, pad::LEFT_TRIGGER), 0.0);
        lock(&only).press(3);
        assert!(pads.is_pressed(1, 8, Some(3)) && !pads.is_pressed(1, 8, Some(2)) && !pads.is_pressed(1, 8, None));
        assert_eq!(pads.any_pressed(), Some(3));
        assert_eq!(pads.label(1, 0), None);
    }
}
