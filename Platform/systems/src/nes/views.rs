//! The NES's debugger views, decoded from the spaces any NES engine publishes (Moon_Debug.md §4, EmuSen_CoreAPI.md §31).

use crate::debug::{Memory, find};

pub const PALETTE: &str = "PALETTE";
pub const CHR: &str = "CHR";
pub const NMI_VECTOR: i32 = 0xFFFA;
pub const RESET_VECTOR: i32 = 0xFFFC;
pub const IRQ_VECTOR: i32 = 0xFFFE;

/// The 64 NTSC colours as RGB triples, emphasis not applied; the debugger's table, not the picture's (Moon_PPU.md §3.5).
pub const MASTER_PALETTE: [u8; 192] = [
    84, 84, 84, 0, 30, 116, 8, 16, 144, 48, 0, 136, 68, 0, 100, 92, 0, 48, 84, 4, 0, 60, 24, 0,
    32, 42, 0, 8, 58, 0, 0, 64, 0, 0, 60, 0, 0, 50, 60, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    152, 150, 152, 8, 76, 196, 48, 50, 236, 92, 30, 228, 136, 20, 176, 160, 20, 100, 152, 34, 32, 120, 60, 0,
    84, 90, 0, 40, 114, 0, 8, 124, 0, 0, 118, 40, 0, 102, 120, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    236, 238, 236, 76, 154, 236, 120, 124, 236, 176, 98, 236, 228, 84, 236, 236, 88, 180, 236, 106, 100, 212, 136, 32,
    160, 170, 0, 116, 196, 0, 76, 208, 32, 56, 204, 108, 56, 180, 204, 60, 60, 60, 0, 0, 0, 0, 0, 0,
    236, 238, 236, 168, 204, 236, 188, 188, 236, 212, 178, 236, 236, 174, 236, 236, 174, 212, 236, 180, 176, 228, 196, 144,
    204, 210, 120, 180, 222, 120, 168, 226, 144, 152, 226, 180, 160, 214, 228, 160, 162, 160, 0, 0, 0, 0, 0, 0,
];

/// $3F10, $3F14, $3F18 and $3F1C are the backdrop entries below them.
pub fn palette_offset(entry: i32) -> i32 {
    let index = entry & 0x1F;
    if index & 0x13 == 0x10 { index & 0x0F } else { index }
}

/// The colours of entries, read in their order as one batch.
fn colours(memory: &mut dyn Memory, palette: &str, entries: impl Iterator<Item = i32>) -> Vec<(u8, u8, u8)> {
    let addresses: Vec<i32> = entries.map(palette_offset).collect();
    memory
        .batch(palette, &addresses)
        .into_iter()
        .map(|b| {
            let offset = (b & 0x3F) as usize * 3;
            (MASTER_PALETTE[offset], MASTER_PALETTE[offset + 1], MASTER_PALETTE[offset + 2])
        })
        .collect()
}

/// A space of the name with some size, as the views ask for one: its name as the engine spells it.
fn sized(memory: &dyn Memory, name: &str) -> Option<String> {
    find(memory, name).filter(|s| s.size > 0).map(|s| s.name.clone())
}

/// The eight palettes of four colours.
pub fn palettes(memory: &mut dyn Memory) -> Vec<(i32, Vec<(u8, u8, u8)>)> {
    let Some(palette) = sized(memory, PALETTE) else { return Vec::new() };
    let all = colours(memory, &palette, 0..32);
    (0..8).map(|p| (p as i32, all[p * 4..p * 4 + 4].to_vec())).collect()
}

/// The 32 entries as 16-pixel swatches, sixteen a row.
pub fn palette_swatch(memory: &mut dyn Memory) -> (Vec<u8>, i32, i32) {
    let Some(palette) = sized(memory, PALETTE) else { return (Vec::new(), 0, 0) };
    const SWATCH: usize = 16;
    const COLUMNS: usize = 16;
    let (width, height) = (COLUMNS * SWATCH, 2 * SWATCH);
    let mut rgba = vec![0u8; width * height * 4];
    let all = colours(memory, &palette, 0..32);
    for (entry, &(r, g, b)) in all.iter().enumerate() {
        let (x0, y0) = (entry % COLUMNS * SWATCH, entry / COLUMNS * SWATCH);
        for y in 0..SWATCH {
            for x in 0..SWATCH {
                let o = ((y0 + y) * width + x0 + x) * 4;
                rgba[o..o + 4].copy_from_slice(&[r, g, b, 0xFF]);
            }
        }
    }
    (rgba, width as i32, height as i32)
}

