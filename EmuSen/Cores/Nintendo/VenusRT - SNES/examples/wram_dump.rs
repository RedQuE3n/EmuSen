//! Runs a ROM for some frames and writes WRAM to a file, for black-box comparisons with the probe's dumps.
//! cargo run --release --example wram_dump <rom> <frames> <out>

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut m = venusrt::machine::Machine::load_rom(&std::fs::read(&a[1]).unwrap()).unwrap();
    let frames: i64 = a[2].parse().unwrap();
    while m.total_frames() < frames {
        m.run_frame();
    }
    eprintln!("{:?} pc {:02X}:{:04X} waiting {} stopped {} unimplemented {}", m.sys.cart.header.map, m.cpu.pbr, m.cpu.pc, m.cpu.waiting, m.cpu.stopped, m.cpu.unimplemented);
    std::fs::write(&a[3], &m.sys.wram).unwrap();
}
