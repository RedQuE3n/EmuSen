//! The single-step harness over SingleStepTests' Z80 suite (`z80/v1`): registers, memory, the I/O transactions and
//! the bus T-state by T-state. The corpus is found through `EMUSEN_BERYL_CORPUS`; without it the corpus tests pass
//! unrun. Beryl_Z80.md §3.1 is the protocol.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use emusen_native::json::{self, Value};

use crate::{Bus, Registers, Z80};

pub const CORPUS_VARIABLE: &str = "EMUSEN_BERYL_CORPUS";
pub const SUITE: &str = "z80/v1";

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

fn int(v: &Value, key: &str) -> u32 {
    v.get(key).and_then(Value::as_i64).unwrap_or_else(|| panic!("no integer {key}")) as u32
}

/// One T-state as the suite samples it: the address pins (`None` for an internal T-state, whose address is not
/// compared), the data pins, and the four control pins as its `rwmi` string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TState {
    pub address: Option<u16>,
    pub data: Option<u8>,
    pub pins: [u8; 4],
}

fn t(address: u16, data: Option<u8>, pins: &[u8; 4]) -> TState {
    TState { address: Some(address), data, pins: *pins }
}

pub fn tstates(case: &Value) -> Vec<TState> {
    case.get("cycles")
        .and_then(Value::as_array)
        .expect("cycles")
        .iter()
        .map(|c| {
            let c = c.as_array().expect("a cycle");
            let pins = c[2].as_str().expect("pins").as_bytes();
            TState { address: c[0].as_i64().map(|a| a as u16), data: c[1].as_i64().map(|d| d as u8), pins: [pins[0], pins[1], pins[2], pins[3]] }
        })
        .collect()
}

/// The registers a case names.
pub fn registers(v: &Value) -> Registers {
    let pair = |h: &str, l: &str| (int(v, h) << 8 | int(v, l)) as u16;
    Registers {
        af: pair("a", "f"),
        bc: pair("b", "c"),
        de: pair("d", "e"),
        hl: pair("h", "l"),
        af_: int(v, "af_") as u16,
        bc_: int(v, "bc_") as u16,
        de_: int(v, "de_") as u16,
        hl_: int(v, "hl_") as u16,
        ix: int(v, "ix") as u16,
        iy: int(v, "iy") as u16,
        sp: int(v, "sp") as u16,
        pc: int(v, "pc") as u16,
        i: int(v, "i") as u8,
        r: int(v, "r") as u8,
        wz: int(v, "wz") as u16,
        q: int(v, "q") as u8,
        p: int(v, "p") != 0,
        iff1: int(v, "iff1") != 0,
        iff2: int(v, "iff2") != 0,
        im: int(v, "im") as u8,
        ei_pending: int(v, "ei") != 0,
    }
}

fn pairs(v: &Value, key: &str) -> Vec<Vec<Value>> {
    v.get(key).and_then(Value::as_array).map(|a| a.iter().map(|p| p.as_array().expect("an entry").to_vec()).collect()).unwrap_or_default()
}

/// 64 KiB of RAM holding what the case lists, the case's port reads answered in order, and every T-state and I/O
/// transaction recorded as the suite's environment samples them.
#[derive(Default)]
pub struct FlatBus {
    pub ram: BTreeMap<u16, u8>,
    pub inputs: Vec<u8>,
    pub log: Vec<TState>,
    pub ports: Vec<(u16, u8, u8)>,
}

impl FlatBus {
    fn byte(&self, a: u16) -> u8 {
        self.ram.get(&a).copied().unwrap_or(0)
    }
}

impl Bus for FlatBus {
    fn fetch(&mut self, address: u16, refresh: u16) -> u8 {
        let v = self.byte(address);
        self.log.extend([t(address, None, b"----"), t(address, None, b"r-m-"), t(refresh, Some(v), b"----"), t(refresh, None, b"----")]);
        v
    }

    fn read(&mut self, address: u16) -> u8 {
        let v = self.byte(address);
        self.log.extend([t(address, None, b"----"), t(address, None, b"r-m-"), t(address, Some(v), b"----")]);
        v
    }

    fn write(&mut self, address: u16, value: u8) {
        self.ram.insert(address, value);
        self.log.extend([t(address, None, b"----"), t(address, Some(value), b"-wm-"), t(address, None, b"----")]);
    }

    fn input(&mut self, port: u16) -> u8 {
        let v = if self.inputs.is_empty() { 0xFF } else { self.inputs.remove(0) };
        self.log.extend([t(port, None, b"----"), t(port, None, b"----"), t(port, None, b"r--i"), t(port, Some(v), b"----")]);
        self.ports.push((port, v, b'r'));
        v
    }

    fn output(&mut self, port: u16, value: u8) {
        self.log.extend([t(port, None, b"----"), t(port, None, b"----"), t(port, Some(value), b"-w-i"), t(port, None, b"----")]);
        self.ports.push((port, value, b'w'));
    }

