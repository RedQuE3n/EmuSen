//! Endou's own state format, "ENDO" (Endou_Plan.md §5.7, Endou_Native.md §9.7, §10.6): the magic, the version, the
//! console and model, the boot and its BIOS's CRC-32, the frame count and the clocks, the Z80's block, the sound's (the
//! PSG, its clock and the step synthesiser's pending differences), the bus's ports and the mapper's registers, the work
//! RAM, the VDP's block with VRAM and CRAM, then the cartridge's RAM where the board has it, and on the Game Gear its
//! system ports and the EEPROM board's chip (Jadeite_Native.md §9.5). Version 6 is stage 6's;
//! versions 1 to 5 reached no player and are refused. The layout listing is pinned by a test.

use emusen_native::ffi::{StateMachine, status};
use emusen_native::{StateWriter};

use crate::board::Mapper;
use crate::machine::Machine;
use crate::media::Console;

pub const STATE_MAGIC: u32 = u32::from_le_bytes(*b"ENDO");
pub const STATE_VERSION: i32 = 7;

/// The last version before the FM unit, read on the consoles without one.
pub const STATE_VERSION_BEFORE_FM: i32 = 6;

emusen_native::state_error! {
    pub enum StateError {
        /// A state of another console or model, by its bytes.
        Console(u8, u8) => status::FOREIGN,
        /// A state of a machine started by another boot or BIOS: the boot's byte and the BIOS's CRC-32.
        Boot(u8, u32) => status::FOREIGN,
    }
}

fn console_byte(c: Console) -> u8 {
    match c {
        Console::Sms => 0,
        Console::Sg1000 => 1,
        Console::Gg => 2,
    }
}

impl Machine {
    fn write(&self, w: &mut StateWriter) {
        let b = &self.bus;
        w.u32("Magic", STATE_MAGIC);
        w.i32("Version", STATE_VERSION);
        w.u8("Console", console_byte(self.console()));
        w.u8("Model", self.model().byte());
        w.u8("Boot", self.boot.byte());
        w.u32("Bios", self.bios_crc);
        w.i64("Frames", self.frames);
        w.u64("Clock", b.clock);
        w.u64("FrameEnd", self.frame_end);
        w.structure("Cpu", &self.cpu);
        b.sound.write_state(w);
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
        if self.model().is_game_gear() {
            b.gg.write_state(w);
        }
        if b.cart.board.mapper == Mapper::SegaEeprom {
            w.u8("EepromRegister", b.cart.eeprom_register);
            b.cart.eeprom.write_state(w);
        }
    }
}

impl StateMachine for Machine {
    type Error = StateError;

