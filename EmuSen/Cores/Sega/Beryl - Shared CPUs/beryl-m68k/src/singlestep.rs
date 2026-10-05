//! The single-step harness over the 68000 suites: SingleStepTests' `m68000/v1` and TomHarte's `680x0/68000/v1`, with
//! registers, memory and every bus transaction compared. The corpus is found through `EMUSEN_BERYL_CORPUS`; without
//! it, or while the processor is not built, the corpus tests pass unrun. Beryl_M68k.md §3.1 is the protocol.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use emusen_native::json::{self, Value};

use crate::{Access, Bus, M68000, Registers, Size};

pub const CORPUS_VARIABLE: &str = "EMUSEN_BERYL_CORPUS";
/// The two suites' folders under the corpus, the primary first.
pub const SUITES: [&str; 2] = ["m68000/v1", "680x0/68000/v1"];

pub fn suite_dir(suite: &str) -> Option<PathBuf> {
    let dir = Path::new(&std::env::var_os(CORPUS_VARIABLE)?).join(suite);
    dir.is_dir().then_some(dir)
}

pub fn suite_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("the suite's folder")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    files
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

/// One bus transaction in the form the processor's `Bus` sees it: a byte cycle's address carries A0 and its value is
/// the byte. `Idle` runs of the suite and of the processor are merged before they are compared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transaction {
    Idle(u32),
    /// `kind` is the suite's letter: r, w, or re/we for a cycle abandoned on an address error, whose value is not
    /// compared.
    Cycle { kind: [u8; 2], function: u8, address: u32, word: bool, value: u32 },
}

/// The suite's transaction, normalised: SingleStepTests' form drops A0, names the lanes UDS and LDS (1 asserted) and
/// gives a byte cycle's value on its lane, so an upper-lane byte reads shifted left by 8; TomHarte's keeps A0, gives
/// the byte, and has no lanes.
pub fn transaction(t: &Value) -> Transaction {
    let a = t.as_array().expect("a transaction");
    let kind = a[0].as_str().expect("a kind");
    if kind == "n" {
        return Transaction::Idle(a[1].as_i64().expect("clocks") as u32);
    }
    let word = a[4].as_str() == Some(".w");
    let mut address = a[3].as_i64().expect("an address") as u32 & 0xFF_FFFF;
    let mut value = a[5].as_i64().unwrap_or(0) as u32;
    if !word && a.len() >= 8 {
        match (a[6].as_i64(), a[7].as_i64()) {
            (Some(0), Some(1)) => address |= 1,
            (Some(1), Some(0)) => value >>= 8,
            _ => {}
        }
    }
    let k = kind.as_bytes();
    Transaction::Cycle { kind: [k[0], k.get(1).copied().unwrap_or(0)], function: a[2].as_i64().expect("a function code") as u8, address, word, value }
}

/// A transaction as the cycles it stands for: TomHarte's `t`, TAS's ten clocks, is the read (its value not recorded),
/// two idle clocks and the write, as SingleStepTests records them.
pub fn cycles(t: &Value) -> Vec<Transaction> {
    match transaction(t) {
        Transaction::Cycle { kind: [b't', 0], function, address, word, value } => vec![
            Transaction::Cycle { kind: *b"tr", function, address, word, value: 0 },
            Transaction::Idle(2),
            Transaction::Cycle { kind: *b"w\0", function, address, word, value },
        ],
        other => vec![other],
    }
}

pub fn merged(list: impl IntoIterator<Item = Transaction>) -> Vec<Transaction> {
    let mut out: Vec<Transaction> = Vec::new();
    for t in list {
        match (out.last_mut(), t) {
            (Some(Transaction::Idle(n)), Transaction::Idle(m)) => *n += m,
            (_, Transaction::Idle(0)) => {}
            _ => out.push(t),
        }
    }
    out
}

