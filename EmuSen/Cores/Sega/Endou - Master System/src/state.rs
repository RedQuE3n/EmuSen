//! Endou's own state format, "ENDO" (Endou_Plan.md §5.7, Endou_Native.md §9.7): the magic, the version, the console
//! and model, the frame count and the clocks, the Z80's block, the PSG's, the bus's ports and the mapper's registers,
//! then every writable memory in id order. Version 2 is stage 2's; version 1, stage 1's stub, reached no player and is
//! refused. The layout listing is pinned by a test.

use emusen_native::ffi::{StateMachine, Status, status};
use emusen_native::{StateReader, StateWriter, Truncated};

use crate::machine::Machine;
use crate::media::Console;

pub const STATE_MAGIC: u32 = u32::from_le_bytes(*b"ENDO");
pub const STATE_VERSION: i32 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    Truncated { at: usize, wanted: usize },
    Foreign(u32),
    Version(i32),
    /// A state of another console or model, by its bytes.
    Console(u8, u8),
    BufferTooSmall { needed: usize },
}

impl Status for StateError {
    fn status(&self) -> i32 {
        match self {
            StateError::Truncated { .. } => status::TRUNCATED,
            StateError::Foreign(_) | StateError::Console(..) => status::FOREIGN,
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
        let b = &self.bus;
        w.u32("Magic", STATE_MAGIC);
        w.i32("Version", STATE_VERSION);
        w.u8("Console", console_byte(self.console()));
        w.u8("Model", self.model().byte());
        w.i64("Frames", self.frames);
        w.u64("Clock", b.clock);
        w.u64("FrameEnd", self.frame_end);
        w.structure("Cpu", &self.cpu);
        b.psg.write_state(w);
        w.u64("PsgTicks", b.psg_ticks);
        w.u8("MemoryControl", b.memory_control);
        w.u8("IoControl", b.io.control);
        w.u8("DataBus", b.data_bus);
        w.bool("Nmi", b.nmi);
        w.bool("PauseHeld", self.pause_held);
        w.bytes("MapperRegisters", &b.cart.regs);
        w.u32s("Slots", &b.cart.slots.map(|s| s as u32));
        for (id, name, read_only) in self.spaces() {
            if !read_only {
                w.bytes(name, self.space(id).unwrap());
            }
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
        let (console, model) = (r.u8()?, r.u8()?);
        if console != console_byte(self.console()) || model != self.model().byte() {
            return Err(StateError::Console(console, model));
        }
        let frames = r.i64()?;
        let (clock, frame_end) = (r.u64()?, r.u64()?);
        let mut cpu = self.cpu.clone();
        emusen_native::State::read_state(&mut cpu, &mut r)?;
        let mut psg = self.bus.psg.clone();
        psg.read_state(&mut r)?;
        let psg_ticks = r.u64()?;
        let (memory_control, io_control, data_bus) = (r.u8()?, r.u8()?, r.u8()?);
        let (nmi, pause_held) = (r.bool()?, r.bool()?);
        let mut regs = [0u8; 4];
        r.bytes(&mut regs)?;
        let mut slots = [0u32; 3];
        r.u32s(&mut slots)?;
        // A slot's offset is a bank of this ROM, or a plain board's fixed slot past a short one.
        let banks = self.bus.cart.rom.len().div_ceil(0x4000).max(3);
        if slots.iter().any(|&s| s as usize % 0x4000 != 0 || s as usize >= banks * 0x4000) {
            return Err(StateError::Foreign(magic));
        }
        let mut loaded = Vec::new();
        for (id, _, read_only) in self.spaces() {
            if !read_only {
                let mut bytes = vec![0; self.space(id).unwrap().len()];
                r.bytes(&mut bytes)?;
                loaded.push((id, bytes));
            }
        }
        self.frames = frames;
        self.frame_end = frame_end;
        self.cpu = cpu;
        self.pause_held = pause_held;
        let b = &mut self.bus;
        b.clock = clock;
        b.psg = psg;
        b.psg_ticks = psg_ticks;
        b.memory_control = memory_control;
        b.io.control = io_control;
        b.data_bus = data_bus;
        b.nmi = nmi;
        b.cart.regs = regs;
        b.cart.slots = slots.map(|s| s as usize);
        for (id, bytes) in loaded {
            self.space_mut(id).unwrap().copy_from_slice(&bytes);
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
    use crate::machine::{CARTRAM_ID, RAM_ID, VRAM_ID};
    use crate::media::sample::cartridge;
    use crate::media::{Market, Media};
    use crate::model::Model;

    fn machine(image: &[u8]) -> Machine {
        let media = Media::read(image).unwrap();
        let model = if media.console == Console::Sg1000 { Model::Sg1000 } else { Model::Sms2 };
        Machine::new(media, Market::Americas, model, image)
    }

    #[test]
    fn the_version_2_layout_is_pinned() {
        let m = machine(&cartridge(0x8000, 4));
        let layout = m.layout();
        let lines: Vec<&str> = layout.lines().collect();
        assert_eq!(&lines[..7], ["0 4 u32 Magic", "4 4 i32 Version", "8 1 u8 Console", "9 1 u8 Model", "10 8 i64 Frames", "18 8 u64 Clock", "26 8 u64 FrameEnd"]);
        let tail = &lines[lines.len() - 3..];
        assert_eq!(tail, ["125 8192 u8[8192] RAM", "8317 16384 u8[16384] VRAM", "24701 32 u8[32] CRAM"], "{layout}");
        assert_eq!(m.state_size(), 24_733);
        let sg = machine(&[0u8; 0x2000]);
        assert_eq!(sg.state_size(), 24_733 - 32 - 0x2000 + 0x400);
        let big = machine(&cartridge(0x20000, 4));
        assert_eq!(big.state_size(), 24_733 + 0x8000, "a board with RAM keeps it");
    }

    // The kit's C7 found a plain 32 KiB board's state refused by a fresh machine (Endou_Native.md §9.12).
    #[test]
    fn every_board_s_state_loads_on_a_machine_just_made() {
        for image in [cartridge(0x8000, 4), cartridge(0x2000 * 6, 4), cartridge(0x20000, 4), vec![0u8; 0x2000]] {
            let mut a = machine(&image);
            a.advance();
            let mut s = vec![0; a.state_size()];
            a.save_state(&mut s).unwrap();
            machine(&image).load_state(&s).unwrap_or_else(|e| panic!("{} bytes: {e:?}", image.len()));
        }
    }

    #[test]
    fn a_state_round_trips_and_a_refused_one_changes_nothing() {
        let image = cartridge(0x20000, 4);
        let mut a = machine(&image);
        a.advance();
        a.bus.ram[5] = 0x5A;
        a.bus.vram[0x3FFF] = 0xA5;
        a.bus.cart.write(0xFFFC, 0x08);
        a.bus.cart.write(0xFFFF, 6);
        a.bus.cart.write(0x8000, 0x77);
        a.bus.io.control = 0x55;
        a.cpu.regs.pc = 0x0038;
        a.bus.psg.write(0x9F);
        let mut s = vec![0; a.state_size()];
        a.save_state(&mut s).unwrap();
        let mut b = machine(&image);
        b.load_state(&s).unwrap();
        assert_eq!((b.frames, b.bus.ram[5], b.bus.vram[0x3FFF], b.cpu.regs.pc, b.bus.clock), (1, 0x5A, 0xA5, 0x0038, a.bus.clock));
        assert_eq!((b.bus.cart.regs, b.bus.cart.slots, b.space(CARTRAM_ID).unwrap()[0], b.bus.io.control), (a.bus.cart.regs, a.bus.cart.slots, 0x77, 0x55));
        assert_eq!(b.bus.psg, a.bus.psg);

        let before = b.space(RAM_ID).unwrap().to_vec();
        assert_eq!(b.load_state(&s[..s.len() - 1]).unwrap_err().status(), status::TRUNCATED);
        assert_eq!(b.load_state(b"NPHR0000").unwrap_err().status(), status::FOREIGN);
        assert_eq!(b.load_state(&[]).unwrap_err().status(), status::FOREIGN);
        let mut v1 = s.clone();
        v1[4] = 1;
        assert_eq!(b.load_state(&v1).unwrap_err().status(), status::VERSION);
        let mut sg = machine(&[0u8; 0x2000]);
        assert_eq!(sg.load_state(&s).unwrap_err(), StateError::Console(0, Model::Sms2.byte()));
        let small = machine(&cartridge(0x8000, 4));
        let mut other = small;
        assert!(other.load_state(&s).is_err(), "a state of another board's size is refused");
        assert_eq!((b.frames, b.space(RAM_ID).unwrap(), b.space(VRAM_ID).unwrap()[0x3FFF]), (1, &before[..], 0xA5));
    }
}
