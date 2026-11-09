//! `emusen-systems`: the systems table, what each console is whichever engine runs it.
//!
//! One data file per system, `data/<id>.json`, embedded when the crate is built, so there is no installed file to go
//! missing and no file from outside is read (EmuSen_Backend.md §5.2, D6). A field belongs here when it is true of the
//! console on every engine; what an engine does (the extensions it receives, its firmware, its controllers, its
//! settings, `starts_empty`) is core info's (§5.3, D9). The packs' code is a module per console family beside the
//! table (§5.2, D6): the save media's format modules first (step B3). EmuSen_Backend.md §12 and §13 record what was built.

pub mod libretro_cores;
pub mod libretro_extensions;
pub mod catalog_rows;
pub mod cheats;
pub mod debug;
pub mod gb;
pub mod genesis;
pub mod media;
pub mod n64;
pub mod nes;
pub mod openvgdb;
pub mod probe;
pub mod ps2;
pub mod psx;
pub mod sega_disc;
pub mod sms;
pub mod snes;

use serde::{Deserialize, Deserializer, Serialize};
use std::sync::OnceLock;

/// A fact that may be unknown but is always written: `null` or a value, and a file without it refused.
fn stated<'de, D: Deserializer<'de>, T: Deserialize<'de>>(d: D) -> Result<Option<T>, D::Error> {
    Option::<T>::deserialize(d)
}

/// ES-DE's full name as written, or empty for `null`, which `parse` reads as `display_name`; a written blank is refused.
fn esde_name<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    match Option::<String>::deserialize(d)? {
        Some(name) if name.trim().is_empty() => Err(serde::de::Error::custom("esde_full_name is blank")),
        name => Ok(name.unwrap_or_default()),
    }
}

/// One system's facts, as its data file states them: every field written, `null` where a fact is unknown, and in the
/// order the table writes them.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct System {
    pub id: String,
    pub name: String,
    pub console: String,
    pub manufacturer: String,
    pub release_year: i32,
    /// What the console's games come as, those the runtime expands included (`.m3u`); lower case, each with its dot.
    pub extensions: Vec<String>,
    /// The libretro cheat database's folders for the console; empty where the database has none (Sapphire_Native.md §3.1).
    pub cheat_systems: Vec<String>,
    /// OpenVGDB's `systemShortName` for the console; empty where OpenVGDB has no such system.
    pub openvgdb_systems: Vec<String>,
    /// Height over width of the console's North American box.
    pub cover_aspect: f64,
    pub esde_system: String,
    /// The system's full name as ES-DE 3.4.1's user guide lists it, which an ES-DE theme reads as `${system.fullName}`;
    /// `null` in a file where the guide lists no such system, read as `display_name` (EmuSen_Qt.md §4.6).
    #[serde(deserialize_with = "esde_name")]
    pub esde_full_name: String,
    /// Whether an image records its own length, so a malformed one must be refused (EmuSen_CoreAPI.md §21.4).
    pub self_delimiting_images: bool,
    /// The console an attachment plugs into, by its `console`; none for a console of its own.
    #[serde(deserialize_with = "stated")]
    pub attached_to: Option<String>,
    /// ScreenScraper's `systemeid`; none where ScreenScraper has no such system.
    #[serde(deserialize_with = "stated")]
    pub screenscraper_system: Option<u32>,
    /// The console's name as the player reads it in a list, "Super Nintendo" where `console` is "SNES" (EmuSen_Cores.md §3.8).
    pub display_name: String,
    /// The console's firmware in the player's words, "a PlayStation 2 BIOS" without its article; none where it has none (EmuSen_Cores.md §3.9).
    #[serde(deserialize_with = "stated")]
    pub firmware_name: Option<String>,
    /// The memory space a console's RAM cheats write into, as every engine of it names the space; none for a console
    /// with no codec that writes RAM (EmuSen_Backend.md F27).
    #[serde(deserialize_with = "stated")]
    pub cheat_space: Option<String>,
    /// The console's save media, each read by a format module of its family's; none for a console that has no medium.
    pub media: Vec<Medium>,
    /// The order of a multi-byte value in the console's memory, `big` or `little`, as every engine's spaces hold it; none
    /// where no one has stated it yet, which a reader takes as little-endian (EmuSen_Backend.md §18.7).
    #[serde(deserialize_with = "stated")]
    pub byte_order: Option<String>,
    /// The libretro database names of the console's games, as a libretro core's `.info` lists them under `database`
    /// (EmuSen_LibretroHost.md §4.1).
    pub libretro_databases: Vec<String>,
    /// The `systemid`s of the libretro cores that run the console, for a core whose `.info` lists no database.
    pub libretro_systemids: Vec<String>,
    /// What the console's firmware is, said once above its files in the firmware overview; none for most (EmuSen_Cores.md §4.10).
    #[serde(deserialize_with = "stated")]
    pub firmware_note: Option<String>,
    /// Whether the console's games can drive a motor in the player's hands, a controller's, an accessory's or a
    /// cartridge's, so that the player's rumble switch is honoured (EmuSen_Input.md §9.2).
    pub rumble: bool,
    /// How the front ends draw the console; none where its data gives none, which `look()` computes.
    #[serde(deserialize_with = "stated")]
    pub look: Option<Look>,
}

