//! The first bytes of SRAM after some frames: byuu's tests keep the failing test's number at $700000.
//! `sram_head <rom> [frames]`
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut m = venusrt::machine::Machine::load_rom(&std::fs::read(&a[1]).unwrap()).unwrap();
    let frames: i64 = a.get(2).and_then(|f| f.parse().ok()).unwrap_or(600);
    while m.total_frames() < frames {
        m.run_frame();
    }
    let s = &m.sys.cart.sram;
    let head: Vec<String> = s.iter().take(4).map(|b| format!("{b:02X}")).collect();
    println!("{} colour0 {:04X}", head.join(" "), m.sys.ppu.cgram[0]);
}
