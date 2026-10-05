//! Runs an image to each frame named and writes its memories there as `nephrite_<space>_f<frame>.bin`, the reference
//! probe's naming, for comparison with the references' dumps (Nephrite_Native.md §9). `dump <image> <dir> <frames...>`.

use nephrite::machine::Machine;
use nephrite::media::Media;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let image = std::fs::read(&args[1]).expect("the image");
    let dir = std::path::Path::new(&args[2]);
    std::fs::create_dir_all(dir).expect("the folder");
    let mut frames: Vec<i64> = args[3..].iter().map(|f| f.parse().expect("a frame")).collect();
    frames.sort();
    let mut m = Machine::new(&image, Media::read(&image));
    let started = std::time::Instant::now();
    for f in frames {
        while m.frames < f {
            m.advance();
        }
        for s in m.spaces().into_iter().filter(|s| !s.read_only) {
            std::fs::write(dir.join(format!("nephrite_{}_f{f:05}.bin", s.name.to_lowercase())), m.bytes(s.id).unwrap()).expect("a dump");
        }
    }
    eprintln!("{} frames in {:.3} s", m.frames, started.elapsed().as_secs_f64());
}
