//! C#'s `MercuryCore`: the machine, and its state in the C# core's own format. See Mercury_Native.md §3.1.

use crate::cpu::{Cpu, IllegalOpcode};
use crate::memory::bus::MemoryBus;
use crate::memory::cartridge::{Cartridge, RomError};
use crate::state::{StateError, StateReader, StateResult, StateWriter};

pub const CPU_CLOCK_HZ: i64 = 4_194_304;
pub const CYCLES_PER_FRAME: i64 = 70_224;

/// "MERC" little-endian, then the format version - see EmuSen_Save_States.md §3.
pub const STATE_MAGIC: u32 = 0x4352_454D;
pub const STATE_VERSION: i32 = 5;

#[derive(Clone, Debug, PartialEq)]
pub struct Machine {
    pub total_frames: i64,
    pub cycles_into_frame: i64,
    pub cpu: Cpu,
    pub bus: MemoryBus,
}

impl Machine {
    /// `MercuryCore.LoadRom` after the file is read: the board, the bus, then both resets, in C#'s order.
    pub fn load_rom(image: Vec<u8>, save_path: Option<String>) -> Result<Machine, RomError> {
        let (cart, mapper) = Cartridge::from_image(image, save_path)?;
        let mut bus = MemoryBus::new(cart, mapper);
        let mut cpu = Cpu::default();
        bus.reset();
        cpu.reset(*bus.cgb);
        // C#'s SetSampleRate(44100) divides in integers (Mercury_Native.md §6.1, D1); the shim passes C#'s own Math.Pow.
        let cycles_per_sample = (CPU_CLOCK_HZ / 44_100) as f64;
        bus.apu.set_sample_rate(cycles_per_sample, crate::apu::HIGH_PASS_SEED.powf(cycles_per_sample));
        Ok(Machine { total_frames: 0, cycles_into_frame: 0, cpu, bus })
    }

    /// `MercuryCore.RunFrame` without its debugger seams: until the PPU completes a frame, or a budget for an LCD that is off.
    pub fn run_frame(&mut self) -> Result<(), IllegalOpcode> {
        let budget = if self.bus.double_speed { CYCLES_PER_FRAME * 2 } else { CYCLES_PER_FRAME };
        while self.cycles_into_frame < budget {
            let (ie, iflags) = (self.bus.interrupt_enable, self.bus.interrupt_flags);
            let (cycles, serviced) = self.cpu.step(&mut self.bus, ie, iflags)?;
            if serviced >= 0 {
                self.bus.interrupt_flags &= !(1u8 << serviced);
            }
            let stall = self.bus.take_pending_stall();
            if stall > 0 {
                self.bus.tick(stall);
            }
            self.cycles_into_frame += (cycles + stall) as i64;
            if self.bus.ppu.frame_complete {
                self.bus.ppu.frame_complete = false;
                self.cycles_into_frame = 0;
                self.total_frames += 1;
                return Ok(());
            }
        }
        self.cycles_into_frame -= budget;
        self.total_frames += 1;
        Ok(())
    }

    /// `MercuryCore.ReadSpace` by number: ROM, VRAM, CARTRAM, WRAM, OAM, HRAM, then CPUBUS through the real decode.
    pub fn read_space(&mut self, space: u32, address: i32) -> u8 {
        let bus = &mut self.bus;
        match space {
            0 => usize::try_from(address).ok().and_then(|a| bus.cart.rom.get(a)).copied().unwrap_or(0xFF),
            1 => bus.vram[wrap(address, bus.vram.len())],
            2 => {
                if bus.cart.ram.is_empty() {
                    0xFF
                } else {
                    bus.cart.ram[wrap(address, bus.cart.ram.len())]
                }
            }
            3 => bus.wram[wrap(address, bus.wram.len())],
            4 => bus.oam[wrap(address, bus.oam.len())],
            5 => bus.high_ram[wrap(address, bus.high_ram.len())],
            6 => bus.read((address & 0xFFFF) as u16),
            _ => 0,
        }
    }

    /// `MercuryCore.WriteSpace`: ROM is the cartridge mask, and a debug write must not corrupt the image.
    pub fn write_space(&mut self, space: u32, address: i32, value: u8) {
        let bus = &mut self.bus;
        match space {
            1 => {
                let i = wrap(address, bus.vram.len());
                bus.vram[i] = value;
            }
            2 => {
                if !bus.cart.ram.is_empty() {
                    let i = wrap(address, bus.cart.ram.len());
                    bus.cart.ram[i] = value;
                }
            }
            3 => {
                let i = wrap(address, bus.wram.len());
                bus.wram[i] = value;
            }
            4 => bus.oam[wrap(address, 0xA0)] = value,
            5 => bus.high_ram[wrap(address, 0x7F)] = value,
            6 => bus.write((address & 0xFFFF) as u16, value),
            _ => {}
        }
    }

    pub fn space_size(&self, space: u32) -> usize {
        match space {
            0 => self.bus.cart.rom.len(),
            1 => self.bus.vram.len(),
            2 => self.bus.cart.ram.len(),
            3 => self.bus.wram.len(),
            4 => 0xA0,
            5 => 0x7F,
            6 => 0x10000,
            _ => 0,
        }
    }

