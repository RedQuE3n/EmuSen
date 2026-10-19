//! `emusen-systems`: the systems table, what each console is whichever engine runs it.
//!
//! One data file per system, `data/<id>.json`, embedded when the crate is built, so there is no installed file to go
//! missing and no file from outside is read (EmuSen_Backend.md §5.2, D6). A field belongs here when it is true of the
//! console on every engine; what an engine does (the extensions it receives, its firmware, its controllers, its
//! settings, `starts_empty`) is core info's (§5.3, D9). The packs' code is a module per console family beside the
//! table (§5.2, D6): the save media's format modules first (step B3). EmuSen_Backend.md §12 and §13 record what was built.

pub mod cheats;
pub mod debug;
pub mod gb;
pub mod genesis;
pub mod media;
pub mod n64;
pub mod nes;
pub mod openvgdb;
pub mod probe;
pub mod psx;
pub mod snes;

use emusen_dotnet::json::{self, Value};
use std::sync::OnceLock;

/// One system's facts, as its data file states them.
#[derive(Clone, Debug, PartialEq)]
pub struct System {
    pub id: String,
    pub name: String,
    pub console: String,
    pub manufacturer: String,
    pub release_year: i32,
    /// What the console's games come as, those the runtime expands included (`.m3u`); lower case, each with its dot.
    pub extensions: Vec<String>,
    /// The libretro cheat database's folders for the console.
    pub cheat_systems: Vec<String>,
    /// OpenVGDB's `systemShortName` for the console.
    pub openvgdb_systems: Vec<String>,
    /// Height over width of the console's North American box.
    pub cover_aspect: f64,
    pub esde_system: String,
    pub esde_full_name: String,
    /// Whether an image records its own length, so a malformed one must be refused (EmuSen_CoreAPI.md §21.4).
    pub self_delimiting_images: bool,
    /// The console an attachment plugs into, by its `console`; none for a console of its own.
    pub attached_to: Option<String>,
    /// ScreenScraper's `systemeid`; none where ScreenScraper has no such system.
    pub screenscraper_system: Option<u32>,
    /// The memory space a console's RAM cheats write into, as every engine of it names the space; none for a console
    /// with no codec that writes RAM (EmuSen_Backend.md F27).
    pub cheat_space: Option<String>,
    /// The console's save media, each read by a format module of its family's; none for a console that has no medium.
    pub media: Vec<Medium>,
    /// The order of a multi-byte value in the console's memory, `big` or `little`, as every engine's spaces hold it; none
    /// where no one has stated it yet, which a reader takes as little-endian (EmuSen_Backend.md §18.7).
    pub byte_order: Option<String>,
}

/// A save medium as the console's own manager names it (EmuSen_SaveMedia.md §6.2).
#[derive(Clone, Debug, PartialEq, Eq)]
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
pub const KEYS: [&str; 17] = [
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
    "cheat_space",
    "media",
    "byte_order",
];

/// Every key a medium's object holds, in the order the table writes them.
pub const MEDIUM_KEYS: [&str; 3] = ["id", "title", "noun"];

/// The data files, by the id each is named for, in the table's order.
pub const DATA: [(&str, &str); 11] = [
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
];

struct Fields<'a> {
    file: &'a str,
    members: &'a [(json::Text, Value)],
}

impl<'a> Fields<'a> {
    fn fail<T>(&self, what: impl Into<String>) -> Result<T, Error> {
        Err(Error { file: self.file.to_string(), what: what.into() })
    }

