// `run <entry's libretro host> <image|--loop-elf OUT|--exit-elf OUT|--data-disc OUT> <frames> [key=value]...`: one machine through the Rust host, its
// image given by path, run for a number of frames, and what it said (EmuSen_LibretroHost.md §9). With EMUSEN_RUN_RING=1
// its pictures go on a Serenity ring; EMUSEN_RUN_PNG=DIR:F,F,... keeps the ring's picture at those frames (§10f).
// EMUSEN_RUN_PADS=FILE presses port 0's buttons by a script; EMUSEN_RUN_FILES=N=PATH,... passes battery files and
// writes them back at the end (§10j).

use std::ffi::CString;
use std::path::Path;

use emusen_core_host::library::Library;
use emusen_core_host::machine::{Create, Machine};

/// A PlayStation 2 program at 0x00100000, an ELF32 for the EE, built here: `--loop-elf` one instruction, a branch to
/// itself; `--exit-elf` the kernel's `Exit` (syscall 4), which hands the console back to its firmware's menu, then the loop.
fn ee_elf(code: &[u32]) -> Vec<u8> {
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
    put32(&mut e, 0x44, 4 * code.len() as u32);
    put32(&mut e, 0x48, 4 * code.len() as u32);
    put32(&mut e, 0x4C, 7);
    put32(&mut e, 0x50, 0x1000);
    for w in code {
        e.extend_from_slice(&w.to_le_bytes());
    }
    e
}

const LOOP: [u32; 2] = [0x1000_FFFF, 0];
const EXIT: [u32; 4] = [0x2403_0004, 0x0000_000C, 0x1000_FFFF, 0];

/// A data disc the console does not boot: ISO 9660 (ECMA-119) with one file, `README.TXT`, and no `SYSTEM.CNF`, so a
/// PlayStation 2's firmware stays at its browser with it in the tray. Built here.
fn data_disc() -> Vec<u8> {
    const S: usize = 2048;
    let text = b"An EmuSen data disc, for the firmware's browser.\r\n";
    let mut d = vec![0u8; 32 * S];
    let both32 = |d: &mut Vec<u8>, at: usize, v: u32| {
        d[at..at + 4].copy_from_slice(&v.to_le_bytes());
        d[at + 4..at + 8].copy_from_slice(&v.to_be_bytes());
    };
    let both16 = |d: &mut Vec<u8>, at: usize, v: u16| {
        d[at..at + 2].copy_from_slice(&v.to_le_bytes());
        d[at + 2..at + 4].copy_from_slice(&v.to_be_bytes());
    };
    let record = |d: &mut Vec<u8>, at: usize, lba: u32, len: u32, dir: bool, name: &[u8]| -> usize {
        let n = 33 + name.len() + (name.len() + 1) % 2;
        d[at] = n as u8;
        both32(d, at + 2, lba);
        both32(d, at + 10, len);
        d[at + 18..at + 25].copy_from_slice(&[126, 11, 1, 0, 0, 0, 0]);
        d[at + 25] = if dir { 2 } else { 0 };
        both16(d, at + 28, 1);
        d[at + 32] = name.len() as u8;
        d[at + 33..at + 33 + name.len()].copy_from_slice(name);
        n
    };
    let pvd = 16 * S;
    d[pvd] = 1;
    d[pvd + 1..pvd + 6].copy_from_slice(b"CD001");
    d[pvd + 6] = 1;
    d[pvd + 8..pvd + 72].fill(b' ');
    d[pvd + 40..pvd + 46].copy_from_slice(b"EMUSEN");
    both32(&mut d, pvd + 80, 32);
    both16(&mut d, pvd + 120, 1);
    both16(&mut d, pvd + 124, 1);
    both16(&mut d, pvd + 128, S as u16);
    both32(&mut d, pvd + 132, 10);
    d[pvd + 140..pvd + 144].copy_from_slice(&18u32.to_le_bytes());
    d[pvd + 148..pvd + 152].copy_from_slice(&19u32.to_be_bytes());
    record(&mut d, pvd + 156, 20, S as u32, true, &[0]);
    d[pvd + 190..pvd + 813].fill(b' ');
    for at in [813, 830, 847, 864] {
        d[pvd + at..pvd + at + 16].fill(b'0');
    }
    d[pvd + 881] = 1;
    d[17 * S] = 255;
    d[17 * S + 1..17 * S + 6].copy_from_slice(b"CD001");
    d[17 * S + 6] = 1;
    for (sector, lba) in [(18, 20u32.to_le_bytes()), (19, 20u32.to_be_bytes())] {
        let at = sector * S;
        d[at] = 1;
        d[at + 2..at + 6].copy_from_slice(&lba);
        d[at + 6] = if sector == 18 { 1 } else { 0 };
        d[at + 7] = if sector == 18 { 0 } else { 1 };
    }
    let mut at = 20 * S;
    at += record(&mut d, at, 20, S as u32, true, &[0]);
    at += record(&mut d, at, 20, S as u32, true, &[1]);
    record(&mut d, at, 21, text.len() as u32, false, b"README.TXT;1");
    d[21 * S..21 * S + text.len()].copy_from_slice(text);
    d
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

/// RetroPad's buttons by name, each its bit in the host's mask.
const BUTTONS: [&str; 16] = ["b", "y", "select", "start", "up", "down", "left", "right", "a", "x", "l", "r", "l2", "r2", "l3", "r3"];

/// The script's words for the tray, opened or closed at a frame instead of a button pressed.
const LID: u32 = 1 << 16;
const LID_OPEN: u32 = 1 << 17;

/// A pad script: each line `FRAME BUTTON[+BUTTON...] [FRAMES]`, the buttons held from that frame for FRAMES (6 unless
/// given), or `FRAME open` or `FRAME close`, the tray's lid; `#` begins a comment. Returned as (first frame, last
/// frame, mask).
fn pad_script(text: &str) -> Vec<(u64, u64, u32)> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let words: Vec<&str> = line.split_whitespace().collect();
        let bad = format!("pad script line {}: `{line}` is not FRAME BUTTON[+BUTTON...] [FRAMES]", n + 1);
        let at: u64 = words.first().and_then(|w| w.parse().ok()).expect(&bad);
        if let Some(lid) = words.get(1).and_then(|w| [("close", LID), ("open", LID | LID_OPEN)].into_iter().find(|l| l.0 == *w)) {
            out.push((at, at, lid.1));
            continue;
        }
        let mask = words.get(1).expect(&bad).split('+').fold(0u32, |m, b| m | 1 << BUTTONS.iter().position(|&x| x == b).expect(&bad));
        let held: u64 = words.get(2).map_or(6, |w| w.parse().expect(&bad));
        out.push((at, at + held.max(1) - 1, mask));
    }
    out
}

