//! A timed run with the picture on, in one of MarsRT's thread modes, printing ms a frame and the joined state's hash. See Mars_Native.md §5.6.8.
//! `cargo run --release --example threads -- <rom> <state or -> <frames> <plain|threaded|deferred|split> [workers] [blocks] [scale=N] [aa=N] [gpu] [trace=<file>]`;
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

    // `trace=<file>`: one line a frame, the emulation, the present, the presenter's join and its job, site 8 and the drain's joins, in nanoseconds (Mars_Native.md §6.11, §6.14).
    let trace = args.iter().skip(5).find_map(|a| a.strip_prefix("trace="));
    let mut lines = String::new();
    let started = Instant::now();
    for frame in 0..frames {
        if trace.is_none() {
            core.run_frame();
            continue;
        }
        let (waited, joined, walked) = (core.scanout.presenter_waits().1, core.scanout.joined, core.scanout.presenter_nanos);
        let site8 = core.machine.bus.dp.threads.as_deref().map_or(0, |t| t.counters.nanos_per_site[8]);
        let drain_joined = core.machine.bus.dp.threads.as_deref().map_or(0, |t| t.counters.join_nanos);
        let t0 = Instant::now();
        core.machine.run_frame();
        let t1 = Instant::now();
        let presented = core.machine.present(&mut core.scanout);
        let t2 = Instant::now();
        let site8 = core.machine.bus.dp.threads.as_deref().map_or(0, |t| t.counters.nanos_per_site[8]) - site8;
        let drain_joined = core.machine.bus.dp.threads.as_deref().map_or(0, |t| t.counters.join_nanos) - drain_joined;
        let joined = core.scanout.joined - joined;
        lines += &format!("{frame} {} {} {} {} {} {} {site8}\n", (t1 - t0).as_nanos(), (t2 - t1).as_nanos(), core.scanout.presenter_waits().1 - waited, joined, if joined > 0 { core.scanout.presenter_nanos - walked } else { 0 }, presented.walked as u8);
        lines.pop();
        lines += &format!(" {drain_joined}\n");
    }
    let elapsed = started.elapsed();
    if let Some(path) = trace {
        std::fs::write(path, "frame emulation present join_wait joined presenter walked site8 drain_join\n".to_string() + &lines).expect("the trace");
    }
    let (joins_waited, waited) = core.scanout.presenter_waits();
    let per_frame = |nanos: i64| nanos as f64 / 1e6 / frames as f64;
    let waits = core.machine.bus.dp.threads.as_deref().map(|t| {
        let (c, s) = (&t.counters, t.shared());
        let busy = s.drain_nanos.load(std::sync::atomic::Ordering::Relaxed);
        let sites: Vec<String> = c.nanos_per_site.iter().enumerate().filter(|(_, n)| per_frame(**n) >= 0.01).map(|(i, n)| format!("{i}: {:.2}", per_frame(*n))).collect();
        let (served, device) = (s.device_head.load(std::sync::atomic::Ordering::Relaxed), s.device_nanos.load(std::sync::atomic::Ordering::Relaxed));
        format!(
            "; waited {:.3} ms a frame (by site {}); joined {:.3} ({} joins that waited); the first worker busy {:.3}, on the device's scans {:.3} ({served} handed to it)",
            per_frame(c.nanos_per_site.iter().sum::<i64>()),
            sites.join(", "),
            per_frame(c.join_nanos),
            c.joins,
            per_frame(busy),
            per_frame(device)
        )
    });

    core.machine.join_presentation(&mut core.scanout);
    core.machine.settle();
    let state = core.machine.save_state_vec(false).expect("a state");
    let hash = state.iter().fold(0xCBF2_9CE4_8422_2325u64, |h, &b| (h ^ b as u64).wrapping_mul(0x0100_0000_01B3));
    let presenter = format!("; the presenter's join {:.3} ms a frame ({joins_waited} of {} joins waited), its jobs {:.3}", per_frame(waited), core.scanout.joined, per_frame(core.scanout.presenter_nanos));
    println!("{mode} {frames} frames: {:.3} ms a frame; state {hash:016X}{}{presenter}", elapsed.as_secs_f64() * 1000.0 / frames as f64, waits.unwrap_or_default());
}
