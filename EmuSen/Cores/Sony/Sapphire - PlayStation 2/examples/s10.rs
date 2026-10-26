//! Stage S10's runner (Sapphire_Native.md §36): a ps2autotests program on the player's BIOS given by path, entered by
//! the fast boot, its text taken from the `tty` space from the expected text's first line to its last and graded whole
//! against the expected text. The BIOS image is handed to the machine and never read here; what is printed is the
//! program's own text, the hardware's reports and the counts of the capture.
//!
//! `cargo run --release --example s10 -- <bios> <americas|europe|japan> <program.elf|disc.iso> <expected> [max-frames] [out.txt]`
//! runs until the text holds the expected text's last line, the text stops growing for `S10_IDLE` frames (600) or the
//! frames run out, and prints PASS, FAIL with the first line that differs, or INCOMPLETE, with the frames and the wall
//! time. `S10_STATE` adds the DMACs', the SIF's and the INTCs' registers at the end, `S10_ALL` the registers answered,
//! `S10_REPORTS` the accesses no unit answers. A disc (§40) is put in the drive of a console started empty and the program
//! its SYSTEM.CNF names is entered. The EE runs recompiled, the core's default, unless `EMUSEN_SAPPHIRE_TIER=interpreter` (S36),
//! with idle skipping, the core's default, unless `EMUSEN_SAPPHIRE_IDLE=off` (S33).

use std::time::Instant;

use sapphire::machine::{Machine, Market, TTY_ID};
use emusen_native::core::{Core, Insert};
use sapphire::media::{Media, Sectors};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let bios = std::fs::read(&a[1]).expect("the BIOS");
    let market = match a[2].as_str() {
        "europe" => Market::Europe,
        "japan" => Market::Japan,
        _ => Market::Americas,
    };
    let program = std::fs::read(&a[3]).expect("the program");
    let expected = normalise(&String::from_utf8_lossy(&std::fs::read(&a[4]).expect("the expected text")));
    let max: i64 = a.get(5).map_or(3000, |f| f.parse().unwrap());
    let out = a.get(6).cloned();
    let mut m = match Media::read(&program).expect("an EE program or a disc") {
        Media::Disc(d) => disc(&program, &d.program, market, &bios),
        media => {
            let mut m = Machine::new(media, market, [Some("ps2.dualshock2"), None]);
            m.insert_bios(&bios);
            m.program = Some(program);
            m
        }
    };
    // The EE's tier as the core chooses it, the recompiler unless `EMUSEN_SAPPHIRE_TIER=interpreter` (S36, Sapphire_Native.md §44).
    if sapphire::v1::recompiler_chosen(None) {
        m.blocks = Some(Box::new(sapphire::blocks::Blocks::from_env()));
    }
    // Idle skipping as the core chooses it, on unless `EMUSEN_SAPPHIRE_IDLE=off` (S33, Sapphire_Native.md §47).
    m.idle.on = sapphire::v1::idle_chosen(None);
    let all = std::env::var_os("S10_ALL").is_some();
    m.sys.board.reports.all = all;
    m.sys.board.iop.bus.reports.all = all;
    let started = Instant::now();
    let first = expected.lines().next().unwrap_or("").to_string();
    let last = expected.lines().last().unwrap_or("").to_string();
    let idle: i64 = std::env::var("S10_IDLE").ok().and_then(|v| v.parse().ok()).unwrap_or(600);
    let mut verdict = None;
    let mut text = String::new();
    let mut grown = (0usize, 0i64);
    while m.frames < max {
        m.advance();
        if m.stop.is_some() {
            verdict = Some("STOPPED (an instruction the interpreter lacks)".to_string());
            break;
        }
        if m.frames % 10 == 0 {
            let len = m.sys.board.iop.tty.len;
            if len != grown.0 {
                grown = (len, m.frames);
            }
            text = program_text(&m, &first, &last);
            if text.lines().any(|l| l.trim_end() == last) || (m.entered.is_some() && m.frames - grown.1 >= idle) {
                break;
            }
        }
        if m.frames % 100 == 0 {
            eprintln!("frame {} at {:.0} s wall: tty {} bytes, {} lines of the program's", m.frames, started.elapsed().as_secs_f64(), m.sys.board.iop.tty.len, text.lines().count());
        }
    }
    text = program_text(&m, &first, &last);
    let verdict = verdict.unwrap_or_else(|| if text.is_empty() { "INCOMPLETE (none of the program's text)".into() } else { grade(&text, &expected) });
    if std::env::var_os("S10_STATE").is_some() {
        state(&m);
    }
    if std::env::var_os("S10_REPORTS").is_some() {
        for l in m.sys.board.reports.lines().iter().chain(m.sys.board.iop.bus.reports.lines().iter()) {
            println!("{l}");
        }
    }
    let t = &m.sys.board.iop.tty;
    println!(
        "{verdict}; {} frames ({:.2} s console), entered at frame {:?}, {:.1} s wall; tty {} bytes, write calls by fd 0/1/2/other {:?}; IOP resets {}",
        m.frames,
        m.sys.board.now as f64 / sapphire::clock::EE_HZ as f64,
        m.entered,
        started.elapsed().as_secs_f64(),
        t.len,
        t.calls,
        m.sys.board.iop_resets
    );
    if let Some(o) = out {
        std::fs::write(o, &text).unwrap();
    }
}

