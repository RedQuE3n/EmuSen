//! What the pads' bookkeeping asks of a gamepad API: SDL's, or a set of simulated pads a test plugs in and pulls out.
//! The C# `IPadDevices`, `SdlPadDevices`, `SimulatedPads` and `SimulatedPad`. See EmuSen_Input.md §8 and
//! EmuSen_Settings_Reference.md §4.61.

use crate::sdl::{self, Sdl};
use md5::{Digest, Md5};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard};

/// An opened pad as the devices know it; 0 is none.
pub type Handle = usize;

/// SDL's gamepad buttons, `SDL_GAMEPAD_BUTTON_COUNT` of them.
pub const SDL_BUTTONS: i32 = 26;

pub trait PadDevices: Send {
    fn init(&mut self) -> bool;
    fn quit(&mut self);
    fn attached(&mut self) -> Vec<u32>;
    fn open(&mut self, id: u32) -> Handle;
    fn close(&mut self, pad: Handle);
    fn is_attached(&mut self, pad: Handle) -> bool;
    /// True once for each batch of added and removed pads, which it consumes.
    fn devices_changed(&mut self) -> bool;
    fn update(&mut self);
    fn button(&mut self, pad: Handle, button: i32) -> bool;
    fn axis(&mut self, pad: Handle, axis: i32) -> i16;
    fn name(&mut self, pad: Handle) -> Option<String>;
    fn kind(&mut self, pad: Handle) -> i32;
    fn label(&mut self, pad: Handle, button: i32) -> i32;
    fn guid(&mut self, pad: Handle) -> String;
    fn path(&mut self, pad: Handle) -> Option<String>;
    fn set_player_index(&mut self, pad: Handle, index: i32);
    /// Hides every pad but these, or none for `None`; only a layer over real devices has pads to hide.
    fn show_only(&mut self, _ids: Option<HashSet<u32>>) {}
}

/// The real devices, through SDL3; `only`, when set, hides every pad but those, so a test of SDL's virtual pads seats no pad on the desk.
pub struct SdlPads {
    sdl: Arc<Sdl>,
    pub only: Option<HashSet<u32>>,
}

impl SdlPads {
    pub fn new(sdl: Arc<Sdl>) -> SdlPads {
        SdlPads { sdl, only: None }
    }

    fn pad(pad: Handle) -> *mut std::ffi::c_void {
        pad as *mut std::ffi::c_void
    }
}

/// SDL's GUID as the C# writes it: the sixteen bytes in SDL's order, in lower-case hexadecimal.
pub fn guid_text(guid: sdl::Guid) -> String {
    guid.data.iter().map(|b| format!("{b:02x}")).collect()
}

impl PadDevices for SdlPads {
    fn init(&mut self) -> bool {
        unsafe { (self.sdl.f.init_sub_system)(sdl::INIT_GAMEPAD) }
    }

    fn quit(&mut self) {
        unsafe { (self.sdl.f.quit_sub_system)(sdl::INIT_GAMEPAD) }
    }

    fn attached(&mut self) -> Vec<u32> {
        let mut count = 0;
        let ids = unsafe { (self.sdl.f.get_gamepads)(&mut count) };
        if ids.is_null() {
            return Vec::new();
        }
        let list = unsafe { std::slice::from_raw_parts(ids, count.max(0) as usize) }.to_vec();
        unsafe { (self.sdl.f.free)(ids.cast()) };
        match &self.only {
            Some(only) => list.into_iter().filter(|id| only.contains(id)).collect(),
            None => list,
        }
    }

    fn open(&mut self, id: u32) -> Handle {
        unsafe { (self.sdl.f.open_gamepad)(id) as Handle }
    }

    fn close(&mut self, pad: Handle) {
        unsafe { (self.sdl.f.close_gamepad)(Self::pad(pad)) }
    }

    fn is_attached(&mut self, pad: Handle) -> bool {
        unsafe { (self.sdl.f.gamepad_connected)(Self::pad(pad)) }
    }

