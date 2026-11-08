//! Endymion's router: the players' pads, as read, routed to a running game's controller ports, each change once and in
//! the order the core is told them (emusen_platform.h, the router; EmuSen_Input.md §8.5). The host reads the pads and
//! applies the changes; the router keeps what was sent.

use crate::error::Result;
use crate::platform::Platform;
use crate::raw;
use std::ptr::NonNull;
use std::sync::Arc;

/// One change for the core: a port's controller (CONNECTED), a PadButton (BUTTON) or a PadAxis (AXIS).
pub type Change = raw::emusen_endymion_change;

/// The router's numbers: the kinds of change and the settings.
pub mod consts {
    pub use crate::raw::{
        EMUSEN_ENDYMION_CHANGE_AXIS as AXIS, EMUSEN_ENDYMION_CHANGE_BUTTON as BUTTON, EMUSEN_ENDYMION_CHANGE_CONNECTED as CONNECTED,
        EMUSEN_ENDYMION_ROUTER_KEYBOARD_PLAYER as KEYBOARD_PLAYER, EMUSEN_ENDYMION_ROUTER_MIRROR_PLAYER1_TO_PLAYER2 as MIRROR,
        EMUSEN_ENDYMION_ROUTER_PORTS as PORTS,
    };
}

/// A game's router. One thread at a time.
pub struct Router {
    platform: Arc<Platform>,
    router: NonNull<raw::emusen_endymion_router>,
}

// The router holds no thread's state; one thread uses it at a time.
unsafe impl Send for Router {}

impl Router {
    pub fn new(platform: &Arc<Platform>) -> Result<Router> {
        let router = unsafe { platform.lib.emusen_endymion_router_new() };
        let router = NonNull::new(router).ok_or_else(|| crate::error::status(raw::EMUSEN_PLATFORM_NOT_SUPPORTED, platform.last_error()))?;
        Ok(Router { platform: platform.clone(), router })
    }

    pub fn get(&self, which: u32) -> Result<i32> {
        self.platform.status(unsafe { self.platform.lib.emusen_endymion_router_get(self.router.as_ptr(), which) }.into()).map(|n| n as i32)
    }

    pub fn set(&self, which: u32, value: i32) -> Result<()> {
        self.platform.status(unsafe { self.platform.lib.emusen_endymion_router_set(self.router.as_ptr(), which, value) }.into()).map(drop)
    }

    /// A game just started: its ports and the axes it reads, by PadAxis, nothing held and nothing sent.
    pub fn reset(&self, ports: i32, axes: &[u32]) -> Result<()> {
        let r = unsafe { self.platform.lib.emusen_endymion_router_reset(self.router.as_ptr(), ports, axes.as_ptr(), axes.len()) };
        self.platform.status(r.into()).map(drop)
    }

    /// The pads as read, `held` a mask by PadButton for each player from player 1 and `axes` each player's read axes in
    /// the reset's order; every change into `out`, which keeps its room between calls.
    pub fn poll(&self, held: &[u16], axes: &[f64], keyboard: u32, seated: u32, out: &mut Vec<Change>) -> Result<()> {
        let lib = &self.platform.lib;
        self.changes(out, |o, cap| unsafe {
            lib.emusen_endymion_router_poll(self.router.as_ptr(), held.as_ptr(), held.len(), axes.as_ptr(), axes.len(), keyboard, seated, o, cap)
        })
    }

    /// A change call's answer into `out`; what had no room is taken after.
    fn changes(&self, out: &mut Vec<Change>, call: impl FnOnce(*mut Change, usize) -> i64) -> Result<()> {
        out.clear();
        let cap = out.capacity().max(64);
        out.resize(cap, Change::default());
        let made = self.platform.status(call(out.as_mut_ptr(), out.len()))? as usize;
        if made > cap {
            out.resize(made, Change::default());
            let r = unsafe { self.platform.lib.emusen_endymion_router_take(self.router.as_ptr(), out.as_mut_ptr(), out.len()) };
            self.platform.status(r)?;
        }
        out.truncate(made);
        Ok(())
    }
}

impl Drop for Router {
    fn drop(&mut self) {
        unsafe { self.platform.lib.emusen_endymion_router_free(self.router.as_ptr()) };
    }
}
