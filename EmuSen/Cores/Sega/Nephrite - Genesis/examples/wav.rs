//! Runs an image for a number of frames and writes the sound it made as a 16-bit stereo WAV file, at the output
//! rate (Nephrite_Native.md §18). `wav <image> <frames> <out.wav>`; `NEPHRITE_PRESS` holds pad 1's buttons as
//! `examples/dump.rs` reads it, `NEPHRITE_MODEL=2` records a model 2, and `NEPHRITE_CIRCUIT=flat` records the mix
//! before the model's output circuit, for measuring a console's circuit against it.

use nephrite::machine::Machine;
use nephrite::media::Media;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let image = nephrite::media::cartridge_bytes(&std::fs::read(&args[1]).expect("the image")).into_owned();
    let frames: i64 = args[2].parse().expect("a frame count");
    let media = Media::read(&image);
    let model = nephrite::genesis::Model { model2: std::env::var("NEPHRITE_MODEL").is_ok_and(|v| v == "2"), ..nephrite::machine::default_model(&media) };
    let mut m = Machine::with_model(&image, media, model);
    if std::env::var("NEPHRITE_CIRCUIT").is_ok_and(|v| v == "flat") {
        m.genesis.hw.sound.circuit = nephrite::sound::Circuit::FLAT;
    }
    let mut samples = Vec::new();
    let mut buf = vec![0i16; 1 << 16];
    let presses: Vec<(i64, u32, i64)> = std::env::var("NEPHRITE_PRESS").unwrap_or_default().split(',').filter(|p| !p.is_empty()).map(|p| {
        let v: Vec<i64> = p.split(':').map(|x| x.parse().expect("a press")).collect();
        (v[0], v[1] as u32, v.get(2).copied().unwrap_or(4))
    }).collect();
    while m.frames < frames {
        m.pads[0] = presses.iter().filter(|p| (p.0..p.0 + p.2).contains(&m.frames)).fold(0, |a, p| a | p.1);
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