    fn devices_changed(&mut self) -> bool {
        let changed = unsafe { (self.sdl.f.has_events)(sdl::EVENT_GAMEPAD_ADDED, sdl::EVENT_GAMEPAD_REMOVED) };
        if changed {
            unsafe { (self.sdl.f.flush_events)(sdl::EVENT_GAMEPAD_ADDED, sdl::EVENT_GAMEPAD_REMOVED) };
        }
        changed
    }

    fn update(&mut self) {
        unsafe { (self.sdl.f.update_gamepads)() }
    }

    fn button(&mut self, pad: Handle, button: i32) -> bool {
        unsafe { (self.sdl.f.get_gamepad_button)(Self::pad(pad), button) }
    }

    fn axis(&mut self, pad: Handle, axis: i32) -> i16 {
        unsafe { (self.sdl.f.get_gamepad_axis)(Self::pad(pad), axis) }
    }

    fn name(&mut self, pad: Handle) -> Option<String> {
        unsafe { sdl::text((self.sdl.f.get_gamepad_name)(Self::pad(pad))) }
    }

    fn kind(&mut self, pad: Handle) -> i32 {
        unsafe { (self.sdl.f.get_gamepad_type)(Self::pad(pad)) }
    }

    fn label(&mut self, pad: Handle, button: i32) -> i32 {
        unsafe { (self.sdl.f.get_gamepad_button_label)(Self::pad(pad), button) }
    }

    fn guid(&mut self, pad: Handle) -> String {
        guid_text(unsafe { (self.sdl.f.get_joystick_guid)((self.sdl.f.get_gamepad_joystick)(Self::pad(pad))) })
    }

    fn path(&mut self, pad: Handle) -> Option<String> {
        unsafe { sdl::text((self.sdl.f.get_gamepad_path)(Self::pad(pad))) }.filter(|path| !path.is_empty())
    }

    fn set_player_index(&mut self, pad: Handle, index: i32) {
        unsafe { (self.sdl.f.set_gamepad_player_index)(Self::pad(pad), index) };
    }

    fn show_only(&mut self, ids: Option<HashSet<u32>>) {
        self.only = ids;
    }
}

/// A pad with no device behind it, shared between the test that presses it and the set it is plugged into.
#[derive(Clone, Debug)]
pub struct SimulatedPad {
    pub name: String,
    /// SDL's GUID is the model's, so two pads of one name share one unless set.
    pub guid: Option<String>,
    pub path: Option<String>,
    /// The player number SDL was last asked to light on it, or -1.
    pub player_index: i32,
    pub kind: i32,
    held: HashSet<i32>,
    axes: HashMap<i32, i16>,
}

impl SimulatedPad {
    /// `path` is the host's to number, one per pad as a USB port's.
    pub fn new(path: Option<String>) -> SimulatedPad {
        SimulatedPad { name: "Simulated pad".to_string(), guid: None, path, player_index: -1, kind: 0, held: HashSet::new(), axes: HashMap::new() }
    }

    pub fn press(&mut self, button: i32) {
        self.held.insert(button);
    }

    pub fn release(&mut self, button: i32) {
        self.held.remove(&button);
    }

    pub fn release_all(&mut self) {
        self.held.clear();
        self.axes.clear();
    }

    /// -1 to 1 for a stick, 0 to 1 for a trigger, kept as SDL's sixteen bits.
    pub fn set_axis(&mut self, axis: i32, value: f64) {
        self.set_axis_raw(axis, (crate::clamp(value, -1.0, 1.0) * i16::MAX as f64).round_ties_even() as i16);
    }

    pub fn set_axis_raw(&mut self, axis: i32, value: i16) {
        self.axes.insert(axis, value);
    }

    pub fn is_held(&self, button: i32) -> bool {
        self.held.contains(&button)
    }

    pub fn axis(&self, axis: i32) -> i16 {
        self.axes.get(&axis).copied().unwrap_or(0)
    }

