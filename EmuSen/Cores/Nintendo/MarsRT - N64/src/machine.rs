//! The C# `MarsCore`'s save state: a header, the CPU, then the bus and everything it owns. See Mars_Native.md §5.1.

use crate::bus::{MemoryBus, RDRAM_SIZE, RDRAM_SIZE_EXPANDED};
use crate::cpu::Cpu;
use crate::dp::SNAPSHOT_WORDS;
use crate::state::{State, StateError, StateReader, StateResult, StateWriter};

/// `StateMagic`: "MARS" little-endian.
pub const STATE_MAGIC: u32 = 0x5352_414D;
/// `StateVersion`: a state, written once the RDP's thread has drained.
pub const STATE_VERSION: i32 = 1;
/// `SnapshotVersion`: the same body, then the RDP words not yet run.
pub const SNAPSHOT_VERSION: i32 = 2;
/// `MarsCore.CycleCap`, which `_lastFrameCycles` starts at.
pub const CYCLE_CAP: i64 = 4_100_000;

/// The whole machine, as a `MarsCore` saves it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Machine {
    /// `TotalFrames`.
    pub total_frames: i64,
    /// `_lastFrameCycles`.
    pub last_frame_cycles: i64,
    pub cpu: Cpu,
    pub bus: MemoryBus,
    /// The version of the state last loaded, 1 or 2; 0 before any.
    pub loaded_version: i32,
}

impl Machine {
    /// A machine with 4 MB or 8 MB of RDRAM; any other size is refused, as `LoadState` refuses it.
    pub fn new(rdram_bytes: usize) -> Option<Machine> {
        if rdram_bytes != RDRAM_SIZE && rdram_bytes != RDRAM_SIZE_EXPANDED {
            return None;
        }
        Some(Machine {
            total_frames: 0,
            last_frame_cycles: CYCLE_CAP,
            cpu: Cpu::default(),
            bus: MemoryBus::new(rdram_bytes),
            loaded_version: 0,
        })
    }

    pub fn rdram_bytes(&self) -> usize {
        self.bus.rdram.len()
    }

    /// `MarsCore.Write`. A state cannot carry pending RDP words, since only running them would drain them.
    pub fn write_state(&self, w: &mut StateWriter, snapshot: bool) -> StateResult {
        let pending = self.bus.dp.pending.len();
        if pending > SNAPSHOT_WORDS {
            return Err(StateError::PendingOverflow(pending as i32));
        }
        if !snapshot && pending != 0 {
            return Err(StateError::PendingInState(pending));
        }
        w.u32("Magic", STATE_MAGIC);
        w.i32("Version", if snapshot { SNAPSHOT_VERSION } else { STATE_VERSION });
        w.i32("Rdram.Length", self.bus.rdram.len() as i32);
        w.i64("TotalFrames", self.total_frames);
        w.i64("_lastFrameCycles", self.last_frame_cycles);
        w.group("Cpu", |w| self.cpu.write_state(w));
        self.bus.write_state_all(w, snapshot);
        Ok(())
    }

    pub fn state_size(&self, snapshot: bool) -> StateResult<usize> {
        let mut w = StateWriter::counter();
        self.write_state(&mut w, snapshot)?;
        Ok(w.len())
    }

    /// Writes into `out`, which must hold `state_size` bytes; returns the bytes written.
    pub fn save_state(&self, out: &mut [u8], snapshot: bool) -> StateResult<usize> {
        let mut w = StateWriter::new(out);
        self.write_state(&mut w, snapshot)?;
        if w.overflowed() {
            return Err(StateError::BufferTooSmall { needed: w.len() });
        }
        Ok(w.len())
    }

    pub fn save_state_vec(&self, snapshot: bool) -> StateResult<Vec<u8>> {
        let mut out = vec![0; self.state_size(snapshot)?];
        self.save_state(&mut out, snapshot)?;
        Ok(out)
    }

    /// One line per field, `offset length type path`, in the order written; the C# side walks its own to compare.
    pub fn layout(&self, snapshot: bool) -> StateResult<String> {
        let mut w = StateWriter::layout();
        self.write_state(&mut w, snapshot)?;
        Ok(w.into_layout())
    }

