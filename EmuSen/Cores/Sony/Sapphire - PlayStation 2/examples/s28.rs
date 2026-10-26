//! Stage S28's runner (Sapphire_Native.md §39): a game's disc in the drive at power-on, the player's BIOS given by path,
//! driven by a control file a person writes while it runs. The BIOS image is handed to the machine and never read here.
//! What is written is pictures, the sound, states, timings and the hardware's own reports; no program counter or
//! instruction of either processor is printed, since the BIOS's OSDSYS runs from the same addresses a game's program is
//! loaded at, so an address alone cannot tell the game's code from the BIOS's (Sapphire_Plan.md §1.4, Sapphire_Native.md
//! §39.11).
//!
//! `cargo run --release --example s28 -- <bios> <americas|europe|japan> <disc.iso> <out-dir> [nvm=<file>]
//! [card=<file>|none] [load=<state>] [sound] [registers]`
//!
//! `<out-dir>/control` holds lines, read again whenever the run reaches its limit: `until <frame>`, `press <frame>
//! <bits> [frames]` (the core's button bits, `+`-joined, held six frames by default), `shot <frame>`, `every <k>`
//! (a picture every k frames), `save <frame>` (the state at that frame to `<out-dir>/state-<n>.sph2`), `ram <frame>`
//! (RDRAM from 00100000h as the EE sees it through its data cache, the game's memory and never the kernel's, to
//! `<out-dir>/ram-<n>.bin`), `gstrace <frame>` (every GS general register the game writes in that frame, in order, to
//! `<out-dir>/gstrace-<n>.txt`), `reports` (the
//! reports printed at the next limit, counting every register access from then on) and `quit`. Pictures are
//! `<out-dir>/frame-<n>.ppm`; `<out-dir>/status` is the frame reached; the EEPROM is written to `<out-dir>/nvm.bin`
//! whenever the console changes it, and the sound, with `sound`, to `<out-dir>/sound.raw` (48,000 Hz, stereo s16).
//! With `registers` every register access the units answer is counted from power-on, for the reports; with `trace` the
//! drive's register accesses are kept in order and written with the reports to `<out-dir>/trace.txt`; with `padtrace`
//! port 1's pad exchanges, to `<out-dir>/padtrace.txt`. The IOP's console, caught at IOMAN as §36.3 does, is written
//! with the reports to `<out-dir>/tty.txt`.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use emusen_native::abi::File;
use emusen_native::core::{Core, Create, Kept, Settings};
use emusen_native::ffi::StateMachine;
use sapphire::machine::{Machine, Market};
use sapphire::media::Media;

#[derive(Default)]
struct Control {
    until: i64,
    presses: BTreeMap<i64, (u32, i64)>,
    shots: Vec<i64>,
    saves: Vec<i64>,
    rams: Vec<i64>,
    gstraces: Vec<i64>,
    every: i64,
    reports: bool,
    quit: bool,
}

fn control(out: &Path) -> Control {
    let mut c = Control::default();
    let text = std::fs::read_to_string(out.join("control")).unwrap_or_default();
    for line in text.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        match w.as_slice() {
            ["until", n] => c.until = n.parse().unwrap_or(c.until),
            ["press", f, bits, rest @ ..] => {
                let mask = bits.split('+').filter_map(|b| b.parse::<u32>().ok()).fold(0u32, |m, b| m | 1 << b);
                let held = rest.first().and_then(|d| d.parse().ok()).unwrap_or(6);
                c.presses.insert(f.parse().unwrap_or(0), (mask, held));
            }
            ["shot", f] => c.shots.extend(f.parse::<i64>().ok()),
            ["save", f] => c.saves.extend(f.parse::<i64>().ok()),
            ["ram", f] => c.rams.extend(f.parse::<i64>().ok()),
            ["gstrace", f] => c.gstraces.extend(f.parse::<i64>().ok()),
            ["every", k] => c.every = k.parse().unwrap_or(0),
            ["reports"] => c.reports = true,
            ["quit"] => c.quit = true,
            _ => {}
        }
    }
    c
}

