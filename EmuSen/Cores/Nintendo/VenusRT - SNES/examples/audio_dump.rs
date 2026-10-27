//! The S-DSP's own 32 kHz output from a state, before any resampling, as a 16-bit stereo WAV and a per-frame index.
//! `audio_dump <rom> <state> <frames> <out.wav>`; the index, `frame first_pair pairs`, beside it as `<out.wav>.idx`.

use emusen_native::ffi::StateMachine;

#[path = "common/load.rs"]
mod load;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut m = load::machine(&a[1]);
    let state = std::fs::read(&a[2]).expect("the state");
    m.load_state(&state).expect("a VenusRT state");
    let frames: u32 = a[3].parse().expect("a frame count");
    let mut pcm: Vec<i16> = Vec::new();
    let mut idx = String::new();
    let mut buf = vec![0i16; 1 << 16];
    m.samples.set_limit(1 << 20);
    for _ in 0..frames {
        m.run_frame();
        let max = buf.len() / 2;
        let n = m.samples.drain(&mut buf, max);
        idx += &format!("{} {} {}\n", m.total_frames(), pcm.len() / 2, n / 2);
        pcm.extend_from_slice(&buf[..n]);
    }
    let mut w = Vec::with_capacity(44 + pcm.len() * 2);
    let data = (pcm.len() * 2) as u32;
    w.extend(b"RIFF");
    w.extend((36 + data).to_le_bytes());
    w.extend(b"WAVEfmt ");
    w.extend(16u32.to_le_bytes());
    w.extend(1u16.to_le_bytes());
    w.extend(2u16.to_le_bytes());
    w.extend(32_000u32.to_le_bytes());
    w.extend(128_000u32.to_le_bytes());
    w.extend(4u16.to_le_bytes());
    w.extend(16u16.to_le_bytes());
    w.extend(b"data");
    w.extend(data.to_le_bytes());
    for s in pcm {
        w.extend(s.to_le_bytes());
    }
    std::fs::write(&a[4], w).expect("the WAV");
    std::fs::write(format!("{}.idx", a[4]), idx).expect("the index");
}
