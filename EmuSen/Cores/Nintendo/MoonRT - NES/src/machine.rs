//! C#'s `MoonCore`: the timeline, the RESET button, the named spaces, and the state in the C# core's own format. See Moon_Core.md, Moon_Native.md §3.1.

use crate::apu::Apu;
use crate::cpu::Cpu;
use crate::memory::Board;
use crate::memory::bus::{MemoryBus, Observed, Plain};
use crate::memory::cartridge::{Cartridge, RomError};
use crate::memory::mappers::Mapper;
use crate::ppu::Ppu;
use crate::state::{StateError, StateReader, StateResult, StateWriter};
use crate::{Fault, Skip};

pub const MASTER_CLOCKS_PER_CPU_CYCLE: i64 = 12;
pub const MASTER_CLOCKS_PER_SCANLINE: i64 = 341 * 4;

/// "MOON" little-endian, then the format version - see Moon_Core.md §5.
pub const STATE_MAGIC: u32 = 0x4E4F_4F4D;
pub const STATE_VERSION: i32 = 5;
/// Version 4 adds the DMA's tail, version 5 the mixer after it; versions 3 and 4 still load (Moon_Native.md §3.9, §3.13).
pub const OLDEST_READABLE_VERSION: i32 = 3;

/// Why a load or a frame could not complete as C# would have.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    Rom(RomError),
    Fault(Fault),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Machine {
    pub total_frames: i64,
    pub line_start_clock: i64,
    pub cpu_budget: i64,
    pub master_clock: i64,
    pub cpu_remainder: i64,
    pub frame_complete: Skip<bool>,
    /// A frame an observed stop left part-run: its scanline's cycles are earned, so its resume must not earn them again.
    pub frame_open: Skip<bool>,
    pub cpu: Cpu,
    pub bus: MemoryBus,
}

impl Machine {
    /// `MoonCore.LoadRom` after the file is read: the board, the PPU and APU, the bus, then the four resets in C#'s order.
    pub fn load_rom(image: &[u8]) -> Result<Machine, LoadError> {
        let cart = Cartridge::parse(image).map_err(LoadError::Rom)?;
        let mapper = Mapper::new(&cart).map_err(LoadError::Rom)?;
        let mut apu = Apu::default();
        apu.set_sample_rate(44100);
        let bus = MemoryBus::new(Board { cart, mapper }, Ppu::default(), apu);
        let mut m = Machine { total_frames: 0, line_start_clock: 0, cpu_budget: 0, master_clock: 0, cpu_remainder: 0, frame_complete: Skip(false), frame_open: Skip(false), cpu: Cpu::default(), bus };
        crate::take_fault();
        m.bus.ppu.reset();
        m.bus.apu.reset();
        m.bus.reset();
        m.cpu.reset(&mut Plain(&mut m.bus));
        if let Some(fault) = crate::take_fault() {
            return Err(LoadError::Fault(fault));
        }
        Ok(m)
    }

    fn reset_schedule(&mut self) {
        self.master_clock = 0;
        self.line_start_clock = 0;
        self.cpu_remainder = 0;
        self.cpu_budget = 0;
        *self.frame_complete = false;
        *self.frame_open = false;
    }

    /// `MoonCore.Reset`: the RESET button - see Moon_Core.md §6.
    pub fn reset(&mut self) -> Result<(), Fault> {
        crate::take_fault();
        self.bus.ppu.soft_reset();
        self.bus.apu.soft_reset();
        self.bus.soft_reset();
        self.cpu.soft_reset(&mut Plain(&mut self.bus));
        self.reset_schedule();
        crate::take_fault().map_or(Ok(()), Err)
    }

    /// `MoonCore.RunFrame` without its debugger seams; C# runs `EndFrame`'s host half after it (Moon_Native.md §2.1).
    pub fn run_frame(&mut self) -> Result<(), Fault> {
        crate::take_fault();
        *self.frame_complete = false;
        let mut earned = std::mem::take(&mut *self.frame_open);
        while !*self.frame_complete {
            let deadline = self.line_start_clock.wrapping_add(MASTER_CLOCKS_PER_SCANLINE);
            if !earned {
                self.earn(deadline)?;
            }
            earned = false;
            self.run_cpu_until_budget_spent();
            if deadline > self.master_clock {
                self.master_clock = deadline;
            }
            self.line_start_clock = self.line_start_clock.wrapping_add(MASTER_CLOCKS_PER_SCANLINE);
            if let Some(fault) = crate::take_fault() {
                return Err(fault);
            }
        }
        Ok(())
    }

