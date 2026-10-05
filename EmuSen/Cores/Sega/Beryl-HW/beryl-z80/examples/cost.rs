//! The Z80's own cost: a loop of ordinary instructions on a flat RAM bus that counts T-states, timed on this machine
//! (Beryl_Z80.md §6). `cargo run --release --example cost [instructions]`.

use beryl_z80::{Bus, Step, Z80};
use std::time::Instant;

struct Ram {
    mem: Vec<u8>,
    t: u64,
}

impl Bus for Ram {
    fn fetch(&mut self, a: u16, _: u16) -> u8 {
        self.t += 4;
        self.mem[a as usize]
    }
    fn read(&mut self, a: u16) -> u8 {
        self.t += 3;
        self.mem[a as usize]
    }
    fn write(&mut self, a: u16, v: u8) {
        self.t += 3;
        self.mem[a as usize] = v;
    }
    fn input(&mut self, _: u16) -> u8 {
        self.t += 4;
        0xFF
    }
    fn output(&mut self, _: u16, _: u8) {
        self.t += 4;
    }
    fn idle(&mut self, t: u32) {
        self.t += t as u64;
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

/// A sound driver's kind of loop: a table read, arithmetic, a store, a branch, a call with an indexed load and an
/// OUT, and DJNZ.
const PROGRAM: [(u16, &[u8]); 2] = [
    (0x100, &[0x21, 0x00, 0x40, 0x11, 0x00, 0x50, 0x06, 0x80, 0x7E, 0x23, 0x81, 0x12, 0x13, 0x4F, 0xE6, 0x0F, 0x28, 0x01, 0x0C, 0xC5, 0xCD, 0x00, 0x02, 0xC1, 0x10, 0xEE, 0xC3, 0x00, 0x01]),
    (0x200, &[0xDD, 0x7E, 0x01, 0xD3, 0x7F, 0xC9]),
];

fn main() {
    let n: u64 = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(50_000_000);
    let mut results = Vec::new();
    for _ in 0..3 {
        let mut ram = Ram { mem: vec![0; 0x10000], t: 0 };
        for (base, bytes) in PROGRAM {
            ram.mem[base as usize..][..bytes.len()].copy_from_slice(bytes);
        }
        let mut cpu = Z80::new();
        cpu.regs.sp = 0xF000;
        cpu.regs.ix = 0x6000;
        cpu.regs.pc = 0x100;
        let t = Instant::now();
        for _ in 0..n {
            if cpu.step(&mut ram) != Step::Instruction {
                panic!("the loop left its instructions at {:X}", cpu.regs.pc);
            }
        }
        results.push((t.elapsed().as_secs_f64(), ram.t));
    }
    results.sort_by(|a, b| a.0.total_cmp(&b.0));
    let (s, t) = results[1];
    let rate = t as f64 / s;
    let frame = 3_579_545.0 / 59.922_743;
    println!("{n} instructions: median {s:.3} s, {:.2} ns an instruction, {:.1} T-states an instruction", s * 1e9 / n as f64, t as f64 / n as f64);
    println!("{:.1} million emulated T-states a second, {:.1} times the Genesis's 3.58 MHz; a frame's Z80 work ({frame:.0} T-states) in {:.3} ms", rate / 1e6, rate / 3_579_545.0, frame / rate * 1e3);
}
