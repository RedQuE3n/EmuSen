//! VRAM after some frames, written to a file, for comparison with the probe's dump.
//! cargo run --release --example vram_dump <rom> <frames> <out>
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut m = venusrt::machine::Machine::load_rom(&std::fs::read(&a[1]).unwrap()).unwrap();
    while m.total_frames() < a[2].parse::<i64>().unwrap() {
        m.run_frame();
    }
    std::fs::write(&a[3], m.vram_bytes()).unwrap();
}
