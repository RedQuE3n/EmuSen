//! Moonrise's finished frame against a C# reference picture, pixel by pixel, under one tolerance shared by the kit's
//! gate and the program's test (EmuSen_Kit.md §7.10, §7.18).

use crate::{Scene, Tree};

/// The C# LunaSplash's finished frame at 1280 x 720 with the draft's 13 consoles and hues: the kit gallery's oracle.
pub const DRAFT_REFERENCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../Testing/moonrise/LunaSplash-1280x720.png");
/// Mistress's own splash at 1280 x 720 with the courts canvas's eleven consoles, each drawn as its body (§24.6).
pub const BODIES_REFERENCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../Testing/moonrise/LunaSplash-bodies-1280x720.png");
/// Levels and reach under which a pixel matches the reference: antialiasing moves an edge by under a pixel.
pub const LEVELS: i32 = 24;
pub const REACH: i32 = 2;
/// The words' and the moon's boxes, where Skia's subpixel text and stroke coverage differ from Qt's: a glyph or the moon
/// out of place still differs by more.
pub const LOOSE_LEVELS: i32 = 128;
/// Pixels that may differ outside them, against the draft.
pub const SPARE: usize = 4;
/// Against the bodies: the edges of the shards and gems are antialiased differently by Skia and by Qt's curve renderer
/// (10 pixels within 30 levels of the tolerance, measured 2026-11-08); a wrong body differs at hundreds.
pub const BODIES_SPARE: usize = 16;

/// The shown frame against `reference`, up to `spare` pixels differing, the differing pixels marked into `marks`; the faults, none when it matches.
pub fn against(scene: &mut Scene<'_>, tree: &Tree, reference: &str, spare: usize, marks: &str) -> Vec<String> {
    let loose: Vec<([i32; 4], i32)> = ["moonriseTitle", "moonriseCaption", "moonOutline"]
        .iter()
        .filter_map(|name| tree.find_named(name))
        .map(|i| {
            let n = &tree.nodes[i as usize];
            ([n.x as i32 - 4, n.y as i32 - 4, n.w as i32 + 8, n.h as i32 + 8], LOOSE_LEVELS)
        })
        .collect();
    if loose.len() != 3 {
        return vec!["the title, the caption or the moon is not in the tree".to_owned()];
    }
    match scene.difference(reference, LEVELS, REACH, &loose, marks) {
        Some(n) if n <= spare => Vec::new(),
        Some(n) => vec![format!("Moonrise differs from {reference} at {n} pixels, marked in {marks}")],
        None => vec![format!("{reference} could not be read, or its size differs")],
    }
}