    fn load_state(&mut self, data: &[u8]) -> Result<(), StateError> {
        let (mut r, _) = emusen_native::shell::open_state(data, STATE_MAGIC, |version| version == STATE_VERSION || (version == STATE_VERSION_BEFORE_FM && self.model().fm().is_none()))?;
        let (console, model) = (r.u8()?, r.u8()?);
        if console != console_byte(self.console()) || model != self.model().byte() {
            return Err(StateError::Console(console, model));
        }
        let (boot, bios) = (r.u8()?, r.u32()?);
        if boot != self.boot.byte() || bios != self.bios_crc {
            return Err(StateError::Boot(boot, bios));
        }
        let frames = r.i64()?;
        let (clock, frame_end) = (r.u64()?, r.u64()?);
        let mut cpu = self.cpu.clone();
        emusen_native::State::read_state(&mut cpu, &mut r)?;
        let mut sound = self.bus.sound.clone();
        if !sound.read_state(&mut r)? {
            return Err(StateError::Foreign(STATE_MAGIC));
        }
        let (memory_control, io_control, data_bus) = (r.u8()?, r.u8()?, r.u8()?);
        let (nmi, pause_held) = (r.bool()?, r.bool()?);
        let mut regs = [0u8; 4];
        r.bytes(&mut regs)?;
        let mut slots = [0u32; 3];
        r.u32s(&mut slots)?;
        // A slot's offset is a bank of this ROM, or a plain board's fixed slot past a short one.
        let banks = self.bus.cart.rom.len().div_ceil(0x4000).max(3);
        if slots.iter().any(|&s| s as usize % 0x4000 != 0 || s as usize >= banks * 0x4000) {
            return Err(StateError::Foreign(STATE_MAGIC));
        }
        let mut ram = vec![0; self.bus.ram.len()];
        r.bytes(&mut ram)?;
        let mut vdp = self.bus.vdp.clone();
        if !vdp.read_state(&mut r)? {
            return Err(StateError::Foreign(STATE_MAGIC));
        }
        let mut cart_ram = vec![0; self.bus.cart.ram.len()];
        r.bytes(&mut cart_ram)?;
        let mut gg = self.bus.gg.clone();
        if self.model().is_game_gear() {
            gg.read_state(&mut r)?;
        }
        let (mut eeprom, mut eeprom_register) = (self.bus.cart.eeprom.clone(), self.bus.cart.eeprom_register);
        if self.bus.cart.board.mapper == Mapper::SegaEeprom {
            eeprom_register = r.u8()?;
            eeprom.read_state(&mut r)?;
        }
        self.frames = frames;
        self.frame_end = frame_end;
        self.debug_open = false;
        self.cpu = cpu;
        self.pause_held = pause_held;
        let b = &mut self.bus;
        b.clock = clock;
        b.sound = sound;
        b.memory_control = memory_control;
        b.io.control = io_control;
        b.data_bus = data_bus;
        b.nmi = nmi;
        b.cart.regs = regs;
        b.cart.slots = slots.map(|s| s as usize);
        b.ram = ram;
        b.vdp = vdp;
        b.gg = gg;
        b.cart.eeprom = eeprom;
        b.cart.eeprom_register = eeprom_register;
        if !cart_ram.is_empty() {
            b.cart.ram = cart_ram;
            b.cart.dirty = true;
        }
        Ok(())
    }

    emusen_native::state_write_methods!(write);
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
    fn the_version_7_layout_is_pinned_and_unchanged_without_an_fm_unit() {
        let m = machine(&cartridge(0x8000, 4));
        let layout = m.layout();
        let lines: Vec<&str> = layout.lines().collect();
        assert_eq!(&lines[..9], ["0 4 u32 Magic", "4 4 i32 Version", "8 1 u8 Console", "9 1 u8 Model", "10 1 u8 Boot", "11 4 u32 Bios", "15 8 i64 Frames", "23 8 u64 Clock", "31 8 u64 FrameEnd"]);
        let sound = lines.iter().position(|l| l.ends_with(" Tone")).unwrap();
        assert_eq!(&lines[sound..sound + 10], ["74 6 u16[3] Tone", "80 4 u8[4] Volume", "84 3 u8[3] NoiseLatchStereo", "87 8 u16[4] Counter", "95 4 bool[4] Bit", "99 2 u16 Shift", "101 8 u64 PsgTicks", "109 8 i32[2] Level", "117 24 u64[3] Sum", "141 1024 u64[128] Pending"], "{layout}");
        let at = lines.iter().position(|l| l.ends_with(" RAM")).unwrap();
        assert_eq!(&lines[at..at + 5], ["1186 8192 u8[8192] RAM", "9378 1 u8 VdpVariant", "9379 2 bool[2] PalGgMode", "9381 11 u8[11] VdpRegisters", "9392 16384 u8[16384] Vram"], "{layout}");
        assert_eq!(lines[at + 5], "25776 64 u8[64] Cram");
        assert_eq!(lines.last(), Some(&"25935 7 u8[7] Sprite"));
        assert_eq!(m.state_size(), 25_942);
        let sg = machine(&[0u8; 0x2000]);
        assert_eq!(sg.state_size(), 25_942 - 0x2000 + 0x400);
        let big = machine(&cartridge(0x20000, 4));
        assert_eq!(big.state_size(), 25_942 + 0x8000, "a board with RAM keeps it");
    }