/// Where the game's memory begins in RDRAM: the EE kernel's lies below it and is never written out.
const GAME_RAM: usize = 0x0010_0000;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let bios = std::fs::read(&a[1]).expect("the BIOS");
    let market = match a[2].as_str() {
        "europe" => Market::Europe,
        "japan" => Market::Japan,
        _ => Market::Americas,
    };
    let image = std::fs::read(&a[3]).expect("the disc");
    let out = PathBuf::from(&a[4]);
    std::fs::create_dir_all(&out).unwrap();
    let opt = |k: &str| a[5..].iter().find_map(|o| o.strip_prefix(&format!("{k}=")).map(str::to_owned));
    let sound = a[5..].iter().any(|o| o == "sound");
    let all = a[5..].iter().any(|o| o == "registers");
    let trace = a[5..].iter().any(|o| o == "trace");
    let padtrace = a[5..].iter().any(|o| o == "padtrace");
    let Ok(Media::Disc(disc)) = Media::read(&image) else { panic!("not a PlayStation 2 disc") };
    println!("disc: {} sectors, program {}, serial {:?}", disc.sectors, disc.program, disc.serial);
    let nvm = opt("nvm").map(|p| std::fs::read(p).expect("the EEPROM"));
    let card = opt("card").filter(|p| p != "none").map(|p| std::fs::read(p).expect("the card"));
    let which = sapphire::v1::bios(market).which;
    let mut files = vec![File { which, data: &bios[..] }];
    if let Some(c) = &card {
        files.push(File { which: sapphire::v1::CARD_FILES[0], data: &c[..] });
    }
    if let Some(n) = &nvm {
        let i = match market {
            Market::Japan => 0,
            Market::Americas => 1,
            Market::Europe => 2,
        };
        files.push(File { which: sapphire::v1::NVM_FILES[i], data: &n[..] });
    }
    let request = Create {
        image: &[],
        settings: Settings::from_pairs(Vec::new()),
        files,
        pixel_formats: 1,
        host_abi_version: emusen_native::core::sys::ABI_VERSION,
        shared_removable: false,
        shared_console: false,
        device: None,
    };
    let mut m = <Machine as Core>::create_empty("ps2", &request).expect("the empty start");
    if opt("card").as_deref() == Some("none") {
        m.sys.board.iop.bus.sio2.slots[0].cards[0] = None;
    }
    let medium = sapphire::cdvd::open(Kept::from(image), Vec::new()).expect("the disc opens");
    m.disc_at_power_on(medium);
    if let Some(s) = opt("load") {
        m.load_state(&std::fs::read(&s).expect("the state")).expect("the state loads");
        println!("loaded {s} at frame {}", m.frames);
    }
    m.sys.board.iop.bus.cdvd.trace = trace.then(Vec::new);
    if let Some(p) = m.sys.board.iop.bus.sio2.pad_mut(0, 0) {
        p.trace = padtrace.then(Vec::new);
    }
    m.sys.board.reports.all = all;
    m.sys.board.iop.bus.reports.all = all;
    let mut sound_file = sound.then(|| std::io::BufWriter::new(std::fs::OpenOptions::new().create(true).append(true).open(out.join("sound.raw")).unwrap()));
    let started = Instant::now();
    let (mut mark_wall, mut mark_frame) = (Instant::now(), m.frames);
    let mut held: Option<(u32, i64)> = None;
    let mut chunk = vec![0i16; 1 << 16];
    let mut reported = false;
    loop {
        let c = control(&out);
        if c.reports && !reported {
            m.sys.board.reports.all = true;
            m.sys.board.iop.bus.reports.all = true;
        }
        if m.frames >= c.until {
            std::fs::write(out.join("status"), format!("{}\n", m.frames)).unwrap();
            if c.reports && !reported {
                reported = true;
                print_reports(&mut m, &out);
            }
            if c.quit {
                break;
            }
            std::thread::sleep(Duration::from_millis(500));
            continue;
        }
        let next = m.frames + 1;
        if let Some(&p) = c.presses.get(&next) {
            held = Some((p.0, next + p.1));
        }
        let mask = held.filter(|h| next < h.1).map_or(0, |h| h.0);
        Core::set_buttons(&mut m, 0, mask, 0xFFFF).unwrap();
        if c.gstraces.contains(&next) {
            m.sys.board.gs.trace = Some((Vec::new(), 0));
        }
        advance(&mut m);
        if let Some((t, images)) = m.sys.board.gs.trace.take() {
            let text: String = t.iter().map(|(r, d)| format!("{r:02X} {d:016X}\n")).collect();
            std::fs::write(out.join(format!("gstrace-{}.txt", m.frames)), format!("{text}HWREG writes {images}\n")).unwrap();
        }
        let n = m.drain_audio(&mut chunk, usize::MAX);
        if let Some(f) = sound_file.as_mut() {
            for s in &chunk[..2 * n] {
                f.write_all(&s.to_le_bytes()).unwrap();
            }
        }
        if m.nvm().dirty {
            m.nvm_mut().dirty = false;
            std::fs::write(out.join("nvm.bin"), &m.nvm().bytes[..]).unwrap();
            println!("frame {}: the console wrote its EEPROM", m.frames);
        }
        if c.saves.contains(&m.frames) {
            let mut s = vec![0; m.state_size()];
            m.save_state(&mut s).expect("the state saves");
            std::fs::write(out.join(format!("state-{}.sph2", m.frames)), &s).unwrap();
        }
        if c.rams.contains(&m.frames) {
            let ram: Vec<u8> = (GAME_RAM as u32..32 << 20).step_by(16).flat_map(|p| m.sys.peek(p, 16).to_le_bytes()).collect();
            std::fs::write(out.join(format!("ram-{}.bin", m.frames)), ram).unwrap();
        }
        if c.shots.contains(&m.frames) || (c.every > 0 && m.frames % c.every == 0) {
            write_ppm(&m, &out.join(format!("frame-{}.ppm", m.frames)));
            let lit = m.sys.board.pcrtc.picture.rgba.chunks(4).filter(|px| px[..3] != [0, 0, 0]).count();
            let fps = (m.frames - mark_frame) as f64 / mark_wall.elapsed().as_secs_f64();
            (mark_wall, mark_frame) = (Instant::now(), m.frames);
            let gs = &m.sys.board.gs;
            let r: Vec<String> = { use sapphire::gs::privileged::*; [PMODE, SMODE2, DISPFB1, DISPLAY1, DISPFB2, DISPLAY2, BGCOLOR] }.iter().map(|&a| format!("{:X}", gs.privileged_register(a))).collect();
            println!("frame {}: PMODE SMODE2 DISPFB1 DISPLAY1 DISPFB2 DISPLAY2 BGCOLOR {}", m.frames, r.join(" "));
            println!(
                "frame {} at {:.0} s wall ({fps:.2} frames/s since the last): picture {:?}, lit {lit}; stop {:?}; IOP held {}",
                m.frames,
                started.elapsed().as_secs_f64(),
                m.sys.board.pcrtc.picture.geometry,
                m.stop,
                m.sys.board.iop.held
            );
        }
    }
    print_reports(&mut m, &out);
}

