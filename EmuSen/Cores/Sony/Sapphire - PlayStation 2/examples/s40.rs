//! Stage S40's runner (Sapphire_Native.md §45.7): a workload on the player's BIOS with VU1 on the tier named, each frame's
//! `advance` timed and nothing else, and after each frame (outside the timed span) the digests of its picture, its sound
//! and the whole state written to a file, so that two tiers' runs are compared frame by frame. The BIOS image is handed
//! to the machine and never read here; nothing printed holds an address, an instruction or memory contents.
//!
//! `cargo run --release --example s40 -- <boot|disc> <bios> <americas|europe|japan> [image] <frames> <interpreter|recompiler>
//! [--digests FILE] [--clock] [--from N] [--state FILE] [--save-at N --save FILE]`
//!
//! - `boot`: the console started with no disc, the BIOS booting to OSDSYS.
//! - `disc <image>`: the disc in the drive of a console started empty, the program its SYSTEM.CNF names entered by the
//!   fast boot (Sapphire_Native.md §40), read in place.
//!
//! Prints the frames' mean, median, 95th percentile and worst milliseconds from frame N, the frames a second, the digests
//! of every picture, the sound and the last state; VU1's pairs a frame, compiled and interpreted; with `--clock`, VU1's
//! time-stamp counter ticks running pairs against the frames', its share of the frame (the clock's reads cost time, so
//! a run with it is not timed for speed). `--state` starts from a state saved by `--save-at N --save FILE`, the
//! machine's own `SPH2` (Sapphire_Native.md §38), so that a workload's later frames are measured without its boot.

use std::time::Instant;

use emusen_native::core::{Core, Insert};
use emusen_native::ffi::StateMachine;
use sapphire::machine::{Machine, Market};
use sapphire::media::{Media, Sectors};
use sapphire::vu::jit::Settings;

const BASIS: u64 = 0xcbf2_9ce4_8422_2325;

fn fnv(h: u64, b: &[u8]) -> u64 {
    b.iter().fold(h, |h, &x| (h ^ x as u64).wrapping_mul(0x0000_0100_0000_01B3))
}