    /// `MercuryCore.SaveState`: the header, then the cartridge, its board, the CPU and the bus, each walked as C# walks it.
    pub fn write_state(&self, w: &mut StateWriter) {
        w.u32("Magic", STATE_MAGIC);
        w.i32("Version", STATE_VERSION);
        w.i64("TotalFrames", self.total_frames);
        w.i64("_cyclesIntoFrame", self.cycles_into_frame);
        w.group("Cart", |w| self.bus.cart.write_state(w));
        w.group("Mapper", |w| self.bus.mapper.write_state(w, &self.bus.cart));
        w.group("Cpu", |w| crate::state::State::write_state(&self.cpu, w));
        w.group("Bus", |w| self.bus.write_state(w));
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        let magic = r.u32()?;
        if magic != STATE_MAGIC {
            return Err(StateError::NotAMercuryState(magic));
        }
        let version = r.i32()?;
        if version != STATE_VERSION {
            return Err(StateError::Version(version));
        }
        self.total_frames = r.i64()?; // TotalFrames
        self.cycles_into_frame = r.i64()?; // _cyclesIntoFrame
        self.bus.cart.read_state(r)?;
        self.bus.mapper.read_state(r, &mut self.bus.cart)?;
        crate::state::State::read_state(&mut self.cpu, r)?;
        self.bus.read_state(r)
    }

    /// `MercuryCore.LoadState`'s fields; a failed load changes nothing, and bytes past the state are ignored as C# ignores them.
    pub fn load_state(&mut self, data: &[u8]) -> StateResult {
        let mut next = self.clone();
        next.read_state(&mut StateReader::new(data))?;
        *self = next;
        Ok(())
    }

    pub fn state_size(&self) -> usize {
        let mut w = StateWriter::counter();
        self.write_state(&mut w);
        w.len()
    }

    pub fn save_state(&self, out: &mut [u8]) -> StateResult<usize> {
        let mut w = StateWriter::new(out);
        self.write_state(&mut w);
        if w.overflowed() {
            return Err(StateError::BufferTooSmall { needed: w.len() });
        }
        Ok(w.len())
    }

    /// One line per field, `offset length type path`, in the order the state writes them.
    pub fn layout(&self) -> String {
        let mut w = StateWriter::layout();
        self.write_state(&mut w);
        w.into_layout()
    }
}

/// C#'s `((address % size) + size) % size`, zero for an empty space.
fn wrap(address: i32, size: usize) -> usize {
    if size == 0 { 0 } else { address.rem_euclid(size as i32) as usize }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::cartridge::{HEADER_CHECKSUM_ADDRESS, header_checksum};

    fn rom(kind: u8, ram_code: u8, cgb: u8) -> Vec<u8> {
        let mut rom = vec![0u8; 0x8000];
        rom[0x0143] = cgb;
        rom[0x0147] = kind;
        rom[0x0149] = ram_code;
        rom[HEADER_CHECKSUM_ADDRESS] = header_checksum(&rom);
        rom
    }

    fn save(m: &Machine) -> Vec<u8> {
        let mut out = vec![0u8; m.state_size()];
        assert_eq!(m.save_state(&mut out), Ok(out.len()));
        out
    }

    #[test]
    fn a_state_round_trips_through_every_board() {
        for (kind, ram, cgb) in [(0x00, 0, 0), (0x03, 2, 0), (0x06, 0, 0xC0), (0x10, 3, 0x80), (0x1E, 4, 0)] {
            let mut m = Machine::load_rom(rom(kind, ram, cgb), Some("/tmp/π/x.srm".into())).unwrap();
            m.bus.cart.ram.iter_mut().enumerate().for_each(|(i, b)| *b = i as u8);
            m.bus.ppu.scx = 7;
            let state = save(&m);
            let mut back = Machine::load_rom(rom(kind, ram, cgb), None).unwrap();
            back.load_state(&state).unwrap();
            assert_eq!(save(&back), state);
            assert_eq!(back.bus.cart.save_path.as_deref(), Some("/tmp/π/x.srm"));
        }
    }

    #[test]
    fn the_cartridge_is_written_three_times_and_the_last_copy_stands() {
        let m = Machine::load_rom(rom(0x03, 2, 0), None).unwrap();
        let layout = m.layout();
        assert_eq!(layout.lines().filter(|l| l.ends_with(" Cart.Ram") || l.ends_with("._cart.Ram")).count(), 3, "{layout}");
        let mut state = save(&m);
        let last = layout.lines().find(|l| l.ends_with("Bus._cart.Ram")).unwrap();
        let offset: usize = last.split(' ').next().unwrap().parse().unwrap();
        state[offset] = 0xAB;
        let mut back = m.clone();
        back.load_state(&state).unwrap();
        assert_eq!(back.bus.cart.ram[0], 0xAB);
    }

    #[test]
    fn a_state_that_is_not_mercurys_or_is_cut_short_changes_nothing() {
        let mut m = Machine::load_rom(rom(0x00, 0, 0), None).unwrap();
        let state = save(&m);
        let before = m.clone();
        assert_eq!(m.load_state(&state[..state.len() - 1]).map_err(|e| e.status()), Err(-2));
        assert_eq!(m.load_state(&[0x4D, 0x41, 0x52, 0x54, 5, 0, 0, 0]).map_err(|e| e.status()), Err(-3));
        let mut wrong = state.clone();
        wrong[4] = 4;
        assert_eq!(m.load_state(&wrong).map_err(|e| e.status()), Err(-4));
        assert_eq!(m, before);
    }
}
