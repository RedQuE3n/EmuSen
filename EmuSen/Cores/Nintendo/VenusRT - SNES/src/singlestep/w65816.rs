//! SingleStepTests 65816: a CPU under test steps one instruction over a flat 16 MiB bus that records each cycle's
//! address, data and pins, and the harness compares registers, memory and the cycle list. See VenusRT_Native.md §2.3.

use std::path::Path;

use emusen_native::json::Value;

use super::{FileReport, Outcome, int, ram, read_cases};

/// MVN and MVP cases stop at this many cycles, mid-move.
pub const CYCLE_CAP: usize = 100;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Registers {
    pub pc: u16,
    pub s: u16,
    pub p: u8,
    pub a: u16,
    pub x: u16,
    pub y: u16,
    pub dbr: u8,
    pub d: u16,
    pub pbr: u8,
    pub e: bool,
}

impl Registers {
    pub fn from_json(v: &Value) -> Registers {
        Registers {
            pc: int(v, "pc") as u16,
            s: int(v, "s") as u16,
            p: int(v, "p") as u8,
            a: int(v, "a") as u16,
            x: int(v, "x") as u16,
            y: int(v, "y") as u16,
            dbr: int(v, "dbr") as u8,
            d: int(v, "d") as u16,
            pbr: int(v, "pbr") as u8,
            e: int(v, "e") != 0,
        }
    }

    fn differences(&self, want: &Registers) -> Option<String> {
        let pairs = [
            ("pc", self.pc as u32, want.pc as u32),
            ("s", self.s as u32, want.s as u32),
            ("p", self.p as u32, want.p as u32),
            ("a", self.a as u32, want.a as u32),
            ("x", self.x as u32, want.x as u32),
            ("y", self.y as u32, want.y as u32),
            ("dbr", self.dbr as u32, want.dbr as u32),
            ("d", self.d as u32, want.d as u32),
            ("pbr", self.pbr as u32, want.pbr as u32),
            ("e", self.e as u32, want.e as u32),
        ];
        let d: Vec<String> = pairs.iter().filter(|(_, g, w)| g != w).map(|(n, g, w)| format!("{n} got {g:X} want {w:X}")).collect();
        (!d.is_empty()).then(|| d.join(", "))
    }
}

/// The pins of one bus cycle, in the suite's order: VDA, VPA, VPB, RWB, E, M, X, MLB.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Signals {
    pub vda: bool,
    pub vpa: bool,
    pub vpb: bool,
    pub write: bool,
    pub e: bool,
    pub m: bool,
    pub x: bool,
    pub mlb: bool,
}

impl Signals {
    pub fn parse(text: &str) -> Option<Signals> {
        let b = text.as_bytes();
        if b.len() != 8 {
            return None;
        }
        let on = |i: usize, c: u8| match b[i] {
            x if x == c => Some(true),
            b'-' => Some(false),
            _ => None,
        };
        Some(Signals {
            vda: on(0, b'd')?,
            vpa: on(1, b'p')?,
            vpb: on(2, b'v')?,
            write: match b[3] {
                b'r' => false,
                b'w' => true,
                _ => return None,
            },
            e: on(4, b'e')?,
            m: on(5, b'm')?,
            x: on(6, b'x')?,
            mlb: on(7, b'l')?,
        })
    }

    pub fn text(&self) -> String {
        let f = |on: bool, c: char| if on { c } else { '-' };
        [f(self.vda, 'd'), f(self.vpa, 'p'), f(self.vpb, 'v'), if self.write { 'w' } else { 'r' }, f(self.e, 'e'), f(self.m, 'm'), f(self.x, 'x'), f(self.mlb, 'l')].iter().collect()
    }

    /// RAM answers only a cycle with VDA, VPA or VPB, as in the suite's own environment.
    pub fn selects_memory(&self) -> bool {
        self.vda || self.vpa || self.vpb
    }
}

/// One bus cycle: its 24-bit address, the byte on the data bus (none on a read RAM did not answer) and the pins;
/// a cycle WAI or STP holds the bus idle has none of the three.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cycle {
    pub address: Option<u32>,
    pub value: Option<u8>,
    pub signals: Option<Signals>,
}

impl Cycle {
    fn from_json(v: &Value) -> Cycle {
        let c = v.as_array().expect("a cycle");
        let pins = c[2].as_str().expect("pins");
        Cycle {
            address: c[0].as_i64().map(|a| a as u32),
            value: c[1].as_i64().map(|b| b as u8),
            signals: if pins == "--------" { None } else { Some(Signals::parse(pins).expect("the pins' letters")) },
        }
    }

    fn text(&self) -> String {
        let address = self.address.map_or("------".into(), |a| format!("{a:06X}"));
        format!("{address} {} {}", self.value.map_or("--".into(), |v| format!("{v:02X}")), self.signals.map_or("--------".into(), |s| s.text()))
    }
}

