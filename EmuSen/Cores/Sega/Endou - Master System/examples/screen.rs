//! Runs an image with pad presses and shows what is on its screen (Endou_Native.md §10.8): the name table at $3800 as
//! text, for test programs whose font is ASCII from tile 0 at space, and the picture as a PPM.
//! `cargo run --release --example screen -- <image> <frames> [--system s] [--model m] [--region r] [--press F:BUTTON[:DUR]]...
//! [--text] [--regs] [--ppm <file>] [--rgba <file>] [--vdp <file>] [--ram <file>] [--wav <file>]`; buttons Up, Down, Left,
//! Right, 1, 2, Pause, Reset. `--vdp` writes the VDP's whole picture as RGBA, 256 wide, before a Game Gear's LCD. `--wav` writes the sound of every frame run as a 16-bit stereo WAV file at the output rate.

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
    let mut m = match flags("--system").pop() {
        Some(system) => Machine::create_as(&system, &request),
        None => Machine::create(&request),
    }
    .unwrap_or_else(|e| panic!("refused: {e}"));
    let mut sound = Vec::new();
    let mut buf = vec![0i16; 1 << 14];
    for f in 0..frames {
        let held = presses.iter().filter(|&&(at, _, d)| (at..at + d).contains(&f)).fold(0, |b, p| b | p.1);
        m.set_buttons(0, held, 0xFF).unwrap();
        Core::advance(&mut m, &mut 0).unwrap();
        let n = m.drain_audio(&mut buf, usize::MAX);
        sound.extend_from_slice(&buf[..n]);
    }
    if let Some(path) = flags("--wav").pop() {
        std::fs::write(path, wav(&sound, endou::sound::RATE)).expect("the WAV file");
    }
    if args.iter().any(|a| a == "--regs") {
        let v = &m.bus.vdp;
        println!("pc {:04X} registers {:02X?} vscroll {:02X} line {} cram {:02X?}", m.cpu.regs.pc, v.regs, v.vscroll, v.line, &v.cram[..m.cram_len()]);
    }
    if args.iter().any(|a| a == "--text") {
        let rows = m.bus.vdp.vram[0x3800..0x3800 + 0x600].chunks(64);
        for row in rows {
            let line: String = row.chunks(2).map(|e| e[0].wrapping_add(32)).map(|c| if (0x20..0x7F).contains(&c) { c as char } else { '.' }).collect();
            println!("{}", line.trim_end());
        }
    }
    let (width, height) = (m.width as usize, m.height as usize);
    let rgba = &m.frame()[..width * height * 4];
    if let Some(path) = flags("--ppm").pop() {
        let mut out = format!("P6\n{} {}\n255\n", width, height).into_bytes();
        out.extend(rgba.chunks(4).flat_map(|p| [p[0], p[1], p[2]]));
        std::fs::write(path, out).expect("the PPM");
    }
    if let Some(path) = flags("--vdp").pop() {
        let v = &m.bus.vdp;
        let colour = |e: u16| if v.gg_mode { beryl_sms_colours::gg_colour(e) } else { beryl_sms_colours::SMS_COLOURS[e as usize & 0x3F] };
        let out: Vec<u8> = v.picture().iter().flat_map(|&e| colour(e)).collect();
        std::fs::write(path, out).expect("the VDP's picture");
        eprintln!("the VDP's picture is 256x{}", v.height);
    }
    if let Some(path) = flags("--ram").pop() {
        std::fs::write(path, &m.bus.ram).expect("the RAM");
    }
    if let Some(path) = flags("--rgba").pop() {
        std::fs::write(path, rgba).expect("the picture");
    }
    eprintln!("{} frames, {}x{}, frame digest {:016X}", m.frames, width, height, rgba.iter().fold(0xCBF29CE484222325u64, |h, &b| (h ^ b as u64).wrapping_mul(0x100000001B3)));
}

/// 16-bit stereo samples as a WAV file.
fn wav(samples: &[i16], rate: u32) -> Vec<u8> {
    let data = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&[1, 0, 2, 0]);
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&(rate * 4).to_le_bytes());
    out.extend_from_slice(&[4, 0, 16, 0]);
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data.to_le_bytes());
    for s in samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}
