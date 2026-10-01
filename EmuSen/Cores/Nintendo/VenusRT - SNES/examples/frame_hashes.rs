//! A hash of the picture after every frame, for proving a renderer change leaves every picture as it was
//! (VenusRT_Native.md §17). `frame_hashes <rom> <frames> <out>` writes one little-endian u64 a frame.
use std::hash::{Hash, Hasher};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut m = venusrt::machine::Machine::load_rom(&std::fs::read(&a[1]).unwrap()).unwrap();
    let frames: i64 = a[2].parse().unwrap();
    let mut out = Vec::with_capacity(frames as usize * 8);
    while m.total_frames() < frames {
        m.run_frame();
        let mut h = std::collections::hash_map::DefaultHasher::new();
        m.sys.ppu.frame.hash(&mut h);
        out.extend(h.finish().to_le_bytes());
    }
    std::fs::write(&a[3], out).unwrap();
}
