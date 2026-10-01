//! Runs a ROM until its CPU stops (byuu's tests end in STP) and prints the DMA registers, the latched counters and
//! the first SRAM bytes, for reading where such a test failed. `stop_regs <rom> [frames]`
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut m = venusrt::machine::Machine::load_rom(&std::fs::read(&a[1]).unwrap()).unwrap();
    let frames: i64 = a.get(2).and_then(|f| f.parse().ok()).unwrap_or(3600);
    while m.total_frames() < frames && !m.cpu.stopped {
        m.step();
    }
    let io = &m.sys.io;
    for ch in 0..2 {
        let regs: Vec<String> = (0..12).map(|i| format!("{:02X}", io[0x300 + ch * 16 + i])).collect();
        println!("43{ch}0: {}", regs.join(" "));
    }
    let sram: Vec<String> = m.sys.cart.sram.iter().take(8).map(|b| format!("{b:02X}")).collect();
    println!("stopped {} frame {} pc {:02X}:{:04X} a {:04X} ophct {:03X} opvct {:03X} colour0 {:04X} sram {}",
        m.cpu.stopped, m.total_frames(), m.cpu.pbr, m.cpu.pc, m.cpu.a, m.sys.ppu.ophct, m.sys.ppu.opvct, m.sys.ppu.cgram[0], sram.join(" "));
}