    /// `MarsCore.LoadState`: a state of the other known RDRAM size rebuilds the machine to it. A failed load changes nothing.
    pub fn load_state(&mut self, data: &[u8]) -> StateResult {
        let mut r = StateReader::new(data);
        let magic = r.u32()?;
        if magic != STATE_MAGIC {
            return Err(StateError::NotAMarsState(magic));
        }
        let version = r.i32()?;
        if version != STATE_VERSION && version != SNAPSHOT_VERSION {
            return Err(StateError::Version(version));
        }
        let rdram = r.i32()?;
        let mut machine = usize::try_from(rdram).ok().and_then(Machine::new).ok_or(StateError::RdramSize(rdram))?;

        machine.total_frames = r.i64()?; // TotalFrames
        machine.last_frame_cycles = r.i64()?; // _lastFrameCycles
        machine.cpu.read_state(&mut r)?; // Cpu
        machine.bus.read_state_all(&mut r, version == SNAPSHOT_VERSION)?; // Bus
        machine.loaded_version = version;

        *self = machine;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::ControllerPak;
    use crate::save::SaveChip;

    fn busy() -> Machine {
        let mut m = Machine::new(RDRAM_SIZE_EXPANDED).unwrap();
        m.total_frames = 1234;
        m.cpu.gpr[5] = 0xDEAD_BEEF;
        m.cpu.tlb.entries[31].page_mask = 0x1FFE000;
        m.bus.rdram[0x7F_FFFF] = 0x5A;
        m.bus.dp.processor.tiles[7].th = -9;
        m.bus.si.due = 0x1_2345_6789;
        m.bus.si.pending_read = 0x1000;
        m.bus.registers.insert(0x0450_0010, 7);
        m.bus.save = SaveChip::new(crate::save::save_type::FLASH_RAM);
        m.bus.si.controllers[2].pak = Some(ControllerPak::default());
        m
    }

    #[test]
    fn a_state_reads_back_into_the_machine_that_wrote_it() {
        for snapshot in [false, true] {
            let mut m = busy();
            if snapshot {
                m.bus.dp.pending = vec![1, 2, 3];
            }
            let bytes = m.save_state_vec(snapshot).unwrap();
            assert_eq!(bytes.len(), m.state_size(snapshot).unwrap());

            let mut loaded = Machine::new(RDRAM_SIZE).unwrap();
            loaded.load_state(&bytes).unwrap();
            m.loaded_version = if snapshot { SNAPSHOT_VERSION } else { STATE_VERSION };
            assert_eq!(loaded, m);
            assert_eq!(loaded.save_state_vec(snapshot).unwrap(), bytes);
        }
    }

    #[test]
    fn the_layout_covers_every_byte_once_in_order() {
        let m = busy();
        let layout = m.layout(true).unwrap();
        let mut next = 0usize;
        for line in layout.lines() {
            let mut parts = line.split(' ');
            let offset: usize = parts.next().unwrap().parse().unwrap();
            let length: usize = parts.next().unwrap().parse().unwrap();
            assert_eq!(offset, next, "{line}");
            next += length;
        }
        assert_eq!(next, m.state_size(true).unwrap());
    }

    /// No Mars state has one, since every class field is built with its owner; C# then reads none of its fields.
    #[test]
    fn a_class_whose_flag_is_clear_has_no_fields_in_the_stream() {
        let mut m = busy();
        m.bus.is_viewer.memory[9] = 0x77;
        let bytes = m.save_state_vec(false).unwrap();
        let layout = m.layout(false).unwrap();
        let find = |path: &str| -> (usize, usize) {
            let line = layout.lines().find(|l| l.ends_with(&format!(" {path}"))).unwrap();
            let mut parts = line.split(' ');
            (parts.next().unwrap().parse().unwrap(), parts.next().unwrap().parse().unwrap())
        };
        let (flag, _) = find("Bus.IsViewer");
        let (memory, length) = find("Bus.IsViewer._memory");
        let mut cut = bytes[..memory].to_vec();
        cut[flag] = 0;
        cut.extend_from_slice(&bytes[memory + length..]);

        let mut loaded = Machine::new(RDRAM_SIZE).unwrap();
        loaded.load_state(&cut).unwrap();
        m.bus.is_viewer = Default::default();
        m.loaded_version = STATE_VERSION;
        assert_eq!(loaded, m);
    }

    #[test]
    fn a_failed_load_leaves_the_machine_as_it_was() {
        let mut m = busy();
        let bytes = m.save_state_vec(false).unwrap();
        let before = m.clone();
        assert!(matches!(m.load_state(&bytes[..bytes.len() - 1]), Err(StateError::Truncated { .. })));
        assert_eq!(m, before);
        let mut wrong = bytes.clone();
        wrong[8] = 3;
        assert_eq!(m.load_state(&wrong), Err(StateError::RdramSize(i32::from_le_bytes([3, 0, 0x80, 0]))));
    }

    #[test]
    fn a_state_refuses_pending_words_and_a_snapshot_carries_them() {
        let mut m = busy();
        m.bus.dp.pending = vec![9];
        assert_eq!(m.state_size(false), Err(StateError::PendingInState(1)));
        assert_eq!(m.state_size(true).unwrap(), {
            m.bus.dp.pending.clear();
            m.state_size(false).unwrap() + 4 + 8 * SNAPSHOT_WORDS
        });
    }
}
