//! The Master System's and the SG-1000's debugger views, from the spaces and units any engine of the two consoles
//! publishes: the twin of `SmsDebugViews`, reading the same bytes in the same order (Endou_Native.md §15.5). A view the
//! two consoles lay out differently, the tiles, the name table and the sprites, takes the system: mode 4 on the Master
//! System, the TMS9918's on the SG-1000.

use beryl_sms_colours::{SMS_COLOURS, SMS1_COLOURS, TMS_COLOURS};

use crate::debug::{Memory, Unit, find, register, unit};

pub const MASTER_SYSTEM: &str = "sms";
pub const SG1000: &str = "sg1000";
pub const VRAM: &str = "VRAM";
pub const CRAM: &str = "CRAM";
pub const RAM: &str = "RAM";
pub const ROM: &str = "ROM";

/// The units an engine of the two consoles lists, by the names the views read them under (Endou_Native.md §15.4).
pub const VDP: &str = "VDP";
pub const PSG: &str = "PSG";
pub const BOARD: &str = "Board";
pub const IO: &str = "IO";

/// The VDP unit's `Variant`: the 315-5124, whose first blue level differs, and the TMS9918A, which has no CRAM.
pub const VARIANT_315_5124: u64 = 1;

pub const CHANNELS: [&str; 4] = ["Tone 1", "Tone 2", "Tone 3", "Noise"];

/// The boards by the `Board` unit's `Mapper`.
pub const BOARDS: [&str; 4] = ["Plain", "Sega", "Codemasters", "SG-1000 RAM"];

fn sg(system: &str) -> bool {
    system == SG1000
}

fn r(u: Option<&Unit>, name: &str) -> u64 {
    u.and_then(|u| register(&u.registers, name)).unwrap_or(0)
}

/// A space of at least `size` bytes, by the name the engine spells it with.
fn space(memory: &dyn Memory, name: &str, size: i32) -> Option<String> {
    find(memory, name).filter(|s| s.size >= size).map(|s| s.name.clone())
}

fn rgb(c: [u8; 4]) -> (u8, u8, u8) {
    (c[0], c[1], c[2])
}

/// CRAM's two palettes of sixteen, the background's then the sprites', in the colours of the VDP unit's chip (the
/// 315-5246's without one); on a console with VRAM and no CRAM, the TMS9918A's sixteen fixed colours.
pub fn palettes(memory: &mut dyn Memory, units: &[Unit]) -> Vec<(i32, Vec<(u8, u8, u8)>)> {
    let Some(cram) = space(memory, CRAM, 32) else {
        return if find(memory, VRAM).is_some() { vec![(0, TMS_COLOURS.iter().map(|&c| rgb(c)).collect())] } else { Vec::new() };
    };
    let table = if r(unit(units, VDP), "Variant") == VARIANT_315_5124 { &SMS1_COLOURS } else { &SMS_COLOURS };
    let addresses: Vec<i32> = (0..32).collect();
    let entries = memory.batch(&cram, &addresses);
    (0..2).map(|p| (p as i32, entries[p * 16..p * 16 + 16].iter().map(|&e| rgb(table[(e & 0x3F) as usize])).collect())).collect()
}

/// Each palette a row of sixteen 16-pixel swatches.
pub fn palette_swatch(memory: &mut dyn Memory, units: &[Unit]) -> (Vec<u8>, i32, i32) {
    let palettes = palettes(memory, units);
    if palettes.is_empty() {
        return (Vec::new(), 0, 0);
    }
    const SWATCH: usize = 16;
    const COLUMNS: usize = 16;
    let (width, height) = (COLUMNS * SWATCH, palettes.len() * SWATCH);
    let mut rgba = vec![0u8; width * height * 4];
    for (p, (_, colours)) in palettes.iter().enumerate() {
        for (c, &(r, g, b)) in colours.iter().enumerate() {
            for y in 0..SWATCH {
                for x in 0..SWATCH {
                    let o = ((p * SWATCH + y) * width + c * SWATCH + x) * 4;
                    rgba[o..o + 4].copy_from_slice(&[r, g, b, 0xFF]);
                }
            }
        }
    }
    (rgba, width as i32, height as i32)
}

