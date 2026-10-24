//! Sapphire's own state format, "SPH2" (Sapphire_Plan.md §4.7, §5.10): the magic, the version, the frame count, the
//! market, then every writable memory in id order. Later stages append their blocks under a new version; the layout
//! listing is pinned by a test. The disc is never in it.

use emusen_native::ffi::{StateMachine, Status, status};
use emusen_native::{StateReader, StateWriter, Truncated};

use crate::machine::{Machine, Market};

pub const STATE_MAGIC: u32 = u32::from_le_bytes(*b"SPH2");
pub const STATE_VERSION: i32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    Truncated { at: usize, wanted: usize },
    Foreign(u32),
    Version(i32),
    /// A state of a console of another market, by its byte.
    Market(u8),
    BufferTooSmall { needed: usize },
}

impl Status for StateError {
    fn status(&self) -> i32 {
        match self {
            StateError::Truncated { .. } => status::TRUNCATED,
            StateError::Foreign(_) | StateError::Market(_) => status::FOREIGN,
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

fn market_byte(m: Market) -> u8 {
    match m {
        Market::Japan => 0,
        Market::Americas => 1,
        Market::Europe => 2,
    }
}

impl Machine {
    fn write(&self, w: &mut StateWriter) {
        w.u32("Magic", STATE_MAGIC);
        w.i32("Version", STATE_VERSION);
        w.i64("Frames", self.frames);
        w.u8("Market", market_byte(self.market));
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
        let market = r.u8()?;
        if market != market_byte(self.market) {
            return Err(StateError::Market(market));
        }
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
    use crate::machine::RAM_ID;
    use crate::media::Media;
    use crate::media::sample::elf;

    fn machine(market: Market) -> Machine {
        Machine::new(Media::read(&elf()).unwrap(), market, [None, None])
    }

    #[test]
    fn the_version_1_layout_is_pinned() {
        let m = machine(Market::Americas);
        let layout = m.layout();
        let lines: Vec<&str> = layout.lines().collect();
        assert_eq!(&lines[..5], ["0 4 u32 Magic", "4 4 i32 Version", "8 8 i64 Frames", "16 1 u8 Market", "17 33554432 u8[33554432] RAM"]);
        assert_eq!(lines.last(), Some(&"39903249 2097152 u8[2097152] SPU2RAM"));
        assert_eq!(m.state_size(), 42_000_401);
    }

    #[test]
    fn a_state_round_trips_and_a_refused_one_changes_nothing() {
        let mut a = machine(Market::Americas);
        a.frames = 7;
        a.memory_mut(RAM_ID).unwrap().bytes[(32 << 20) - 1] = 0x5A;
        a.memory_mut(9).unwrap().bytes[3] = 0xA5;
        let mut s = vec![0; a.state_size()];
        a.save_state(&mut s).unwrap();
        let mut b = machine(Market::Americas);
        b.load_state(&s).unwrap();
        assert_eq!((b.frames, b.memory(RAM_ID).unwrap().bytes[(32 << 20) - 1], b.memory(9).unwrap().bytes[3]), (7, 0x5A, 0xA5));
        let mut c = machine(Market::Americas);
        assert_eq!(c.load_state(&s[..s.len() - 1]).unwrap_err().status(), status::TRUNCATED);
        assert_eq!(c.load_state(b"DMND0000").unwrap_err().status(), status::FOREIGN);
        assert_eq!(c.load_state(&[]).unwrap_err().status(), status::FOREIGN);
        let mut v2 = s.clone();
        v2[4] = 2;
        assert_eq!(c.load_state(&v2).unwrap_err().status(), status::VERSION);
        assert_eq!(machine(Market::Europe).load_state(&s).unwrap_err(), StateError::Market(1));
        assert_eq!((c.frames, c.memory(RAM_ID).unwrap().bytes[(32 << 20) - 1]), (0, 0), "a refused load changes nothing");
    }
}
