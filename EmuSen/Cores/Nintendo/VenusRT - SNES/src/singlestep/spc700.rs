//! SingleStepTests SPC700: one instruction over a flat 64 KiB bus that records each read, write and wait, compared
//! as the 65816 harness compares. See VenusRT_Native.md §2.3.

use std::path::Path;

use emusen_native::json::Value;

use super::{FileReport, Outcome, int, ram, read_cases};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Registers {
    pub pc: u16,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub psw: u8,
}

impl Registers {
    pub fn from_json(v: &Value) -> Registers {
        Registers { pc: int(v, "pc") as u16, a: int(v, "a") as u8, x: int(v, "x") as u8, y: int(v, "y") as u8, sp: int(v, "sp") as u8, psw: int(v, "psw") as u8 }
    }

    fn differences(&self, want: &Registers) -> Option<String> {
        let pairs = [
            ("pc", self.pc as u32, want.pc as u32),
            ("a", self.a as u32, want.a as u32),
            ("x", self.x as u32, want.x as u32),
            ("y", self.y as u32, want.y as u32),
            ("sp", self.sp as u32, want.sp as u32),
            ("psw", self.psw as u32, want.psw as u32),
        ];
        let d: Vec<String> = pairs.iter().filter(|(_, g, w)| g != w).map(|(n, g, w)| format!("{n} got {g:X} want {w:X}")).collect();
        (!d.is_empty()).then(|| d.join(", "))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Read,
    Write,
    Wait,
}

/// One cycle; a wait has neither address nor data in the suite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cycle {
    pub address: Option<u16>,
    pub value: Option<u8>,
    pub kind: Kind,
}

impl Cycle {
    fn from_json(v: &Value) -> Cycle {
        let c = v.as_array().expect("a cycle");
        let kind = match c[2].as_str().expect("a kind") {
            "read" => Kind::Read,
            "write" => Kind::Write,
            "wait" => Kind::Wait,
            other => panic!("cycle kind {other}"),
        };
        Cycle { address: c[0].as_i64().map(|a| a as u16), value: c[1].as_i64().map(|b| b as u8), kind }
    }

    /// A read the suite records without data is a dummy read of memory the case does not list: its address and kind count.
    fn matches(&self, got: &Cycle) -> bool {
        self.address == got.address && self.kind == got.kind && (self.value == got.value || (self.kind == Kind::Read && self.value.is_none()))
    }

    fn text(&self) -> String {
        format!("{} {} {:?}", self.address.map_or("----".into(), |a| format!("{a:04X}")), self.value.map_or("--".into(), |v| format!("{v:02X}")), self.kind)
    }
}

pub trait Bus {
    fn read(&mut self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
    fn wait(&mut self);
}

pub trait Cpu {
    fn set_registers(&mut self, r: &Registers);
    fn registers(&self) -> Registers;
    fn step<B: Bus>(&mut self, bus: &mut B);
    /// SLEEP or STOP ran; the harness then steps on until the suite's cycle count, as for the 65816's WAI and STP.
    fn halted(&self) -> bool {
        false
    }
}

/// 64 KiB of RAM with no I/O page, as the suite assumes.
pub struct FlatBus {
    pub memory: Vec<u8>,
    pub log: Vec<Cycle>,
}

impl Default for FlatBus {
    fn default() -> Self {
        FlatBus { memory: vec![0; 0x10000], log: Vec::new() }
    }
}

impl Bus for FlatBus {
    fn read(&mut self, address: u16) -> u8 {
        let v = self.memory[address as usize];
        self.log.push(Cycle { address: Some(address), value: Some(v), kind: Kind::Read });
        v
    }

    fn write(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
        self.log.push(Cycle { address: Some(address), value: Some(value), kind: Kind::Write });
    }