    /// `EarnCpuCycles` for the scanline ending at `deadline`.
    #[inline(always)]
    fn earn(&mut self, deadline: i64) -> Result<(), Fault> {
        let delta = deadline.wrapping_sub(self.master_clock);
        if delta < 0 {
            return Err(Fault::ArgumentOutOfRange);
        }
        let scaled = delta.wrapping_add(self.cpu_remainder);
        self.cpu_remainder = scaled % MASTER_CLOCKS_PER_CPU_CYCLE;
        self.cpu_budget = self.cpu_budget.wrapping_add(scaled / MASTER_CLOCKS_PER_CPU_CYCLE);
        Ok(())
    }

    /// `MoonCore.RunFrame` with its debugger seams, stopping where the hooks say; a stop leaves the frame open (Moon_Native.md §8.4).
    pub fn run_frame_debug(&mut self, flags: u32) -> Result<u32, Fault> {
        crate::take_fault();
        *self.frame_complete = false;
        let mut earned = std::mem::take(&mut *self.frame_open);
        let mut unchecked = flags & emusen_native::debug::run::UNCHECKED != 0;
        let result = (|| {
            while !*self.frame_complete {
                let deadline = self.line_start_clock.wrapping_add(MASTER_CLOCKS_PER_SCANLINE);
                if !earned {
                    self.earn(deadline)?;
                }
                earned = false;
                while self.cpu_budget > 0 {
                    self.cpu_budget -= self.bus.take_pending_dma_cycles() as i64;
                    if self.cpu_budget <= 0 {
                        break;
                    }
                    let nmi = self.bus.ppu.nmi_output();
                    self.cpu.set_nmi_line(nmi);
                    let pc = self.cpu.pc as u32;
                    if !unchecked {
                        let why = self.bus.hooks.stop_before(pc);
                        if why != emusen_native::debug::stop::FRAME {
                            *self.frame_open = true;
                            return Ok(why);
                        }
                    }
                    unchecked = false;
                    self.bus.hooks.record(pc);
                    let mark = self.bus.hooks.writes_log.len();
                    self.cpu_budget -= self.cpu.step(&mut Observed(&mut self.bus)) as i64;
                    self.bus.hooks.stamp(mark, self.cpu.last_instruction_pc as u32);
                    self.cpu_budget -= self.bus.take_stolen_cycles() as i64;
                    if !self.bus.ppu.frame_complete {
                        continue;
                    }
                    self.bus.ppu.frame_complete = false;
                    self.total_frames = self.total_frames.wrapping_add(1);
                    *self.frame_complete = true;
                    break;
                }
                if deadline > self.master_clock {
                    self.master_clock = deadline;
                }
                self.line_start_clock = self.line_start_clock.wrapping_add(MASTER_CLOCKS_PER_SCANLINE);
                if let Some(fault) = crate::take_fault() {
                    return Err(fault);
                }
            }
            Ok(emusen_native::debug::stop::FRAME)
        })();
        result
    }

    #[inline(always)]
    fn run_cpu_until_budget_spent(&mut self) {
        while self.cpu_budget > 0 {
            self.cpu_budget -= self.bus.take_pending_dma_cycles() as i64;
            if self.cpu_budget <= 0 {
                break;
            }
            let nmi = self.bus.ppu.nmi_output();
            self.cpu.set_nmi_line(nmi);
            self.cpu_budget -= self.cpu.step(&mut Plain(&mut self.bus)) as i64;
            self.cpu_budget -= self.bus.take_stolen_cycles() as i64;
            if !self.bus.ppu.frame_complete {
                continue;
            }
            self.bus.ppu.frame_complete = false;
            self.total_frames = self.total_frames.wrapping_add(1);
            *self.frame_complete = true;
            return;
        }
    }

