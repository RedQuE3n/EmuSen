//! Times a Sega CD disc's loading waits from its devices alone: `waits <sheet> <frames> [press...]`, with
//! `NEPHRITE_BIOS` the boot ROM and `NEPHRITE_BRAM` a backup RAM image. Prints each drive command as the sub CPU sends
//! it, each change of the drive's status with the head, and each frame where the picture turns blank or shows again,
//! so that a wait reads as frames from a command to the next picture (Nephrite_Native.md §64). A press is
//! `frame:bits:frames`, pad 1. Nothing of the boot ROM is read or printed.

use emusen_native::core::Kept;
use nephrite::machine::{Machine, default_model};
use nephrite::mcd::SeenKind;
use nephrite::media::Media;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = std::path::Path::new(&args[1]);
    let frames: i64 = args[2].parse().expect("frames");
    let presses: Vec<(i64, u32, i64)> = args[3..].iter().map(|p| {
        let v: Vec<i64> = p.split(':').map(|x| x.parse().expect("a press")).collect();
        (v[0], v[1] as u32, v[2])
    }).collect();
    let image = std::fs::read(path).expect("the image");
    let sheet = emusen_disc::cue::Sheet::parse(&String::from_utf8_lossy(&image)).expect("a sheet");
    let companions: Vec<Kept> = sheet.files.iter().map(|f| Kept::from(std::fs::read(path.parent().unwrap().join(f)).expect("a track"))).collect();
    let mut disc = nephrite::mcd::open_disc(Kept::from(image), companions).expect("a disc");
    let media = Media::read(&nephrite::mcd::system_area(&mut disc));
    let bios = std::env::var_os("NEPHRITE_BIOS").map(|p| if p == "open" { nephrite::mcd::bios::OPEN_BIOS.to_vec() } else { std::fs::read(p).expect("the boot ROM") });
    let model = default_model(&media);
    let mut m = Machine::with_disc(media, model, bios, Some(disc));
    if let Some(p) = std::env::var_os("NEPHRITE_BRAM") {
        let b = std::fs::read(p).expect("the backup RAM");
        let ram = &mut m.mcd_mut().unwrap().hw.bram;
        let n = b.len().min(ram.len());
        ram[..n].copy_from_slice(&b[..n]);
    }
    let mut audio = vec![0i16; 4096];
    let mut nibbles = [0u8; 10];
    let mut status = 0xFFu8;
    let mut blank = None;
    // Levels 4 and 5 the sub CPU took since the last line printed.
    let mut taken = [0u32; 8];
    m.mcd_mut().unwrap().hw.seen = Some(Vec::new());
    while m.frames < frames {
        m.pads[0] = presses.iter().filter(|p| (p.0..p.0 + p.2).contains(&m.frames)).fold(0, |b, p| b | p.1);
        m.advance();
        while emusen_native::core::Core::drain_audio(&mut m, &mut audio, 2048) != 0 {}
        let f = m.frames;
        let hw = &mut m.mcd_mut().unwrap().hw;
        let seen = std::mem::take(hw.seen.as_mut().unwrap());
        for a in seen {
            if a.what == SeenKind::Taken {
                taken[a.reg as usize & 7] += 1;
            }
            if a.what == SeenKind::SubWrite && (0x42..=0x4B).contains(&a.reg) {
                let i = (a.reg as usize - 0x42) & !1;
                (nibbles[i], nibbles[i + 1]) = ((a.value >> 8) as u8 & 15, a.value as u8 & 15);
                if i == 8 && !matches!(nibbles[0], 0 | 2) {
                    let n: String = nibbles.iter().map(|x| format!("{x:X}")).collect();
                    println!("{f:6} command {n} at {:6} head {:6}  levels 4 {} 5 {}", sector(&nibbles), hw.cdd.head, taken[4], taken[5]);
                    taken = [0; 8];
                }
            }
            if a.what == SeenKind::SubRead && a.reg == 0x38 && (a.value >> 8) as u8 & 15 != status {
                status = (a.value >> 8) as u8 & 15;
                println!("{f:6} status {status:X} head {:6}  levels 4 {} 5 {}", hw.cdd.head, taken[4], taken[5]);
                taken = [0; 8];
            }
        }
        let p = &m.picture;
        let now = p.chunks_exact(4).all(|px| px[..3] == p[..3]);
        if blank != Some(now) {
            println!("{f:6} picture {}", if now { "blank" } else { "shown" });
            blank = Some(now);
        }
    }
}

/// The sector a play or seek command names, from its BCD minutes, seconds and frames.
fn sector(n: &[u8; 10]) -> i64 {
    let (mm, ss, ff) = ((n[2] * 10 + n[3]) as i64, (n[4] * 10 + n[5]) as i64, (n[6] * 10 + n[7]) as i64);
    (mm * 60 + ss) * 75 + ff
}
