//! Stage S25 part B's runner (Sapphire_Native.md §30.15): the console started with nothing in it through the core ABI's
//! empty start, on a player's BIOS given by path and a card in slot 1, driven by a control file a person writes while it
//! runs. The image is handed to the machine and never read here; what is written is pictures, the card's file and the
//! hardware's own reports, nothing holding a program counter, an instruction or memory contents.
//!
//! `cargo run --release --example s25b -- <bios> <americas|europe|japan> <out-dir> [<card-file>|-|none]`
//!
//! `-` leaves slot 1 the erased card a machine holds without a file; `none` takes the card out of slot 1.
//!
//! `<out-dir>/control` holds lines, read again whenever the run reaches its limit: `until <frame>` (run to that frame,
//! then wait for more), `press <frame> <bits> [frames]` (the core's button bits, `+`-joined, held from that frame, six
//! frames by default), `shot <frame>` (a picture at that frame), `every <k>` (a picture every k frames) and `quit`.
//! Pictures are `<out-dir>/frame-<n>.ppm`; `<out-dir>/status` is the frame reached; on `quit` the card in slot 1 is
//! written to `<out-dir>/card1.ps2` with whether the console wrote it, and the reports are printed.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use emusen_native::abi::File;
use emusen_native::core::{Core, Create, Settings};
use sapphire::machine::{Machine, Market};

#[derive(Default)]
struct Control {
    until: i64,
    presses: BTreeMap<i64, (u32, i64)>,
    shots: Vec<i64>,
    every: i64,
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
            ["every", k] => c.every = k.parse().unwrap_or(0),
            ["quit"] => c.quit = true,
            _ => {}
        }
    }
    c
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let bios = std::fs::read(&a[1]).expect("the BIOS");
    let market = match a[2].as_str() {
        "europe" => Market::Europe,
        "japan" => Market::Japan,
        _ => Market::Americas,
    };
    let out = PathBuf::from(&a[3]);
    std::fs::create_dir_all(&out).unwrap();
    let card = a.get(4).filter(|p| !matches!(p.as_str(), "-" | "none")).map(|p| std::fs::read(p).expect("the card"));
    let which = sapphire::v1::bios(market).which;
    let mut files = vec![File { which, data: &bios[..] }];
    if let Some(c) = &card {
        files.push(File { which: sapphire::v1::CARD_FILES[0], data: &c[..] });
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
    if a.get(4).map(String::as_str) == Some("none") {
        m.sys.board.iop.bus.sio2.slots[0].cards[0] = None;
    }
    println!("started empty: market {:?}, booted {}, drive {:?}", m.market, m.booted, m.drives().iter().map(|d| (d.lid_open, d.loaded)).collect::<Vec<_>>());
    let started = Instant::now();
    let mut held: Option<(u32, i64)> = None;
    loop {
        let c = control(&out);
        if m.frames >= c.until {
            std::fs::write(out.join("status"), format!("{}\n", m.frames)).unwrap();
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
        m.advance();
        if c.shots.contains(&m.frames) || (c.every > 0 && m.frames % c.every == 0) {
            write_ppm(&m, &out.join(format!("frame-{}.ppm", m.frames)));
            let lit = m.sys.board.pcrtc.picture.rgba.chunks(4).filter(|px| px[..3] != [0, 0, 0]).count();
            println!("frame {} at {:.0} s wall: picture {:?}, lit {lit}; stop {:?}; IOP held {}", m.frames, started.elapsed().as_secs_f64(), m.sys.board.pcrtc.picture.geometry, m.stop, m.sys.board.iop.held);
        }
    }
    println!("stop {:?}; EE exceptions {:?} level 2 {:?}; IOP exceptions {:?}", m.stop, m.sys.ee.exceptions, m.sys.ee.exceptions2, m.sys.board.iop.exceptions);
    let cd = &m.sys.board.iop.bus.cdvd;
    let counts = |c: &[u32; 256]| c.iter().enumerate().filter(|p| *p.1 > 0).map(|(n, k)| format!("{n:02X}h x{k}")).collect::<Vec<_>>().join(", ");
    println!("CDVD S commands: {}; N commands: {}", counts(&cd.s_count), counts(&cd.n_count));
    if let Some(c) = m.card(0) {
        std::fs::write(out.join("card1.ps2"), &c.image).unwrap();
        println!("card 1 written by the console: {}", c.dirty);
    }
    for l in m.sys.board.reports.lines().iter().chain(m.sys.board.iop.bus.reports.lines().iter()) {
        println!("{l}");
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
