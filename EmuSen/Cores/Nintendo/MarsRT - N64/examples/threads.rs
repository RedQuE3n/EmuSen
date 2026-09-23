//! A timed run with the picture on, in one of MarsRT's thread modes, printing ms a frame and the joined state's hash. See Mars_Native.md §5.6.8.
//! `cargo run --release --example threads -- <rom> <state or -> <frames> <plain|threaded|deferred|split> [workers] [blocks] [scale=N] [aa=N] [gpu]`;
//! the last three as the shim's `RenderScale`, `Antialiasing` and `Gpu` (Mars_Native.md §6.4).

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
    let blocks = args.iter().skip(5).any(|a| a == "blocks");
    let number = |key: &str| args.iter().skip(5).find_map(|a| a.strip_prefix(key).and_then(|v| v.parse::<i32>().ok())).unwrap_or(1);
    let (scale, aa, gpu) = (number("scale="), number("aa="), args.iter().skip(5).any(|a| a == "gpu"));

    let mut core = Core::new(Machine::load_rom(Arc::new(image), true, None, None));
    core.scanout.repeat_rows = true;
    if args[2] != "-" {
        core.machine.restore_state(&std::fs::read(&args[2]).expect("the state")).expect("a Mars state");
    }
    core.machine.set_rdp_workers(if mode == "split" { workers } else { 1 });
    core.machine.set_threaded_rdp(mode != "plain");
    core.machine.set_deferred(mode == "deferred" || mode == "split");
    core.machine.set_recompiler(blocks);
    core.set_multiple(scale, aa, gpu);
    if gpu {
        eprintln!("device: {}", core.gpu_report());
    }

    let started = Instant::now();
    for _ in 0..frames {
        core.run_frame();
    }
    let elapsed = started.elapsed();
    let per_frame = |nanos: i64| nanos as f64 / 1e6 / frames as f64;
    let waits = core.machine.bus.dp.threads.as_deref().map(|t| {
        let (c, s) = (&t.counters, t.shared());
        let busy = s.drain_nanos.load(std::sync::atomic::Ordering::Relaxed);
        let sites: Vec<String> = c.nanos_per_site.iter().enumerate().filter(|(_, n)| per_frame(**n) >= 0.01).map(|(i, n)| format!("{i}: {:.2}", per_frame(*n))).collect();
        format!("; waited {:.3} ms a frame (by site {}); the first worker busy {:.3}", per_frame(c.nanos_per_site.iter().sum::<i64>()), sites.join(", "), per_frame(busy))
    });

    core.machine.join_presentation(&mut core.scanout);
    core.machine.settle();
    let state = core.machine.save_state_vec(false).expect("a state");
    let hash = state.iter().fold(0xCBF2_9CE4_8422_2325u64, |h, &b| (h ^ b as u64).wrapping_mul(0x0100_0000_01B3));
    println!("{mode} {frames} frames: {:.3} ms a frame; state {hash:016X}{}", elapsed.as_secs_f64() * 1000.0 / frames as f64, waits.unwrap_or_default());
}
