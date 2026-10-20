//! Runs an image with pad presses and prints a line a frame: whether the picture changed from the frame before, the VDP's
//! X and Y scroll registers and what the frame cost, so that a game's own cadence can be held against a reference's
//! (Endou_Pacing.md §3.4). `cargo run --release --example cadence -- <image> <frames> [--press F:BUTTON[:DUR]]...`;
//! buttons Up, Down, Left, Right, 1, 2, Pause, Reset.

use std::time::Instant;

use emusen_native::core::{Core, Create, Settings, sys};
use endou::machine::Machine;

fn bit(name: &str) -> u32 {
    let names = ["Up", "Down", "Left", "Right", "1", "2", "Pause", "Reset"];
    1 << names.iter().position(|n| *n == name).unwrap_or_else(|| panic!("no button {name}"))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let image = std::fs::read(&args[1]).expect("the image");
    let frames: u32 = args[2].parse().expect("a frame count");
    let presses: Vec<(u32, u32, u32)> = args
        .iter()
        .enumerate()
        .filter(|(_, a)| *a == "--press")
        .filter_map(|(i, _)| args.get(i + 1))
        .map(|p| {
            let f: Vec<&str> = p.split(':').collect();
            (f[0].parse().unwrap(), bit(f[1]), f.get(2).map_or(4, |d| d.parse().unwrap()))
        })
        .collect();
    let request = Create { image: &image, settings: Settings::from_pairs(Vec::new()), files: Vec::new(), pixel_formats: 1, host_abi_version: sys::ABI_VERSION, shared_removable: false, shared_console: false, device: None };
    let mut m = Machine::create(&request).unwrap_or_else(|e| panic!("refused: {e}"));
    let mut sound = vec![0i16; 1 << 14];
    let mut last = 0u64;
    println!("frame changed x_scroll y_scroll us");
    for f in 0..frames {
        let held = presses.iter().filter(|&&(at, _, d)| (at..at + d).contains(&f)).fold(0, |b, p| b | p.1);
        m.set_buttons(0, held, 0xFF).unwrap();
        let start = Instant::now();
        Core::advance(&mut m, &mut 0).unwrap();
        let us = start.elapsed().as_secs_f64() * 1e6;
        m.drain_audio(&mut sound, usize::MAX);
        let digest = m.frame_bytes().iter().fold(0xCBF29CE484222325u64, |h, &b| (h ^ b as u64).wrapping_mul(0x100000001B3));
        let regs = &m.bus.vdp.regs;
        println!("{f} {} {:02X} {:02X} {us:.1}", (digest != last) as u8, regs[8], regs[9]);
        last = digest;
    }
}
