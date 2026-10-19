//! Builds Nephrite's open Sega CD BIOS from its assembly sources with Beryl's 68000 assembler (Nephrite_OpenBios.md):
//! the sub CPU's program, the font from `beryl-font`, and the main CPU's program around them, into a 128 KiB boot ROM
//! in `OUT_DIR`.

use std::path::Path;

use beryl_m68k::asm::{Program, assemble_with};

const ROM_SIZE: usize = 0x2_0000;
/// The sub CPU's program runs from PRG-RAM `$0000`; its code must end where the work area begins (BM §1-4).
const SUB_IMAGE: usize = 0x6000;
const SUB_WORK: i64 = 0x5400;

fn main() {
    println!("cargo::rerun-if-changed=src/mcd/bios");
    if std::env::var_os("CARGO_FEATURE_MCD").is_none() {
        return;
    }
    let dir = Path::new("src/mcd/bios");
    let read = |name: &str| std::fs::read_to_string(dir.join(name)).ok();
    let source = |name: &str| read(name).unwrap_or_else(|| panic!("{name} is missing"));
    let sub = build("sub.s", assemble_with(&source("sub.s"), 0, &[], &read));
    assert!(sub.symbols["code_end"] <= SUB_WORK, "the sub CPU's code runs past $5400");
    assert_eq!(sub.bytes.len(), SUB_IMAGE, "the sub CPU's image ends at the jump table's end, $6000");
    let main = build("main.s", assemble_with(&source("main.s"), 0, &[("SUB_SIZE", SUB_IMAGE as i64)], &read));
    let (font_at, sub_at) = (main.symbols["FONT_AT"] as usize, main.symbols["SUB_AT"] as usize);
    assert!(main.bytes.len() <= font_at, "the main CPU's program runs into the font");
    let font = font();
    assert!(font_at + font.len() <= sub_at && sub_at + sub.bytes.len() <= ROM_SIZE);
    let mut rom = vec![0xFF; ROM_SIZE];
    rom[..main.bytes.len()].copy_from_slice(&main.bytes);
    rom[font_at..font_at + font.len()].copy_from_slice(&font);
    rom[sub_at..sub_at + sub.bytes.len()].copy_from_slice(&sub.bytes);
    let out = Path::new(&std::env::var_os("OUT_DIR").expect("OUT_DIR")).to_path_buf();
    std::fs::write(out.join("open_bios.bin"), rom).expect("the boot ROM written");
    let symbols: String = ["bram_call", "BRAMWORK", "BW_SIZE"]
        .iter()
        .map(|n| format!("pub const {}: u32 = {:#x};\n", n.to_uppercase(), sub.symbols[*n]))
        .collect();
    std::fs::write(out.join("open_bios_symbols.rs"), symbols).expect("the symbols written");
}

fn build(name: &str, p: Result<Program, beryl_m68k::asm::AsmError>) -> Program {
    p.unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// `beryl-font`'s glyphs as 64 tiles of 4-bit pixels, colour 1 on 0, each glyph's five columns from the tile's second
/// and its seven rows from the first.
fn font() -> Vec<u8> {
    let mut tiles = vec![0u8; 64 * 32];
    for g in beryl_font::glyphs() {
        let tile = (g.char - 32) as usize;
        for (row, bits) in g.rows.into_iter().enumerate() {
            for x in 0..5 {
                if bits & (0x10 >> x) != 0 {
                    let px = x + 1;
                    tiles[tile * 32 + row * 4 + px / 2] |= if px % 2 == 0 { 0x10 } else { 0x01 };
                }
            }
        }
    }
    tiles
}