/// Every tile of CHR, sixteen a row, grey by pixel value, since a tile has no palette until a nametable picks one. A
/// space without side effects is read whole; one with them byte by byte in the order the C# view read it.
pub fn tile_sheet(memory: &mut dyn Memory) -> (Vec<u8>, i32, i32) {
    let Some(chr) = find(memory, CHR).filter(|s| s.size > 0).cloned() else { return (Vec::new(), 0, 0) };
    const PER_ROW: usize = 16;
    let tiles = (chr.size / 16) as usize;
    let rows = tiles.div_ceil(PER_ROW);
    let (width, height) = (PER_ROW * 8, rows * 8);
    let mut rgba = vec![0u8; width * height * 4];
    // The bytes in the order the C# view read them, unless no read disturbs the space, when it is read whole.
    let order: Vec<i32> = (0..tiles * 16).map(|k| ((k / 16) * 16 + (k % 16) / 2 + (k % 2) * 8) as i32).collect();
    let read = if chr.side_effects { memory.batch(&chr.name, &order) } else { memory.run(&chr.name, tiles * 16) };
    let byte = |tile: usize, row: usize, high: bool| if chr.side_effects { read[tile * 16 + row * 2 + high as usize] } else { read[tile * 16 + row + if high { 8 } else { 0 }] };
    const SHADES: [u8; 4] = [0, 85, 170, 255];
    for tile in 0..tiles {
        let (x0, y0) = (tile % PER_ROW * 8, tile / PER_ROW * 8);
        for row in 0..8 {
            let (low, high) = (byte(tile, row, false), byte(tile, row, true));
            for column in 0..8 {
                let bit = 7 - column;
                let shade = SHADES[((low >> bit) & 1 | ((high >> bit) & 1) << 1) as usize];
                let o = ((y0 + row) * width + x0 + column) * 4;
                rgba[o..o + 4].copy_from_slice(&[shade, shade, shade, 0xFF]);
            }
        }
    }
    (rgba, width as i32, height as i32)
}

/// One byte per nametable cell; the attribute byte is separate and coarser.
pub const TILEMAP_ENTRY_STRIDE: i32 = 1;

/// A nametable cell: its tile and the palette its attribute byte gives it, or the attribute byte itself at 0x3C0 and up.
pub fn decode_tilemap_entry(memory: &mut dyn Memory, space: &str, address: i32) -> String {
    let (page, offset) = ((address / 0x400) & 3, address % 0x400);
    if offset >= 0x3C0 {
        return format!("attr ${:02X}", memory.read(space, address));
    }
    let (x, y) = (offset % 32, offset / 32);
    let read = memory.batch(space, &[address, page * 0x400 + 0x3C0 + y / 4 * 8 + x / 4]);
    let (tile, attribute) = (read[0], read[1]);
    let quadrant = ((y & 2) << 1) | (x & 2);
    format!("${tile:02X} p{}", (attribute >> quadrant) & 3)
}

/// 16 bytes per tile: eight rows of the low bitplane, then eight of the high one; a depth other than 2 refused with the
/// C# ArgumentException's words and parameter.
pub fn decode_tile_pixels(memory: &mut dyn Memory, space: &str, address: i32, bpp: i32) -> Result<Vec<u8>, (String, Option<&'static str>)> {
    if bpp != 2 {
        return Err((format!("An NES tile is always 2bpp; got {bpp}."), Some("bpp")));
    }
    let mut pixels = vec![0u8; 64];
    let addresses: Vec<i32> = (0..8).flat_map(|row| [address + row, address + row + 8]).collect();
    let read = memory.batch(space, &addresses);
    for row in 0..8 {
        let (low, high) = (read[row as usize * 2], read[row as usize * 2 + 1]);
        for column in 0..8 {
            let bit = 7 - column;
            pixels[(row * 8 + column) as usize] = ((low >> bit) & 1) | (((high >> bit) & 1) << 1);
        }
    }
    Ok(pixels)
}

/// Where each vector's pointer lives.
pub const VECTORS: [(&str, i32, i32, &str); 3] = [("NMI", NMI_VECTOR, 2, ""), ("RESET", RESET_VECTOR, 2, ""), ("IRQ/BRK", IRQ_VECTOR, 2, "")];

/// Where a CPU address lies, for the ranges the console fixes: the space, the offset in it and whether a debugger reads
/// it there; `None` for `$8000` and up, which the board banks (EmuSen_Backend.md §19.3.3, D13).
pub fn resolve_physical(cpu_address: i32) -> Option<(&'static str, i32, bool)> {
    match cpu_address {
        a if a < 0x2000 => Some(("RAM", a & 0x07FF, true)),
        a if a < 0x4000 => Some(("PPUREG", a & 0x07, false)),
        a if a < 0x4020 => Some(("APUREG", a - 0x4000, false)),
        a if a < 0x8000 => Some(("PRGRAM", a & 0x1FFF, true)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_physical;

    // The fixed ranges, their mirrors and their edges, and nothing for the banked half.
    #[test]
    fn the_fixed_ranges_resolve_and_the_banked_half_does_not() {
        assert_eq!(resolve_physical(0x0000), Some(("RAM", 0, true)));
        assert_eq!(resolve_physical(0x1FFF), Some(("RAM", 0x07FF, true)));
        assert_eq!(resolve_physical(0x2000), Some(("PPUREG", 0, false)));
        assert_eq!(resolve_physical(0x3FFF), Some(("PPUREG", 7, false)));
        assert_eq!(resolve_physical(0x4015), Some(("APUREG", 0x15, false)));
        assert_eq!(resolve_physical(0x4020), Some(("PRGRAM", 0x0020, true)));
        assert_eq!(resolve_physical(0x7FFF), Some(("PRGRAM", 0x1FFF, true)));
        assert_eq!(resolve_physical(0x8000), None);
        assert_eq!(resolve_physical(0xFFFF), None);
    }
}
