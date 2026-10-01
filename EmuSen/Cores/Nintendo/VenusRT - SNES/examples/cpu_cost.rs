//! The 65816's own cost per bus cycle over a flat bus that only counts master clocks: VenusRT_Native.md §10.5.
//! cargo run --release --example cpu_cost [seconds] [moves]; block moves are left out of the mix unless asked for.

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
    fn halted(&mut self) {
        self.clock += 6;
        self.calls += 1;
    }
}

fn main() {
    let seconds: f64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(3.0);
    let moves = std::env::args().nth(2).as_deref() == Some("moves");
    // Stage 1 step 1's mix, for comparison: its opcodes only, straight-line, no restarts.
    let step1 = std::env::args().nth(2).as_deref() == Some("step1");
    let probe = Cpu::default();
    let implemented: Vec<u8> = (0..=255u8)
        .filter(|&op| {
            let mut c = probe;
            let mut b = Flat { memory: vec![op; 0x10000], clock: 0, calls: 0 };
            c.step(&mut b);
            !c.unimplemented && !matches!(op, 0xFB | 0xC2 | 0xE2 | 0xCB | 0xDB) && (moves || !matches!(op, 0x44 | 0x54)) && !(step1 && venusrt::cpu::is_control(op))
        })
        .collect();
    let mut seed = 0x2545_F491_4F6C_DD1Du64;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    // Jumps land on operand bytes too, so no byte anywhere may be WAI or STP.
    let mut operand = || loop {
        let v = next() as u8;
        if v != 0xCB && v != 0xDB && (moves || (v != 0x44 && v != 0x54)) {
            break v;
        }
    };
    let memory: Vec<u8> = (0..0x10000).map(|i| if i % 3 == 0 { implemented[operand() as usize % implemented.len()] } else { operand() }).collect();
    let mut bus = Flat { memory, clock: 0, calls: 0 };
    let mut cpu = Cpu { s: 0x1FF, p: 0x30, ..Cpu::default() };
    let (mut instructions, mut halts) = (0u64, 0u64);
    let start = Instant::now();
    while start.elapsed().as_secs_f64() < seconds {
        for i in 0..100_000u32 {
            // A random program with branches falls into a loop the host predicts; a restart every 32 keeps it random.
            if i % 32 == 0 && !step1 {
                cpu.pc = (i.wrapping_mul(0x9E37_79B9) >> 16) as u16 ^ (instructions as u16);
                cpu.pbr = 0;
            }
            cpu.step(&mut bus);
            instructions += 1;
            if cpu.waiting || cpu.stopped {
                halts += 1;
                cpu.waiting = false;
                cpu.stopped = false;
            }
            cpu.unimplemented = false;
        }
    }
    let t = start.elapsed().as_secs_f64();
    let per_cycle = t * 1e9 / bus.calls as f64;
    let clocks_per_cycle = bus.clock as f64 / bus.calls as f64;
    println!("{halts} halts written by the program's own stores and cleared");
    println!("{instructions} instructions, {} bus cycles in {t:.2} s: {:.2} ns per instruction, {per_cycle:.2} ns per cycle", bus.calls, t * 1e9 / instructions as f64);
    println!("{:.3} ms for an NTSC frame's 357,368 master clocks at {clocks_per_cycle:.2} clocks a cycle", per_cycle * 357_368.0 / clocks_per_cycle / 1e6);
}
