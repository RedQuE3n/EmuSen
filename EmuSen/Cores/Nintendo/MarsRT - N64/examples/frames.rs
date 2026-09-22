//! Runs a cartridge, from power-on or a state, for a number of frames, and prints the time and the final state's hash.
//! `cargo run --release --example frames -- <rom> <state or -> <frames> [plain] [scan] [blocks]`, for profiling. See Mars_Native.md §5.2.

use std::sync::Arc;
use std::time::Instant;

use marsrt::ffi::Core;
use marsrt::machine::Machine;
use marsrt::rom::RomImage;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("usage: frames <rom> <state or -> <frames> [plain] [scan]");
        std::process::exit(2);
    }
    let image = RomImage::from_image(&std::fs::read(&args[1]).expect("the ROM")).expect("a Nintendo 64 image");
    let frames: usize = args[3].parse().expect("a frame count");
    let plain = args.iter().any(|a| a == "plain");
    let scan = args.iter().any(|a| a == "scan");
    let blocks = args.iter().any(|a| a == "blocks");

    let mut core = Core::new(Machine::load_rom(Arc::new(image), true, None, None));
    core.skip_rendering = !scan;
    core.scanout.repeat_rows = true;
    core.machine.options.idle_skip = !plain;
    core.machine.set_recompiler(blocks);
    if args[2] != "-" {
        core.machine.restore_state(&std::fs::read(&args[2]).expect("the state")).expect("a Mars state");
    }

    let started = Instant::now();
    for _ in 0..frames {
        core.run_frame();
    }
    let elapsed = started.elapsed();

    core.machine.settle();
    let state = core.machine.save_state_vec(false).expect("a state");
    let hash = state.iter().fold(0xCBF2_9CE4_8422_2325u64, |h, &b| (h ^ b as u64).wrapping_mul(0x0100_0000_01B3));
    println!(
        "{frames} frames in {:.3} s, {:.3} ms a frame; {} instructions, {} idle turns passed; state {hash:016X}",
        elapsed.as_secs_f64(),
        elapsed.as_secs_f64() * 1000.0 / frames as f64,
        core.machine.cpu.instructions,
        core.machine.cpu.run.idle_turns_passed,
    );
    if blocks {
        let s = core.machine.blocks.stats;
        println!(
            "blocks: {} live, {} shaped, {} discarded; {} entries, {} instructions in blocks ({:.1} an entry), {} stepped, {} mapped",
            core.machine.blocks.live(),
            s.shaped,
            s.discarded,
            s.entries,
            s.instructions,
            s.instructions as f64 / s.entries.max(1) as f64,
            s.stepped,
            s.mapped
        );
    }
}