/// Every tile of VRAM, grey by pixel value: on the Master System its 512 of mode 4, 32 bytes of four planes each,
/// sixteen a row; on the SG-1000 every 8 bytes as a pattern of one plane, thirty-two a row.
pub fn tile_sheet(memory: &mut dyn Memory, system: &str) -> (Vec<u8>, i32, i32) {
    let Some(vram) = space(memory, VRAM, 1) else { return (Vec::new(), 0, 0) };
    let size = find(memory, VRAM).map_or(0, |s| s.size) as usize;
    let (bytes_per, per_row) = if sg(system) { (8, 32) } else { (32, 16) };
    let tiles = size / bytes_per;
    let (width, height) = (per_row * 8, tiles.div_ceil(per_row) * 8);
    let mut rgba = vec![0u8; width * height * 4];
    let bytes = memory.run(&vram, tiles * bytes_per);
    for tile in 0..tiles {
        let (x0, y0) = (tile % per_row * 8, tile / per_row * 8);
        for row in 0..8 {
            for column in 0..8 {
                let shade = if sg(system) {
                    if bytes[tile * 8 + row] << column & 0x80 != 0 { 0xFF } else { 0 }
                } else {
                    planar(&bytes[tile * 32 + row * 4..tile * 32 + row * 4 + 4], column) * 17
                };
                let o = ((y0 + row) * width + x0 + column) * 4;
                rgba[o..o + 4].copy_from_slice(&[shade, shade, shade, 0xFF]);
            }
        }
    }
    (rgba, width as i32, height as i32)
}

/// A mode 4 pixel from its row's four plane bytes, plane 0 the lowest bit.
fn planar(planes: &[u8], column: usize) -> u8 {
    (0..4).fold(0, |v, p| v | (planes[p] >> (7 - column) & 1) << p)
}

/// Mode 4's name table entries are words; the TMS9918's are bytes.
pub fn tilemap_entry_stride(system: &str) -> i32 {
    if sg(system) { 1 } else { 2 }
}

/// A name table entry: on the Master System the word at `address`, low byte first, as its tile, palette, flips and
/// priority; on the SG-1000 the pattern's name.
pub fn decode_tilemap_entry(memory: &mut dyn Memory, space: &str, address: i32, system: &str) -> String {
    if sg(system) {
        return format!("${:02X}", memory.read(space, address));
    }
    let both = memory.batch(space, &[address, address.wrapping_add(1)]);
    let w = both[0] as u16 | (both[1] as u16) << 8;
    let flips = format!("{}{}", if w & 0x200 != 0 { "X" } else { "-" }, if w & 0x400 != 0 { "Y" } else { "-" });
    format!("${:03X} p{} {flips}{}", w & 0x1FF, (w >> 11) & 1, if w & 0x1000 != 0 { " pri" } else { "" })
}

/// A tile's 64 pixels: mode 4's four planes, or the TMS9918's one; another depth refused with the C# words.
pub fn decode_tile_pixels(memory: &mut dyn Memory, space: &str, address: i32, bpp: i32, system: &str) -> Result<Vec<u8>, (String, Option<&'static str>)> {
    let (depth, length) = if sg(system) { (1, 8) } else { (4, 32) };
    if bpp != depth {
        let words = if sg(system) { format!("An SG-1000 pattern is always 1bpp; got {bpp}.") } else { format!("A Master System tile is always 4bpp; got {bpp}.") };
        return Err((words, Some("bpp")));
    }
    let addresses: Vec<i32> = (0..length).map(|i| address.wrapping_add(i)).collect();
    let bytes = memory.batch(space, &addresses);
    let mut pixels = vec![0u8; 64];
    for row in 0..8 {
        for column in 0..8 {
            pixels[row * 8 + column] = if sg(system) { bytes[row] >> (7 - column) & 1 } else { planar(&bytes[row * 4..row * 4 + 4], column) };
        }
    }
    Ok(pixels)
}

/// The sprite table's capacity: mode 4's 64, the TMS9918's 32.
pub fn max_sprites(system: &str) -> i32 {
    if sg(system) { 32 } else { 64 }
}

/// A sprite as `DebugSpriteInfo` holds it: index, x, y, width, height, tile, palette, priority, flip x, flip y.
pub type Sprite = (i32, i32, i32, i32, i32, i32, i32, i32, bool, bool);

