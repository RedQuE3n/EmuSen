//! C#'s `MercuryCore`: the machine, and its state in the C# core's own format. See Mercury_Native.md §3.1.

use crate::Skip;
use crate::cpu::{Cpu, IllegalOpcode};
use crate::debug::{Hooks, ObservedBus, stop};
use crate::memory::bus::MemoryBus;
use crate::memory::cartridge::{Cartridge, RomError};
use crate::state::{StateError, StateReader, StateResult, StateWriter};

pub const CPU_CLOCK_HZ: i64 = 4_194_304;
pub const CYCLES_PER_FRAME: i64 = 70_224;

/// "MERC" little-endian, then the format version - see EmuSen_Save_States.md §3.
pub const STATE_MAGIC: u32 = 0x4352_454D;
pub const STATE_VERSION: i32 = 6;
/// The oldest version a load still reads, its retired fields read and dropped (Mercury_Native.md §9.3).
pub const OLDEST_READABLE_VERSION: i32 = 5;

/// `run_frame_debug`'s flags: the first step runs unchecked, as the one a halt stopped in front of does.
pub const RUN_UNCHECKED: u32 = 1;
/// ... and the frame is the one the last call stopped inside, its budget kept, rather than a new call of `RunFrame`.
pub const RUN_CONTINUING: u32 = 2;

#[derive(Clone, Debug, PartialEq)]
pub struct Machine {
    pub total_frames: i64,
    pub cycles_into_frame: i64,
    pub cpu: Cpu,
    pub bus: MemoryBus,
    /// The debugger's tables and logs; in no state, so a load keeps them - see Mercury_Native.md §8.5.
    pub hooks: Skip<Box<Hooks>>,
}

impl Machine {
    /// `MercuryCore.LoadRom` after the file is read: the board, the bus, then both resets, in C#'s order.
    pub fn load_rom(image: Vec<u8>) -> Result<Machine, RomError> {
        let (cart, mapper) = Cartridge::from_image(image)?;
        let mut bus = MemoryBus::new(cart, mapper);
        let mut cpu = Cpu::default();
        bus.reset();
        cpu.reset(*bus.cgb);
        // C#'s SetSampleRate(44100), divided in doubles (Mercury_Native.md §9.1); the shim passes C#'s own Math.Pow.
        let cycles_per_sample = CPU_CLOCK_HZ as f64 / 44_100.0;
        bus.apu.set_sample_rate(cycles_per_sample, crate::apu::HIGH_PASS_SEED.powf(cycles_per_sample));
        Ok(Machine { total_frames: 0, cycles_into_frame: 0, cpu, bus, hooks: Skip(Box::default()) })
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

    /// `MercuryCore.RunFrame` with its seams: before each step the tables are asked, and a frame that meets one returns the reasons
    /// (`debug::stop`) with the machine at a step boundary; zero is the frame's end. What it records goes to the hooks' logs.
    pub fn run_frame_debug(&mut self, flags: u32) -> Result<u32, IllegalOpcode> {
        let Machine { cpu, bus, hooks, cycles_into_frame, total_frames } = self;
        let hooks: &mut Hooks = hooks;
        if flags & RUN_CONTINUING == 0 {
            hooks.budget = if bus.double_speed { CYCLES_PER_FRAME * 2 } else { CYCLES_PER_FRAME };
        }
        let budget = hooks.budget;
        let mut unchecked = flags & RUN_UNCHECKED != 0;
        while *cycles_into_frame < budget {
            if !unchecked {
                let why = hooks.stop_before(cpu.pc);
                if why != stop::FRAME {
                    return Ok(why);
                }
            }
            unchecked = false;
            hooks.record(cpu.pc);
            let mark = hooks.writes_log.len();
            let (ie, iflags) = (bus.interrupt_enable, bus.interrupt_flags);
            let stepped = cpu.step(&mut ObservedBus { bus, hooks }, ie, iflags);
            hooks.stamp(mark, cpu.last_instruction_pc);
            let (cycles, serviced) = stepped?;
            if serviced >= 0 {
                bus.interrupt_flags &= !(1u8 << serviced);
            }
            let stall = bus.take_pending_stall();
            if stall > 0 {
                bus.tick(stall);
            }
            *cycles_into_frame += (cycles + stall) as i64;
            if bus.ppu.frame_complete {
                bus.ppu.frame_complete = false;
                *cycles_into_frame = 0;
                *total_frames += 1;
                return Ok(stop::FRAME);
            }
        }
        *cycles_into_frame -= budget;
        *total_frames += 1;
        Ok(stop::FRAME)
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
            // C#'s CPUBUS write is `Bus.Write`, which its observer hears whoever calls it: a cheat's poke, the debugger's.
            6 => {
                let address = (address & 0xFFFF) as u16;
                let reported = if self.hooks.writes { bus.reported_space(address) } else { None };
                bus.write(address, value);
                if let Some((space, offset)) = reported {
                    self.hooks.note_write(space, offset, value, self.cpu.last_instruction_pc);
                }
            }
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
        w.group("Mapper", |w| self.bus.mapper.write_state(w));
        w.group("Cpu", |w| crate::state::State::write_state(&self.cpu, w));
        w.group("Bus", |w| self.bus.write_state(w));
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        let magic = r.u32()?;
        if magic != STATE_MAGIC {
            return Err(StateError::NotAMercuryState(magic));
        }
        let version = r.i32()?;
        if !(OLDEST_READABLE_VERSION..=STATE_VERSION).contains(&version) {
            return Err(StateError::Version(version));
        }
        r.set_version(version);
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
            let mut m = Machine::load_rom(rom(kind, ram, cgb)).unwrap();
            m.bus.cart.ram.iter_mut().enumerate().for_each(|(i, b)| *b = i as u8);
            m.bus.ppu.scx = 7;
            let state = save(&m);
            let mut back = Machine::load_rom(rom(kind, ram, cgb)).unwrap();
            back.load_state(&state).unwrap();
            assert_eq!(save(&back), state);
        }
    }

