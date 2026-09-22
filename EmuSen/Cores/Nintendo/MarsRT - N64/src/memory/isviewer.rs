//! The IS-Viewer, the debug text port the hardware corpus prints through: the C# `IsViewer`. See Mars_Memory.md §4.

use crate::Skip;
use crate::state::{State, StateReader, StateResult, StateWriter, boxed};

pub const LENGTH_REGISTER_OFFSET: u32 = 0x14;
pub const BUFFER_OFFSET: u32 = 0x20;
const SIZE: u32 = 0x1000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IsViewer {
    pub memory: Box<[u8; 4096]>,
    /// `_text`: the harness's transcript of the port, not the port.
    pub text: Skip<Vec<u8>>,
}

impl Default for IsViewer {
    fn default() -> Self {
        IsViewer { memory: boxed(0), text: Skip(Vec::new()) }
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

impl IsViewer {
    pub fn read32(&self, offset: u32) -> u32 {
        let at = offset as usize;
        u32::from_be_bytes([self.memory[at], self.memory[at + 1], self.memory[at + 2], self.memory[at + 3]])
    }

    /// A length write emits that many bytes from the buffer, and the register keeps its value.
    pub fn write32(&mut self, offset: u32, value: u32) {
        let at = offset as usize;
        self.memory[at..at + 4].copy_from_slice(&value.to_be_bytes());
        if offset == LENGTH_REGISTER_OFFSET && value != 0 {
            let count = value.min(SIZE - BUFFER_OFFSET) as usize;
            let from = BUFFER_OFFSET as usize;
            self.text.extend_from_slice(&self.memory[from..from + count]);
        }
    }
}