    /// One pass of `RunCpuUntilBudgetSpent`'s body without the budget or the frame's end: a test ABI; the cycles charged.
    pub fn step(&mut self) -> i32 {
        let dma = self.bus.take_pending_dma_cycles();
        let nmi = self.bus.ppu.nmi_output();
        self.cpu.set_nmi_line(nmi);
        let cycles = self.cpu.step(&mut Plain(&mut self.bus));
        dma + cycles + self.bus.take_stolen_cycles()
    }

    /// `MoonCore.ReadSpace` by number: RAM, PRGROM, PRGRAM, CHR, CIRAM, OAM, PALETTE, then CPUBUS through the real decode.
    pub fn read_space(&mut self, space: u32, address: i32) -> u8 {
        let bus = &mut self.bus;
        let cart = &bus.board.cart;
        match space {
            0 => bus.ram[(address & 0x07FF) as usize],
            1 => {
                if cart.prg_rom.is_empty() {
                    0
                } else {
                    crate::at(&cart.prg_rom, address.wrapping_rem(cart.prg_len()))
                }
            }
            2 => cart.prg_ram[(address & 0x1FFF) as usize],
            3 => {
                if cart.chr.is_empty() {
                    0
                } else {
                    crate::at(&cart.chr, address.wrapping_rem(cart.chr_len()))
                }
            }
            4 => bus.ppu.ciram[(address & 0x0FFF) as usize],
            5 => bus.ppu.oam[(address & 0xFF) as usize],
            6 => bus.ppu.palette_ram[(address & 0x1F) as usize],
            7 => bus.read((address & 0xFFFF) as u16),
            _ => 0,
        }
    }

    /// `MoonCore.WriteSpace`: PRG ROM refuses, CHR only when it is RAM.
    pub fn write_space(&mut self, space: u32, address: i32, value: u8) {
        let bus = &mut self.bus;
        match space {
            0 => bus.ram[(address & 0x07FF) as usize] = value,
            2 => bus.board.cart.prg_ram[(address & 0x1FFF) as usize] = value,
            3 => {
                let cart = &mut bus.board.cart;
                if *cart.chr_is_ram && !cart.chr.is_empty() {
                    let i = address.wrapping_rem(cart.chr_len());
                    crate::put(&mut cart.chr, i, value);
                }
            }
            4 => bus.ppu.ciram[(address & 0x0FFF) as usize] = value,
            5 => bus.ppu.oam[(address & 0xFF) as usize] = value,
            6 => bus.ppu.palette_ram[(address & 0x1F) as usize] = value,
            7 => {
                // A host store the debugger listens to is reported as C#'s bus reports the processor's, under the last instruction.
                let mark = bus.hooks.writes_log.len();
                let address = (address & 0xFFFF) as u16;
                let irq = if bus.hooks.writes { bus.write_reported(address, value, self.cpu.cycles) } else { bus.write(address, value, self.cpu.cycles) };
                if let Some(irq) = irq {
                    self.cpu.irq_line = irq;
                }
                bus.hooks.stamp(mark, self.cpu.last_instruction_pc as u32);
            }
            _ => {}
        }
    }

    pub fn space_size(&self, space: u32) -> usize {
        let cart = &self.bus.board.cart;
        match space {
            0 => 0x0800,
            1 => cart.prg_rom.len(),
            2 => cart.prg_ram.len(),
            3 => cart.chr.len(),
            4 => 0x1000,
            5 => 0x100,
            6 => 0x20,
            7 => 0x10000,
            _ => 0,
        }
    }