    fn offset_of(layout: &str, path: &str) -> usize {
        let line = layout.lines().find(|l| l.ends_with(&format!(" {path}"))).unwrap_or_else(|| panic!("{path} not in {layout}"));
        line.split(' ').next().unwrap().parse().unwrap()
    }

    /// A version-5 state as C# wrote it, from this machine's version-6 one: the path after each RAM, two more copies, each with its own fill.
    fn version_5(m: &Machine, path: &str, fills: [u8; 3]) -> Vec<u8> {
        let (v6, layout) = (save(m), m.layout());
        let copy = |fill: u8, class: bool| {
            let mut bytes = if class { vec![1u8] } else { vec![] };
            bytes.extend(std::iter::repeat_n(fill, m.bus.cart.ram.len()));
            bytes.push(path.len() as u8);
            bytes.extend(path.as_bytes());
            bytes
        };
        let (cart_ram, mapper_at, bus_at) = (offset_of(&layout, "Cart.Ram"), offset_of(&layout, "Mapper._ramEnabled"), offset_of(&layout, "Bus._divCounter"));
        let mut v5 = v6[..cart_ram].to_vec();
        v5[4] = 5;
        v5.extend(copy(fills[0], false));
        v5.extend(&v6[cart_ram + m.bus.cart.ram.len()..mapper_at]);
        v5.extend(copy(fills[1], true));
        v5.extend(&v6[mapper_at..bus_at]);
        v5.extend(copy(fills[2], true));
        v5.extend(&v6[bus_at..]);
        v5
    }

    #[test]
    fn version_6_writes_the_cartridge_once_and_version_5s_three_copies_read_with_the_last_standing() {
        let mut m = Machine::load_rom(rom(0x03, 2, 0)).unwrap();
        m.bus.ppu.scx = 9;
        let layout = m.layout();
        assert_eq!(layout.lines().filter(|l| l.contains("Ram") && l.contains("u8[8192]")).count(), 1, "{layout}");
        assert!(!layout.contains("_savePath") && !layout.contains("_cart"), "{layout}");

        let v5 = version_5(&m, "/tmp/π/x.srm", [0x11, 0x22, 0xAB]);
        let mut back = Machine::load_rom(rom(0x03, 2, 0)).unwrap();
        back.load_state(&v5).unwrap();
        assert!(back.bus.cart.ram.iter().all(|&b| b == 0xAB));
        assert_eq!(back.bus.ppu.scx, 9);
        m.bus.cart.ram.fill(0xAB);
        assert_eq!(save(&back), save(&m));

        let before = back.clone();
        assert_eq!(back.load_state(&v5[..v5.len() - 1]).map_err(|e| e.status()), Err(-2));
        assert_eq!(back, before);
    }

    #[test]
    fn a_state_that_is_not_mercurys_or_is_cut_short_changes_nothing() {
        let mut m = Machine::load_rom(rom(0x00, 0, 0)).unwrap();
        let state = save(&m);
        let before = m.clone();
        assert_eq!(m.load_state(&state[..state.len() - 1]).map_err(|e| e.status()), Err(-2));
        assert_eq!(m.load_state(&[0x4D, 0x41, 0x52, 0x54, 5, 0, 0, 0]).map_err(|e| e.status()), Err(-3));
        let mut wrong = state.clone();
        wrong[4] = 4;
        assert_eq!(m.load_state(&wrong).map_err(|e| e.status()), Err(-4));
        wrong[4] = 7;
        assert_eq!(m.load_state(&wrong).map_err(|e| e.status()), Err(-4));
        assert_eq!(m, before);
    }
}
