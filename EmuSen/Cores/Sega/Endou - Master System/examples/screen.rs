//! Runs an image with pad presses and shows what is on its screen (Endou_Native.md §10.8): the name table at $3800 as
//! text, for test programs whose font is ASCII from tile 0 at space, and the picture as a PPM.
//! `cargo run --release --example screen -- <image> <frames> [--model m] [--region r] [--press F:BUTTON[:DUR]]...
//! [--text] [--regs] [--ppm <file>] [--rgba <file>] [--ram <file>]`; buttons Up, Down, Left, Right, 1, 2, Pause, Reset.

use emusen_native::core::{Core, Create, Settings, sys};
use endou::machine::{Machine, WIDTH};

fn bit(name: &str) -> u32 {
    let names = ["Up", "Down", "Left", "Right", "1", "2", "Pause", "Reset"];
    1 << names.iter().position(|n| *n == name).unwrap_or_else(|| panic!("no button {name}"))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let image = std::fs::read(&args[1]).expect("the image");
    let frames: u32 = args[2].parse().expect("a frame count");
    let flags = |name: &str| args.iter().enumerate().filter(|(_, a)| *a == name).filter_map(|(i, _)| args.get(i + 1).cloned()).collect::<Vec<_>>();
    let mut settings = Vec::new();
    for (key, flag) in [("model", "--model"), ("region", "--region")] {
        if let Some(v) = flags(flag).pop() {
            settings.push((key.to_owned(), v));
        }
    }
    let presses: Vec<(u32, u32, u32)> = flags("--press")
        .iter()
        .map(|p| {
            let f: Vec<&str> = p.split(':').collect();
            (f[0].parse().unwrap(), bit(f[1]), f.get(2).map_or(4, |d| d.parse().unwrap()))
        })
        .collect();
    let request = Create { image: &image, settings: Settings::from_pairs(settings), files: Vec::new(), pixel_formats: 1, host_abi_version: sys::ABI_VERSION, shared_removable: false, shared_console: false, device: None };
    let mut m = Machine::create(&request).unwrap_or_else(|e| panic!("refused: {e}"));
    for f in 0..frames {
        let held = presses.iter().filter(|&&(at, _, d)| (at..at + d).contains(&f)).fold(0, |b, p| b | p.1);
        m.set_buttons(0, held, 0xFF).unwrap();
        Core::advance(&mut m, &mut 0).unwrap();
    }
    if args.iter().any(|a| a == "--regs") {
        let v = &m.bus.vdp;
        println!("registers {:02X?} vscroll {:02X} line {} cram {:02X?}", v.regs, v.vscroll, v.line, &v.cram[..32]);
    }
    if args.iter().any(|a| a == "--text") {
        let rows = m.bus.vdp.vram[0x3800..0x3800 + 0x600].chunks(64);
        for row in rows {
            let line: String = row.chunks(2).map(|e| e[0].wrapping_add(32)).map(|c| if (0x20..0x7F).contains(&c) { c as char } else { '.' }).collect();
            println!("{}", line.trim_end());
        }
    }
    let height = m.height as usize;
    let rgba = &m.frame()[..WIDTH as usize * height * 4];
    if let Some(path) = flags("--ppm").pop() {
        let mut out = format!("P6\n{} {}\n255\n", WIDTH, height).into_bytes();
        out.extend(rgba.chunks(4).flat_map(|p| [p[0], p[1], p[2]]));
        std::fs::write(path, out).expect("the PPM");
    }
    if let Some(path) = flags("--ram").pop() {
        std::fs::write(path, &m.bus.ram).expect("the RAM");
    }
    if let Some(path) = flags("--rgba").pop() {
        std::fs::write(path, rgba).expect("the picture");
    }
    eprintln!("{} frames, {}x{}, frame digest {:016X}", m.frames, WIDTH, height, rgba.iter().fold(0xCBF29CE484222325u64, |h, &b| (h ^ b as u64).wrapping_mul(0x100000001B3)));
}
