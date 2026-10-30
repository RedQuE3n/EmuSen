//! Stage S32's bench (Sapphire_Native.md §43): one workload run headless on the player's BIOS for a fixed count of
//! frames, each frame's `advance` timed and nothing else, and the units' own counts over the frames measured.
//!
//! `cargo run --release --example bench -- <workload> <bios> <americas|europe|japan> [image] [frames] [--from N]
//! [--times FILE] [--every K DIR] [--registers] [--press F:BITS]... [--card] [--blocks] [--idle] [--gs-workers N] [--edram]
//! [--vu0] [--vu1] [--defaults] [--nvm FILE] [--state FILE] [--save-at N FILE] [--script FILE] [--window NAME] [--census FILE] [--working FILE]`
//!
//! - `boot`: the console started empty with no disc, the BIOS booting to OSDSYS; slot 1 empty unless `--card`.
//! - `elf <program.elf>`: an EE program entered by the fast boot (Sapphire_Native.md §36), a ps2autotests program.
//! - `disc <image>`: a disc put in the drive of a console started empty and the program its SYSTEM.CNF names entered by
//!   the fast boot (§40, §40.14): a synthetic disc of an IOP program, or a game's own program.
//! - `game <image>`: the disc in the drive at power-on and the BIOS booting it, as the game runner `s28.rs` does, with
//!   a DualShock 2 in each port, the console's erased card in slot 1 and the EEPROM `--nvm` names; `--state` starts from a state the runner or
//!   `--save-at` saved, and `--script` drives the pad by the runner's `press` and `stick` lines (`common/script.rs`).
//!
//! The frames and `--from` are the machine's own count, so that a run from a state times the same frames as one from
//! power-on: the run stops when the machine reaches the frames and times those after `--from`. `--window NAME` names one
//! of `WINDOWS`, a window of a game with its press script built in, and sets both (Sapphire_Speed.md §51).
//!
//! Prints one line in Diamond's bench form (the mean, median, 95th percentile and worst milliseconds a frame from frame
//! N, default 0), the frames a second and the wall time of that window, and the digests of every frame's picture, of the
//! sound and of the final state, each FNV-1a and taken outside the timed span; then the units' counts a frame over the
//! window: the EE's steps by mode and its cycles, the IOP's instructions, and each VU's pairs in micro mode. `--times`
//! writes each frame's milliseconds, `--every K DIR` the picture every K frames as `DIR/frame-<n>.ppm`, and
//! `--census FILE` writes each timed frame's counts as a tab-separated row (Sapphire_Redesign.md, R0 results, R0.1): the
//! frame, its milliseconds, the EE's instructions executed, those idle skipping passed and their cycles, and each VU's pairs.
//! `--working FILE`, built with the `census` feature, writes each timed frame's working set of compiled blocks: the
//! blocks entered, their guest instructions, their entries and steps, and the fewest blocks holding nine tenths of the
//! entries with their guest instructions (Sapphire_Redesign.md, R0 results).
//! `--registers` counts every register access the units answer over the window (S24's reports, which slow the run, so
//! its times are not the bench's) and prints the most frequent a frame, for finding which device a processor waits on.
//! `--press F:BITS` holds the pad's buttons (the core's bit numbers, `+`-joined) in port 1 for six frames from frame F.
//! `--blocks` runs the EE recompiled (S36, S37), with the settings the environment gives (`Blocks::from_env`), and prints
//! the dispatcher's counts; without it the EE is interpreted, the reference.
//! `--idle` turns idle skipping on (S33, Sapphire_Native.md §47); without it the waits are stepped, the reference.
//! `--gs-workers N` draws on N workers (S43, Sapphire_Native.md §50; `auto` for the automatic count, 1 the GS's own
//! thread) and prints their counts; `--vu1` runs VU1's microprograms compiled (S40, §45) and `--vu0` VU0's (Sapphire_Speed.md
//! §55), each with the settings the environment gives and left interpreted under `SAPPHIRE_VU1=interpreter` or
//! `SAPPHIRE_VU0=interpreter`; `--defaults` is the levers as a player has them, `--blocks --idle --gs-workers auto --vu0 --vu1`; `--edram` digests the GS's local memory after every frame, outside the timed span,
//! which joins the workers there, so its times are not the bench's.
//! The BIOS
//! image is handed to the machine and never read here; nothing printed holds an address, an instruction or memory.

