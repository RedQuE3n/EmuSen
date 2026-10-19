//! A session's debugger at the frame's (b) points: `emusen-crystal-debug`'s `Debugger` and its cheat registry, and the
//! recorder, behind the seam part (a) wrote the frame against (EmuSen_Backend.md §29.4, §29.30). The session service
//! reaches the debugger itself, through `d`.

use emusen_core_host::Machine;
use emusen_crystal_debug::Debugger as Inner;
use emusen_recorder::Recorder;

use crate::seam::{Debugger, Ran};

/// One session's debugger and recorder.
pub struct SessionDebugger {
    pub d: Inner,
    pub recorder: Recorder,
    /// The engine's name the recording's folder and ledger carry, `IDebugTarget.CoreName`.
    pub core_name: String,
}

impl Default for SessionDebugger {
    fn default() -> SessionDebugger {
        SessionDebugger { d: Inner::new(), recorder: Recorder::new(), core_name: String::new() }
    }
}

impl Debugger for SessionDebugger {
    fn attach(&mut self, m: &mut Machine) {
        self.d.attach(m);
    }

    fn info_changed(&mut self, m: &mut Machine) {
        self.d.info_changed(m);
    }

    fn detach(&mut self) {
        self.d.detach();
    }

    fn cheats_stale(&mut self) {
        self.d.cheats_stale();
    }

    fn refresh_cheats(&mut self, m: &mut Machine) {
        self.d.refresh_cheats(m);
    }

    fn armed(&self) -> bool {
        self.d.armed()
    }

    fn run_frame(&mut self, m: &mut Machine, resuming: bool) -> Result<Ran, (i32, u64)> {
        self.d.run_frame(m, resuming).map(|ran| match ran {
            emusen_crystal_debug::Ran::Completed => Ran::Completed,
            emusen_crystal_debug::Ran::Halted { processor, address } => Ran::Halted { processor, address },
        })
    }

    fn frame_end(&mut self, m: &mut Machine) {
        self.d.frame_end(m);
    }

    fn refresh(&mut self, m: &mut Machine, last_frame_ms: f64) {
        self.d.refresh(m, last_frame_ms);
    }

    fn take_said(&mut self, lines: &mut Vec<String>) {
        self.d.take_said(lines);
    }

    fn recording(&self) -> bool {
        self.recorder.is_recording()
    }

    fn capture(&mut self, frame: i64, rgba: &[u8], width: i32, height: i32) {
        let name = &self.core_name;
        self.recorder.capture(frame, name, |path| {
            if width > 0 && height > 0 {
                let _ = crate::png::write(&path.to_string_lossy(), rgba, width as u32, height as u32);
            }
        });
    }
}
