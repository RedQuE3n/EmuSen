//! How the goldens run the programs and discs (Diamond_Enhancements.md §6): the project's programs as each is made, ps1-tests'
//! release by path with its frames and market, and the bench discs with their press scripts. Shared by `programs.rs`'s
//! goldens and `renderer.rs`'s gate, so that the renderer is graded on the runs the goldens pin.
#![allow(dead_code)]

use diamond::exe;
use diamond::kernel::{self, Kernel};
use diamond::machine::{Config, Gpu, Machine};
use diamond::media::{Market, Media};

/// A disc of 40 mode 2 form 1 sectors from 00:02:00 whose user data names each one, sector 16 holding "CD001".
pub fn cd_disc() -> diamond::medium::Disc {
    let mut bin = Vec::new();
    for i in 0..40u32 {
        let mut user: Vec<u8> = (0..2048u32).map(|k| (k * 7 + i) as u8).collect();
        if i == 16 {
            user[..6].copy_from_slice(b"\x01CD001");
        }
        let mut s = [0u8; 2352];
        emusen_disc::ecma130::mode2_form1(150 + i, [0, 0, emusen_disc::ecma130::submode::DATA, 0], &user, &mut s);
        bin.extend_from_slice(&s);
    }
    diamond::medium::Disc::from_image(bin.into(), emusen_disc::IsoForm::Mode2Form1).unwrap()
}

/// A project program with no kernel, as `run_set_up` makes it.
pub fn bare(source: &str) -> Machine {
    let words = emusen_mips::assemble(source, 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).expect("assembles");
    let image = exe::build(&words);
    Machine::new(Media::read(&image).unwrap(), Config { market: Market::Americas, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None], &image, None)
}

/// One of the project's programs as the goldens run it: on the open kernel for the kernel's own, bare otherwise, a card
/// in slot 1 for the card programs and the stage 7 disc in the drive for the CD's.
pub fn golden_program(name: &str, source: &str) -> Machine {
    let mut m = if name.starts_with("kernel-") || name == "cd-kernel.s" {
        let words = emusen_mips::assemble(source, 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).expect("assembles");
        let image = exe::build(&words);
        Machine::new(Media::read(&image).unwrap(), Config { market: Market::Americas, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None], &image, kernel::open_kernel().map(|k| (k, Kernel::Open)))
    } else {
        bare(source)
    };
    if name.starts_with("card") || name == "kernel-card.s" {
        m.insert_card(0, None);
    }
    if name.starts_with("cd-") || name == "kernel-cd.s" {
        m.insert_disc(cd_disc());
    }
    m
}

pub const GOLDEN_PROGRAM_FRAMES: usize = 60;

/// The project's programs in `programs/`, each run as `golden_program` makes it.
pub fn own_programs() -> Vec<(String, String)> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("programs");
    let mut v: Vec<(String, String)> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".s"))
        .map(|n| (n.clone(), std::fs::read_to_string(dir.join(&n)).unwrap()))
        .collect();
    v.sort();
    v
}

/// The frames a ps1-tests program runs for, as WiseMan's gates run it, and its market: 900 for those graded by their
/// TTY, 1,500 for `gpu/bandwidth`, 300 for the rest; `timers` on a PAL console as its log was.
pub fn ps1_tests_run(path: &str) -> (usize, Market) {
    let tty = ["cpu/cop/", "cpu/code-in-io/", "dma/otc-test/", "dma/dpcr/", "cpu/access-time/", "dma/chain-looping/", "timers/", "gpu/gp0-e1/", "gpu/mask-bit/", "gpu/version-detect/", "spu/memory-transfer/", "spu/test/"];
    let frames = match path {
        "gpu/bandwidth/bandwidth.exe" => 1500,
        "mdec/movie/movie-15bit.exe" => 315,
        "mdec/movie/movie-24bit.exe" => 309,
        p if tty.iter().any(|t| p.starts_with(t)) => 900,
        _ => 300,
    };
    (frames, if path.starts_with("timers/") { Market::Europe } else { Market::Americas })
}

/// Every program of ps1-tests' release but its tools, by path.
pub fn ps1_tests_programs(corpus: &std::path::Path) -> Vec<String> {
    fn walk(root: &std::path::Path, dir: &std::path::Path, out: &mut Vec<String>) {
        for e in std::fs::read_dir(dir).unwrap().filter_map(|e| e.ok()) {
            let p = e.path();
            if p.is_dir() {
                walk(root, &p, out);
            } else if p.extension().is_some_and(|x| x == "exe") {
                let rel = p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
                if !rel.starts_with("tools/") {
                    out.push(rel);
                }
            }
        }
    }
    let mut v = Vec::new();
    walk(corpus, corpus, &mut v);
    v.sort();
    v
}

pub fn ps1_tests_machine(corpus: &std::path::Path, path: &str, market: Market) -> Machine {
    let image = std::fs::read(corpus.join(path)).expect("the release's program");
    Machine::new(Media::read(&image).unwrap(), Config { market, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None], &image, kernel::open_kernel().map(|k| (k, Kernel::Open)))
}

