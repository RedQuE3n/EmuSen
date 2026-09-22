//! A timed run with the picture on, in one of MarsRT's thread modes, printing ms a frame and the joined state's hash. See Mars_Native.md §5.6.8.
//! `cargo run --release --example threads -- <rom> <state or -> <frames> <plain|threaded|deferred|split> [workers]`

use std::sync::Arc;
use std::time::Instant;

use marsrt::ffi::Core;
use marsrt::machine::Machine;
use marsrt::rom::RomImage;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 5 {
        eprintln!("usage: threads <rom> <state or -> <frames> <plain|threaded|deferred|split> [workers]");
        std::process::exit(2);
    }
    let image = RomImage::from_image(&std::fs::read(&args[1]).expect("the ROM")).expect("a Nintendo 64 image");
    let frames: usize = args[3].parse().expect("a frame count");
    let mode = args[4].as_str();
    let workers: usize = args.get(5).and_then(|w| w.parse().ok()).unwrap_or(4);

    let mut core = Core::new(Machine::load_rom(Arc::new(image), true, None, None));
    core.scanout.repeat_rows = true;
    if args[2] != "-" {
        core.machine.restore_state(&std::fs::read(&args[2]).expect("the state")).expect("a Mars state");
    }
    core.machine.set_rdp_workers(if mode == "split" { workers } else { 1 });
    core.machine.set_threaded_rdp(mode != "plain");
    core.machine.set_deferred(mode == "deferred" || mode == "split");

    let started = Instant::now();
    for _ in 0..frames {
        core.run_frame();
    }
    let elapsed = started.elapsed();

    core.machine.join_presentation(&mut core.scanout);
    core.machine.settle();
    let state = core.machine.save_state_vec(false).expect("a state");
    let hash = state.iter().fold(0xCBF2_9CE4_8422_2325u64, |h, &b| (h ^ b as u64).wrapping_mul(0x0100_0000_01B3));
    println!("{mode} {frames} frames: {:.3} ms a frame; state {hash:016X}", elapsed.as_secs_f64() * 1000.0 / frames as f64);
}