impl System {
    /// The console's look as its data gives it, else the one computed from its id.
    pub fn look(&self) -> Look {
        self.look.clone().unwrap_or_else(|| Look::computed(&self.id))
    }
}

/// How the front ends draw a console: its hue and its body, a planet's palette and features as opacities from 0 to 1
/// (EmuSen_Interface.md §22, §24.6).
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Look {
    /// Degrees, 0 to 360.
    pub hue: f64,
    /// One of `BODIES`.
    pub body: String,
    /// A planet's light, middle and rim colours as `#RRGGBB`; empty derives them from the hue.
    pub palette: Vec<String>,
    pub craters: f64,
    pub bands: f64,
    pub cap: f64,
    /// Moonlets, 0 to 2.
    pub moons: u32,
}

impl Look {
    /// A console's look when its data gives none: a plain planet in a hue taken from its id, the same on every start.
    pub fn computed(id: &str) -> Look {
        let hue = id.chars().flat_map(char::to_lowercase).fold(17u32, |h, c| h.wrapping_mul(31).wrapping_add(c as u32)) % 360;
        Look { hue: f64::from(hue), body: "planet".into(), palette: Vec::new(), craters: 0.0, bands: 0.0, cap: 0.0, moons: 0 }
    }
}

/// Every key a look's object holds, in the order the table writes them.
pub const LOOK_KEYS: [&str; 7] = ["hue", "body", "palette", "craters", "bands", "cap", "moons"];

/// The bodies a console is drawn as.
pub const BODIES: [&str; 3] = ["planet", "shards", "gem"];

/// The table-wide facts beside each console's own: today the cover shape of a console whose data gives none.
pub const TABLE_FACTS: &str = include_str!("../data/table.json");

/// Height over width of a box whose console's is unknown (OpenEmu's fallback), from `data/table.json`.
pub fn default_cover_aspect() -> f64 {
    static ASPECT: OnceLock<f64> = OnceLock::new();
    *ASPECT.get_or_init(|| {
        #[derive(Deserialize)]
        struct TableFacts {
            cover_aspect: f64,
        }
        serde_json::from_str::<TableFacts>(TABLE_FACTS).ok().map(|t| t.cover_aspect).filter(|a| a.is_finite() && *a > 0.0).unwrap_or(1.0)
    })
}

/// A save medium as the console's own manager names it (EmuSen_SaveMedia.md §6.2).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Medium {
    pub id: String,
    pub title: String,
    pub noun: String,
}

/// Why a data file is not a system: the file, and the words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub file: String,
    pub what: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.json: {}", self.file, self.what)
    }
}

/// Every key a data file holds, in the order the table writes them; a file with any other key is refused.
pub const KEYS: [&str; 24] = [
    "id",
    "name",
    "console",
    "manufacturer",
    "release_year",
    "extensions",
    "cheat_systems",
    "openvgdb_systems",
    "cover_aspect",
    "esde_system",
    "esde_full_name",
    "self_delimiting_images",
    "attached_to",
    "screenscraper_system",
    "display_name",
    "firmware_name",
    "cheat_space",
    "media",
    "byte_order",
    "libretro_databases",
    "libretro_systemids",
    "firmware_note",
    "rumble",
    "look",
];

/// Every key a medium's object holds, in the order the table writes them.
pub const MEDIUM_KEYS: [&str; 3] = ["id", "title", "noun"];

/// The data files, by the id each is named for, in the table's order.
pub const DATA: [(&str, &str); 14] = [
    ("nes", include_str!("../data/nes.json")),
    ("snes", include_str!("../data/snes.json")),
    ("md", include_str!("../data/md.json")),
    ("mcd", include_str!("../data/mcd.json")),
    ("32x", include_str!("../data/32x.json")),
    ("psx", include_str!("../data/psx.json")),
    ("gb", include_str!("../data/gb.json")),
    ("gbc", include_str!("../data/gbc.json")),
    ("n64", include_str!("../data/n64.json")),
    ("sms", include_str!("../data/sms.json")),
    ("sg1000", include_str!("../data/sg1000.json")),
    ("ps2", include_str!("../data/ps2.json")),
    ("saturn", include_str!("../data/saturn.json")),
    ("dreamcast", include_str!("../data/dreamcast.json")),
];

