//! Runs a ROM for some frames and prints the printable runs of VRAM's low bytes, the text a test ROM writes.
#[path = "common/load.rs"]
mod load;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut m = load::machine(&a[1]);
    while m.total_frames() < a[2].parse::<i64>().unwrap() {
        m.run_frame();
    }
    let low: String = m.sys.ppu.vram.iter().map(|w| { let c = (w & 0xFF) as u8; if (0x20..0x7F).contains(&c) { c as char } else { '\n' } }).collect();
    let runs: Vec<&str> = low.split('\n').filter(|r| r.len() >= 4 && r.chars().collect::<std::collections::HashSet<_>>().len() > 2).collect();
    println!("{} | pc {:02X}:{:04X} waiting {} | {}", a[1].rsplit('/').next().unwrap(), m.cpu.pbr, m.cpu.pc, m.cpu.waiting, runs.join(" | "));
}
