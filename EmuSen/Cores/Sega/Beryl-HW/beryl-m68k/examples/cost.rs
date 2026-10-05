//! The 68000's own cost: a loop of ordinary instructions on a flat RAM bus that counts clocks, timed on this machine
//! (Beryl_M68k.md §5.6). `cargo run --release --example cost [instructions]`.

use beryl_m68k::{Access, Bus, M68000, Size, Step};
use std::time::Instant;

struct Ram {
    mem: Vec<u8>,
    clocks: u64,
}

impl Bus for Ram {
    fn read(&mut self, a: Access) -> u16 {
        self.clocks += 4;
        let i = a.address as usize;
        match a.size {
            Size::Word => u16::from_be_bytes([self.mem[i], self.mem[i + 1]]),
            Size::Byte => self.mem[i] as u16,
        }
    }
    fn write(&mut self, a: Access, v: u16) {
        self.clocks += 4;
        let i = a.address as usize;
        match a.size {
            Size::Word => self.mem[i..i + 2].copy_from_slice(&v.to_be_bytes()),
            Size::Byte => self.mem[i] = v as u8,
        }
    }
    fn idle(&mut self, clocks: u32) {
        self.clocks += clocks as u64;
    }
    fn interrupt_level(&mut self) -> u8 {
        0
    }
    fn acknowledge(&mut self, _: u8) -> Option<u8> {
        None
    }
}

const PROGRAM: [(u32, &[u16]); 2] = [
    (0x400, &[0x41F9, 0x0001, 0x0000, 0x43F9, 0x0002, 0x0000, 0x3E3C, 0x00FF, 0x22D8, 0xD240, 0xE74A, 0xC8C3, 0xBA41, 0x6702, 0x5246, 0x4EB9, 0x0000, 0x0500, 0x51CF, 0xFFEA, 0x60D6]),
    (0x500, &[0x3010, 0x4E75]),
];

fn main() {
    let n: u64 = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(50_000_000);
    let mut results = Vec::new();
    for _ in 0..3 {
        let mut ram = Ram { mem: vec![0; 1 << 24], clocks: 0 };
        for (base, words) in PROGRAM {
            for (i, w) in words.iter().enumerate() {
                ram.mem[base as usize + 2 * i..][..2].copy_from_slice(&w.to_be_bytes());
            }
        }
        let mut cpu = M68000::new();
        cpu.regs.sr = 0x2700;
        cpu.regs.a[7] = 0xF000;
        cpu.regs.pc = 0x400;
        cpu.regs.prefetch = [0x41F9, 0x0001];
        let t = Instant::now();
        for _ in 0..n {
            if cpu.step(&mut ram) != Step::Instruction {
                panic!("the loop left its instructions at {:X}", cpu.regs.pc);
            }
        }
        let s = t.elapsed().as_secs_f64();
        results.push((s, ram.clocks));
    }
    results.sort_by(|a, b| a.0.total_cmp(&b.0));
    let (s, clocks) = results[1];
    let ns = s * 1e9 / n as f64;
    let rate = clocks as f64 / s;
    let frame = 7_670_454.0 / 59.922_743;
    println!("{n} instructions: median {s:.3} s, {ns:.2} ns an instruction, {:.1} clocks an instruction", clocks as f64 / n as f64);
    println!("{:.1} million emulated clocks a second, {:.1} times the Genesis's 7.67 MHz; a frame's 68000 work ({frame:.0} clocks) in {:.3} ms", rate / 1e6, rate / 7_670_454.0, frame / rate * 1e3);
}
