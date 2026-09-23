//! The signal processor alone on a game's own microcode: machines taken while it ran in a gameplay state, each run again a fixed
//! number of steps one tick at a time and to its events, the events themselves untimed. See Mars_Native.md §6.12.
//! `cargo run --release --example rsp_bench -- <rom> <state> [snapshots=24] [steps=4000] [rounds=5]`; `MODES=tick,whole,tickb,wholeb`.

use std::sync::Arc;
use std::time::{Duration, Instant};

use marsrt::machine::Machine;
use marsrt::rom::RomImage;

fn hash(m: &Machine, h: u64) -> u64 {
    let p = &m.bus.sp.processor;
    let mut h = h;
    let mut eat = |b: u8| h = (h ^ b as u64).wrapping_mul(0x0100_0000_01B3);
    for w in p.gpr.iter().chain([p.pc, p.next_pc].iter()) {
        w.to_le_bytes().into_iter().for_each(&mut eat);
    }
    for r in p.vector.iter().chain(p.accumulator.iter()) {
        r.iter().for_each(|e| e.to_le_bytes().into_iter().for_each(&mut eat));
    }
    m.bus.sp_dmem.iter().for_each(|b| eat(*b));
    h
}

fn is_event(m: &Machine) -> bool {
    let pc = m.bus.sp.processor.pc as usize;
    let word = u32::from_be_bytes(m.bus.sp_imem[pc..pc + 4].try_into().unwrap());
    word >> 26 == 0x10 || (word >> 26 == 0 && word & 0x3F == 0x0D)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let image = RomImage::from_image(&std::fs::read(&args[1]).expect("the ROM")).expect("a Nintendo 64 image");
    let count: usize = args.get(3).and_then(|a| a.parse().ok()).unwrap_or(24);
    let steps: u64 = args.get(4).and_then(|a| a.parse().ok()).unwrap_or(4000);
    let rounds: usize = args.get(5).and_then(|a| a.parse().ok()).unwrap_or(5);
    let mut m = Machine::load_rom(Arc::new(image), true, None, None);
    m.restore_state(&std::fs::read(&args[2]).expect("the state")).expect("a Mars state");

    // Whole machines with a running processor, spaced through the frames that follow the state.
    let mut shots = Vec::new();
    let mut since = u64::MAX / 2;
    while shots.len() < count {
        m.run_steps(500);
        since += 500;
        if !m.bus.sp.processor.halted && since >= 40_000 {
            shots.push(m.clone());
            since = 0;
        }
    }

    // `NOP=1`: one processor running no-operations from address 0, which leaves the step's own cost alone.
    if std::env::var("NOP").is_ok_and(|v| v == "1") {
        let mut idle = Machine::new(m.rdram_bytes()).expect("a machine");
        idle.bus.sp.processor.start(0);
        shots = vec![idle];
    }

    let modes = std::env::var("MODES").unwrap_or("tick,whole".into());
    for mode in modes.split(',') {
        let (tick, blocks) = (mode.starts_with('t'), mode.ends_with('b'));
        let mut best = f64::MAX;
        let (mut total, mut events, mut h) = (0u64, 0u64, 0u64);
        for _ in 0..rounds {
            let (mut spent, mut ran_all, mut ev) = (Duration::ZERO, 0u64, 0u64);
            h = 0xCBF2_9CE4_8422_2325;
            for s in &shots {
                let mut m = s.clone();
                set_blocks(&mut m, blocks);
                let mut ran = 0u64;
                while ran < steps && !m.bus.sp.processor.halted {
                    if is_event(&m) {
                        m.bus.sp_step(1);
                        ran += 1;
                        ev += 1;
                        continue;
                    }
                    let started = Instant::now();
                    if tick {
                        while ran < steps && !is_event(&m) {
                            m.bus.sp_step(1);
                            ran += 1;
                        }
                    } else {
                        ran += run_pure(&mut m, steps - ran);
                    }
                    spent += started.elapsed();
                }
                ran_all += ran;
                h = hash(&m, h);
            }
            best = best.min(spent.as_nanos() as f64 / (ran_all - ev) as f64);
            total = ran_all;
            events = ev;
        }
        println!("{mode:6}: {best:.3} ns a step over {} steps and {events} events untimed ({} snapshots); processors {h:016X}", total - events, shots.len());
    }
}

fn run_pure(m: &mut Machine, budget: u64) -> u64 {
    m.bus.rsp_run_pure(budget)
}

fn set_blocks(m: &mut Machine, on: bool) {
    m.set_rsp_blocks(on);
}
