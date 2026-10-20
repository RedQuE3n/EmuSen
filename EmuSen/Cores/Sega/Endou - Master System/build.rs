//! Builds Endou's open boot program from its assembly sources with Beryl's Z80 assembler (Endou_OpenBoot.md §3): the
//! RAM program for $C700, the font from `beryl-font` as Master System tiles, and the ROM program around them, into an
//! 8 KiB image in `OUT_DIR`, once for the export Master Systems and once for the Japanese one, which checks no header.

use std::path::Path;

use beryl_z80::asm::{Program, assemble_with};

/// The BIOS's size, its program, the font's place and the RAM program's place in it.
const IMAGE: usize = 0x2000;
const FONT_AT: usize = 0x1000;
const RAM_AT: usize = 0x1800;
const RAM_PROGRAM: u32 = 0xC700;
/// Where the RAM program returns to the ROM: no game, or a refusal.
const ROM_NO_GAME: i64 = 0x0080;
const ROM_REFUSE: i64 = 0x0083;

fn main() {
    println!("cargo::rerun-if-changed=src/boot");
    let out = Path::new(&std::env::var_os("OUT_DIR").expect("OUT_DIR")).to_path_buf();
    let export = image(false);
    let japan = image(true);
    assert_eq!(export.1, japan.1, "both builds place the same symbols");
    std::fs::write(out.join("open_boot.bin"), export.0).expect("the image written");
    std::fs::write(out.join("open_boot_japan.bin"), japan.0).expect("the image written");
    std::fs::write(out.join("open_boot_symbols.rs"), export.1).expect("the symbols written");
}

/// The image for an export Master System, or with `japan` for the Japanese one, and its symbols' source.
fn image(japan: bool) -> (Vec<u8>, String) {
    let dir = Path::new("src/boot");
    let read = |name: &str| std::fs::read_to_string(dir.join(name)).ok();
    let source = |name: &str| read(name).unwrap_or_else(|| panic!("{name} is missing"));
    let ram = build("ram.s", assemble_with(&source("ram.s"), RAM_PROGRAM, &[("ROM_NO_GAME", ROM_NO_GAME), ("ROM_REFUSE", ROM_REFUSE), ("JAPAN", japan as i64)], &read));
    assert_eq!(ram.origin, RAM_PROGRAM);
    assert!(ram.bytes.len() <= IMAGE - RAM_AT, "the RAM program runs past the image");
    assert_eq!(ram.symbols["ram_end"] as usize, RAM_PROGRAM as usize + ram.bytes.len(), "the RAM program's variables end it");
    let defines = [("FONT_AT", FONT_AT as i64), ("RAM_AT", RAM_AT as i64), ("RAM_LEN", ram.bytes.len() as i64)];
    let rom = build("rom.s", assemble_with(&source("rom.s"), 0, &defines, &read));
    assert!(rom.bytes.len() <= FONT_AT, "the ROM program runs into the font");
    assert_eq!((rom.symbols["no_game"], rom.symbols["refuse"]), (rom_target(&rom.bytes, ROM_NO_GAME), rom_target(&rom.bytes, ROM_REFUSE)));
    let font = font();
    let mut image = vec![0xFF; IMAGE];
    image[..rom.bytes.len()].copy_from_slice(&rom.bytes);
    image[FONT_AT..FONT_AT + font.len()].copy_from_slice(&font);
    image[RAM_AT..RAM_AT + ram.bytes.len()].copy_from_slice(&ram.bytes);
    let symbols = format!(
        "/// Where the RAM program runs and how long it is, and the ROM's wait after a refusal, from the assembly.\npub const RAM_PROGRAM: u16 = {RAM_PROGRAM:#06x};\npub const RAM_LEN: usize = {:#x};\npub const RAM_AT: usize = {RAM_AT:#x};\npub const WAIT: u16 = {:#06x};\n",
        ram.bytes.len(),
        rom.symbols["wait"]
    );
    (image, symbols)
}

fn build(name: &str, p: Result<Program, beryl_z80::asm::AsmError>) -> Program {
    p.unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// The address a `JP nn` at `at` jumps to.
fn rom_target(rom: &[u8], at: i64) -> i64 {
    let at = at as usize;
    assert_eq!(rom[at], 0xC3, "a jump at {at:#06X}");
    i64::from(u16::from_le_bytes([rom[at + 1], rom[at + 2]]))
}

/// `beryl-font`'s glyphs as 64 Master System tiles, colour 1 on 0: each row's five dots from the tile's second pixel in
/// bitplane 0, the other three planes clear, the seven rows from the tile's first.
fn font() -> Vec<u8> {
    let mut tiles = vec![0u8; 64 * 32];
    for g in beryl_font::glyphs() {
        let tile = (g.char - 32) as usize;
        for (row, bits) in g.rows.into_iter().enumerate() {
            tiles[tile * 32 + row * 4] = bits << 2;
        }
    }
    tiles
}
