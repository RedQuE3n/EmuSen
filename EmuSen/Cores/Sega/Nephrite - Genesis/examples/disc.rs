//! Runs a Sega CD disc on a boot ROM given as a firmware path: `disc <sheet or image> <frames> <dir> [press...]`, with
//! `NEPHRITE_BIOS` the boot ROM. Writes the picture every `NEPHRITE_EVERY` frames (default 60) as raw RGBA with its
//! size, and prints what the drive and the CDC did, for watching Nephrite's devices serve a disc (Nephrite_Native.md §53).
//! A press is `frame:bits:frames`, pad 1. Nothing of the boot ROM is read or printed.

use emusen_native::core::Kept;
use nephrite::machine::{Machine, default_model};
use nephrite::media::Media;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = std::path::Path::new(&args[1]);
    let frames: i64 = args[2].parse().expect("frames");
    let dir = std::path::Path::new(&args[3]);
    std::fs::create_dir_all(dir).expect("the folder");
    let presses: Vec<(i64, u32, i64)> = args[4..].iter().map(|p| {
        let v: Vec<i64> = p.split(':').map(|x| x.parse().expect("a press")).collect();
        (v[0], v[1] as u32, v[2])
    }).collect();
    let every: i64 = std::env::var("NEPHRITE_EVERY").ok().and_then(|v| v.parse().ok()).unwrap_or(60);
    let image = std::fs::read(path).expect("the image");
    let companions: Vec<Kept> = if emusen_disc::is_sheet(&image) {
        let sheet = emusen_disc::cue::Sheet::parse(&String::from_utf8_lossy(&image)).expect("a sheet");
        sheet.files.iter().map(|f| Kept::from(std::fs::read(path.parent().unwrap().join(f)).expect("a track"))).collect()
    } else {
        Vec::new()
    };
    let mut disc = nephrite::mcd::open_disc(Kept::from(image), companions).expect("a disc");
    let media = Media::read(&nephrite::mcd::system_area(&mut disc));
    let bios = std::env::var_os("NEPHRITE_BIOS").map(|p| std::fs::read(p).expect("the boot ROM"));
    let model = default_model(&media);
    let mut m = Machine::with_disc(&[], media, model, bios, Some(disc));
    let mut audio = vec![0i16; 4096];
    let (mut peak, mut sum) = (0i32, 0f64);
    let mut last = (0u8, 0u8, false, 0u8, 0u8, 0u8);
    while m.frames < frames {
        m.pads[0] = presses.iter().filter(|p| (p.0..p.0 + p.2).contains(&m.frames)).fold(0, |b, p| b | p.1);
        m.advance();
        loop {
            let n = emusen_native::core::Core::drain_audio(&mut m, &mut audio, 2048);
            if n == 0 {
                break;
            }
            for s in &audio[..2 * n] {
                peak = peak.max((*s as i32).abs());
                sum += (*s as f64) * (*s as f64);
            }
        }
        // The devices at their registers, as a game or a boot ROM drives them: the drive's command, the gate's switches.
        let s = &m.genesis.hw.mcd.as_ref().unwrap().hw;
        let seen = (s.cdd.status[0], s.cdd.command[0], s.cdd.hock, s.g.mask, s.g.cfm, s.g.cfs);
        if std::env::var_os("NEPHRITE_DEVICES").is_some() && seen != last {
            println!("  frame {:5}: drive status {:X} command {:X} hock {} interrupt mask {:02X} flags {:02X} {:02X}", m.frames, seen.0, seen.1, seen.2, seen.3, seen.4, seen.5);
            last = seen;
        }
        if m.frames % every == 0 {
            let f = &m.genesis.hw.vdp.frame;
            let mut out = format!("{} {}\n", f.width, f.height).into_bytes();
            out.extend_from_slice(&m.picture);
            std::fs::write(dir.join(format!("f{:05}.rgba", m.frames)), out).expect("the picture");
            let s = &m.genesis.hw.mcd.as_ref().unwrap().hw;
            println!("frame {:5}: drive status {:X} head {:6} report {}  cdc dd {} edt {}  fader {:04X}  audio peak {:5} rms {:7.1}", m.frames, s.cdd.status[0], s.cdd.head, s.cdd.report, s.cdc.dd, s.cdc.edt, s.fader, peak, (sum / (every as f64 * 1600.0)).sqrt());
            (peak, sum) = (0, 0.0);
        }
    }
}
