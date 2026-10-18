//! The Game Boy's and the Color's debugger views, from the spaces and units any Game Boy engine publishes: the twin of
//! `GameBoyDebugViews`, reading the same bytes in the same order (Mercury_Debug.md §3, EmuSen_Backend.md §17.2).

use super::cartridge;
use crate::debug::{Memory, Unit, find, register, unit};

pub const VRAM: &str = "VRAM";
pub const OAM: &str = "OAM";
pub const ROM: &str = "ROM";
pub const BG_PALETTES: &str = "BGPAL";
pub const OBJ_PALETTES: &str = "OBJPAL";
pub const SPRITE_COUNT: i32 = 40;
pub const TILEMAP_ENTRY_STRIDE: i32 = 1;
const VRAM_BANK: i32 = 0x2000;
const WRAM_BANK: i32 = 0x1000;
const SCREEN_HEIGHT: i32 = 144;

/// The four greys a Game Boy picture is shown in, lightest first.
pub const SHADES: [u8; 4] = [0xFF, 0xAA, 0x55, 0x00];

pub const CHANNELS: [&str; 4] = ["Pulse 1", "Pulse 2", "Wave", "Noise"];

pub const VECTORS: [(&str, i32, i32, &str); 5] = [("VBlank", 0x40, 2, ""), ("LCD STAT", 0x48, 2, ""), ("Timer", 0x50, 2, ""), ("Serial", 0x58, 2, ""), ("Joypad", 0x60, 2, "")];

fn r(u: Option<&Unit>, name: &str) -> u64 {
    u.and_then(|u| register(&u.registers, name)).unwrap_or(0)
}

fn has(u: Option<&Unit>, name: &str) -> bool {
    u.is_some_and(|u| register(&u.registers, name).is_some())
}

fn colour(ppu: Option<&Unit>) -> bool {
    has(ppu, "VBK")
}

fn compatibility(memory: &dyn Memory, ppu: Option<&Unit>) -> bool {
    !colour(ppu) && find(memory, VRAM).is_some_and(|s| s.size == 2 * VRAM_BANK)
}

/// A space of at least `size` bytes, by the name the engine spells it with.
fn space(memory: &dyn Memory, name: &str, size: i32) -> Option<String> {
    find(memory, name).filter(|s| s.size >= size).map(|s| s.name.clone())
}

fn expand(c: u32) -> u8 {
    ((c << 3) | (c >> 2)) as u8
}

/// The four colours at `entries`, each read low byte then high in one batch, as the C# reads them one by one.
fn colours(memory: &mut dyn Memory, ram: &str, entries: [i32; 4]) -> Vec<(u8, u8, u8)> {
    let addresses: Vec<i32> = entries.iter().flat_map(|&e| [e, e + 1]).collect();
    let bytes = memory.batch(ram, &addresses);
    bytes
        .chunks(2)
        .map(|b| {
            let v = b[0] as u32 | (b[1] as u32) << 8;
            (expand(v & 0x1F), expand((v >> 5) & 0x1F), expand((v >> 10) & 0x1F))
        })
        .collect()
}

fn colour_palette(memory: &mut dyn Memory, ram: &str, palette: i32) -> Vec<(u8, u8, u8)> {
    colours(memory, ram, [0, 1, 2, 3].map(|c| palette * 8 + c * 2))
}

fn compatibility_palette(memory: &mut dyn Memory, register: u64, ram: &str, palette: i32) -> Vec<(u8, u8, u8)> {
    colours(memory, ram, [0, 1, 2, 3].map(|c| palette * 8 + ((register >> (c * 2)) & 3) as i32 * 2))
}

fn shade_palette(register: u64) -> Vec<(u8, u8, u8)> {
    (0..4)
        .map(|c| {
            let s = SHADES[((register >> (c * 2)) & 3) as usize];
            (s, s, s)
        })
        .collect()
}

