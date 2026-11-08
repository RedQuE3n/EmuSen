//! Endymion's pads: every connected pad opened, hot-plugged and let go, read through the stick and trigger rules. The
//! host seats what a start or a poll opened; a pad is known by its key. Simulated pads stand in for SDL's in tests.

use crate::error::Result;
use crate::out::text;
use crate::platform::Platform;
use crate::raw;
use std::ptr::NonNull;
use std::sync::Arc;

/// What a start, a poll or a change of devices did to one pad.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PadEvent {
    pub opened: bool,
    /// For an opened pad: false for one present at start, which the host does not announce.
    pub announce: bool,
    pub key: u64,
}

/// The pads' numbers: what `get`, `text` and `axis` read, and the settings.
pub mod consts {
    pub use crate::raw::{
        EMUSEN_ENDYMION_AXIS_PAD as AXIS_PAD, EMUSEN_ENDYMION_AXIS_RAW as AXIS_RAW, EMUSEN_ENDYMION_FAMILY_GENERIC as FAMILY_GENERIC,
        EMUSEN_ENDYMION_FAMILY_NINTENDO as FAMILY_NINTENDO, EMUSEN_ENDYMION_FAMILY_PLAYSTATION as FAMILY_PLAYSTATION,
        EMUSEN_ENDYMION_FAMILY_XBOX as FAMILY_XBOX, EMUSEN_ENDYMION_PADS_ANY_PRESSED as ANY_PRESSED,
        EMUSEN_ENDYMION_PADS_AXIS_VALUE as AXIS_VALUE, EMUSEN_ENDYMION_PADS_FAMILY as FAMILY,
        EMUSEN_ENDYMION_PADS_FRONTEND_COUNT as FRONTEND_COUNT, EMUSEN_ENDYMION_PADS_GUID as GUID, EMUSEN_ENDYMION_PADS_ID as ID,
        EMUSEN_ENDYMION_PADS_IS_OPEN as IS_OPEN, EMUSEN_ENDYMION_PADS_KIND as KIND, EMUSEN_ENDYMION_PADS_LABEL as LABEL,
        EMUSEN_ENDYMION_PADS_NAME as NAME, EMUSEN_ENDYMION_PADS_PATH as PATH, EMUSEN_ENDYMION_PADS_RAW_PRESSED as RAW_PRESSED,
        EMUSEN_ENDYMION_PADS_STARTED as STARTED, EMUSEN_ENDYMION_PADS_VENDOR_PRODUCT as VENDOR_PRODUCT,
        EMUSEN_ENDYMION_SETTING_ANALOG_DEADZONE as SETTING_ANALOG_DEADZONE,
        EMUSEN_ENDYMION_SETTING_FIRST_CONTROLLER_ONLY as SETTING_FIRST_CONTROLLER_ONLY,
        EMUSEN_ENDYMION_SETTING_LEFT_STICK_IS_ANALOG as SETTING_LEFT_STICK_IS_ANALOG,
        EMUSEN_ENDYMION_SETTING_STICK_AS_DPAD as SETTING_STICK_AS_DPAD, EMUSEN_ENDYMION_SETTING_STICK_DEADZONE as SETTING_STICK_DEADZONE,
        EMUSEN_ENDYMION_SIM_PAD_KIND as SIM_PAD_KIND, EMUSEN_ENDYMION_SIM_PAD_NAME as SIM_PAD_NAME,
        EMUSEN_ENDYMION_SIM_PAD_RUMBLE as SIM_PAD_RUMBLE,
    };
}

/// The most pad events one start or poll hands back; more wait for `take`.
const EVENTS: usize = 16;

/// Every connected pad, on SDL's devices or a simulated set's.
pub struct Pads {
    platform: Arc<Platform>,
    pads: NonNull<raw::emusen_endymion_pads>,
}

// The header: a pads handle locks itself and may be used from any thread.
unsafe impl Send for Pads {}
unsafe impl Sync for Pads {}