/// One data file read as a system, refused with words where it is not one; `file` is the id it is named for. serde
/// refuses a field missing, unknown, written twice or of the wrong kind; the rest is checked here.
pub fn parse(file: &str, text: &str) -> Result<System, Error> {
    let fail = |what: String| Error { file: file.to_string(), what };
    let mut system: System = serde_json::from_str(text).map_err(|e| fail(e.to_string()))?;
    if system.esde_full_name.is_empty() {
        system.esde_full_name = system.display_name.clone();
    }
    check(&system).map_err(fail)?;
    if system.id != file {
        return Err(fail(format!("its id is {}, not the {file} it is named for", system.id)));
    }
    Ok(system)
}

/// What a data file states beyond its fields' kinds: no blank words, no word twice in a list, a year, a positive shape,
/// a known byte order, dotted lower-case extensions, distinct media and a look within its ranges.
fn check(s: &System) -> Result<(), String> {
    let blank = |key: &str, text: &str| if text.trim().is_empty() { Err(format!("{key} is blank")) } else { Ok(()) };
    for (key, text) in [("id", &s.id), ("name", &s.name), ("console", &s.console), ("manufacturer", &s.manufacturer), ("esde_system", &s.esde_system), ("esde_full_name", &s.esde_full_name), ("display_name", &s.display_name)] {
        blank(key, text)?;
    }
    for (key, text) in [("attached_to", &s.attached_to), ("firmware_name", &s.firmware_name), ("cheat_space", &s.cheat_space), ("byte_order", &s.byte_order), ("firmware_note", &s.firmware_note)] {
        if let Some(text) = text {
            blank(key, text)?;
        }
    }
    for (key, list) in [("extensions", &s.extensions), ("cheat_systems", &s.cheat_systems), ("openvgdb_systems", &s.openvgdb_systems), ("libretro_databases", &s.libretro_databases), ("libretro_systemids", &s.libretro_systemids)] {
        if list.iter().any(|w| w.trim().is_empty()) {
            return Err(format!("{key} holds something that is not a word"));
        }
        if (1..list.len()).any(|i| list[..i].contains(&list[i])) {
            return Err(format!("{key} names one twice"));
        }
    }
    if s.extensions.is_empty() {
        return Err("extensions is empty".into());
    }
    if let Some(bad) = s.extensions.iter().find(|e| e.len() < 2 || !e.starts_with('.') || e.chars().any(|c| c.is_uppercase() || c.is_whitespace())) {
        return Err(format!("the extension {bad} is not a dot and lower-case letters"));
    }
    if !(1970..=2100).contains(&s.release_year) {
        return Err("release_year is not a year".into());
    }
    if !(s.cover_aspect.is_finite() && s.cover_aspect > 0.0) {
        return Err("cover_aspect is not a positive number".into());
    }
    if s.screenscraper_system == Some(0) {
        return Err("screenscraper_system is not a positive whole number".into());
    }
    if let Some(order) = s.byte_order.as_deref().filter(|o| *o != "big" && *o != "little") {
        return Err(format!("byte_order {order} is neither big nor little"));
    }
    for (i, m) in s.media.iter().enumerate() {
        for (key, text) in [("id", &m.id), ("title", &m.title), ("noun", &m.noun)] {
            blank(key, text)?;
        }
        if s.media[..i].iter().any(|other| other.id == m.id) {
            return Err(format!("the medium {} is named twice", m.id));
        }
    }
    if let Some(l) = &s.look {
        for (key, value, max) in [("hue", l.hue, 360.0), ("craters", l.craters, 1.0), ("bands", l.bands, 1.0), ("cap", l.cap, 1.0)] {
            if !(0.0..=max).contains(&value) {
                return Err(format!("the look's {key} is not a number from 0 to {max}"));
            }
        }
        if l.moons > 2 {
            return Err("the look's moons is not 0, 1 or 2".into());
        }
        if !BODIES.contains(&l.body.as_str()) {
            return Err(format!("the look's body {} is not one of {}", l.body, BODIES.join(", ")));
        }
        let colour = |c: &String| c.len() == 7 && c.starts_with('#') && c[1..].chars().all(|d| d.is_ascii_hexdigit());
        if !(l.palette.is_empty() || l.palette.len() == 3 && l.palette.iter().all(colour)) {
            return Err("the look's palette is not three #RRGGBB colours or none".into());
        }
    }
    Ok(())
}

