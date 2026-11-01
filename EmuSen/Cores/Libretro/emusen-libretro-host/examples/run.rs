// `run <entry's libretro host> <image|--loop-elf OUT> <frames> [key=value]...`: one machine through the Rust host, its
// image given by path, run for a number of frames, and what it said (EmuSen_LibretroHost.md §9). With EMUSEN_RUN_RING=1
// its pictures go on a Serenity ring; EMUSEN_RUN_PNG=DIR:F,F,... keeps the ring's picture at those frames (§10f).

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

/// A Serenity ring of the machine's largest picture, lent to it, with the device it lives on; none unless asked for.
fn ring_for(m: &mut Machine) -> Option<(emusen_serenity::vulkan::Device, emusen_serenity::ring::Ring)> {
    std::env::var("EMUSEN_RUN_RING").ok().filter(|v| v == "1")?;
    let info = m.machine_info().unwrap_or_default();
    let num = |key: &str| info.split(&format!("\"{key}\":")).nth(1).and_then(|r| r.split(|c: char| !c.is_ascii_digit()).next()?.parse::<u32>().ok()).unwrap_or(0);
    let (w, h) = (num("max_width").max(64), num("max_height").max(64));
    let gpu = emusen_serenity::vulkan::Device::create(std::env::var("EMUSEN_GPU_DEVICE").ok().as_deref(), false).unwrap();
    let ring = gpu.create_ring(w, h, 3).unwrap();
    let images: Vec<emusen_native::core::sys::RingImage> = ring.images.iter().map(|i| emusen_native::core::sys::RingImage { size: 40, memory_type: i.memory_type, handle: i.handle as u64, allocation_size: i.allocation_size, width: i.width, height: i.height, format: i.format, usage: i.usage }).collect();
    let lend = emusen_native::core::sys::RingLend { size: size_of::<emusen_native::core::sys::RingLend>() as u32, count: 3, images: images.as_ptr(), image_size: 40, ready: ring.ready_fd as u64, done: ring.done_fd as u64, device_uuid: ring.device_uuid, driver_uuid: ring.driver_uuid };
    match m.ring_lend(Some(&lend)) {
        Ok(()) => println!("ring: {w}x{h} lent"),
        Err(e) => println!("ring: refused {e}"),
    }
    Some((gpu, ring))
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
    if std::env::var_os("EMUSEN_RUN_SCHEMA").is_some() {
        println!("schema: {}", lib.settings_text().unwrap_or_default());
    }
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
            let race = std::env::var_os("EMUSEN_RUN_RACE").is_some();
            let (mut first, mut second) = (Vec::new(), Vec::new());
            let mut size = 0;
            let mut moved = 0;
            let timed_from: u64 = std::env::var("EMUSEN_RUN_TIMED_FROM").ok().and_then(|v| v.parse().ok()).unwrap_or(u64::MAX);
            let mut timed = None;
            let mut ring = ring_for(&mut m);
            let png = std::env::var("EMUSEN_RUN_PNG").ok();
            let (png_dir, png_frames): (String, Vec<u64>) = png.as_deref().and_then(|p| p.split_once(':')).map_or((String::new(), Vec::new()), |(d, f)| (d.to_owned(), f.split(',').filter_map(|x| x.parse().ok()).collect()));
            let (mut emulation, mut in_ring) = (0f64, 0u64);
            for n in 0..frames {
                if n == timed_from {
                    timed = Some(std::time::Instant::now());
                }
                let t = std::time::Instant::now();
                if let Err(e) = m.advance() {
                    println!("advance: {e:?} {}", m.last_error());
                    break;
                }
                let shown = if ring.is_some() { m.frame_ring().ok().flatten() } else { None };
                if n >= timed_from {
                    emulation += t.elapsed().as_secs_f64();
                }
                if let (Some((_, r)), Some(f)) = (ring.as_ref(), shown) {
                    in_ring += 1;
                    let rgba = r.read(f.slot as usize, f.value, f.width as u32, f.height as u32).unwrap();
                    if png_frames.contains(&n) {
                        let path = format!("{png_dir}/frame-{n}-{}x{}.png", f.width, f.height);
                        emusen_crystal::png::write(&path, &rgba, f.width as u32, f.height as u32).unwrap();
                        println!("png: {path} fields {} aspect {}:{}", f.fields, f.aspect_num, f.aspect_den);
                    }
                }
                if ring.is_some() && m.events().map_or(false, |e| e.iter().any(|e| e.kind == emusen_native::core::sys::event::MACHINE_INFO)) {
                    ring = ring_for(&mut m);
                }
                samples += m.audio_drain(&mut audio).map_or(0, |a| a.0);
                if race && size == 0 {
                    size = m.space_size(0).unwrap_or(0).min(1 << 20) as usize;
                    (first, second) = (vec![0u8; size], vec![0u8; size]);
                }
                if race && size > 0 {
                    let _ = m.space_read(0, 0, &mut first);
                    std::thread::sleep(std::time::Duration::from_micros(500));
                    let _ = m.space_read(0, 0, &mut second);
                    moved += (first != second) as u32;
                }
            }
            if let Some(t) = timed {
                let k = frames - timed_from;
                println!("timed: frames {timed_from} to {frames}, {k} frames in {:?}, {:.3} ms a frame, {:.3} ms of it on the emulation thread; {in_ring} pictures in the ring", t.elapsed(), t.elapsed().as_secs_f64() * 1000.0 / k as f64, emulation * 1000.0 / k as f64);
            }
            if race {
                println!("race: {moved} of {frames} frames saw the first {size} bytes of space 0 move between two reads");
            }
            let f = m.frame_info().ok();
            println!("frames {} in {:?}; picture {:?}; samples {samples}", m.frame_count(), start.elapsed(), f.map(|f| (f.width, f.height, f.format, f.serial)));
            println!("machine: {}", m.machine_info().unwrap_or_default());
            let _ = m.log_drain(&mut log);
        }
    }
    println!("log:\n{log}");
}