impl Pads {
    /// SDL's pads; SDL must have been lent.
    pub fn sdl(platform: &Arc<Platform>) -> Result<Pads> {
        let pads =
            unsafe { platform.lib.emusen_endymion_pads_new(raw::EMUSEN_ENDYMION_DEVICES_SDL, std::ptr::null(), std::ptr::null(), 0) };
        Pads::made(platform, pads)
    }

    /// A simulated set's pads, as a test plugs them in; the handle holds its own reference to the set.
    pub fn simulated(platform: &Arc<Platform>, set: &SimSet) -> Result<Pads> {
        let pads =
            unsafe { platform.lib.emusen_endymion_pads_new(raw::EMUSEN_ENDYMION_DEVICES_SIMULATED, set.set.as_ptr(), std::ptr::null(), 0) };
        Pads::made(platform, pads)
    }

    /// The platform library the pads were made on.
    pub fn platform(&self) -> &Arc<Platform> {
        &self.platform
    }

    fn made(platform: &Arc<Platform>, pads: *mut raw::emusen_endymion_pads) -> Result<Pads> {
        let pads = NonNull::new(pads).ok_or_else(|| crate::error::status(raw::EMUSEN_PLATFORM_NOT_SUPPORTED, platform.last_error()))?;
        Ok(Pads { platform: platform.clone(), pads })
    }

    fn events(&self, call: impl Fn(*mut raw::emusen_endymion_pad_event, usize) -> i64) -> Result<Vec<PadEvent>> {
        let mut out = [raw::emusen_endymion_pad_event::default(); EVENTS];
        let n = self.platform.status(call(out.as_mut_ptr(), out.len()))? as usize;
        Ok(out[..n.min(EVENTS)].iter().map(|e| PadEvent { opened: e.kind == 0, announce: e.announce != 0, key: e.key }).collect())
    }

    /// Starts the devices; the pads opened.
    pub fn start(&self) -> Result<Vec<PadEvent>> {
        self.events(|o, c| unsafe { self.platform.lib.emusen_endymion_pads_start(self.pads.as_ptr(), o, c) })
    }

    /// One poll; the pads opened and closed, none before the devices start.
    pub fn poll(&self) -> Result<Vec<PadEvent>> {
        match self.events(|o, c| unsafe { self.platform.lib.emusen_endymion_pads_poll(self.pads.as_ptr(), o, c) }) {
            Err(e) if crate::error::absent(&e) => Ok(Vec::new()),
            other => other,
        }
    }

    /// SDL's pads of these ids alone from the next start or poll, or every pad for none: a test's, so that it opens SDL's
    /// virtual pads and none on the desk.
    pub fn show_only(&self, only: Option<&[u32]>) -> Result<()> {
        let (p, n) = only.map_or((std::ptr::null(), 0), |ids| (ids.as_ptr(), ids.len()));
        self.platform.status(unsafe { self.platform.lib.emusen_endymion_pads_show_only(self.pads.as_ptr(), p, n) }.into()).map(drop)
    }

    /// The events a start or poll had no room for.
    pub fn take(&self) -> Result<Vec<PadEvent>> {
        self.events(|o, c| unsafe { self.platform.lib.emusen_endymion_pads_take(self.pads.as_ptr(), o, c) })
    }

    /// The devices brought up to date, after seating what a poll opened.
    pub fn update(&self) -> Result<()> {
        self.platform.status(unsafe { self.platform.lib.emusen_endymion_pads_update(self.pads.as_ptr()) }.into()).map(drop)
    }

    /// Every pad closed.
    pub fn close_all(&self) -> Result<()> {
        self.platform.status(unsafe { self.platform.lib.emusen_endymion_pads_close_all(self.pads.as_ptr()) }.into()).map(drop)
    }

