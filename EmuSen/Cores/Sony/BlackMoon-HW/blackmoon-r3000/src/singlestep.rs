//! The single-step harness over SingleStepTests' r3000 suite (`r3000/v1`, as its own `transcode_json.py` writes it):
//! the registers it records, both delay slots, and the bus cycle by cycle. The corpus is found through
//! `EMUSEN_BLACKMOON_CORPUS`; without it the corpus tests pass unrun. BlackMoon_R3000.md §3 is the protocol, and §8
//! the disputes whose cases are graded in their own way here.

use std::path::{Path, PathBuf};

use emusen_native::json::Value;
use emusen_singlestep::{FileReport, Outcome, default_threads, first_mismatch, int, run_cases};

use crate::{BranchDelay, Bus, LoadDelay, R3000, Registers, Size};

pub const CORPUS_VARIABLE: &str = "EMUSEN_BLACKMOON_CORPUS";
pub const SUITE: &str = "r3000/v1";

pub fn suite_files() -> Option<Vec<PathBuf>> {
    let dir = Path::new(&std::env::var_os(CORPUS_VARIABLE)?).join(SUITE);
    dir.is_dir().then(|| emusen_singlestep::suite_files(&dir))
}

fn flag(v: &Value, key: &str) -> bool {
    match v.get(key) {
        Some(Value::Bool(b)) => *b,
        _ => panic!("no boolean {key}"),
    }
}

/// The suite's `actions`: what a cycle did.
pub const READ: u32 = 1;
pub const WRITE: u32 = 2;
pub const FETCH: u32 = 4;

/// One cycle as the suite records it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cycle {
    pub actions: u32,
    pub size: u32,
    pub address: u32,
    pub value: u32,
}

pub fn cycles(case: &Value) -> Vec<Cycle> {
    case.get("cycles")
        .and_then(Value::as_array)
        .expect("cycles")
        .iter()
        .map(|c| Cycle { actions: int(c, "actions") as u32, size: int(c, "sz") as u32, address: int(c, "addr") as u32, value: int(c, "val") as u32 })
        .collect()
}

/// What the suite records of the processor: the general registers, HI, LO, EPC, TAR, CAUSE, the PC and both delays.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Recorded {
    pub r: [u32; 32],
    pub hi: u32,
    pub lo: u32,
    pub epc: u32,
    pub tar: u32,
    pub cause: u32,
    pub pc: u32,
    pub branch: BranchDelay,
    pub load: LoadDelay,
}

impl Recorded {
    pub fn of(r: &Registers) -> Recorded {
        Recorded { r: r.r, hi: r.hi, lo: r.lo, epc: r.epc, tar: r.tar, cause: r.cause, pc: r.pc, branch: r.branch, load: r.load }
    }

    /// The suite's state as the processor's registers; what the suite does not record stays zero.
    pub fn registers(&self) -> Registers {
        Registers { r: self.r, hi: self.hi, lo: self.lo, epc: self.epc, tar: self.tar, cause: self.cause, pc: self.pc, branch: self.branch, load: self.load, ..Registers::default() }
    }
}

/// A state as the suite writes it; a load target of -1 is no pending load.
pub fn recorded(v: &Value) -> Recorded {
    let mut r = [0u32; 32];
    for (i, x) in v.get("R").and_then(Value::as_array).expect("R").iter().enumerate() {
        r[i] = x.as_i64().expect("a register") as u32;
    }
    let delay = v.get("delay").expect("delay");
    let branch = delay.get("branch").expect("delay.branch");
    let load = delay.get("load").expect("delay.load");
    let target = int(load, "target");
    Recorded {
        r,
        hi: int(v, "hi") as u32,
        lo: int(v, "lo") as u32,
        epc: int(v, "EPC") as u32,
        tar: int(v, "TAR") as u32,
        cause: int(v, "CAUSE") as u32,
        pc: int(v, "PC") as u32,
        branch: BranchDelay { slot: flag(branch, "slot"), taken: flag(branch, "take"), target: int(branch, "target") as u32 },
        load: LoadDelay { register: (target >= 0).then_some(target as u8), value: int(load, "val") as u32 },
    }
}

/// A bus that answers each fetch and read with the value the case recorded for that address and width, in order,
/// and records every access as the suite's cycles. The suite's memory is flat and holds no I/O.
pub struct FlatBus {
    pub recorded: Vec<Cycle>,
    served: Vec<bool>,
    pub log: Vec<Cycle>,
    /// The interrupt line, held where the case's Cause has IP2.
    pub line: bool,
    clock: u64,
}

