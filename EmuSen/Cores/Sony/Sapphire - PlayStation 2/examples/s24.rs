//! Stage S24's runner (Sapphire_Native.md §35): the console started on a player's BIOS given by path, run for a count
//! of frames, with the hardware's own reports and the pictures at chosen frames. The image is handed to the machine
//! and never read here; nothing printed holds a program counter, an instruction or memory contents.
//!
//! `cargo run --release --example s24 -- <bios> <americas|europe|japan> <frames> <out-dir> [frame[:buttons]...]`
//! writes `<out-dir>/frame-<n>.ppm` at each frame named, pressing the buttons (the core's bit numbers, `+`-joined) for
//! that frame's run when given, and prints the reports at the end.

use std::time::Instant;

use emusen_native::core::Core;
use sapphire::machine::{Machine, Market};
use sapphire::media::Media;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let bios = std::fs::read(&a[1]).expect("the BIOS");
    let market = match a[2].as_str() {
        "europe" => Market::Europe,
        "japan" => Market::Japan,
        _ => Market::Americas,
    };
    let frames: i64 = a[3].parse().unwrap();
    let out = std::path::PathBuf::from(&a[4]);
    std::fs::create_dir_all(&out).unwrap();
    let mut shots = std::collections::BTreeMap::new();
    for s in &a[5..] {
        let (f, b) = s.split_once(':').unwrap_or((s, ""));
        let mask = b.split('+').filter(|x| !x.is_empty()).fold(0u32, |m, bit| m | 1 << bit.parse::<u32>().unwrap());
        shots.insert(f.parse::<i64>().unwrap(), mask);
    }
    let mut m = Machine::new(Media::read(&sapphire::media::program(&[0], 0x0010_0000)).unwrap(), market, [Some("ps2.dualshock2"), None]);
    m.insert_bios(&bios);
    let all = std::env::var_os("S24_ALL").is_some();
    m.sys.board.reports.all = all;
    m.sys.board.iop.bus.reports.all = all;
    let started = Instant::now();
    let mut last_report = Instant::now();
    while m.frames < frames {
        let press = shots.get(&(m.frames + 1)).copied().unwrap_or(0);
        Core::set_buttons(&mut m, 0, press, 0xFFFF).unwrap();
        m.advance();
        if shots.contains_key(&m.frames) {
            write_ppm(&m, &out.join(format!("frame-{}.ppm", m.frames)));
            let gs = &m.sys.board.gs;
            let r: Vec<String> = { use sapphire::gs::privileged::*; [PMODE, SMODE1, SMODE2, DISPFB1, DISPLAY1, DISPFB2, DISPLAY2, BGCOLOR] }.iter().map(|&a| format!("{:X}", gs.privileged_register(a))).collect();
            println!("frame {} PMODE SMODE1 SMODE2 DISPFB1 DISPLAY1 DISPFB2 DISPLAY2 BGCOLOR {}; picture {:?}", m.frames, r.join(" "), m.sys.board.pcrtc.picture.geometry);
        }
        if last_report.elapsed().as_secs() >= 20 {
            last_report = Instant::now();
            summary(&m, started);
        }
    }
    summary(&m, started);
    for l in m.sys.board.reports.lines().iter().chain(m.sys.board.iop.bus.reports.lines().iter()) {
        println!("{l}");
    }
    println!("EE exceptions {:?} level 2 {:?}; IOP exceptions {:?}", m.sys.ee.exceptions, m.sys.ee.exceptions2, m.sys.board.iop.exceptions);
    sif_state(&m);
    let cd = &m.sys.board.iop.bus.cdvd;
    let counts = |c: &[u32; 256]| c.iter().enumerate().filter(|p| *p.1 > 0).map(|(n, k)| format!("{n:02X}h x{k}")).collect::<Vec<_>>().join(", ");
    println!("CDVD S commands: {}; N commands: {}", counts(&cd.s_count), counts(&cd.n_count));
    for (n, (p, len)) in cd.s_last.iter().enumerate().filter(|(n, _)| cd.s_count[*n] > 0) {
        println!("  S {n:02X}h last parameters {:02X?}", &p[..*len]);
    }
    println!("-- registers answered");
    for l in m.sys.board.reports.register_lines().iter().chain(m.sys.board.iop.bus.reports.register_lines().iter()) {
        println!("{l}");
    }
}

fn summary(m: &Machine, started: Instant) {
    let b = &m.sys.board;
    let p = &b.pcrtc.picture;
    let lit = p.rgba.chunks(4).filter(|px| px[..3] != [0, 0, 0]).count();
    let sif = &b.iop.bus.sif;
    println!(
        "traffic {:?}; frame {} at {:.1} s wall, {:.2} s console; stop {:?}; IOP held {} resets {}; picture {}x{} lit {}; SIF MSFLG {:08X} SMFLG {:08X}; samples {}",
        b.traffic,
        m.frames,
        started.elapsed().as_secs_f64(),
        b.now as f64 / sapphire::clock::EE_HZ as f64,
        m.stop,
        b.iop.held,
        b.iop_resets,
        p.geometry.width,
        p.geometry.height,
        lit,
        sif.msflg,
        sif.smflg,
        b.iop.bus.spu2.out.len()
    );
}

/// The SIF's channels on both sides and its FIFOs, register values only.
fn sif_state(m: &Machine) {
    let b = &m.sys.board;
    for c in [5usize, 6] {
        let ch = &b.dmac.ch[c];
        println!("EE ch{c} chcr {:08X} madr {:08X} qwc {:08X} tadr {:08X}", ch.chcr, ch.madr, ch.qwc, ch.tadr);
    }
    println!("EE D_STAT {:08X} D_CTRL {:08X} D_ENABLE {:08X} I_STAT {:08X} I_MASK {:08X}", b.dmac.stat, b.dmac.ctrl, b.dmac.enable, b.intc.stat, b.intc.mask);
    let d = &b.iop.bus.dmac;
    for c in [4usize, 7, 9, 10] {
        let ch = &d.ch[c];
        println!("IOP ch{c} chcr {:08X} madr {:08X} bcr {:08X} tadr {:08X}", ch.chcr, ch.madr, ch.bcr, ch.tadr);
    }
    let spu = &b.iop.bus.spu2;
    for (n, core) in spu.cores.iter().enumerate() {
        println!("SPU2 core{n} ATTR {:04X} STATX {:04X} TSA {:08X}", core.read(sapphire::spu2::regs::ATTR), core.read(sapphire::spu2::regs::STATX), core.tsa);
    }
    println!("IOP DPCR {:08X} DICR {:08X}", d.dpcr, d.dicr);
    println!("IOP DPCR2 {:08X} DICR2 {:08X} DMACEN {:08X}; FIFOs {:?}", d.dpcr2, d.dicr2, d.dmacen, b.iop.bus.sif.fifo.iter().map(|f| f.len()).collect::<Vec<_>>());
}

fn write_ppm(m: &Machine, path: &std::path::Path) {
    let p = &m.sys.board.pcrtc.picture;
    let mut v = format!("P6\n{} {}\n255\n", p.geometry.width, p.geometry.height).into_bytes();
    for px in p.rgba.chunks(4) {
        v.extend_from_slice(&px[..3]);
    }
    std::fs::write(path, v).unwrap();
}
