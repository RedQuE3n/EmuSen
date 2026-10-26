//! The debugger (Diamond_Plan.md §4.7, stage 12): the R3000A observed through `Observer` in a frame of its own, which
//! stops where the host's tables say; the call stack, stores and loads reported in the bus and in the memory they land in,
//! coverage and the profile kept in the shared hooks; the R3000A's and the GTE's registers; code disassembled by
//! emusen-mips with the GTE's names. The plain frame never meets any of it. Diamond_Debugger.md records each rule.

use blackmoon_r3000::{Observer, Step};
pub use blackmoon_r3000::reference;
use emusen_mips::disassemble;
use emusen_native::core::Instruction;
use emusen_native::debug::{Hooks, kind, run, stop};

use crate::bus::{Reached, reached};
use crate::machine::{BIOS_ID, Machine, RAM_ID, SCRATCH_ID, side_load, step_on};

/// The processor's address space as a space, at the id the plan reserved for it (Diamond_Plan.md §4.3).
pub const R3000BUS_ID: u32 = 0;
pub const R3000BUS_SIZE: i64 = 1 << 32;

/// The processors' numbers in machine info: the R3000A, whose breakpoints the shared hooks keep, and the GTE, which runs
/// no program of its own and is listed for its registers.
pub const R3000: u32 = 0;
pub const GTE: u32 = 1;

/// Coverage's width: the host's registry keeps 24 bits of address, so the mirrors of an address in KUSEG, KSEG0 and
/// KSEG1 share one bit, and the BIOS's code falls at C00000h.
pub const COVERAGE_BITS: u32 = 24;

/// Where code in a memory space runs, for a listing's branch and jump targets: RAM and the scratchpad in KSEG0, the BIOS
/// in KSEG1, as the processor reaches them; any other space at its own offsets.
fn runs_at(space: u32) -> u32 {
    match space {
        RAM_ID => 0x8000_0000,
        SCRATCH_ID => 0x9F80_0000,
        BIOS_ID => 0xBFC0_0000,
        _ => 0,
    }
}

/// The hooks the host pushes its tables into, the frame a stop left open, and the two events that land only once a
/// delay slot has run.
pub struct Debugger {
    pub hooks: Hooks,
    pub open: bool,
    pub stopped: u32,
    /// A `jr ra` whose delay slot has not yet run; its return is noted after the slot, as MarsRT's is.
    returning: bool,
    /// A call whose delay slot has not yet run; an exception taken in the slot abandons it, since EPC is the call's.
    calling: bool,
}

impl Default for Debugger {
    fn default() -> Debugger {
        Debugger::new()
    }
}

impl Debugger {
    pub fn new() -> Debugger {
        Debugger { hooks: Hooks::new(&[COVERAGE_BITS, 0]), open: false, stopped: R3000, returning: false, calling: false }
    }

    /// A state loaded: the open frame and the events in flight belong to the machine that was.
    pub fn close(&mut self) {
        self.open = false;
        self.returning = false;
        self.calling = false;
    }
}

/// The R3000A's observer for one step of the debugger's frame.
struct Watch<'a> {
    hooks: &'a mut Hooks,
    /// The step runs unasked, as the host's resume past a stop asks.
    skip: bool,
    /// The RAM the bus maps, for reporting a store in RAM's own space.
    window: u32,
    /// Whether the step made a call or a return.
    called: bool,
    returned: bool,
    /// A call is open whose delay slot this step runs.
    abandon: bool,
    /// The bytes the step's load would read, its first address and count, with each byte as the bus holds it, when read
    /// data breakpoints are armed (Diamond_Debugger.md §9.3).
    ahead: Option<(u32, u32, [u8; 4])>,
}

impl Watch<'_> {
    /// A byte of a load, in the bus at its address and in RAM or the scratchpad at the offset it lands at.
    fn in_spaces(window: u32, address: u32, mut note: impl FnMut(u32, u32)) {
        note(R3000BUS_ID, address);
        match reached(address, window) {
            Some(Reached::Ram(at)) => note(RAM_ID, at),
            Some(Reached::Scratch(at)) => note(SCRATCH_ID, at),
            None => {}
        }
    }
}