fn advance(m: &mut Machine) {
    let target = m.sys.board.pcrtc.fields + 1;
    let (mut steps, mut warned) = (0u64, 0u32);
    while m.sys.board.pcrtc.fields < target {
        steps += 1;
        if steps % 200_000 == 0 && warned < 20 {
            warned += 1;
            let gs = &m.sys.board.gs;
            let r: Vec<String> = { use sapphire::gs::privileged::*; [PMODE, SMODE1, SMODE2, SYNCH1, SYNCH2, SYNCV, DISPLAY1, DISPLAY2, CSR] }.iter().map(|&a| format!("{:X}", gs.privileged_register(a))).collect();
            println!("frame {}: no field after {} steps; board at {} cycles; PMODE SMODE1 SMODE2 SYNCH1 SYNCH2 SYNCV DISPLAY1 DISPLAY2 CSR {}", m.frames, steps * 1024, m.sys.board.now, r.join(" "));
        }
        if m.booted && m.stop.is_none() {
            for _ in 0..1024 {
                if m.sys.step().is_err() {
                    m.stop = Some(sapphire::machine::Stop::Unimplemented);
                    println!("frame {}: the EE stopped on an instruction the interpreter lacks", m.frames);
                    break;
                }
            }
        } else {
            let t = m.sys.board.now + 16_384;
            let sapphire::board::System { ee, board } = &mut *m.sys;
            board.run_to(t, &mut ee.spram[..]);
        }
    }
    if m.booted {
        m.sys.board.iop.resolve_tty();
    }
    m.frames += 1;
}