    /// The open pads' keys, first controller first.
    pub fn open(&self) -> Result<Vec<u64>> {
        let mut keys = [0u64; EVENTS];
        let n = self
            .platform
            .status(unsafe { self.platform.lib.emusen_endymion_pads_open(self.pads.as_ptr(), keys.as_mut_ptr(), keys.len()) })?;
        Ok(keys[..(n as usize).min(EVENTS)].to_vec())
    }

    /// A number of a pad's, consts' ID to VENDOR_PRODUCT, with its argument.
    pub fn get(&self, key: u64, which: u32, arg: i32) -> Result<i64> {
        self.platform.status(unsafe { self.platform.lib.emusen_endymion_pads_get(self.pads.as_ptr(), key, which, arg) })
    }

    /// A pad's GUID, path or name; none for a path it has not.
    pub fn text(&self, key: u64, which: u32) -> Result<Option<String>> {
        let bytes =
            self.platform.read(|o, l| unsafe { self.platform.lib.emusen_endymion_pads_text(self.pads.as_ptr(), key, which, o, l) })?;
        Ok(bytes.map(|b| String::from_utf8_lossy(&b).into_owned()))
    }

    /// An axis by SDL's numbering (AXIS_RAW) or the pad's (AXIS_PAD).
    pub fn axis(&self, key: u64, which: u32, axis: u32) -> Result<f64> {
        let mut v = 0.0;
        self.platform
            .status(unsafe { self.platform.lib.emusen_endymion_pads_axis(self.pads.as_ptr(), key, which, axis, &mut v) }.into())?;
        Ok(v)
    }

    /// A PadButton as the game hears it, `bound` the player's SDL button or -1.
    pub fn pressed(&self, key: u64, button: u32, bound: i32) -> Result<bool> {
        self.platform
            .status(unsafe { self.platform.lib.emusen_endymion_pads_pressed(self.pads.as_ptr(), key, button, bound) }.into())
            .map(|n| n != 0)
    }

    /// The player number the pad lights.
    pub fn set_player_index(&self, key: u64, index: i32) -> Result<()> {
        self.platform
            .status(unsafe { self.platform.lib.emusen_endymion_pads_set_player_index(self.pads.as_ptr(), key, index) }.into())
            .map(drop)
    }

    /// The pad's motors for 100 ms.
    pub fn rumble(&self, key: u64, strong: u16, weak: u16) -> Result<()> {
        self.platform
            .status(unsafe { self.platform.lib.emusen_endymion_pads_rumble(self.pads.as_ptr(), key, strong, weak) }.into())
            .map(drop)
    }

    /// A setting, consts' SETTING_*.
    pub fn set_setting(&self, which: u32, value: f64) -> Result<()> {
        self.platform
            .status(unsafe { self.platform.lib.emusen_endymion_pads_set_setting(self.pads.as_ptr(), which, value) }.into())
            .map(drop)
    }

    /// A setting's value.
    pub fn setting(&self, which: u32) -> Result<f64> {
        let mut v = 0.0;
        self.platform.status(unsafe { self.platform.lib.emusen_endymion_pads_setting(self.pads.as_ptr(), which, &mut v) }.into())?;
        Ok(v)
    }
}

impl Drop for Pads {
    fn drop(&mut self) {
        unsafe {
            self.platform.lib.emusen_endymion_pads_close_all(self.pads.as_ptr());
            self.platform.lib.emusen_endymion_pads_free(self.pads.as_ptr());
        }
    }
}

/// A simulated pad, which a test presses and plugs into a set in a real pad's place; a set holds its own reference.
pub struct SimPad {
    platform: Arc<Platform>,
    pad: NonNull<raw::emusen_endymion_sim_pad>,
}

unsafe impl Send for SimPad {}

