// `run <entry's libretro host> <image|--loop-elf OUT> <frames> [key=value]...`: one machine through the Rust host, its
// image given by path, run for a number of frames, and what it said (EmuSen_LibretroHost.md §9).

use std::ffi::CString;
use std::path::Path;

use emusen_core_host::library::Library;
use emusen_core_host::machine::{Create, Machine};

/// A PlayStation 2 program of one instruction, a branch to itself at 0x00100000: an ELF32 for the EE, built here.
fn loop_elf() -> Vec<u8> {
    let mut e = vec![0u8; 0x80];
    e[..4].copy_from_slice(b"\x7fELF");
    e[4] = 1;
    e[5] = 1;
    e[6] = 1;
    let put16 = |e: &mut Vec<u8>, at: usize, v: u16| e[at..at + 2].copy_from_slice(&v.to_le_bytes());
    let put32 = |e: &mut Vec<u8>, at: usize, v: u32| e[at..at + 4].copy_from_slice(&v.to_le_bytes());
    put16(&mut e, 16, 2);
    put16(&mut e, 18, 8);
    put32(&mut e, 20, 1);
    put32(&mut e, 24, 0x0010_0000);
    put32(&mut e, 28, 0x34);
    put16(&mut e, 40, 0x34);
    put16(&mut e, 42, 0x20);
    put16(&mut e, 44, 1);
    put32(&mut e, 0x34, 1);
    put32(&mut e, 0x38, 0x80);
    put32(&mut e, 0x3C, 0x0010_0000);
    put32(&mut e, 0x40, 0x0010_0000);
    put32(&mut e, 0x44, 8);
    put32(&mut e, 0x48, 8);
    put32(&mut e, 0x4C, 7);
    put32(&mut e, 0x50, 0x1000);
    e.extend_from_slice(&0x1000_FFFFu32.to_le_bytes());
    e.extend_from_slice(&0u32.to_le_bytes());
    e
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("usage: run <entry's libretro host> <image|--loop-elf OUT> <frames> [key=value]...");
        std::process::exit(2);
    }
    let image_path = if args[2] == "--loop-elf" {
        let out = &args[3];
        std::fs::write(out, loop_elf()).unwrap();
        out.clone()
    } else {
        args[2].clone()
    };
    let rest = if args[2] == "--loop-elf" { &args[4..] } else { &args[3..] };
    let frames: u64 = rest.first().and_then(|f| f.parse().ok()).unwrap_or(60);
    let settings = rest.iter().skip(1).cloned().collect::<Vec<_>>().join("\n");
    let lib = Library::load(Path::new(&args[1])).unwrap_or_else(|e| panic!("{e:?}"));
    println!("info: {}", lib.info_text().unwrap_or_default());
    let image = std::fs::read(&image_path).unwrap();
    let path = CString::new(std::fs::canonicalize(&image_path).unwrap().to_string_lossy().as_bytes()).unwrap();
    let c = Create { settings: &settings, pixel_formats: 0b111, image_path: Some(&path), ..Create::new(&image) };
    let mut log = String::new();
    match Machine::create(lib, &c) {
        Err(r) => println!("refused: {} {}", r.status, r.words),
        Ok(mut m) => {
            let start = std::time::Instant::now();
            let mut audio = vec![0i16; 1 << 16];
            let mut samples = 0;
            for _ in 0..frames {
                if let Err(e) = m.advance() {
                    println!("advance: {e:?} {}", m.last_error());
                    break;
                }
                samples += m.audio_drain(&mut audio).map_or(0, |a| a.0);
            }
            let f = m.frame_info().ok();
            println!("frames {} in {:?}; picture {:?}; samples {samples}", m.frame_count(), start.elapsed(), f.map(|f| (f.width, f.height, f.format, f.serial)));
            println!("machine: {}", m.machine_info().unwrap_or_default());
            let _ = m.log_drain(&mut log);
        }
    }
    println!("log:\n{log}");
}