    fn value(&self, key: &str) -> Result<&'a Value, Error> {
        match self.members.iter().find(|(name, _)| name.text == key) {
            Some((_, value)) => Ok(value),
            None => self.fail(format!("{key} is missing")),
        }
    }

    fn text(&self, key: &str) -> Result<String, Error> {
        match self.value(key)?.as_str() {
            Some(text) if !text.trim().is_empty() => Ok(text.to_string()),
            Some(_) => self.fail(format!("{key} is blank")),
            None => self.fail(format!("{key} is not a string")),
        }
    }

    fn optional_text(&self, key: &str) -> Result<Option<String>, Error> {
        if self.value(key)?.is_null() { Ok(None) } else { self.text(key).map(Some) }
    }

    fn texts(&self, key: &str) -> Result<Vec<String>, Error> {
        let Value::Array(items) = self.value(key)? else { return self.fail(format!("{key} is not a list")) };
        let mut texts = Vec::with_capacity(items.len());
        for item in items {
            match item.as_str() {
                Some(text) if !text.trim().is_empty() => texts.push(text.to_string()),
                _ => return self.fail(format!("{key} holds something that is not a word")),
            }
        }
        if texts.is_empty() {
            return self.fail(format!("{key} is empty"));
        }
        if (1..texts.len()).any(|i| texts[..i].contains(&texts[i])) {
            return self.fail(format!("{key} names one twice"));
        }
        Ok(texts)
    }

    fn token(&self, key: &str) -> Result<&'a str, Error> {
        match self.value(key)? {
            Value::Number(token) => Ok(token),
            _ => self.fail(format!("{key} is not a number")),
        }
    }

    fn media(&self) -> Result<Vec<Medium>, Error> {
        let Value::Array(items) = self.value("media")? else { return self.fail("media is not a list") };
        let mut media: Vec<Medium> = Vec::with_capacity(items.len());
        for item in items {
            let Value::Object(members) = item else { return self.fail("media holds something that is not a medium") };
            if members.len() != MEDIUM_KEYS.len() || members.iter().zip(MEDIUM_KEYS).any(|((name, _), key)| name.text != key) {
                return self.fail(format!("a medium's fields are not {}", MEDIUM_KEYS.join(", ")));
            }
            let m = Fields { file: self.file, members };
            let medium = Medium { id: m.text("id")?, title: m.text("title")?, noun: m.text("noun")? };
            if media.iter().any(|other| other.id == medium.id) {
                return self.fail(format!("the medium {} is named twice", medium.id));
            }
            media.push(medium);
        }
        Ok(media)
    }

    fn bool(&self, key: &str) -> Result<bool, Error> {
        match self.value(key)?.as_bool() {
            Some(value) => Ok(value),
            None => self.fail(format!("{key} is not true or false")),
        }
    }
}

