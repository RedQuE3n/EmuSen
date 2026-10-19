//! The stage 2 test programs as cartridge images (Endou_Native.md §9.8), shared by the crate's tests and the referee
//! example: each source assembled with Beryl's Z80 assembler at $0000, in an image of its banks with each bank's own
//! number at $3F00.

use beryl_z80::asm::assemble;

pub struct Program {
    pub name: &'static str,
    pub source: &'static str,
    /// 16 KiB banks; more than three makes the board Sega's.
    pub banks: usize,
    /// Codemasters' header written over the image.
    pub codemasters: bool,
    /// Bytes placed after assembly, as (offset, value).
    pub patch: &'static [(usize, u8)],
    /// Sega's header at $7FF0 with this region code, so that a host takes the image as the Master System's.
    pub header: Option<u8>,
}

const fn program(name: &'static str, source: &'static str, banks: usize) -> Program {
    Program { name, source, banks, codemasters: false, patch: &[], header: Some(4) }
}

pub const PADS: Program = program("pads", include_str!("pads.s"), 2);
pub const PAUSE: Program = program("pause", include_str!("pause.s"), 2);
pub const REGION: Program = program("region", include_str!("region.s"), 2);
pub const SEGA: Program = program("sega", include_str!("sega.s"), 32);
pub const CODEMASTERS: Program = Program { codemasters: true, ..program("codemasters", include_str!("codemasters.s"), 8) };
pub const PORT_3E: Program = Program { patch: &[(0x100, 0x42)], ..program("port3e", include_str!("port3e.s"), 2) };
pub const SG_RAM: Program = Program { header: None, ..program("sgram", include_str!("sgram.s"), 1) };
pub const PSG: Program = program("psg", include_str!("psg.s"), 2);

pub const PROGRAMS: [&Program; 8] = [&PADS, &PAUSE, &REGION, &SEGA, &CODEMASTERS, &PORT_3E, &SG_RAM, &PSG];

pub fn image(p: &Program) -> Vec<u8> {
    let mut rom = vec![0u8; p.banks * 0x4000];
    for b in 0..p.banks {
        rom[b * 0x4000 + 0x3F00] = b as u8;
    }
    let code = assemble(p.source, 0).unwrap_or_else(|e| panic!("{}: {e}", p.name)).bytes;
    rom[..code.len()].copy_from_slice(&code);
    for &(at, v) in p.patch {
        rom[at] = v;
    }
    if let Some(region) = p.header {
        rom[0x7FF0..0x7FF8].copy_from_slice(b"TMR SEGA");
        rom[0x7FFF] = region << 4 | 0xC;
    }
    if p.codemasters { with_codemasters_header(rom) } else { rom }
}

/// The image with Codemasters' header written over all its banks, its checksum right [R SMS Power!'s "Codemasters
/// Header"].
pub fn with_codemasters_header(mut rom: Vec<u8>) -> Vec<u8> {
    let size = rom.len();
    rom[0x7FE0..0x7FF0].fill(0);
    rom[0x7FE0] = (size / 0x4000) as u8;
    rom[0x7FE1..0x7FE6].copy_from_slice(&[0x31, 0x08, 0x93, 0x10, 0x59]);
    let sum = rom.chunks(2).enumerate().filter(|(i, _)| !(0x3FF8..0x4000).contains(i)).fold(0u16, |s, (_, w)| s.wrapping_add(u16::from_le_bytes([w[0], w[1]])));
    rom[0x7FE6..0x7FE8].copy_from_slice(&sum.to_le_bytes());
    rom[0x7FE8..0x7FEA].copy_from_slice(&sum.wrapping_neg().to_le_bytes());
    rom
}
