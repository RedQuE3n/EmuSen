//! The console packs' views by system, the C# `ISystemDebugViews` each pack gives (`SystemCode.Views` over the C#
//! views, `N64DebugViews`): every view is `emusen-systems`' and reads memory through its `Memory`; a system with no pack,
//! or a view its pack has not, answers what the C# interface's default answers.

use emusen_systems::debug::{Memory, Reference, Unit, unit};
use emusen_systems::{gb, n64, nes, snes};

/// A palette's index and colours.
pub type Palette = (i32, Vec<(u8, u8, u8)>);

/// A picture: RGBA, width, height.
pub type Picture = (Vec<u8>, i32, i32);

/// A sprite: index, x, y, width, height, tile, palette, priority, flip x, flip y.
pub type Sprite = (i32, i32, i32, i32, i32, i32, i32, i32, bool, bool);

/// A sound channel: index, name, active, level, muted, the line.
pub type AudioChannel = (i32, String, bool, i32, bool, String);

/// A block-transfer channel, `DebugDmaChannel`'s fields in order.
pub type DmaChannel = (i32, bool, bool, u8, u8, i32, i32, bool, i32, u8, i32);

/// An interrupt vector: name, address, width, mode.
pub type Vector = (&'static str, i32, i32, &'static str);

/// One instruction a pack lists: address, bytes, mnemonic, operands, the address it names.
pub type Listed = (i32, Vec<u8>, String, String, Option<(Reference, i32)>);

fn is_gb(system: &str) -> bool {
    system == "gb" || system == "gbc"
}

/// Whether the system has a pack with views at all, `Views` not null.
pub fn has(system: &str) -> bool {
    matches!(system, "nes" | "snes" | "gb" | "gbc" | "n64")
}

pub fn palettes(system: &str, m: &mut dyn Memory, units: &[Unit]) -> Vec<Palette> {
    match system {
        "nes" => nes::views::palettes(m),
        "snes" => snes::views::palettes(m),
        s if is_gb(s) => gb::views::palettes(m, units),
        _ => Vec::new(),
    }
}

pub fn palette_swatch(system: &str, m: &mut dyn Memory, units: &[Unit]) -> Picture {
    match system {
        "nes" => nes::views::palette_swatch(m),
        "snes" => snes::views::palette_swatch(m),
        s if is_gb(s) => gb::views::palette_swatch(m, units),
        _ => (Vec::new(), 0, 0),
    }
}

pub fn tile_sheet(system: &str, m: &mut dyn Memory) -> Picture {
    match system {
        "nes" => nes::views::tile_sheet(m),
        "snes" => snes::views::tile_sheet(m),
        s if is_gb(s) => gb::views::tile_sheet(m),
        _ => (Vec::new(), 0, 0),
    }
}

pub fn tilemap_entry_stride(system: &str) -> i32 {
    match system {
        "nes" => nes::views::TILEMAP_ENTRY_STRIDE,
        "snes" => snes::views::TILEMAP_ENTRY_STRIDE,
        s if is_gb(s) => gb::views::TILEMAP_ENTRY_STRIDE,
        _ => 0,
    }
}

pub fn decode_tilemap_entry(system: &str, m: &mut dyn Memory, space: &str, address: i32, units: &[Unit]) -> String {
    match system {
        "nes" => nes::views::decode_tilemap_entry(m, space, address),
        "snes" => snes::views::decode_tilemap_entry(m, space, address),
        s if is_gb(s) => gb::views::decode_tilemap_entry(m, space, address, units),
        _ => String::new(),
    }
}

/// A tile's 64 palette indices; the refusal's words and the parameter it names, as an `ArgumentException` carries them.
pub fn decode_tile_pixels(system: &str, m: &mut dyn Memory, space: &str, address: i32, bpp: i32) -> Result<Vec<u8>, (String, Option<&'static str>)> {
    match system {
        "nes" => nes::views::decode_tile_pixels(m, space, address, bpp),
        "snes" => snes::views::decode_tile_pixels(m, space, address, bpp),
        s if is_gb(s) => gb::views::decode_tile_pixels(m, space, address, bpp),
        _ => Ok(Vec::new()),
    }
}

pub fn dma_channels(system: &str, m: &mut dyn Memory, units: &[Unit]) -> Vec<DmaChannel> {
    match system {
        "snes" => snes::views::dma_channels(m)
            .into_iter()
            .map(|c| (c.index, c.general, c.hdma, c.control, c.destination, c.source, c.size, c.hdma_active, c.table, c.line_counter, c.indirect))
            .collect(),
        s if is_gb(s) => gb::views::dma_channels(units).into_iter().map(|(g, h, c, d, s, z, a, t, l, i)| (0, g, h, c, d, s, z, a, t, l, i)).collect(),
        _ => Vec::new(),
    }
}

