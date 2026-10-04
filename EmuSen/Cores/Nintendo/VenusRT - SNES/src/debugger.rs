//! The debugger's frame: the machine's loop with the shared hooks asked before each S-CPU step, calls and stores
//! noted after it, and the SPC700 and the cartridge's processor caught up after every S-CPU instruction so that
//! their probes see each step. A frame that meets a table stops with its reasons and stays open. Nothing here runs
//! in a plain frame. See VenusRT_Native.md §36.

use emusen_native::debug::{kind, run, stop};

use crate::machine::Machine;
use crate::probe::Probe;

/// The cartridge's processor, the debugger's processor 2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chip {
    Sa1,
    Gsu,
    Dsp,
}

impl Machine {
    pub fn chip(&self) -> Option<Chip> {
        let c = &self.sys.cart;
        if c.sa1.is_some() {
            Some(Chip::Sa1)
        } else if c.gsu.is_some() {
            Some(Chip::Gsu)
        } else if c.dsp.as_ref().is_some_and(|(d, _)| d.lle().is_some()) {
            Some(Chip::Dsp)
        } else {
            None
        }
    }

    /// Processor 1's probe, the SPC700's, or processor 2's, the cartridge's.
    fn probe_slot(&mut self, processor: u32) -> Option<&mut Option<Box<Probe>>> {
        let c = &mut self.sys.cart;
        match processor {
            1 => Some(&mut self.sys.apu.probe),
            2 => {
                if let Some(s) = c.sa1.as_mut() {
                    Some(&mut s.probe)
                } else if let Some(g) = c.gsu.as_mut() {
                    Some(&mut g.probe)
                } else {
                    c.dsp.as_mut().and_then(|(d, _)| d.lle_mut()).map(|d| &mut d.probe)
                }
            }
            _ => None,
        }
    }

    /// Fits each probe the tables ask for and removes the others; `resume` is the processor whose halted step is
    /// taken unchecked.
    fn arm(&mut self, resume: u32) {
        for n in 1..=2u32 {
            let breakpoints = self.debug_breakpoints[n as usize - 1].clone();
            let coverage = self.hooks.coverage.get(n as usize).is_some_and(Option::is_some);
            let stores = n == 1 && self.hooks.writes;
            let Some(slot) = self.probe_slot(n) else { continue };
            if breakpoints.is_empty() && !coverage && !stores {
                *slot = None;
                continue;
            }
            let p = slot.get_or_insert_with(Default::default);
            p.breakpoints = breakpoints;
            p.coverage = coverage;
            p.halted = None;
            p.resume = resume == n;
            if !stores {
                p.stores = None;
            } else if p.stores.is_none() {
                p.stores = Some(Vec::new());
            }
        }
        if !self.hooks.writes {
            self.sys.stores = None;
        } else if self.sys.stores.is_none() {
            self.sys.stores = Some(Vec::new());
        }
    }

    /// A plain frame runs with no probe fitted and no frame of the debugger's left open.
    pub fn disarm(&mut self) {
        if self.debug_open.is_none() && self.sys.stores.is_none() && self.sys.apu.probe.is_none() && self.chip_unprobed() {
            return;
        }
        for n in 1..=2 {
            if let Some(slot) = self.probe_slot(n) {
                *slot = None;
            }
        }
        self.sys.stores = None;
        self.debug_open = None;
    }

    fn chip_unprobed(&self) -> bool {
        let c = &self.sys.cart;
        c.sa1.as_ref().is_none_or(|s| s.probe.is_none()) && c.gsu.as_ref().is_none_or(|g| g.probe.is_none()) && c.dsp.as_ref().and_then(|(d, _)| d.lle()).is_none_or(|d| d.probe.is_none())
    }

