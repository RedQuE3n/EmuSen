//! Stage S32's bench (Sapphire_Native.md §43): one workload run headless on the player's BIOS for a fixed count of
//! frames, each frame's `advance` timed and nothing else, and the units' own counts over the frames measured.
//!
//! `cargo run --release --example bench -- <workload> <bios> <americas|europe|japan> [image] [frames] [--from N]
//! [--times FILE] [--every K DIR] [--registers] [--press F:BITS]... [--card] [--blocks] [--idle] [--gs-workers N] [--edram]`
//!
//! - `boot`: the console started empty with no disc, the BIOS booting to OSDSYS; slot 1 empty unless `--card`.
//! - `elf <program.elf>`: an EE program entered by the fast boot (Sapphire_Native.md §36), a ps2autotests program.
//! - `disc <image>`: a disc put in the drive of a console started empty and the program its SYSTEM.CNF names entered by
//!   the fast boot (§40, §40.14): a synthetic disc of an IOP program, or a game's own program.
//!
//! Prints one line in Diamond's bench form (the mean, median, 95th percentile and worst milliseconds a frame from frame
//! N, default 0), the frames a second and the wall time of that window, and the digests of every frame's picture, of the
//! sound and of the final state, each FNV-1a and taken outside the timed span; then the units' counts a frame over the
//! window: the EE's steps by mode and its cycles, the IOP's instructions, and each VU's pairs in micro mode. `--times`
//! writes each frame's milliseconds, `--every K DIR` the picture every K frames as `DIR/frame-<n>.ppm`, and
//! `--registers` counts every register access the units answer over the window (S24's reports, which slow the run, so
//! its times are not the bench's) and prints the most frequent a frame, for finding which device a processor waits on.
//! `--press F:BITS` holds the pad's buttons (the core's bit numbers, `+`-joined) in port 1 for six frames from frame F.
//! `--blocks` runs the EE recompiled (S36, S37), with the settings the environment gives (`Blocks::from_env`), and prints
//! the dispatcher's counts; without it the EE is interpreted, the reference.
//! `--idle` turns idle skipping on (S33, Sapphire_Native.md §47); without it the waits are stepped, the reference.
//! `--gs-workers N` draws on N workers (S43, Sapphire_Native.md §50; `auto` for the automatic count, 1 the GS's own
//! thread) and prints their counts; `--edram` digests the GS's local memory after every frame, outside the timed span,
//! which joins the workers there, so its times are not the bench's.
//! The BIOS
//! image is handed to the machine and never read here; nothing printed holds an address, an instruction or memory.

use std::time::Instant;

use emusen_native::core::{Core, Kept};
use emusen_native::ffi::StateMachine;
use sapphire::machine::{Machine, Market};
use sapphire::media::{Media, Sectors};

const BASIS: u64 = 0xcbf2_9ce4_8422_2325;

fn fnv(h: u64, b: &[u8]) -> u64 {
    b.iter().fold(h, |h, &x| (h ^ x as u64).wrapping_mul(0x0000_0100_0000_01B3))
}

/// The units' counts at one instant.
#[derive(Clone, Copy, Default)]
struct Counts {
    modes: [u64; 5],
    ee_instructions: u64,
    ee_cycles: u64,
    iop_instructions: u64,
    pairs: [u64; 2],
}

