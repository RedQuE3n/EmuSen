//! The debugger (stage S31, Sapphire_Plan.md §4.7, Sapphire_Native.md §41): the generic one over `DEBUG`,
//! `DEBUG_REGISTERS`, `DEBUG_DISASSEMBLE` and `DEBUG_READS`, with four processors. The EE is observed by a frame of its
//! own that asks the shared hooks before each instruction and reads what the instruction did after it, the interpreter
//! untouched; the IOP and the two VUs carry a probe beside their units while the debugger fits one, which stops each in
//! front of a breakpoint. Registers at their widths, disassembly by `emusen-mips`'s tables. The plain frame meets
//! none of it.

use emusen_native::core::{Instruction, Reference};
use emusen_native::debug::{Hooks, kind, run, stop};

use crate::board::System;
use crate::ee::Place;
use crate::machine::{BIOS_ID, Machine, RAM_ID, Stop};

/// The processors' numbers in machine info.
pub const EE: u32 = 0;
pub const IOP: u32 = 1;
pub const VU0: u32 = 2;
pub const VU1: u32 = 3;

/// The memories the listings and the watches name, by their space ids (`machine::MEMORIES`).
pub const SCRATCH_ID: u32 = 2;
pub const VU0MICRO_ID: u32 = 4;
pub const VU1MICRO_ID: u32 = 6;
pub const IOPRAM_ID: u32 = 8;

/// Coverage's widths: the host's registry keeps 24 bits of address for the EE and the IOP, so mirrors share a bit; each
/// VU's micro memory by its byte address, 4 KiB and 16 KiB.
pub const COVERAGE_BITS: [u32; 4] = [24, 24, 12, 14];

/// The EE instructions a frame runs between its checks of the field, as the plain frame runs them (`Machine::advance`).
const BATCH: u32 = 1024;
/// EE cycles a step of the board while no processor runs, as the plain frame's.
const IDLE_STEP: u64 = 16_384;

/// What the debugger fits beside the IOP or a VU: that processor's breakpoints and coverage, and where it stopped.
#[derive(Clone, Debug, Default)]
pub struct Probe {
    /// Breakpoints as (first, last), both included, compared as the host's registry compares its `int`s.
    pub breakpoints: Vec<(i32, i32)>,
    pub coverage: bool,
    /// The addresses run since the debugger last took them, while coverage is on.
    pub steps: Vec<u32>,
    /// The address the processor stands in front of, held there by a breakpoint.
    pub halted: Option<u32>,
    /// The step in front of which the processor stopped is to run unasked.
    pub resume: bool,
    /// A breakpoint's address run where the processor could not stop: inside an EE instruction that waits on it or
    /// reads it, or a DMA event's run; the frame stops after that instruction instead (Sapphire_Native.md §41.4).
    pub passed: Option<u32>,
}

impl Probe {
    /// Whether the step at `pc` may run: not where a breakpoint covers it while the processor `may_halt`, unless it is
    /// the one being resumed; a breakpoint where it may not halt is noted as passed.
    #[inline]
    pub fn admit(&mut self, pc: u32, may_halt: bool) -> bool {
        if !std::mem::take(&mut self.resume) && self.breakpoints.iter().any(|&(a, b)| (pc as i32) >= a && (pc as i32) <= b) {
            if may_halt {
                self.halted = Some(pc);
                return false;
            }
            self.passed.get_or_insert(pc);
        }
        self.halted = None;
        if self.coverage {
            self.steps.push(pc);
        }
        true
    }
}

/// The hooks the host pushes its tables into, the other processors' breakpoints, and the frame a stop left open.
pub struct Debugger {
    pub hooks: Hooks,
    /// The IOP's, VU0's and VU1's breakpoints, which their probes take at each frame.
    pub breakpoints: [Vec<(i32, i32)>; 3],
    pub open: bool,
    pub stopped: u32,
    /// The field the open frame ends at, and the EE instructions left in its current batch.
    target: u64,
    batch: u32,
    /// A `jr ra` whose delay slot has not yet run; its return is noted after the slot.
    returning: bool,
    /// A call whose delay slot has not yet run; an exception taken in the slot abandons it, EPC being the call's.
    calling: bool,
}

impl Default for Debugger {
    fn default() -> Debugger {
        Debugger::new()
    }
}

impl Debugger {
    pub fn new() -> Debugger {
        Debugger { hooks: Hooks::new(&COVERAGE_BITS), breakpoints: Default::default(), open: false, stopped: EE, target: 0, batch: 0, returning: false, calling: false }
    }

    /// A state loaded: the open frame and the events in flight belong to the machine that was.
    pub fn close(&mut self) {
        self.open = false;
        self.batch = 0;
        self.returning = false;
        self.calling = false;
    }
}

/// The probe slot of processor `n` beside its unit: the IOP's, or a VU's micro mode's.
fn slot(sys: &mut System, n: u32) -> Option<&mut Option<Box<Probe>>> {
    match n {
        IOP => Some(&mut sys.board.iop.probe),
        VU0 => sys.board.vpu.micro0.probe(),
        VU1 => sys.board.vpu.micro1.probe(),
        _ => None,
    }
}

/// The R5900's registers as `debug_registers` lists them: each 128-bit general register as its two halves, bits 63..0
/// under its name and bits 127..64 as `.hi`, the core ABI's register value being 64 bits (Sapphire_Native.md §41.3);
/// the PC, HI and LO with pipeline 1's HI1 and LO1, SA; COP0 by the TX79 manual's names; the debug registers; the
/// performance counters; COP1's registers, its accumulator and FCR31.
pub fn ee_registers() -> Vec<(String, u32)> {
    let mut v = Vec::with_capacity(156);
    for n in emusen_mips::GPR_NAMES {
        v.push((n.to_string(), 64));
        v.push((format!("{n}.hi"), 64));
    }
    v.extend([("pc", 32), ("hi", 64), ("hi1", 64), ("lo", 64), ("lo1", 64), ("sa", 32)].map(|(n, b)| (n.to_string(), b)));
    for (i, n) in emusen_mips::r5900::COP0_NAMES.iter().enumerate() {
        v.push((if n.is_empty() { format!("${i}") } else { n.to_string() }, 32));
    }
    v.extend(["BPC", "IAB", "IABM", "DAB", "DABM", "DVB", "DVBM", "PCCR", "PCR0", "PCR1"].map(|n| (n.to_string(), 32)));
    v.extend((0..32).map(|i| (format!("f{i}"), 32)));
    v.extend([("ACC", 32), ("FCR31", 32)].map(|(n, b)| (n.to_string(), b)));
    v.extend(IDLE_EE.map(|n| (n.to_string(), 64)));
    v
}