/// What the CPU under test drives: every cycle is one call, reads and internal cycles alike.
pub trait Bus {
    fn read(&mut self, address: u32, signals: Signals) -> u8;
    fn write(&mut self, address: u32, value: u8, signals: Signals);
    /// A cycle in which WAI or STP holds the bus idle.
    fn halted(&mut self);
}

pub trait Cpu {
    fn set_registers(&mut self, r: &Registers);
    fn registers(&self) -> Registers;
    /// One instruction, or one repetition of MVN and MVP.
    fn step<B: Bus>(&mut self, bus: &mut B);
}

/// 16 MiB of RAM that records each cycle; an unselected read returns zero and records no data.
pub struct FlatBus {
    pub memory: Vec<u8>,
    pub log: Vec<Cycle>,
    touched: Vec<u32>,
}

impl Default for FlatBus {
    fn default() -> Self {
        FlatBus { memory: vec![0; 1 << 24], log: Vec::new(), touched: Vec::new() }
    }
}

impl FlatBus {
    fn load(&mut self, ram: &[(u32, u8)]) {
        for &(a, v) in ram {
            self.memory[a as usize & 0xFF_FFFF] = v;
            self.touched.push(a);
        }
        self.log.clear();
    }

    fn clear(&mut self) {
        for a in self.touched.drain(..) {
            self.memory[a as usize & 0xFF_FFFF] = 0;
        }
    }
}

impl Bus for FlatBus {
    fn read(&mut self, address: u32, signals: Signals) -> u8 {
        let value = signals.selects_memory().then(|| self.memory[address as usize & 0xFF_FFFF]);
        self.log.push(Cycle { address: Some(address), value, signals: Some(signals) });
        value.unwrap_or(0)
    }

    fn write(&mut self, address: u32, value: u8, signals: Signals) {
        if signals.selects_memory() {
            self.memory[address as usize & 0xFF_FFFF] = value;
            self.touched.push(address);
        }
        self.log.push(Cycle { address: Some(address), value: Some(value), signals: Some(signals) });
    }

    fn halted(&mut self) {
        self.log.push(Cycle { address: None, value: None, signals: None });
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

fn first_cycle_difference(got: &[Cycle], want: &[Cycle], same: &impl Fn(&Cycle, &Cycle) -> bool) -> Option<String> {
    for (i, (g, w)) in got.iter().zip(want).enumerate() {
        if !same(g, w) {
            return Some(format!("cycle {i}: got {} want {}", g.text(), w.text()));
        }
    }
    (got.len() != want.len()).then(|| format!("{} cycles, want {}", got.len(), want.len()))
}

/// One case: registers, then memory, then the cycles, each checked on its own.
pub fn run_case<C: Cpu>(cpu: &mut C, bus: &mut FlatBus, want: &Expected) -> Outcome {
    run_case_with(cpu, bus, want, |got, want| got == want)
}

/// As `run_case`, with the cycles compared by `same`, for a disputes-log entry that names a pin difference.
pub fn run_case_with<C: Cpu>(cpu: &mut C, bus: &mut FlatBus, want: &Expected, same: impl Fn(&Cycle, &Cycle) -> bool) -> Outcome {
    bus.load(&want.initial_ram);
    let r = &want.initial;
    let opcode = bus.memory[((r.pbr as usize) << 16) | r.pc as usize];
    let capped = matches!(opcode, 0x44 | 0x54) && want.cycles.len() >= CYCLE_CAP;
    let repeats = matches!(opcode, 0x44 | 0x54 | 0xCB | 0xDB);
    cpu.set_registers(r);
    cpu.step(bus);
    let mut steps = 1;
    while repeats && bus.log.len() < want.cycles.len() && steps < 64 {
        cpu.step(bus);
        steps += 1;
    }
    let mut o = Outcome { capped, ..Outcome::default() };
    let got = if capped { &bus.log[..bus.log.len().min(want.cycles.len())] } else { &bus.log[..] };
    let cycles = first_cycle_difference(got, &want.cycles, &same);
    o.cycles = cycles.is_none();
    if capped {
        o.registers = true;
        o.memory = true;
        o.first_difference = cycles;
    } else {
        let registers = cpu.registers().differences(&want.last);
        let memory = want.last_ram.iter().find(|&&(a, v)| bus.memory[a as usize & 0xFF_FFFF] != v).map(|&(a, v)| format!("{a:06X} got {:02X} want {v:02X}", bus.memory[a as usize & 0xFF_FFFF]));
        o.registers = registers.is_none();
        o.memory = memory.is_none();
        o.first_difference = registers.or(memory).or(cycles);
    }
    bus.clear();
    o
}

/// Every case of one file, with a CPU made for each case.
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

/// A way to spoil the replay, so that the harness is shown to notice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spoil {
    None,
    DropLastCycle,
    FlipWrittenBit,
    FlipCarry,
    FlipMPin,
}

/// The positive control: performs the suite's own cycles and ends in its final registers, spoiled as asked.
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
        let mut flipped = false;
        for (i, c) in self.cycles.iter().take(n).enumerate() {
            let (Some(address), Some(mut pins)) = (c.address, c.signals) else {
                bus.halted();
                continue;
            };
            if i == 0 && self.spoil == Spoil::FlipMPin {
                pins.m = !pins.m;
            }
            if pins.write {
                let mut v = c.value.expect("a written byte");
                if self.spoil == Spoil::FlipWrittenBit && !flipped {
                    v ^= 1;
                    flipped = true;
                }
                bus.write(address, v, pins);
            } else {
                bus.read(address, pins);
            }
        }
        self.cycles.clear();
        self.now = self.last;
        if self.spoil == Spoil::FlipCarry {
            self.now.p ^= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{run_suite, suite_dir, suite_files, summary, threads};
    use super::*;
    use emusen_native::json;

    const CASE: &[u8] = br#"{"name": "95 n 1", "initial": {"pc": 100, "s": 511, "p": 4, "a": 4660, "x": 2, "y": 0, "dbr": 0, "d": 16, "pbr": 1, "e": 0, "ram": [[65636, 149], [65637, 32]]},
        "final": {"pc": 102, "s": 511, "p": 4, "a": 4660, "x": 2, "y": 0, "dbr": 0, "d": 16, "pbr": 1, "e": 0, "ram": [[65636, 149], [65637, 32], [50, 52], [51, 18]]},
        "cycles": [[65636, 149, "dp-r----"], [65637, 32, "-p-r----"], [65637, null, "---r----"], [50, 52, "d--w----"], [51, 18, "d--w----"]]}"#;

