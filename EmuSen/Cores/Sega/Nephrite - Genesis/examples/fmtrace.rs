//! Runs an image for a number of frames with the YM2612's channel trace on and writes each sample's six nine-bit
//! channel outputs as little-endian i16, for comparison with the board's output pins (Nephrite_Native.md §19).
//! `fmtrace <image> <frames> <out>`.

use nephrite::machine::Machine;
use nephrite::media::Media;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let image = nephrite::media::cartridge_bytes(&std::fs::read(&args[1]).expect("the image")).into_owned();
    let frames: i64 = args[2].parse().expect("a frame count");
    let mut m = Machine::new(&image, Media::read(&image));
    m.genesis.hw.sound.ym.trace = Some(Vec::new());
    while m.frames < frames {
        m.advance();
    }
    let t = m.genesis.hw.sound.ym.trace.take().unwrap();
    let bytes: Vec<u8> = t.iter().flat_map(|c| c.iter().flat_map(|&v| (v as i16).to_le_bytes())).collect();
    std::fs::write(&args[3], bytes).expect("the trace");
}
