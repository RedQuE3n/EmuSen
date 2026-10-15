//! The cost of a frame: `bench <image> [frames] [runs]`, the best run's mean in ms, the picture drawn. A disc runs with
//! `NEPHRITE_BIOS` as its boot ROM, a cue sheet with its tracks, and `NEPHRITE_WARM` frames are run before the timing
//! starts (Nephrite_Native.md §50, §60).

use emusen_native::core::Kept;
use nephrite::machine::{Machine, default_model};
use nephrite::media::Media;

fn machine(path: &std::path::Path, raw: &[u8], bios: &Option<Vec<u8>>) -> Machine {
    if emusen_disc::is_sheet(raw) {
        let sheet = emusen_disc::cue::Sheet::parse(&String::from_utf8_lossy(raw)).expect("a sheet");
        let tracks = sheet.files.iter().map(|f| Kept::from(std::fs::read(path.parent().unwrap().join(f)).expect("a track"))).collect();
        let mut disc = nephrite::mcd::open_disc(Kept::from(raw.to_vec()), tracks).expect("a disc");
        let media = Media::read(&nephrite::mcd::system_area(&mut disc));
        return Machine::with_disc(media.clone(), default_model(&media), bios.clone(), Some(disc));
    }
    let image = nephrite::media::cartridge_bytes(raw).into_owned();
    let media = Media::read(&image);
    match bios {
        Some(b) => Machine::with_bios(&image, media.clone(), default_model(&media), Some(b.clone())),
        None => Machine::new(&image, media),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = std::path::Path::new(&args[1]);
    let raw = std::fs::read(path).expect("the image");
    let frames: u32 = args.get(2).map_or(600, |f| f.parse().expect("frames"));
    let runs: u32 = args.get(3).map_or(5, |r| r.parse().expect("runs"));
    let warm: u32 = std::env::var("NEPHRITE_WARM").ok().map_or(0, |w| w.parse().expect("warm-up frames"));
    let bios = std::env::var_os("NEPHRITE_BIOS").map(|p| std::fs::read(p).expect("the boot ROM"));
    let mut best = f64::MAX;
    for _ in 0..runs {
        let mut m = machine(path, &raw, &bios);
        for _ in 0..warm {
            m.advance();
        }
        let t = std::time::Instant::now();
        for _ in 0..frames {
            m.advance();
        }
        best = best.min(t.elapsed().as_secs_f64() * 1000.0 / frames as f64);
    }
    println!("{best:.3}");
}
