//! Prints the VDP's registers after a number of frames, for reading a picture's settings (Nephrite_Native.md §14).
//! `regs <image> <frame>`.

use nephrite::machine::Machine;
use nephrite::media::Media;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let image = nephrite::media::cartridge_bytes(&std::fs::read(&args[1]).expect("the image")).into_owned();
    let frame: i64 = args[2].parse().expect("a frame");
    let mut m = Machine::new(&image, Media::read(&image));
    while m.frames < frame {
        m.advance();
    }
    let r = &m.genesis.hw.vdp.regs;
    println!("68000 pc {:06X} sr {:04X}; Z80 pc {:04X}", m.genesis.cpu.regs.pc, m.genesis.cpu.regs.sr, m.genesis.z80.regs.pc);
    let v = &m.genesis.hw.vdp;
    let hs = ((r[13] & 0x3F) as usize) << 10;
    println!("hscroll A {:04X} B {:04X}; vsram 0 {:04X} 1 {:04X}", u16::from_be_bytes([v.vram[hs], v.vram[hs + 1]]), u16::from_be_bytes([v.vram[hs + 2], v.vram[hs + 3]]), v.vsram_word(0), v.vsram_word(1));
    println!("{}", r.iter().enumerate().map(|(i, v)| format!("{i:02}:{v:02X}")).collect::<Vec<_>>().join(" "));
}
