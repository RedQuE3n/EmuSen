//! The picture at a frame, as raw RGBA, and the PPU registers that name the features a scene uses (VenusRT_Native.md §15).
//! cargo run --release --example picture_dump <rom> <frame> <out.rgba>
#[path = "common/load.rs"]
mod load;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let mut m = load::machine(&a[1]);
    let frame: i64 = a[2].parse().unwrap();
    while m.total_frames() < frame {
        m.run_frame();
    }
    std::fs::write(&a[3], m.sys.ppu.picture()).unwrap();
    let r = &m.sys.ppu.regs;
    println!(
        "inidisp {:02X} obsel {:02X} bgmode {:02X} mosaic {:02X} tm {:02X} ts {:02X} tmw {:02X} w12sel {:02X} w34sel {:02X} wobjsel {:02X} cgwsel {:02X} cgadsub {:02X} setini {:02X}",
        r[0x00], r[0x01], r[0x05], r[0x06], r[0x2C], r[0x2D], r[0x2E], r[0x23], r[0x24], r[0x25], r[0x30], r[0x31], r[0x33]
    );
    let p = &m.sys.ppu;
    println!("sc {:02X} {:02X} {:02X} {:02X} nba {:02X} {:02X} hofs {:04X?} vofs {:04X?}", r[0x07], r[0x08], r[0x09], r[0x0A], r[0x0B], r[0x0C], p.hofs, p.vofs);
}
