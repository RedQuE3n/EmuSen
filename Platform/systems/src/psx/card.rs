//! The memory card's module: `emusen-psx-card`'s file system with each save named by its directory name, titled by
//! its title frame and given its game's serial, and each refusal in the C# module's words (EmuSen_SaveMedia.md §6.2).

use crate::media::{Fault, Listing, Save, ascii};
use emusen_psx_card as card;

/// The serial a save's name begins with after its region, BASLUS-00067 giving SLUS-00067; none for a name of no such form.
pub fn game_of(name: &str) -> Option<String> {
    let b = name.as_bytes();
    (b.len() >= 12 && b[0] == b'B' && matches!(b[1], b'A' | b'E' | b'I') && b[6] == b'-').then(|| name[2..12].to_string())
}

/// A save's title: its title frame's Shift-JIS read with full-width letters as their ASCII, trimmed; none where the
/// frame is no title or its text is blank.
pub fn title_of(image: &[u8], entry: usize) -> Option<String> {
    let text = super::shift_jis::nfkc(&super::shift_jis::decode(card::title(image, entry)?));
    let text = emusen_dotnet::text::trim(&text);
    (!text.is_empty()).then(|| text.to_string())
}

fn name_of(image: &[u8], entry: usize) -> String {
    ascii(card::name(image, entry))
}

pub fn read(image: &[u8]) -> Listing {
    if image.len() != card::CARD_LENGTH {
        return Listing::unreadable(format!("a memory card is {} bytes; this file is {}", card::CARD_LENGTH, image.len()));
    }
    if !card::is_formatted(image) {
        return Listing {
            formatted: false,
            saves: Vec::new(),
            free: card::BLOCKS,
            problem: Some("not formatted: a game or the console formats it when it first saves".into()),
        };
    }
    let saves = card::saves(image)
        .map(|s| {
            let name = name_of(image, s.entry);
            Save { title: title_of(image, s.entry).unwrap_or_else(|| name.clone()), units: s.blocks, game: game_of(&name), damaged: s.damaged, id: name }
        })
        .collect();
    Listing { formatted: true, saves, free: card::free_blocks(image), problem: None }
}

/// The first entry of the save named `id`.
fn find(image: &[u8], id: &str) -> Option<usize> {
    (0..card::BLOCKS).find(|&e| card::state(image, e) == card::FIRST && name_of(image, e) == id)
}

fn refused<T>(words: String) -> Result<T, Fault> {
    Err(Fault::Refused(words))
}

pub fn copy(from: &[u8], save: &str, to: &[u8]) -> Result<Vec<u8>, Fault> {
    if !card::is_formatted(from) {
        return refused("the card copied from is not a formatted memory card".into());
    }
    if !card::is_formatted(to) {
        return refused("the card copied to is not formatted".into());
    }
    let Some(first) = find(from, save) else { return refused(format!("no save named {save} is on the card")) };
    if card::chain(from, first).is_none() {
        return refused(format!("{save}'s blocks are not a whole chain, so it cannot be copied"));
    }
    if find(to, save).is_some() {
        return refused(format!("the card already holds a save named {save}"));
    }
    let mut image = to.to_vec();
    match card::copy(from, first, &mut image) {
        Ok(()) => Ok(image),
        Err(card::CopyError::NoRoom { needed, free }) => refused(format!("{save} needs {needed} block(s) and the card has {free} free")),
        Err(card::CopyError::FromNotFormatted) => refused("the card copied from is not a formatted memory card".into()),
        Err(card::CopyError::ToNotFormatted) => refused("the card copied to is not formatted".into()),
        Err(card::CopyError::BrokenChain) => refused(format!("{save}'s blocks are not a whole chain, so it cannot be copied")),
    }
}

pub fn delete(image: &[u8], save: &str) -> Result<Vec<u8>, Fault> {
    if !card::is_formatted(image) {
        return refused("the card is not formatted".into());
    }
    let Some(first) = find(image, save) else { return refused(format!("no save named {save} is on the card")) };
    let mut edited = image.to_vec();
    card::delete(&mut edited, first);
    Ok(edited)
}