    /// `MoonCore.SaveState`: the header, then the cartridge, its board, the CPU (with the bus inside), the bus, the PPU and the APU.
    pub fn write_state(&self, w: &mut StateWriter) {
        w.u32("Magic", STATE_MAGIC);
        w.i32("Version", STATE_VERSION);
        w.i64("TotalFrames", self.total_frames);
        w.i64("_lineStartClock", self.line_start_clock);
        w.i64("_cpuBudget", self.cpu_budget);
        w.i64("_masterClock", self.master_clock);
        w.i64("_cpuRemainder", self.cpu_remainder);
        w.group("Cart", |w| self.bus.board.cart.write_state(w));
        w.group("Mapper", |w| self.bus.board.mapper.write_state(w));
        w.group("Cpu", |w| self.cpu.write_state(w, &self.bus));
        w.group("Bus", |w| self.bus.write_state(w));
        w.group("Ppu", |w| self.bus.ppu.write_state(w));
        w.group("Apu", |w| self.bus.apu.write_state(w));
        w.u8("Apu.Dmc.SampleBuffer", *self.bus.apu.dmc.sample_buffer);
        w.bool("Apu.Dmc.BufferFull", *self.bus.apu.dmc.buffer_full);
        w.i32("Apu.Dmc.LoadDelay", *self.bus.apu.dmc.load_delay);
        w.bool("Bus.OamDmaPending", *self.bus.oam_dma_pending);
        w.u8("Bus.OamDmaPage", *self.bus.oam_dma_page);
        let mix = &*self.bus.apu.mixer;
        w.f64("Apu._sampleAccumulator", mix.sample_accumulator);
        w.i32("Apu._sampleCount", mix.sample_count);
        w.f64("Apu._cycleFraction", mix.cycle_fraction);
        w.f64("Apu._hp90", mix.hp90);
        w.f64("Apu._hp90Prev", mix.hp90_prev);
        w.f64("Apu._hp440", mix.hp440);
        w.f64("Apu._hp440Prev", mix.hp440_prev);
        w.f64("Apu._lp14k", mix.lp14k);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        let magic = r.u32()?;
        let version = r.i32()?;
        if magic != STATE_MAGIC {
            return Err(StateError::NotAMoonState(magic));
        }
        if !(OLDEST_READABLE_VERSION..=STATE_VERSION).contains(&version) {
            return Err(StateError::Version(version));
        }
        self.total_frames = r.i64()?; // TotalFrames
        self.line_start_clock = r.i64()?; // _lineStartClock
        self.cpu_budget = r.i64()?; // _cpuBudget
        self.master_clock = r.i64()?; // _masterClock
        self.cpu_remainder = r.i64()?; // _cpuRemainder
        self.bus.board.cart.read_state(r)?;
        self.bus.board.mapper.read_state(r)?;
        self.cpu.read_state(r, &mut self.bus)?;
        self.bus.read_state(r)?;
        self.bus.ppu.read_state(r)?;
        self.bus.apu.read_state(r)?;
        let tail = version >= 4;
        *self.bus.apu.dmc.sample_buffer = if tail { r.u8()? } else { 0 };
        *self.bus.apu.dmc.buffer_full = tail && r.bool()?;
        *self.bus.apu.dmc.load_delay = if tail { r.i32()? } else { 0 };
        *self.bus.oam_dma_pending = tail && r.bool()?;
        *self.bus.oam_dma_page = if tail { r.u8()? } else { 0 };
        // Before version 5 the mixer was not in the state: it keeps what it held, as it did then.
        if version >= 5 {
            let mix = &mut *self.bus.apu.mixer;
            mix.sample_accumulator = r.f64()?;
            mix.sample_count = r.i32()?;
            mix.cycle_fraction = r.f64()?;
            mix.hp90 = r.f64()?;
            mix.hp90_prev = r.f64()?;
            mix.hp440 = r.f64()?;
            mix.hp440_prev = r.f64()?;
            mix.lp14k = r.f64()?;
        }
        self.bus.forget_last_read();
        *self.bus.internal_bus = self.bus.open_bus;
        *self.bus.apu.frame_irq_readable = self.bus.apu.frame_irq_pending;
        self.bus.apu.end_length_cycle();
        Ok(())
    }

