//! VRAM after some frames, written to a file, for comparison with the probe's dump.
//! cargo run --release --example vram_dump <rom> <frames> <out>
#[path = "common/load.rs"]
mod load;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut m = load::machine(&a[1]);
    while m.total_frames() < a[2].parse::<i64>().unwrap() {
        m.run_frame();
    }
    std::fs::write(&a[3], m.vram_bytes()).unwrap();
}
