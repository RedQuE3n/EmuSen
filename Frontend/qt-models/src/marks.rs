//! The screens' drawn marks as SVG paths in a 24-unit box, the courts canvas's own (EmuSen_Interface.md §24.1, §24.6):
//! a place's icon by its key, an engine's mark by `library::engine_mark`, and the fixed marks a screen names.

/// A mark's path by its name; empty for a name with none.
pub fn path(name: &str) -> &'static str {
    match name {
        "all" => "M4 5h7v6H4zM13 5h7v6h-7zM4 13h7v6H4zM13 13h7v6h-7z",
        crate::library::FAVOURITES => "M12 20s-7-4.4-7-10a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 5.6-7 10-7 10z",
        crate::library::RECENT => "M12 7v5l3 2M21 12a9 9 0 1 1-3-6.7M21 4v5h-5",
        "moon" => "M15 3.5a8.5 8.5 0 1 0 0 17a7 7 0 1 1 0-17z",
        "mercury" => "M8 3a4 4 0 0 0 8 0M12 6.5a4.5 4.5 0 1 0 0 9a4.5 4.5 0 1 0 0-9M12 15.5V22M9 19h6",
        "venus" => "M12 3a5 5 0 1 0 0 10a5 5 0 1 0 0-10M12 13v9M8.5 18.5h7",
        "mars" => "M10 9a5.5 5.5 0 1 0 0 11a5.5 5.5 0 1 0 0-11M14 10l6-6M15 4h5v5",
        "shard" => "M12 3l3.5 17h-7zM6 10l2 10M18 9l-2 11",
        "gem" => "M4 9l3.5-4.5h9L20 9l-8 11zM4 9h16",
        "planet" => "M12 5a7 7 0 1 0 0 14a7 7 0 1 0 0-14",
        "games" => "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
        "scraper" => "M21 12a9 9 0 1 1-3-6.7M21 3v6h-6",
        "packages" => "M3 7l9-4 9 4-9 4zM3 7v10l9 4 9-4V7M12 11v10",
        "library" => "M4 4h6a2 2 0 0 1 2 2v14a2 2 0 0 0-2-2H4zM20 4h-6a2 2 0 0 0-2 2v14a2 2 0 0 1 2-2h6z",
        "cores" => "M8 8h8v8H8zM4 10h4M4 14h4M16 10h4M16 14h4M10 4v4M14 4v4M10 16v4M14 16v4",
        "settings" => "M12 8a4 4 0 1 0 0 8a4 4 0 1 0 0-8M12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9l2.1 2.1M17 17l2.1 2.1M4.9 19.1L7 17M17 7l2.1-2.1",
        "power" => "M12 2v10M18.4 6.6a9 9 0 1 1-12.8 0",
        _ => "",
    }
}

/// The favourite's heart, filling its box.
pub const HEART: &str = "M12 23.8s-10.15-6.38-10.15-14.5a5.8 5.8 0 0 1 10.15-3.77a5.8 5.8 0 0 1 10.15 3.77c0 8.12-10.15 14.5-10.15 14.5z";
/// A four-pointed sparkle beside the focused body.
pub const SPARKLE: &str = "M12 2L14 10L22 12L14 14L12 22L10 14L2 12L10 10Z";
/// The crescent before the wordmark: a disc less a disc set up and to the right.
pub const CRESCENT: &str = "M12.06 0A12 12 0 1 0 18.4 22.15A12 12 0 0 1 12.06 0Z";
/// A status needing the player: a narrow triangle with its mark cut out.
pub const ATTENTION: &str = "M12 1L18.5 23H5.5zM11 8h2v8h-2zM11 18h2v2.5h-2z";
/// libretro's badge: a rounded square in an 11-unit box.
pub const BADGE: &str = "M1.6 0h7.8a1.6 1.6 0 0 1 1.6 1.6v7.8a1.6 1.6 0 0 1-1.6 1.6h-7.8a1.6 1.6 0 0 1-1.6-1.6v-7.8a1.6 1.6 0 0 1 1.6-1.6z";

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