/// Data files read as one table, in their order: each a system, no id or console twice, and every attachment's host a console of the table.
pub fn build(files: &[(&str, &str)]) -> Result<Vec<System>, Error> {
    let mut systems: Vec<System> = Vec::with_capacity(files.len());
    for (file, text) in files {
        let system = parse(file, text)?;
        let fail = |what: String| Err(Error { file: file.to_string(), what });
        if systems.iter().any(|s| s.id == system.id) {
            return fail(format!("the id {} is another file's", system.id));
        }
        if systems.iter().any(|s| s.console.eq_ignore_ascii_case(&system.console)) {
            return fail(format!("the console {} is another file's", system.console));
        }
        if let Some(m) = system.media.iter().find(|m| systems.iter().any(|s| s.media.iter().any(|o| o.id == m.id))) {
            return fail(format!("the medium {} is another file's", m.id));
        }
        systems.push(system);
    }
    for system in &systems {
        if let Some(host) = &system.attached_to
            && !systems.iter().any(|s| s.console == *host && s.attached_to.is_none())
        {
            return Err(Error { file: system.id.clone(), what: format!("it is attached to {host}, which is no console of its own in the table") });
        }
    }
    Ok(systems)
}

/// The table as JSON, one compact array of objects, each member in `KEYS`' order, written by serde_json.
pub fn to_json(systems: &[System]) -> String {
    serde_json::to_string(systems).unwrap_or_default()
}

static TABLE: OnceLock<Result<(Vec<System>, String), Error>> = OnceLock::new();

fn built() -> &'static Result<(Vec<System>, String), Error> {
    TABLE.get_or_init(|| build(&DATA).map(|systems| {
        let text = to_json(&systems);
        (systems, text)
    }))
}

/// The embedded table, read once; an error only if a data file was committed broken, which the crate's tests refuse.
pub fn table() -> Result<&'static [System], &'static Error> {
    built().as_ref().map(|(systems, _)| systems.as_slice())
}

/// The embedded table as `to_json` writes it, made once.
pub fn table_json() -> Result<&'static str, &'static Error> {
    built().as_ref().map(|(_, text)| text.as_str())
}

/// A system by its id.
/// Each player's controller id from the console's settings, where the system's pack counts them; none where it does
/// not (`SystemPack.PlayerControllers`).
pub fn player_controllers(id: &str, setting: &dyn Fn(&str) -> Option<String>) -> Option<Vec<String>> {
    match id {
        "md" | "mcd" | "32x" => Some(genesis::player_controllers(setting)),
        "psx" => Some(psx::player_controllers(setting)),
        _ => None,
    }
}

pub fn find(id: &str) -> Option<&'static System> {
    table().ok()?.iter().find(|s| s.id == id)
}

