//! Runs an image to each frame named and writes its memories there as `nephrite_<space>_f<frame>.bin`, the reference
//! probe's naming, and the picture as RGBA, for comparison with the references' dumps (Nephrite_Native.md §9, §14).
//! `dump <image> <dir> <frames...>`.

use nephrite::machine::Machine;
use nephrite::media::Media;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let image = nephrite::media::cartridge_bytes(&std::fs::read(&args[1]).expect("the image")).into_owned();
    let dir = std::path::Path::new(&args[2]);
    std::fs::create_dir_all(dir).expect("the folder");
    let mut frames: Vec<i64> = args[3..].iter().map(|f| f.parse().expect("a frame")).collect();
    frames.sort();
    let mut m = Machine::new(&image, Media::read(&image));
    // NEPHRITE_PRESS="frame:bits:frames,..." holds pad 1's bits (Up 0 ... Start 7) for the frames given.
    let presses: Vec<(i64, u32, i64)> = std::env::var("NEPHRITE_PRESS").unwrap_or_default().split(',').filter(|p| !p.is_empty()).map(|p| {
        let v: Vec<i64> = p.split(':').map(|x| x.parse().expect("a press")).collect();
        (v[0], v[1] as u32, v.get(2).copied().unwrap_or(4))
    }).collect();
    let started = std::time::Instant::now();
    for f in frames {
        while m.frames < f {
            m.pads[0] = presses.iter().filter(|p| (p.0..p.0 + p.2).contains(&m.frames)).fold(0, |a, p| a | p.1);
            m.advance();
        }
        for s in m.spaces().into_iter().filter(|s| !s.read_only) {
            std::fs::write(dir.join(format!("nephrite_{}_f{f:05}.bin", s.name.to_lowercase())), m.bytes(s.id).unwrap()).expect("a dump");
        }
        let fr = &m.genesis.hw.vdp.frame;
        std::fs::write(dir.join(format!("nephrite_screen_f{f:05}_{}x{}.rgba", fr.width, fr.height)), &m.picture).expect("the picture");
    }
    eprintln!("{} frames in {:.3} s", m.frames, started.elapsed().as_secs_f64());
}