impl Observer for Watch<'_> {
    #[inline(always)]
    fn before(&mut self, _processor: usize, pc: u32) -> u32 {
        if !std::mem::take(&mut self.skip) {
            let mut why = self.hooks.stop_before(pc);
            if let Some((first, count, bytes)) = self.ahead {
                for i in 0..count {
                    let (hooks, value) = (&mut *self.hooks, bytes[i as usize]);
                    Self::in_spaces(self.window, first.wrapping_add(i), |space, at| {
                        if hooks.note_read_ahead(space, at, value, pc) {
                            why |= stop::READ;
                        }
                    });
                }
            }
            if why != stop::FRAME {
                return why;
            }
        }
        self.hooks.record(pc);
        stop::FRAME
    }

    fn wrote(&mut self, _space: u32, address: u32, value: u8, pc: u32) {
        if self.hooks.writes {
            self.hooks.note_write(R3000BUS_ID, address, value, pc);
            match reached(address, self.window) {
                Some(Reached::Ram(at)) => self.hooks.note_write(RAM_ID, at, value, pc),
                Some(Reached::Scratch(at)) => self.hooks.note_write(SCRATCH_ID, at, value, pc),
                None => {}
            }
        }
    }

    fn read(&mut self, _space: u32, address: u32, value: u8, pc: u32) {
        if self.hooks.reads {
            let hooks = &mut *self.hooks;
            Self::in_spaces(self.window, address, |space, at| hooks.note_read(space, at, value, pc));
        }
    }

    fn called(&mut self, source: u32, target: u32, k: u32) {
        if k == kind::CALL {
            self.called = true;
        } else if std::mem::take(&mut self.abandon) {
            self.hooks.note_return();
        }
        self.hooks.note_call(source, target, k);
    }

    fn returned(&mut self) {
        self.returned = true;
    }
}

impl Machine {
    /// The frame through the observed loop: the stop's reasons, zero at the frame's end. A stop leaves the frame open,
    /// the instruction in front of it not run, and the next call goes on from it; `run::UNCHECKED` runs that
    /// instruction unasked.
    pub fn run_frame_debug(&mut self, flags: u32) -> u32 {
        if !self.debug.open {
            self.board.field_done = false;
            self.debug.open = true;
        }
        let mut skip = flags & run::UNCHECKED != 0;
        while !self.board.field_done {
            let window = self.board.ram_window();
            let d = &mut self.debug;
            let ahead = if !skip && d.hooks.reads && !d.hooks.read_break_ranges.is_empty() {
                side_load(&mut self.cpu, &mut self.board, &mut self.exe);
                let at = self.cpu.regs.pc;
                let b = &self.board;
                b.peek_code(at).and_then(|word| self.cpu.load_ahead(word)).map(|(first, count)| (first, count, std::array::from_fn(|i| b.peek(first.wrapping_add(i as u32)))))
            } else {
                None
            };
            let mut w = Watch { hooks: &mut d.hooks, skip, window, called: false, returned: false, abandon: d.calling, ahead };
            let step = if self.board.gpu.layer.moves() {
                step_on::<_, crate::layer::Layered>(&mut self.cpu, &mut self.board, &mut self.exe, self.kernel, &mut w)
            } else {
                step_on::<_, crate::layer::Plain>(&mut self.cpu, &mut self.board, &mut self.exe, self.kernel, &mut w)
            };
            let (called, returned) = (w.called, w.returned);
            skip = false;
            match step {
                Step::Observed(why) => {
                    d.stopped = R3000;
                    return why;
                }
                Step::Exception(_) => {
                    d.returning = false;
                    d.calling = false;
                }
                Step::Instruction => {
                    if std::mem::take(&mut d.returning) {
                        d.hooks.note_return();
                    }
                    let slot = self.cpu.regs.branch.slot;
                    if returned {
                        if slot {
                            d.returning = true;
                        } else {
                            d.hooks.note_return();
                        }
                    }
                    d.calling = called && slot;
                }
            }
        }
        self.board.sync_spu();
        self.frames += 1;
        self.debug.open = false;
        self.debug.stopped = R3000;
        stop::FRAME
    }

    /// The address of the instruction the R3000A stands in front of; the GTE's is 0, since it runs no program.
    pub fn debug_pc(&self, processor: u32) -> Option<u64> {
        match processor {
            R3000 => Some(self.cpu.regs.pc as u64),
            GTE => Some(0),
            _ => None,
        }
    }

    /// A processor's registers in machine info's order: the R3000A's 45 (`blackmoon_r3000::DEBUG_REGISTERS`), the
    /// GTE's 64 as a read returns them, the data registers first.
    pub fn debug_registers(&self, processor: u32) -> Option<Vec<i64>> {
        match processor {
            R3000 => {
                let r = &self.cpu.regs;
                let mut v: Vec<i64> = r.r.iter().map(|&x| x as i64).collect();
                v.extend([r.hi, r.lo, r.pc, r.sr, r.cause, r.epc, r.bad_vaddr, r.tar, r.dcic, r.bpc, r.bda, r.bpcm, r.bdam].map(|x| x as i64));
                Some(v)
            }
            GTE => Some((0..64).map(|n| self.board.gte.register(n) as i64).collect()),
            _ => None,
        }
    }