fn same(a: &Transaction, b: &Transaction) -> bool {
    match (a, b) {
        (Transaction::Cycle { kind: k1, function: f1, address: x1, word: w1, .. }, Transaction::Cycle { kind: k2, function: f2, address: x2, word: w2, .. })
            if (k1 == b"tr" && k2[0] == b'r') || (k2 == b"tr" && k1[0] == b'r') =>
        {
            f1 == f2 && x1 == x2 && w1 == w2
        }
        (Transaction::Cycle { kind: [_, b'e'], .. }, Transaction::Cycle { kind: [_, b'e'], .. }) => {
            matches!((a, b), (Transaction::Cycle { kind: k1, function: f1, address: x1, word: w1, .. }, Transaction::Cycle { kind: k2, function: f2, address: x2, word: w2, .. }) if k1 == k2 && f1 == f2 && x1 & !1 == x2 & !1 && w1 == w2)
        }
        _ => a == b,
    }
}

/// The PC each suite records, less the opcode's address: SingleStepTests' is the next prefetch's, four on.
pub fn pc_offset(suite: &str) -> u32 {
    if suite.starts_with("m68000") { 4 } else { 0 }
}

/// The registers a case names, A7 as the stack pointer of the mode its SR selects, PC as the opcode's address.
pub fn registers(v: &Value, offset: u32) -> Registers {
    let mut r = Registers::default();
    for i in 0..8 {
        r.d[i] = int(v, &format!("d{i}"));
    }
    for i in 0..7 {
        r.a[i] = int(v, &format!("a{i}"));
    }
    r.sr = int(v, "sr") as u16;
    r.pc = int(v, "pc").wrapping_sub(offset);
    let (usp, ssp) = (int(v, "usp"), int(v, "ssp"));
    (r.a[7], r.other_sp) = if r.supervisor() { (ssp, usp) } else { (usp, ssp) };
    let p = v.get("prefetch").and_then(Value::as_array).expect("prefetch");
    r.prefetch = [p[0].as_i64().expect("a word") as u16, p[1].as_i64().expect("a word") as u16];
    r
}

fn ram(v: &Value) -> Vec<(u32, u8)> {
    v.get("ram")
        .and_then(Value::as_array)
        .expect("ram")
        .iter()
        .map(|p| {
            let p = p.as_array().expect("a pair");
            (p[0].as_i64().expect("an address") as u32 & 0xFF_FFFF, p[1].as_i64().expect("a byte") as u8)
        })
        .collect()
}

/// A flat 16 MiB of RAM holding only what the case lists, recording every transaction.
#[derive(Default)]
pub struct FlatBus {
    pub ram: BTreeMap<u32, u8>,
    pub log: Vec<Transaction>,
}

impl FlatBus {
    fn byte(&self, a: u32) -> u8 {
        self.ram.get(&(a & 0xFF_FFFF)).copied().unwrap_or(0)
    }
}

impl Bus for FlatBus {
    fn read(&mut self, a: Access) -> u16 {
        let value = match a.size {
            Size::Word => (self.byte(a.address & !1) as u16) << 8 | self.byte(a.address | 1) as u16,
            Size::Byte => self.byte(a.address) as u16,
        };
        self.log.push(Transaction::Cycle { kind: *b"r\0", function: a.function, address: a.address & 0xFF_FFFF, word: a.size == Size::Word, value: value as u32 });
        value
    }

    fn write(&mut self, a: Access, value: u16) {
        match a.size {
            Size::Word => {
                self.ram.insert(a.address & 0xFF_FFFE, (value >> 8) as u8);
                self.ram.insert((a.address | 1) & 0xFF_FFFF, value as u8);
            }
            Size::Byte => {
                self.ram.insert(a.address & 0xFF_FFFF, value as u8);
            }
        }
        let value = if a.size == Size::Word { value } else { value & 0xFF };
        self.log.push(Transaction::Cycle { kind: *b"w\0", function: a.function, address: a.address & 0xFF_FFFF, word: a.size == Size::Word, value: value as u32 });
    }

    fn idle(&mut self, clocks: u32) {
        self.log.push(Transaction::Idle(clocks));
    }

    fn address_error(&mut self, a: Access, write: bool) {
        self.log.push(Transaction::Cycle { kind: [if write { b'w' } else { b'r' }, b'e'], function: a.function, address: a.address & 0xFF_FFFF, word: a.size == Size::Word, value: 0 });
    }

    fn interrupt_level(&mut self) -> u8 {
        0
    }

