//! Each console's screen and the accurate filter's profile made for it, one row for each console of the systems
//! table, and the consoles whose row is still owed. See EmuSen_CRT.md §31 and §31.8.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Television,
    Lcd,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Television => "Television",
            Kind::Lcd => "Lcd",
        }
    }
}

/// A console's screen: for a television, `profile` is its row of the CRT's console timing; for an LCD, the filter that models the panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Screen {
    pub console: &'static str,
    pub kind: Kind,
    pub profile: &'static str,
    /// An attachment with no video output of its own, whose picture is its host's signal.
    pub hosts_picture: bool,
    /// A dated note of what this console's own profile lacks.
    pub gap: Option<&'static str>,
}

const fn tv(console: &'static str, profile: &'static str) -> Screen {
    Screen { console, kind: Kind::Television, profile, hosts_picture: false, gap: None }
}

const LCD_GAP: &str = "2026-10-20: a fit to measurements of the panel, not a model of it; to be recreated more accurately (EmuSen_CRT.md §31.7)";

pub const ALL: &[Screen] = &[
    tv("NES", "NES"),
    tv("SNES", "SNES"),
    tv("Genesis", "Genesis"),
    Screen { hosts_picture: true, ..tv("Sega CD", "Genesis") },
    Screen {
        gap: Some("2026-10-20: drawn on the Genesis's row; the 32X puts the mixed picture out through an output stage of its own, whose encoder no document read names (EmuSen_CRT.md §31.4)"),
        ..tv("32X", "Genesis")
    },
    tv("N64", "N64"),
    tv("PlayStation", "PlayStation"),
    tv("PlayStation 2", "PlayStation 2"),
    tv("Master System", "Master System"),
    tv("SG-1000", "SG-1000"),
    Screen { console: "GB", kind: Kind::Lcd, profile: "Game Boy LCD", hosts_picture: false, gap: Some(LCD_GAP) },
    Screen { console: "GBC", kind: Kind::Lcd, profile: "Game Boy Color LCD", hosts_picture: false, gap: Some(LCD_GAP) },
];

/// A console of the systems table still in development, offered to no player, whose screen row is owed, with why.
pub const OWED: &[(&str, &str)] = &[
    ("Saturn", "2026-11-01: a console only a hosted core runs; its profile is separate work (EmuSen_Cores.md §4.9)"),
    ("Dreamcast", "2026-11-01: a console only a hosted core runs; its profile is separate work (EmuSen_Cores.md §4.9)"),
];

/// The consoles whose television is drawn on a row of their own, which every filter of a tube is offered for.
pub fn televisions() -> Vec<&'static str> {
    ALL.iter().filter(|s| s.kind == Kind::Television && s.profile == s.console).map(|s| s.console).collect()
}