/// The sprite table the VDP unit's registers point at, to the $D0 that ends it where the chip honours one: mode 4's
/// on the Master System (Y at the table's start, X and the tile from $80, the sprites' palette, register 6's tile
/// bank), the TMS9918's on the SG-1000 (four bytes each, the colour as the palette, the early clock's 32 pixels). A
/// sprite shows from the line after its Y; one wholly below the active lines is hidden, and one past line 255 starts
/// above the first.
pub fn sprites(memory: &mut dyn Memory, units: &[Unit], system: &str) -> Vec<Sprite> {
    let Some(vdp) = unit(units, VDP) else { return Vec::new() };
    let Some(vram) = space(memory, VRAM, 0x4000) else { return Vec::new() };
    let v = Some(vdp);
    let (r0, r1, r5, r6, lines) = (r(v, "R0"), r(v, "R1"), r(v, "R5"), r(v, "R6"), r(v, "Lines").max(1) as i32);
    let size = if r1 & 2 != 0 { 16 } else { 8 };
    let zoom = if r1 & 1 != 0 { 2 } else { 1 };
    let mut out = Vec::new();
    if sg(system) {
        let base = ((r5 & 0x7F) << 7) as i32;
        let addresses: Vec<i32> = (0..128).map(|i| base + i).collect();
        let table = memory.batch(&vram, &addresses);
        for i in 0..32 {
            let e = &table[i * 4..i * 4 + 4];
            if e[0] == 0xD0 {
                break;
            }
            let Some(y) = shown_y(e[0], size * zoom, lines) else { continue };
            let tile = if size == 16 { e[2] & 0xFC } else { e[2] } as i32;
            let x = e[1] as i32 - if e[3] & 0x80 != 0 { 32 } else { 0 };
            out.push((i as i32, x, y, size * zoom, size * zoom, tile, (e[3] & 0x0F) as i32, 1, false, false));
        }
        return out;
    }
    let base = ((r5 & 0x7E) << 7) as i32;
    let addresses: Vec<i32> = (0..256).map(|i| base + i).collect();
    let table = memory.batch(&vram, &addresses);
    let height = size * zoom;
    for i in 0..64 {
        let y = table[i];
        if lines == 192 && y == 0xD0 {
            break;
        }
        let Some(y) = shown_y(y, height, lines) else { continue };
        let x = table[0x80 + 2 * i] as i32 - if r0 & 8 != 0 { 8 } else { 0 };
        let tile = table[0x81 + 2 * i] as i32 | ((r6 & 4) << 6) as i32;
        out.push((i as i32, x, y, 8 * zoom, height, tile, 1, 1, false, false));
    }
    out
}

/// The first line a sprite of `height` lines at `y` shows on, none for one wholly below the active lines.
fn shown_y(y: u8, height: i32, lines: i32) -> Option<i32> {
    let top = y as i32 + 1;
    if top + height > 256 {
        return Some(top - 256);
    }
    (top < lines).then_some(top)
}

/// A channel as `DebugAudioChannelInfo` holds it: index, name, active, level, muted, the line.
pub type Channel = (i32, &'static str, bool, i32, bool, String);

fn shown(b: bool) -> &'static str {
    if b { "True" } else { "False" }
}

/// The PSG's three tones and its noise from the PSG unit: a channel sounds while its attenuation is under 15, its level
/// the attenuation's complement out of 100 by whole steps.
pub fn audio_channels(units: &[Unit], mutes: u32) -> Vec<Channel> {
    let Some(psg) = unit(units, PSG) else { return Vec::new() };
    let p = Some(psg);
    CHANNELS
        .iter()
        .enumerate()
        .map(|(i, &name)| {
            let att = r(p, &format!("Volume{i}"));
            let level = ((15 - att.min(15)) * 100 / 15) as i32;
            let line = if i < 3 {
                format!("Period={} Att={att}", r(p, &format!("Tone{i}")))
            } else {
                let noise = r(p, "Noise");
                format!("Rate={} White={} Att={att}", noise & 3, shown(noise & 4 != 0))
            };
            (i as i32, name, att < 15, level, mutes & (1 << i) != 0, line)
        })
        .collect()
}