fn ticks() -> u64 {
    #[cfg(target_arch = "x86_64")]
    // SAFETY: RDTSC has no preconditions.
    unsafe {
        core::arch::x86_64::_rdtsc()
    }
    #[cfg(not(target_arch = "x86_64"))]
    0
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| args.iter().position(|a| a == name).map(|i| args[i + 1].clone());
    let has = |name: &str| args.iter().any(|a| a == name);
    let usage = "s40 <boot|disc> <bios> <americas|europe|japan> [image] <frames> <interpreter|recompiler> [--digests FILE] [--clock] [--from N]";
    let workload = args.first().unwrap_or_else(|| panic!("{usage}")).as_str();
    let bios = std::fs::read(args.get(1).unwrap_or_else(|| panic!("{usage}"))).expect("the BIOS");
    let market = match args.get(2).map(String::as_str) {
        Some("europe") => Market::Europe,
        Some("japan") => Market::Japan,
        _ => Market::Americas,
    };
    let ports = [Some("ps2.dualshock2"), None];
    let (mut m, rest) = match workload {
        "boot" => {
            let mut m = Machine::new(Media::Empty, market, ports);
            m.insert_bios(&bios);
            (m, 3)
        }
        "disc" => {
            let image = std::fs::read(&args[3]).expect("the disc image");
            let Media::Disc(d) = Media::read(&image).expect("a disc") else { panic!("{} is not a disc", args[3]) };
            let name = d.program.rsplit(['\\', ':']).next().unwrap_or("").split(';').next().unwrap_or("").to_string();
            let program = Sectors::new(&image).root_file(&name).unwrap_or_else(|| panic!("the disc's root holds no {name}"));
            let mut m = Machine::new(Media::Empty, market, ports);
            m.insert_bios(&bios);
            m.media_lid(0, true).expect("the tray opens");
            m.media_insert(0, &Insert { image: Some(&image), files: Vec::new(), restore: false }).expect("the disc goes in");
            m.media_lid(0, false).expect("the tray closes");
            m.program = Some(program);
            (m, 4)
        }
        other => panic!("{other}: {usage}"),
    };
    let frames: usize = args[rest].parse().expect("a count of frames");
    let tier = args[rest + 1].as_str();
    match tier {
        "recompiler" => m.sys.board.vpu.set_tier(1, Some(Settings::from_environment())),
        "interpreter" => m.sys.board.vpu.set_tier(1, None),
        other => panic!("{other}: {usage}"),
    }
    let from: usize = flag("--from").map_or(0, |f| f.parse().expect("a frame"));
    let save_at: Option<usize> = flag("--save-at").map(|f| f.parse().expect("a frame"));
    if let Some(file) = flag("--state") {
        let data = std::fs::read(file).expect("the state");
        m.load_state(&data).expect("the state loads");
    }
    let clock = has("--clock");
    m.sys.board.vpu.micro1.set_clock(clock);
    let mut digests = String::new();
    let (mut picture, mut sound) = (BASIS, BASIS);
    let mut chunk = vec![0i16; 1 << 16];
    let mut times = Vec::with_capacity(frames);
    let mut state = vec![0u8; m.state_size()];
    let (mut frame_ticks, mut vu1_at_from, mut wall) = (0u64, [0u64; 3], 0.0);
    let mut counts_at_from = m.sys.board.vpu.jit_counts(1).unwrap_or_default();
    let started = Instant::now();
    for f in 0..frames {
        if f == from {
            vu1_at_from = m.sys.board.vpu.micro1.clock().unwrap_or_default();
            counts_at_from = m.sys.board.vpu.jit_counts(1).unwrap_or_default();
        }
        let t0 = Instant::now();
        let k0 = ticks();
        m.advance();
        let k1 = ticks();
        let t = t0.elapsed().as_secs_f64() * 1e3;
        times.push(t);
        if f >= from {
            wall += t;
            frame_ticks += k1 - k0;
        }
        let p = fnv(BASIS, &m.sys.board.pcrtc.picture.rgba);
        picture = fnv(picture, &p.to_le_bytes());
        let n = m.drain_audio(&mut chunk, usize::MAX);
        let s = chunk[..n].iter().fold(BASIS, |h, &x| fnv(h, &x.to_le_bytes()));
        sound = fnv(sound, &s.to_le_bytes());
        if flag("--digests").is_some() {
            if state.len() < m.state_size() {
                state.resize(m.state_size(), 0);
            }
            let k = m.save_state(&mut state).expect("the state saves");
            digests.push_str(&format!("{} {p:016X} {s:016X} {:016X}\n", f + 1, fnv(BASIS, &state[..k])));
        }
        if (f + 1) % 100 == 0 {
            let v = m.sys.board.vpu.micro1.clock().unwrap_or_default();
            eprintln!("frame {} at {:.0} s, VU1 pairs {} in all", f + 1, started.elapsed().as_secs_f64(), v[2]);
        }
        if save_at == Some(f + 1)
            && let Some(file) = flag("--save")
        {
            state.resize(m.state_size(), 0);
            let k = m.save_state(&mut state).expect("the state saves");
            std::fs::write(file, &state[..k]).expect("the state written");
        }
    }
    if let Some(file) = flag("--digests") {
        std::fs::write(file, digests).expect("the digests written");
    }
    state.resize(m.state_size(), 0);
    let k = m.save_state(&mut state).expect("the state saves");
    let last = fnv(BASIS, &state[..k]);
    let mut window = times[from.min(times.len())..].to_vec();
    let count = window.len().max(1) as f64;
    let mean = window.iter().sum::<f64>() / count;
    window.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let at = |q: f64| window.get(((window.len() as f64 * q) as usize).min(window.len().saturating_sub(1))).copied().unwrap_or(0.0);
    println!(
        "{workload} vu1={tier}\tframes {from}-{frames}\tmean {mean:.3}\tmedian {:.3}\tp95 {:.3}\tmax {:.3}\tfps {:.3}\tpicture {picture:016X}\tsound {sound:016X}\tstate {last:016X}",
        at(0.5),
        at(0.95),
        at(1.0),
        1e3 * count / wall.max(1e-9)
    );
    let c = m.sys.board.vpu.jit_counts(1).unwrap_or_default();
    println!(
        "vu1\tcompiled pairs {:.0} a frame\tinterpreted {:.0}\tcalls {:.0}\tprograms {}\tinvalidated {}\trefused {}\tcode {} bytes\tcompiling {:.1} ms",
        (c.compiled_pairs - counts_at_from.compiled_pairs) as f64 / count,
        (c.interpreted_pairs - counts_at_from.interpreted_pairs) as f64 / count,
        (c.calls - counts_at_from.calls) as f64 / count,
        c.programs,
        c.invalidated,
        c.refused,
        c.bytes,
        c.nanos as f64 / 1e6
    );
    if let Some(v) = m.sys.board.vpu.micro1.clock() {
        let d = [v[0] - vu1_at_from[0], v[1] - vu1_at_from[1], v[2] - vu1_at_from[2]];
        println!(
            "vu1 clock\t{:.0} pairs a frame\t{:.0} calls a frame\t{:.2}% of the frame's ticks\t{:.1} ticks a pair",
            d[2] as f64 / count,
            d[1] as f64 / count,
            100.0 * d[0] as f64 / frame_ticks.max(1) as f64,
            d[0] as f64 / d[2].max(1) as f64
        );
    }
    println!("machine\tentered at frame {:?}\tstopped {:?}", m.entered, m.stop);
}
