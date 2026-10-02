//! The S-SMP alone: a ROM run to a frame, then its sound unit copied out and run on for more frames' worth of master
//! clock with the ports held, timed (VenusRT_Native.md §21). `apu_cost <rom> [at] [frames]`.

use std::time::Instant;

#[path = "common/load.rs"]
mod load;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let at: i64 = a.get(2).and_then(|f| f.parse().ok()).unwrap_or(600);
    let frames: u64 = a.get(3).and_then(|f| f.parse().ok()).unwrap_or(600);
    let mut m = load::machine(&a[1]);
    while m.total_frames() < at {
        m.run_frame();
    }
    let start_clock = m.sys.timing.clock;
    let per_frame = if m.sys.apu.ratio == venusrt::apu::smp::PAL_RATIO { 1364 * 312 } else { 357_366 };
    let mut best = f64::MAX;
    let mut cycles = 0;
    for _ in 0..3 {
        let mut apu = m.sys.apu.clone();
        let first = apu.cycles;
        let t = Instant::now();
        for f in 1..=frames {
            apu.run_to(start_clock + f * per_frame);
        }
        best = best.min(t.elapsed().as_secs_f64() * 1e3 / frames as f64);
        cycles = (apu.cycles - first) / frames;
    }
    println!("{}: SPC700 {best:.4} ms a frame ({cycles} cycles a frame) over {frames} frames from frame {at}, best of three", a[1].rsplit('/').next().unwrap());
}