/// Idle skipping's counts, read-only, after each processor's registers (Sapphire_Native.md §47.9): the EE's skips, the
/// cycles and steps they passed and the turns passed while a VIF, the GIF or a VU ran; the IOP's cycles and instructions
/// passed. Counts only, never an address.
pub const IDLE_EE: [&str; 4] = ["idle.skips", "idle.cycles", "idle.steps", "idle.busy"];
pub const IDLE_IOP: [&str; 2] = ["idle.cycles", "idle.instructions"];

/// A VU's registers: each vector register's four fields x to w, ACC's, the integer registers, I, Q, P, R, the status,
/// MAC and clipping flags, TPC and the PC, each at its width (VU User's Manual §3.1, §3.3).
pub fn vu_registers(unit: usize) -> Vec<(String, u32)> {
    let fields = |r: &str| ["x", "y", "z", "w"].map(|f| (format!("{r}.{f}"), 32));
    let mut v: Vec<(String, u32)> = (0..32).flat_map(|i| fields(&format!("vf{i}"))).collect();
    v.extend(fields("acc"));
    v.extend((0..16).map(|i| (format!("vi{i}"), 16)));
    v.extend([("I", 32), ("Q", 32), ("P", 32), ("R", 23), ("status", 12), ("mac", 16), ("clip", 24), ("TPC", 16), ("PC", if unit == 0 { 12 } else { 14 })].map(|(n, b)| (n.to_string(), b)));
    v
}

/// The IOP's registers, `blackmoon-r3000`'s list, each 32 bits.
pub fn iop_registers() -> Vec<(String, u32)> {
    let mut v: Vec<(String, u32)> = blackmoon_r3000::DEBUG_REGISTERS.iter().map(|&n| (n.to_string(), 32)).collect();
    v.extend(IDLE_IOP.map(|n| (n.to_string(), 64)));
    v
}

/// Where code in a memory runs, for a listing's branch and jump targets: the EE's programs in kuseg, where the project's
/// and an ELF's are linked, the scratchpad at 70000000h, the boot ROM in kseg1; the IOP's RAM in kuseg.
fn runs_at(processor: u32, space: u32) -> u32 {
    match (processor, space) {
        (EE, SCRATCH_ID) => 0x7000_0000,
        (_, BIOS_ID) => 0xBFC0_0000,
        _ => 0,
    }
}

/// The static reference of the decoder's kind as the core ABI names it.
fn reference(r: emusen_mips::vr4300::Reference) -> Reference {
    match r {
        emusen_mips::vr4300::Reference::Call => Reference::Call,
        emusen_mips::vr4300::Reference::Read => Reference::Read,
        emusen_mips::vr4300::Reference::Write => Reference::Write,
    }
}

impl Machine {
    /// The frame through the observed loop: the stop's reasons, zero at the frame's end. A stop leaves the frame open,
    /// the step in front of it not run, and the next call goes on from it; `run::UNCHECKED` runs that step unasked on
    /// the processor last stopped on. The EE's steps follow the plain frame's order exactly, its batches included.
    pub fn run_frame_debug(&mut self, flags: u32) -> u32 {
        let mut unchecked = flags & run::UNCHECKED != 0;
        if self.arm_probes() {
            self.catch_up();
        }
        if !self.debug.open {
            self.debug.open = true;
            self.debug.target = self.sys.board.pcrtc.fields + 1;
            self.debug.batch = 0;
        }
        if let Some(n) = self.halted() {
            if !(unchecked && n == self.debug.stopped) {
                self.debug.stopped = n;
                return stop::BREAKPOINT;
            }
            if let Some(Some(p)) = slot(&mut self.sys, n) {
                p.resume = true;
            }
            self.catch_up();
            unchecked = false;
            if let Some(n) = self.observe_others() {
                self.debug.stopped = n;
                return stop::BREAKPOINT;
            }
        } else if self.debug.stopped != EE {
            unchecked = false;
        }
        loop {
            if self.debug.batch == 0 {
                if self.sys.board.pcrtc.fields >= self.debug.target {
                    break;
                }
                if self.booted && self.stop.is_none() && self.waiting() {
                    if self.at_entry() {
                        self.enter_observed();
                        continue;
                    }
                    self.debug.batch = 1;
                } else if self.booted && self.stop.is_none() {
                    self.debug.batch = BATCH;
                } else {
                    // A held EE stands in front of its instruction; each step of the board alone is a step it waits.
                    if !std::mem::take(&mut unchecked) {
                        let why = self.debug.hooks.stop_before(self.sys.ee.pc);
                        if why != stop::FRAME {
                            self.debug.stopped = EE;
                            return why;
                        }
                    }
                    let System { ee, board } = &mut *self.sys;
                    board.run_to(board.now + IDLE_STEP, &mut ee.spram[..]);
                    if let Some(n) = self.observe_others() {
                        self.debug.stopped = n;
                        return stop::BREAKPOINT;
                    }
                    continue;
                }
            }
            if let Some(why) = self.ee_step(&mut unchecked) {
                self.debug.stopped = EE;
                return why;
            }
            if let Some(n) = self.observe_others() {
                self.debug.stopped = n;
                return stop::BREAKPOINT;
            }
        }
        if self.booted {
            self.sys.board.iop.resolve_tty();
        }
        self.frames += 1;
        self.debug.open = false;
        self.debug.stopped = EE;
        stop::FRAME
    }

    /// One EE instruction observed: the hooks asked first, unless the step runs unasked; then the step, and what it
    /// did noted. A stop's reasons, the instruction not run.
    fn ee_step(&mut self, unchecked: &mut bool) -> Option<u32> {
        let pc = self.sys.ee.pc;
        let h = &self.debug.hooks;
        let word = if h.calls || h.writes || h.reads { self.code(pc) } else { None };
        let reach = word.and_then(|w| self.sys.ee.reach(w));
        if !std::mem::take(unchecked) {
            let mut why = self.debug.hooks.stop_before(pc);
            if let Some(r) = reach.filter(|r| !r.store && self.debug.hooks.reads && !self.debug.hooks.read_break_ranges.is_empty()) {
                for i in 0..r.bytes {
                    if let Some((space, at, value)) = self.byte(r.vaddr.wrapping_add(i)) {
                        why |= if self.debug.hooks.note_read_ahead(space, at, value, pc) { stop::READ } else { 0 };
                    }
                }
            }
            if why != stop::FRAME {
                return Some(why);
            }
        }
        self.debug.hooks.record(pc);
        let before = self.taken();
        let step = self.sys.step();
        self.debug.batch -= 1;
        if step.is_err() {
            self.stop = Some(Stop::Unimplemented);
            self.debug.batch = 0;
            return None;
        }
        self.noted(pc, word, reach, before);
        None
    }

