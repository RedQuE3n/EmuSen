//! A core's planet mark, the one owner of which mark a core has and how each mark is drawn (EmuSen_Interface.md §24.6,
//! EmuSen_Kit.md §7.115): the kit's `PlanetMark` draws from it and the program's `Marks` reads its paths from it.

/// The cores named after the Moon and the planets, whose mark is their astronomical symbol.
pub const SKY_NAMED: [&str; 4] = ["moon", "mercury", "venus", "mars"];

/// The mark of a hosted libretro core: libretro's badge with the project's alien cut through it.
pub const LIBRETRO: &str = "libretro";

/// The side of the box a mark's path is written in, a constant of the courts canvas's drawing.
pub const BOX: f64 = 24.0;

/// The width a mark's path is stroked at in its box, a constant of the courts canvas's drawing.
pub const STROKE: f64 = 2.0;

/// The side of the box libretro's badge and the alien are written in, a constant of the courts canvas's drawing.
pub const BADGE_BOX: f64 = 11.0;

/// libretro's badge: a rounded square in the badge's box.
pub const BADGE: &str = "M1.6 0h7.8a1.6 1.6 0 0 1 1.6 1.6v7.8a1.6 1.6 0 0 1-1.6 1.6h-7.8a1.6 1.6 0 0 1-1.6-1.6v-7.8a1.6 1.6 0 0 1 1.6-1.6z";

/// The mark of a core, a property of the core: libretro's for a hosted core, a sky-named core's own symbol, else its
/// console's body glyph ("shard", "gem") or "planet".
pub fn of(name: &str, libretro: bool, body: &str) -> String {
    let lower = name.to_lowercase();
    if libretro {
        LIBRETRO.into()
    } else if SKY_NAMED.contains(&lower.as_str()) {
        lower
    } else if body.contains("shard") {
        "shard".into()
    } else if body.contains("gem") {
        "gem".into()
    } else {
        "planet".into()
    }
}

/// A mark's path in the 24-unit box, stroked; empty for libretro's (drawn from `BADGE` and `alien`) and for a name with none.
pub fn path(mark: &str) -> &'static str {
    match mark {
        "moon" => "M15 3.5a8.5 8.5 0 1 0 0 17a7 7 0 1 1 0-17z",
        "mercury" => "M8 3a4 4 0 0 0 8 0M12 6.5a4.5 4.5 0 1 0 0 9a4.5 4.5 0 1 0 0-9M12 15.5V22M9 19h6",
        "venus" => "M12 3a5 5 0 1 0 0 10a5 5 0 1 0 0-10M12 13v9M8.5 18.5h7",
        "mars" => "M10 9a5.5 5.5 0 1 0 0 11a5.5 5.5 0 1 0 0-11M14 10l6-6M15 4h5v5",
        "shard" => "M12 3l3.5 17h-7zM6 10l2 10M18 9l-2 11",
        "gem" => "M4 9l3.5-4.5h9L20 9l-8 11zM4 9h16",
        "planet" => "M12 5a7 7 0 1 0 0 14a7 7 0 1 0 0-14",
        _ => "",
    }
}

/// The project's own pixel alien on libretro's badge, one unit a pixel, as the courts canvas draws it.
pub fn alien() -> String {
    const ROWS: [&str; 8] = ["X.......X", ".X.....X.", ".XXXXXXX.", "XX.XXX.XX", "XXXXXXXXX", ".XX...XX.", ".X.....X.", "XX.....XX"];
    let mut out = String::new();
    for (y, row) in ROWS.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            if c == 'X' {
                out.push_str(&format!("M{} {}h1v1h-1z", x + 1, y as f64 + 1.5));
            }
        }
    }
    out
}
