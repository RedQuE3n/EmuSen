//! The per-frame calls as thin wrappers: size-prefixed structs and caller-owned buffers, so a frame allocates nothing
//! here; only a failure's words do (EmuSen_Qt.md §1.1.1).

use crate::error::Result;
use crate::raw;
use crate::runtime::Runtime;
use crate::shapes::SessionId;
use std::ffi::c_void;
use std::ptr::NonNull;

/// What `frame` and `rewind_step` fill, emusen_crystal_frame_info as the header lays it out.
pub type FrameInfo = raw::emusen_crystal_frame_info;

/// What `raw_copy` fills.
pub type RawInfo = raw::emusen_crystal_raw_info;

/// A frame info with its size set, as its allocator must.
pub fn frame_info() -> FrameInfo {
    FrameInfo { size: std::mem::size_of::<FrameInfo>() as u32, ..Default::default() }
}

/// A raw info with its size set.
pub fn raw_info() -> RawInfo {
    RawInfo { size: std::mem::size_of::<RawInfo>() as u32, ..Default::default() }
}

impl Runtime {
    /// One frame on the session's frame thread: the input held applied, the machine advanced, `info` filled.
    pub fn frame(&self, session: SessionId, info: &mut FrameInfo) -> Result<()> {
        self.status(unsafe { self.lib().emusen_crystal_frame(self.rt(), session, info) }.into()).map(drop)
    }

    /// One frame advance: the session paused, and the next frame runs once.
    pub fn frame_advance(&self, session: SessionId) -> Result<()> {
        self.status(unsafe { self.lib().emusen_crystal_frame_advance(self.rt(), session) }.into()).map(drop)
    }

    /// The picture as RGBA8888 into `out`; its whole length, written only when `out` holds it.
    pub fn picture_copy(&self, session: SessionId, out: &mut [u8]) -> Result<usize> {
        self.status(unsafe { self.lib().emusen_crystal_picture_copy(self.rt(), session, out.as_mut_ptr(), out.len()) }).map(|n| n as usize)
    }

    /// The machine's lend, emusen_serenity_lend, for the presenter to borrow; asked again after each frame.
    pub fn picture_lend(&self, session: SessionId) -> Option<NonNull<c_void>> {
        NonNull::new(unsafe { self.lib().emusen_crystal_picture_lend(self.rt(), session) } as *mut c_void)
    }

    /// The machine's ring, emusen_serenity_ring, for the presenter to borrow; asked again after each frame.
    pub fn picture_ring(&self, session: SessionId) -> Option<NonNull<c_void>> {
        NonNull::new(unsafe { self.lib().emusen_crystal_picture_ring(self.rt(), session) } as *mut c_void)
    }

    /// The encoding of the picture before its palette kept from the next frame; none keeps none.
    pub fn raw_select(&self, session: SessionId, encoding: Option<&str>) -> Result<()> {
        let (e, el) = crate::out::text(encoding);
        self.status(unsafe { self.lib().emusen_crystal_raw_select(self.rt(), session, e, el) }.into()).map(drop)
    }