    /// The word at `address` of a space as a listing reads it: the bus by `peek`, a memory by its bytes ($FF past its end).
    fn listed_word(&self, space: u32, address: u32) -> Option<u32> {
        if space == R3000BUS_ID {
            return Some(self.board.peek_word(address));
        }
        let m = self.space(space)?;
        Some(u32::from_le_bytes(std::array::from_fn(|i| m.get(address as usize + i).copied().unwrap_or(0xFF))))
    }

    /// `count` R3000A instructions from `address` (a word boundary) of `space`, each with the static reference its bytes
    /// alone give: a JAL's or BxxZAL's target as a call, a load or store based on `zero` as a read or a write.
    pub fn debug_disassemble(&self, processor: u32, space: u32, address: u32, count: u32) -> Option<Vec<Instruction>> {
        if processor != R3000 {
            return None;
        }
        self.listed_word(space, 0)?;
        let dialect = blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR));
        let base = runs_at(space);
        let mut out = Vec::with_capacity(count.min(4096) as usize);
        let mut at = address & !3;
        for _ in 0..count {
            let word = self.listed_word(space, at)?;
            let pc = base.wrapping_add(at);
            let text = disassemble(word, pc, &dialect).to_string();
            let (mnemonic, operands) = text.split_once(' ').unwrap_or((&text, ""));
            out.push(Instruction { address: at, bytes: word.to_le_bytes().to_vec(), mnemonic: mnemonic.into(), operands: operands.into(), reference: reference(word, pc) });
            at = at.wrapping_add(4);
        }
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::tests::{machine_for, program};
    use crate::media::Market;
    use emusen_native::core::Reference;
    use emusen_native::debug::{flag, range};
    use emusen_native::ffi::StateMachine;

    /// A loop at 80010034h that calls a routine at 80010100h through JAL, JALR and BGEZAL, the routine storing a count in RAM and the scratchpad
    /// and running a GTE command; with `irq`, the vertical blank's interrupt on and a handler that acknowledges it.
    fn calls(irq: bool) -> String {
        let sr = if irq { "0x0401" } else { "0x0000" };
        format!(
            "lui sp, 0x801F\nlui s0, 0x8002\nlui s1, 0x1F80\nlui t0, 0x0000\nori t0, t0, 1\nsw t0, 0x1074(s1)\nlui t0, 0x4000\nori t0, t0, {sr}\nmtc0 t0, SR\nlui t0, 0x0012\nori t0, t0, 0x3456\nmtc2 t0, VXY0\nctc2 t0, TRX
loop: jal routine\nnop\nlui t9, 0x8001\nori t9, t9, 0x0100\njalr t9\nnop\nbgezal zero, routine\nnop\nj loop\nnop\n{}
routine: lw t1, 0(s0)\nnop\naddiu t1, t1, 1\nsw t1, 0(s0)\nsb t1, 0x10(s1)\nrtps\njr ra\nnop",
            "nop\n".repeat(41)
        )
    }

    /// The handler at 80000080h: I_STAT acknowledged, then back to EPC with RFE in the jump's delay slot.
    const HANDLER: &str = "
        lui k0, 0x1F80
        sw zero, 0x1070(k0)
        mfc0 k0, EPC
        nop
        jr k0
        rfe";

    fn machine(irq: bool) -> Machine {
        let mut m = machine_for(&program(&calls(irq)), Market::Americas);
        let handler = emusen_mips::assemble(HANDLER, 0x8000_0080, &blackmoon_r3000::dialect(None)).unwrap();
        for (i, w) in handler.iter().enumerate() {
            m.board.ram[0x80 + 4 * i..0x84 + 4 * i].copy_from_slice(&w.to_le_bytes());
        }
        assert_eq!(m.debug_disassemble(R3000, R3000BUS_ID, 0x8001_0100, 1).unwrap()[0].mnemonic, "lw", "the routine is at 80010100h");
        m
    }

    fn state(m: &Machine) -> Vec<u8> {
        let mut v = vec![0; m.state_size()];
        let n = m.save_state(&mut v).unwrap();
        v.truncate(n);
        v
    }

    fn arm(m: &mut Machine, flags: u32) {
        m.debug.hooks.configure(flags, i32::MIN, -1);
    }

    /// Runs the open frame to its end, each stop continued and a full log drained, as the host does; the stops for
    /// any other reason.
    fn run_out(m: &mut Machine, mut flags: u32) -> u32 {
        let mut stops = 0;
        loop {
            match m.run_frame_debug(flags) {
                stop::FRAME => return stops,
                stop::RING => drain(m),
                _ => stops += 1,
            }
            flags = run::UNCHECKED;
        }
    }

    fn drain(m: &mut Machine) {
        m.debug.hooks.calls_log.clear();
        m.debug.hooks.writes_log.clear();
        m.debug.hooks.reads_log.clear();
    }

    /// The pc of each stop under `flags` from a stop at `at`, `n` times.
    fn stops(m: &mut Machine, n: usize) -> Vec<u32> {
        (0..n)
            .map(|_| {
                assert_ne!(m.run_frame_debug(run::UNCHECKED), stop::FRAME);
                m.cpu.regs.pc
            })
            .collect()
    }

    fn stop_at(m: &mut Machine, pc: u32) {
        m.debug.hooks.set_breakpoints(&[pc as i32, pc as i32]);
        let why = m.run_frame_debug(0);
        assert_eq!((why, m.cpu.regs.pc), (stop::BREAKPOINT, pc));
        m.debug.hooks.set_breakpoints(&[]);
    }

    /// The kit's C15 in the crate: every table armed with nothing to hit, and a frame stopped before every instruction,
    /// give the plain frames' state, picture and sound, with the vertical blank interrupting calls in flight.
    #[test]
    fn an_armed_frame_with_nothing_to_hit_is_the_plain_frame() {
        let (mut plain, mut armed, mut stepped) = (machine(true), machine(true), machine(true));
        arm(&mut armed, flag::CALLS | flag::WRITES | flag::READS | flag::PROFILING | 1 << flag::COVERAGE);
        armed.debug.hooks.set_breakpoints(&[-2, -2]);
        armed.debug.hooks.set_ranges(0, &[RAM_ID, 0x1F_FFFC, 0x1F_FFFF, R3000BUS_ID, 0xFFFF_FFF0, 0xFFFF_FFFF]);
        armed.debug.hooks.set_ranges(1, &[SCRATCH_ID, 0x3FF, 0x3FF]);
        armed.debug.hooks.set_ranges(range::READ_WATCH, &[RAM_ID, 0x1F_FFFC, 0x1F_FFFF, R3000BUS_ID, 0xFFFF_FFF0, 0xFFFF_FFFF]);
        armed.debug.hooks.set_ranges(range::READ_BREAK, &[SCRATCH_ID, 0x3FF, 0x3FF]);
        arm(&mut stepped, flag::EACH | flag::CALLS);
        let mut steps = 0;
        for frame in 0..6 {
            plain.advance();
            assert_eq!(run_out(&mut armed, 0), 0, "frame {frame} stopped");
            drain(&mut armed);
            steps += run_out(&mut stepped, 0);
            drain(&mut stepped);
            for m in [&armed, &stepped] {
                assert_eq!(state(&plain), state(m), "frame {frame}");
                assert_eq!(plain.board.gpu.display.frame(), m.board.gpu.display.frame());
                assert_eq!(plain.board.spu.out.len(), m.board.spu.out.len());
            }
        }
        assert!(steps > 100_000, "{steps} stops");
        assert!(armed.debug.hooks.coverage[0].as_ref().unwrap().covered > 0 && !armed.debug.hooks.profile.is_empty());
    }

    /// A breakpoint stops in front of its instruction with nothing run, and a resume past it runs one pass of the loop.
    #[test]
    fn a_breakpoint_halts_in_front_of_its_instruction_and_resumes() {
        let mut m = machine(false);
        m.advance();
        m.debug.hooks.set_breakpoints(&[0x8001_0108u32 as i32, 0x8001_0108u32 as i32]);
        let (count, now) = (m.board.ram_word(0x20000), m.board.now);
        assert_eq!(m.run_frame_debug(0), stop::BREAKPOINT);
        assert_eq!((m.debug_pc(R3000), m.debug.stopped), (Some(0x8001_0108), R3000));
        let held = state(&m);
        assert_eq!(m.run_frame_debug(0), stop::BREAKPOINT, "asked again, the same stop");
        assert_eq!(held, state(&m), "a stop runs nothing");
        assert_eq!(m.run_frame_debug(run::UNCHECKED), stop::BREAKPOINT);
        assert_eq!((m.cpu.regs.pc, m.board.ram_word(0x20000)), (0x8001_0108, count + 1), "the routine's next call, this one's store run");
        assert!(m.board.now > now);
        m.debug.hooks.set_breakpoints(&[]);
        assert_eq!(m.run_frame_debug(run::UNCHECKED), stop::FRAME);
        assert_eq!(m.frames, 2);
    }

    /// `EACH` stops before each instruction in order: through a JAL, its delay slot and into the routine.
    #[test]
    fn each_stops_in_front_of_every_instruction_in_order() {
        let mut m = machine(false);
        m.advance();
        stop_at(&mut m, 0x8001_0034);
        arm(&mut m, flag::EACH | flag::CALLS);
        let r = 0x8001_0100;
        assert_eq!(stops(&mut m, 10), [0x8001_0038, r, r + 4, r + 8, r + 12, r + 16, r + 20, r + 24, r + 28, 0x8001_003C]);
    }

    /// The call stack: a JAL's call with its source and target, its return once the JR's delay slot has run, and the
    /// depth targets that make step over and step out.
    #[test]
    fn the_call_stack_follows_the_calls_and_steps_over_and_out_of_them() {
        let mut m = machine(false);
        m.advance();
        stop_at(&mut m, 0x8001_0034);
        arm(&mut m, flag::CALLS | flag::EACH);
        drain(&mut m);
        stops(&mut m, 1);
        assert_eq!((m.debug.hooks.depth(), m.debug.hooks.stack[0]), (1, (0x8001_0034, 0x8001_0100)), "pushed at the JAL");
        stops(&mut m, 8);
        assert_eq!((m.cpu.regs.pc, m.debug.hooks.depth()), (0x8001_011C, 1), "still in the routine in the JR's delay slot");
        stops(&mut m, 1);
        assert_eq!((m.cpu.regs.pc, m.debug.hooks.depth()), (0x8001_003C, 0), "returned once the slot ran");

        m.debug.hooks.configure(flag::CALLS, i32::MIN, -1);
        let count = m.board.ram_word(0x20000);
        stop_at(&mut m, 0x8001_0044);
        m.debug.hooks.configure(flag::CALLS, 0, -1);
        assert_eq!(m.run_frame_debug(run::UNCHECKED), stop::DEPTH);
        assert_eq!((m.cpu.regs.pc, m.board.ram_word(0x20000)), (0x8001_004C, count + 1), "stepped over the JALR, the routine run whole");

        m.debug.hooks.configure(flag::CALLS, i32::MIN, -1);
        stop_at(&mut m, 0x8001_010C);
        assert_eq!(m.debug.hooks.depth(), 1);
        m.debug.hooks.configure(flag::CALLS, 0, -1);
        assert_eq!(m.run_frame_debug(run::UNCHECKED), stop::DEPTH);
        assert_eq!(m.cpu.regs.pc, 0x8001_0054, "stepped out of the BGEZAL's call to the instruction after its slot");
        let calls: Vec<_> = m.debug.hooks.calls_log.iter().filter(|c| c.kind == kind::CALL).map(|c| (c.source, c.target)).collect();
        assert!(calls.contains(&(0x8001_0044, 0x8001_0100)) && calls.contains(&(0x8001_004C, 0x8001_0100)), "{calls:x?}");
    }

    /// An interrupt is an IRQ frame, from the interrupted instruction to the vector, and RFE in the JR's slot ends it; an
    /// interrupt taken in a call's delay slot abandons the call, which runs again after it, so the depth stays exact.
    #[test]
    fn interrupts_push_and_rfe_pops_and_a_call_interrupted_in_its_slot_is_abandoned() {
        let mut m = machine(true);
        arm(&mut m, flag::CALLS | flag::INTERRUPTS);
        let (mut handled, mut in_slot) = (0, 0);
        for _ in 0..240 {
            loop {
                let why = m.run_frame_debug(run::UNCHECKED);
                if why == stop::FRAME {
                    break;
                }
                if why == stop::RING {
                    drain(&mut m);
                    continue;
                }
                assert_eq!((why, m.cpu.regs.pc), (stop::INTERRUPT, 0x8000_0080));
                let top = *m.debug.hooks.stack.last().unwrap();
                assert_eq!(top.1, 0x8000_0080);
                assert!(m.debug.hooks.depth() <= 2, "{:x?}", m.debug.hooks.stack);
                if m.cpu.regs.cause & 1 << 31 != 0 && [0x8001_0034, 0x8001_0044, 0x8001_004C].contains(&m.cpu.regs.epc) {
                    in_slot += 1;
                    assert_eq!(m.debug.hooks.depth(), 1, "the call in flight abandoned");
                }
                handled += 1;
            }
        }
        assert!(handled >= 200 && in_slot > 0, "{handled} interrupts, {in_slot} in a call's slot");
        assert_eq!(m.debug.hooks.unmatched_returns, 0);
        arm(&mut m, flag::CALLS);
        stop_at(&mut m, 0x8001_0034);
        assert_eq!(m.debug.hooks.depth(), 0);
    }

    /// Stores are reported byte by byte with the storing instruction: in the bus at the address stored to, and in RAM or
    /// the scratchpad at the offset it lands at; a data breakpoint stops in front of the next instruction, the store run.
    #[test]
    fn watches_see_stores_in_the_bus_and_in_the_memory_they_land_in() {
        let mut m = machine(false);
        m.advance();
        arm(&mut m, flag::WRITES);
        m.debug.hooks.set_ranges(0, &[RAM_ID, 0x20000, 0x20003, SCRATCH_ID, 0x10, 0x10, R3000BUS_ID, 0x8002_0000, 0x8002_0000]);
        run_out(&mut m, 0);
        let (count, scratch) = (m.board.ram_word(0x20000), m.board.scratch[0x10] as u32);
        let log = &m.debug.hooks.writes_log;
        let last = |space: u32| log.iter().rev().find(|w| w.space == space).copied().unwrap();
        assert_eq!((last(RAM_ID).address, last(RAM_ID).value, last(RAM_ID).pc), (0x20003, count >> 24, 0x8001_010C));
        assert_eq!((last(SCRATCH_ID).address, last(SCRATCH_ID).value, last(SCRATCH_ID).pc), (0x10, scratch, 0x8001_0110));
        assert_eq!((last(R3000BUS_ID).address, last(R3000BUS_ID).value), (0x8002_0000, count & 0xFF));
        let (words, bytes) = (log.iter().filter(|w| w.space == RAM_ID).count(), log.iter().filter(|w| w.space == SCRATCH_ID).count());
        assert!(words == 4 * bytes || words == 4 * bytes + 4, "a word stored for each byte, the frame ending between them or not: {words} {bytes}");

        m.debug.hooks.set_ranges(1, &[SCRATCH_ID, 0x10, 0x10]);
        assert_eq!(m.run_frame_debug(0), stop::DATA);
        assert_eq!((m.cpu.regs.pc, m.board.scratch[0x10] as u32), (0x8001_0114, m.board.ram_word(0x20000) & 0xFF));
    }

    /// A loop at 80010010h whose loads read RAM and the scratchpad in every width: LW, LBU, LWL and LWR of two bytes each,
    /// LH, and LWC2 into VXY0, then a count stored back; RAM 80020004h-8002000Bh and the scratchpad's 10h hold constants.
    fn loads() -> Machine {
        let source = "lui s0, 0x8002\nlui s1, 0x1F80\nlui t0, 0x4000\nmtc0 t0, SR
loop: lw t1, 0(s0)\nlbu t2, 0x10(s1)\nlwl t3, 0x5(s0)\nlwr t3, 0x6(s0)\nlh t4, 0x22(s1)\nlwc2 VXY0, 0x8(s0)\naddiu t1, t1, 1\nsw t1, 0(s0)\nsb t1, 0x22(s1)\nj loop\nnop";
        let mut m = machine_for(&program(source), Market::Americas);
        m.board.ram[0x2_0004..0x2_000C].copy_from_slice(&[0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]);
        m.board.scratch[0x10] = 0x5A;
        m
    }

    /// Loads are reported byte by byte once read, with the loading instruction: in the bus at the address read, and in RAM
    /// or the scratchpad at the offset it lands at; LWL and LWR at the bytes of the word they reach, LWC2 at its word.
    #[test]
    fn read_watches_see_loads_in_the_bus_ram_and_the_scratchpad() {
        let mut m = loads();
        arm(&mut m, flag::READS);
        m.debug.hooks.set_ranges(range::READ_WATCH, &[RAM_ID, 0x2_0000, 0x2_000B, SCRATCH_ID, 0x10, 0x10, SCRATCH_ID, 0x22, 0x23, R3000BUS_ID, 0x8002_0000, 0x8002_0003]);
        m.debug.hooks.capacity = 1 << 24;
        assert_eq!(run_out(&mut m, 0), 0);
        let log = m.debug.hooks.reads_log.clone();
        assert!(log.iter().all(|r| r.kind == range::READ_WATCH));
        let at = |pc: u32, space: u32, n: usize| -> Vec<(u32, u32)> { log.iter().filter(|r| r.pc == pc && r.space == space).map(|r| (r.address, r.value)).take(n).collect() };
        let base = 0x8001_0010;
        assert_eq!(at(base + 8, RAM_ID, 2), [(0x2_0004, 0x11), (0x2_0005, 0x22)], "LWL at 80020005h reads the word's first two bytes");
        assert_eq!(at(base + 12, RAM_ID, 2), [(0x2_0006, 0x33), (0x2_0007, 0x44)], "LWR at 80020006h reads its last two");
        assert_eq!(at(base + 20, RAM_ID, 4), [(0x2_0008, 0x55), (0x2_0009, 0x66), (0x2_000A, 0x77), (0x2_000B, 0x88)], "LWC2");
        assert_eq!(at(base + 4, SCRATCH_ID, 1), [(0x10, 0x5A)]);
        assert_eq!(at(base + 16, SCRATCH_ID, 2).iter().map(|r| r.0).collect::<Vec<_>>(), [0x22, 0x23]);
        let words: Vec<u32> = log.iter().filter(|r| r.pc == base && r.space == RAM_ID).collect::<Vec<_>>().chunks(4).map(|c| c.iter().enumerate().fold(0, |w, (i, r)| w | r.value << (8 * i))).collect();
        assert!(words.len() > 1000 && words.windows(2).all(|p| p[1] == p[0] + 1), "each pass's LW reads the count the last one stored");
        let bus: Vec<_> = log.iter().filter(|r| r.space == R3000BUS_ID).map(|r| (r.address - 0x8000_0000, r.value, r.pc)).collect();
        let ram: Vec<_> = log.iter().filter(|r| r.space == RAM_ID && r.address < 0x2_0004).map(|r| (r.address, r.value, r.pc)).collect();
        assert_eq!(bus, ram, "the bus's watch and RAM's name the same loads");
        assert_eq!(m.debug_registers(GTE).unwrap()[0], 0x8877_6655, "the LWC2 ran");
    }

    /// A read data breakpoint stops in front of the load with nothing of it run, logging each byte it covers with the
    /// value its space holds there; asked again it stops again, and resumed the load runs. In the bus or in RAM it names
    /// the bytes LWL and LWR reach, not the words around them.
    #[test]
    fn a_read_data_breakpoint_halts_in_front_of_the_load_and_resumes() {
        let mut m = loads();
        m.advance();
        arm(&mut m, flag::READS);
        m.debug.hooks.set_ranges(range::READ_BREAK, &[SCRATCH_ID, 0x22, 0x22]);
        assert_eq!(m.run_frame_debug(0), stop::READ);
        assert_eq!(m.cpu.regs.pc, 0x8001_0020, "in front of the LH");
        let (held, now, value) = (state(&m), m.board.now, m.board.scratch[0x22] as u32);
        assert_eq!(m.run_frame_debug(0), stop::READ, "asked again, the same stop");
        assert_eq!((held, now), (state(&m), m.board.now), "a stop runs nothing");
        let logged: Vec<_> = m.debug.hooks.reads_log.iter().map(|r| (r.space, r.address, r.value, r.pc, r.kind)).collect();
        assert_eq!(logged, [(SCRATCH_ID, 0x22, value, 0x8001_0020, range::READ_BREAK); 2]);
        drain(&mut m);
        assert_eq!(m.run_frame_debug(run::UNCHECKED), stop::READ);
        assert_eq!(m.cpu.regs.pc, 0x8001_0020, "the next pass's LH");
        assert!(m.board.now > now);
        assert_eq!(m.cpu.regs.r[12] as u16, u16::from_le_bytes([value as u8, m.board.scratch[0x23]]), "the load ran once resumed");

        for (space, address, pc) in [(R3000BUS_ID, 0x8002_0006, 0x8001_001C), (R3000BUS_ID, 0x8002_0005, 0x8001_0018), (RAM_ID, 0x2_0005, 0x8001_0018), (RAM_ID, 0x2_000B, 0x8001_0024)] {
            m.debug.hooks.set_ranges(range::READ_BREAK, &[space, address, address]);
            drain(&mut m);
            assert_eq!((m.run_frame_debug(run::UNCHECKED), m.cpu.regs.pc), (stop::READ, pc), "{space} {address:08X}");
            assert_eq!(m.debug.hooks.reads_log.len(), 1, "{space} {address:08X}");
        }
        m.debug.hooks.set_ranges(range::READ_BREAK, &[R3000BUS_ID, 0xA002_0005, 0xA002_0005]);
        assert_eq!(run_out(&mut m, run::UNCHECKED), 0, "KSEG1's mirror is another bus address than the KSEG0 the loop reads");
    }

    /// Coverage marks each address that ran, KSEG0's at its 24 bits, and the profile charges the routine its instructions.
    #[test]
    fn coverage_and_the_profile_record_what_ran() {
        let mut m = machine(false);
        m.advance();
        arm(&mut m, flag::CALLS | flag::PROFILING | 1 << flag::COVERAGE);
        run_out(&mut m, 0);
        let c = m.debug.hooks.coverage[0].as_ref().unwrap();
        let ran = |a: u32| c.bits[(a as usize & 0xFF_FFFF) >> 3] & 1 << (a & 7) != 0;
        assert!((0x8001_0100..0x8001_0120).step_by(4).all(ran) && (0x8001_0034..=0x8001_0058).step_by(4).all(ran));
        assert!(!ran(0x8001_005C) && !ran(0x8001_0101) && !ran(0x8001_0000), "the word after the loop, a byte inside a word, the start-up");
        let covered = c.covered;
        m.debug.hooks.flush();
        let (routine, outside) = (m.debug.hooks.charged(0x8001_0100), m.debug.hooks.charged(0));
        assert_eq!(m.debug.hooks.profile.len(), 2);
        assert_eq!(routine + outside, covered, "every instruction charged once");
        // A pass is seven instructions outside and three calls of nine: the routine's eight and the call's delay slot.
        let ratio = routine as f64 / outside as f64;
        assert!((ratio - 27.0 / 7.0).abs() < 0.01, "{routine} {outside}");
    }

    /// The registers in machine info's order, the R3000A's and the GTE's, as the processor and the engine hold them.
    #[test]
    fn the_registers_are_the_processors_and_the_gtes() {
        let mut m = machine(false);
        m.advance();
        let r = m.debug_registers(R3000).unwrap();
        assert_eq!(r.len(), blackmoon_r3000::DEBUG_REGISTERS.len());
        assert_eq!((r[16], r[29], r[34], r[35]), (0x8002_0000, 0x801F_0000, m.cpu.regs.pc as i64, m.cpu.regs.sr as i64));
        let g = m.debug_registers(GTE).unwrap();
        assert_eq!(g.len(), 64);
        assert_eq!((g[0], g[32 + 5]), (0x0012_3456, 0x0012_3456), "VXY0 and TRX as the program set them");
        assert_eq!(g, (0..64).map(|n| m.board.gte.register(n) as i64).collect::<Vec<_>>());
        assert!(m.debug_registers(2).is_none() && m.debug_pc(2).is_none());
        assert_eq!(m.debug_pc(GTE), Some(0));
    }

    /// The listing reads the bus as the processor would, RAM's offsets as code there runs, GTE commands and registers by
    /// their names, and the static references the bytes give.
    #[test]
    fn code_lists_from_the_bus_and_from_ram_with_the_gtes_names() {
        let m = machine(false);
        let text = |v: Vec<Instruction>| v.iter().map(|i| format!("{} {}", i.mnemonic, i.operands).trim().to_owned()).collect::<Vec<_>>();
        let bus = m.debug_disassemble(R3000, R3000BUS_ID, 0x8001_0034, 3).unwrap();
        assert_eq!(text(bus.clone()), ["jal 0x80010100", "nop", "lui t9, 0x8001"]);
        assert_eq!(bus[0].reference, Some((Reference::Call, 0x8001_0100)));
        assert_eq!(text(m.debug_disassemble(R3000, RAM_ID, 0x1_0034, 3).unwrap()), text(bus));
        assert_eq!(m.debug_disassemble(R3000, RAM_ID, 0x1_0036, 1).unwrap()[0].address, 0x1_0034, "a word's boundary");
        assert_eq!(text(m.debug_disassemble(R3000, R3000BUS_ID, 0x8001_002C, 2).unwrap()), ["mtc2 t0, VXY0", "ctc2 t0, TRX"]);
        assert_eq!(text(m.debug_disassemble(R3000, R3000BUS_ID, 0x8001_0114, 1).unwrap()), ["rtps"]);
        let bgezal = &m.debug_disassemble(R3000, R3000BUS_ID, 0x8001_004C, 1).unwrap()[0];
        assert_eq!((bgezal.mnemonic.as_str(), bgezal.reference), ("bgezal", Some((Reference::Call, 0x8001_0100))));
        assert_eq!(reference(0xAC08_1070, 0), Some((Reference::Write, 0x1070)), "sw t0, 0x1070(zero)");
        assert_eq!(reference(0x8C08_FFF0, 0), Some((Reference::Read, 0xFFFF_FFF0)));
        assert_eq!(m.debug_disassemble(R3000, R3000BUS_ID, 0x1F80_1070, 1).unwrap()[0].bytes, [0xFF; 4], "the ports are not read");
        assert!(m.debug_disassemble(GTE, R3000BUS_ID, 0, 1).is_none() && m.debug_disassemble(R3000, 99, 0, 1).is_none());
    }

    /// A state loaded while a frame is stopped closes it: the next frame is a whole one, as the plain run's.
    #[test]
    fn a_state_loaded_at_a_stop_starts_a_whole_frame() {
        let (mut m, mut plain) = (machine(false), machine(false));
        m.advance();
        plain.advance();
        let saved = state(&m);
        stop_at(&mut m, 0x8001_0100);
        m.load_state(&saved).unwrap();
        assert!(!m.debug.open);
        assert_eq!(run_out(&mut m, 0), 0);
        plain.advance();
        assert_eq!(state(&plain), state(&m));
    }
}
