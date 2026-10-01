//! The 65816 under the single-step harness: the CPU's bus calls turned into the suite's cycles, and the grade by
//! opcode group. See VenusRT_Native.md §10.4.

use super::w65816::{self as w, Registers, Signals};
use crate::cpu::{self, Cpu, pin};

struct Adapter<'a, B: w::Bus>(&'a mut B);

fn signals(pins: u8, write: bool) -> Signals {
    Signals {
        vda: pins & pin::VDA != 0,
        vpa: pins & pin::VPA != 0,
        vpb: pins & pin::VPB != 0,
        write,
        e: pins & pin::E != 0,
        m: pins & pin::M != 0,
        x: pins & pin::X != 0,
        mlb: pins & pin::MLB != 0,
    }
}

impl<B: w::Bus> cpu::Bus for Adapter<'_, B> {
    fn read(&mut self, address: u32, pins: u8) -> u8 {
        self.0.read(address, signals(pins, false))
    }

    fn write(&mut self, address: u32, value: u8, pins: u8) {
        self.0.write(address, value, signals(pins, true))
    }

    fn idle(&mut self, address: u32, pins: u8) {
        self.0.read(address, signals(pins & !(pin::VDA | pin::VPA), false));
    }
}

impl w::Cpu for Cpu {
    fn set_registers(&mut self, r: &Registers) {
        // Emulation-mode cases start with a stack high byte no 65816 can hold there; it is loaded as the chip holds it.
        let s = if r.e { 0x0100 | (r.s & 0xFF) } else { r.s };
        *self = Cpu { a: r.a, x: r.x, y: r.y, s, d: r.d, dbr: r.dbr, pbr: r.pbr, pc: r.pc, p: r.p, e: r.e, unimplemented: false };
    }

    fn registers(&self) -> Registers {
        Registers { pc: self.pc, s: self.s, p: self.p, a: self.a, x: self.x, y: self.y, dbr: self.dbr, d: self.d, pbr: self.pbr, e: self.e }
    }

    fn step<B: w::Bus>(&mut self, bus: &mut B) {
        Cpu::step(self, &mut Adapter(bus));
    }
}

/// The opcode's group, as the build record reports them.
pub fn group(opcode: u8) -> &'static str {
    match opcode {
        0xA1 | 0xA3 | 0xA5 | 0xA7 | 0xA9 | 0xAD | 0xAF | 0xB1 | 0xB2 | 0xB3 | 0xB5 | 0xB7 | 0xB9 | 0xBD | 0xBF | 0xA2 | 0xA6
        | 0xB6 | 0xAE | 0xBE | 0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => "load",
        0x81 | 0x83 | 0x85 | 0x87 | 0x8D | 0x8F | 0x91 | 0x92 | 0x93 | 0x95 | 0x97 | 0x99 | 0x9D | 0x9F | 0x86 | 0x96 | 0x8E
        | 0x84 | 0x94 | 0x8C | 0x64 | 0x74 | 0x9C | 0x9E => "store",
        0xAA | 0xA8 | 0x9B | 0xBB | 0xBA | 0x8A | 0x98 | 0x9A | 0x1B | 0x3B | 0x5B | 0x7B | 0xEB | 0xFB => "transfer",
        0x18 | 0x38 | 0x58 | 0x78 | 0xB8 | 0xD8 | 0xF8 | 0xC2 | 0xE2 => "flag",
        o if matches!(o & 0x1F, 0x01 | 0x03 | 0x05 | 0x07 | 0x09 | 0x0D | 0x0F | 0x11 | 0x12 | 0x13 | 0x15 | 0x17 | 0x19 | 0x1D | 0x1F)
            && matches!(o >> 5, 3 | 7) => "add/subtract",
        o if matches!(o & 0x1F, 0x01 | 0x03 | 0x05 | 0x07 | 0x09 | 0x0D | 0x0F | 0x11 | 0x12 | 0x13 | 0x15 | 0x17 | 0x19 | 0x1D | 0x1F)
            && o >> 5 == 6 => "compare",
        0xE0 | 0xE4 | 0xEC | 0xC0 | 0xC4 | 0xCC => "compare",
        o if matches!(o & 0x1F, 0x01 | 0x03 | 0x05 | 0x07 | 0x09 | 0x0D | 0x0F | 0x11 | 0x12 | 0x13 | 0x15 | 0x17 | 0x19 | 0x1D | 0x1F)
            && o >> 5 <= 2 => "logic",
        0x89 | 0x24 | 0x34 | 0x2C | 0x3C => "logic",
        0x0A | 0x2A | 0x4A | 0x6A | 0x1A | 0x3A | 0x06 | 0x16 | 0x0E | 0x1E | 0x26 | 0x36 | 0x2E | 0x3E | 0x46 | 0x56 | 0x4E
        | 0x5E | 0x66 | 0x76 | 0x6E | 0x7E | 0xC6 | 0xD6 | 0xCE | 0xDE | 0xE6 | 0xF6 | 0xEE | 0xFE | 0x04 | 0x0C | 0x14 | 0x1C
        | 0xE8 | 0xC8 | 0xCA | 0x88 => "read-modify-write, increment",
        0xEA => "flag",
        _ => "later steps",
    }
}