    fn idle(&mut self, t_states: u32) {
        for _ in 0..t_states {
            self.log.push(TState { address: None, data: None, pins: *b"----" });
        }
    }

    fn int_line(&mut self) -> bool {
        false
    }

    fn nmi_edge(&mut self) -> bool {
        false
    }

    fn acknowledge(&mut self) -> u8 {
        0xFF
    }
}

#[derive(Clone, Debug, Default)]
pub struct Outcome {
    pub registers: bool,
    pub memory: bool,
    pub cycles: bool,
    pub first_difference: Option<String>,
}

impl Outcome {
    pub fn passed(&self) -> bool {
        self.registers && self.memory && self.cycles
    }
}

fn same(want: &TState, got: &TState) -> bool {
    want.pins == got.pins && want.data == got.data && (got.address.is_none() || want.address == got.address)
}

pub fn grade(case: &Value, processor: &mut dyn FnMut(&Value, &mut FlatBus) -> Registers) -> Outcome {
    let initial = case.get("initial").expect("initial");
    let fin = case.get("final").expect("final");
    let ports = pairs(case, "ports");
    let mut bus = FlatBus {
        ram: pairs(initial, "ram").iter().map(|p| (p[0].as_i64().unwrap() as u16, p[1].as_i64().unwrap() as u8)).collect(),
        inputs: ports.iter().filter(|p| p[2].as_str() == Some("r")).map(|p| p[1].as_i64().unwrap() as u8).collect(),
        ..FlatBus::default()
    };
    let got = processor(case, &mut bus);
    let want = registers(fin);
    let mut o = Outcome { registers: got == want, ..Outcome::default() };
    if !o.registers {
        o.first_difference = Some(format!("registers: got {got:X?} want {want:X?}"));
    }
    let wrong: Vec<_> = pairs(fin, "ram").iter().map(|p| (p[0].as_i64().unwrap() as u16, p[1].as_i64().unwrap() as u8)).filter(|&(a, v)| bus.byte(a) != v).collect();
    o.memory = wrong.is_empty();
    if !o.memory && o.first_difference.is_none() {
        o.first_difference = Some(format!("memory: {wrong:X?} wanted"));
    }
    let expected = tstates(case);
    let want_ports: Vec<(u16, u8, u8)> = ports.iter().map(|p| (p[0].as_i64().unwrap() as u16, p[1].as_i64().unwrap() as u8, p[2].as_str().unwrap().as_bytes()[0])).collect();
    o.cycles = expected.len() == bus.log.len() && expected.iter().zip(&bus.log).all(|(w, g)| same(w, g)) && want_ports == bus.ports;
    if !o.cycles && o.first_difference.is_none() {
        let at = expected.iter().zip(&bus.log).position(|(w, g)| !same(w, g)).unwrap_or(expected.len().min(bus.log.len()));
        o.first_difference = Some(format!("T-state {at}: got {:?} want {:?}; ports got {:?} want {:?}", bus.log.get(at), expected.get(at), bus.ports, want_ports));
    }
    o
}

/// The positive control: the suite's T-states read back into the bus calls that make them, performed in order, ending
/// in the suite's registers.
pub fn replay(case: &Value, bus: &mut FlatBus) -> Registers {
    let l = tstates(case);
    let pins = |i: usize| l.get(i).map(|s| &s.pins);
    let mut i = 0;
    while i < l.len() {
        let a = l[i].address.unwrap_or(0);
        if pins(i + 1) == Some(b"r-m-") && i + 3 < l.len() && l[i + 2].address != l[i].address {
            bus.fetch(a, l[i + 2].address.unwrap_or(0));
            i += 4;
        } else if pins(i + 1) == Some(b"r-m-") {
            bus.read(a);
            i += 3;
        } else if pins(i + 1) == Some(b"-wm-") {
            bus.write(a, l[i + 1].data.unwrap_or(0));
            i += 3;
        } else if pins(i + 2) == Some(b"r--i") {
            bus.input(a);
            i += 4;
        } else if pins(i + 2) == Some(b"-w-i") {
            bus.output(a, l[i + 2].data.unwrap_or(0));
            i += 4;
        } else {
            bus.idle(1);
            i += 1;
        }
    }
    registers(case.get("final").expect("final"))
}

pub fn processor(case: &Value, bus: &mut FlatBus) -> Registers {
    let mut cpu = Z80 { regs: registers(case.get("initial").expect("initial")), ..Z80::new() };
    let _ = cpu.step(bus);
    cpu.regs
}

#[derive(Clone, Debug, Default)]
pub struct Tally {
    pub cases: usize,
    pub passed: usize,
    pub registers: usize,
    pub memory: usize,
    pub cycles: usize,
    pub first_failure: Option<String>,
}