    fn outcome(spoil: Spoil) -> Outcome {
        let want = Expected::from_json(&json::parse(CASE).unwrap());
        run_case(&mut Replay::new(&want, spoil), &mut FlatBus::default(), &want)
    }

    #[test]
    fn the_pins_read_and_write_back_as_the_suite_spells_them() {
        for text in ["dp-remx-", "-p-r-mx-", "d--w----", "---r----", "dpvwemxl"] {
            assert_eq!(Signals::parse(text).unwrap().text(), text);
        }
        assert_eq!(Signals::parse("dp-remx"), None);
        assert_eq!(Signals::parse("xp-remx-"), None);
    }

    #[test]
    fn the_replay_passes_and_each_spoiling_fails_its_own_check() {
        assert!(outcome(Spoil::None).passed(), "{:?}", outcome(Spoil::None));
        let dropped = outcome(Spoil::DropLastCycle);
        assert!(!dropped.cycles && !dropped.memory && dropped.registers);
        let flipped = outcome(Spoil::FlipWrittenBit);
        assert!(!flipped.cycles && !flipped.memory && flipped.registers);
        let carry = outcome(Spoil::FlipCarry);
        assert!(carry.cycles && carry.memory && !carry.registers);
        let pin = outcome(Spoil::FlipMPin);
        assert!(!pin.cycles && pin.memory && pin.registers);
    }

    // Coverage: the whole suite read, and the replay passing every case, is what says the harness reads it right.
    #[test]
    fn the_replay_passes_every_case_of_the_suite() {
        let Some(dir) = suite_dir("SingleStepTests-65816") else {
            eprintln!("EMUSEN_VENUSRT_CORPUS unset, not run");
            return;
        };
        let files = suite_files(&dir);
        assert_eq!(files.len(), 512);
        let reports = run_suite(&files, threads(), |p| run_file(p, usize::MAX, |w| Replay::new(w, Spoil::None)));
        eprintln!("65816 replay: {}", summary(&reports));
        for r in &reports {
            assert_eq!(r.passed, r.cases, "{}: {:?}", r.file, r.first_failure);
        }
        let capped: Vec<String> = reports.iter().filter(|r| r.capped > 0).map(|r| format!("{} {}", r.file, r.capped)).collect();
        eprintln!("capped: {}", capped.join(", "));
        assert!(reports.iter().all(|r| r.capped == 0 || r.file.starts_with("44") || r.file.starts_with("54")));
    }

    #[test]
    fn every_spoiling_is_caught_on_every_case_it_applies_to() {
        let Some(dir) = suite_dir("SingleStepTests-65816") else {
            eprintln!("EMUSEN_VENUSRT_CORPUS unset, not run");
            return;
        };
        let files: Vec<_> = suite_files(&dir).into_iter().step_by(8).collect();
        for spoil in [Spoil::DropLastCycle, Spoil::FlipWrittenBit, Spoil::FlipCarry, Spoil::FlipMPin] {
            let reports = run_suite(&files, threads(), |p| {
                let mut r = FileReport::default();
                let mut bus = FlatBus::default();
                for case in read_cases(p).iter().take(500) {
                    let want = Expected::from_json(case);
                    let applies = match spoil {
                        Spoil::FlipWrittenBit => want.cycles.iter().any(|c| c.signals.is_some_and(|s| s.write)),
                        Spoil::FlipCarry => !(want.cycles.len() >= CYCLE_CAP),
                        Spoil::FlipMPin => want.cycles.first().is_some_and(|c| c.signals.is_some()),
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