/// A cartridge's board by the `Board` unit's `Mapper`; "?" for a number no board is named for; none for another unit.
pub fn unit_label(u: &Unit) -> Option<String> {
    if u.kind != "cartridge" {
        return None;
    }
    let mapper = register(&u.registers, "Mapper")?;
    Some(BOARDS.get(mapper as usize).copied().unwrap_or("?").to_string())
}

/// Where a Z80 address lies, for the range the console fixes: $C000 and up is work RAM and its mirrors, or the
/// cartridge's RAM where Sega's mapper puts it there, and nothing while port $3E disables work RAM; below, the board,
/// the BIOS and port $3E decide, and none is given.
pub fn resolve_physical(address: i32, memory: &dyn Memory, units: &[Unit]) -> Option<(&'static str, i32, bool)> {
    if !(0xC000..=0xFFFF).contains(&address) {
        return None;
    }
    let board = unit(units, BOARD);
    let cart_ram = r(board, "Ram") as i32;
    if r(board, "Mapper") == 1 && r(board, "Control") & 0x10 != 0 && cart_ram > 0 {
        return Some(("CARTRAM", (address & 0x3FFF) % cart_ram.min(0x4000), true));
    }
    if r(unit(units, IO), "MemoryControl") & 0x10 != 0 {
        return None;
    }
    let size = find(memory, RAM).map_or(0, |s| s.size);
    (size > 0).then(|| ("RAM", (address - 0xC000) % size, true))
}