pub fn name_dma_destination(system: &str, register: u8) -> Option<String> {
    match system {
        "snes" => Some(snes::views::name_dma_destination(register)),
        s if is_gb(s) => Some("VRAM".to_string()),
        _ => None,
    }
}

pub fn vectors(system: &str) -> &'static [Vector] {
    match system {
        "nes" => &nes::views::VECTORS,
        "snes" => &snes::views::VECTORS,
        s if is_gb(s) => &gb::views::VECTORS,
        _ => &[],
    }
}

pub fn symbol_names(system: &str) -> Vec<String> {
    match system {
        "snes" => snes::views::symbol_names().into_iter().map(str::to_string).collect(),
        _ => Vec::new(),
    }
}

/// A symbol the console derives from the main processor's registers.
pub fn symbol(system: &str, name: &str, registers: &[(String, u64)]) -> Option<i64> {
    match system {
        "snes" => snes::views::symbol(name, registers),
        _ => None,
    }
}

/// The lines a console adds to a summary; none for none.
pub fn summary(system: &str, registers: &[(String, u64)], m: &mut dyn Memory, units: &[Unit]) -> Option<String> {
    match system {
        "nes" => nes::views::summary(registers, m, units),
        "snes" => snes::views::summary(registers),
        s if is_gb(s) => gb::views::summary(registers, m, units),
        "n64" => {
            let vi = units.iter().find(|u| u.kind == "video" && emusen_dotnet::text::equals_ignore_case(&u.name, "VI")).map(|u| u.registers.as_slice());
            n64::views::summary(registers, vi)
        }
        _ => None,
    }
}

pub fn max_sprites(system: &str) -> i32 {
    match system {
        "nes" => nes::views::MAX_SPRITES,
        s if is_gb(s) => gb::views::SPRITE_COUNT,
        _ => 0,
    }
}

pub fn sprites(system: &str, m: &mut dyn Memory, units: &[Unit]) -> Vec<Sprite> {
    match system {
        "nes" => nes::views::sprites(m, units),
        s if is_gb(s) => gb::views::sprites(m, units),
        _ => Vec::new(),
    }
}

pub fn audio_channels(system: &str, units: &[Unit], mutes: u32) -> Vec<AudioChannel> {
    let named = |v: Vec<(i32, &'static str, bool, i32, bool, String)>| v.into_iter().map(|(i, n, a, l, m, s)| (i, n.to_string(), a, l, m, s)).collect();
    match system {
        "nes" => named(nes::views::audio_channels(units, mutes)),
        s if is_gb(s) => named(gb::views::audio_channels(units, mutes)),
        _ => Vec::new(),
    }
}

/// What an address on the main processor's bus decodes to: the space or device, the offset, whether it is addressable.
pub fn resolve_physical(system: &str, address: i32, m: &mut dyn Memory, units: &[Unit]) -> Option<(String, i32, bool)> {
    let owned = |(n, o, a): (&str, i32, bool)| (n.to_string(), o, a);
    match system {
        "nes" => nes::views::resolve_physical(address).map(owned),
        s if is_gb(s) => gb::views::resolve_physical(address, units).map(owned),
        "n64" => {
            let rdram = emusen_systems::debug::find(m, "RDRAM").map_or(0, |s| s.size as i64);
            Some(owned(n64::views::physical(address as u32, rdram)))
        }
        _ => None,
    }
}

/// The words a unit is shown under, a cartridge's board by its name; none for the unit's own name.
pub fn unit_label(system: &str, u: &Unit) -> Option<String> {
    match system {
        "nes" => nes::views::unit_label(u),
        _ => None,
    }
}

/// A space's code listed by the processor that runs it, for an engine that decodes none.
pub fn disassemble(system: &str, m: &mut dyn Memory, space: &str, address: i32, count: i32) -> Vec<Listed> {
    match system {
        "n64" => n64::views::disassemble(m, space, address, count)
            .into_iter()
            .map(|l| (l.address as i32, l.bytes.to_vec(), l.mnemonic, l.operands, l.reference.map(|(k, t)| (k, t as i32))))
            .collect(),
        _ => Vec::new(),
    }
}

/// A unit of a list by name, for a caller that holds units.
pub fn find_unit<'a>(units: &'a [Unit], name: &str) -> Option<&'a Unit> {
    unit(units, name)
}