    fn wait(&mut self) {
        self.log.push(Cycle { address: None, value: None, kind: Kind::Wait });
    }
}

pub struct Expected {
    pub initial: Registers,
    pub initial_ram: Vec<(u32, u8)>,
    pub last: Registers,
    pub last_ram: Vec<(u32, u8)>,
    pub cycles: Vec<Cycle>,
}

impl Expected {
    pub fn from_json(case: &Value) -> Expected {
        let initial = case.get("initial").expect("initial");
        let last = case.get("final").expect("final");
        Expected {
            initial: Registers::from_json(initial),
            initial_ram: ram(initial),
            last: Registers::from_json(last),
            last_ram: ram(last),
            cycles: case.get("cycles").and_then(Value::as_array).expect("cycles").iter().map(Cycle::from_json).collect(),
        }
    }
}

pub fn run_case<C: Cpu>(cpu: &mut C, bus: &mut FlatBus, want: &Expected) -> Outcome {
    bus.memory.fill(0);
    bus.log.clear();
    for &(a, v) in &want.initial_ram {
        bus.memory[a as usize & 0xFFFF] = v;
    }
    cpu.set_registers(&want.initial);
    cpu.step(bus);
    while cpu.halted() && bus.log.len() < want.cycles.len() {
        cpu.step(bus);
    }
    let registers = cpu.registers().differences(&want.last);
    let memory = want.last_ram.iter().find(|&&(a, v)| bus.memory[a as usize & 0xFFFF] != v).map(|&(a, v)| format!("{a:04X} got {:02X} want {v:02X}", bus.memory[a as usize & 0xFFFF]));
    let cycles = bus.log.iter().zip(&want.cycles).enumerate().find(|(_, (g, w))| !w.matches(g)).map(|(i, (g, w))| format!("cycle {i}: got {} want {}", g.text(), w.text()))
        .or_else(|| (bus.log.len() != want.cycles.len()).then(|| format!("{} cycles, want {}", bus.log.len(), want.cycles.len())));
    Outcome { registers: registers.is_none(), memory: memory.is_none(), cycles: cycles.is_none(), capped: false, first_difference: registers.or(memory).or(cycles) }
}

pub fn run_file<C: Cpu>(path: &Path, limit: usize, mut make: impl FnMut(&Expected) -> C) -> FileReport {
    let mut report = FileReport { file: path.file_name().unwrap().to_string_lossy().into_owned(), ..FileReport::default() };
    let mut bus = FlatBus::default();
    for case in read_cases(path).iter().take(limit) {
        let want = Expected::from_json(case);
        let mut cpu = make(&want);
        let o = run_case(&mut cpu, &mut bus, &want);
        report.add(case.get("name").and_then(Value::as_str).unwrap_or("?"), &o);
    }
    report
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spoil {
    None,
    DropLastCycle,
    FlipWrittenBit,
    FlipCarry,
    WaitForRead,
}

/// The positive control, as the 65816's.
pub struct Replay {
    cycles: Vec<Cycle>,
    last: Registers,
    now: Registers,
    spoil: Spoil,
}

impl Replay {
    pub fn new(want: &Expected, spoil: Spoil) -> Replay {
        Replay { cycles: want.cycles.clone(), last: want.last, now: want.initial, spoil }
    }
}

impl Cpu for Replay {
    fn set_registers(&mut self, r: &Registers) {
        self.now = *r;
    }

    fn registers(&self) -> Registers {
        self.now
    }

    fn step<B: Bus>(&mut self, bus: &mut B) {
        let n = self.cycles.len() - (self.spoil == Spoil::DropLastCycle) as usize;
        let (mut flipped, mut swapped) = (false, false);
        for c in self.cycles.iter().take(n) {
            match c.kind {
                Kind::Read if self.spoil == Spoil::WaitForRead && !swapped => {
                    swapped = true;
                    bus.wait();
                }
                Kind::Read => {
                    bus.read(c.address.expect("a read's address"));
                }
                Kind::Write => {
                    let mut v = c.value.expect("a written byte");
                    if self.spoil == Spoil::FlipWrittenBit && !flipped {
                        v ^= 1;
                        flipped = true;
                    }
                    bus.write(c.address.expect("a write's address"), v);
                }
                Kind::Wait => bus.wait(),
            }
        }
        self.now = self.last;
        if self.spoil == Spoil::FlipCarry {
            self.now.psw ^= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{run_suite, suite_dir, suite_files, summary, threads};
    use super::*;
    use emusen_native::json;

    const CASE: &[u8] = br#"{"name": "3F 0000", "initial": {"pc": 9619, "a": 141, "x": 39, "y": 68, "sp": 224, "psw": 215, "ram": [[479, 0], [480, 0], [9619, 63], [9620, 187], [9621, 166]]}, "final": {"a": 141, "x": 39, "y": 68, "sp": 222, "pc": 42683, "psw": 215, "ram": [[479, 150], [480, 37], [9619, 63], [9620, 187], [9621, 166]]}, "cycles": [[9619, 63, "read"], [9620, 187, "read"], [9621, 166, "read"], [null, null, "wait"], [480, 37, "write"], [479, 150, "write"], [null, null, "wait"], [null, null, "wait"]]}"#;

    fn outcome(spoil: Spoil) -> Outcome {
        let want = Expected::from_json(&json::parse(CASE).unwrap());
        run_case(&mut Replay::new(&want, spoil), &mut FlatBus::default(), &want)
    }

    #[test]
    fn the_replay_passes_and_each_spoiling_fails_its_own_check() {
        assert!(outcome(Spoil::None).passed(), "{:?}", outcome(Spoil::None));
        let dropped = outcome(Spoil::DropLastCycle);
        assert!(!dropped.cycles && dropped.memory && dropped.registers);
        let flipped = outcome(Spoil::FlipWrittenBit);
        assert!(!flipped.cycles && !flipped.memory && flipped.registers);
        assert!(!outcome(Spoil::FlipCarry).registers && outcome(Spoil::FlipCarry).cycles);
        assert!(!outcome(Spoil::WaitForRead).cycles);
    }

    #[test]
    fn the_replay_passes_every_case_and_every_spoiling_is_caught() {
        let Some(dir) = suite_dir("SingleStepTests-spc700") else {
            eprintln!("EMUSEN_VENUSRT_CORPUS unset, not run");
            return;
        };
        let files = suite_files(&dir);
        assert_eq!(files.len(), 256);
        let reports = run_suite(&files, threads(), |p| run_file(p, usize::MAX, |w| Replay::new(w, Spoil::None)));
        eprintln!("SPC700 replay: {}", summary(&reports));
        for r in &reports {
            assert_eq!(r.passed, r.cases, "{}: {:?}", r.file, r.first_failure);
        }
        for spoil in [Spoil::DropLastCycle, Spoil::FlipWrittenBit, Spoil::FlipCarry, Spoil::WaitForRead] {
            let reports = run_suite(&files, threads(), |p| {
                let mut r = FileReport::default();
                let mut bus = FlatBus::default();
                for case in read_cases(p).iter().take(200) {
                    let want = Expected::from_json(case);
                    let applies = match spoil {
                        Spoil::FlipWrittenBit => want.cycles.iter().any(|c| c.kind == Kind::Write),
                        Spoil::WaitForRead => want.cycles.iter().any(|c| c.kind == Kind::Read),
                        _ => !want.cycles.is_empty(),
                    };
                    if applies {
                        r.add("", &run_case(&mut Replay::new(&want, spoil), &mut bus, &want));
                    }
                }
                r
            });
            let cases: usize = reports.iter().map(|r| r.cases).sum();
            let passed: usize = reports.iter().map(|r| r.passed).sum();
            eprintln!("{spoil:?}: {cases} cases it applies to, {passed} passed");
            assert!(cases > 0);
            assert_eq!(passed, 0, "{spoil:?} went unnoticed");
        }
    }
}
