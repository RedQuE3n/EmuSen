//! WRAM after every frame, one file a frame, for the probe's frame-by-frame differential (VenusRT_Native.md §13.6).
//! cargo run --release --example wram_frames <rom> <frames> <dir>
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut m = venusrt::machine::Machine::load_rom(&std::fs::read(&a[1]).unwrap()).unwrap();
    for f in 1..=a[2].parse::<i64>().unwrap() {
        m.run_frame();
        std::fs::write(format!("{}/venusrt_wram_f{f:05}.bin", a[3]), &m.sys.wram).unwrap();
    }
    eprintln!("pc {:02X}:{:04X} waiting {} stopped {} nmitimen {:02X}", m.cpu.pbr, m.cpu.pc, m.cpu.waiting, m.cpu.stopped, m.sys.dev.nmitimen);
}