#[cfg(test)]
mod tests {
    use super::super::w65816::{FlatBus, run_case, run_file, Expected};
    use super::super::{FileReport, run_suite, suite_dir, suite_files, threads};
    use super::*;
    use std::collections::BTreeMap;

    fn opcode_of(file: &str) -> u8 {
        u8::from_str_radix(&file[..2], 16).unwrap()
    }

    // The whole suite through the CPU; the tally by group and mode, every failing file's first difference, and a
    // floor per group that a change may not lower (VenusRT_Native.md §10.4).
    #[test]
    fn the_cpu_through_the_whole_suite() {
        let Some(dir) = suite_dir("SingleStepTests-65816") else {
            eprintln!("EMUSEN_VENUSRT_CORPUS unset, not run");
            return;
        };
        let reports = run_suite(&suite_files(&dir), threads(), |p| run_file(p, usize::MAX, |_| Cpu::default()));
        let mut groups: BTreeMap<(&str, char), (usize, usize, usize, usize, usize)> = BTreeMap::new();
        let mut report = String::new();
        for r in &reports {
            let mode = if r.file.contains(".e.") { 'e' } else { 'n' };
            let g = groups.entry((group(opcode_of(&r.file)), mode)).or_default();
            g.0 += r.cases;
            g.1 += r.passed;
            g.2 += r.registers;
            g.3 += r.memory;
            g.4 += r.cycles;
            if r.passed != r.cases && group(opcode_of(&r.file)) != "later steps" {
                report += &format!("{} {}/{}: {}\n", r.file, r.passed, r.cases, r.first_failure.as_deref().unwrap_or(""));
            }
        }
        for ((g, m), (n, p, reg, mem, cyc)) in &groups {
            eprintln!("{g:32} {m}: {p:>8}/{n:<8} registers {reg} memory {mem} cycles {cyc}");
        }
        eprint!("{report}");
        if let Ok(path) = std::env::var("EMUSEN_VENUSRT_SINGLESTEP_REPORT") {
            let rows: String = reports.iter().map(|r| format!("{}\t{}\t{}\t{}\t{}\t{}\n", r.file, r.cases, r.passed, r.registers, r.memory, r.cycles)).collect();
            std::fs::write(path, rows).unwrap();
        }
        for ((g, _), (n, p, ..)) in &groups {
            if *g != "later steps" {
                assert_eq!(p, n, "{g}");
            }
        }
    }

    // One case written out by hand, so the CPU's harness wiring is checked without the corpus.
    #[test]
    fn sta_direct_x_in_native_mode_takes_its_documented_cycles() {
        let case = emusen_native::json::parse(br#"{"name": "95 n", "initial": {"pc": 100, "s": 511, "p": 0, "a": 4660, "x": 2, "y": 0, "dbr": 0, "d": 16, "pbr": 1, "e": 0, "ram": [[65636, 149], [65637, 32]]},
            "final": {"pc": 102, "s": 511, "p": 0, "a": 4660, "x": 2, "y": 0, "dbr": 0, "d": 16, "pbr": 1, "e": 0, "ram": [[50, 52], [51, 18]]},
            "cycles": [[65636, 149, "dp-r----"], [65637, 32, "-p-r----"], [65637, null, "---r----"], [65637, null, "---r----"], [50, 52, "d--w----"], [51, 18, "d--w----"]]}"#).unwrap();
        let want = Expected::from_json(&case);
        let o = run_case(&mut Cpu::default(), &mut FlatBus::default(), &want);
        assert!(o.passed(), "{o:?}");
        let _ = FileReport::default();
    }
}