    /// The frame through the observed loop: `stop`'s reasons, zero at the frame's end. `run::UNCHECKED` takes the
    /// first step of the processor last stopped on without asking; a stop leaves the frame open for the next call.
    pub fn run_frame_debug(&mut self, flags: u32) -> u32 {
        let mut unchecked = flags & run::UNCHECKED != 0;
        let resumed = if unchecked { self.debug_stopped } else { 0 };
        self.arm(resumed);
        if resumed != 0 {
            unchecked = false;
        }
        let frame = *self.debug_open.get_or_insert(self.sys.timing.frame);
        while self.sys.timing.frame == frame {
            let pc = self.cpu.pc_address();
            if !unchecked {
                let why = self.hooks.stop_before(pc);
                if why != stop::FRAME {
                    self.debug_stopped = 0;
                    return why;
                }
            }
            unchecked = false;
            self.hooks.record(pc);
            let opcode = self.sys.read_value(pc, false).unwrap_or(self.sys.mdr);
            let idle = self.cpu.waiting || self.cpu.stopped;
            self.took = 0;
            self.step();
            self.note_flow(pc, opcode, idle);
            if let Some(log) = self.sys.stores.as_mut() {
                for (space, at, value) in log.drain(..) {
                    self.hooks.note_write(space, at, value, pc);
                }
            }
            if let Some(n) = self.observe_others() {
                self.debug_stopped = n;
                return stop::BREAKPOINT;
            }
        }
        self.debug_open = None;
        self.finish_frame();
        stop::FRAME
    }

    /// A call, a return or an interrupt the step just taken made, as the call stack records them.
    fn note_flow(&mut self, pc: u32, opcode: u8, idle: bool) {
        let target = self.cpu.pc_address();
        match self.took {
            1 => self.hooks.note_call(pc, target, kind::NMI),
            2 => self.hooks.note_call(pc, target, kind::IRQ),
            _ if idle => {}
            _ => match opcode {
                0x20 | 0x22 | 0xFC => self.hooks.note_call(pc, target, kind::CALL),
                0x60 | 0x6B | 0x40 => self.hooks.note_return(),
                0x00 => self.hooks.note_call(pc, target, kind::BRK),
                0x02 => self.hooks.note_call(pc, target, kind::COP),
                _ => {}
            },
        }
    }

    /// The SPC700 and a NEC DSP caught up to the S-CPU (the SA-1 and the GSU already are), their probes' steps and
    /// stores taken into the hooks; the processor that stopped at a breakpoint, if one did.
    fn observe_others(&mut self) -> Option<u32> {
        let clock = self.sys.timing.clock;
        self.sys.apu.run_to(clock);
        if let Some((dsp, _)) = self.sys.cart.dsp.as_mut() {
            dsp.run_to(clock);
        }
        let mut halted = None;
        for n in 1..=2u32 {
            let Some(Some(p)) = self.probe_slot(n) else { continue };
            let steps = std::mem::take(&mut p.steps);
            let stores = p.stores.as_mut().map(std::mem::take).unwrap_or_default();
            if p.halted.is_some() && halted.is_none() {
                halted = Some(n);
            }
            for pc in steps {
                self.hooks.record_on(n as usize, pc);
            }
            for (at, value, pc) in stores {
                self.hooks.note_write(7, at, value, pc);
            }
        }
        halted
    }

    /// The address of the step `processor` stands in front of, in the space its breakpoints and disassembly use.
    pub fn debug_pc(&self, processor: u32) -> Option<u64> {
        let c = &self.sys.cart;
        match processor {
            0 => Some(self.cpu.pc_address() as u64),
            1 => Some(self.sys.apu.cpu.pc as u64),
            2 => match self.chip()? {
                Chip::Sa1 => c.sa1.as_ref().map(|s| ((s.cpu.pbr as u64) << 16) | s.cpu.pc as u64),
                Chip::Gsu => c.gsu.as_ref().map(|g| ((g.pbr as u64) << 16) | g.pipe_at as u64),
                Chip::Dsp => c.dsp.as_ref()?.0.lle().map(|d| d.pc as u64 * 3),
            },
            _ => None,
        }
    }

