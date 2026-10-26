//! The save media's format modules as a host reads them, by medium id: a listing of saves in words, and copy, delete
//! and format with their refusals in words. Each file system is a crate of its own with no standard library
//! (`emusen-psx-card`, `emusen-n64-pak`, `emusen-mcd-bram`, EmuSen_Backend.md D14); a family's module here gives its
//! saves their names and titles and its refusals their words, as the C# modules did (EmuSen_SaveMedia.md §6.2).

use emusen_dotnet::json::{self, Value};

/// A save as the console's own manager shows it; `id` is the medium's own name for it, which a copy carries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Save {
    pub id: String,
    pub title: String,
    pub units: usize,
    pub game: Option<String>,
    pub damaged: bool,
}

/// A medium's saves and its free units, or why it is not read as one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Listing {
    pub formatted: bool,
    pub saves: Vec<Save>,
    pub free: usize,
    pub problem: Option<String>,
}

impl Listing {
    pub fn unreadable(problem: impl Into<String>) -> Listing {
        Listing { formatted: false, saves: Vec::new(), free: 0, problem: Some(problem.into()) }
    }
}

/// Why an operation gave no answer: refused, with the words for the player; or a damaged directory reaching outside
/// the image, where the C# module threw `ArgumentOutOfRangeException`, which the facade throws in its place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault {
    Refused(String),
    Outside,
}

/// A medium's module: the file's length and the medium's capacity in its own unit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Module {
    pub id: &'static str,
    pub length: usize,
    pub capacity: usize,
    pub unit: &'static str,
}

/// Every medium a module reads, by the id the systems table gives it.
pub const MODULES: [Module; 5] = [
    Module { id: "psx.card", length: emusen_psx_card::CARD_LENGTH, capacity: emusen_psx_card::BLOCKS, unit: "block" },
    Module { id: "n64.pak", length: emusen_n64_pak::PAK_LENGTH, capacity: emusen_n64_pak::DATA_PAGES, unit: "page" },
    Module { id: "mcd.ramcart", length: 0x2_0000, capacity: 0x2_0000 / 64 - 3, unit: "block" },
    Module { id: "mcd.backup", length: 0x2000, capacity: 0x2000 / 64 - 3, unit: "block" },
    Module { id: "ps2.card", length: emusen_ps2_card::CARD_LENGTH, capacity: crate::ps2::card::CAPACITY, unit: "kilobyte" },
];

pub fn module(id: &str) -> Option<&'static Module> {
    MODULES.iter().find(|m| m.id == id)
}

/// Bytes as `Encoding.ASCII` reads them: each byte past 7Fh a question mark.
pub(crate) fn ascii(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| if b < 0x80 { b as char } else { '?' }).collect()
}

/// A medium's listing; none for a medium no module reads.
pub fn read(id: &str, image: &[u8]) -> Option<Result<Listing, Fault>> {
    Some(match module(id)?.id {
        "psx.card" => Ok(crate::psx::card::read(image)),
        "n64.pak" => Ok(crate::n64::pak::read(image)),
        "ps2.card" => Ok(crate::ps2::card::read(image)),
        _ => crate::genesis::bram::read(image),
    })
}

/// `to` with the save named `save` on `from` copied onto it.
pub fn copy(id: &str, from: &[u8], save: &str, to: &[u8]) -> Option<Result<Vec<u8>, Fault>> {
    Some(match module(id)?.id {
        "psx.card" => crate::psx::card::copy(from, save, to),
        "n64.pak" => crate::n64::pak::copy(from, save, to),
        "ps2.card" => crate::ps2::card::copy(from, save, to),
        _ => crate::genesis::bram::copy(from, save, to),
    })
}

