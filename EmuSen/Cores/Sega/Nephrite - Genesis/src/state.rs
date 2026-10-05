//! Nephrite's own state format, "NPHR" (Nephrite_Plan.md §5.7): the magic, the version, the system, the frame count
//! and every writable memory in id order. The layout listing is pinned by a test, there being no other record.

use emusen_native::ffi::{StateMachine, Status, status};
use emusen_native::{StateReader, StateWriter, Truncated};

use crate::machine::Machine;
use crate::media::System;

pub const STATE_MAGIC: u32 = u32::from_le_bytes(*b"NPHR");
pub const STATE_VERSION: i32 = 1;
/// A state of this core made for another of its systems: a Genesis state offered to a 32X machine.
pub const STATUS_OTHER_SYSTEM: i32 = -10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    Truncated { at: usize, wanted: usize },
    Foreign(u32),
    Version(i32),
    OtherSystem(u8),
    BufferTooSmall { needed: usize },
}

impl Status for StateError {
    fn status(&self) -> i32 {
        match self {
            StateError::Truncated { .. } => status::TRUNCATED,
            StateError::Foreign(_) => status::FOREIGN,
            StateError::Version(_) => status::VERSION,
            StateError::OtherSystem(_) => STATUS_OTHER_SYSTEM,
            StateError::BufferTooSmall { .. } => status::BUFFER_TOO_SMALL,
        }
    }
}

impl From<Truncated> for StateError {
    fn from(t: Truncated) -> Self {
        StateError::Truncated { at: t.at, wanted: t.wanted }
    }
}

fn system_byte(s: System) -> u8 {
    match s {
        System::Md => 0,
        System::Mcd => 1,
        System::S32x => 2,
    }
}

impl Machine {
    fn write(&self, w: &mut StateWriter) {
        w.u32("Magic", STATE_MAGIC);
        w.i32("Version", STATE_VERSION);
        w.u8("System", system_byte(self.media.system));
        w.i64("Frames", self.frames);
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
        let system = r.u8()?;
        if system != system_byte(self.media.system) {
            return Err(StateError::OtherSystem(system));
        }
        let frames = r.i64()?;
        let mut loaded = Vec::new();
        for m in self.memories.iter().filter(|m| !m.read_only) {
            let mut bytes = vec![0; m.bytes.len()];
            r.bytes(&mut bytes)?;
            loaded.push(bytes);
        }
        self.frames = frames;
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
    use crate::media::{Media, cartridge};

    fn machine(system: &str) -> Machine {
        let rom = cartridge(system, "U", None);
        Machine::new(&rom, Media::read(&rom))
    }

    #[test]
    fn the_version_1_layout_is_pinned() {
        assert_eq!(
            machine("SEGA GENESIS").layout(),
            "0 4 u32 Magic\n4 4 i32 Version\n8 1 u8 System\n9 8 i64 Frames\n17 65536 u8[65536] WRAM\n65553 8192 u8[8192] Z80RAM\n73745 65536 u8[65536] VRAM\n139281 128 u8[128] CRAM\n139409 80 u8[80] VSRAM\n"
        );
    }

    #[test]
    fn a_state_round_trips_and_a_refused_one_changes_nothing() {
        let mut a = machine("SEGA GENESIS");
        a.frames = 7;
        a.memory_mut(2).unwrap().bytes[5] = 0x5A;
        let mut s = vec![0; a.state_size()];
        a.save_state(&mut s).unwrap();
        let mut b = machine("SEGA GENESIS");
        b.load_state(&s).unwrap();
        assert_eq!((b.frames, b.memory(2).unwrap().bytes[5]), (7, 0x5A));

        let before = b.memory(2).unwrap().bytes.clone();
        assert_eq!(b.load_state(&s[..s.len() - 1]).unwrap_err().status(), status::TRUNCATED);
        assert_eq!(b.load_state(b"SNES0000").unwrap_err().status(), status::FOREIGN);
        assert_eq!(b.load_state(&[]).unwrap_err().status(), status::FOREIGN);
        assert_eq!(machine("SEGA 32X").load_state(&s).unwrap_err().status(), STATUS_OTHER_SYSTEM);
        assert_eq!((b.frames, &b.memory(2).unwrap().bytes), (7, &before));
    }
}
