//! Replays a GS stream that the bench's `--gs-stream` wrote (Sapphire_Gs.md §3) into a GS made fresh, drawing in software
//! (the reference, on its own thread or `--workers N`), on the device (`--device`) or planning only (`--plan`), and prints
//! one line a field: the digest of the picture the display circuits show, and with `--memory` of the whole local memory
//! (which reads everything back, so its times are not the run's), then the run's milliseconds and the device's counts.
//! Each field's picture is read as the PCRTC reads it, after the V-sync, which is also where the device path waits.
//!
//! `cargo run --release --example gsstream -- <stream> [--device | --plan | --workers N] [--memory] [--pictures DIR K]`
//!
//! `--no-pictures` reads no picture (the PCRTC's part, which every way of drawing pays). `--parse` reads the events and applies none, the replay's own cost. `--pictures DIR K` writes every Kth field's picture as `DIR/field-<n>.ppm`. Every read the stream recorded is checked
//! against the GS's answer, and a difference is counted.

use sapphire::gs::psm::Psm;
use sapphire::gs::stream::{Event, Replay};
use sapphire::gs::{Gs, privileged};

const BASIS: u64 = 0xcbf2_9ce4_8422_2325;

fn fnv(h: u64, b: &[u8]) -> u64 {
    b.iter().fold(h, |h, &x| (h ^ u64::from(x)).wrapping_mul(0x0000_0100_0000_01B3))
}

