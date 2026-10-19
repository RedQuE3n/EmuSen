//! The frame's (b) points, EmuSen_Backend.md §29.4, in the shape `emusen-crystal-debug`'s `Debugger` gives them:
//! what the debugger, the cheat registry and the recorder do at each, called by the frame in its order. Part (a) ran
//! every session on a debugger whose calls did nothing, so the order was whole before part (b) filled each point; since
//! part (c) each session's is `debug::SessionDebugger`, the order unmoved (§29.21).

use emusen_core_host::Machine;

/// How a frame the debugger ran ended: to its end, or halted on a processor (machine info's id) at an address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ran {
    Completed,
    Halted { processor: u32, address: u32 },
}

pub trait Debugger: Send {
    /// A machine made, at open or made again.
    fn attach(&mut self, _m: &mut Machine) {}

    /// Machine info read again after its event.
    fn info_changed(&mut self, _m: &mut Machine) {}

    /// The session closed.
    fn detach(&mut self) {}

    /// The cheats sent again before the next frame whatever their version, as after a state load.
    fn cheats_stale(&mut self) {}

    /// The cheats' patch table and pokes brought up to date before the frame.
    fn refresh_cheats(&mut self, _m: &mut Machine) {}

    /// Whether the frame is the debugger's to run: breakpoints armed.
    fn armed(&self) -> bool {
        false
    }

    /// The frame run under the debugger, failing as `Machine::advance` fails.
    fn run_frame(&mut self, m: &mut Machine, _resuming: bool) -> Result<Ran, (i32, u64)> {
        m.advance().map(|()| Ran::Completed)
    }

    /// The frame-end work of a completed frame: the frame log, the host's cheat pokes unless the frame holds them, the
    /// freezes and the breakpoints' frame, in that order.
    fn frame_end(&mut self, _m: &mut Machine) {}

    /// The snapshots refreshed after a frame, a halt, a reset, a state load or a rewind step.
    fn refresh(&mut self, _m: &mut Machine, _last_frame_ms: f64) {}

    /// The runtime's lines the debugger said since the last call, moved into `lines`.
    fn take_said(&mut self, _lines: &mut Vec<String>) {}

    /// Whether the recorder is recording, which draws every frame.
    fn recording(&self) -> bool {
        false
    }

    /// The recorder's picture of a drawn frame: RGBA rows, the width and the height as shown.
    fn capture(&mut self, _frame: i64, _rgba: &[u8], _width: i32, _height: i32) {}
}
