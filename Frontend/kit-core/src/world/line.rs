//! A world as one line of plain text, the declarative way a host hands the kit its worlds (EmuSen_Kit.md §7.17):
//! `HUE SHAPE CRATERS BANDS CAP RING MOONS [#LIGHT #MIDDLE #RIM]`, shapes `planet`, `shards`, `gem`, rings `none`,
//! `subtle`, `big`; a world without the three colours is drawn from its hue.

use super::body::{WorldBody, WorldRing, WorldShape};
use crate::tokens::Colour;

fn shape_word(shape: WorldShape) -> &'static str {
    match shape {
        WorldShape::Planet => "planet",
        WorldShape::Shards => "shards",
        WorldShape::Gem => "gem",
    }
}

fn ring_word(ring: WorldRing) -> &'static str {
    match ring {
        WorldRing::None => "none",
        WorldRing::Subtle => "subtle",
        WorldRing::Big => "big",
    }
}

/// A shape by its word; an unknown word is the plain planet's shape.
pub fn shape_of(word: &str) -> WorldShape {
    match word {
        "shards" => WorldShape::Shards,
        "gem" => WorldShape::Gem,
        _ => WorldShape::Planet,
    }
}

/// A colour written `#RRGGBB`.
pub fn hex(text: &str) -> Option<Colour> {
    let h = text.strip_prefix('#').filter(|h| h.len() == 6)?;
    let byte = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).ok();
    Some(Colour::rgb(byte(0)?, byte(2)?, byte(4)?))
}

/// The line for one world.
pub fn world_line(hue: f64, body: &WorldBody) -> String {
    let mut line =
        format!("{hue} {} {} {} {} {} {}", shape_word(body.shape), body.craters, body.bands, body.cap, ring_word(body.ring), body.moons);
    if let Some(palette) = body.palette {
        for c in palette {
            line.push_str(&format!(" #{:02X}{:02X}{:02X}", c.r, c.g, c.b));
        }
    }
    line
}

/// One world from its line; none when a field is missing or unreadable.
pub fn parse_world(line: &str) -> Option<(f64, WorldBody)> {
    let f: Vec<&str> = line.split_whitespace().collect();
    if f.len() != 7 && f.len() != 10 {
        return None;
    }
    let num = |i: usize| f[i].parse::<f64>().ok().filter(|v| v.is_finite());
    let ring = match f[5] {
        "none" => WorldRing::None,
        "subtle" => WorldRing::Subtle,
        "big" => WorldRing::Big,
        _ => return None,
    };
    let palette = if f.len() == 10 { Some([hex(f[7])?, hex(f[8])?, hex(f[9])?]) } else { None };
    let body = WorldBody {
        shape: match f[1] {
            "planet" | "shards" | "gem" => shape_of(f[1]),
            _ => return None,
        },
        palette,
        craters: num(2)?,
        bands: num(3)?,
        cap: num(4)?,
        ring,
        moons: f[6].parse::<u8>().ok()?.min(2),
    };
    Some((num(0)?, body))
}

/// Every readable line's world, in order; a line that cannot be read is left out.
pub fn parse_worlds(text: &str) -> Vec<(f64, WorldBody)> {
    text.lines().filter_map(parse_world).collect()
}
