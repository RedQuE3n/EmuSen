//! `cargo run --release --example frames -- <rom> <frames> [skip]`: the bench's run with no C# in the process, for timing and for rtsample.

use mercuryrt::machine::Machine;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let rom = std::fs::read(&args[1]).expect("rom");
    let frames: usize = args[2].parse().expect("frames");
    let mut m = Machine::load_rom(rom).expect("a supported board");
    *m.bus.ppu.skip_rendering = args.get(3).is_some_and(|s| s == "skip");
    let mut audio = vec![0i16; 1 << 16];
    let drive = |m: &mut Machine, f: usize| {
        let k = f % 90;
        m.bus.joypad.start = k < 5;
        m.bus.joypad.a = (45..50).contains(&k);
    };
    for f in 0..900 {
        drive(&mut m, f);
        m.run_frame().expect("a legal program");
        m.bus.apu.drain(&mut audio, usize::MAX);
    }
    let mut times = Vec::with_capacity(frames);
    for _ in 0..frames {
        let t = Instant::now();
        m.run_frame().expect("a legal program");
        m.bus.apu.drain(&mut audio, usize::MAX);
        times.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    let mean = times.iter().sum::<f64>() / frames as f64;
    times.sort_by(f64::total_cmp);
    println!("{}: mean {mean:.4} p50 {:.4} ms over {frames} frames", args[1], times[frames / 2]);
}