    /// A processor's registers, in `register_names`' order.
    pub fn debug_registers(&self, processor: u32) -> Option<Vec<i64>> {
        let cpu = |c: &crate::cpu::Cpu| vec![c.a as i64, c.x as i64, c.y as i64, c.s as i64, c.d as i64, c.pbr as i64, c.pc as i64, c.dbr as i64, c.p as i64, c.e as i64];
        let c = &self.sys.cart;
        match processor {
            0 => Some(cpu(&self.cpu)),
            1 => {
                let s = &self.sys.apu.cpu;
                Some(vec![s.a as i64, s.x as i64, s.y as i64, s.sp as i64, s.pc as i64, s.psw as i64])
            }
            2 => match self.chip()? {
                Chip::Sa1 => c.sa1.as_ref().map(|s| cpu(&s.cpu)),
                Chip::Gsu => c.gsu.as_ref().map(|g| {
                    let mut v: Vec<i64> = g.r.iter().map(|&r| r as i64).collect();
                    v.extend([g.sfr as i64, g.pbr as i64, g.rombr as i64, g.rambr as i64, g.cbr as i64, g.scbr as i64, g.scmr as i64, g.colr as i64, g.por as i64, g.cfgr as i64, g.clsr as i64]);
                    v
                }),
                Chip::Dsp => c.dsp.as_ref()?.0.lle().map(|d| vec![d.pc as i64, d.rp as i64, d.dp as i64, d.sp as i64, d.k as i64, d.l as i64, d.a as i64, d.b as i64, d.tr as i64, d.trb as i64, d.sr as i64, d.dr as i64, d.so as i64, d.si as i64]),
            },
            _ => None,
        }
    }
}

/// The 65C816's registers as the debugger lists them, for the S-CPU and the SA-1.
pub const CPU_REGISTERS: [(&str, u32); 10] = [("A", 16), ("X", 16), ("Y", 16), ("S", 16), ("D", 16), ("PB", 8), ("PC", 16), ("DB", 8), ("P", 8), ("E", 1)];
pub const SPC_REGISTERS: [(&str, u32); 6] = [("A", 8), ("X", 8), ("Y", 8), ("SP", 8), ("PC", 16), ("PSW", 8)];
pub const GSU_REGISTERS: [(&str, u32); 27] = [
    ("R0", 16), ("R1", 16), ("R2", 16), ("R3", 16), ("R4", 16), ("R5", 16), ("R6", 16), ("R7", 16), ("R8", 16), ("R9", 16), ("R10", 16), ("R11", 16), ("R12", 16),
    ("R13", 16), ("R14", 16), ("R15", 16), ("SFR", 16), ("PBR", 8), ("ROMBR", 8), ("RAMBR", 8), ("CBR", 16), ("SCBR", 8), ("SCMR", 8), ("COLR", 8), ("POR", 8),
    ("CFGR", 8), ("CLSR", 8),
];
pub const DSP_REGISTERS: [(&str, u32); 14] = [("PC", 16), ("RP", 16), ("DP", 16), ("SP", 8), ("K", 16), ("L", 16), ("A", 16), ("B", 16), ("TR", 16), ("TRB", 16), ("SR", 16), ("DR", 16), ("SO", 16), ("SI", 16)];

#[cfg(test)]
mod tests {
    use super::*;
    use emusen_native::debug::{flag, Range};

    /// LoROM: JSR $8010 in a loop; the routine stores $5A at $7E0020 and returns. The SPC700 waits in its boot program.
    fn machine() -> Machine {
        let mut image = crate::machine::tests::rom(&[0x20, 0x10, 0x80, 0x80, 0xFB]);
        image[0x10..0x16].copy_from_slice(&[0xA9, 0x5A, 0x8D, 0x20, 0x00, 0x60]);
        let mut m = Machine::load_rom(&image).unwrap();
        m.run_frame();
        m
    }

    fn state(m: &Machine) -> Vec<u8> {
        let mut v = vec![0; m.state_size()];
        m.save_state(&mut v).unwrap();
        v
    }

