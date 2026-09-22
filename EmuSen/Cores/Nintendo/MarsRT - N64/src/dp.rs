//! The display processor's command interface, the C# `DpInterface`, and the words a snapshot carries.

use crate::rdp::Rdp;
use crate::state::{State, StateError, StateReader, StateResult, StateWriter};

/// `DpInterface.SnapshotWords`: a snapshot's tail always holds this many words, the unused ones zero.
pub const SNAPSHOT_WORDS: usize = 1 << 15;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct DpInterface {
    pub processor: Rdp,
    pub current: u32,
    pub end: u32,
    pub freeze: bool,
    pub running: bool,
    pub start: u32,
    pub start_valid: bool,
    pub xbus: bool,
    /// The words handed over and not yet run, which only a snapshot carries (`WritePending`); `[SkipInState]` in C#.
    pub pending: Vec<u64>,
}

impl State for DpInterface {
    fn write_state(&self, w: &mut StateWriter) {
        w.class("Processor", &self.processor);
        w.u32("_current", self.current);
        w.u32("_end", self.end);
        w.bool("_freeze", self.freeze);
        w.bool("_running", self.running);
        w.u32("_start", self.start);
        w.bool("_startValid", self.start_valid);
        w.bool("_xbus", self.xbus);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        r.class(&mut self.processor)?; // Processor
        self.current = r.u32()?; // _current
        self.end = r.u32()?; // _end
        self.freeze = r.bool()?; // _freeze
        self.running = r.bool()?; // _running
        self.start = r.u32()?; // _start
        self.start_valid = r.bool()?; // _startValid
        self.xbus = r.bool()?; // _xbus
        Ok(())
    }
}

impl DpInterface {
    /// `WritePending`: the count, the words, then zeros to `SNAPSHOT_WORDS`.
    pub fn write_pending(&self, w: &mut StateWriter) {
        w.group("Dp.Pending", |w| {
            w.i32("Count", self.pending.len() as i32);
            w.u64s("Words", &self.pending);
            let padding = SNAPSHOT_WORDS.saturating_sub(self.pending.len());
            w.u64s("Padding", &ZEROS[..padding]);
        });
    }

    /// `ReadPending`: the C# skips the padding unread, and so does this.
    pub fn read_pending(&mut self, r: &mut StateReader) -> StateResult {
        let count = r.i32()?;
        if count < 0 || count as usize > SNAPSHOT_WORDS {
            return Err(StateError::PendingOverflow(count));
        }
        self.pending = vec![0; count as usize];
        r.u64s(&mut self.pending)?;
        r.skip((SNAPSHOT_WORDS - count as usize) * 8)
    }
}

static ZEROS: [u64; SNAPSHOT_WORDS] = [0; SNAPSHOT_WORDS];