    /// The fast boot's entry (S10), its call of ExecPS2 noted as a system call when calls are tracked.
    fn enter_observed(&mut self) {
        let (pc, before) = (self.sys.ee.pc, self.taken());
        self.enter_program();
        if self.taken() != before {
            self.debug.returning = false;
            self.debug.calling = false;
            self.debug.hooks.note_call(pc, self.sys.ee.pc, kind::BRK);
        }
    }

    /// The exceptions the EE has taken, by kind: interrupts, system calls and breakpoints, level 2, the rest.
    fn taken(&self) -> [u64; 4] {
        let e = &self.sys.ee;
        let all: u64 = e.exceptions.iter().sum();
        [e.exceptions[0], e.exceptions[8] + e.exceptions[9], e.exceptions2.iter().sum(), all]
    }

    /// What the instruction at `pc` did, for the hooks: an exception taken as a call of its kind (abandoning a call
    /// whose delay slot it interrupted); else its stores and loads byte by byte, a call once taken, a return once the
    /// jump's delay slot has run, ERET's return.
    fn noted(&mut self, pc: u32, word: Option<u32>, reach: Option<crate::ee::observe::Reach>, before: [u64; 4]) {
        let after = self.taken();
        let d = &mut self.debug;
        if after != before {
            d.returning = false;
            if std::mem::take(&mut d.calling) {
                d.hooks.note_return();
            }
            let k = if after[1] != before[1] {
                kind::BRK
            } else if after[2] != before[2] {
                kind::NMI
            } else {
                kind::IRQ
            };
            d.hooks.note_call(pc, self.sys.ee.pc, k);
            return;
        }
        if std::mem::take(&mut d.returning) {
            d.hooks.note_return();
        }
        if let Some(r) = reach {
            let (writes, reads) = (self.debug.hooks.writes, self.debug.hooks.reads);
            if (r.store && writes) || (!r.store && reads) {
                for i in 0..r.bytes {
                    if let Some((space, at, value)) = self.byte(r.vaddr.wrapping_add(i)) {
                        if r.store {
                            self.debug.hooks.note_write(space, at, value, pc);
                        } else {
                            self.debug.hooks.note_read(space, at, value, pc);
                        }
                    }
                }
            }
        }
        let Some(w) = word else { return };
        let ee = &self.sys.ee;
        let slot = ee.in_delay_slot();
        let d = &mut self.debug;
        let name = match emusen_mips::r5900::operation(w) {
            Some(emusen_mips::r5900::Op::Cpu(name, _)) => name,
            _ => "",
        };
        let call = match name {
            "jal" | "jalr" => slot,
            "bltzal" | "bgezal" | "bltzall" | "bgezall" => slot && ee.next_pc() != pc.wrapping_add(8),
            _ => false,
        };
        if call {
            d.hooks.note_call(pc, ee.next_pc(), kind::CALL);
        }
        match name {
            "jr" if w >> 21 & 31 == 31 => {
                if slot {
                    d.returning = true;
                } else {
                    d.hooks.note_return();
                }
            }
            "eret" => d.hooks.note_return(),
            _ => {}
        }
        d.calling = call;
    }

    /// The word at the EE's `pc` as a fetch would read it, nothing changed.
    fn code(&self, pc: u32) -> Option<u32> {
        let b = &self.sys.board;
        self.sys.ee.peek_code(&b.map.rdram, &b.map.rom, pc)
    }

    /// The byte at the EE's `vaddr` in the space a watch names it in: RAM or the scratchpad, at the offset it lands at,
    /// with its value as the core sees it; none elsewhere.
    fn byte(&self, vaddr: u32) -> Option<(u32, u32, u8)> {
        let ee = &self.sys.ee;
        let place = ee.place(vaddr)?;
        let (space, at) = match place {
            Place::Scratchpad(o) => (SCRATCH_ID, o & 0x3FFF),
            Place::Physical(p, _) if p < 32 << 20 => (RAM_ID, p),
            Place::Physical(..) => return None,
        };
        let b = &self.sys.board;
        Some((space, at, ee.peek_byte(&b.map.rdram, &b.map.rom, place)?))
    }

    /// Fits each probe the tables ask for and removes the others; whether a processor held by a probe now removed has
    /// to be brought to the board's time.
    fn arm_probes(&mut self) -> bool {
        let mut released = false;
        for n in IOP..=VU1 {
            let breakpoints = self.debug.breakpoints[n as usize - 1].clone();
            let coverage = self.debug.hooks.coverage.get(n as usize).is_some_and(Option::is_some);
            let Some(s) = slot(&mut self.sys, n) else { continue };
            if breakpoints.is_empty() && !coverage {
                released |= s.take().is_some_and(|p| p.halted.is_some());
                continue;
            }
            let p = s.get_or_insert_with(Default::default);
            p.breakpoints = breakpoints;
            p.coverage = coverage;
        }
        released
    }

    /// A state loaded: the frame a stop left open is closed and no probe holds its processor, the state being the whole
    /// machine.
    pub fn forget_stop(&mut self) {
        self.debug.close();
        for n in IOP..=VU1 {
            if let Some(Some(p)) = slot(&mut self.sys, n) {
                (p.halted, p.resume, p.passed) = (None, false, None);
                p.steps.clear();
            }
        }
    }

    /// A plain frame runs with no probe fitted and no frame of the debugger's open.
    pub fn disarm(&mut self) {
        if !self.debug.open && self.sys.board.iop.probe.is_none() && (VU0..=VU1).all(|n| slot(&mut self.sys, n).is_none_or(|s| s.is_none())) {
            return;
        }
        let mut held = false;
        for n in IOP..=VU1 {
            if let Some(s) = slot(&mut self.sys, n) {
                held |= s.take().is_some_and(|p| p.halted.is_some());
            }
        }
        if held {
            self.catch_up();
        }
        self.debug.close();
    }

    /// The board brought again to its own time, as the end of the EE's last step brought it, so that a processor a probe
    /// held runs what it was asked to; then the EE's lines set from it, as `System::step` sets them.
    fn catch_up(&mut self) {
        let System { ee, board } = &mut *self.sys;
        board.run_to(board.now, &mut ee.spram[..]);
        (ee.int0, ee.int1, ee.cpcond0) = board.lines();
    }