    // A breakpoint stops in front of its instruction and nothing has run; the unchecked resume passes it.
    #[test]
    fn a_breakpoint_stops_before_its_instruction_and_a_resume_passes_it() {
        let mut m = machine();
        m.hooks.set_breakpoints(&[0x8010, 0x8010]);
        assert_eq!(m.run_frame_debug(0), stop::BREAKPOINT);
        assert_eq!((m.debug_stopped, m.debug_pc(0)), (0, Some(0x8010)));
        let frame = m.total_frames();
        assert_eq!(m.run_frame_debug(run::UNCHECKED | run::CONTINUE), stop::BREAKPOINT);
        assert_eq!((m.debug_pc(0), m.total_frames()), (Some(0x8010), frame));
        m.hooks.set_breakpoints(&[]);
        assert_eq!(m.run_frame_debug(run::UNCHECKED | run::CONTINUE), stop::FRAME);
        assert_eq!(m.total_frames(), frame + 1);
    }

    // EACH stops before every instruction; calls and returns are logged with their addresses, a watched store with
    // its instruction's, and coverage marks what ran.
    #[test]
    fn steps_calls_stores_and_coverage_are_reported() {
        let mut m = machine();
        m.hooks.configure(flag::EACH | flag::CALLS | flag::WRITES | (1 << flag::COVERAGE), i32::MIN, -1);
        m.hooks.watch_ranges = vec![Range { space: 2, start: 0x20, end: 0x20 }];
        let mut seen = Vec::new();
        for _ in 0..12 {
            assert_eq!(m.run_frame_debug(run::UNCHECKED | run::CONTINUE) & stop::EACH, stop::EACH);
            seen.push(m.debug_pc(0).unwrap());
        }
        assert!(seen.windows(4).any(|w| w == [0x8010, 0x8012, 0x8015, 0x8003]), "{seen:X?}");
        assert!(m.hooks.calls_log.iter().any(|c| (c.kind, c.source, c.target) == (kind::CALL, 0x8000, 0x8010)));
        assert!(m.hooks.calls_log.iter().any(|c| c.kind == kind::RETURN));
        assert!(m.hooks.writes_log.iter().any(|w| (w.space, w.address, w.value, w.pc) == (2, 0x20, 0x5A, 0x8012)));
        let bits = &m.hooks.coverage[0].as_ref().unwrap().bits;
        assert!(bits[0x8010 >> 3] & 1 != 0 && bits[0x8000 >> 3] & 1 != 0 && bits[0x9000 >> 3] == 0);
    }

    // The SPC700's breakpoint, on the boot program's wait for $CC, stops the frame on processor 1 in front of its
    // instruction, and its resume passes it.
    #[test]
    fn the_spc700_stops_at_its_breakpoint() {
        let mut m = machine();
        m.debug_breakpoints[0] = vec![(0xFFD2, 0xFFD2)];
        assert_eq!(m.run_frame_debug(0), stop::BREAKPOINT);
        assert_eq!((m.debug_stopped, m.debug_pc(1)), (1, Some(0xFFD2)));
        let cycles = m.sys.apu.cycles;
        assert_eq!(m.run_frame_debug(run::UNCHECKED | run::CONTINUE), stop::BREAKPOINT);
        assert!(m.sys.apu.cycles > cycles && m.debug_stopped == 1);
        m.debug_breakpoints[0].clear();
        assert_eq!(m.run_frame_debug(run::UNCHECKED | run::CONTINUE), stop::FRAME);
    }

    // Every table armed and nothing hit: the machine is where a plain run leaves it.
    #[test]
    fn an_armed_frame_with_nothing_to_hit_is_a_plain_frame() {
        let (mut plain, mut armed) = (machine(), machine());
        armed.hooks.configure(flag::CALLS | flag::WRITES | flag::INTERRUPTS | flag::PROFILING | (7 << flag::COVERAGE), i32::MIN, -1);
        armed.hooks.watch_ranges = vec![Range { space: 2, start: 0, end: 0x1FFFF }, Range { space: 7, start: 0, end: 0xFFFF }];
        armed.debug_breakpoints[0] = vec![(0x1234, 0x1234)];
        for _ in 0..5 {
            plain.run_frame();
            while armed.run_frame_debug(run::CONTINUE) != stop::FRAME {
                armed.hooks.writes_log.clear();
                armed.hooks.calls_log.clear();
            }
            assert_eq!(state(&plain), state(&armed));
        }
        assert!(armed.hooks.coverage[1].as_ref().unwrap().covered > 0);
    }
}
