//! The Sega CD backup RAM's module, the internal RAM's and a RAM cartridge's alike: `emusen-mcd-bram`'s file system
//! with each file named by its directory name, and each refusal in the C# module's words (EmuSen_SaveMedia.md §6.2,
//! Nephrite_Bram.md).

use crate::media::{Fault, Listing, Save, ascii};
use emusen_mcd_bram as bram;

fn refused<T>(words: String) -> Result<T, Fault> {
    Err(Fault::Refused(words))
}

/// Every entry of the directory, or `Outside` where one lies outside the volume, as the C# reading threw.
fn entries(image: &[u8]) -> Result<Vec<bram::Entry>, Fault> {
    bram::entries(image).map(|e| e.ok_or(Fault::Outside)).collect()
}

fn formatted(image: &[u8]) -> bool {
    bram::size_ok(image.len()) && bram::kind(image) == bram::Kind::Sega
}

pub fn read(image: &[u8]) -> Result<Listing, Fault> {
    if !bram::size_ok(image.len()) {
        return Ok(Listing::unreadable(format!("a Sega CD backup RAM is 8 KiB to 512 KiB, a power of two; this file is {} bytes", image.len())));
    }
    let not = |problem: String| Ok(Listing { formatted: false, saves: Vec::new(), free: 0, problem: Some(problem) });
    match bram::kind(image) {
        bram::Kind::Unformatted => return not("not formatted: a game or the console formats it when it first saves".into()),
        bram::Kind::Other => return not(format!("another format, {}", ascii(bram::format_name(image)))),
        bram::Kind::Sega => {}
    }
    let saves = entries(image)?
        .into_iter()
        .map(|e| {
            let name = ascii(&e.name);
            Save { title: name.trim_end_matches('_').to_string(), id: name, units: e.blocks, game: None, damaged: e.damaged }
        })
        .collect();
    Ok(Listing { formatted: true, saves, free: bram::counts(image).0, problem: None })
}

/// The index, from 1, of the entry named `save`.
fn find(entries: &[bram::Entry], save: &str) -> Option<usize> {
    entries.iter().position(|e| ascii(&e.name) == save).map(|k| k + 1)
}

pub fn copy(from: &[u8], save: &str, to: &[u8]) -> Result<Vec<u8>, Fault> {
    if !formatted(from) {
        return refused("the backup RAM copied from is not a formatted Sega CD backup RAM".into());
    }
    if !formatted(to) {
        return refused("the backup RAM copied to is not formatted".into());
    }
    let source = entries(from)?;
    let Some(index) = find(&source, save) else { return refused(format!("no save named {save} is on the backup RAM")) };
    if source[index - 1].damaged {
        return refused(format!("{save}'s directory entry is damaged, so it cannot be copied"));
    }
    if find(&entries(to)?, save).is_some() {
        return refused(format!("the backup RAM already holds a save named {save}"));
    }
    let mut image = to.to_vec();
    match bram::copy(from, index, &mut image) {
        Ok(()) => Ok(image),
        Err(bram::EditError::NoRoom { needed, free }) => refused(format!("{save} needs {needed} block(s) and the backup RAM has {free} free")),
        Err(bram::EditError::Damaged) => refused(format!("{save}'s directory entry is damaged, so it cannot be copied")),
        Err(bram::EditError::FromNotFormatted) => refused("the backup RAM copied from is not a formatted Sega CD backup RAM".into()),
        Err(bram::EditError::ToNotFormatted | bram::EditError::NotFormatted) => refused("the backup RAM copied to is not formatted".into()),
        Err(bram::EditError::DirectoryOutside | bram::EditError::MoveOutside) => Err(Fault::Outside),
    }
}

pub fn delete(image: &[u8], save: &str) -> Result<Vec<u8>, Fault> {
    if !formatted(image) {
        return refused("the backup RAM is not formatted".into());
    }
    let Some(index) = find(&entries(image)?, save) else { return refused(format!("no save named {save} is on the backup RAM")) };
    let mut edited = image.to_vec();
    match bram::delete(image, index, &mut edited) {
        Ok(()) => Ok(edited),
        Err(bram::EditError::DirectoryOutside | bram::EditError::MoveOutside) => Err(Fault::Outside),
        Err(_) => refused("the backup RAM is not formatted".into()),
    }
}