    /// The first processor a probe holds in front of a breakpoint.
    fn halted(&mut self) -> Option<u32> {
        (IOP..=VU1).find(|&n| matches!(slot(&mut self.sys, n), Some(Some(p)) if p.halted.is_some() && !p.resume))
    }

    /// The other processors' steps since the last call taken into coverage; the first one a breakpoint holds.
    fn observe_others(&mut self) -> Option<u32> {
        let mut held = None;
        for n in IOP..=VU1 {
            let Some(Some(p)) = slot(&mut self.sys, n) else { continue };
            for pc in p.steps.drain(..) {
                self.debug.hooks.record_on(n as usize, pc);
            }
            if (p.halted.is_some() && !p.resume || p.passed.take().is_some()) && held.is_none() {
                held = Some(n);
            }
        }
        held
    }

    /// The address of the step `processor` stands in front of: the EE's and the IOP's PC, a VU's pair by its byte
    /// address in its micro memory.
    pub fn debug_pc(&self, processor: u32) -> Option<u64> {
        let v = &self.sys.board.vpu;
        match processor {
            EE => Some(self.sys.ee.pc as u64),
            IOP => Some(self.sys.board.iop.cpu.regs.pc as u64),
            VU0 => Some(v.micro0.pc() as u64 * 8),
            VU1 => Some(v.micro1.pc() as u64 * 8),
            _ => None,
        }
    }

    /// A processor's registers in machine info's order (`ee_registers`, `iop_registers`, `vu_registers`).
    pub fn debug_registers(&self, processor: u32) -> Option<Vec<i64>> {
        let b = &self.sys.board;
        match processor {
            EE => {
                let e = &self.sys.ee;
                let half = |q: u128| [q as u64 as i64, (q >> 64) as u64 as i64];
                let mut v: Vec<i64> = e.gpr.iter().flat_map(|&q| half(q)).collect();
                v.push(e.pc as i64);
                v.extend(half(e.hi));
                v.extend(half(e.lo));
                v.push(e.sa as i64);
                v.extend(e.cop0.iter().map(|&x| x as i64));
                v.push(e.debug[0] as i64);
                v.extend(e.debug[2..].iter().map(|&x| x as i64));
                v.extend([e.pccr, e.pcr[0], e.pcr[1]].map(|x| x as i64));
                v.extend(e.fpr.iter().map(|&x| x as i64));
                v.extend([e.acc, e.fcr31].map(|x| x as i64));
                let k = &self.idle.counts;
                v.extend([k.ee_skips, k.ee_cycles, k.ee_steps, k.busy].map(|x| x as i64));
                Some(v)
            }
            IOP => {
                let r = &b.iop.cpu.regs;
                let mut v: Vec<i64> = r.r.iter().map(|&x| x as i64).collect();
                v.extend([r.hi, r.lo, r.pc, r.sr, r.cause, r.epc, r.bad_vaddr, r.tar, r.dcic, r.bpc, r.bda, r.bpcm, r.bdam].map(|x| x as i64));
                let k = &self.idle.counts;
                v.extend([k.iop_cycles, k.iop_instructions].map(|x| x as i64));
                Some(v)
            }
            VU0 | VU1 => {
                let (u, m) = if processor == VU0 { (&b.vpu.units.vu0, &b.vpu.micro0) } else { (&b.vpu.units.vu1, &b.vpu.micro1) };
                let mut v: Vec<i64> = (0..32).flat_map(|i| u.vf(i)).map(|x| x as i64).collect();
                v.extend(u.acc.map(|x| x as i64));
                v.extend((0..16).map(|i| u.vi(i) as i64));
                v.extend([u.i, u.q, u.p, u.r, u.status, u.mac, u.clip, u.tpc as u32, m.pc() as u32 * 8].map(|x| x as i64));
                Some(v)
            }
            _ => None,
        }
    }

    /// `count` instructions of `processor` from `address` of `space`: the EE's from RAM, the scratchpad or the boot ROM
    /// by the R5900's table, the IOP's from its RAM or the boot ROM by the R3000A's, a VU's pairs from its micro memory
    /// by the VU tables, upper and lower; each with the static reference its bytes alone give. None for a space the
    /// processor runs no code from.
    pub fn debug_disassemble(&self, processor: u32, space: u32, address: u32, count: u32) -> Option<Vec<Instruction>> {
        let allowed: &[u32] = match processor {
            EE => &[RAM_ID, SCRATCH_ID, BIOS_ID],
            IOP => &[IOPRAM_ID, BIOS_ID],
            VU0 => &[VU0MICRO_ID],
            VU1 => &[VU1MICRO_ID],
            _ => return None,
        };
        if !allowed.contains(&space) {
            return None;
        }
        let width = if processor >= VU0 { 8 } else { 4 };
        let bytes = |at: u32| -> Vec<u8> { (0..width).map(|i| self.space_byte(space, at.wrapping_add(i) as usize).unwrap_or(0xFF)).collect() };
        let base = runs_at(processor, space);
        let iop = blackmoon_r3000::dialect(None);
        let mut out = Vec::with_capacity(count.min(4096) as usize);
        let mut at = address & !(width - 1);
        for _ in 0..count {
            let b = bytes(at);
            let pc = base.wrapping_add(at);
            let (text, r) = match processor {
                EE => {
                    let i = emusen_mips::r5900::decode(u32::from_le_bytes(b[..4].try_into().unwrap()), pc);
                    (format!("{} {}", i.mnemonic, i.operands), i.reference.map(|(k, t)| (reference(k), t)))
                }
                IOP => {
                    let w = u32::from_le_bytes(b[..4].try_into().unwrap());
                    (emusen_mips::disassemble(w, pc, &iop).to_string(), blackmoon_r3000::reference(w, pc))
                }
                _ => {
                    let pair = u64::from_le_bytes(b[..8].try_into().unwrap());
                    let mask = if processor == VU0 { 0x1FF } else { 0x7FF };
                    let lower = pair as u32;
                    let bal = (pair >> 32) as u32 & emusen_mips::vu::I_BIT == 0 && emusen_mips::vu::lower(lower).is_some_and(|o| o.name == "bal");
                    let target = ((at / 8) as i32 + 1 + emusen_mips::vu::imm11(lower)) as u32 & mask;
                    (emusen_mips::vu::disassemble(pair), bal.then_some((Reference::Call, target * 8)))
                }
            };
            let (mnemonic, operands) = text.trim_end().split_once(' ').unwrap_or((text.trim_end(), ""));
            out.push(Instruction { address: at, bytes: b, mnemonic: mnemonic.into(), operands: operands.into(), reference: r });
            at = at.wrapping_add(width);
        }
        Some(out)
    }