impl FlatBus {
    pub fn new(recorded: Vec<Cycle>) -> FlatBus {
        FlatBus { served: vec![false; recorded.len()], recorded, log: Vec::new(), line: false, clock: 0 }
    }

    fn answer(&mut self, actions: u32, address: u32, size: u32) -> u32 {
        self.clock += 1;
        let found = (0..self.recorded.len()).find(|&i| !self.served[i] && self.recorded[i].actions == actions && self.recorded[i].address == address && self.recorded[i].size == size);
        let value = found.map_or(0, |i| {
            self.served[i] = true;
            self.recorded[i].value
        });
        self.log.push(Cycle { actions, size, address, value });
        value
    }
}

impl Bus for FlatBus {
    fn fetch(&mut self, address: u32) -> u32 {
        self.answer(FETCH, address, 4)
    }

    fn read(&mut self, address: u32, size: Size) -> u32 {
        self.answer(READ, address, size.bytes())
    }

    fn write(&mut self, address: u32, size: Size, value: u32) {
        self.clock += 1;
        self.log.push(Cycle { actions: WRITE, size: size.bytes(), address, value });
    }

    fn isolated_read(&mut self, address: u32, size: Size) -> u32 {
        self.read(address, size)
    }

    fn isolated_write(&mut self, address: u32, size: Size, value: u32) {
        self.write(address, size, value);
    }

    fn idle(&mut self, cycles: u32) {
        self.clock += cycles as u64;
    }

    fn cycles(&mut self) -> u64 {
        self.clock
    }

    fn interrupt(&mut self) -> bool {
        self.line
    }

    fn cop2_data(&mut self, _register: u8) -> u32 {
        0
    }

    fn set_cop2_data(&mut self, _register: u8, _value: u32) {}

    fn cop2_control(&mut self, _register: u8) -> u32 {
        0
    }

    fn set_cop2_control(&mut self, _register: u8, _value: u32) {}

    fn cop2_command(&mut self, _command: u32) {}
}

/// The dispute a case is graded under, if any (BlackMoon_R3000.md §8).
pub fn dispute(case: &Value) -> Option<&'static str> {
    let c = cycles(case);
    (c.len() == 3 && c[1].actions != FETCH && c[1].actions == c[2].actions).then_some("D-1")
}

/// D-1: the suite records a three-byte LWL, LWR, SWL or SWR as a halfword and a byte; the manuals make it one access
/// of three byte lanes. The two recorded cycles become that one, its datum the two values joined.
fn merged(mut c: Vec<Cycle>) -> Vec<Cycle> {
    let (a, b) = (c[1], c[2]);
    assert_eq!(b.address, a.address.wrapping_add(a.size), "two adjacent pieces");
    let value = if a.size == 2 { a.value & 0xFFFF | (b.value & 0xFF) << 16 } else { a.value & 0xFF | (b.value & 0xFFFF) << 8 };
    c.truncate(1);
    c.push(Cycle { actions: a.actions, size: 3, address: a.address, value });
    c
}

/// Cycles equal as the suite records them; a three-byte access by its three bytes.
fn same(want: &Cycle, got: &Cycle) -> bool {
    if want.size == 3 {
        Cycle { value: want.value & 0xFF_FFFF, ..*want } == Cycle { value: got.value & 0xFF_FFFF, ..*got }
    } else {
        want == got
    }
}

pub fn grade(case: &Value, processor: &mut dyn FnMut(&Value, &mut FlatBus) -> Recorded) -> Outcome {
    grade_as(case, processor, true)
}

/// A case graded on its recorded state and cycles, its dispute's form applied or not.
pub fn grade_as(case: &Value, processor: &mut dyn FnMut(&Value, &mut FlatBus) -> Recorded, ruled: bool) -> Outcome {
    let mut expected = cycles(case);
    if ruled && dispute(case) == Some("D-1") {
        expected = merged(expected);
    }
    let mut bus = FlatBus::new(expected.clone());
    let got = processor(case, &mut bus);
    let want = recorded(case.get("final").expect("final"));
    let registers = (got != want).then(|| format!("registers: got {got:X?} want {want:X?}"));
    let at = first_mismatch(&expected, &bus.log, same);
    let cycles = at.map(|at| format!("cycle {at}: got {:X?} want {:X?}", bus.log.get(at), expected.get(at)));
    Outcome::from_checks(registers, None, cycles)
}

