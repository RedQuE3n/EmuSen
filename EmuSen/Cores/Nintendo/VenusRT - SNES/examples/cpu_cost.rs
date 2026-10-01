//! The 65816's own cost per bus cycle over a flat bus that only counts master clocks: VenusRT_Native.md §10.5.
//! cargo run --release --example cpu_cost [seconds]

use std::time::Instant;
use venusrt::cpu::{Bus, Cpu, pin};

struct Flat {
    memory: Vec<u8>,
    clock: u64,
    calls: u64,
}

impl Bus for Flat {
    #[inline(always)]
    fn read(&mut self, address: u32, _pins: u8) -> u8 {
        self.clock += 8;
        self.calls += 1;
        self.memory[address as usize & 0xFFFF]
    }
    #[inline(always)]
    fn write(&mut self, address: u32, value: u8, pins: u8) {
        self.clock += 8;
        self.calls += 1;
        if pins & pin::VDA != 0 {
            self.memory[address as usize & 0xFFFF] = value;
        }
    }
    #[inline(always)]
    fn idle(&mut self, _address: u32, _pins: u8) {
        self.clock += 6;
        self.calls += 1;
    }
}

fn main() {
    let seconds: f64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(3.0);
    let probe = Cpu::default();
    let implemented: Vec<u8> = (0..=255u8)
        .filter(|&op| {
            let mut c = probe;
            let mut b = Flat { memory: vec![op; 0x10000], clock: 0, calls: 0 };
            c.step(&mut b);
            !c.unimplemented && !matches!(op, 0xFB | 0xC2 | 0xE2)
        })
        .collect();
    let mut seed = 0x2545_F491_4F6C_DD1Du64;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let memory: Vec<u8> = (0..0x10000).map(|i| if i % 3 == 0 { implemented[next() as usize % implemented.len()] } else { next() as u8 }).collect();
    let mut bus = Flat { memory, clock: 0, calls: 0 };
    let mut cpu = Cpu { s: 0x1FF, p: 0x30, ..Cpu::default() };
    let mut instructions = 0u64;
    let start = Instant::now();
    while start.elapsed().as_secs_f64() < seconds {
        for _ in 0..100_000 {
            cpu.step(&mut bus);
            instructions += 1;
            cpu.unimplemented = false;
        }
    }
    let t = start.elapsed().as_secs_f64();
    let per_cycle = t * 1e9 / bus.calls as f64;
    let clocks_per_cycle = bus.clock as f64 / bus.calls as f64;
    println!("{instructions} instructions, {} bus cycles in {t:.2} s: {:.2} ns per instruction, {per_cycle:.2} ns per cycle", bus.calls, t * 1e9 / instructions as f64);
    println!("{:.3} ms for an NTSC frame's 357,368 master clocks at {clocks_per_cycle:.2} clocks a cycle", per_cycle * 357_368.0 / clocks_per_cycle / 1e6);
}