    /// A processor's breakpoints: the EE's are the shared hooks', the others' go to their probes at the next frame.
    pub fn debug_breakpoints(&mut self, processor: u32, pairs: &[i32]) -> bool {
        let list = pairs.as_chunks::<2>().0.iter().map(|p| (p[0], p[1])).collect();
        match processor {
            EE => self.debug.hooks.breakpoints = list,
            IOP..=VU1 => self.debug.breakpoints[processor as usize - 1] = list,
            _ => return false,
        }
        true
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::machine::Market;
    use crate::media::Media;
    use emusen_native::debug::{flag, range};
    use emusen_native::ffi::StateMachine;

    pub const ORIGIN: u32 = 0x0010_0000;
    const LOOP: u32 = ORIGIN + 8;
    const ROUTINE: u32 = ORIGIN + 24;
    const COUNT: u32 = 0x0020_0000;
    const IOP_ORIGIN: u32 = 0x0001_0000;
    const IOP_LOOP: u32 = 0x8001_0004;

    /// A loop at 00100008h that calls a routine at 00100018h through JAL; the routine counts in RAM at 00200000h, stores
    /// the count's low byte in the scratchpad, and moves a quadword with LQ and SQ.
    const EE_PROGRAM: &str = "lui s0, 0x20\nlui s1, 0x7000\nloop: jal routine\nnop\nj loop\nnop
routine: lw t1, 0(s0)\naddiu t1, t1, 1\nsw t1, 0(s0)\nsb t1, 0x10(s1)\nlq t2, 0x20(s0)\nsq t2, 0x40(s0)\njr ra\nnop";

    /// The IOP counting at its RAM's 100h, from 80010000h.
    const IOP_PROGRAM: &str = "lui t0, 0\nloop: addiu t0, t0, 1\nsw t0, 0x100(zero)\nj loop\nnop";

    /// VU1 counting in vi1 in a loop of three pairs, the branch's delay slot the third.
    const VU1_PROGRAM: &str = "loop: nop | iaddiu vi1, vi1, 0x1\nnop | b loop\nnop | move vf0, vf0";

    /// A console running the project's program form: the EE from `ORIGIN`, and with `others` the IOP and VU1 started.
    pub fn machine(others: bool) -> Machine {
        let mut m = Machine::new(Media::read(&crate::media::program(&[0], ORIGIN)).unwrap(), Market::Americas, [Some("ps2.dualshock2"), None]);
        let mut sys = System::new(ORIGIN);
        sys.board.map.load(ORIGIN, &emusen_mips::assemble(EE_PROGRAM, ORIGIN, &emusen_mips::Dialect::R5900).unwrap());
        sys.board.map.rdram[COUNT as usize + 0x20..COUNT as usize + 0x30].copy_from_slice(&std::array::from_fn::<u8, 16, _>(|i| 0x10 + i as u8));
        if others {
            let words = emusen_mips::assemble(IOP_PROGRAM, 0x8000_0000 | IOP_ORIGIN, &blackmoon_r3000::dialect(None)).unwrap();
            sys.board.iop.bus.load(IOP_ORIGIN, &words);
            sys.board.iop.enter(0x8000_0000 | IOP_ORIGIN);
            for (i, p) in emusen_mips::vu::assemble(VU1_PROGRAM).unwrap().iter().enumerate() {
                sys.board.vpu.units.vu1.micro[8 * i..8 * i + 8].copy_from_slice(&p.to_le_bytes());
            }
            let now = sys.board.now;
            assert!(sys.board.vpu.start(1, 0, now));
        }
        m.sys = Box::new(sys);
        m.booted = true;
        m
    }

    pub fn state(m: &Machine) -> Vec<u8> {
        let mut v = vec![0; m.state_size()];
        let n = m.save_state(&mut v).unwrap();
        v.truncate(n);
        v
    }

    fn arm(m: &mut Machine, flags: u32) {
        m.debug.hooks.configure(flags, i32::MIN, -1);
    }

    /// The routine's count as the EE sees it, its data cache over RAM.
    fn count(m: &Machine) -> u32 {
        u32::from_le_bytes(std::array::from_fn(|i| m.byte(COUNT + i as u32).unwrap().2))
    }

    fn drain(m: &mut Machine) {
        m.debug.hooks.calls_log.clear();
        m.debug.hooks.writes_log.clear();
        m.debug.hooks.reads_log.clear();
    }

    /// The open frame run to its end, each stop resumed and a full log drained; the stops for any other reason.
    pub fn run_out(m: &mut Machine, mut flags: u32) -> u32 {
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

    fn stop_at(m: &mut Machine, pc: u32) {
        m.debug.hooks.set_breakpoints(&[pc as i32, pc as i32]);
        assert_eq!((m.run_frame_debug(0), m.sys.ee.pc), (stop::BREAKPOINT, pc));
        m.debug.hooks.set_breakpoints(&[]);
    }

    fn named(names: &[(String, u32)], values: &[i64], name: &str) -> i64 {
        values[names.iter().position(|n| n.0 == name).unwrap_or_else(|| panic!("no register {name}"))]
    }

    /// Each processor's registers in machine info's order at their widths, as set: both halves of a 128-bit general
    /// register, HI1 and LO1 as HI's and LO's upper halves, COP0, COP1 and its accumulator; the IOP's; a VU's fields,
    /// integers, ACC and the special registers.
    #[test]
    fn the_registers_read_back_as_set() {
        let mut m = machine(true);
        let e = &mut m.sys.ee;
        e.gpr[5] = 0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210;
        e.hi = 0x1111_2222_3333_4444_5555_6666_7777_8888;
        e.lo = 0x9999_AAAA_BBBB_CCCC_DDDD_EEEE_FFFF_0000;
        e.sa = 0x48;
        e.cop0[12] = 0x7000_0010;
        e.debug[2] = 0x0010_0004;
        e.fpr[31] = 0x3F80_0000;
        e.acc = 0x4040_0000;
        let v = m.debug_registers(EE).unwrap();
        let n = ee_registers();
        assert_eq!(v.len(), n.len());
        assert_eq!((named(&n, &v, "a1") as u64, named(&n, &v, "a1.hi") as u64), (0xFEDC_BA98_7654_3210, 0x0123_4567_89AB_CDEF), "both halves");
        assert_eq!([named(&n, &v, "hi"), named(&n, &v, "hi1"), named(&n, &v, "lo"), named(&n, &v, "lo1")].map(|x| x as u64), [0x5555_6666_7777_8888, 0x1111_2222_3333_4444, 0xDDDD_EEEE_FFFF_0000, 0x9999_AAAA_BBBB_CCCC]);
        assert_eq!([named(&n, &v, "sa"), named(&n, &v, "Status"), named(&n, &v, "IAB"), named(&n, &v, "f31"), named(&n, &v, "ACC"), named(&n, &v, "pc")], [0x48, 0x7000_0010, 0x0010_0004, 0x3F80_0000, 0x4040_0000, ORIGIN as i64]);
        assert!(n.iter().all(|r| r.1 <= 64) && n.iter().filter(|r| r.0.ends_with(".hi")).count() == 32);

        m.sys.board.iop.cpu.regs.r[9] = 0xCAFE_F00D;
        let (v, n) = (m.debug_registers(IOP).unwrap(), iop_registers());
        assert_eq!((v.len(), named(&n, &v, "t1"), named(&n, &v, "pc")), (47, 0xCAFE_F00D, 0x8001_0000));
        m.idle.counts.ee_cycles = 1234;
        m.idle.counts.iop_instructions = 56;
        let (v, n) = (m.debug_registers(EE).unwrap(), ee_registers());
        assert_eq!(named(&n, &v, "idle.cycles"), 1234, "idle skipping's counts after the EE's registers");
        let (v, n) = (m.debug_registers(IOP).unwrap(), iop_registers());
        assert_eq!(named(&n, &v, "idle.instructions"), 56);

        let u = &mut m.sys.board.vpu.units.vu1;
        u.set_vf(7, [1, 2, 3, 0x4080_0000]);
        u.set_vi(3, 0xBEEF);
        (u.acc, u.i, u.q, u.p, u.r, u.clip) = ([5, 6, 7, 8], 9, 10, 11, 0x7F_FFFF, 0xAB_CDEF);
        let (v, n) = (m.debug_registers(VU1).unwrap(), vu_registers(1));
        assert_eq!(v.len(), n.len());
        assert_eq!(["vf7.x", "vf7.w", "vf0.w", "acc.z", "vi3", "I", "Q", "P", "R", "clip", "PC"].map(|r| named(&n, &v, r)), [1, 0x4080_0000, 0x3F80_0000, 7, 0xBEEF, 9, 10, 11, 0x7F_FFFF, 0xAB_CDEF, 0]);
        assert!(m.debug_registers(4).is_none() && m.debug_pc(4).is_none());
    }

    /// Each processor's listing is its table's: the R5900's with its own instructions and references, the R3000A's, and a
    /// VU pair as upper and lower, the lower word read by the lower table.
    #[test]
    fn each_processors_listing_is_its_table() {
        let mut m = machine(true);
        let text = |v: Vec<Instruction>| v.iter().map(|i| format!("{} {}", i.mnemonic, i.operands).trim().to_owned()).collect::<Vec<_>>();
        let ee = m.debug_disassemble(EE, RAM_ID, LOOP, 1).unwrap();
        assert_eq!((text(ee.clone()), ee[0].reference), (vec!["jal 0x00100018".to_string()], Some((Reference::Call, ROUTINE))));
        assert_eq!(text(m.debug_disassemble(EE, RAM_ID, ROUTINE + 16, 2).unwrap()), ["lq t2, 0x20(s0)", "sq t2, 0x40(s0)"]);
        for at in (ORIGIN..ORIGIN + 56).step_by(4) {
            let w = u32::from_le_bytes(m.sys.board.map.rdram[at as usize..at as usize + 4].try_into().unwrap());
            let i = emusen_mips::r5900::decode(w, at);
            assert_eq!(text(m.debug_disassemble(EE, RAM_ID, at + 2, 1).unwrap()), [format!("{} {}", i.mnemonic, i.operands).trim()], "{at:x}");
        }
        assert_eq!(text(m.debug_disassemble(IOP, IOPRAM_ID, IOP_ORIGIN + 4, 2).unwrap()), ["addiu t0, t0, 0x1", "sw t0, 0x100(zero)"]);
        assert_eq!(m.debug_disassemble(IOP, IOPRAM_ID, IOP_ORIGIN + 8, 1).unwrap()[0].reference, Some((Reference::Write, 0x100)));
        let vu = m.debug_disassemble(VU1, VU1MICRO_ID, 0, 3).unwrap();
        assert_eq!(text(vu.clone()), ["nop | iaddiu vi1, vi1, 0x1", "nop | b -0x2", "nop | move vf0, vf0"]);
        assert_eq!((vu[1].address, vu[1].bytes.len()), (8, 8));
        m.sys.board.vpu.units.vu1.micro[24..32].copy_from_slice(&emusen_mips::vu::assemble_pair("nop | bal vi15, 0x10").unwrap().to_le_bytes());
        assert_eq!(m.debug_disassemble(VU1, VU1MICRO_ID, 24, 1).unwrap()[0].reference, Some((Reference::Call, (4 + 0x10) * 8)));
        assert!(m.debug_disassemble(VU0, VU1MICRO_ID, 0, 1).is_none() && m.debug_disassemble(EE, IOPRAM_ID, 0, 1).is_none() && m.debug_disassemble(4, RAM_ID, 0, 1).is_none());
    }

    /// An EE breakpoint stops in front of its instruction with nothing of it run, asked again it stops again, resumed it
    /// runs that instruction and stops at the next pass; one never reached never stops.
    #[test]
    fn an_ee_breakpoint_fires_at_its_address_and_only_there() {
        let mut m = machine(false);
        m.advance();
        let store = ROUTINE + 8;
        m.debug.hooks.set_breakpoints(&[store as i32, store as i32]);
        assert_eq!((m.run_frame_debug(0), m.debug_pc(EE), m.debug.stopped), (stop::BREAKPOINT, Some(store as u64), EE));
        let at_stop = (count(&m), state(&m));
        assert_eq!(m.sys.ee.gpr[9] as u32, at_stop.0 + 1, "the pass's LW and ADDIU run, its store not");
        assert_eq!(m.run_frame_debug(0), stop::BREAKPOINT);
        assert_eq!(state(&m), at_stop.1, "a stop runs nothing");
        assert_eq!((m.run_frame_debug(run::UNCHECKED), m.sys.ee.pc, count(&m)), (stop::BREAKPOINT, store, at_stop.0 + 1));
        m.debug.hooks.set_breakpoints(&[(ORIGIN + 0x400) as i32, (ORIGIN + 0x400) as i32]);
        assert_eq!(run_out(&mut m, run::UNCHECKED), 0, "never reached");
    }

    /// `EACH` stops in front of every EE instruction in order, through the JAL, its delay slot and the routine.
    #[test]
    fn each_steps_the_ee_one_instruction_at_a_time() {
        let mut m = machine(false);
        m.advance();
        stop_at(&mut m, LOOP);
        arm(&mut m, flag::EACH);
        let r = ROUTINE;
        let pcs: Vec<u32> = (0..10)
            .map(|_| {
                assert_eq!(m.run_frame_debug(run::UNCHECKED), stop::EACH);
                m.sys.ee.pc
            })
            .collect();
        assert_eq!(pcs, [LOOP + 4, r, r + 4, r + 8, r + 12, r + 16, r + 20, r + 24, r + 28, LOOP + 8]);
    }

    /// The call stack follows JAL and the return once JR's delay slot has run; a depth target steps over the call.
    #[test]
    fn the_call_stack_follows_jal_and_jr() {
        let mut m = machine(false);
        m.advance();
        stop_at(&mut m, LOOP);
        arm(&mut m, flag::CALLS | flag::EACH);
        assert_eq!(m.run_frame_debug(run::UNCHECKED), stop::EACH);
        assert_eq!(m.debug.hooks.stack, [(LOOP, ROUTINE)], "pushed at the JAL");
        for _ in 0..8 {
            m.run_frame_debug(run::UNCHECKED);
        }
        assert_eq!((m.sys.ee.pc, m.debug.hooks.depth()), (ROUTINE + 28, 1), "in the JR's delay slot");
        m.run_frame_debug(run::UNCHECKED);
        assert_eq!((m.sys.ee.pc, m.debug.hooks.depth()), (LOOP + 8, 0), "returned once the slot ran");
        arm(&mut m, flag::CALLS);
        stop_at(&mut m, LOOP);
        let before = count(&m);
        m.debug.hooks.configure(flag::CALLS, 0, -1);
        assert_eq!(m.run_frame_debug(run::UNCHECKED), stop::DEPTH);
        assert_eq!((m.sys.ee.pc, count(&m)), (LOOP + 8, before + 1), "stepped over the call, the routine run whole");
    }

    /// Stores and loads are reported byte by byte with their instruction, in RAM and the scratchpad at the offsets they
    /// land at, LQ and SQ as sixteen; a data breakpoint stops in front of the next instruction with the store run; a read
    /// data breakpoint in front of the load with nothing run.
    #[test]
    fn the_watches_see_the_ees_stores_and_loads() {
        let mut m = machine(false);
        m.advance();
        arm(&mut m, flag::WRITES | flag::READS);
        m.debug.hooks.capacity = 1 << 24;
        m.debug.hooks.set_ranges(range::WATCH, &[RAM_ID, COUNT, COUNT + 3, SCRATCH_ID, 0x10, 0x10, RAM_ID, COUNT + 0x40, COUNT + 0x4F]);
        m.debug.hooks.set_ranges(range::READ_WATCH, &[RAM_ID, COUNT + 0x20, COUNT + 0x2F]);
        run_out(&mut m, 0);
        let h = &m.debug.hooks;
        let last = |space: u32, at: u32| h.writes_log.iter().rev().find(|w| w.space == space && w.address == at).copied().unwrap();
        assert_eq!((last(RAM_ID, COUNT).value, last(RAM_ID, COUNT).pc), (count(&m) & 0xFF, ROUTINE + 8));
        assert_eq!((last(SCRATCH_ID, 0x10).value, last(SCRATCH_ID, 0x10).pc), (count(&m) & 0xFF, ROUTINE + 12));
        assert_eq!((last(RAM_ID, COUNT + 0x4F).value, last(RAM_ID, COUNT + 0x4F).pc), (0x1F, ROUTINE + 20), "SQ's sixteenth byte");
        let reads: Vec<_> = h.reads_log.iter().rev().take(16).map(|r| (r.address - COUNT, r.value, r.pc, r.kind)).collect();
        assert_eq!(reads, (0..16).rev().map(|i| (0x20 + i, 0x10 + i, ROUTINE + 16, range::READ_WATCH)).collect::<Vec<_>>());
        drain(&mut m);
        m.debug.hooks.set_ranges(range::BREAK, &[SCRATCH_ID, 0x10, 0x10]);
        assert_eq!((m.run_frame_debug(0), m.sys.ee.pc), (stop::DATA, ROUTINE + 16), "in front of the instruction after the SB");
        assert_eq!(m.sys.ee.spram[0x10] as u32, count(&m) & 0xFF, "the store ran");
        m.debug.hooks.set_ranges(range::BREAK, &[]);
        m.debug.hooks.set_ranges(range::READ_BREAK, &[RAM_ID, COUNT + 1, COUNT + 1]);
        drain(&mut m);
        let before = state(&m);
        assert_eq!((m.run_frame_debug(run::UNCHECKED), m.sys.ee.pc), (stop::READ, ROUTINE), "in front of the LW");
        let held = state(&m);
        assert_eq!(m.run_frame_debug(0), stop::READ);
        assert_eq!(held, state(&m), "nothing of it run");
        assert_ne!(before, held);
        assert_eq!(*m.debug.hooks.reads_log.iter().find(|r| r.kind == range::READ_BREAK).unwrap(), emusen_native::debug::Read { space: RAM_ID, address: COUNT + 1, value: count(&m) >> 8 & 0xFF, pc: ROUTINE, kind: range::READ_BREAK });
    }

    /// The IOP and VU1 stop in front of their breakpoints, with that processor named and its address, nothing of the
    /// step run; resumed, each runs it; a breakpoint over every address steps that processor one step at a time.
    #[test]
    fn the_iop_and_vu1_stop_at_their_breakpoints_and_step() {
        let mut m = machine(true);
        m.advance();
        let iop_count = |m: &Machine| m.sys.board.iop.bus.word(0x100);
        assert!(m.debug_breakpoints(IOP, &[IOP_LOOP as i32 + 4, IOP_LOOP as i32 + 4]));
        assert_eq!((m.run_frame_debug(0), m.debug.stopped, m.debug_pc(IOP)), (stop::BREAKPOINT, IOP, Some(IOP_LOOP as u64 + 4)));
        let (held, c) = (state(&m), iop_count(&m));
        assert_eq!(m.run_frame_debug(0), stop::BREAKPOINT);
        assert_eq!(held, state(&m), "a stop runs nothing");
        assert_eq!((m.run_frame_debug(run::UNCHECKED), m.debug.stopped, iop_count(&m)), (stop::BREAKPOINT, IOP, c + 1), "the store ran, the next pass stopped");
        m.debug_breakpoints(IOP, &[i32::MIN, i32::MAX]);
        let steps: Vec<u64> = (0..5)
            .map(|_| {
                assert_eq!((m.run_frame_debug(run::UNCHECKED), m.debug.stopped), (stop::BREAKPOINT, IOP));
                m.debug_pc(IOP).unwrap()
            })
            .collect();
        let l = IOP_LOOP as u64;
        assert_eq!(steps, [l + 8, l + 12, l, l + 4, l + 8]);
        m.debug_breakpoints(IOP, &[]);

        assert!(m.debug_breakpoints(VU1, &[8, 8]));
        assert_eq!((m.run_frame_debug(run::UNCHECKED), m.debug.stopped, m.debug_pc(VU1)), (stop::BREAKPOINT, VU1, Some(8)));
        let vi1 = |m: &Machine| m.sys.board.vpu.units.vu1.vi(1);
        let v = vi1(&m);
        assert_eq!((m.run_frame_debug(0), vi1(&m)), (stop::BREAKPOINT, v));
        assert_eq!((m.run_frame_debug(run::UNCHECKED), m.debug_pc(VU1), vi1(&m)), (stop::BREAKPOINT, Some(8), v + 1), "one pass of the loop");
        m.debug_breakpoints(VU1, &[i32::MIN, i32::MAX]);
        let pairs: Vec<u64> = (0..4)
            .map(|_| {
                assert_eq!((m.run_frame_debug(run::UNCHECKED), m.debug.stopped), (stop::BREAKPOINT, VU1));
                m.debug_pc(VU1).unwrap()
            })
            .collect();
        assert_eq!(pairs, [16, 0, 8, 16]);
        m.debug_breakpoints(VU1, &[4, 4]);
        assert_eq!(run_out(&mut m, run::UNCHECKED), 0, "no pair is at 4");
    }

    /// Coverage marks what ran on each processor: the EE's loop and routine and not the word past them, the IOP's loop,
    /// VU1's three pairs and not the fourth; the EE's count is every instruction it ran.
    #[test]
    fn coverage_records_what_each_processor_ran() {
        let mut m = machine(true);
        m.advance();
        arm(&mut m, (0..4).fold(flag::PROFILING, |f, n| f | 1 << (flag::COVERAGE + n)));
        let passes = |m: &Machine| [count(m) as i64, m.sys.board.iop.bus.word(0x100) as i64, m.sys.board.vpu.units.vu1.vi(1) as i64];
        let before = passes(&m);
        run_out(&mut m, 0);
        let after = passes(&m);
        let h = &m.debug.hooks;
        // A pass is twelve EE instructions (the loop's four, the routine's eight), four of the IOP's and three VU1 pairs,
        // each count moved once a pass (vi1 modulo its 16 bits): what ran is the passes made, within one pass.
        for (n, (k, per, wrap)) in [(0usize, 12i64, 1i64 << 40), (1, 4, 1 << 40), (3, 3, 1 << 16)].into_iter().enumerate() {
            let covered = h.coverage[k].as_ref().unwrap().covered;
            let off = (covered / per - (after[n] - before[n])).rem_euclid(wrap);
            assert!(covered > 1000 && (off <= 1 || off == wrap - 1), "processor {k}: {covered} steps, passes off by {off}");
        }
        let ran = |n: usize, a: u32| {
            let c = h.coverage[n].as_ref().unwrap();
            let a = a & ((1 << COVERAGE_BITS[n]) - 1);
            c.bits[(a >> 3) as usize] & 1 << (a & 7) != 0
        };
        assert!((LOOP..ROUTINE + 32).step_by(4).all(|a| ran(0, a)) && !ran(0, ROUTINE + 32) && !ran(0, ORIGIN + 2));
        assert!((IOP_LOOP..IOP_LOOP + 16).step_by(4).all(|a| ran(1, a)) && !ran(1, IOP_LOOP + 16));
        assert!([0, 8, 16].iter().all(|&a| ran(3, a)) && !ran(3, 24) && h.coverage[2].as_ref().unwrap().covered == 0);
        let mut h = h.clone();
        h.flush();
        assert_eq!(h.charged(0), h.coverage[0].as_ref().unwrap().covered, "every EE step counted once");
    }

    /// With no BIOS the EE is held at its reset vector: a breakpoint there stops in front of it and `EACH` before each
    /// step of the board, which moves the board's time and nothing of the EE; armed with nothing to hit, the frame is
    /// the plain one.
    #[test]
    fn a_held_ee_stands_in_front_of_its_reset_vector() {
        let new = || Machine::new(Media::read(&crate::media::program(&[0], ORIGIN)).unwrap(), Market::Americas, [None, None]);
        let (mut m, mut plain) = (new(), new());
        m.debug.hooks.set_breakpoints(&[0xBFC0_0000u32 as i32, 0xBFC0_0000u32 as i32]);
        assert_eq!((m.run_frame_debug(0), m.debug_pc(EE)), (stop::BREAKPOINT, Some(0xBFC0_0000)));
        m.debug.hooks.set_breakpoints(&[]);
        arm(&mut m, flag::EACH);
        let now = m.sys.board.now;
        assert_eq!((m.run_frame_debug(0), m.sys.board.now), (stop::EACH, now), "asked, nothing moves");
        assert_eq!(m.run_frame_debug(run::UNCHECKED), stop::EACH);
        assert!(m.sys.board.now > now && m.sys.ee.pc == 0xBFC0_0000, "a step of the board, the EE held");
        arm(&mut m, flag::CALLS | 1 << flag::COVERAGE);
        run_out(&mut m, run::UNCHECKED);
        plain.advance();
        assert_eq!(state(&m), state(&plain));
    }

    /// A state loaded while a frame is stopped closes it and frees a processor a probe held: the next frame is whole.
    #[test]
    fn a_state_loaded_at_a_stop_starts_a_whole_frame() {
        let (mut m, mut plain) = (machine(true), machine(true));
        m.advance();
        plain.advance();
        let saved = state(&m);
        m.debug_breakpoints(IOP, &[IOP_LOOP as i32, IOP_LOOP as i32]);
        assert_eq!(m.run_frame_debug(0), stop::BREAKPOINT);
        m.load_state(&saved).unwrap();
        m.debug_breakpoints(IOP, &[]);
        assert!(!m.debug.open && !m.sys.board.iop.stopped());
        assert_eq!(run_out(&mut m, 0), 0);
        plain.advance();
        assert_eq!(state(&plain), state(&m));
    }
}