/// The positive control: a fake processor that performs the case's own cycles through the bus, in order, and ends in
/// the case's own final state.
pub fn replay(case: &Value, bus: &mut FlatBus) -> Recorded {
    for c in bus.recorded.clone() {
        match c.actions {
            FETCH => {
                bus.fetch(c.address);
            }
            READ => {
                bus.read(c.address, Size::of(c.size));
            }
            _ => bus.write(c.address, Size::of(c.size), c.value),
        }
    }
    recorded(case.get("final").expect("final"))
}

/// The processor from the case's initial state, one step.
pub fn processor(case: &Value, bus: &mut FlatBus) -> Recorded {
    let mut cpu = R3000 { regs: recorded(case.get("initial").expect("initial")).registers(), ..R3000::new() };
    bus.line = cpu.regs.cause & 1 << 10 != 0;
    cpu.step(bus);
    Recorded::of(&cpu.regs)
}

pub fn run_suite(files: &[PathBuf], limit: Option<usize>, processor: fn(&Value, &mut FlatBus) -> Recorded) -> Vec<FileReport> {
    emusen_singlestep::run_suite(files, default_threads(), |path| run_cases(path, limit.unwrap_or(usize::MAX), |case| grade(case, &mut |c, b| processor(c, b))))
}

/// The spoiled replays: one field of the final state perturbed, and the last cycle dropped.
fn spoiled_hi(c: &Value, b: &mut FlatBus) -> Recorded {
    let mut r = replay(c, b);
    r.hi ^= 1;
    r
}

fn spoiled_cycle(c: &Value, b: &mut FlatBus) -> Recorded {
    let r = replay(c, b);
    b.log.pop();
    r
}

#[cfg(test)]
mod tests {
    use emusen_native::json;
    use emusen_singlestep::{print_misses, read_cases, total};

    use super::*;

