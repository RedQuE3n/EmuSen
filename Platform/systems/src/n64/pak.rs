//! The Controller Pak's module: `emusen-n64-pak`'s file system with each note named by its game code, publisher code,
//! name and extension, and each refusal in the C# module's words (EmuSen_SaveMedia.md §7.2).

use crate::media::{Fault, Listing, Save};
use emusen_n64_pak as pak;

/// A name field read in the pak's characters up to its first NUL, its trailing white space kept off.
pub fn text_of(field: &[u8]) -> String {
    let text: String = field.iter().take_while(|&&b| b != 0).map(|&b| pak::char_of(b)).collect();
    emusen_dotnet::text::trim_end(&text).to_string()
}

/// A code field as text where it is printable ASCII, as Nintendo's codes are, else in upper-case hex, as a cheat
/// device's may be.
fn code(field: &[u8]) -> String {
    if field.iter().all(|&b| (0x20..0x7F).contains(&b)) {
        field.iter().map(|&b| b as char).collect()
    } else {
        field.iter().map(|b| format!("{b:02X}")).collect()
    }
}

/// A note's id: its game code, publisher code, name and extension, which tell two games' notes apart as the
/// console's manager does.
pub fn id_of(note: &[u8]) -> String {
    let extension = text_of(&note[0x0C..0x10]);
    let dot = if extension.is_empty() { String::new() } else { format!(".{extension}") };
    format!("{}{}/{}{dot}", code(&note[..4]), code(&note[4..6]), text_of(&note[0x10..0x20]))
}

fn title_of(note: &[u8]) -> String {
    let (name, extension) = (text_of(&note[0x10..0x20]), text_of(&note[0x0C..0x10]));
    if extension.is_empty() { name } else { format!("{name}.{extension}") }
}

pub fn read(image: &[u8]) -> Listing {
    if image.len() != pak::PAK_LENGTH {
        return Listing::unreadable(format!("a Controller Pak is {} bytes; this file is {}", pak::PAK_LENGTH, image.len()));
    }
    if !pak::is_formatted(image) {
        return Listing {
            formatted: false,
            saves: Vec::new(),
            free: pak::DATA_PAGES,
            problem: Some("not formatted: a game formats it when it first saves".into()),
        };
    }
    let Some(index) = pak::index_page(image) else { return Listing::unreadable("both copies of its index of pages are damaged") };
    let saves = pak::saves(image, index)
        .map(|s| {
            let note = pak::note(image, s.note);
            Save { id: id_of(note), title: title_of(note), units: s.pages, game: Some(code(&note[..4])), damaged: s.damaged }
        })
        .collect();
    Listing { formatted: true, saves, free: pak::free_pages(image, index), problem: None }
}

/// The note whose id is `id`.
fn find(image: &[u8], id: &str) -> Option<usize> {
    (0..pak::NOTES).find(|&n| pak::in_use(pak::note(image, n)) && id_of(pak::note(image, n)) == id)
}

fn refused<T>(words: String) -> Result<T, Fault> {
    Err(Fault::Refused(words))
}

fn formatted(image: &[u8]) -> Option<usize> {
    pak::index_page(image).filter(|_| pak::is_formatted(image))
}

pub fn copy(from: &[u8], save: &str, to: &[u8]) -> Result<Vec<u8>, Fault> {
    let Some(from_index) = formatted(from) else { return refused("the pak copied from is not a formatted Controller Pak".into()) };
    if formatted(to).is_none() {
        return refused("the pak copied to is not formatted".into());
    }
    let Some(note) = find(from, save) else { return refused(format!("no note {save} is on the pak")) };
    let index = &from[from_index * pak::PAGE_LENGTH..(from_index + 1) * pak::PAGE_LENGTH];
    if pak::chain(index, pak::start_of(pak::note(from, note))).is_none() {
        return refused(format!("{save}'s pages are not a whole chain, so it cannot be copied"));
    }
    if find(to, save).is_some() {
        return refused(format!("the pak already holds a note {save}"));
    }
    let mut image = to.to_vec();
    match pak::copy(from, note, &mut image) {
        Ok(()) => Ok(image),
        Err(pak::CopyError::NotesFull) => refused(format!("the pak already holds {} notes, the most it can", pak::NOTES)),
        Err(pak::CopyError::NoRoom { needed, free }) => refused(format!("{save} needs {needed} page(s) and the pak has {free} free")),
        Err(pak::CopyError::FromNotFormatted) => refused("the pak copied from is not a formatted Controller Pak".into()),
        Err(pak::CopyError::ToNotFormatted) => refused("the pak copied to is not formatted".into()),
        Err(pak::CopyError::BrokenChain) => refused(format!("{save}'s pages are not a whole chain, so it cannot be copied")),
    }
}

pub fn delete(image: &[u8], save: &str) -> Result<Vec<u8>, Fault> {
    if formatted(image).is_none() {
        return refused("the pak is not formatted".into());
    }
    let Some(note) = find(image, save) else { return refused(format!("no note {save} is on the pak")) };
    let mut edited = image.to_vec();
    match pak::delete(&mut edited, note) {
        Ok(()) => Ok(edited),
        Err(pak::DeleteError::NotFormatted) => refused("the pak is not formatted".into()),
    }
}
