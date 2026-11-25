//! The screens' drawn marks as SVG paths in a 24-unit box, the courts canvas's own (EmuSen_Interface.md §24.1, §24.6):
//! a place's icon by its key, an engine's mark from kit-core's `planet_mark` (its one owner), and the fixed marks a screen names.

/// A mark's path by its name; empty for a name with none.
pub fn path(name: &str) -> &'static str {
    match name {
        "all" => "M4 5h7v6H4zM13 5h7v6h-7zM4 13h7v6H4zM13 13h7v6h-7z",
        crate::library::FAVOURITES => "M12 20s-7-4.4-7-10a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 5.6-7 10-7 10z",
        crate::library::RECENT => "M12 7v5l3 2M21 12a9 9 0 1 1-3-6.7M21 4v5h-5",
        "games" => "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
        "scraper" => "M21 12a9 9 0 1 1-3-6.7M21 3v6h-6",
        "packages" => "M3 7l9-4 9 4-9 4zM3 7v10l9 4 9-4V7M12 11v10",
        "library" => "M4 4h6a2 2 0 0 1 2 2v14a2 2 0 0 0-2-2H4zM20 4h-6a2 2 0 0 0-2 2v14a2 2 0 0 1 2-2h6z",
        "cores" => "M8 8h8v8H8zM4 10h4M4 14h4M16 10h4M16 14h4M10 4v4M14 4v4M10 16v4M14 16v4",
        "settings" => "M12 8a4 4 0 1 0 0 8a4 4 0 1 0 0-8M12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9l2.1 2.1M17 17l2.1 2.1M4.9 19.1L7 17M17 7l2.1-2.1",
        "power" => "M12 2v10M18.4 6.6a9 9 0 1 1-12.8 0",
        "menu.gamePage" => "M5 4h14v16H5zM9 9h6M9 13h6M9 17h3",
        "menu.search" => "M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14zM16.5 16.5L21 21",
        "menu.library" => "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
        "menu.tools" => "M7 7h10v10H7zM10 3v4M14 3v4M10 17v4M14 17v4M3 10h4M3 14h4M17 10h4M17 14h4",
        "menu.settings" => "M4 7h9M17 7h3M4 17h3M11 17h9M15 5v4M9 15v4M4 12h16",
        "menu.power" => "M12 3v9M7 6.5a8 8 0 1 0 10 0",
        "menu.quit" => "M14 4h5v16h-5M10 8l-4 4 4 4M6 12h9",
        "notice.warning" => "M12 4l9 16H3zM12 10v5M12 17.5h.01",
        "notice.ok" => "M4.5 12.5l5 5 10-10",
        "power.sleep" => "M20 14.500A8.500 8.500 0 0 1 9.500 4a8.500 8.500 0 1 0 10.500 10.500z",
        "power.restart" => "M4 12a8 8 0 1 0 2.600-5.900M4 4v4.500h4.500",
        "power.shutdown" => "M12 3v9M7 6.500a8 8 0 1 0 10 0",
        "power.desktop" => "M4 5h16v11H4zM9 20h6M12 16v4",
        "power.quit" => "M14 4h5v16h-5M10 8l-4 4 4 4M6 12h9",
        _ => emusen_kit_core::planet_mark::path(name),
    }
}

/// The favourite's heart, filling its box.
pub const HEART: &str = "M12 23.8s-10.15-6.38-10.15-14.5a5.8 5.8 0 0 1 10.15-3.77a5.8 5.8 0 0 1 10.15 3.77c0 8.12-10.15 14.5-10.15 14.5z";
/// A four-pointed sparkle beside the focused body.
pub const SPARKLE: &str = "M12 2L14 10L22 12L14 14L12 22L10 14L2 12L10 10Z";
/// The crescent before the wordmark: a disc less a disc set up and to the right.
pub const CRESCENT: &str = "M12.06 0A12 12 0 1 0 18.4 22.15A12 12 0 0 1 12.06 0Z";
/// Eclipse Home Card's crescent mark in its 34-unit box (EclipseHomeCard:30).
pub const MENU_CRESCENT: &str = "M24 4.5A13.5 13.5 0 1 0 24 29.5 10.5 10.5 0 1 1 24 4.5Z";
/// A status needing the player: a narrow triangle with its mark cut out.
pub const ATTENTION: &str = "M12 1L18.5 23H5.5zM11 8h2v8h-2zM11 18h2v2.5h-2z";
pub use emusen_kit_core::planet_mark::{BADGE, alien};
