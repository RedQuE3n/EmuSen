//! The IS-Viewer's buffer, the C# `IsViewer`.

use crate::state::{State, StateReader, StateResult, StateWriter, boxed};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IsViewer {
    pub memory: Box<[u8; 4096]>,
}

impl Default for IsViewer {
    fn default() -> Self {
        IsViewer {
            memory: boxed(0),
        }
    }
}

impl State for IsViewer {
    fn write_state(&self, w: &mut StateWriter) {
        w.bytes("_memory", &self.memory[..]);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        r.bytes(&mut self.memory[..])?; // _memory
        Ok(())
    }
}
