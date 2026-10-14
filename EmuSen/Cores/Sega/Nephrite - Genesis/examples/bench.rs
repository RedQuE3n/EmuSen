//! The cost of a frame: `bench <image> [frames] [runs]`, the best run's mean in ms, the picture drawn. A disc runs with
//! `NEPHRITE_BIOS` as its boot ROM (Nephrite_Native.md §50).

use nephrite::machine::{Machine, default_model};
use nephrite::media::Media;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let image = nephrite::media::cartridge_bytes(&std::fs::read(&args[1]).expect("the image")).into_owned();
    let frames: u32 = args.get(2).map_or(600, |f| f.parse().expect("frames"));
    let runs: u32 = args.get(3).map_or(5, |r| r.parse().expect("runs"));
    let bios = std::env::var_os("NEPHRITE_BIOS").map(|p| std::fs::read(p).expect("the boot ROM"));
    let mut best = f64::MAX;
    for _ in 0..runs {
        let media = Media::read(&image);
        let mut m = match &bios {
            Some(b) => Machine::with_bios(&image, media.clone(), default_model(&media), Some(b.clone())),
            None => Machine::new(&image, media),
        };
        let t = std::time::Instant::now();
        for _ in 0..frames {
            m.advance();
        }
        best = best.min(t.elapsed().as_secs_f64() * 1000.0 / frames as f64);
    }
    println!("{best:.3}");
}