/// One data file read as a system, refused with words where it is not one; `file` is the id it is named for.
pub fn parse(file: &str, text: &str) -> Result<System, Error> {
    let fail = |what: String| Error { file: file.to_string(), what };
    let document = json::parse(text).map_err(|e| fail(e.to_string()))?;
    let Value::Object(members) = &document else { return Err(fail("the file is not an object".into())) };
    for (i, (name, _)) in members.iter().enumerate() {
        if !KEYS.contains(&name.text.as_str()) {
            return Err(fail(format!("{} is not a field of a system", name.text)));
        }
        if members[..i].iter().any(|(earlier, _)| earlier.text == name.text) {
            return Err(fail(format!("{} is given twice", name.text)));
        }
    }
    let f = Fields { file, members };
    let system = System {
        id: f.text("id")?,
        name: f.text("name")?,
        console: f.text("console")?,
        manufacturer: f.text("manufacturer")?,
        release_year: match f.token("release_year")?.parse::<i32>() {
            Ok(year) if (1970..=2100).contains(&year) => year,
            _ => return f.fail("release_year is not a year"),
        },
        extensions: f.texts("extensions")?,
        cheat_systems: f.texts("cheat_systems")?,
        openvgdb_systems: f.texts("openvgdb_systems")?,
        cover_aspect: match f.token("cover_aspect")?.parse::<f64>() {
            Ok(aspect) if aspect.is_finite() && aspect > 0.0 => aspect,
            _ => return f.fail("cover_aspect is not a positive number"),
        },
        esde_system: f.text("esde_system")?,
        esde_full_name: f.text("esde_full_name")?,
        self_delimiting_images: f.bool("self_delimiting_images")?,
        attached_to: f.optional_text("attached_to")?,
        media: Vec::new(),
        cheat_space: f.optional_text("cheat_space")?,
        byte_order: match f.optional_text("byte_order")? {
            Some(order) if order != "big" && order != "little" => return f.fail(format!("byte_order {order} is neither big nor little")),
            order => order,
        },
        screenscraper_system: if f.value("screenscraper_system")?.is_null() {
            None
        } else {
            match f.token("screenscraper_system")?.parse::<u32>() {
                Ok(id) if id > 0 => Some(id),
                _ => return f.fail("screenscraper_system is not a positive whole number"),
            }
        },
    };
    let system = System { media: f.media()?, ..system };
    if system.id != file {
        return f.fail(format!("its id is {}, not the {file} it is named for", system.id));
    }
    if let Some(bad) = system.extensions.iter().find(|e| e.len() < 2 || !e.starts_with('.') || e.chars().any(|c| c.is_uppercase() || c.is_whitespace())) {
        return f.fail(format!("the extension {bad} is not a dot and lower-case letters"));
    }
    Ok(system)
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

/// The table as JSON, one compact array of objects, each member in `KEYS`' order and each number as .NET writes it.
pub fn to_json(systems: &[System]) -> String {
    fn words(list: &[String]) -> Value {
        Value::Array(list.iter().map(|w| Value::string(w.as_str())).collect())
    }
    fn or_null(value: Option<Value>) -> Value {
        value.unwrap_or(Value::Null)
    }
    let rows = systems
        .iter()
        .map(|s| {
            let values = [
                Value::string(s.id.as_str()),
                Value::string(s.name.as_str()),
                Value::string(s.console.as_str()),
                Value::string(s.manufacturer.as_str()),
                Value::number(s.release_year),
                words(&s.extensions),
                words(&s.cheat_systems),
                words(&s.openvgdb_systems),
                // Finite by parse, so .NET has a number for it.
                Value::Number(json::format_f64(s.cover_aspect).unwrap_or_else(|| "0".into())),
                Value::string(s.esde_system.as_str()),
                Value::string(s.esde_full_name.as_str()),
                Value::Bool(s.self_delimiting_images),
                or_null(s.attached_to.as_deref().map(Value::string)),
                or_null(s.screenscraper_system.map(Value::number)),
                or_null(s.cheat_space.as_deref().map(Value::string)),
                Value::Array(
                    s.media
                        .iter()
                        .map(|m| {
                            let values = [m.id.as_str(), m.title.as_str(), m.noun.as_str()].map(Value::string);
                            Value::Object(MEDIUM_KEYS.iter().zip(values).map(|(key, value)| (json::Text::new(*key), value)).collect())
                        })
                        .collect(),
                ),
                or_null(s.byte_order.as_deref().map(Value::string)),
            ];
            Value::Object(KEYS.iter().zip(values).map(|(key, value)| (json::Text::new(*key), value)).collect())
        })
        .collect();
    let mut out = String::new();
    json::write_compact(&mut out, &Value::Array(rows));
    out
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

#[cfg(test)]
mod tests {
    use super::*;

    fn nes() -> String {
        DATA[0].1.to_string()
    }

    #[test]
    fn a_byte_order_is_big_little_or_unstated() {
        let table = build(&DATA).unwrap();
        let order = |id: &str| table.iter().find(|s| s.id == id).unwrap().byte_order.clone();
        assert_eq!((order("n64").as_deref(), order("nes").as_deref(), order("md").as_deref()), (Some("big"), Some("little"), Some("big")));
        assert_eq!(parse("nes", &nes().replace("\"byte_order\": \"little\"", "\"byte_order\": null")).unwrap().byte_order, None);
        assert_eq!(refused("nes", &nes().replace("\"byte_order\": \"little\"", "\"byte_order\": \"middle\"")), "byte_order middle is neither big nor little");
    }

    fn refused(file: &str, text: &str) -> String {
        parse(file, text).expect_err("the file should be refused").what
    }

    #[test]
    fn the_embedded_table_reads_whole_in_its_order() {
        let systems = table().expect("the committed data files are a table");
        let ids: Vec<&str> = systems.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, ["nes", "snes", "md", "mcd", "32x", "psx", "gb", "gbc", "n64", "sms", "sg1000"]);
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
        assert_eq!((sms.byte_order.as_deref(), sms.cheat_space.as_deref(), sms.media.len()), (Some("little"), None, 0));
    }

    #[test]
    fn the_json_has_every_key_in_order_and_numbers_as_dotnet_writes_them() {
        let text = table_json().unwrap();
        let Value::Array(rows) = json::parse(text).unwrap() else { panic!("not an array") };
        assert_eq!(rows.len(), DATA.len());
        for row in &rows {
            let Value::Object(members) = row else { panic!("not an object") };
            let keys: Vec<&str> = members.iter().map(|(k, _)| k.text.as_str()).collect();
            assert_eq!(keys, KEYS);
        }
        assert!(text.contains(r#""cover_aspect":1,"#), "1.0 is written as .NET writes it");
        assert!(text.contains(r#""cover_aspect":0.73,"#));
        assert!(text.contains(r#""attached_to":null,"#));
        assert!(text.starts_with(r#"[{"id":"nes","name":"Nintendo Entertainment System","console":"NES""#));
    }

    #[test]
    fn the_json_reads_back_as_the_same_table() {
        let text = table_json().unwrap();
        let Value::Array(rows) = json::parse(text).unwrap() else { panic!("not an array") };
        for (row, system) in rows.iter().zip(table().unwrap()) {
            let mut one = String::new();
            json::write_compact(&mut one, row);
            assert_eq!(&parse(&system.id, &one).unwrap(), system);
        }
    }

    #[test]
    fn a_file_with_a_field_missing_unknown_or_twice_is_refused() {
        assert_eq!(refused("nes", &nes().replace("  \"console\": \"NES\",\n", "")), "console is missing");
        assert_eq!(refused("nes", &nes().replace("\"console\"", "\"consol\"")), "consol is not a field of a system");
        assert_eq!(refused("nes", &nes().replace("\"id\": \"nes\",", "\"id\": \"nes\", \"id\": \"nes\",")), "id is given twice");
    }

    #[test]
    fn a_field_of_the_wrong_kind_is_refused() {
        assert_eq!(refused("nes", &nes().replace("1983", "\"1983\"")), "release_year is not a number");
        assert_eq!(refused("nes", &nes().replace("1983", "83.5")), "release_year is not a year");
        assert_eq!(refused("nes", &nes().replace("1.43", "-1")), "cover_aspect is not a positive number");
        assert_eq!(refused("nes", &nes().replace("\"self_delimiting_images\": false", "\"self_delimiting_images\": 0")), "self_delimiting_images is not true or false");
        assert_eq!(refused("nes", &nes().replace("\"screenscraper_system\": 3", "\"screenscraper_system\": 0")), "screenscraper_system is not a positive whole number");
        assert_eq!(refused("nes", &nes().replace("\"NES\",\n  \"manufacturer\"", "\"  \",\n  \"manufacturer\"")), "console is blank");
        assert_eq!(refused("nes", &nes().replace("[\".nes\"]", "[]")), "extensions is empty");
        assert_eq!(refused("nes", &nes().replace("[\".nes\"]", "[\".nes\", \".nes\"]")), "extensions names one twice");
        assert_eq!(refused("nes", &nes().replace("[\".nes\"]", "[\".NES\"]")), "the extension .NES is not a dot and lower-case letters");
        assert_eq!(refused("nes", &nes().replace("[\".nes\"]", "[\"nes\"]")), "the extension nes is not a dot and lower-case letters");
        assert_eq!(refused("nes", "[1]"), "the file is not an object");
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
        assert_eq!(refused("psx", &psx.replace(r#""noun": "Card" }"#, r#""noun": "Card", "x": 1 }"#)), "a medium's fields are not id, title, noun");
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
        assert_eq!(refused("nes", &nes().replace(",\n  \"screenscraper_system\": 3", "")), "screenscraper_system is missing");
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