    /// The GUID it reports: its own, or the MD5 of its name as the model's.
    pub fn reported_guid(&self) -> String {
        match &self.guid {
            Some(guid) => guid.clone(),
            None => Md5::digest(self.name.as_bytes()).iter().map(|b| format!("{b:02x}")).collect(),
        }
    }
}

pub type SharedPad = Arc<Mutex<SimulatedPad>>;

/// Locks a pad, taking it back from a test thread that failed while holding it.
pub fn lock(pad: &SharedPad) -> MutexGuard<'_, SimulatedPad> {
    pad.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Simulated pads plugged in and pulled out, and the handles opened on them.
#[derive(Default)]
pub struct SimulatedPads {
    attached: BTreeMap<u32, SharedPad>,
    open: HashMap<Handle, (u32, SharedPad)>,
    next_id: u32,
    next_handle: Handle,
    changed: bool,
    pub opens: i32,
    pub closes: i32,
    pub initialized: bool,
}

impl SimulatedPads {
    pub fn new() -> SimulatedPads {
        SimulatedPads { next_id: 1, next_handle: 0x1000, ..Default::default() }
    }

    pub fn connect(&mut self, pad: SharedPad) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.attached.insert(id, pad);
        self.changed = true;
        id
    }

    pub fn disconnect(&mut self, pad: &SharedPad) {
        self.attached.retain(|_, attached| !Arc::ptr_eq(attached, pad));
        self.changed = true;
    }

    /// The ids of the pads attached, in the order they were plugged in.
    pub fn attached_ids(&self) -> Vec<u32> {
        self.attached.keys().copied().collect()
    }

    /// The first pad plugged in that is still attached.
    pub fn first(&self) -> Option<&SharedPad> {
        self.attached.values().next()
    }

    pub fn open_handles(&self) -> usize {
        self.open.len()
    }

    fn pad(&self, handle: Handle) -> Option<&SharedPad> {
        self.open.get(&handle).map(|(_, pad)| pad)
    }
}

impl PadDevices for SimulatedPads {
    fn init(&mut self) -> bool {
        self.initialized = true;
        true
    }

    fn quit(&mut self) {
        self.initialized = false;
    }

    fn attached(&mut self) -> Vec<u32> {
        self.attached.keys().copied().collect()
    }

    fn open(&mut self, id: u32) -> Handle {
        let Some(pad) = self.attached.get(&id) else { return 0 };
        let handle = self.next_handle;
        self.next_handle += 1;
        self.open.insert(handle, (id, pad.clone()));
        self.opens += 1;
        handle
    }

    fn close(&mut self, pad: Handle) {
        if self.open.remove(&pad).is_some() {
            self.closes += 1;
        }
    }

    fn is_attached(&mut self, pad: Handle) -> bool {
        self.open.get(&pad).is_some_and(|(id, _)| self.attached.contains_key(id))
    }

    fn devices_changed(&mut self) -> bool {
        std::mem::take(&mut self.changed)
    }

    fn update(&mut self) {}

    fn button(&mut self, pad: Handle, button: i32) -> bool {
        self.pad(pad).is_some_and(|p| lock(p).is_held(button))
    }

    fn axis(&mut self, pad: Handle, axis: i32) -> i16 {
        self.pad(pad).map_or(0, |p| lock(p).axis(axis))
    }

    fn name(&mut self, pad: Handle) -> Option<String> {
        self.pad(pad).map(|p| lock(p).name.clone())
    }

    fn kind(&mut self, pad: Handle) -> i32 {
        self.pad(pad).map_or(0, |p| lock(p).kind)
    }

    fn label(&mut self, _pad: Handle, _button: i32) -> i32 {
        0
    }

    fn guid(&mut self, pad: Handle) -> String {
        self.pad(pad).map_or_else(String::new, |p| lock(p).reported_guid())
    }

    fn path(&mut self, pad: Handle) -> Option<String> {
        self.pad(pad).and_then(|p| lock(p).path.clone())
    }

    fn set_player_index(&mut self, pad: Handle, index: i32) {
        if let Some(p) = self.pad(pad) {
            lock(p).player_index = index;
        }
    }
}

