//! The single-step harness over SingleStepTests' r3000 suite (`r3000/v1`, as its own `transcode_json.py` writes it):
//! the registers it records, both delay slots, and the bus cycle by cycle. The corpus is found through
//! `EMUSEN_BLACKMOON_CORPUS`; without it the corpus tests pass unrun. BlackMoon_R3000.md §3 is the protocol.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use emusen_native::json::{self, Value};

use crate::{BranchDelay, Bus, LoadDelay, R3000, Registers, Size};

pub const CORPUS_VARIABLE: &str = "EMUSEN_BLACKMOON_CORPUS";
pub const SUITE: &str = "r3000/v1";

pub fn suite_files() -> Option<Vec<PathBuf>> {
    let dir = Path::new(&std::env::var_os(CORPUS_VARIABLE)?).join(SUITE);
    if !dir.is_dir() {
        return None;
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("the suite's folder")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    Some(files)
}

fn read_cases(path: &Path) -> Vec<Value> {
    match json::parse(&std::fs::read(path).expect("a suite file")) {
        Ok(Value::Array(cases)) => cases,
        other => panic!("{}: not an array of cases ({:?})", path.display(), other.err()),
    }
}

fn int(v: &Value, key: &str) -> i64 {
    v.get(key).and_then(Value::as_i64).unwrap_or_else(|| panic!("no integer {key}"))
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
}

impl FlatBus {
    pub fn new(recorded: Vec<Cycle>) -> FlatBus {
        FlatBus { served: vec![false; recorded.len()], recorded, log: Vec::new() }
    }

    fn answer(&mut self, actions: u32, address: u32, size: u32) -> u32 {
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
        self.log.push(Cycle { actions: WRITE, size: size.bytes(), address, value });
    }

    fn idle(&mut self, _cycles: u32) {}

    fn interrupt(&mut self) -> bool {
        false
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

#[derive(Clone, Debug, Default)]
pub struct Outcome {
    pub registers: bool,
    pub cycles: bool,
    pub first_difference: Option<String>,
}

impl Outcome {
    pub fn passed(&self) -> bool {
        self.registers && self.cycles
    }
}

pub fn grade(case: &Value, processor: &mut dyn FnMut(&Value, &mut FlatBus) -> Recorded) -> Outcome {
    let expected = cycles(case);
    let mut bus = FlatBus::new(expected.clone());
    let got = processor(case, &mut bus);
    let want = recorded(case.get("final").expect("final"));
    let mut o = Outcome { registers: got == want, ..Outcome::default() };
    if !o.registers {
        o.first_difference = Some(format!("registers: got {got:X?} want {want:X?}"));
    }
    o.cycles = bus.log == expected;
    if !o.cycles && o.first_difference.is_none() {
        let at = expected.iter().zip(&bus.log).position(|(w, g)| w != g).unwrap_or(expected.len().min(bus.log.len()));
        o.first_difference = Some(format!("cycle {at}: got {:X?} want {:X?}", bus.log.get(at), expected.get(at)));
    }
    o
}

fn size(bytes: u32) -> Size {
    match bytes {
        1 => Size::Byte,
        2 => Size::Half,
        _ => Size::Word,
    }
}

/// The positive control: a fake processor that performs the case's own cycles through the bus, in order, and ends in
/// the case's own final state.
pub fn replay(case: &Value, bus: &mut FlatBus) -> Recorded {
    for c in cycles(case) {
        match c.actions {
            FETCH => {
                bus.fetch(c.address);
            }
            READ => {
                bus.read(c.address, size(c.size));
            }
            _ => bus.write(c.address, size(c.size), c.value),
        }
    }
    recorded(case.get("final").expect("final"))
}

/// The processor from the case's initial state, one step.
pub fn processor(case: &Value, bus: &mut FlatBus) -> Recorded {
    let mut cpu = R3000 { regs: recorded(case.get("initial").expect("initial")).registers() };
    let _ = cpu.step(bus);
    Recorded::of(&cpu.regs)
}

#[derive(Clone, Debug, Default)]
pub struct Tally {
    pub cases: usize,
    pub passed: usize,
    pub registers: usize,
    pub cycles: usize,
    pub first_failure: Option<String>,
}

pub fn run_suite(files: &[PathBuf], limit: Option<usize>, processor: fn(&Value, &mut FlatBus) -> Recorded) -> Vec<(String, Tally)> {
    let next = Mutex::new(0usize);
    let out = Mutex::new(Vec::new());
    let threads = std::thread::available_parallelism().map_or(2, |n| n.get().min(4));
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                loop {
                    let i = {
                        let mut n = next.lock().unwrap();
                        *n += 1;
                        *n - 1
                    };
                    let Some(path) = files.get(i) else { break };
                    let mut t = Tally::default();
                    for case in read_cases(path).iter().take(limit.unwrap_or(usize::MAX)) {
                        let o = grade(case, &mut |c, b| processor(c, b));
                        t.cases += 1;
                        t.passed += o.passed() as usize;
                        t.registers += o.registers as usize;
                        t.cycles += o.cycles as usize;
                        if !o.passed() && t.first_failure.is_none() {
                            let name = case.get("name").and_then(Value::as_str).unwrap_or("?");
                            t.first_failure = Some(format!("{name}: {}", o.first_difference.unwrap_or_default()));
                        }
                    }
                    out.lock().unwrap().push((path.file_name().unwrap().to_string_lossy().into_owned(), t));
                }
            });
        }
    });
    let mut v = out.into_inner().unwrap();
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v
}

fn total(reports: &[(String, Tally)]) -> Tally {
    reports.iter().fold(Tally::default(), |mut t, (_, r)| {
        t.cases += r.cases;
        t.passed += r.passed;
        t.registers += r.registers;
        t.cycles += r.cycles;
        t
    })
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
        assert!(!grade(&c, &mut processor).passed(), "the unbuilt processor passes the inline case");
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
        for (f, r) in reports.iter().filter(|(_, r)| r.passed != r.cases).take(20) {
            eprintln!("  {f}: {} of {}; {}", r.passed, r.cases, r.first_failure.as_deref().unwrap_or(""));
        }
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
        for (f, r) in reports.iter().filter(|(_, r)| r.passed != r.cases).take(40) {
            eprintln!("  {f}: {} of {}; {}", r.passed, r.cases, r.first_failure.as_deref().unwrap_or(""));
        }
        assert_eq!(t.passed, t.cases, "a case fails");
    }
}