    fn acknowledge(&mut self, _level: u8) -> Option<u8> {
        None
    }
}

/// What one case showed: which of the three checks held, and the first difference.
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

/// A case run through `processor`, which steps once over the bus and returns the registers it ends with.
pub fn grade(case: &Value, offset: u32, processor: &mut dyn FnMut(&Value, u32, &mut FlatBus) -> Registers) -> Outcome {
    let initial = case.get("initial").expect("initial");
    let fin = case.get("final").expect("final");
    let mut bus = FlatBus { ram: ram(initial).into_iter().collect(), log: Vec::new() };
    let got = processor(case, offset, &mut bus);
    let want = registers(fin, offset);
    let mut o = Outcome { registers: got == want, ..Outcome::default() };
    if !o.registers {
        o.first_difference = Some(format!("registers: got {got:X?} want {want:X?}"));
    }
    let wrong: Vec<_> = ram(fin).into_iter().filter(|&(a, v)| bus.byte(a) != v).collect();
    o.memory = wrong.is_empty();
    if !o.memory && o.first_difference.is_none() {
        o.first_difference = Some(format!("memory: {:X?} wanted", &wrong[..wrong.len().min(4)]));
    }
    let expected = merged(case.get("transactions").and_then(Value::as_array).expect("transactions").iter().flat_map(cycles));
    let actual = merged(bus.log);
    o.cycles = expected.len() == actual.len() && expected.iter().zip(&actual).all(|(a, b)| same(a, b));
    if !o.cycles && o.first_difference.is_none() {
        let at = expected.iter().zip(&actual).position(|(a, b)| !same(a, b)).unwrap_or(expected.len().min(actual.len()));
        o.first_difference = Some(format!("transaction {at}: got {:?} want {:?}", actual.get(at), expected.get(at)));
    }
    o
}

/// The positive control: the suite's own transactions performed through the bus, ending in the suite's registers.
pub fn replay(case: &Value, offset: u32, bus: &mut FlatBus) -> Registers {
    for t in case.get("transactions").and_then(Value::as_array).expect("transactions").iter().flat_map(cycles) {
        match t {
            Transaction::Idle(n) => bus.idle(n),
            Transaction::Cycle { kind, function, address, word, value } => {
                let access = Access { address, size: if word { Size::Word } else { Size::Byte }, function, locked: kind == *b"tr" };
                match kind {
                    [b'r', 0] | [b't', b'r'] => {
                        bus.read(access);
                    }
                    [b'w', 0] => bus.write(access, value as u16),
                    [k, b'e'] => bus.address_error(access, k == b'w'),
                    _ => panic!("an unknown transaction {kind:?}"),
                }
            }
        }
    }
    registers(case.get("final").expect("final"), offset)
}