/// Four DMG palettes of greys, the Color's sixteen, or the DMG registers through palette RAM.
pub fn palettes(memory: &mut dyn Memory, units: &[Unit]) -> Vec<(i32, Vec<(u8, u8, u8)>)> {
    let Some(ppu) = unit(units, "PPU") else { return Vec::new() };
    let (bgp, obp0, obp1) = (r(Some(ppu), "BGP"), r(Some(ppu), "OBP0"), r(Some(ppu), "OBP1"));
    if colour(Some(ppu)) || compatibility(memory, Some(ppu)) {
        let (Some(bg), Some(obj)) = (space(memory, BG_PALETTES, 64), space(memory, OBJ_PALETTES, 64)) else { return Vec::new() };
        if colour(Some(ppu)) {
            let mut out: Vec<(i32, Vec<(u8, u8, u8)>)> = (0..8).map(|p| (p, colour_palette(memory, &bg, p))).collect();
            out.extend((0..8).map(|p| (8 + p, colour_palette(memory, &obj, p))));
            return out;
        }
        return vec![(0, compatibility_palette(memory, bgp, &bg, 0)), (1, compatibility_palette(memory, obp0, &obj, 0)), (2, compatibility_palette(memory, obp1, &obj, 1))];
    }
    vec![(0, shade_palette(bgp)), (1, shade_palette(obp0)), (2, shade_palette(obp1))]
}

