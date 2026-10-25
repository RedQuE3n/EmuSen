//! The PlayStation 2 memory card's module: `emusen-ps2-card`'s file system with each save, a directory at the card's
//! top level, named by its directory name, titled by its `icon.sys` and given its game's serial, and its sizes in
//! kilobytes, the card's cluster, as the console's browser shows them in KB (EmuSen_SaveMedia.md §6.2, §7.4;
//! Sapphire_Native.md §30.16).

use crate::media::{Fault, Listing, Save, ascii};
use emusen_ps2_card::{self as card, Card, Entry, Time};

/// The card's capacity in its unit, the kilobyte a cluster holds: what a formatted card has free, as the console's
/// browser shows it, 7,998 KB (Hardware/Shared/EmuSen_MediumFormats.md §6.2; Sapphire_Native.md §30.17).
pub const CAPACITY: usize = 7998;

const BLANK_FREE: usize = CAPACITY;

/// `icon.sys`'s length, its title's place and length, and where it says the title's second line begins (gothi, "icon.sys
/// format", ps2savetools.com, read 2026-10-25).
const ICON_SYS: &[u8] = b"icon.sys";
const ICON_SYS_LENGTH: usize = 964;
const TITLE_AT: usize = 192;
const TITLE_LENGTH: usize = 68;
const LINE_BREAK_AT: usize = 6;

/// A save's title from its `icon.sys`: the Shift-JIS title up to its NUL, full-width letters read as their ASCII, its
/// two lines joined by a space; none where the file is missing, short, not `PS2D` or blank.
pub fn title_of(c: &Card<'_>, save: &Entry) -> Option<String> {
    let dir = c.dir(save).ok()?;
    let (_, icon) = c.find(dir, ICON_SYS).ok()?;
    let mut sys = [0u8; ICON_SYS_LENGTH];
    if c.read(&icon, 0, &mut sys).ok()? != ICON_SYS_LENGTH || &sys[..4] != b"PS2D" {
        return None;
    }
    let field = &sys[TITLE_AT..TITLE_AT + TITLE_LENGTH];
    let field = &field[..field.iter().position(|&b| b == 0).unwrap_or(TITLE_LENGTH)];
    let split = (u16::from_le_bytes([sys[LINE_BREAK_AT], sys[LINE_BREAK_AT + 1]]) as usize).min(field.len());
    let line = |b: &[u8]| crate::psx::shift_jis::nfkc(&crate::psx::shift_jis::decode(b));
    let (first, second) = (line(&field[..split]), line(&field[split..]));
    let text = format!("{} {}", emusen_dotnet::text::trim(&first), emusen_dotnet::text::trim(&second));
    let text = emusen_dotnet::text::trim(&text);
    (!text.is_empty()).then(|| text.to_string())
}

fn unformatted() -> Listing {
    Listing {
        formatted: false,
        saves: Vec::new(),
        free: CAPACITY,
        problem: Some("not formatted: the console's browser formats it, or New Card makes a formatted one".into()),
    }
}

pub fn read(image: &[u8]) -> Listing {
    let c = match Card::open(image) {
        Ok(c) => c,
        Err(card::Damage::Length) => return Listing::unreadable(format!("a PlayStation 2 memory card is {} bytes; this file is {}", card::CARD_LENGTH, image.len())),
        Err(card::Damage::Unformatted) => return unformatted(),
        Err(d) => return Listing::unreadable(d.words()),
    };
    let mut saves = Vec::new();
    for s in card::saves(&c) {
        let Ok((_, e)) = s else { return Listing::unreadable(card::Damage::Directory.words()) };
        let id = ascii(e.name());
        let units = c.clusters_of(&e);
        let title = if e.is_dir() { title_of(&c, &e) } else { None };
        saves.push(Save { title: title.unwrap_or_else(|| id.clone()), units: units.unwrap_or(1), game: crate::psx::card::game_of(&id), damaged: units.is_err(), id });
    }
    match c.free_clusters() {
        Ok(free) => Listing { formatted: true, saves, free, problem: None },
        Err(d) => Listing::unreadable(d.words()),
    }
}

/// The top-level entry whose name reads as `id`, its name's bytes.
fn name_of(image: &[u8], id: &str) -> Option<Vec<u8>> {
    let c = Card::open(image).ok()?;
    card::saves(&c).filter_map(Result::ok).map(|(_, e)| e.name().to_vec()).find(|n| ascii(n) == id)
}

fn refused<T>(words: impl Into<String>) -> Result<T, Fault> {
    Err(Fault::Refused(words.into()))
}

fn opened(image: &[u8], which: &str) -> Result<(), Fault> {
    match Card::open(image) {
        Ok(_) => Ok(()),
        Err(card::Damage::Unformatted) => refused(format!("the card {which} is not formatted")),
        Err(d) => refused(format!("the card {which} cannot be read: {}", d.words())),
    }
}