    /// The last frame's picture before its palette into `out`, its shape into `info`; none where none is kept.
    pub fn raw_copy(&self, session: SessionId, info: &mut RawInfo, out: &mut [u8]) -> Result<Option<usize>> {
        match self.status(unsafe { self.lib().emusen_crystal_raw_copy(self.rt(), session, info, out.as_mut_ptr(), out.len()) }) {
            Ok(n) => Ok(Some(n as usize)),
            Err(e) if e.is_absent() => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Up to `out.len() / 2` stereo frames drained into `out`; the frames written and their rate.
    pub fn audio_drain(&self, session: SessionId, out: &mut [i16]) -> Result<(usize, u32)> {
        let mut rate = 0u32;
        let n = unsafe { self.lib().emusen_crystal_audio_drain(self.rt(), session, out.as_mut_ptr(), out.len() / 2, &mut rate) };
        self.status(n).map(|n| (n as usize, rate))
    }

    /// The stereo frames ready, none drained.
    pub fn audio_ready(&self, session: SessionId) -> Result<usize> {
        let n = unsafe { self.lib().emusen_crystal_audio_drain(self.rt(), session, std::ptr::null_mut(), 0, std::ptr::null_mut()) };
        self.status(n).map(|n| n as usize)
    }

    /// While the session is held paused: true when a change was applied or run control asked to run in time.
    pub fn wait(&self, session: SessionId, timeout_ms: u32) -> Result<bool> {
        self.status(unsafe { self.lib().emusen_crystal_wait(self.rt(), session, timeout_ms) }.into()).map(|n| n == 1)
    }

    /// The last frame's phase times in milliseconds into `out`; how many there are.
    pub fn phases(&self, session: SessionId, out: &mut [f64]) -> Result<usize> {
        self.status(unsafe { self.lib().emusen_crystal_phases(self.rt(), session, out.as_mut_ptr(), out.len()) }).map(|n| n as usize)
    }

    /// A canonical control pressed or released on a port.
    pub fn input_button(&self, session: SessionId, port: u32, control: u32, pressed: bool) -> Result<()> {
        self.status(unsafe { self.lib().emusen_crystal_input_button(self.rt(), session, port, control, pressed as u32) }.into()).map(drop)
    }

    /// An axis: sticks -1 to 1, right and down positive; triggers 0 to 1.
    pub fn input_axis(&self, session: SessionId, port: u32, axis: u32, value: f64) -> Result<()> {
        self.status(unsafe { self.lib().emusen_crystal_input_axis(self.rt(), session, port, axis, value) }.into()).map(drop)
    }

    /// A controller plugged into a port or pulled out.
    pub fn input_connected(&self, session: SessionId, port: u32, connected: bool) -> Result<()> {
        self.status(unsafe { self.lib().emusen_crystal_input_connected(self.rt(), session, port, connected as u32) }.into()).map(drop)
    }

    /// A port's motors, strong and weak, 0 to 65535.
    pub fn input_rumble(&self, session: SessionId, port: u32) -> Result<(u16, u16)> {
        let (mut strong, mut weak) = (0u16, 0u16);
        self.status(unsafe { self.lib().emusen_crystal_input_rumble(self.rt(), session, port, &mut strong, &mut weak) }.into())?;
        Ok((strong, weak))
    }

    /// The player's rumble switch turned while the session runs.
    pub fn input_rumble_switch(&self, session: SessionId, on: bool) -> Result<()> {
        self.status(unsafe { self.lib().emusen_crystal_input_rumble_switch(self.rt(), session, on as u32) }.into()).map(drop)
    }

    /// The three speeds in percent and EMUSEN_CRYSTAL_SPEED_* flags.
    pub fn speed_set(&self, session: SessionId, percent: u32, turbo_percent: u32, slow_percent: u32, flags: u32) -> Result<()> {
        self.status(unsafe { self.lib().emusen_crystal_speed_set(self.rt(), session, percent, turbo_percent, slow_percent, flags) }.into())
            .map(drop)
    }

    /// Which speed applies, EMUSEN_CRYSTAL_SPEED_NORMAL, _TURBO or _SLOW.
    pub fn speed_mode(&self, session: SessionId, mode: u32) -> Result<()> {
        self.status(unsafe { self.lib().emusen_crystal_speed_mode(self.rt(), session, mode) }.into()).map(drop)
    }

    /// In place of a frame: up to `count` snapshots back; how many it took, `info` filled for the picture on show.
    pub fn rewind_step(&self, session: SessionId, count: u32, info: &mut FrameInfo) -> Result<u32> {
        self.status(unsafe { self.lib().emusen_crystal_rewind_step(self.rt(), session, count, info) }).map(|n| n as u32)
    }
}
