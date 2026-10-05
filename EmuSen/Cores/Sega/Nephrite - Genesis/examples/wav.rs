//! Runs an image for a number of frames and writes the sound it made as a 16-bit stereo WAV file, at the output
//! rate (Nephrite_Native.md §18). `wav <image> <frames> <out.wav>`.

use nephrite::machine::Machine;
use nephrite::media::Media;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let image = nephrite::media::cartridge_bytes(&std::fs::read(&args[1]).expect("the image")).into_owned();
    let frames: i64 = args[2].parse().expect("a frame count");
    let mut m = Machine::new(&image, Media::read(&image));
    let mut samples = Vec::new();
    let mut buf = vec![0i16; 1 << 16];
    while m.frames < frames {
        m.advance();
        let half = buf.len() / 2;
        let n = m.audio.drain(&mut buf, half);
        samples.extend_from_slice(&buf[..n]);
    }
    let rate = nephrite::sound::RATE;
    let mut out = Vec::with_capacity(44 + samples.len() * 2);
    let data = (samples.len() * 2) as u32;
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&[1, 0, 2, 0]);
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&(rate * 4).to_le_bytes());
    out.extend_from_slice(&[4, 0, 16, 0]);
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data.to_le_bytes());
    for s in samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    std::fs::write(&args[3], out).expect("the WAV file");
}