/// The pixels the enabled display circuits show, read from the buffer as `pcrtc` reads it: (width, height, RGB).
fn picture(gs: &Gs) -> (u32, u32, Vec<u8>) {
    let pmode = gs.privileged_register(privileged::PMODE);
    let circuit = if pmode & 2 != 0 { 1 } else { 0 };
    if pmode & 3 == 0 {
        return (0, 0, Vec::new());
    }
    let dispfb = gs.privileged_register(if circuit == 1 { privileged::DISPFB2 } else { privileged::DISPFB1 });
    let display = gs.privileged_register(if circuit == 1 { privileged::DISPLAY2 } else { privileged::DISPLAY1 });
    let (fbp, fbw, psm) = ((dispfb & 0x1FF) as u32, (dispfb >> 9 & 0x3F) as u32, (dispfb >> 15 & 0x1F) as u32);
    let (dbx, dby) = ((dispfb >> 32 & 0x7FF) as u32, (dispfb >> 43 & 0x7FF) as u32);
    let (magh, magv) = ((display >> 23 & 0xF) as u32 + 1, (display >> 27 & 3) as u32 + 1);
    let (w, h) = (((display >> 32 & 0xFFF) as u32 + 1) / magh, ((display >> 44 & 0x7FF) as u32 + 1) / magv);
    let Some(p) = Psm::from_code(psm) else { return (0, 0, Vec::new()) };
    gs.settle(p, fbp * 32, fbw, (0, 0x7FF), (dby, (dby + h) & 0x7FF));
    let mut rgb = Vec::with_capacity((w * h * 3) as usize);
    for y in 0..h {
        for x in 0..w {
            let v = gs.peek(p, fbp * 32, fbw, (dbx + x) & 0x7FF, (dby + y) & 0x7FF);
            match p.bits() {
                16 => rgb.extend_from_slice(&[((v & 31) << 3) as u8, ((v >> 5 & 31) << 3) as u8, ((v >> 10 & 31) << 3) as u8]),
                _ => rgb.extend_from_slice(&[v as u8, (v >> 8) as u8, (v >> 16) as u8]),
            }
        }
    }
    (w, h, rgb)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| args.iter().position(|a| a == name);
    let bytes = std::fs::read(&args[0]).expect("the stream");
    let mut replay = Replay::open(&bytes).expect("a GS stream");
    let mut gs = Gs::new();
    if let Some(i) = flag("--workers") {
        gs.set_workers(sapphire::gs::workers::parse(&args[i + 1]).expect("a count of workers"));
    }
    if flag("--device").is_some() || flag("--plan").is_some() {
        gs.set_device(flag("--plan").is_some()).expect("the GS's device");
    }
    replay.restore(&mut gs).expect("the stream's state");
    let memory = flag("--memory").is_some();
    let parse_only = flag("--parse").is_some();
    let no_pictures = flag("--no-pictures").is_some();
    let pictures = flag("--pictures").map(|i| (std::path::PathBuf::from(&args[i + 1]), args[i + 2].parse::<u64>().expect("a count of fields")));
    if let Some((dir, _)) = &pictures {
        std::fs::create_dir_all(dir).expect("the pictures' folder");
    }
    let (mut field, mut misread, mut all) = (0u64, 0u64, BASIS);
    let started = std::time::Instant::now();
    let mut drawing = 0.0f64;
    let mut t0 = std::time::Instant::now();
    while let Some(e) = replay.next_event() {
        if parse_only {
            continue;
        }
        if !Replay::apply(&mut gs, e) {
            misread += 1;
        }
        if let Event::Vsync(_) = e {
            let (w, h, rgb) = if no_pictures { (0, 0, Vec::new()) } else { picture(&gs) };
            drawing += t0.elapsed().as_secs_f64();
            let p = fnv(BASIS, &rgb);
            all = fnv(all, &p.to_le_bytes());
            let m = if memory { gs.local().words().iter().fold(BASIS, |h, &x| (h ^ u64::from(x)).wrapping_mul(0x0000_0100_0000_01B3)) } else { 0 };
            println!("field {field}\t{w}x{h}\tpicture {p:016X}\tmemory {m:016X}");
            if let Some((dir, k)) = &pictures
                && field % k == 0
            {
                let mut v = format!("P6\n{w} {h}\n255\n").into_bytes();
                v.extend_from_slice(&rgb);
                std::fs::write(dir.join(format!("field-{field}.ppm")), v).expect("the picture written");
            }
            field += 1;
            t0 = std::time::Instant::now();
        }
    }
    let wall = started.elapsed().as_secs_f64();
    println!("fields {field}\tpictures {all:016X}\tmisread {misread}\tdrawing {:.3} ms a field\twall {wall:.2} s", 1e3 * drawing / field.max(1) as f64);
    if let Some(c) = gs.worker_counts() {
        let n = field.max(1) as f64;
        println!("workers\tprimitives {:.0} split {:.0} alone {:.0} (pages met {:.0}, texture met {:.0}) empty {:.0} a field\ttransitions {:.1} a field", c.primitives as f64 / n, c.split as f64 / n, c.alone as f64 / n, c.alone_why[0] as f64 / n, c.alone_why[1] as f64 / n, c.empty as f64 / n, c.transitions as f64 / n);
    }
    if let Some((c, name)) = gs.device_counts() {
        let p = c.plan;
        let n = field.max(1) as f64;
        let waits: Vec<String> = c.waits.iter().map(|&(k, ns)| format!("{k}/{:.1}ms", ns as f64 / 1e6)).collect();
        println!(
            "device\t{name}\trecords {:.0} empty {:.0} fallbacks {:?} snapshot records/stretches {:?}\tstretches {:.1} closes {:?}\tpairs {:.0} tiles {:.0} envs {:.0} cluts {:.0} staged pages {:.1} submissions {:.1} a field\twaits clutload/download/copy/upload/fallback/display/join {}\tread back pages {:.1} a field\tsubmitting {:.3} ms a field",
            p.records as f64 / n, p.empty as f64 / n, p.fallback, p.snapshot, p.stretches as f64 / n, p.closes, p.pairs as f64 / n, p.tiles as f64 / n, p.envs as f64 / n, p.cluts as f64 / n, p.staged_pages as f64 / n, p.submissions as f64 / n, waits.join(" "), c.read_back_pages as f64 / n, c.submit_ns as f64 / 1e6 / n
        );
    }
}
