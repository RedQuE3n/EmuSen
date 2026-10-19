//! Runs each image twice on its default console (Endou_Native.md §13): once through the core ABI's create, as a player
//! starts it, the open boot program first where the console has a BIOS; once with no BIOS, the cartridge from reset.
//! The booted run is aligned at its hand-over, and the two compared at frames after it: the picture's digest, and the
//! bytes of RAM that differ. Images are read in place and never written.
//! `cargo run --release --example bootcheck -- <frames,frames,...> <image>...`

use emusen_native::core::{Core, Create, Settings, sys};
use endou::boot::Boot;
use endou::machine::Machine;

fn digest(m: &Machine) -> u64 {
    m.frame_bytes().iter().fold(0xCBF2_9CE4_8422_2325, |h, &b| (h ^ b as u64).wrapping_mul(0x100_0000_01B3))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let at: Vec<usize> = args[1].split(',').map(|f| f.parse().expect("a frame")).collect();
    let last = *at.iter().max().unwrap();
    for path in &args[2..] {
        let image = std::fs::read(path).expect("the image");
        let name = std::path::Path::new(path).file_name().unwrap().to_string_lossy().into_owned();
        let request = Create { image: &image, settings: Settings::default(), files: Vec::new(), pixel_formats: 1, host_abi_version: sys::ABI_VERSION, shared_removable: false, shared_console: false, device: None };
        let mut booted = Machine::create(&request).unwrap_or_else(|e| panic!("{name}: refused {e}"));
        let model = booted.model();
        let rom = image[booted.media.skipped..].to_vec();
        let mut plain = Machine::with_board(booted.media.clone(), booted.market, model, booted.bus.cart.board, rom);
        let mut handed = 0usize;
        if booted.boot != Boot::None {
            while !(booted.bus.memory_control == 0xAB && booted.cpu.regs.pc < 0xC000) && handed < 600 {
                booted.advance();
                handed += 1;
            }
            if handed == 600 {
                println!("{name}\t{model:?}\trefused or waiting\t{}", text(&booted));
                continue;
            }
        }
        let mut out = format!("{name}\t{model:?}\thanded over in frame {}", handed.saturating_sub(1));
        for f in 1..=last {
            plain.advance();
            if f > 1 || booted.boot == Boot::None {
                booted.advance();
            }
            if at.contains(&f) {
                let ram = (0..plain.bus.ram.len()).filter(|&i| plain.bus.ram[i] != booted.bus.ram[i]).count();
                let same = digest(&plain) == digest(&booted);
                out += &format!("\t{f}: {} ram {ram}", if same { "same" } else { "DIFFERENT" });
            }
        }
        println!("{out}");
    }
}

/// The open boot program's words, its name table's rows 13 to 16.
fn text(m: &Machine) -> String {
    (13..17)
        .map(|r| m.bus.vdp.vram[0x3800 + r * 64..0x3800 + r * 64 + 64].chunks(2).map(|e| (e[0].wrapping_add(32)) as char).collect::<String>().trim().to_owned())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}
