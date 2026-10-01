//! Runs a ROM with the power-on position moved some master clocks into line 0, and prints the text it leaves in VRAM.
//! `power_phase <rom> <frames> <clocks> [dir]`, with a dir writing VRAM, CGRAM and OAM there; dispute D-6's experiment.
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut m = venusrt::machine::Machine::load_rom(&std::fs::read(&a[1]).unwrap()).unwrap();
    let clocks: u16 = a[3].parse().unwrap();
    if clocks > 0 {
        m.sys.timing.line_clock += clocks;
        m.sys.timing.clock += clocks as u64;
        m.sys.timing.schedule();
    }
    while m.total_frames() < a[2].parse::<i64>().unwrap() {
        m.run_frame();
    }
    let low: String = m.sys.ppu.vram.iter().map(|w| { let c = (w & 0xFF) as u8; if (0x20..0x7F).contains(&c) { c as char } else { '\n' } }).collect();
    let runs: Vec<&str> = low.split('\n').map(str::trim).filter(|r| r.len() >= 4 && r.chars().collect::<std::collections::HashSet<_>>().len() > 2).collect();
    if let Some(dir) = a.get(4) {
        std::fs::write(format!("{dir}/vram.bin"), m.vram_bytes()).unwrap();
        std::fs::write(format!("{dir}/cgram.bin"), m.cgram_bytes()).unwrap();
        std::fs::write(format!("{dir}/oam.bin"), &m.sys.ppu.oam).unwrap();
    }
    println!("{clocks}\t{}", runs.join(" | "));
}