impl SimPad {
    /// A pad named "Simulated pad", at `path` or none.
    pub fn new(platform: &Arc<Platform>, path: Option<&str>) -> Result<SimPad> {
        let (p, pl) = text(path);
        let pad = unsafe { platform.lib.emusen_endymion_sim_pad_new(p, pl) };
        let pad = NonNull::new(pad).ok_or_else(|| crate::error::status(raw::EMUSEN_PLATFORM_NULL, platform.last_error()))?;
        Ok(SimPad { platform: platform.clone(), pad })
    }

    /// An SDL_GamepadButton pressed or released.
    pub fn press(&self, button: i32, down: bool) -> Result<()> {
        self.platform
            .status(unsafe { self.platform.lib.emusen_endymion_sim_pad_press(self.pad.as_ptr(), button, down as u32) }.into())
            .map(drop)
    }

    /// An SDL_GamepadAxis set, -1 to 1 for a stick, 0 to 1 for a trigger.
    pub fn set_axis(&self, axis: i32, value: f64) -> Result<()> {
        self.platform.status(unsafe { self.platform.lib.emusen_endymion_sim_pad_set_axis(self.pad.as_ptr(), axis, value) }.into()).map(drop)
    }

    /// A number of the pad's set, EMUSEN_ENDYMION_SIM_PAD_* (its SDL type, its vendor and product, its player light).
    pub fn set(&self, which: u32, value: i32) -> Result<()> {
        self.platform.status(unsafe { self.platform.lib.emusen_endymion_sim_pad_set(self.pad.as_ptr(), which, value) }.into()).map(drop)
    }

    /// A text of the pad's set, EMUSEN_ENDYMION_SIM_PAD_NAME, _GUID or _PATH.
    pub fn set_text(&self, which: u32, value: &str) -> Result<()> {
        let n = unsafe { self.platform.lib.emusen_endymion_sim_pad_set_text(self.pad.as_ptr(), which, value.as_ptr(), value.len()) };
        self.platform.status(n.into()).map(drop)
    }

    /// A number of the pad's, EMUSEN_ENDYMION_SIM_PAD_*.
    pub fn get(&self, which: u32) -> Result<i32> {
        let mut v = 0;
        self.platform.status(unsafe { self.platform.lib.emusen_endymion_sim_pad_get(self.pad.as_ptr(), which, &mut v) }.into())?;
        Ok(v)
    }
}

impl Drop for SimPad {
    fn drop(&mut self) {
        unsafe { self.platform.lib.emusen_endymion_sim_pad_free(self.pad.as_ptr()) };
    }
}

/// A set of simulated pads, a device layer in SDL's place.
pub struct SimSet {
    platform: Arc<Platform>,
    set: NonNull<raw::emusen_endymion_sim_set>,
}

unsafe impl Send for SimSet {}
unsafe impl Sync for SimSet {}

impl SimSet {
    /// An empty set.
    pub fn new(platform: &Arc<Platform>) -> Result<SimSet> {
        let set = unsafe { platform.lib.emusen_endymion_sim_set_new() };
        let set = NonNull::new(set).ok_or_else(|| crate::error::status(raw::EMUSEN_PLATFORM_NULL, platform.last_error()))?;
        Ok(SimSet { platform: platform.clone(), set })
    }

    /// A pad plugged in; its id, from 1.
    pub fn connect(&self, pad: &SimPad) -> Result<u64> {
        self.platform
            .status(unsafe { self.platform.lib.emusen_endymion_sim_set_connect(self.set.as_ptr(), pad.pad.as_ptr()) })
            .map(|n| n as u64)
    }

    /// A pad pulled out.
    pub fn disconnect(&self, pad: &SimPad) -> Result<()> {
        self.platform
            .status(unsafe { self.platform.lib.emusen_endymion_sim_set_disconnect(self.set.as_ptr(), pad.pad.as_ptr()) }.into())
            .map(drop)
    }
}

impl Drop for SimSet {
    fn drop(&mut self) {
        unsafe { self.platform.lib.emusen_endymion_sim_set_free(self.set.as_ptr()) };
    }
}
