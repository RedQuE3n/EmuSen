//! The SPC700 under the single-step harness, graded by opcode group with and without the cycle lists. See
//! VenusRT_Native.md §21.

use super::spc700::{self as s, Registers};
use crate::apu::spc700::{self as cpu, Spc700};

struct Adapter<'a, B: s::Bus>(&'a mut B);

impl<B: s::Bus> cpu::Bus for Adapter<'_, B> {
    fn read(&mut self, address: u16) -> u8 {
        self.0.read(address)
    }

    fn write(&mut self, address: u16, value: u8) {
        self.0.write(address, value)
    }

    fn idle(&mut self) {
        self.0.wait()
    }
}

impl s::Cpu for Spc700 {
    fn set_registers(&mut self, r: &Registers) {
        *self = Spc700 { pc: r.pc, a: r.a, x: r.x, y: r.y, sp: r.sp, psw: r.psw, stopped: false };
    }

    fn registers(&self) -> Registers {
        Registers { pc: self.pc, a: self.a, x: self.x, y: self.y, sp: self.sp, psw: self.psw }
    }

    fn step<B: s::Bus>(&mut self, bus: &mut B) {
        Spc700::step(self, &mut Adapter(bus));
    }

    fn halted(&self) -> bool {
        self.stopped
    }
}

/// The opcode's group, as the build record reports them.
pub fn group(op: u8) -> &'static str {
    let lo = op & 0x1F;
    match op {
        0x9E | 0xCF => "multiply and divide",
        0x7A | 0x9A | 0x5A | 0x3A | 0x1A | 0xBA | 0xDA => "16-bit",
        0xDF | 0xBE | 0x9F | 0x0E | 0x4E => "decimal, XCN, test-and-set",
        0x60 | 0x80 | 0x20 | 0x40 | 0xE0 | 0xED | 0xA0 | 0xC0 | 0x00 | 0xEF | 0xFF => "flags and control",
        0xEA | 0xCA | 0xAA | 0x0A | 0x2A | 0x4A | 0x6A | 0x8A => "one-bit",
        0x2D | 0x4D | 0x6D | 0x0D | 0xAE | 0xCE | 0xEE | 0x8E | 0x7D | 0x5D | 0xDD | 0xFD | 0x9D | 0xBD => "stack and transfer",
        0x5F | 0x1F | 0x3F | 0x4F | 0x0F | 0x6F | 0x7F => "jump, call, return",
        _ if lo == 0x01 || lo == 0x11 => "jump, call, return",
        0x10 | 0x30 | 0x50 | 0x70 | 0x90 | 0xB0 | 0xD0 | 0xF0 | 0x2F | 0x2E | 0xDE | 0x6E | 0xFE => "branch",
        _ if lo == 0x03 || lo == 0x13 => "branch",
        _ if lo == 0x02 || lo == 0x12 => "one-bit",
        0xE8 | 0xCD | 0x8D | 0xE6 | 0xBF | 0xE4 | 0xF8 | 0xEB | 0xF4 | 0xF9 | 0xFB | 0xE5 | 0xE9 | 0xEC | 0xF5 | 0xF6 | 0xE7 | 0xF7 => "load",
        0xC4 | 0xD8 | 0xCB | 0xD4 | 0xDB | 0xD9 | 0xC5 | 0xC9 | 0xCC | 0xD5 | 0xD6 | 0xC6 | 0xAF | 0xC7 | 0xD7 | 0x8F | 0xFA => "store",
        0xC8 | 0xAD | 0x3E | 0x7E | 0x1E | 0x5E => "ALU",
        _ if (op >> 5) <= 5 && matches!(lo & 0x0F, 0x04..=0x09) || matches!(lo, 0x14..=0x19) => "ALU",
        _ => "shift, increment, decrement",
    }
}

#[cfg(test)]
mod tests {
    use super::super::{FileReport, run_suite, suite_dir, suite_files, threads};
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn every_opcode_has_a_group_and_the_groups_cover_the_set() {
        let mut groups = BTreeMap::new();
        for op in 0..=255u8 {
            *groups.entry(group(op)).or_insert(0) += 1;
        }
        assert_eq!(groups.values().sum::<i32>(), 256);
        assert_eq!(groups.len(), 12, "{groups:?}");
    }

    // The suite through the SPC700, by group; the counts are VenusRT_Native.md §21's.
    #[test]
    fn the_spc700_suite_with_cycles() {
        let Some(dir) = suite_dir("SingleStepTests-spc700") else {
            eprintln!("EMUSEN_VENUSRT_CORPUS unset, not run");
            return;
        };
        let files = suite_files(&dir);
        let reports = run_suite(&files, threads(), |p| s::run_file(p, usize::MAX, |_| Spc700::default()));
        let mut by: BTreeMap<&str, (usize, usize, usize, usize)> = BTreeMap::new();
        let mut failing = Vec::new();
        for r in &reports {
            let op = u8::from_str_radix(&r.file[..2], 16).expect("an opcode's file");
            let g = by.entry(group(op)).or_default();
            *g = (g.0 + r.cases, g.1 + r.passed, g.2 + r.state, g.3 + 1);
            if r.passed != r.cases {
                failing.push((r.file.clone(), r.cases - r.passed, r.first_failure.clone()));
            }
        }
        for (g, (cases, passed, state, files)) in &by {
            eprintln!("{g}: {files} opcodes, {cases} cases: {passed} with cycles, {state} on state");
        }
        for f in &failing {
            eprintln!("  {} failing {}: {:?}", f.0, f.1, f.2);
        }
        let all: FileReport = reports.iter().fold(FileReport::default(), |mut a, r| {
            a.cases += r.cases;
            a.passed += r.passed;
            a.state += r.state;
            a
        });
        eprintln!("SPC700: {} of {} with cycles, {} on state", all.passed, all.cases, all.state);
        assert_eq!(all.cases, 256_000);
    }
}