/// A set shared between the test that plugs pads in and the manager that opens them.
pub type SharedSet = Arc<Mutex<SimulatedPads>>;

/// Locks a set, as `lock` does a pad.
pub fn lock_set(set: &SharedSet) -> MutexGuard<'_, SimulatedPads> {
    set.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl PadDevices for SharedSet {
    fn init(&mut self) -> bool {
        lock_set(self).init()
    }
    fn quit(&mut self) {
        lock_set(self).quit()
    }
    fn attached(&mut self) -> Vec<u32> {
        lock_set(self).attached()
    }
    fn open(&mut self, id: u32) -> Handle {
        lock_set(self).open(id)
    }
    fn close(&mut self, pad: Handle) {
        lock_set(self).close(pad)
    }
    fn is_attached(&mut self, pad: Handle) -> bool {
        lock_set(self).is_attached(pad)
    }
    fn devices_changed(&mut self) -> bool {
        lock_set(self).devices_changed()
    }
    fn update(&mut self) {}
    fn button(&mut self, pad: Handle, button: i32) -> bool {
        lock_set(self).button(pad, button)
    }
    fn axis(&mut self, pad: Handle, axis: i32) -> i16 {
        lock_set(self).axis(pad, axis)
    }
    fn name(&mut self, pad: Handle) -> Option<String> {
        lock_set(self).name(pad)
    }
    fn kind(&mut self, pad: Handle) -> i32 {
        lock_set(self).kind(pad)
    }
    fn label(&mut self, pad: Handle, button: i32) -> i32 {
        lock_set(self).label(pad, button)
    }
    fn guid(&mut self, pad: Handle) -> String {
        lock_set(self).guid(pad)
    }
    fn path(&mut self, pad: Handle) -> Option<String> {
        lock_set(self).path(pad)
    }
    fn set_player_index(&mut self, pad: Handle, index: i32) {
        lock_set(self).set_player_index(pad, index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shared(name: &str) -> SharedPad {
        let mut pad = SimulatedPad::new(Some(format!("/dev/input/{name}")));
        pad.name = name.to_string();
        Arc::new(Mutex::new(pad))
    }

    #[test]
    fn a_handle_reads_its_pad_until_it_is_closed_and_a_pulled_pad_is_no_longer_attached() {
        let mut set = SimulatedPads::new();
        let pad = shared("First");
        let id = set.connect(pad.clone());
        assert!(set.devices_changed());
        assert!(!set.devices_changed());
        let handle = set.open(id);
        assert_eq!(handle, 0x1000);
        lock(&pad).press(3);
        assert!(set.button(handle, 3));
        assert_eq!(set.name(handle).as_deref(), Some("First"));
        set.disconnect(&pad);
        assert!(!set.is_attached(handle));
        assert!(set.button(handle, 3), "a pulled pad's handle still reads until it is closed");
        set.close(handle);
        assert!(!set.button(handle, 3));
        assert_eq!((set.opens, set.closes, set.open_handles()), (1, 1, 0));
        assert_eq!(set.open(id), 0);
    }

    #[test]
    fn a_pad_without_a_guid_reports_its_names_md5_and_an_axis_rounds_to_even() {
        let mut pad = SimulatedPad::new(None);
        assert_eq!(pad.reported_guid(), "3797373e514244ffe4f5038b59b0cc3f");
        pad.guid = Some("own".into());
        assert_eq!(pad.reported_guid(), "own");
        pad.set_axis(0, 1.0);
        pad.set_axis(1, -2.0);
        pad.set_axis(2, f64::NAN);
        pad.set_axis(3, 0.5 / 32767.0 * 3.0);
        assert_eq!((pad.axis(0), pad.axis(1), pad.axis(2), pad.axis(3)), (32767, -32767, 0, 2));
    }

    #[test]
    fn the_guid_is_written_in_sdls_byte_order() {
        let mut data = [0u8; 16];
        data[0] = 0x03;
        data[15] = 0xab;
        assert_eq!(guid_text(sdl::Guid { data }), "030000000000000000000000000000ab");
    }
}