/// `image` with the save named `save` deleted as the console's manager deletes.
pub fn delete(id: &str, image: &[u8], save: &str) -> Option<Result<Vec<u8>, Fault>> {
    Some(match module(id)?.id {
        "psx.card" => crate::psx::card::delete(image, save),
        "n64.pak" => crate::n64::pak::delete(image, save),
        "ps2.card" => crate::ps2::card::delete(image, save),
        _ => crate::genesis::bram::delete(image, save),
    })
}

/// A formatted blank of the medium.
pub fn format(id: &str) -> Option<Vec<u8>> {
    let m = module(id)?;
    let mut image = vec![0u8; m.length];
    match m.id {
        "psx.card" => emusen_psx_card::format(&mut image),
        "n64.pak" => emusen_n64_pak::format(&mut image),
        "ps2.card" => crate::ps2::card::format(&mut image),
        _ => emusen_mcd_bram::format(&mut image),
    }
    Some(image)
}

fn text_or_null(text: Option<&str>) -> Value {
    text.map_or(Value::Null, Value::string)
}

fn object(members: Vec<(&str, Value)>) -> Value {
    Value::Object(members.into_iter().map(|(k, v)| (json::Text::new(k), v)).collect())
}

/// A listing as JSON: `formatted`, `saves` (each `id`, `title`, `units`, `game`, `damaged`), `free`, `problem`.
pub fn listing_json(listing: &Listing) -> String {
    let saves = listing
        .saves
        .iter()
        .map(|s| {
            object(vec![
                ("id", Value::string(s.id.as_str())),
                ("title", Value::string(s.title.as_str())),
                ("units", Value::number(s.units as i64)),
                ("game", text_or_null(s.game.as_deref())),
                ("damaged", Value::Bool(s.damaged)),
            ])
        })
        .collect();
    let value = object(vec![
        ("formatted", Value::Bool(listing.formatted)),
        ("saves", Value::Array(saves)),
        ("free", Value::number(listing.free as i64)),
        ("problem", text_or_null(listing.problem.as_deref())),
    ]);
    let mut out = String::new();
    json::write_compact(&mut out, &value);
    out
}

/// Every module's facts as JSON: `id`, `length`, `capacity`, `unit`, in `MODULES`' order.
pub fn modules_json() -> String {
    let rows = MODULES
        .iter()
        .map(|m| {
            object(vec![
                ("id", Value::string(m.id)),
                ("length", Value::number(m.length as i64)),
                ("capacity", Value::number(m.capacity as i64)),
                ("unit", Value::string(m.unit)),
            ])
        })
        .collect();
    let mut out = String::new();
    json::write_compact(&mut out, &Value::Array(rows));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_medium_of_the_table_has_a_module_and_every_module_a_medium() {
        let named: Vec<&str> = crate::table().unwrap().iter().flat_map(|s| s.media.iter().map(|m| m.id.as_str())).collect();
        let mut modules: Vec<&str> = MODULES.iter().map(|m| m.id).collect();
        let mut sorted = named.clone();
        sorted.sort();
        modules.sort();
        assert_eq!(sorted, modules);
    }

    #[test]
    fn a_formatted_blank_reads_as_formatted_with_its_capacity_free() {
        for m in MODULES {
            let blank = format(m.id).unwrap();
            assert_eq!(blank.len(), m.length);
            let listing = read(m.id, &blank).unwrap().unwrap();
            assert!(listing.formatted && listing.saves.is_empty(), "{}", m.id);
            assert_eq!(listing.free, m.capacity, "{}", m.id);
        }
        assert!(read("snes.sram", &[]).is_none());
    }

    #[test]
    fn a_listing_writes_every_key_in_order() {
        let listing = Listing {
            formatted: true,
            saves: vec![Save { id: "A".into(), title: "Ä".into(), units: 2, game: None, damaged: true }],
            free: 3,
            problem: None,
        };
        assert_eq!(listing_json(&listing), r#"{"formatted":true,"saves":[{"id":"A","title":"Ä","units":2,"game":null,"damaged":true}],"free":3,"problem":null}"#);
    }
}