/// The board from its unit and the ROM's size, then the Z80, the VDP, the PSG and the I/O chip's ports.
pub fn summary(registers: &[(String, u64)], memory: &mut dyn Memory, units: &[Unit]) -> Option<String> {
    register(registers, "PC")?;
    let vdp = Some(unit(units, VDP)?);
    let c = |n: &str| register(registers, n).unwrap_or(0);
    let mut text = String::new();
    if let Some(board) = unit(units, BOARD) {
        let rom = find(memory, ROM).map_or(0, |s| s.size) / 1024;
        let ram = r(Some(board), "Ram") / 1024;
        let label = unit_label(board).unwrap_or_default();
        text.push_str(&format!("Board {label}, {rom} KiB ROM{}{}\n", if ram > 0 { format!(", {ram} KiB RAM") } else { String::new() }, if r(Some(board), "Battery") != 0 { ", battery" } else { "" }));
    }
    text.push_str(&format!(
        "Z80  AF={:04X} BC={:04X} DE={:04X} HL={:04X} IX={:04X} IY={:04X} SP={:04X} PC={:04X}\n",
        c("AF"),
        c("BC"),
        c("DE"),
        c("HL"),
        c("IX"),
        c("IY"),
        c("SP"),
        c("PC")
    ));
    text.push_str(&format!("     I={:02X} R={:02X} IM={} IFF1={} IFF2={}\n", c("I"), c("R"), c("IM"), c("IFF1"), c("IFF2")));
    text.push_str(&format!(
        "VDP  R0={:02X} R1={:02X} mode {}, display {}, line {} of {}, status={:02X}",
        r(vdp, "R0"),
        r(vdp, "R1"),
        if r(vdp, "Mode4") != 0 { "4" } else { "TMS" },
        if r(vdp, "R1") & 0x40 != 0 { "on" } else { "off" },
        r(vdp, "Line"),
        r(vdp, "Lines"),
        r(vdp, "Status")
    ));
    if let Some(psg) = unit(units, PSG) {
        let p = Some(psg);
        text.push_str(&format!(
            "\nPSG  tones {:03X} {:03X} {:03X} noise {:X} att {:X}{:X}{:X}{:X}",
            r(p, "Tone0"),
            r(p, "Tone1"),
            r(p, "Tone2"),
            r(p, "Noise"),
            r(p, "Volume0"),
            r(p, "Volume1"),
            r(p, "Volume2"),
            r(p, "Volume3")
        ));
    }
    if let Some(io) = unit(units, IO) {
        text.push_str(&format!("\nIO   $3E={:02X} $3F={:02X}", r(Some(io), "MemoryControl"), r(Some(io), "IoControl")));
    }
    Some(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::debug::Space;

    struct Mem(Vec<Space>, Vec<Vec<u8>>);

    impl Memory for Mem {
        fn spaces(&self) -> &[Space] {
            &self.0
        }

        fn read(&mut self, space: &str, address: i32) -> u8 {
            let i = self.0.iter().position(|s| s.name == space).unwrap();
            self.1[i].get(address as usize).copied().unwrap_or(0)
        }
    }

    fn mem(spaces: &[(&str, Vec<u8>)]) -> Mem {
        Mem(spaces.iter().map(|(n, d)| Space { name: n.to_string(), size: d.len() as i32, side_effects: false }).collect(), spaces.iter().map(|s| s.1.clone()).collect())
    }

    fn vdp(registers: &[(&str, u64)]) -> Unit {
        Unit { name: VDP.into(), kind: "video".into(), registers: registers.iter().map(|&(n, v)| (n.to_string(), v)).collect() }
    }

    // CRAM's two palettes in the chip's colours: the 315-5124's first blue level, the 315-5246's otherwise; the TMS9918A's sixteen without CRAM.
    #[test]
    fn the_palettes_are_cram_s_in_the_chip_s_colours() {
        let mut cram = vec![0u8; 32];
        cram[1] = 0x10;
        cram[17] = 0x3F;
        let mut m = mem(&[("VRAM", vec![0; 0x4000]), ("CRAM", cram)]);
        let p = palettes(&mut m, &[vdp(&[("Variant", 1)])]);
        assert_eq!((p.len(), p[0].1[1], p[1].1[1]), (2, (0, 0, 0x68), (255, 255, 255)));
        assert_eq!(palettes(&mut m, &[])[0].1[1], (0, 0, 0x55), "the 315-5246 without a unit");
        let mut sg = mem(&[("VRAM", vec![0; 0x4000])]);
        assert_eq!((palettes(&mut sg, &[])[0].1.len(), palettes(&mut sg, &[])[0].1[15]), (16, (255, 255, 255)));
        assert!(palettes(&mut mem(&[]), &[]).is_empty());
    }

    // Mode 4's table ends at $D0 in the 192-line mode, its tile bank from register 6, its X shifted by register 0's bit 3; the TMS9918's from register 5 with its colour and early clock.
    #[test]
    fn the_sprites_are_the_table_the_registers_point_at() {
        let mut vram = vec![0u8; 0x4000];
        vram[0x3F00..0x3F03].copy_from_slice(&[0x4F, 0xF8, 0xD0]);
        vram[0x3F80..0x3F84].copy_from_slice(&[0x64, 0x02, 0x10, 0x03]);
        let mut m = mem(&[("VRAM", vram.clone())]);
        let regs = [("R0", 8), ("R1", 0x02), ("R5", 0xFF), ("R6", 0xFF), ("Lines", 192)];
        assert_eq!(sprites(&mut m, &[vdp(&regs)], MASTER_SYSTEM), [(0, 0x5C, 0x50, 8, 16, 0x102, 1, 1, false, false), (1, 0x08, -7, 8, 16, 0x103, 1, 1, false, false)]);
        let regs = [("R0", 0), ("R1", 0x01), ("R5", 0xFF), ("R6", 0), ("Lines", 224)];
        let tall = sprites(&mut m, &[vdp(&regs)], MASTER_SYSTEM);
        assert_eq!((tall.len(), tall[2].2, tall[2].4), (64, 0xD1, 16), "no terminator at 224 lines: the $D0 shows from line 209, zoomed");
        let regs = [("R0", 0), ("R1", 0), ("R5", 0xFF), ("R6", 0), ("Lines", 240)];
        vram_hidden(&mut m, &regs);
        vram[0x1000..0x1008].copy_from_slice(&[0x10, 0x20, 0x07, 0x8C, 0xD0, 0, 0, 0]);
        let mut sg = mem(&[("VRAM", vram)]);
        assert_eq!(sprites(&mut sg, &[vdp(&[("R1", 0x02), ("R5", 0x20), ("Lines", 192)])], SG1000), [(0, 0x00, 0x11, 16, 16, 0x04, 0x0C, 1, false, false)]);
        assert!(sprites(&mut sg, &[], SG1000).is_empty());
    }

    /// A sprite wholly below the active lines is hidden; one past line 255 starts above the first.
    fn vram_hidden(m: &mut Mem, regs: &[(&str, u64)]) {
        m.1[0][0x3F00] = 0xF0;
        m.1[0][0x3F01] = 0xFA;
        let s = sprites(m, &[vdp(regs)], MASTER_SYSTEM);
        assert_eq!((s.len(), s[0].0, s[0].2, s[1].2), (63, 1, -5, 0xD1), "$F0 starts at line 241, wholly below 240 and hidden; $FA's eight lines pass 255 and start at -5");
        let s = sprites(m, &[vdp(&[("R1", 0), ("R5", 0xFF), ("Lines", 192)])], MASTER_SYSTEM);
        assert_eq!(s.len(), 1, "at 192 lines $F0 is below them, and the $D0 at sprite 2 ends the table");
    }

    // A word entry's tile, palette, flips and priority; a pattern's name; four planes and one, another depth refused.
    #[test]
    fn the_name_table_and_the_tiles_decode_by_console() {
        let mut vram = vec![0u8; 0x4000];
        vram[0x3802..0x3804].copy_from_slice(&[0x05, 0x1A]);
        vram[0x40..0x44].copy_from_slice(&[0x80, 0x80, 0x00, 0x80]);
        let mut m = mem(&[("VRAM", vram)]);
        assert_eq!(decode_tilemap_entry(&mut m, "VRAM", 0x3802, MASTER_SYSTEM), "$005 p1 X- pri");
        assert_eq!(decode_tilemap_entry(&mut m, "VRAM", 0x3802, SG1000), "$05");
        assert_eq!(decode_tile_pixels(&mut m, "VRAM", 0x40, 4, MASTER_SYSTEM).unwrap()[..2], [0b1011, 0]);
        assert_eq!(decode_tile_pixels(&mut m, "VRAM", 0x40, 1, SG1000).unwrap()[..9], [1, 0, 0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(decode_tile_pixels(&mut m, "VRAM", 0, 2, MASTER_SYSTEM).unwrap_err().0, "A Master System tile is always 4bpp; got 2.");
        assert_eq!((tilemap_entry_stride(MASTER_SYSTEM), tilemap_entry_stride(SG1000), max_sprites(MASTER_SYSTEM), max_sprites(SG1000)), (2, 1, 64, 32));
        assert_eq!(tile_sheet(&mut m, MASTER_SYSTEM).1..=tile_sheet(&mut m, MASTER_SYSTEM).2, 128..=256);
        assert_eq!((tile_sheet(&mut m, SG1000).1, tile_sheet(&mut m, SG1000).2), (256, 512));
    }

    // The PSG's channels from its unit; the board's label; the fixed range of the address map.
    #[test]
    fn the_channels_the_board_and_the_map_read_their_units() {
        let psg = Unit { name: PSG.into(), kind: "audio".into(), registers: [("Tone0", 0xFE), ("Volume0", 0), ("Volume1", 15), ("Volume2", 5), ("Volume3", 2), ("Noise", 6)].iter().map(|&(n, v)| (n.to_string(), v)).collect() };
        let c = audio_channels(&[psg], 0b1000);
        assert_eq!(c[0], (0, "Tone 1", true, 100, false, "Period=254 Att=0".to_string()));
        assert_eq!((c[1].2, c[2].3), (false, 66));
        assert_eq!(c[3], (3, "Noise", true, 86, true, "Rate=2 White=True Att=2".to_string()));
        let board = |mapper: u64, control: u64| Unit { name: BOARD.into(), kind: "cartridge".into(), registers: vec![("Mapper".into(), mapper), ("Control".into(), control), ("Ram".into(), 0x8000)] };
        assert_eq!((unit_label(&board(2, 0)).as_deref(), unit_label(&board(9, 0)).as_deref(), unit_label(&vdp(&[]))), (Some("Codemasters"), Some("?"), None));
        let m = mem(&[("RAM", vec![0; 0x2000])]);
        assert_eq!((resolve_physical(0xE123, &m, &[]), resolve_physical(0xBFFF, &m, &[])), (Some(("RAM", 0x0123, true)), None));
        assert_eq!(resolve_physical(0xC005, &m, &[board(1, 0x10)]), Some(("CARTRAM", 5, true)));
        let off = Unit { name: IO.into(), kind: "io".into(), registers: vec![("MemoryControl".into(), 0xBB)] };
        assert_eq!(resolve_physical(0xC000, &m, &[off]), None, "work RAM disabled by port $3E");
    }
}