fn print_reports(m: &mut Machine, out: &Path) {
    println!("frame {}; stop {:?}; EE exceptions {:?} level 2 {:?}; IOP exceptions {:?}", m.frames, m.stop, m.sys.ee.exceptions, m.sys.ee.exceptions2, m.sys.board.iop.exceptions);
    if let Some(t) = m.sys.board.iop.bus.cdvd.trace.take() {
        let mut runs: Vec<(u32, bool, u8, usize)> = Vec::new();
        for (o, w, v) in t {
            match runs.last_mut() {
                Some(r) if (r.0, r.1, r.2) == (o, w, v) => r.3 += 1,
                _ => runs.push((o, w, v, 1)),
            }
        }
        let text: String = runs.iter().map(|(o, w, v, k)| format!("{} {o:02X}h {v:02X}h x{k}\n", if *w { "W" } else { "R" })).collect();
        std::fs::write(out.join("trace.txt"), text).unwrap();
        println!("-- the drive's register accesses in order, repeats counted, {} runs in trace.txt", runs.len());
    }
    if let Some(t) = m.sys.board.iop.bus.sio2.pad_mut(0, 0).and_then(|p| p.trace.take()) {
        let text: String = t.iter().map(|(c, r, k)| format!("{c:02X?} -> {r:02X?} x{k}\n")).collect();
        std::fs::write(out.join("padtrace.txt"), text).unwrap();
    }
    let tty = &m.sys.board.iop.tty;
    std::fs::write(out.join("tty.txt"), &tty.text[..tty.len.min(tty.text.len())]).unwrap();
    println!("the IOP's console: {} bytes in tty.txt", tty.len);
    let regs: Vec<u8> = (0x20..0x40).map(|r| m.sys.board.iop.bus.cdvd.read8(r)).collect();
    println!("CDVD registers 20h-3Fh {regs:02X?}");
    let cd = &m.sys.board.iop.bus.cdvd;
    let counts = |c: &[u32; 256]| c.iter().enumerate().filter(|p| *p.1 > 0).map(|(n, k)| format!("{n:02X}h x{k}")).collect::<Vec<_>>().join(", ");
    println!("CDVD S commands: {}; N commands: {}", counts(&cd.s_count), counts(&cd.n_count));
    for (n, (p, len)) in cd.n_last.iter().enumerate().filter(|(n, _)| cd.n_count[*n] > 0) {
        println!("  N {n:02X}h last parameters {:02X?}", &p[..*len]);
    }
    for l in m.sys.board.reports.lines().iter().chain(m.sys.board.iop.bus.reports.lines().iter()) {
        println!("{l}");
    }
    if m.sys.board.reports.all {
        println!("-- registers answered");
        for l in m.sys.board.reports.register_lines().iter().chain(m.sys.board.iop.bus.reports.register_lines().iter()) {
            println!("{l}");
        }
    }
}

fn write_ppm(m: &Machine, path: &Path) {
    let p = &m.sys.board.pcrtc.picture;
    let mut v = format!("P6\n{} {}\n255\n", p.geometry.width, p.geometry.height).into_bytes();
    for px in p.rgba.chunks(4) {
        v.extend_from_slice(&px[..3]);
    }
    std::fs::write(path, v).unwrap();
}