pub fn run_suite(files: &[PathBuf], limit: Option<usize>, processor: fn(&Value, &mut FlatBus) -> Registers) -> Vec<(String, Tally)> {
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
                        t.memory += o.memory as usize;
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
        t.memory += r.memory;
        t.cycles += r.cycles;
        t
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// IN A,(n) as the suite records it: two fetches' worth of T-states, then the port read.
    const CASE: &str = r#"{"name": "DB 0000", "initial": {"pc": 49774, "sp": 7765, "a": 227, "b": 173, "c": 249, "d": 142,
        "e": 238, "f": 140, "h": 21, "l": 190, "i": 62, "r": 0, "ei": 0, "wz": 6102, "ix": 29661, "iy": 5464, "af_": 27987,
        "bc_": 37392, "de_": 52402, "hl_": 41834, "im": 0, "p": 1, "q": 0, "iff1": 0, "iff2": 1, "ram": [[49774, 219],
        [49775, 249]]}, "final": {"a": 155, "b": 173, "c": 249, "d": 142, "e": 238, "f": 140, "h": 21, "l": 190, "i": 62,
        "r": 1, "af_": 27987, "bc_": 37392, "de_": 52402, "hl_": 41834, "ix": 29661, "iy": 5464, "pc": 49776, "sp": 7765,
        "wz": 58362, "iff1": 0, "iff2": 1, "im": 0, "ei": 0, "p": 0, "q": 0, "ram": [[49774, 219], [49775, 249]]},
        "cycles": [[49774, null, "----"], [49774, null, "r-m-"], [15872, 219, "----"], [15872, null, "----"],
        [49775, null, "----"], [49775, null, "r-m-"], [49775, 249, "----"], [58361, null, "----"], [58361, null, "----"],
        [58361, null, "r--i"], [58361, 155, "----"]], "ports": [[58361, 155, "r"]]}"#;

    #[test]
    fn the_replay_passes_and_spoiled_replays_fail_on_the_inline_case() {
        let c = json::parse(CASE.as_bytes()).unwrap();
        let o = grade(&c, &mut replay);
        assert!(o.passed(), "{:?}", o.first_difference);
        let carry = grade(&c, &mut |c, b| {
            let mut r = replay(c, b);
            r.af ^= 1;
            r
        });
        assert!(!carry.registers && carry.memory && carry.cycles);
        let dropped = grade(&c, &mut |c, b| {
            let r = replay(c, b);
            b.log.pop();
            r
        });
        assert!(!dropped.cycles);
        assert!(grade(&c, &mut processor).passed(), "the processor fails the inline case");
    }

    fn corpus() -> Option<Vec<PathBuf>> {
        let files = suite_files();
        if files.is_none() {
            eprintln!("{CORPUS_VARIABLE} does not hold {SUITE}: not run");
        }
        files
    }

    /// The harness loads, answers and records as the suite's environment did (Beryl_Z80.md §3.1).
    #[test]
    fn the_replay_control_passes_every_case() {
        let Some(files) = corpus() else { return };
        let reports = run_suite(&files, None, replay);
        let t = total(&reports);
        eprintln!("{SUITE}: replay {} of {} in {} files", t.passed, t.cases, reports.len());
        for (f, r) in reports.iter().filter(|(_, r)| r.passed != r.cases).take(20) {
            eprintln!("  {f}: {} of {}; {}", r.passed, r.cases, r.first_failure.as_deref().unwrap_or(""));
        }
        assert_eq!(t.passed, t.cases);
    }

    #[test]
    fn spoiled_replays_pass_nothing() {
        let Some(files) = corpus() else { return };
        let sample: Vec<PathBuf> = files.iter().step_by(8).cloned().collect();
        let carry = |c: &Value, b: &mut FlatBus| {
            let mut r = replay(c, b);
            r.af ^= 1;
            r
        };
        let dropped = |c: &Value, b: &mut FlatBus| {
            let r = replay(c, b);
            b.log.pop();
            r
        };
        for (name, spoil) in [("carry", carry as fn(&Value, &mut FlatBus) -> Registers), ("last T-state", dropped)] {
            let t = total(&run_suite(&sample, Some(200), spoil));
            eprintln!("{SUITE}: spoiled ({name}) {} of {}", t.passed, t.cases);
            assert_eq!(t.passed, 0, "spoiled by {name}");
        }
    }

    #[test]
    fn the_processor_against_the_suite() {
        let Some(files) = corpus() else { return };
        let reports = run_suite(&files, None, processor);
        let t = total(&reports);
        eprintln!("{SUITE}: {} of {} (registers {}, memory {}, cycles {})", t.passed, t.cases, t.registers, t.memory, t.cycles);
        for (f, r) in reports.iter().filter(|(_, r)| r.passed != r.cases).take(40) {
            eprintln!("  {f}: {} of {}; {}", r.passed, r.cases, r.first_failure.as_deref().unwrap_or(""));
        }
        assert_eq!(t.passed, t.cases, "a case fails");
    }
}