    // Stage 9: a console with an FM unit keeps the chip, its sample count, port $F2 and its level after the PSG's block,
    // and a state taken mid-note continues the sound exactly; a version 6 state loads only where no FM unit is (§16.7).
    #[test]
    fn the_fm_unit_s_state_is_pinned_and_continues_the_sound() {
        let image = crate::programs::images::image(&crate::programs::images::OPLL);
        let media = Media::read(&image).unwrap();
        let mut a = Machine::new(media.clone(), Market::Japan, Model::JapaneseSms, &image);
        let layout = a.layout();
        let lines: Vec<&str> = layout.lines().collect();
        let at = lines.iter().position(|l| l.ends_with(" PsgTicks")).unwrap();
        assert_eq!(&lines[at..at + 11], ["101 8 u64 PsgTicks", "109 1 u8 OpllAddress", "110 64 u8[64] OpllRegisters", "174 72 u32[18] OpllPhase", "246 18 u8[18] OpllLevel", "264 18 u8[18] OpllStage", "282 72 u16[36] OpllOut", "354 4 u32 OpllCounter", "358 1 u8 OpllAm", "359 4 u32 OpllNoise", "363 8 u64 FmSamples"], "{layout}");
        assert_eq!(a.state_size(), 25_942 + 267, "the chip's 254 bytes, the sample count, port $F2 and the level");
        let bytes = crate::model::ALL.map(|m| (m.id(), m.byte()));
        assert_eq!(bytes, [("mark3", 0), ("sms1", 1), ("sms2", 2), ("sg1000", 3), ("gg", 4), ("mark3fm", 5), ("jsms", 6), ("ggjp", 7)], "the state's model byte");
        let (mut sa, mut sb) = (vec![0i16; 1 << 16], vec![0i16; 1 << 16]);
        for _ in 0..90 {
            a.advance();
            a.audio.drain(&mut sa, usize::MAX);
        }
        let mut s = vec![0; a.state_size()];
        a.save_state(&mut s).unwrap();
        let mut b = Machine::new(media, Market::Japan, Model::JapaneseSms, &image);
        b.load_state(&s).unwrap();
        for _ in 0..30 {
            a.advance();
            b.advance();
        }
        let (na, nb) = (a.audio.drain(&mut sa, usize::MAX), b.audio.drain(&mut sb, usize::MAX));
        assert_eq!((nb, &sb[..nb]), (na, &sa[..na]));
        assert!(sa[..na].iter().any(|&v| v != 0), "the FM sounds");
        let mut v6 = s.clone();
        v6[4] = 6;
        assert_eq!(b.load_state(&v6).unwrap_err(), StateError::Version(6));
        let plain = machine(&cartridge(0x8000, 4));
        let mut s6 = vec![0; plain.state_size()];
        plain.save_state(&mut s6).unwrap();
        s6[4] = 6;
        machine(&cartridge(0x8000, 4)).load_state(&s6).unwrap();
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
        a.bus.sound.write(0x9F, a.bus.clock);
        let mut s = vec![0; a.state_size()];
        a.save_state(&mut s).unwrap();
        let mut b = machine(&image);
        b.load_state(&s).unwrap();
        assert_eq!((b.frames, b.bus.ram[5], b.bus.vdp.vram[0x3FFF], b.cpu.regs.pc, b.bus.clock), (1, 0x5A, 0xA5, 0x0038, a.bus.clock));
        assert_eq!((b.bus.cart.regs, b.bus.cart.slots, b.space(CARTRAM_ID).unwrap()[0], b.bus.io.control), (a.bus.cart.regs, a.bus.cart.slots, 0x77, 0x55));
        assert_eq!(b.bus.sound.psg, a.bus.sound.psg);

        let before = b.space(RAM_ID).unwrap().to_vec();
        emusen_native::shell::assert_header_refusals(&mut b, &s);
        let mut sg = machine(&[0u8; 0x2000]);
        assert_eq!(sg.load_state(&s).unwrap_err(), StateError::Console(0, Model::Sms2.byte()));
        let small = machine(&cartridge(0x8000, 4));
        let mut other = small;
        assert!(other.load_state(&s).is_err(), "a state of another board's size is refused");
        assert_eq!((b.frames, b.space(RAM_ID).unwrap(), b.space(VRAM_ID).unwrap()[0x3FFF]), (1, &before[..], 0xA5));
    }
}