/// The battery files to pass and write back: (number, path, its bytes or none where the file is absent).
fn battery_files(spec: &str) -> Vec<(u32, String, Option<Vec<u8>>)> {
    spec.split(',').filter(|s| !s.is_empty()).map(|s| {
        let (n, path) = s.split_once('=').unwrap_or_else(|| panic!("`{s}` is not N=PATH"));
        (n.parse().unwrap_or_else(|_| panic!("`{n}` is not a battery file's number")), path.to_owned(), std::fs::read(path).ok())
    }).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("usage: run <entry's libretro host> <image|--loop-elf OUT|--exit-elf OUT|--data-disc OUT> <frames> [key=value]...");
        std::process::exit(2);
    }
    let built = match args[2].as_str() {
        "--loop-elf" => Some(ee_elf(&LOOP)),
        "--exit-elf" => Some(ee_elf(&EXIT)),
        "--data-disc" => Some(data_disc()),
        _ => None,
    };
    let image_path = match &built {
        Some(bytes) => {
            std::fs::write(&args[3], bytes).unwrap();
            args[3].clone()
        }
        None => args[2].clone(),
    };
    let rest = if built.is_some() { &args[4..] } else { &args[3..] };
    let frames: u64 = rest.first().and_then(|f| f.parse().ok()).unwrap_or(60);
    let settings = rest.iter().skip(1).cloned().collect::<Vec<_>>().join("\n");
    let lib = Library::load(Path::new(&args[1])).unwrap_or_else(|e| panic!("{e:?}"));
    println!("info: {}", lib.info_text().unwrap_or_default());
    if std::env::var_os("EMUSEN_RUN_SCHEMA").is_some() {
        println!("schema: {}", lib.settings_text().unwrap_or_default());
    }
    let image = std::fs::read(&image_path).unwrap();
    let path = CString::new(std::fs::canonicalize(&image_path).unwrap().to_string_lossy().as_bytes()).unwrap();
    let pads = std::env::var("EMUSEN_RUN_PADS").map(|f| pad_script(&std::fs::read_to_string(&f).unwrap_or_else(|e| panic!("{f}: {e}")))).unwrap_or_default();
    let batteries = battery_files(&std::env::var("EMUSEN_RUN_FILES").unwrap_or_default());
    let passed: Vec<(u32, &[u8])> = batteries.iter().filter_map(|(n, _, b)| b.as_deref().map(|b| (*n, b))).collect();
    let c = Create { settings: &settings, pixel_formats: 0b111, image_path: Some(&path), files: &passed, ..Create::new(&image) };
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
                let mask = pads.iter().filter(|p| (p.0..=p.1).contains(&n)).fold(0, |m, p| m | p.2);
                if mask & LID != 0 {
                    println!("lid at frame {n}: {:?}", m.media_lid(0, mask & LID_OPEN != 0));
                }
                if !pads.is_empty() {
                    m.set_buttons(0, mask & 0xFFFF, 0xFFFF).unwrap();
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
            for (n, path, _) in &batteries {
                let mut bytes = Vec::new();
                match m.battery(*n, &mut bytes) {
                    Ok(_) if !bytes.is_empty() => {
                        std::fs::write(path, &bytes).unwrap_or_else(|e| panic!("{path}: {e}"));
                        println!("file {n}: {} bytes written to {path}", bytes.len());
                    }
                    r => println!("file {n}: nothing to write ({r:?})"),
                }
            }
            let _ = m.log_drain(&mut log);
        }
    }
    println!("log:\n{log}");
}
