//! Endou's own state format, "ENDO" (Endou_Plan.md §5.7): the magic, the version, the console, the frame count, the
//! Z80's block, the PSG's, and every writable memory in id order. Later stages append their blocks under a new
//! version; the layout listing is pinned by a test.

use emusen_native::ffi::{StateMachine, Status, status};
use emusen_native::{StateReader, StateWriter, Truncated};

use crate::machine::Machine;
use crate::media::Console;

pub const STATE_MAGIC: u32 = u32::from_le_bytes(*b"ENDO");
pub const STATE_VERSION: i32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    Truncated { at: usize, wanted: usize },
    Foreign(u32),
    Version(i32),
    /// A state of the other console, by its byte.
    Console(u8),
    BufferTooSmall { needed: usize },
}

impl Status for StateError {
    fn status(&self) -> i32 {
        match self {
            StateError::Truncated { .. } => status::TRUNCATED,
            StateError::Foreign(_) | StateError::Console(_) => status::FOREIGN,
            StateError::Version(_) => status::VERSION,
            StateError::BufferTooSmall { .. } => status::BUFFER_TOO_SMALL,
        }
    }
}

impl From<Truncated> for StateError {
    fn from(t: Truncated) -> Self {
        StateError::Truncated { at: t.at, wanted: t.wanted }
    }
}

fn console_byte(c: Console) -> u8 {
    match c {
        Console::Sms => 0,
        Console::Sg1000 => 1,
    }
}

impl Machine {
    fn write(&self, w: &mut StateWriter) {
        w.u32("Magic", STATE_MAGIC);
        w.i32("Version", STATE_VERSION);
        w.u8("Console", console_byte(self.console()));
        w.i64("Frames", self.frames);
        w.structure("Cpu", &self.cpu);
        self.psg.write_state(w);
        for m in self.memories.iter().filter(|m| !m.read_only) {
            w.bytes(m.name, &m.bytes);
        }
    }
}

impl StateMachine for Machine {
    type Error = StateError;

    fn load_state(&mut self, data: &[u8]) -> Result<(), StateError> {
        let mut r = StateReader::new(data);
        let magic = r.u32().map_err(|_| StateError::Foreign(0))?;
        if magic != STATE_MAGIC {
            return Err(StateError::Foreign(magic));
        }
        let version = r.i32()?;
        if version != STATE_VERSION {
            return Err(StateError::Version(version));
        }
        let console = r.u8()?;
        if console != console_byte(self.console()) {
            return Err(StateError::Console(console));
        }
        let frames = r.i64()?;
        let mut cpu = self.cpu.clone();
        emusen_native::State::read_state(&mut cpu, &mut r)?;
        let mut psg = self.psg.clone();
        psg.read_state(&mut r)?;
        let mut loaded = Vec::new();
        for m in self.memories.iter().filter(|m| !m.read_only) {
            let mut bytes = vec![0; m.bytes.len()];
            r.bytes(&mut bytes)?;
            loaded.push(bytes);
        }
        self.frames = frames;
        self.cpu = cpu;
        self.psg = psg;
        for (m, bytes) in self.memories.iter_mut().filter(|m| !m.read_only).zip(loaded) {
            m.bytes = bytes;
        }
        Ok(())
    }

    fn state_size(&self) -> usize {
        let mut w = StateWriter::counter();
        self.write(&mut w);
        w.len()
    }

    fn save_state(&self, out: &mut [u8]) -> Result<usize, StateError> {
        let needed = self.state_size();
        if out.len() < needed {
            return Err(StateError::BufferTooSmall { needed });
        }
        let mut w = StateWriter::new(out);
        self.write(&mut w);
        Ok(w.len())
    }

    fn layout(&self) -> String {
        let mut w = StateWriter::layout();
        self.write(&mut w);
        w.into_layout()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::{RAM_ID, VRAM_ID};
    use crate::media::sample::cartridge;
    use crate::media::{Market, Media};

    fn machine(image: &[u8]) -> Machine {
        Machine::new(Media::read(image).unwrap(), Market::Americas, image)
    }

    #[test]
    fn the_version_1_layout_is_pinned() {
        let m = machine(&cartridge(0x8000, 4));
        let layout = m.layout();
        let lines: Vec<&str> = layout.lines().collect();
        assert_eq!(&lines[..4], ["0 4 u32 Magic", "4 4 i32 Version", "8 1 u8 Console", "9 8 i64 Frames"]);
        assert_eq!(&lines[lines.len() - 3..], ["79 8192 u8[8192] RAM", "8271 16384 u8[16384] VRAM", "24655 32 u8[32] CRAM"]);
        assert_eq!(m.state_size(), 24_687);
        let sg = machine(&[0u8; 0x2000]);
        assert_eq!(sg.state_size(), 24_687 - 32 - 0x2000 + 0x400);
    }

    #[test]
    fn a_state_round_trips_and_a_refused_one_changes_nothing() {
        let image = cartridge(0x8000, 4);
        let mut a = machine(&image);
        a.frames = 7;
        a.memory_mut(RAM_ID).unwrap().bytes[5] = 0x5A;
        a.memory_mut(VRAM_ID).unwrap().bytes[0x3FFF] = 0xA5;
        a.cpu.regs.pc = 0x0038;
        a.psg.write(0x9F);
        let mut s = vec![0; a.state_size()];
        a.save_state(&mut s).unwrap();
        let mut b = machine(&image);
        b.load_state(&s).unwrap();
        assert_eq!((b.frames, b.memory(RAM_ID).unwrap().bytes[5], b.memory(VRAM_ID).unwrap().bytes[0x3FFF], b.cpu.regs.pc), (7, 0x5A, 0xA5, 0x0038));
        assert_eq!(b.psg, a.psg);

        let before = b.memory(RAM_ID).unwrap().bytes.clone();
        assert_eq!(b.load_state(&s[..s.len() - 1]).unwrap_err().status(), status::TRUNCATED);
        assert_eq!(b.load_state(b"NPHR0000").unwrap_err().status(), status::FOREIGN);
        assert_eq!(b.load_state(&[]).unwrap_err().status(), status::FOREIGN);
        let mut v2 = s.clone();
        v2[4] = 2;
        assert_eq!(b.load_state(&v2).unwrap_err().status(), status::VERSION);
        let mut sg = machine(&[0u8; 0x2000]);
        assert_eq!(sg.load_state(&s).unwrap_err(), StateError::Console(0));
        assert_eq!((b.frames, &b.memory(RAM_ID).unwrap().bytes, b.cpu.regs.pc), (7, &before, 0x0038));
    }
}
