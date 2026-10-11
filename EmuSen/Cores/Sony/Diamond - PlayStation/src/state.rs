//! Diamond's own state format, "DMND" (Diamond_Plan.md §4.7, §5.10): the magic, the version, the frame count, the
//! R3000A's and the GTE's blocks, and every writable memory in id order. The layout listing is pinned by a test.

use emusen_native::ffi::{StateMachine, Status, status};
use emusen_native::{State, StateReader, StateWriter, Truncated};

use crate::machine::Machine;

pub const STATE_MAGIC: u32 = u32::from_le_bytes(*b"DMND");
pub const STATE_VERSION: i32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    Truncated { at: usize, wanted: usize },
    Foreign(u32),
    Version(i32),
    BufferTooSmall { needed: usize },
}

impl Status for StateError {
    fn status(&self) -> i32 {
        match self {
            StateError::Truncated { .. } => status::TRUNCATED,
            StateError::Foreign(_) => status::FOREIGN,
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

impl Machine {
    fn write(&self, w: &mut StateWriter) {
        w.u32("Magic", STATE_MAGIC);
        w.i32("Version", STATE_VERSION);
        w.i64("Frames", self.frames);
        w.structure("Cpu", &self.cpu);
        w.structure("Gte", &self.gte);
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
        let frames = r.i64()?;
        let mut cpu = self.cpu.clone();
        cpu.read_state(&mut r)?;
        let mut gte = self.gte.clone();
        gte.read_state(&mut r)?;
        let mut loaded = Vec::new();
        for m in self.memories.iter().filter(|m| !m.read_only) {
            let mut bytes = vec![0; m.bytes.len()];
            r.bytes(&mut bytes)?;
            loaded.push(bytes);
        }
        self.frames = frames;
        self.cpu = cpu;
        self.gte = gte;
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
    use crate::machine::{Config, Gpu, RAM_ID};
    use crate::media::{Kind, Market, Media};

    fn machine() -> Machine {
        Machine::new(Media { kind: Kind::Disc { raw: false }, market: None }, Config { market: Market::Americas, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None])
    }

    #[test]
    fn the_version_1_layout_is_pinned() {
        let layout = machine().layout();
        let head: Vec<&str> = layout.lines().take(5).collect();
        assert_eq!(head, ["0 4 u32 Magic", "4 4 i32 Version", "8 8 i64 Frames", "16 128 u32[32] Cpu.R", "144 4 u32 Cpu.Hi"]);
        let tail: Vec<&str> = layout.lines().skip(14).collect();
        assert_eq!(
            tail,
            [
                "208 8 u64 Cpu.HiLoReady",
                "216 128 u32[32] Gte.Data",
                "344 128 u32[32] Gte.Control",
                "472 2097152 u8[2097152] RAM",
                "2097624 1024 u8[1024] SCRATCH",
                "2098648 1048576 u8[1048576] VRAM",
                "3147224 524288 u8[524288] SPURAM"
            ]
        );
        assert_eq!(machine().state_size(), 3_671_512);
    }

    #[test]
    fn a_state_round_trips_and_a_refused_one_changes_nothing() {
        let mut a = machine();
        a.frames = 7;
        a.memory_mut(RAM_ID).unwrap().bytes[5] = 0x5A;
        a.cpu.regs.pc = 0xBFC0_0000;
        a.gte.regs.control[31] = 0x8000_0000;
        let mut s = vec![0; a.state_size()];
        a.save_state(&mut s).unwrap();
        let mut b = machine();
        b.load_state(&s).unwrap();
        assert_eq!((b.frames, b.memory(RAM_ID).unwrap().bytes[5], b.cpu.regs.pc, b.gte.regs.control[31]), (7, 0x5A, 0xBFC0_0000, 0x8000_0000));

        let before = b.memory(RAM_ID).unwrap().bytes.clone();
        assert_eq!(b.load_state(&s[..s.len() - 1]).unwrap_err().status(), status::TRUNCATED);
        assert_eq!(b.load_state(&s[..300]).unwrap_err().status(), status::TRUNCATED);
        assert_eq!(b.load_state(b"NPHR0000").unwrap_err().status(), status::FOREIGN);
        assert_eq!(b.load_state(&[]).unwrap_err().status(), status::FOREIGN);
        let mut v2 = s.clone();
        v2[4] = 2;
        assert_eq!(b.load_state(&v2).unwrap_err().status(), status::VERSION);
        assert_eq!((b.frames, &b.memory(RAM_ID).unwrap().bytes, b.cpu.regs.pc), (7, &before, 0xBFC0_0000));
    }
}