/// The processor under test, from the case's initial registers, for one step.
pub fn processor(case: &Value, offset: u32, bus: &mut FlatBus) -> Registers {
    let mut cpu = M68000 { regs: registers(case.get("initial").expect("initial"), offset), ..M68000::new() };
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

/// Every file through `run`, a few at a time, each case graded with `processor`; the first `limit` cases a file, when
/// given.
pub fn run_suite(files: &[PathBuf], offset: u32, limit: Option<usize>, processor: fn(&Value, u32, &mut FlatBus) -> Registers) -> Vec<(String, Tally)> {
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
                        let o = grade(case, offset, &mut |c, f, b| processor(c, f, b));
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

    const CASE: &str = r#"{"name": "t", "initial": {"d0": 0, "d1": 0, "d2": 0, "d3": 0, "d4": 0, "d5": 0, "d6": 0, "d7": 0,
        "a0": 0, "a1": 0, "a2": 0, "a3": 0, "a4": 0, "a5": 0, "a6": 0, "usp": 16, "ssp": 32, "sr": 9984, "pc": 1024,
        "prefetch": [1, 2], "ram": [[2000, 7], [2001, 7]]}, "final": {"d0": 0, "d1": 0, "d2": 0, "d3": 0, "d4": 0, "d5": 0, "d6": 0,
        "d7": 0, "a0": 0, "a1": 0, "a2": 0, "a3": 0, "a4": 0, "a5": 0, "a6": 0, "usp": 16, "ssp": 32, "sr": 9984, "pc": 1026,
        "prefetch": [2, 3], "ram": [[2000, 7], [2001, 9]]}, "transactions": [["n", 2], ["n", 2], ["r", 4, 5, 2000, ".b", 7, 0, 1],
        ["w", 4, 5, 2000, ".b", 9, 0, 1], ["r", 4, 5, 2000, ".b", 1792, 1, 0], ["re", 4, 5, 3001, ".w", 0, 1, 1]], "length": 16}"#;

    #[test]
    fn transactions_normalise_both_suites_forms() {
        let c = json::parse(CASE.as_bytes()).unwrap();
        let t = merged(c.get("transactions").unwrap().as_array().unwrap().iter().flat_map(cycles));
        assert_eq!(t[0], Transaction::Idle(4));
        assert_eq!(t[1], Transaction::Cycle { kind: *b"r\0", function: 5, address: 2001, word: false, value: 7 });
        assert_eq!(t[3], Transaction::Cycle { kind: *b"r\0", function: 5, address: 2000, word: false, value: 7 });
        let tomharte = json::parse(br#"["r", 4, 5, 2001, ".b", 7]"#).unwrap();
        assert_eq!(transaction(&tomharte), t[1]);
        let tas = json::parse(br#"["t", 10, 5, 2001, ".b", 9]"#).unwrap();
        assert_eq!(cycles(&tas).len(), 3);
        let r = registers(c.get("initial").unwrap(), 4);
        assert_eq!((r.a[7], r.other_sp, r.ssp(), r.usp(), r.pc), (32, 16, 32, 16, 1020));
    }

    #[test]
    fn the_replay_passes_and_a_spoiled_replay_fails_on_the_inline_case() {
        let c = json::parse(CASE.as_bytes()).unwrap();
        assert!(grade(&c, 4, &mut replay).passed());
        let o = grade(&c, 4, &mut |c, f, b| {
            let mut r = replay(c, f, b);
            r.sr ^= 1;
            r
        });
        assert!(!o.registers && o.memory && o.cycles);
    }

    fn corpus(suite: &str) -> Option<Vec<PathBuf>> {
        let dir = suite_dir(suite);
        if dir.is_none() {
            eprintln!("{CORPUS_VARIABLE} does not hold {suite}: not run");
        }
        dir.map(|d| suite_files(&d))
    }

    /// The harness loads, answers and records as the suites' environment did (Beryl_M68k.md §3.1).
    #[test]
    fn the_replay_control_passes_every_case() {
        for suite in SUITES {
            let Some(files) = corpus(suite) else { continue };
            let reports = run_suite(&files, pc_offset(suite), None, replay);
            let t = total(&reports);
            eprintln!("{suite}: replay {} of {} in {} files", t.passed, t.cases, reports.len());
            for (f, r) in reports.iter().filter(|(_, r)| r.passed != r.cases) {
                eprintln!("  {f}: {} of {}; {}", r.passed, r.cases, r.first_failure.as_deref().unwrap_or(""));
            }
            assert_eq!(t.passed, t.cases, "{suite}");
        }
    }

    /// Each spoiling must fail every case it applies to, on the first 200 cases of every eighth file.
    #[test]
    fn spoiled_replays_pass_nothing() {
        for suite in SUITES {
            let Some(files) = corpus(suite) else { continue };
            let sample: Vec<PathBuf> = files.iter().step_by(8).cloned().collect();
            let carry = |c: &Value, f: u32, b: &mut FlatBus| {
                let mut r = replay(c, f, b);
                r.sr ^= 1;
                r
            };
            let dropped = |c: &Value, f: u32, b: &mut FlatBus| {
                let r = replay(c, f, b);
                b.log.pop();
                r
            };
            for (name, spoil) in [("carry", carry as fn(&Value, u32, &mut FlatBus) -> Registers), ("last transaction", dropped)] {
                let t = total(&run_suite(&sample, pc_offset(suite), Some(200), spoil));
                eprintln!("{suite}: spoiled ({name}) {} of {}", t.passed, t.cases);
                assert_eq!(t.passed, 0, "{suite} spoiled by {name}");
            }
        }
    }

    /// The processor against the suites, file by file, written to `EMUSEN_BERYL_REPORT` when it is set. Every case of
    /// SingleStepTests' suite must pass; TomHarte's is measured, its disagreements being Beryl_M68k.md §3.3's.
    #[test]
    fn the_processor_against_the_suites() {
        let mut lines = Vec::new();
        let mut failed_primary = false;
        for suite in SUITES {
            let Some(files) = corpus(suite) else { continue };
            let reports = run_suite(&files, pc_offset(suite), None, processor);
            let t = total(&reports);
            let line = format!("{suite}: {} of {} (registers {}, memory {}, cycles {})", t.passed, t.cases, t.registers, t.memory, t.cycles);
            eprintln!("{line}");
            lines.push(line);
            for (f, r) in &reports {
                lines.push(format!("  {f}\t{}\t{}\t{}\t{}\t{}\t{}", r.cases, r.passed, r.registers, r.memory, r.cycles, r.first_failure.as_deref().unwrap_or("")));
            }
            if suite == SUITES[0] {
                failed_primary = t.passed != t.cases;
            }
        }
        if let Some(path) = std::env::var_os("EMUSEN_BERYL_REPORT") {
            std::fs::write(path, lines.join("\n") + "\n").unwrap();
        }
        assert!(!failed_primary, "a SingleStepTests case fails");
    }

    /// SingleStepTests' cases the processor misses, each written to the `.jsonl` file `EMUSEN_BERYL_MISSES` names with
    /// the processor's own result, for the referee (Beryl_M68k.md §6); a measurement, failing nothing.
    #[test]
    fn the_primary_misses_listed() {
        let Some(out) = std::env::var_os("EMUSEN_BERYL_MISSES") else { return };
        let Some(files) = corpus(SUITES[0]) else { return };
        let mut lines = String::new();
        for path in &files {
            let file = path.file_name().unwrap().to_string_lossy().into_owned();
            for (index, case) in read_cases(path).into_iter().enumerate() {
                if grade(&case, 4, &mut processor).passed() {
                    continue;
                }
                let initial = case.get("initial").unwrap();
                let mut bus = FlatBus { ram: ram(initial).into_iter().collect(), log: Vec::new() };
                let mut cpu = M68000 { regs: registers(initial, 4), ..M68000::new() };
                let _ = cpu.step(&mut bus);
                lines.push_str(&listing_line(&file, index, &case, "miss", &cpu.regs, &merged(bus.log)));
            }
        }
        std::fs::write(out, lines).expect("the listing");
    }

    /// A development aid: `EMUSEN_BERYL_EXPLAIN=<suite folder>/<file>[:<skip>]` prints the first failing cases of a
    /// file after `skip`, with both transaction lists and the registers.
    #[test]
    fn explain() {
        let Some(spec) = std::env::var("EMUSEN_BERYL_EXPLAIN").ok() else { return };
        let (path, skip) = spec.split_once(':').map_or((spec.as_str(), 0), |(p, n)| (p, n.parse().unwrap_or(0)));
        let suite = SUITES.iter().find(|s| path.starts_with(*s)).expect("a suite folder");
        let file = Path::new(&std::env::var_os(CORPUS_VARIABLE).unwrap()).join(path);
        let offset = pc_offset(suite);
        let mut shown = 0;
        for case in read_cases(&file).iter().skip(skip) {
            let o = grade(case, offset, &mut processor);
            if o.passed() {
                continue;
            }
            let initial = case.get("initial").unwrap();
            let mut bus = FlatBus { ram: ram(initial).into_iter().collect(), log: Vec::new() };
            let got = processor(case, offset, &mut bus);
            eprintln!("== {} {}", case.get("name").and_then(Value::as_str).unwrap_or("?"), o.first_difference.unwrap_or_default());
            eprintln!("   initial {:X?}", registers(initial, offset));
            eprintln!("   want    {:X?}", registers(case.get("final").unwrap(), offset));
            eprintln!("   got     {got:X?}");
            eprintln!("   want tx {:?}", merged(case.get("transactions").unwrap().as_array().unwrap().iter().flat_map(cycles)));
            eprintln!("   got tx  {:?}", merged(bus.log));
            shown += 1;
            if shown == 4 {
                break;
            }
        }
    }

    /// One disputed case as a JSON line: where it is, its class, and the registers and transactions the processor gave.
    fn listing_line(file: &str, index: usize, case: &Value, class: &str, r: &Registers, tx: &[Transaction]) -> String {
        let name = case.get("name").and_then(Value::as_str).unwrap_or("?").replace('"', "'");
        let (usp, ssp) = if r.supervisor() { (r.other_sp, r.a[7]) } else { (r.a[7], r.other_sp) };
        let mut regs: Vec<String> = (0..8).map(|i| format!("\"d{i}\":{}", r.d[i])).collect();
        regs.extend((0..7).map(|i| format!("\"a{i}\":{}", r.a[i])));
        regs.push(format!("\"usp\":{usp},\"ssp\":{ssp},\"sr\":{},\"pc\":{}", r.sr, r.pc));
        let tx: Vec<String> = tx
            .iter()
            .map(|t| match *t {
                Transaction::Idle(n) => format!("[\"n\",{n}]"),
                Transaction::Cycle { kind, function, address, word, value } => {
                    let k: String = kind.iter().filter(|&&b| b != 0).map(|&b| b as char).collect();
                    format!("[\"{k}\",{function},{address},{},{value}]", if word { "\"w\"" } else { "\"b\"" })
                }
            })
            .collect();
        format!("{{\"file\":\"{file}\",\"index\":{index},\"name\":\"{name}\",\"class\":\"{class}\",\"final\":{{{}}},\"transactions\":[{}]}}\n", regs.join(","), tx.join(","))
    }

    /// TomHarte's suite against SingleStepTests', where they disagree: each failing case of the second suite put in
    /// the first of the classes Beryl_M68k.md §3.3 names, printed with counts; a measurement, failing nothing. When the
    /// variable names a `.jsonl` file, each case is also written there with the processor's own result, for the
    /// referee's comparison (§6).
    #[test]
    fn the_second_suite_disagreements() {
        if std::env::var_os("EMUSEN_BERYL_DISAGREEMENTS").is_none() {
            return;
        }
        let Some(files) = corpus(SUITES[1]) else { return };
        let listing = std::env::var("EMUSEN_BERYL_DISAGREEMENTS").ok().filter(|p| p.ends_with(".jsonl"));
        let mut lines = String::new();
        let mut classes: BTreeMap<String, usize> = BTreeMap::new();
        for path in &files {
            let file = path.file_name().unwrap().to_string_lossy().into_owned();
            for (index, case) in read_cases(path).into_iter().enumerate() {
                if grade(&case, 0, &mut processor).passed() {
                    continue;
                }
                let initial = case.get("initial").unwrap();
                let mut bus = FlatBus { ram: ram(initial).into_iter().collect(), log: Vec::new() };
                let mut cpu = M68000 { regs: registers(initial, 0), ..M68000::new() };
                let step = cpu.step(&mut bus);
                let want = merged(case.get("transactions").unwrap().as_array().unwrap().iter().flat_map(cycles));
                let got = merged(bus.log.clone());
                let strip = |l: &[Transaction]| -> Vec<Transaction> {
                    l.iter().map(|t| match *t {
                        Transaction::Cycle { kind, address, word, value, .. } => Transaction::Cycle { kind, function: 0, address, word, value },
                        t => t,
                    }).collect()
                };
                let class = if step == crate::Step::Exception(3) {
                    "address error".to_string()
                } else if strip(&want) == strip(&got) && registers(case.get("final").unwrap(), 0) == cpu.regs {
                    "function code only".to_string()
                } else {
                    format!("other: {file}")
                };
                if listing.is_some() {
                    lines.push_str(&listing_line(&file, index, &case, &class, &cpu.regs, &got));
                }
                *classes.entry(class).or_default() += 1;
            }
        }
        for (c, n) in &classes {
            eprintln!("{n:8} {c}");
        }
        if let Some(p) = listing {
            std::fs::write(&p, lines).expect("the listing");
        }
    }
}