pub fn copy(from: &[u8], save: &str, to: &[u8]) -> Result<Vec<u8>, Fault> {
    opened(from, "copied from")?;
    opened(to, "copied to")?;
    let Some(name) = name_of(from, save) else { return refused(format!("no save named {save} is on the card")) };
    if name_of(to, save).is_some() {
        return refused(format!("the card already holds a save named {save}"));
    }
    let need = Card::open(from).ok().and_then(|c| card::saves(&c).filter_map(Result::ok).find(|(_, e)| e.name() == name.as_slice()).and_then(|(_, e)| c.clusters_of(&e).ok()));
    let mut image = to.to_vec();
    match card::copy_save(from, &name, &mut image) {
        Ok(()) => Ok(image),
        Err(card::Refusal::Full) => {
            let free = Card::open(to).and_then(|c| c.free_clusters()).unwrap_or(0);
            refused(format!("{save} needs {} kilobyte(s) and the card has {free} free", need.map_or(String::from("more"), |n| (n + 1).to_string())))
        }
        Err(card::Refusal::Kind) => refused(format!("{save} is a file, not a save's directory, so it cannot be copied")),
        Err(r) => refused(format!("{save} cannot be copied: {}", r.words())),
    }
}

pub fn delete(image: &[u8], save: &str) -> Result<Vec<u8>, Fault> {
    opened(image, "")?;
    let Some(name) = name_of(image, save) else { return refused(format!("no save named {save} is on the card")) };
    let mut edited = image.to_vec();
    match card::delete(&mut edited, &name) {
        Ok(()) => Ok(edited),
        Err(r) => refused(format!("{save} cannot be deleted: {}", r.words())),
    }
}

/// A formatted blank, its root made at a fixed time so that every blank is the same bytes.
pub fn format(image: &mut [u8]) {
    card::format(image, Time { sec: 0, min: 0, hour: 0, day: 1, month: 1, year: 2000 }).expect("the module's length is the card's");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank() -> Vec<u8> {
        let mut image = vec![0u8; card::CARD_LENGTH];
        format(&mut image);
        image
    }

    /// An icon.sys of the documented layout with a two-line Shift-JIS title.
    fn icon_sys(title: &[u8], split: u16) -> Vec<u8> {
        let mut sys = vec![0u8; ICON_SYS_LENGTH];
        sys[..4].copy_from_slice(b"PS2D");
        sys[6..8].copy_from_slice(&split.to_le_bytes());
        sys[TITLE_AT..TITLE_AT + title.len()].copy_from_slice(title);
        sys
    }

    fn with_save(mut image: Vec<u8>, name: &[u8], files: &[(&[u8], Vec<u8>)]) -> Vec<u8> {
        let t = Time { sec: 1, min: 2, hour: 3, day: 4, month: 5, year: 2006 };
        card::mkdir(&mut image, b"", name, t).unwrap();
        for (f, data) in files {
            card::create(&mut image, name, f, data, t).unwrap();
        }
        image
    }

    #[test]
    fn a_save_is_listed_with_its_title_size_and_game_and_the_card_its_free_kilobytes() {
        // "ＥＭＵＳＥＮ" in full-width Shift-JIS, then the second line in ASCII.
        let title = [0x82, 0x64, 0x82, 0x6C, 0x82, 0x74, 0x82, 0x72, 0x82, 0x64, 0x82, 0x6D, b'S', b'a', b'v', b'e'];
        let card = with_save(blank(), b"BASLUS-20999EMUSEN", &[(b"icon.sys", icon_sys(&title, 12)), (b"DATA", vec![7; 3000])]);
        let l = read(&card);
        assert!(l.formatted && l.problem.is_none());
        assert_eq!(l.saves, [Save { id: "BASLUS-20999EMUSEN".into(), title: "EMUSEN Save".into(), units: 2 + 1 + 3, game: Some("SLUS-20999".into()), damaged: false }]);
        assert_eq!(l.free, BLANK_FREE - 1 - 6, "the root grown by a cluster for its third entry; the save's directory of four entries, icon.sys and the data's three");
        assert_eq!((read(&blank()).free, read(&vec![0xFF; card::CARD_LENGTH]).formatted), (BLANK_FREE, false));
        assert!(read(&[0u8; 100]).problem.unwrap().contains("8650752"));
    }

    #[test]
    fn a_save_copies_whole_deletes_and_refuses_in_words() {
        let card = with_save(blank(), b"BESLES-50000DEMO", &[(b"icon.sys", icon_sys(b"Demo", 0)), (b"DATA", (0..5000u32).map(|b| b as u8).collect())]);
        let copied = copy(&card, "BESLES-50000DEMO", &blank()).unwrap();
        assert_eq!(read(&copied).saves, read(&card).saves);
        let (a, b) = (Card::open(&card).unwrap(), Card::open(&copied).unwrap());
        let file = |c: &Card<'_>| {
            let dir = c.dir(&c.find(c.root().unwrap(), b"BESLES-50000DEMO").unwrap().1).unwrap();
            let (_, f) = c.find(dir, b"DATA").unwrap();
            let mut out = vec![0u8; 6000];
            let n = c.read(&f, 0, &mut out).unwrap();
            out.truncate(n);
            out
        };
        assert_eq!(file(&a), file(&b), "every byte, the last cluster's with it");
        assert!(matches!(copy(&card, "BESLES-50000DEMO", &copied), Err(Fault::Refused(w)) if w.contains("already holds")));
        assert!(matches!(copy(&card, "NONE", &blank()), Err(Fault::Refused(w)) if w.contains("no save")));
        assert!(matches!(copy(&card, "BESLES-50000DEMO", &vec![0xFF; card::CARD_LENGTH]), Err(Fault::Refused(w)) if w.contains("not formatted")));
        let gone = delete(&copied, "BESLES-50000DEMO").unwrap();
        assert_eq!((read(&gone).saves.len(), read(&gone).free), (0, BLANK_FREE - 1), "the root keeps its second cluster");
    }
}