/// The systems a libretro core runs: those whose `libretro_databases` share a name with the core's `database` list, or,
/// where the core lists none, the one whose `libretro_systemids` hold its `systemid` (EmuSen_LibretroHost.md §4.1).
pub fn for_libretro(systemid: &str, databases: &[&str]) -> Vec<&'static System> {
    let Ok(table) = table() else { return Vec::new() };
    let databases: Vec<&str> = databases.iter().map(|d| d.trim()).filter(|d| !d.is_empty()).collect();
    if databases.is_empty() {
        return table.iter().filter(|s| !systemid.is_empty() && s.libretro_systemids.iter().any(|i| i == systemid)).collect();
    }
    table.iter().filter(|s| s.libretro_databases.iter().any(|d| databases.contains(&d.as_str()))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nes() -> String {
        DATA[0].1.to_string()
    }

    #[test]
    fn a_libretro_core_runs_the_systems_its_databases_name_or_else_its_systemid() {
        let ids = |systemid: &str, databases: &[&str]| for_libretro(systemid, databases).iter().map(|s| s.id.as_str()).collect::<Vec<_>>();
        let gpgx = ["Sega - Master System - Mark III", "Sega - Game Gear", "Sega - Mega-CD - Sega CD", "Sega - Mega Drive - Genesis", "Sega - PICO", "Sega - SG-1000"];
        assert_eq!(ids("mega_drive", &gpgx), ["md", "mcd", "sms", "sg1000"]);
        assert_eq!(ids("playstation2", &["Sony - PlayStation 2"]), ["ps2"]);
        assert_eq!(ids("game_boy", &[]), ["gb"]);
        assert_eq!(ids("", &[" "]), Vec::<&str>::new());
        assert_eq!(ids("wonderswan", &["Bandai - WonderSwan"]), Vec::<&str>::new());
    }

    #[test]
    fn a_byte_order_is_big_little_or_unstated() {
        let table = build(&DATA).unwrap();
        let order = |id: &str| table.iter().find(|s| s.id == id).unwrap().byte_order.clone();
        assert_eq!((order("n64").as_deref(), order("nes").as_deref(), order("md").as_deref()), (Some("big"), Some("little"), Some("big")));
        assert_eq!(parse("nes", &nes().replace("\"byte_order\": \"little\"", "\"byte_order\": null")).unwrap().byte_order, None);
        assert_eq!(refused("nes", &nes().replace("\"byte_order\": \"little\"", "\"byte_order\": \"middle\"")), "byte_order middle is neither big nor little");
    }

    // ES-DE 3.4.1's user guide, "Supported game systems", its Full name column (EmuSen_Qt.md §4.6).
    #[test]
    fn every_console_s_es_de_full_name_is_es_de_3_4_1_s() {
        let documented = [
            ("nes", "Nintendo Entertainment System"),
            ("snes", "Nintendo SNES (Super Nintendo)"),
            ("md", "Sega Genesis"),
            ("mcd", "Sega CD"),
            ("32x", "Sega Mega Drive 32X"),
            ("psx", "Sony PlayStation"),
            ("gb", "Nintendo Game Boy"),
            ("gbc", "Nintendo Game Boy Color"),
            ("n64", "Nintendo 64"),
            ("sms", "Sega Master System"),
            ("sg1000", "Sega SG-1000"),
            ("ps2", "Sony PlayStation 2"),
            ("saturn", "Sega Saturn"),
            ("dreamcast", "Sega Dreamcast"),
        ];
        let table: Vec<(&str, &str)> = table().unwrap().iter().map(|s| (s.id.as_str(), s.esde_full_name.as_str())).collect();
        assert_eq!(table, documented);
        assert_eq!(find("snes").unwrap().display_name, "Super Nintendo", "the program's own name stays its own");
    }

    #[test]
    fn a_console_es_de_does_not_list_reads_its_display_name() {
        let unlisted = nes().replace("\"esde_full_name\": \"Nintendo Entertainment System\"", "\"esde_full_name\": null");
        assert_eq!(parse("nes", &unlisted).unwrap().esde_full_name, "NES");
        assert!(to_json(&[parse("nes", &unlisted).unwrap()]).contains(r#""esde_full_name":"NES""#), "the table's JSON never carries a null name");
        let named = |name: &str| nes().replace("\"esde_full_name\": \"Nintendo Entertainment System\"", &format!("\"esde_full_name\": {name}"));
        assert!(refused("nes", &named("\" \"")).contains("esde_full_name is blank"));
        assert!(refused("nes", &named("\"\"")).contains("esde_full_name is blank"), "only null falls back");
        assert!(refused("nes", &nes().replace("  \"esde_full_name\": \"Nintendo Entertainment System\",\n", "")).contains("missing field `esde_full_name`"));
    }

    #[test]
    fn a_look_is_the_datas_or_computed_from_the_id() {
        let n64 = find("n64").unwrap().look();
        assert_eq!((n64.hue, n64.body.as_str(), n64.palette.len(), n64.cap, n64.moons), (352.0, "planet", 3, 0.9, 2));
        assert_eq!(find("ps2").unwrap().look().body, "gem");
        assert_eq!(parse("nes", &nes().replace(r#""look": {"hue": 330"#, r#""look": null, "x": {"hue": 330"#)).map(|_| ()).unwrap_err().what.split(',').next().unwrap(), "unknown field `x`");
        let plain = parse("nes", &with_look(&nes(), "null")).unwrap();
        assert_eq!(plain.look(), Look::computed("nes"));
        assert_eq!(Look::computed("NES"), Look::computed("nes"), "the id is read without regard to case");
        assert_eq!(refused("nes", &with_look(&nes(), r#"{"hue": 400, "body": "planet", "palette": [], "craters": 0, "bands": 0, "cap": 0, "moons": 0}"#)), "the look's hue is not a number from 0 to 360");
        assert_eq!(refused("nes", &with_look(&nes(), r#"{"hue": 1, "body": "cube", "palette": [], "craters": 0, "bands": 0, "cap": 0, "moons": 0}"#)), "the look's body cube is not one of planet, shards, gem");
        assert_eq!(refused("nes", &with_look(&nes(), r##"{"hue": 1, "body": "planet", "palette": ["#FFF"], "craters": 0, "bands": 0, "cap": 0, "moons": 0}"##)), "the look's palette is not three #RRGGBB colours or none");
        assert!(to_json(&build(&DATA).unwrap()).contains(r##""look":{"hue":352.0,"body":"planet","palette":["#FFD3D8","#EE4E66","#7E1A38"],"craters":0.0,"bands":0.0,"cap":0.9,"moons":2}"##));
        assert_eq!(default_cover_aspect(), 1.365385);
    }

    fn with_look(text: &str, look: &str) -> String {
        let at = text.find(r#""look": "#).unwrap();
        let end = text[at..].find('}').unwrap() + at + 1;
        format!("{}\"look\": {look}{}", &text[..at], &text[end..])
    }

    fn refused(file: &str, text: &str) -> String {
        parse(file, text).expect_err("the file should be refused").what
    }

    #[test]
    fn the_embedded_table_reads_whole_in_its_order() {
        let systems = table().expect("the committed data files are a table");
        let ids: Vec<&str> = systems.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, ["nes", "snes", "md", "mcd", "32x", "psx", "gb", "gbc", "n64", "sms", "sg1000", "ps2", "saturn", "dreamcast"]);
        for (file, _) in DATA {
            assert!(find(file).is_some(), "{file} is in the table");
        }
        assert!(find("tst").is_none());
    }

    #[test]
    fn a_file_reads_as_the_facts_it_states() {
        let sega_cd = find("mcd").unwrap();
        assert_eq!(sega_cd.console, "Sega CD");
        assert_eq!(sega_cd.release_year, 1991);
        assert_eq!(sega_cd.extensions, [".iso", ".cue", ".chd"]);
        assert_eq!(sega_cd.attached_to.as_deref(), Some("Genesis"));
        assert_eq!(sega_cd.screenscraper_system, Some(20));
        let snes = find("snes").unwrap();
        assert_eq!(snes.cheat_systems, ["Nintendo - Super Nintendo Entertainment System", "Nintendo - Satellaview"]);
        assert_eq!(snes.cover_aspect, 0.73);
        assert!(!snes.self_delimiting_images);
        assert_eq!(snes.attached_to, None);
        assert_eq!(find("gbc").unwrap().release_year, 1998);
        assert_eq!((snes.console.as_str(), snes.display_name.as_str(), snes.firmware_name.as_deref()), ("SNES", "Super Nintendo", None));
        assert_eq!(find("ps2").unwrap().firmware_name.as_deref(), Some("PlayStation 2 BIOS"));
        assert_eq!((sega_cd.display_name.as_str(), find("n64").unwrap().display_name.as_str()), ("Sega CD", "Nintendo 64"));
    }

    // Endou's two systems as Endou_Native.md §3 sources each fact: ES-DE's names, ScreenScraper's ids, the cheat database's folders.
    #[test]
    fn the_master_system_and_the_sg1000_are_two_systems_of_one_extension_each() {
        let sms = find("sms").unwrap();
        assert_eq!((sms.console.as_str(), sms.esde_system.as_str(), sms.esde_full_name.as_str(), sms.screenscraper_system), ("Master System", "mastersystem", "Sega Master System", Some(2)));
        assert_eq!((sms.extensions.as_slice(), sms.cheat_systems.as_slice(), sms.release_year), (&[".sms".to_string()][..], &["Sega - Master System - Mark III".to_string()][..], 1985));
        let sg = find("sg1000").unwrap();
        assert_eq!((sg.console.as_str(), sg.esde_system.as_str(), sg.screenscraper_system, sg.openvgdb_systems.as_slice()), ("SG-1000", "sg-1000", Some(109), &["SG1000".to_string()][..]));
        let table = table().unwrap();
        for e in sms.extensions.iter().chain(&sg.extensions) {
            assert_eq!(table.iter().filter(|s| s.extensions.contains(e)).count(), 1, "{e} is one system's, so routing needs no recogniser");
        }
        assert!(!crate::probe::recognises("sms") && !crate::probe::recognises("sg1000"));
        assert_eq!((sms.byte_order.as_deref(), sms.cheat_space.as_deref(), sms.media.len()), (Some("little"), Some("RAM"), 0));
        assert_eq!((sms.cover_aspect, sg.cover_aspect, sg.cheat_space.as_deref()), (1.4, 1.4, Some("RAM")), "stage 7's values, Endou_Native.md §14");
        assert_eq!(crate::cheats::PAIRS.iter().filter(|p| p.1 == Some("sms.par") && p.2 == Some("sms.gg")).map(|p| p.0).collect::<Vec<_>>(), ["sms", "sg1000"]);
    }

    // Sapphire's system as Sapphire_Native.md §3.1 sources each fact; no cheat folder and no OpenVGDB system exist for it.
    #[test]
    fn the_playstation_2_is_a_system_with_no_cheat_folder_or_openvgdb_system() {
        let ps2 = find("ps2").unwrap();
        assert_eq!((ps2.console.as_str(), ps2.esde_system.as_str(), ps2.esde_full_name.as_str(), ps2.screenscraper_system, ps2.release_year), ("PlayStation 2", "ps2", "Sony PlayStation 2", Some(58), 2000));
        assert_eq!(ps2.extensions, [".elf", ".iso", ".chd", ".cue", ".bin", ".m3u"]);
        assert!(ps2.cheat_systems.is_empty() && ps2.openvgdb_systems.is_empty() && ps2.cheat_space.is_none());
        assert_eq!(ps2.media.iter().map(|m| (m.id.as_str(), m.title.as_str(), m.noun.as_str())).collect::<Vec<_>>(), [("ps2.card", "Memory Card (PS2)", "Card")], "Sapphire_Native.md §30.16");
        assert!(crate::probe::recognises("ps2") && crate::cheats::PAIRS.iter().all(|p| p.0 != "ps2"));
        assert_eq!(refused("nes", &nes().replace("\"cheat_systems\": [", "\"cheat_systems\": [\"A\", \"A\", ")), "cheat_systems names one twice");
    }

    #[test]
    fn the_consoles_whose_games_drive_a_motor_say_so_and_the_rest_do_not() {
        let rumbles: Vec<&str> = table().unwrap().iter().filter(|s| s.rumble).map(|s| s.id.as_str()).collect();
        assert_eq!(rumbles, ["psx", "gb", "gbc", "n64", "ps2", "dreamcast"]);
        assert!(refused("nes", &nes().replace("\"rumble\": false", "\"rumble\": 1")).contains("expected a boolean"), "expected a boolean");
    }

    #[test]
    fn the_json_has_every_key_in_order() {
        let text = table_json().unwrap();
        let rows: Vec<serde_json::Map<String, serde_json::Value>> = serde_json::from_str(text).unwrap();
        assert_eq!(rows.len(), DATA.len());
        for row in &rows {
            assert_eq!(row.keys().map(String::as_str).collect::<Vec<_>>(), KEYS);
        }
        assert!(text.contains(r#""cover_aspect":1.0,"#));
        assert!(text.contains(r#""cover_aspect":0.73,"#));
        assert!(text.contains(r#""attached_to":null,"#));
        assert!(text.starts_with(r#"[{"id":"nes","name":"Nintendo Entertainment System","console":"NES""#));
    }

    #[test]
    fn the_json_reads_back_as_the_same_table() {
        let text = table_json().unwrap();
        let rows: Vec<serde_json::Value> = serde_json::from_str(text).unwrap();
        for (row, system) in rows.iter().zip(table().unwrap()) {
            assert_eq!(&parse(&system.id, &row.to_string()).unwrap(), system);
        }
    }

    #[test]
    fn a_file_with_a_field_missing_unknown_or_twice_is_refused() {
        assert!(refused("nes", &nes().replace("  \"console\": \"NES\",\n", "")).contains("missing field `console`"), "missing field `console`");
        assert!(refused("nes", &nes().replace("\"console\"", "\"consol\"")).contains("unknown field `consol`"), "unknown field `consol`");
        assert!(refused("nes", &nes().replace("\"id\": \"nes\",", "\"id\": \"nes\", \"id\": \"nes\",")).contains("duplicate field `id`"), "duplicate field `id`");
    }

    #[test]
    fn a_field_of_the_wrong_kind_is_refused() {
        assert!(refused("nes", &nes().replace("1983", "\"1983\"")).contains("invalid type: string \"1983\", expected i32"), "invalid type: string '1983', expected i32");
        assert!(refused("nes", &nes().replace("1983", "83.5")).contains("expected i32"), "expected i32");
        assert_eq!(refused("nes", &nes().replace("1.43", "-1")), "cover_aspect is not a positive number");
        assert!(refused("nes", &nes().replace("\"self_delimiting_images\": false", "\"self_delimiting_images\": 0")).contains("expected a boolean"), "expected a boolean");
        assert_eq!(refused("nes", &nes().replace("\"screenscraper_system\": 3", "\"screenscraper_system\": 0")), "screenscraper_system is not a positive whole number");
        assert_eq!(refused("nes", &nes().replace("\"console\": \"NES\"", "\"console\": \"  \"")), "console is blank");
        assert_eq!(refused("nes", &nes().replace("\"display_name\": \"NES\"", "\"display_name\": \"  \"")), "display_name is blank");
        assert_eq!(refused("nes", &nes().replace("[\".nes\"]", "[]")), "extensions is empty");
        assert_eq!(refused("nes", &nes().replace("[\".nes\"]", "[\".nes\", \".nes\"]")), "extensions names one twice");
        assert_eq!(refused("nes", &nes().replace("[\".nes\"]", "[\".NES\"]")), "the extension .NES is not a dot and lower-case letters");
        assert_eq!(refused("nes", &nes().replace("[\".nes\"]", "[\"nes\"]")), "the extension nes is not a dot and lower-case letters");
        assert!(!refused("nes", "[1]").is_empty(), "a list is no system");
        assert!(refused("nes", "{").contains("line 1"), "a file that is not JSON says where");
    }

    #[test]
    fn a_file_named_for_another_id_is_refused() {
        assert_eq!(refused("famicom", &nes()), "its id is nes, not the famicom it is named for");
    }

    #[test]
    fn the_table_refuses_an_id_or_console_twice_and_a_host_it_lacks() {
        let twice = build(&[DATA[0], DATA[0]]).unwrap_err();
        assert_eq!(twice.what, "the id nes is another file's");
        let famicom = nes().replace("\"id\": \"nes\"", "\"id\": \"fc\"");
        assert_eq!(build(&[DATA[0], ("fc", &famicom)]).unwrap_err().what, "the console NES is another file's");
        let orphan = build(&[DATA[3]]).unwrap_err();
        assert_eq!((orphan.file.as_str(), orphan.what.as_str()), ("mcd", "it is attached to Genesis, which is no console of its own in the table"));
        assert!(build(&[DATA[2], DATA[3]]).is_ok());
    }

    #[test]
    fn a_medium_is_its_three_words_once() {
        let psx = DATA[5].1;
        assert_eq!(find("psx").unwrap().media, [Medium { id: "psx.card".into(), title: "Memory Card".into(), noun: "Card".into() }]);
        assert!(refused("psx", &psx.replace(r#""noun": "Card" }"#, r#""noun": "Card", "x": 1 }"#)).contains("unknown field `x`, expected one of `id`, `title`, `noun`"), "unknown field `x`, expected one of `id`, `title`, `noun`");
        assert_eq!(refused("psx", &psx.replace(r#""title": "Memory Card""#, r#""title": " ""#)), "title is blank");
        let twice = psx.replace(r#"[{ "id": "psx.card", "title": "Memory Card", "noun": "Card" }]"#, r#"[{ "id": "psx.card", "title": "A", "noun": "B" }, { "id": "psx.card", "title": "C", "noun": "D" }]"#);
        assert_eq!(refused("psx", &twice), "the medium psx.card is named twice");
        let stolen = DATA[8].1.replace("n64.pak", "psx.card");
        assert_eq!(build(&[DATA[5], ("n64", &stolen)]).unwrap_err().what, "the medium psx.card is another file's");
    }

    #[test]
    fn an_absent_optional_fact_is_null_not_missing() {
        let unknown = nes().replace("\"screenscraper_system\": 3", "\"screenscraper_system\": null");
        assert_eq!(parse("nes", &unknown).unwrap().screenscraper_system, None);
        assert!(refused("nes", &nes().replace(",\n  \"screenscraper_system\": 3", "")).contains("missing field `screenscraper_system`"), "missing field `screenscraper_system`");
    }
}

#[cfg(test)]
mod player_tests {
    use super::player_controllers;

    #[test]
    fn players_follow_the_genesis_and_playstation_port_rules() {
        let of = |pairs: &'static [(&'static str, &'static str)]| move |k: &str| pairs.iter().find(|(key, _)| *key == k).map(|(_, v)| v.to_string());
        assert_eq!(player_controllers("md", &of(&[])).unwrap(), ["md.pad3", "md.pad3"]);
        assert_eq!(player_controllers("mcd", &of(&[("pad1", "md.4way6"), ("pad2", "md.pad6")])).unwrap(), ["md.pad6"; 4]);
        assert_eq!(player_controllers("32x", &of(&[("pad1", "md.pad6"), ("pad2", "md.4way3")])).unwrap(), ["md.pad6", "md.pad3"]);
        assert_eq!(player_controllers("md", &of(&[("pad2", "md.teamplayer3")])).unwrap().len(), 5);
        assert_eq!(player_controllers("psx", &of(&[("pad2", "none")])).unwrap(), ["psx.dualshock"]);
        assert!(player_controllers("snes", &of(&[])).is_none());
    }
}