/// Each palette a row of four 16-pixel swatches.
pub fn palette_swatch(memory: &mut dyn Memory, units: &[Unit]) -> (Vec<u8>, i32, i32) {
    let palettes = palettes(memory, units);
    if palettes.is_empty() {
        return (Vec::new(), 0, 0);
    }
    const SWATCH: usize = 16;
    const COLUMNS: usize = 4;
    let (width, height) = (COLUMNS * SWATCH, palettes.len() * SWATCH);
    let mut rgba = vec![0u8; width * height * 4];
    for (p, (_, colours)) in palettes.iter().enumerate() {
        for (c, &(r, g, b)) in colours.iter().enumerate().take(COLUMNS) {
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

/// Every tile of VRAM, sixteen a row, grey by pixel value.
pub fn tile_sheet(memory: &mut dyn Memory) -> (Vec<u8>, i32, i32) {
    let Some(vram) = find(memory, VRAM).filter(|s| s.size > 0).cloned() else { return (Vec::new(), 0, 0) };
    const PER_ROW: usize = 16;
    let tiles = (vram.size / 16) as usize;
    let (width, height) = (PER_ROW * 8, tiles.div_ceil(PER_ROW) * 8);
    let mut rgba = vec![0u8; width * height * 4];
    let bytes = memory.run(&vram.name, tiles * 16);
    for tile in 0..tiles {
        let (x0, y0) = (tile % PER_ROW * 8, tile / PER_ROW * 8);
        for row in 0..8 {
            let (low, high) = (bytes[tile * 16 + row * 2], bytes[tile * 16 + row * 2 + 1]);
            for column in 0..8 {
                let bit = 7 - column;
                let s = SHADES[((low >> bit) & 1 | ((high >> bit) & 1) << 1) as usize];
                let o = ((y0 + row) * width + x0 + column) * 4;
                rgba[o..o + 4].copy_from_slice(&[s, s, s, 0xFF]);
            }
        }
    }
    (rgba, width as i32, height as i32)
}

/// The tile number, and in colour mode the attribute from bank 1.
pub fn decode_tilemap_entry(memory: &mut dyn Memory, space: &str, address: i32, units: &[Unit]) -> String {
    if !colour(unit(units, "PPU")) {
        return format!("${:02X}", memory.read(space, address));
    }
    let both = memory.batch(space, &[address, address.wrapping_add(VRAM_BANK)]);
    let (tile, a) = (both[0], both[1]);
    let flips = format!("{}{}", if a & 0x20 != 0 { "X" } else { "-" }, if a & 0x40 != 0 { "Y" } else { "-" });
    format!("${tile:02X} p{} b{} {flips}{}", a & 7, (a >> 3) & 1, if a & 0x80 != 0 { " pri" } else { "" })
}

/// A tile's 64 pixels; a Game Boy tile is always 2bpp.
pub fn decode_tile_pixels(memory: &mut dyn Memory, space: &str, address: i32, bpp: i32) -> Result<Vec<u8>, (String, Option<&'static str>)> {
    if bpp != 2 {
        return Err((format!("A Game Boy tile is always 2bpp; got {bpp}."), Some("bpp")));
    }
    let mut pixels = vec![0u8; 64];
    let addresses: Vec<i32> = (0..16).map(|i| address.wrapping_add(i)).collect();
    let bytes = memory.batch(space, &addresses);
    for row in 0..8 {
        let (low, high) = (bytes[row as usize * 2], bytes[row as usize * 2 + 1]);
        for column in 0..8 {
            let bit = 7 - column;
            pixels[(row * 8 + column) as usize] = (low >> bit) & 1 | ((high >> bit) & 1) << 1;
        }
    }
    Ok(pixels)
}

/// A sprite as `DebugSpriteInfo` holds it: index, x, y, width, height, tile, palette, priority, flip x, flip y.
pub type Sprite = (i32, i32, i32, i32, i32, i32, i32, i32, bool, bool);

/// OAM's forty entries with LCDC's height; one parked off the top or bottom is hidden.
pub fn sprites(memory: &mut dyn Memory, units: &[Unit]) -> Vec<Sprite> {
    let Some(ppu) = unit(units, "PPU") else { return Vec::new() };
    let Some(oam) = space(memory, OAM, 4 * SPRITE_COUNT) else { return Vec::new() };
    let height = if r(Some(ppu), "LCDC") & 4 != 0 { 16 } else { 8 };
    let colour = colour(Some(ppu));
    let mut out = Vec::new();
    let table = memory.run(&oam, 4 * SPRITE_COUNT as usize);
    for i in 0..SPRITE_COUNT {
        let e = &table[i as usize * 4..i as usize * 4 + 4];
        let (y, x, tile, a) = (e[0] as i32 - 16, e[1] as i32 - 8, e[2] as i32, e[3]);
        if y <= -height || y >= SCREEN_HEIGHT {
            continue;
        }
        let palette = if colour { (a & 7) as i32 } else if a & 0x10 != 0 { 1 } else { 0 };
        out.push((i, x, y, 8, height, tile, palette, if a & 0x80 != 0 { 0 } else { 1 }, a & 0x20 != 0, a & 0x40 != 0));
    }
    out
}

fn scale(value: u64, max: u64) -> i32 {
    (value as f64 / max as f64 * 100.0).round() as i32
}

fn shown(b: bool) -> &'static str {
    if b { "True" } else { "False" }
}

/// A channel as `DebugAudioChannelInfo` holds it: index, name, active, level, muted, the line.
pub type Channel = (i32, &'static str, bool, i32, bool, String);

/// Each channel from its unit; a level is a 0-100 rescale of its own volume, which for the wave is a shift.
pub fn audio_channels(units: &[Unit], mutes: u32) -> Vec<Channel> {
    let mut out = Vec::new();
    for (i, &name) in CHANNELS.iter().enumerate() {
        let Some(c) = unit(units, name) else { continue };
        let c = Some(c);
        let active = r(c, "Enabled") != 0 && r(c, "Dac") != 0;
        let muted = mutes & (1 << i) != 0;
        let vol = r(c, "Vol");
        let entry = match i {
            2 => (if vol == 0 { 0 } else { scale(4u64.wrapping_sub(vol), 3) }, format!("Freq={} Vol={vol} Len={} Dac={}", r(c, "Freq"), r(c, "Len"), shown(r(c, "Dac") != 0))),
            3 => (scale(vol, 15), format!("Shift={} Div={} Short={} Vol={vol} Len={}", r(c, "Shift"), r(c, "Div"), shown(r(c, "Short") != 0), r(c, "Len"))),
            _ => {
                let sweep = if has(c, "SweepPeriod") { format!(" Sweep={}/{}{}", r(c, "SweepPeriod"), r(c, "SweepShift"), if r(c, "SweepNegate") != 0 { "-" } else { "+" }) } else { String::new() };
                (scale(vol, 15), format!("Duty={} Freq={} Vol={vol} Len={}{sweep}", r(c, "Duty"), r(c, "Freq"), r(c, "Len")))
            }
        };
        out.push((i as i32, name, active, entry.0, muted, entry.1));
    }
    out
}

/// HDMA as one row: general, hdma, control, destination register, source, size, active, table, line counter, indirect.
pub type Dma = (bool, bool, u8, u8, i32, i32, bool, i32, u8, i32);

pub fn dma_channels(units: &[Unit]) -> Vec<Dma> {
    let Some(h) = unit(units, "HDMA") else { return Vec::new() };
    let h = Some(h);
    let blocks = r(h, "BlocksLeft") as i32;
    let hblank = r(h, "HBlank") != 0;
    vec![(blocks > 0 && !hblank, hblank, if hblank { 0x80 } else { 0 }, 0, r(h, "Source") as i32, blocks.wrapping_mul(16), hblank && blocks > 0, 0, blocks.clamp(0, 255) as u8, 0x8000 | r(h, "Destination") as i32)]
}

/// Only the ranges whose destination does not depend on the cartridge's bank state; None where it does.
pub fn resolve_physical(address: i32, units: &[Unit]) -> Option<(&'static str, i32, bool)> {
    let ppu = Some(unit(units, "PPU")?);
    Some(match address {
        ..0x4000 => (ROM, address, true),
        ..0x8000 => return None,
        ..0xA000 => (VRAM, r(ppu, "VBK") as i32 * VRAM_BANK + (address - 0x8000), true),
        ..0xC000 => return None,
        ..0xD000 => ("WRAM", address - 0xC000, true),
        ..0xE000 => ("WRAM", (r(ppu, "SVBK") as i32).max(1) * WRAM_BANK + (address - 0xD000), true),
        ..0xFE00 => return None,
        ..0xFEA0 => (OAM, address - 0xFE00, true),
        ..0xFF00 => ("PROHIBITED", address - 0xFEA0, false),
        ..0xFF80 => ("IOREG", address - 0xFF00, false),
        ..0xFFFF => ("HRAM", address - 0xFF80, true),
        _ => ("IE", 0, false),
    })
}

/// The cartridge from its header, then the CPU, PPU, colour and sound lines.
pub fn summary(registers: &[(String, u64)], memory: &mut dyn Memory, units: &[Unit]) -> Option<String> {
    register(registers, "PC")?;
    let ppu = Some(unit(units, "PPU")?);
    let c = |n: &str| register(registers, n).unwrap_or(0);
    let mut text = String::new();
    if let Some(rom) = find(memory, ROM).filter(|s| s.size >= cartridge::HEADER_LENGTH as i32).cloned() {
        let head = memory.run(&rom.name, cartridge::HEADER_LENGTH);
        if let Ok(cart) = cartridge::read(&head, rom.size as usize) {
            let ram = if cart.ram_length == 0 { "no RAM".to_string() } else { format!("{}K RAM", cart.ram_length / 1024) };
            text.push_str(&format!("{}, {}x16K ROM, {ram}{}{}\n", cart.board, cart.rom_banks, if cart.battery { ", battery" } else { "" }, if cart.timer { ", RTC" } else { "" }));
        }
    }
    text.push_str(&format!("CPU  A={:02X} F={:02X} BC={:04X} DE={:04X} HL={:04X} SP={:04X} PC={:04X}\n", c("A"), c("F"), c("BC"), c("DE"), c("HL"), c("SP"), c("PC")));
    text.push_str(&format!("     IME={} HALT={} IE={:02X} IF={:02X}\n", c("IME"), c("HALT"), c("IE"), c("IF")));
    text.push_str(&format!("PPU  LCDC={:02X} STAT={:02X} SCY={:02X} SCX={:02X} WY={:02X} WX={:02X}", r(ppu, "LCDC"), r(ppu, "STAT"), r(ppu, "SCY"), r(ppu, "SCX"), r(ppu, "WY"), r(ppu, "WX")));
    if colour(ppu) {
        let blocks = r(unit(units, "HDMA"), "BlocksLeft");
        let hdma = if blocks == 0 { "idle".to_string() } else { format!("{blocks} blocks") };
        text.push_str(&format!("\nCGB  VBK={} SVBK={} speed={} HDMA={hdma}", r(ppu, "VBK"), r(ppu, "SVBK"), if r(ppu, "KEY1") & 0x80 != 0 { "double" } else { "normal" }));
    }
    if let Some(apu) = unit(units, "APU") {
        let (nr50, nr51) = (r(Some(apu), "$FF24"), r(Some(apu), "$FF25"));
        text.push_str(&format!("\nAPU  power={} vol={}/{} pan={nr51:02X}", if r(Some(apu), "Power") != 0 { "on" } else { "off" }, (nr50 >> 4) & 7, nr50 & 7));
    }
    Some(text)
}