use std::time::Instant;

use emusen_native::core::{Core, Kept};
use emusen_native::ffi::StateMachine;
use sapphire::machine::{Machine, Market};
use sapphire::media::{Media, Sectors};

#[path = "common/script.rs"]
mod script;

/// A window of a game: its name, the game's press script, the frame of the state it starts from (saved by the runner or
/// `--save-at` with that script from power-on), and its frames, timed after `from` up to `to` (Sapphire_Speed.md §51).
struct Window {
    name: &'static str,
    script: &'static str,
    state: i64,
    from: i64,
    to: i64,
}

const JAK: &str = include_str!("scripts/jak-and-daxter.txt");

/// Jak and Daxter's opening drawn in its engine, the nearest to play the core reaches (D-900): the village's dock at night,
/// Misty Island from the sea, and its arena, each timed after 60 frames that fill the recompilers' caches from a state.
const WINDOWS: [Window; 3] = [
    Window { name: "jak-dock", script: JAK, state: 6000, from: 6060, to: 6360 },
    Window { name: "jak-island", script: JAK, state: 7500, from: 7560, to: 7860 },
    Window { name: "jak-arena", script: JAK, state: 9000, from: 9060, to: 9360 },
];

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
    /// The recompiler's compiled and interpreted steps, and the EE's and the IOP's cycles and steps idle skipping passed.
    compiled: u64,
    interpreted: u64,
    idle: [u64; 4],
    /// Each VU's recompiler: pairs compiled and interpreted, microprograms compiled, words found changed, the budget's
    /// flushes, the bytes of code held, the compiler's nanoseconds, the microprograms kept and the most for one start.
    vu: [[u64; 11]; 2],
    /// The EE's compiler across its budget's drops: nanoseconds on its thread and on the dispatcher's, blocks compiled on each, and the drops.
    jit: [u64; 5],
}