fn counts(m: &Machine) -> Counts {
    let s = &m.sys;
    Counts {
        modes: s.ee.modes,
        ee_instructions: s.ee.timing.instructions,
        ee_cycles: s.board.now,
        iop_instructions: s.board.iop.instructions,
        pairs: s.board.vpu.pairs_run(),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| args.iter().position(|a| a == name).map(|i| args[i + 1].clone());
    let plain: Vec<&String> = {
        let mut v = Vec::new();
        let mut i = 0;
        while i < args.len() {
            if matches!(args[i].as_str(), "--from" | "--times" | "--press" | "--gs-workers") {
                i += 2;
                continue;
            }
            if args[i] == "--every" {
                i += 3;
                continue;
            }
            if !args[i].starts_with("--") {
                v.push(&args[i]);
            }
            i += 1;
        }
        v
    };
    let usage = "bench <boot|elf|disc> <bios> <americas|europe|japan> [image] [frames] [--from N] [--times FILE] [--every K DIR] [--registers] [--press F:BITS]... [--card] [--blocks] [--idle] [--gs-workers N] [--edram]";
    let workload = plain.first().unwrap_or_else(|| panic!("{usage}")).as_str();
    let bios = std::fs::read(plain.get(1).unwrap_or_else(|| panic!("{usage}"))).expect("the BIOS");
    let market = match plain.get(2).map(|s| s.as_str()) {
        Some("europe") => Market::Europe,
        Some("japan") => Market::Japan,
        _ => Market::Americas,
    };
    let (mut m, name, rest) = match workload {
        "boot" => {
            let mut m = Machine::new(Media::Empty, market, [Some("ps2.dualshock2"), None]);
            m.insert_bios(&bios);
            (m, "boot".to_string(), 3)
        }
        "elf" | "disc" => {
            let path = plain.get(3).unwrap_or_else(|| panic!("{usage}"));
            let name = std::path::Path::new(path.as_str()).file_name().unwrap().to_string_lossy().into_owned();
            (machine(workload, path, market, &bios), name, 4)
        }
        other => panic!("{other}: {usage}"),
    };
    if !args.iter().any(|a| a == "--card") {
        m.sys.board.iop.bus.sio2.slots[0].cards[0] = None;
    }
    if args.iter().any(|a| a == "--blocks") {
        m.blocks = Some(Box::new(sapphire::blocks::Blocks::from_env()));
    }
    m.idle.on = args.iter().any(|a| a == "--idle");
    if let Some(n) = flag("--gs-workers") {
        m.sys.board.gs.set_workers(sapphire::gs::workers::parse(&n).expect("a count of workers or auto"));
    }
    let edram_each = args.iter().any(|a| a == "--edram");
    let mut edram = BASIS;
    let frames: usize = plain.get(rest).map_or(600, |f| f.parse().expect("a count of frames"));
    let from: usize = flag("--from").map_or(0, |f| f.parse().expect("a frame"));
    let every = args.iter().position(|a| a == "--every").map(|i| (args[i + 1].parse::<usize>().expect("a count of frames"), args[i + 2].clone()));
    if let Some((_, dir)) = &every {
        std::fs::create_dir_all(dir).expect("the pictures' folder");
    }
    let registers = args.iter().any(|a| a == "--registers");
    let presses: Vec<(usize, u32)> = args
        .windows(2)
        .filter(|w| w[0] == "--press")
        .map(|w| {
            let (f, bits) = w[1].split_once(':').expect("--press F:BITS");
            (f.parse().expect("a frame"), bits.split('+').fold(0u32, |m, b| m | 1 << b.parse::<u32>().expect("a bit")))
        })
        .collect();
    let (mut picture, mut sound) = (BASIS, BASIS);
    let mut chunk = vec![0i16; 1 << 16];
    let mut times = Vec::with_capacity(frames);
    let mut at_from = counts(&m);
    let mut wall = 0.0;
    let started = Instant::now();
    for f in 0..frames {
        if f == from {
            at_from = counts(&m);
            if registers {
                m.sys.board.reports.registers.clear();
                m.sys.board.iop.bus.reports.registers.clear();
                m.sys.board.reports.all = true;
                m.sys.board.iop.bus.reports.all = true;
            }
        }
        let held = presses.iter().filter(|&&(at, _)| f + 1 >= at && f + 1 < at + 6).fold(0, |m, &(_, b)| m | b);
        Core::set_buttons(&mut m, 0, held, 0xFFFF).expect("port 1");
        let start = Instant::now();
        m.advance();
        let t = start.elapsed().as_secs_f64() * 1e3;
        times.push(t);
        if f >= from {
            wall += t;
        }
        picture = fnv(picture, &m.sys.board.pcrtc.picture.rgba);
        if edram_each {
            edram = m.sys.board.gs.local().words().iter().fold(edram, |h, &w| (h ^ u64::from(w)).wrapping_mul(0x0000_0100_0000_01B3));
        }
        let n = m.drain_audio(&mut chunk, usize::MAX);
        sound = chunk[..n].iter().fold(sound, |h, &s| fnv(h, &s.to_le_bytes()));
        if let Some((k, dir)) = &every
            && (f + 1) % k == 0
        {
            write_ppm(&m, &std::path::Path::new(dir).join(format!("frame-{}.ppm", f + 1)));
        }
        if (f + 1) % 100 == 0 {
            eprintln!("frame {} at {:.0} s", f + 1, started.elapsed().as_secs_f64());
        }
    }
    let end = counts(&m);
    if let Some(file) = flag("--times") {
        let text: String = times.iter().map(|t| format!("{t:.4}\n")).collect();
        std::fs::write(file, text).expect("written");
    }
    let mut s = vec![0; m.state_size()];
    let n = m.save_state(&mut s).expect("the state saves");
    let state = fnv(BASIS, &s[..n]);
    let mut window = times[from.min(times.len())..].to_vec();
    let k = window.len().max(1) as f64;
    let mean = window.iter().sum::<f64>() / k;
    window.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let at = |q: f64| window.get(((window.len() as f64 * q) as usize).min(window.len().saturating_sub(1))).copied().unwrap_or(0.0);
    println!(
        "{name}\tframes {from}-{frames}\tmean {mean:.3}\tmedian {:.3}\tp95 {:.3}\tmax {:.3}\tfps {:.3}\twall {:.2}\tpicture {picture:016X}\tsound {sound:016X}\tstate {state:016X}",
        at(0.5),
        at(0.95),
        at(1.0),
        1e3 * k / wall.max(1e-9),
        wall / 1e3
    );
    let d = |a: u64, b: u64| (b - a) as f64 / k;
    let steps: u64 = (0..5).map(|i| end.modes[i] - at_from.modes[i]).sum();
    let share: Vec<String> = (0..5).map(|i| format!("{:.1}", 100.0 * (end.modes[i] - at_from.modes[i]) as f64 / steps.max(1) as f64)).collect();
    let ee = d(at_from.ee_instructions, end.ee_instructions);
    let cycles = d(at_from.ee_cycles, end.ee_cycles);
    println!(
        "ee\t{ee:.0} instructions a frame\t{cycles:.0} cycles a frame\t{:.2} cycles an instruction\tsteps by mode user/supervisor/kernel/level-1/level-2 % {}\t{:.1} ns of the frame an instruction",
        cycles / ee.max(1.0),
        share.join("/"),
        mean * 1e6 / ee.max(1.0)
    );
    println!("iop\t{:.0} instructions a frame", d(at_from.iop_instructions, end.iop_instructions));
    for u in 0..2 {
        let p = d(at_from.pairs[u], end.pairs[u]);
        println!("vu{u}\t{p:.0} pairs a frame\t{:.2}% of the EE's cycles", 100.0 * p / cycles.max(1.0));
    }
    if registers {
        let b = &m.sys.board;
        let mut all: Vec<_> = b.reports.registers.iter().chain(b.iop.bus.reports.registers.iter()).collect();
        all.sort_by_key(|(_, r)| std::cmp::Reverse(r.count));
        for ((side, a, acc), r) in all.into_iter().take(16) {
            println!("register\t{side:?} {a:08X}h {acc:?}\t{:.0} a frame", r.count as f64 / k);
        }
    }
    let gs = &m.sys.board.gs;
    if edram_each {
        println!("edram\t{edram:016X}\tafter every frame");
    }
    if let Some(c) = gs.worker_counts() {
        let n = gs.worker_count();
        let waits: Vec<String> = c.waits.iter().map(|&(k, ns)| format!("{k}/{:.1}ms", ns as f64 / 1e6)).collect();
        let busy: Vec<String> = (0..n).map(|w| format!("{:.1}/{:.1}", c.busy[w] as f64 / 1e6 / frames as f64, c.stalled[w] as f64 / 1e6 / frames as f64)).collect();
        println!(
            "gs\t{n} workers\tprimitives {} split {} alone {} (pages met {}, texture met {}) empty {}\tuploads {}\tclut copies {}\twaits ring/clut/download/copy/clutload/join/display {}\tbusy/stalled ms a frame {}",
            c.primitives,
            c.split,
            c.alone,
            c.alone_why[0],
            c.alone_why[1],
            c.empty,
            c.uploads,
            c.clut_copies,
            waits.join(" "),
            busy.join(" ")
        );
    }
    if let Some(b) = m.blocks.as_deref() {
        let s = b.stats;
        let (compiled, refused, bytes) = b.compiled();
        println!(
            "blocks\tmemory {}\tentered {}\tcompiled steps {}\tinterpreted {}\trollbacks {}\trun again {}\tkept past {}\tDMAC stops {}\tpages filled {}\tgenerations {}\tleft at the event/misaligned/page/miss/accelerated load {:?}\tcompiled {compiled}\trefused {refused}\tbytes {bytes}",
            b.memory, s.entered, s.compiled_steps, s.interpreted, s.rollbacks, s.rerun_steps, s.kept_past, s.dmac_stops, s.fills, s.generations, s.left
        );
    }
    let cd = &m.sys.board.iop.bus.cdvd;
    println!(
        "machine\tentered at frame {:?}\tstopped {:?}\tdrive N commands {}\tS commands {}",
        m.entered,
        m.stop,
        cd.n_count.iter().sum::<u32>(),
        cd.s_count.iter().sum::<u32>()
    );
}

/// The machine for an EE program or a disc, its program waiting for the fast boot.
fn machine(workload: &str, path: &str, market: Market, bios: &[u8]) -> Machine {
    let image = std::fs::read(path).expect("the image");
    let ports = [Some("ps2.dualshock2"), None];
    match (workload, Media::read(&image).expect("an EE program or a disc")) {
        ("elf", media @ Media::Elf(_)) => {
            let mut m = Machine::new(media, market, ports);
            m.insert_bios(bios);
            m.program = Some(image);
            m
        }
        ("disc", Media::Disc(d)) => {
            let name = d.program.rsplit(['\\', ':']).next().unwrap_or("").split(';').next().unwrap_or("").to_string();
            let program = Sectors::new(&image).root_file(&name).unwrap_or_else(|| panic!("the disc's root holds no {name}"));
            let disc = sapphire::cdvd::open(Kept::from(image), Vec::new()).expect("the disc opens");
            let mut m = Machine::new(Media::Empty, market, ports);
            m.insert_bios(bios);
            m.set_tray(true);
            m.insert_disc(Some(disc));
            m.set_tray(false);
            m.program = Some(program);
            m
        }
        (w, _) => panic!("{path} is not the image `{w}` takes"),
    }
}

fn write_ppm(m: &Machine, path: &std::path::Path) {
    let p = &m.sys.board.pcrtc.picture;
    let mut v = format!("P6\n{} {}\n255\n", p.geometry.width, p.geometry.height).into_bytes();
    for px in p.rgba.chunks(4) {
        v.extend_from_slice(&px[..3]);
    }
    std::fs::write(path, v).expect("the picture is written");
}