    /// SB as the suite records it: the fetch, then a byte write, with a load pending from before.
    const CASE: &str = r#"{"name": "SB $000", "opcode": 2722497290, "opcode_addr": 2828994496,
        "initial": {"R": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
        27, 28, 29, 30, 31], "hi": 7, "lo": 8, "EPC": 9, "TAR": 10, "CAUSE": 11, "PC": 2828994496,
        "delay": {"branch": {"slot": false, "take": false, "target": 0}, "load": {"target": 6, "val": 4076945535}}},
        "final": {"R": [0, 1, 2, 3, 4, 5, 4076945535, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31], "hi": 7, "lo": 8, "EPC": 9, "TAR": 10, "CAUSE": 11, "PC": 2828994500,
        "delay": {"branch": {"slot": false, "take": false, "target": 0}, "load": {"target": -1, "val": 0}}},
        "cycles": [{"actions": 4, "sz": 4, "addr": 2828994496, "val": 2722497290},
                   {"actions": 2, "sz": 1, "addr": 2969261648, "val": 3977034564}]}"#;

    #[test]
    fn the_replay_passes_and_spoiled_replays_fail_on_the_inline_case() {
        let c = json::parse(CASE.as_bytes()).unwrap();
        let o = grade(&c, &mut replay);
        assert!(o.passed(), "{:?}", o.first_difference);
        let hi = grade(&c, &mut spoiled_hi);
        assert!(!hi.registers && hi.cycles);
        let dropped = grade(&c, &mut spoiled_cycle);
        assert!(dropped.registers && !dropped.cycles);
        assert_eq!(recorded(c.get("initial").unwrap()).load, LoadDelay { register: Some(6), value: 4076945535 });
    }

    fn corpus() -> Option<Vec<PathBuf>> {
        let files = suite_files();
        if files.is_none() {
            eprintln!("{CORPUS_VARIABLE} does not hold {SUITE}: not run");
        }
        files
    }

    /// The harness loads, answers and records as the suite's environment did (BlackMoon_R3000.md §3.1).
    #[test]
    fn the_replay_control_passes_every_case() {
        let Some(files) = corpus() else { return };
        let reports = run_suite(&files, None, replay);
        let t = total(&reports);
        eprintln!("{SUITE}: replay {} of {} in {} files", t.passed, t.cases, reports.len());
        print_misses(&reports, 20);
        assert!(t.cases > 0);
        assert_eq!(t.passed, t.cases);
    }

    #[test]
    fn spoiled_replays_pass_nothing() {
        let Some(files) = corpus() else { return };
        for (name, spoil) in [("hi", spoiled_hi as fn(&Value, &mut FlatBus) -> Recorded), ("last cycle", spoiled_cycle)] {
            let t = total(&run_suite(&files, None, spoil));
            eprintln!("{SUITE}: spoiled ({name}) {} of {}", t.passed, t.cases);
            assert!(t.cases > 0);
            assert_eq!(t.passed, 0, "spoiled by {name}");
        }
    }

    #[test]
    fn the_processor_against_the_suite() {
        let Some(files) = corpus() else { return };
        if !crate::BUILT {
            eprintln!("{SUITE}: the processor is not built: not run");
            return;
        }
        let reports = run_suite(&files, None, processor);
        let t = total(&reports);
        eprintln!("{SUITE}: {} of {} (registers {}, cycles {})", t.passed, t.cases, t.registers, t.cycles);
        for r in &reports {
            eprintln!("  {}: {} of {}; {}", r.file, r.passed, r.cases, r.first_failure.as_deref().unwrap_or(""));
        }
        assert_eq!(t.passed, t.cases, "a case fails");
    }

    /// D-1's cases: as the suite records them the cycles differ in every one (a read's value then has no recorded
    /// cycle to come from), and in D-1's form each passes.
    #[test]
    fn d1_cases_differ_from_the_suite_only_in_the_split_access() {
        let Some(files) = corpus() else { return };
        let (mut n, mut by_file) = (0, std::collections::BTreeMap::new());
        for path in &files {
            for case in read_cases(path).iter().filter(|c| dispute(c) == Some("D-1")) {
                let o = grade_as(case, &mut processor, false);
                assert!(!o.cycles, "{:?}", case.get("name"));
                assert!(grade(case, &mut processor).passed());
                *by_file.entry(path.file_name().unwrap().to_string_lossy().into_owned()).or_insert(0) += 1;
                n += 1;
            }
        }
        eprintln!("{SUITE}: D-1 names {n} cases: {by_file:?}");
        assert!(n > 0);
    }

    /// The suite's file each operation's cases are in.
    fn file_of(op: emusen_mips::Op) -> &'static str {
        use emusen_mips::Op::*;
        match op {
            Bltz | Bgez | Bltzal | Bgezal => "BCondZ",
            Sh => "SHL",
            other => {
                let m = other.mnemonic();
                Box::leak(m.to_ascii_uppercase().into_boxed_str())
            }
        }
    }

    /// `emusen-mips` over every opcode the suite holds: each decodes to its file's instruction, and its disassembly
    /// assembles to a word that agrees on every field the instruction uses and is zero elsewhere.
    #[test]
    fn every_suite_opcode_decodes_to_its_file_and_round_trips_through_the_assembler() {
        let Some(files) = corpus() else { return };
        let dialect = crate::dialect(None);
        let mut n = 0;
        for path in &files {
            let file = path.file_stem().unwrap().to_string_lossy().into_owned();
            for case in read_cases(path) {
                let word = int(&case, "opcode") as u32;
                let pc = int(&case, "opcode_addr") as u32;
                let op = emusen_mips::decode(word);
                assert_eq!(file_of(op), file, "{word:08X}");
                let text = emusen_mips::disassemble(word, pc, &dialect).to_string();
                let back = emusen_mips::assemble_line(&text, pc, &dialect).unwrap_or_else(|e| panic!("{word:08X} \"{text}\": {e}"));
                assert_eq!(emusen_mips::decode(back), op);
                assert_eq!(emusen_mips::disassemble(back, pc, &dialect).to_string(), text);
                let used = used_bits(op);
                assert_eq!((back & used, back & !used), (word & used, op.canonical() & !used), "{word:08X} \"{text}\"");
                n += 1;
            }
        }
        eprintln!("{SUITE}: {n} opcodes round-tripped");
        assert_eq!(n, files.len() * 1000);
    }

    fn used_bits(op: emusen_mips::Op) -> u32 {
        use emusen_mips::Operand::*;
        op.operands().iter().fold(0xFC00_0000, |m, o| {
            m | match o {
                Rd | CopData | CopControl => 31 << 11,
                Rs => 31 << 21,
                Rt | CopRt => 31 << 16,
                Sa => 31 << 6,
                Simm | Uimm | Branch => 0xFFFF,
                Mem => 31 << 21 | 0xFFFF,
                Jump => 0x03FF_FFFF,
                Code => 0xF_FFFF << 6,
                Cofun => 0x01FF_FFFF,
            }
        })
    }
}
