//! The CPU and the S-CPU's bus by whole frames of the machine: a ROM run for some frames, timed (VenusRT_Native.md §12.6).
//! cargo run --release --example frame_cost <rom> <frames>

use std::time::Instant;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let image = std::fs::read(&a[1]).unwrap();
    let frames: i64 = a.get(2).and_then(|f| f.parse().ok()).unwrap_or(600);
    let mut best = f64::MAX;
    for _ in 0..3 {
        let mut m = venusrt::machine::Machine::load_rom(&image).unwrap();
        let start = Instant::now();
        while m.total_frames() < frames {
            m.run_frame();
        }
        best = best.min(start.elapsed().as_secs_f64() * 1e3 / frames as f64);
    }
    println!("{}: {best:.4} ms a frame over {frames} frames, best of three", a[1].rsplit('/').next().unwrap());
}