    /// `MoonCore.LoadState`'s fields; a failed load changes nothing, and bytes past the state are ignored as C# ignores them.
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


impl emusen_native::ffi::StateMachine for Machine {
    type Error = StateError;
    fn load_state(&mut self, data: &[u8]) -> StateResult {
        Machine::load_state(self, data)
    }
    fn state_size(&self) -> usize {
        Machine::state_size(self)
    }
    fn save_state(&self, out: &mut [u8]) -> StateResult<usize> {
        Machine::save_state(self, out)
    }
    fn layout(&self) -> String {
        Machine::layout(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn rom(mapper: u8, prg_banks: u8, chr_banks: u8, program: &[u8]) -> Vec<u8> {
        let mut image = vec![0x4E, 0x45, 0x53, 0x1A, prg_banks, chr_banks, (mapper & 0x0F) << 4 | 0x02, mapper & 0xF0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut prg = vec![0xEAu8; prg_banks as usize * 0x4000];
        let last = prg.len() - 0x4000;
        prg[last..last + program.len()].copy_from_slice(program);
        for v in [0x3FFA, 0x3FFC, 0x3FFE] {
            prg[last + v] = 0x00;
            prg[last + v + 1] = 0xC0;
        }
        image.extend(prg);
        image.extend((0..chr_banks as usize * 0x2000).map(|i| i as u8));
        image
    }

    fn save(m: &Machine) -> Vec<u8> {
        let mut out = vec![0u8; m.state_size()];
        assert_eq!(m.save_state(&mut out), Ok(out.len()));
        out
    }

    #[test]
    fn a_state_round_trips_through_every_board() {
        for mapper in [0u8, 1, 2, 3, 4, 7, 9, 11, 64, 65, 66, 67, 68, 69, 71, 79] {
            let mut m = Machine::load_rom(&rom(mapper, 8, 8, &[0x4C, 0x00, 0xC0])).unwrap();
            for _ in 0..3 {
                m.run_frame().unwrap();
            }
            m.bus.ram[5] = 0x55;
            let state = save(&m);
            let mut back = Machine::load_rom(&rom(mapper, 8, 8, &[0x4C, 0x00, 0xC0])).unwrap();
            back.load_state(&state).unwrap();
            assert_eq!(save(&back), state, "mapper {mapper}");
            assert_eq!(back.bus.ram[5], 0x55);
        }
    }

    #[test]
    fn the_bus_is_written_twice_and_the_last_copy_stands() {
        let m = Machine::load_rom(&rom(0, 1, 1, &[0x4C, 0x00, 0xC0])).unwrap();
        let layout = m.layout();
        assert_eq!(layout.lines().filter(|l| l.ends_with(" Cpu._bus.Ram") || l.ends_with(" Bus.Ram")).count(), 2, "{layout}");
        let mut state = save(&m);
        let last = layout.lines().find(|l| l.ends_with(" Bus.Ram")).unwrap();
        let offset: usize = last.split(' ').next().unwrap().parse().unwrap();
        state[offset] = 0xAB;
        let mut back = m.clone();
        back.load_state(&state).unwrap();
        assert_eq!(back.bus.ram[0], 0xAB);
    }

    #[test]
    fn a_state_that_is_not_moons_or_is_cut_short_changes_nothing() {
        let mut m = Machine::load_rom(&rom(0, 1, 1, &[0x4C, 0x00, 0xC0])).unwrap();
        let state = save(&m);
        let before = m.clone();
        assert_eq!(m.load_state(&state[..state.len() - 1]).map_err(|e| e.status()), Err(-2));
        assert_eq!(m.load_state(&[0x4D, 0x45, 0x52, 0x43, 3, 0, 0, 0]).map_err(|e| e.status()), Err(-3));
        let mut wrong = state.clone();
        wrong[4] = 2;
        assert_eq!(m.load_state(&wrong).map_err(|e| e.status()), Err(-4));
        let mut later = m.clone();
        for _ in 0..3 {
            later.run_frame().unwrap();
        }
        let moved = save(&later);
        assert_ne!(moved, state);
        assert_eq!(m.load_state(&moved[..moved.len() - 1]).map_err(|e| e.status()), Err(-2));
        assert_eq!(m, before);
    }

    #[test]
    fn an_image_without_the_magic_is_not_ines_at_any_length() {
        let mut image = rom(0, 1, 1, &[0x4C, 0x00, 0xC0]);
        image[0] = b'M';
        assert_eq!(Machine::load_rom(&image).map(|_| ()), Err(LoadError::Rom(RomError::NotInes)));
        assert_eq!(Machine::load_rom(&image[..3]).map(|_| ()), Err(LoadError::Rom(RomError::NotInes)));
    }

    #[test]
    fn an_image_without_prg_faults_as_csharp_throws() {
        let mut image = rom(0, 1, 1, &[]);
        image[4] = 0;
        assert_eq!(Machine::load_rom(&image).map(|_| ()), Err(LoadError::Fault(Fault::IndexOutOfRange)));
        let mut image = rom(1, 1, 1, &[]);
        image[4] = 0;
        assert_eq!(Machine::load_rom(&image).map(|_| ()), Err(LoadError::Fault(Fault::DivideByZero)));
    }
}