/// A disc's run (Sapphire_Native.md §40): the console started empty, the disc put in through the tray as the core
/// ABI's `MEDIA` puts it, and the program its SYSTEM.CNF names entered by the fast boot; the IOP reads the disc through
/// the drive.
fn disc(image: &[u8], boot2: &str, market: Market, bios: &[u8]) -> Machine {
    let name = boot2.rsplit(['\\', ':']).next().unwrap_or("").split(';').next().unwrap_or("");
    let program = Sectors::new(image).root_file(name).unwrap_or_else(|| panic!("the disc's root holds no {name}"));
    let mut m = Machine::new(Media::Empty, market, [Some("ps2.dualshock2"), None]);
    m.insert_bios(bios);
    m.media_lid(0, true).expect("the tray opens");
    m.media_insert(0, &Insert { image: Some(image), files: Vec::new(), restore: false }).expect("the disc goes in");
    m.media_lid(0, false).expect("the tray closes");
    m.program = Some(program);
    m
}

/// Where the machine stands at the end, registers and modes only: which unit waits on which (Sapphire_Plan.md §1.4).
fn state(m: &Machine) {
    let b = &m.sys.board;
    if b.reports.all {
        for l in b.reports.register_lines().iter().chain(b.iop.bus.reports.register_lines().iter()) {
            println!("{l}");
        }
    }
    println!("EE: kernel mode {} status {:08X} exceptions {:?}", m.sys.ee.kernel(), m.sys.ee.cop0[12], m.sys.ee.exceptions);
    for c in [5usize, 6] {
        let ch = &b.dmac.ch[c];
        println!("EE ch{c} chcr {:08X} madr {:08X} qwc {:08X} tadr {:08X}", ch.chcr, ch.madr, ch.qwc, ch.tadr);
    }
    println!("EE D_STAT {:08X} D_CTRL {:08X} D_ENABLE {:08X} I_STAT {:08X} I_MASK {:08X}", b.dmac.stat, b.dmac.ctrl, b.dmac.enable, b.intc.stat, b.intc.mask);
    let d = &b.iop.bus.dmac;
    for c in [9usize, 10] {
        let ch = &d.ch[c];
        println!("IOP ch{c} chcr {:08X} madr {:08X} bcr {:08X} tadr {:08X}", ch.chcr, ch.madr, ch.bcr, ch.tadr);
    }
    let sif = &b.iop.bus.sif;
    println!(
        "IOP SR {:08X} DICR {:08X} DICR2 {:08X}; SIF FIFOs {:?} MSFLG {:08X} SMFLG {:08X}; IOP I_STAT {:08X} I_MASK {:08X}; IOP exceptions {:?}",
        b.iop.cpu.regs.sr,
        d.read(sapphire::iop::dma::DICR),
        d.dicr2,
        sif.fifo.iter().map(|f| f.len()).collect::<Vec<_>>(),
        sif.msflg,
        sif.smflg,
        b.iop.bus.intc.stat,
        b.iop.bus.intc.mask,
        b.iop.exceptions
    );
}

fn normalise(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n").trim_end_matches('\n').to_string()
}

/// The `tty` space's text from the expected text's first line on, what the BIOS printed before the program left out, to
/// the program's closing line, what follows being the BIOS's once the program has exited.
fn program_text(m: &Machine, first: &str, last: &str) -> String {
    let len = m.sys.board.iop.tty.len;
    let bytes: Vec<u8> = (0..len).filter_map(|i| m.space_byte(TTY_ID, i)).collect();
    let all = normalise(&String::from_utf8_lossy(&bytes));
    let Some(at) = all.find(first) else { return String::new() };
    let mut out = Vec::new();
    for l in all[at..].lines() {
        out.push(l);
        if l.trim_end() == last {
            break;
        }
    }
    out.join("\n")
}

/// Every line equal and nothing after the last, as WiseMan's ExpectedText grades.
fn grade(text: &str, expected: &str) -> String {
    let got: Vec<&str> = text.lines().collect();
    let want: Vec<&str> = expected.lines().collect();
    for (i, w) in want.iter().enumerate() {
        match got.get(i) {
            Some(g) if g.trim_end() == w.trim_end() => {}
            Some(g) => return format!("FAIL at line {}: expected {:?}, got {:?}", i + 1, w, g),
            None => return format!("INCOMPLETE at line {}", i + 1),
        }
    }
    if got.len() > want.len() {
        return format!("FAIL: {} lines past the end, the first {:?}", got.len() - want.len(), got[want.len()]);
    }
    "PASS".into()
}