fn counts(m: &Machine) -> Counts {
    let s = &m.sys;
    Counts {
        modes: s.ee.modes,
        ee_instructions: s.ee.timing.instructions,
        ee_cycles: s.board.now,
        iop_instructions: s.board.iop.instructions,
        pairs: s.board.vpu.pairs_run(),
        compiled: m.blocks.as_ref().map_or(0, |b| b.stats.compiled_steps),
        interpreted: m.blocks.as_ref().map_or(0, |b| b.stats.interpreted),
        idle: { let c = &m.idle.counts; [c.ee_cycles, c.ee_steps, c.iop_cycles, c.iop_instructions] },
        vu: [0, 1].map(|u| s.board.vpu.jit_counts(u).map_or([0; 11], |c| [c.compiled_pairs, c.interpreted_pairs, c.programs, c.invalidated, c.flushes, c.bytes, c.nanos, c.kept, c.widest, c.calls, c.dead_flags])),
        jit: m.blocks.as_ref().map_or([0; 5], |b| {
            let ([t, h], _) = b.spent();
            [t.nanos, h.nanos, t.compiled, h.compiled, b.flushes()]
        }),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| args.iter().position(|a| a == name).map(|i| args[i + 1].clone());
    let plain: Vec<&String> = {
        let mut v = Vec::new();
        let mut i = 0;
        while i < args.len() {
            if matches!(args[i].as_str(), "--from" | "--times" | "--press" | "--gs-workers" | "--nvm" | "--state" | "--script" | "--window" | "--census" | "--working") {
                i += 2;
                continue;
            }
            if matches!(args[i].as_str(), "--every" | "--save-at") {
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
    let usage = "bench <boot|elf|disc|game> <bios> <americas|europe|japan> [image] [frames] [--from N] [--times FILE] [--every K DIR] [--registers] [--press F:BITS]... [--card] [--blocks] [--idle] [--gs-workers N] [--edram] [--vu0] [--vu1] [--defaults] [--nvm FILE] [--state FILE] [--save-at N FILE] [--script FILE] [--window NAME] [--census FILE] [--working FILE]";
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
        "game" => {
            let path = plain.get(3).unwrap_or_else(|| panic!("{usage}"));
            let name = std::path::Path::new(path.as_str()).file_name().unwrap().to_string_lossy().into_owned();
            // A DualShock 2 in each port, as a host's create plugs them, so that the runner's states are this machine's.
            let mut m = Machine::new(Media::Empty, market, [Some("ps2.dualshock2"), Some("ps2.dualshock2")]);
            m.insert_bios(&bios);
            if let Some(f) = flag("--nvm") {
                m.nvm_mut().bytes.copy_from_slice(&std::fs::read(f).expect("the EEPROM"));
            }
            m.disc_at_power_on(sapphire::cdvd::open(Kept::from(std::fs::read(path).expect("the disc")), Vec::new()).expect("the disc opens"));
            (m, name, 4)
        }
        "elf" | "disc" => {
            let path = plain.get(3).unwrap_or_else(|| panic!("{usage}"));
            let name = std::path::Path::new(path.as_str()).file_name().unwrap().to_string_lossy().into_owned();
            (machine(workload, path, market, &bios), name, 4)
        }
        other => panic!("{other}: {usage}"),
    };
    if workload != "game" && !args.iter().any(|a| a == "--card") {
        m.sys.board.iop.bus.sio2.slots[0].cards[0] = None;
    }
    let defaults = args.iter().any(|a| a == "--defaults");
    if defaults || args.iter().any(|a| a == "--blocks") {
        m.blocks = Some(Box::new(sapphire::blocks::Blocks::from_env()));
    }
    for (u, flag, key) in [(0, "--vu0", "SAPPHIRE_VU0"), (1, "--vu1", "SAPPHIRE_VU1")] {
        if (defaults || args.iter().any(|a| a == flag)) && std::env::var(key).ok().as_deref() != Some("interpreter") {
            m.sys.board.vpu.set_tier(u, Some(sapphire::vu::jit::Settings::from_environment()));
        }
    }
    m.idle.on = defaults || args.iter().any(|a| a == "--idle");
    if let Some(n) = flag("--gs-workers").or(defaults.then(|| "auto".to_string())) {
        m.sys.board.gs.set_workers(sapphire::gs::workers::parse(&n).expect("a count of workers or auto"));
    }
    if let Some(f) = flag("--state") {
        m.load_state(&std::fs::read(f).expect("the state")).expect("the state loads");
        eprintln!("loaded at frame {}", m.frames);
    }
    let window = flag("--window").map(|n| WINDOWS.iter().find(|w| w.name == n).unwrap_or_else(|| panic!("{n}: the windows are {:?}", WINDOWS.iter().map(|w| w.name).collect::<Vec<_>>())));
    let mut save_at = args.iter().position(|a| a == "--save-at").map(|i| (args[i + 1].parse::<i64>().expect("a frame"), args[i + 2].clone()));
    let edram_each = args.iter().any(|a| a == "--edram");
    let mut edram = BASIS;
    let frames: i64 = window.map(|w| w.to).or_else(|| plain.get(rest).map(|f| f.parse().expect("a count of frames"))).unwrap_or(600);
    let from: i64 = window.map(|w| w.from).or_else(|| flag("--from").map(|f| f.parse().expect("a frame"))).unwrap_or(0);
    let every = args.iter().position(|a| a == "--every").map(|i| (args[i + 1].parse::<usize>().expect("a count of frames"), args[i + 2].clone()));
    if let Some((_, dir)) = &every {
        std::fs::create_dir_all(dir).expect("the pictures' folder");
    }
    let registers = args.iter().any(|a| a == "--registers");
    let mut lines = window.map_or(String::new(), |w| w.script.to_owned());
    if let Some(f) = flag("--script") {
        lines += &std::fs::read_to_string(f).expect("the script");
    }
    for w in args.windows(2).filter(|w| w[0] == "--press") {
        let (f, bits) = w[1].split_once(':').expect("--press F:BITS");
        lines += &format!("\npress {f} {bits}\n");
    }
    let input = script::Script::parse(&lines);
    let (mut picture, mut sound) = (BASIS, BASIS);
    let mut chunk = vec![0i16; 1 << 16];
    let mut times = Vec::with_capacity(frames.max(0) as usize);
    let mut at_from = counts(&m);
    let mut wall = 0.0;
    let mut timed = 0usize;
    let started = Instant::now();
    let first = m.frames;
    #[cfg(feature = "census")]
    let mut working = flag("--working").map(|f| {
        let mut file = std::fs::File::create(f).expect("the working-set file");
        use std::io::Write;
        writeln!(file, "frame\tblocks\twords\tentries\tsteps\thot_blocks\thot_words").expect("written");
        file
    });
    let mut census = flag("--census").map(|f| (std::fs::File::create(f).expect("the census file"), counts(&m)));
    if let Some((file, _)) = &mut census {
        use std::io::Write;
        writeln!(file, "frame\tms\tee_executed\tee_passed\tee_passed_cycles\tvu0_pairs\tvu1_pairs").expect("written");
    }
    if let Some(w) = window {
        eprintln!("{}: from the state at frame {}, timed after frame {} to {}", w.name, w.state, w.from, w.to);
    }
    while m.frames < frames {
        let f = m.frames;
        if f == from.max(first) {
            at_from = counts(&m);
            if registers {
                m.sys.board.reports.registers.clear();
                m.sys.board.iop.bus.reports.registers.clear();
                m.sys.board.reports.all = true;
                m.sys.board.iop.bus.reports.all = true;
            }
        }
        Core::set_buttons(&mut m, 0, input.buttons(f + 1), 0xFFFF).expect("port 1");
        if input.has_sticks() {
            for axis in 0..4 {
                Core::set_axis(&mut m, 0, axis, input.axis(f + 1, axis)).expect("port 1");
            }
        }
        let start = Instant::now();
        m.advance();
        let t = start.elapsed().as_secs_f64() * 1e3;
        times.push(t);
        if f >= from {
            wall += t;
            timed += 1;
        }
        #[cfg(feature = "census")]
        if let (Some(file), Some(b)) = (&mut working, m.blocks.as_deref_mut()) {
            let set = std::mem::take(&mut b.census);
            if f >= from {
                use std::io::Write;
                let mut v: Vec<(u64, u64, u64)> = set.into_values().collect();
                v.sort_unstable_by_key(|x| std::cmp::Reverse(x.0));
                let entries: u64 = v.iter().map(|x| x.0).sum();
                let (mut run, mut hot, mut hot_words) = (0, 0, 0);
                for x in &v {
                    if 10 * run >= 9 * entries {
                        break;
                    }
                    run += x.0;
                    hot += 1;
                    hot_words += x.2;
                }
                let row = [v.len() as u64, v.iter().map(|x| x.2).sum(), entries, v.iter().map(|x| x.1).sum(), hot, hot_words];
                writeln!(file, "{}\t{}", m.frames, row.map(|x| x.to_string()).join("\t")).expect("written");
            }
        }
        if let Some((file, last)) = &mut census {
            use std::io::Write;
            let now = counts(&m);
            if f >= from {
                let all = now.ee_instructions - last.ee_instructions;
                let passed = now.idle[1] - last.idle[1];
                let row = [all - passed, passed, now.idle[0] - last.idle[0], now.pairs[0] - last.pairs[0], now.pairs[1] - last.pairs[1]];
                writeln!(file, "{}\t{t:.3}\t{}", m.frames, row.map(|v| v.to_string()).join("\t")).expect("written");
            }
            *last = now;
        }
        if let Some((at, file)) = &save_at
            && m.frames >= *at
        {
            // A queue past its slots cannot be saved at this instant (Sapphire_Native.md §38.2); the next frame is tried.
            let mut s = vec![0; m.state_size()];
            if let Ok(n) = m.save_state(&mut s) {
                std::fs::write(file, &s[..n]).expect("the state written");
                eprintln!("the state saved at frame {}", m.frames);
                save_at = None;
            }
        }
        picture = fnv(picture, &m.sys.board.pcrtc.picture.rgba);
        if edram_each {
            edram = m.sys.board.gs.local().words().iter().fold(edram, |h, &w| (h ^ u64::from(w)).wrapping_mul(0x0000_0100_0000_01B3));
        }
        let n = m.drain_audio(&mut chunk, usize::MAX);
        sound = chunk[..n].iter().fold(sound, |h, &s| fnv(h, &s.to_le_bytes()));
        if let Some((k, dir)) = &every
            && m.frames % *k as i64 == 0
        {
            write_ppm(&m, &std::path::Path::new(dir).join(format!("frame-{}.ppm", m.frames)));
        }
        if m.frames % 100 == 0 {
            eprintln!("frame {} at {:.0} s", m.frames, started.elapsed().as_secs_f64());
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
    let mut span = times[times.len() - timed..].to_vec();
    let k = span.len().max(1) as f64;
    let mean = span.iter().sum::<f64>() / k;
    span.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let at = |q: f64| span.get(((span.len() as f64 * q) as usize).min(span.len().saturating_sub(1))).copied().unwrap_or(0.0);
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
    if m.blocks.is_some() {
        let (c, i) = (d(at_from.compiled, end.compiled), d(at_from.interpreted, end.interpreted));
        println!("blocks\t{c:.0} compiled steps a frame\t{i:.0} interpreted\t{:.1}% compiled", 100.0 * c / (c + i).max(1.0));
        let j = |n: usize| end.jit[n] - at_from.jit[n];
        let b = m.blocks.as_deref().expect("blocks");
        println!(
            "ee recompiler\t{} flushes at its code budget of {} MiB ({} of them for a full code memory)\t{:.1} MiB of code held at the end\tits thread busy {:.1}% of the window, {:.1} blocks a frame\ton the dispatcher's thread {:.3} ms and {:.1} blocks a frame",
            j(4),
            b.budget() >> 20,
            b.full_flushes(),
            b.spent().1 as f64 / 1048576.0,
            100.0 * j(0) as f64 / (wall * 1e6).max(1.0),
            j(2) as f64 / k,
            j(1) as f64 / 1e6 / k,
            j(3) as f64 / k
        );
    }
    if m.idle.on {
        let g = |n: usize| d(at_from.idle[n], end.idle[n]);
        println!("idle\tEE {:.0} cycles ({:.1}% of its cycles) and {:.0} steps passed a frame\tIOP {:.0} cycles and {:.0} instructions passed a frame", g(0), 100.0 * g(0) / cycles.max(1.0), g(1), g(2), g(3));
    }
    for u in (0..2).filter(|&u| m.sys.board.vpu.jit_counts(u).is_some()) {
        let g = |n: usize| d(at_from.vu[u][n], end.vu[u][n]);
        println!("vu{u} recompiler\t{:.0} pairs compiled and {:.0} interpreted a frame in {:.0} calls, {:.0} dead flag writes left out\t{:.2} microprograms compiled and {:.2} found changed a frame\t{} flushes at its code budget\t{:.1} MiB of code and {} microprograms held at the end, at most {} for a start", g(0), g(1), g(9), g(10), g(2), g(3), end.vu[u][4] - at_from.vu[u][4], end.vu[u][5] as f64 / 1048576.0, end.vu[u][7], end.vu[u][8]);
    }
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
        let busy: Vec<String> = (0..n).map(|w| format!("{:.1}/{:.1}", c.busy[w] as f64 / 1e6 / times.len() as f64, c.stalled[w] as f64 / 1e6 / times.len() as f64)).collect();
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
    #[cfg(feature = "forms")]
    print!("{}", sapphire::ee::forms_report());
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
