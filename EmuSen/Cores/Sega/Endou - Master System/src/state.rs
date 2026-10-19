//! Endou's own state format, "ENDO" (Endou_Plan.md §5.7, Endou_Native.md §9.7, §10.6): the magic, the version, the
//! console and model, the frame count and the clocks, the Z80's block, the PSG's, the bus's ports and the mapper's
//! registers, the work RAM, the VDP's block with VRAM and CRAM, then the cartridge's RAM where the board has it.
//! Version 3 is stage 3's; versions 1 and 2 reached no player and are refused. The layout listing is pinned by a test.

use emusen_native::ffi::{StateMachine, Status, status};
use emusen_native::{StateReader, StateWriter, Truncated};

use crate::machine::Machine;
use crate::media::Console;

pub const STATE_MAGIC: u32 = u32::from_le_bytes(*b"ENDO");
pub const STATE_VERSION: i32 = 3;

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
        w.bytes("RAM", &b.ram);
        b.vdp.write_state(w);
        if !b.cart.ram.is_empty() {
            w.bytes("CARTRAM", &b.cart.ram);
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
        let mut ram = vec![0; self.bus.ram.len()];
        r.bytes(&mut ram)?;
        let mut vdp = self.bus.vdp.clone();
        if !vdp.read_state(&mut r)? {
            return Err(StateError::Foreign(magic));
        }
        let mut cart_ram = vec![0; self.bus.cart.ram.len()];
        r.bytes(&mut cart_ram)?;
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
        b.ram = ram;
        b.vdp = vdp;
        if !cart_ram.is_empty() {
            b.cart.ram = cart_ram;
            b.cart.dirty = true;
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
    fn the_version_3_layout_is_pinned() {
        let m = machine(&cartridge(0x8000, 4));
        let layout = m.layout();
        let lines: Vec<&str> = layout.lines().collect();
        assert_eq!(&lines[..7], ["0 4 u32 Magic", "4 4 i32 Version", "8 1 u8 Console", "9 1 u8 Model", "10 8 i64 Frames", "18 8 u64 Clock", "26 8 u64 FrameEnd"]);
        let at = lines.iter().position(|l| l.ends_with(" RAM")).unwrap();
        assert_eq!(&lines[at..at + 5], ["125 8192 u8[8192] RAM", "8317 1 u8 VdpVariant", "8318 2 bool[2] PalGgMode", "8320 11 u8[11] VdpRegisters", "8331 16384 u8[16384] Vram"], "{layout}");
        assert_eq!(lines[at + 5], "24715 64 u8[64] Cram");
        assert_eq!(lines.last(), Some(&"24872 7 u8[7] Sprite"));
        assert_eq!(m.state_size(), 24_879);
        let sg = machine(&[0u8; 0x2000]);
        assert_eq!(sg.state_size(), 24_879 - 0x2000 + 0x400);
        let big = machine(&cartridge(0x20000, 4));
        assert_eq!(big.state_size(), 24_879 + 0x8000, "a board with RAM keeps it");
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
        a.bus.vdp.vram[0x3FFF] = 0xA5;
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
        assert_eq!((b.frames, b.bus.ram[5], b.bus.vdp.vram[0x3FFF], b.cpu.regs.pc, b.bus.clock), (1, 0x5A, 0xA5, 0x0038, a.bus.clock));
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