/// The eight bench discs of the goldens, by their files in the folder `EMUSEN_DIAMOND_DISCS` names, and the press
/// scripts of `EmuSen.WiseMan/Reference/analysis/psxplay/` they are played by.
pub const GOLDEN_DISCS: [(&str, &str); 8] = [
    ("Ape Escape (USA).cue", "ape-escape.txt"),
    ("Castlevania - Symphony of the Night (USA)/Castlevania - Symphony of the Night (USA).cue", "castlevania-sotn.txt"),
    ("Crash Bandicoot (USA).bin", "crash-bandicoot.txt"),
    ("Mega Man Legends (USA) (Track 1).bin", "mega-man-legends.txt"),
    ("Rayman (USA)/Rayman (USA).cue", "rayman.txt"),
    ("Silent Hill (USA)/Silent Hill (USA).cue", "silent-hill.txt"),
    ("Spyro the Dragon (USA).bin", "spyro-the-dragon.txt"),
    ("Tekken 3/Tekken 3.cue", "tekken-3.txt"),
];

pub const GOLDEN_DISC_FRAMES: i64 = 3000;

pub const GOLDEN_DISC_ANCHOR: i64 = 300;

/// A press script as `psxplay.py` reads it, its anchor the frame its second line gives for Diamond's open kernel: the
/// buttons as (first frame, the report's mask, frames) and the sticks as (first frame, axis, value, frames).
pub fn press_script(name: &str) -> (i64, Vec<(i64, u32, i64)>, Vec<(i64, u32, f64, i64)>) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../EmuSen.WiseMan/Reference/analysis/psxplay").join(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{}", path.display()));
    let anchor: i64 = text.lines().find_map(|l| l.strip_prefix("# T: frame ")).and_then(|r| r.split_whitespace().next()).and_then(|n| n.parse().ok()).expect("the anchor on Diamond's open kernel");
    let bits = |b: &str| match b {
        "Select" => 0x1, "L3" => 0x2, "R3" => 0x4, "Start" => 0x8, "Up" => 0x10, "Right" => 0x20, "Down" => 0x40, "Left" => 0x80,
        "L2" => 0x100, "R2" => 0x200, "L1" => 0x400, "R1" => 0x800, "Triangle" => 0x1000, "Circle" => 0x2000, "Cross" => 0x4000, "Square" => 0x8000,
        other => panic!("{name}: the button {other}"),
    };
    let (mut presses, mut sticks) = (Vec::new(), Vec::new());
    for line in text.lines() {
        let line = line.split('#').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split_whitespace().collect();
        let at = if let Some(n) = f[0].strip_prefix('+') { anchor + n.parse::<i64>().unwrap() } else { f[0][1..].parse().unwrap() };
        let n: i64 = f[2].parse().unwrap();
        let mut mask = 0;
        for b in f[1].split('+') {
            if let Some(s) = b.strip_prefix("stick:") {
                let (axis, value) = s.split_once(':').unwrap();
                sticks.push((at, axis.parse().unwrap(), value.parse().unwrap(), n));
            } else {
                mask |= bits(b);
            }
        }
        if mask != 0 {
            presses.push((at, mask, n));
        }
    }
    (anchor, presses, sticks)
}

/// A disc's maker: the image at `path`, its tracks beside it, on the open kernel with no firmware and a DualShock in
/// port 1, Americas.
pub fn disc_maker(path: std::path::PathBuf) -> impl Fn() -> Machine {
    let image = std::fs::read(&path).unwrap_or_else(|_| panic!("the disc {}", path.display()));
    let mut files = Vec::new();
    if diamond::medium::is_sheet(&image) {
        for name in emusen_disc::set::companions(&String::from_utf8_lossy(&image)).expect("a sheet") {
            files.push(std::fs::read(path.parent().unwrap().join(name)).expect("a track file"));
        }
    }
    move || {
        let Ok(diamond::medium::Opened::Disc(disc, media)) = diamond::medium::open(emusen_native::core::Kept::from(image.clone()), files.iter().map(|f| emusen_native::core::Kept::from(f.clone())).collect()) else { panic!("the disc opens") };
        let mut m = Machine::new(media, Config { market: Market::Americas, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None], &image, kernel::open_kernel().map(|k| (k, Kernel::Open)));
        m.insert_disc(*disc);
        m
    }
}

/// A press script's buttons and sticks put on port 1 for the machine's next frame, as the goldens play a disc.
pub fn press(m: &mut Machine, presses: &[(i64, u32, i64)], sticks: &[(i64, u32, f64, i64)]) {
    let mask = presses.iter().filter(|(at, _, n)| (*at..*at + *n).contains(&m.frames)).fold(0, |a, p| a | p.1);
    m.board.sio0.set_buttons(0, mask);
    for axis in 0..4 {
        if sticks.iter().any(|s| s.1 == axis) {
            let v = sticks.iter().rev().find(|(at, a, _, n)| *a == axis && (*at..*at + *n).contains(&m.frames)).map_or(0.0, |s| s.2);
            if let Some(c) = m.board.sio0.controller(0) {
                c.set_axis(axis, v);
            }
        }
    }
}
