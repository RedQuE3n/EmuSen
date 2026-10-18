//! The card manager, the C# `SaveMediaManager` (EmuSen_SaveMedia.md §6): over files, each medium's format module in
//! emusen-systems and Galaxia's medium lock, and needing no core. Which medium a file is, by its extension, is the
//! host's to say (EmuSen_Backend.md F28); everything after that is here, words included. A change to a card is one
//! atomic write under the manager's own lock, refused where a game or another program holds the card.

use emusen_galaxia::atomic;
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::medium_lock;
use emusen_galaxia::tree::file_exists;
use emusen_systems::media::{self, Fault, Listing, Module, Save};
use std::cell::RefCell;

/// Who the manager's own lock names.
pub const HOLDER: &str = "the card manager";

/// A medium as its system's row names it, with its format module where one reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Medium {
    pub id: String,
    pub title: String,
    pub noun: String,
    pub module: Option<&'static Module>,
}

impl Medium {
    /// The medium the table names by this id, its words from its system's row; none for an id no row names.
    pub fn by_id(id: &str) -> Option<Medium> {
        let table = emusen_systems::table().ok()?;
        let row = table.iter().flat_map(|s| s.media.iter()).find(|m| m.id == id)?;
        Some(Medium { id: row.id.clone(), title: row.title.clone(), noun: row.noun.clone(), module: media::module(id) })
    }
}

/// Which medium a file is: the host's answer, by the file's extension among the packs' media.
pub trait Media {
    fn medium_of(&self, file: &str) -> Option<Medium>;
}

/// What an operation did, or why it did nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub done: bool,
    pub words: String,
}

fn ok(words: String) -> Outcome {
    Outcome { done: true, words }
}

fn no(words: String) -> Outcome {
    Outcome { done: false, words }
}

/// What the C# threw: a damaged directory reaching outside its image, or a rename the system refused.
#[derive(Debug)]
pub enum Thrown {
    Range,
    Io(std::io::Error),
}

pub type Done = Result<Outcome, Thrown>;

/// A card as the manager reads it: whether a file is there, its length, and its saves where a module reads them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Card {
    pub medium: Medium,
    pub exists: bool,
    pub length: u64,
    pub listing: Option<Listing>,
}

/// `MediumWords.InText`: a medium's noun or title inside a sentence, every word lower case but an acronym.
pub fn in_text(words: &str) -> String {
    words
        .split(' ')
        .map(|w| {
            let acronym = emusen_dotnet::text::utf16_len(w) > 1 && w.chars().all(|c| !emusen_dotnet::text::is_letter(c) || c.is_uppercase());
            if acronym { w.to_string() } else { emusen_dotnet::text::to_lower(w) }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// `SaveMediaManager.Find`: a save by its id, or by its place in the card's list from 1.
pub fn find<'a>(listing: &'a Listing, save: &str) -> Option<&'a Save> {
    listing.saves.iter().find(|s| s.id == save).or_else(|| {
        let n = emusen_dotnet::number::parse_i32(save)?;
        (n >= 1 && n as usize <= listing.saves.len()).then(|| &listing.saves[n as usize - 1])
    })
}

fn show(file: &str) -> &str {
    path::file_name(Style::HOST, file)
}

fn module_read(module: &Module, image: &[u8]) -> Result<Listing, Thrown> {
    media::read(module.id, image).expect("a module reads its own medium").map_err(|_| Thrown::Range)
}

enum Edit {
    Image(Vec<u8>),
    Refused(String),
}

fn edited(result: Option<Result<Vec<u8>, Fault>>) -> Result<Edit, Thrown> {
    match result.expect("a module edits its own medium") {
        Ok(image) => Ok(Edit::Image(image)),
        Err(Fault::Refused(words)) => Ok(Edit::Refused(words)),
        Err(Fault::Outside) => Err(Thrown::Range),
    }
}

/// The manager over a host's media, with what it would report through `ConfigDiagnostics` kept for the host.
pub struct Manager<'a, M: Media> {
    media: &'a M,
    said: RefCell<Vec<String>>,
}

impl<'a, M: Media> Manager<'a, M> {
    pub fn new(media: &'a M) -> Self {
        Manager { media, said: RefCell::new(Vec::new()) }
    }

    /// The diagnostics a read or a write left, in order, taken.
    pub fn said(&self) -> Vec<String> {
        std::mem::take(&mut self.said.borrow_mut())
    }

    /// `AtomicFile.TryRead`: none for a missing file, and for one that will not read, said.
    fn try_read(&self, file: &str) -> Option<Vec<u8>> {
        match atomic::try_read(file) {
            Ok(bytes) => bytes,
            Err(words) => {
                self.said.borrow_mut().push(words);
                None
            }
        }
    }

    fn write(&self, file: &str, bytes: &[u8]) -> bool {
        match atomic::write(file, bytes) {
            Ok(()) => true,
            Err(words) => {
                self.said.borrow_mut().push(words);
                false
            }
        }
    }

    fn held_by(&self, file: &str) -> Option<String> {
        medium_lock::holder_of(file)
    }

    fn in_use(&self, file: &str) -> String {
        format!("{} is in use by {}", show(file), self.held_by(file).unwrap_or_else(|| "another program".to_string()))
    }

    /// `SaveMediaManager.Open`: none for a file of no medium.
    pub fn open(&self, file: &str) -> Result<Option<Card>, Thrown> {
        let Some(medium) = self.media.medium_of(file) else { return Ok(None) };
        let bytes = if file_exists(file) { self.try_read(file) } else { None };
        let listing = match (medium.module, &bytes) {
            (Some(module), None) => Some(Listing { formatted: false, saves: Vec::new(), free: module.capacity, problem: Some("no file yet: a new, blank card".to_string()) }),
            (Some(module), Some(b)) => Some(module_read(module, b)?),
            (None, _) => None,
        };
        Ok(Some(Card { medium, exists: bytes.is_some(), length: bytes.as_ref().map_or(0, |b| b.len() as u64), listing }))
    }

    fn readable(&self, file: &str) -> Result<(&'static Module, Vec<u8>), String> {
        let medium = self.media.medium_of(file);
        let Some(module) = medium.as_ref().and_then(|m| m.module) else {
            return Err(if medium.is_none() { format!("{} is of no medium a system pack names", show(file)) } else { format!("no format module reads {}'s medium", show(file)) });
        };
        self.try_read(file).map(|bytes| (module, bytes)).ok_or_else(|| format!("{} could not be read", show(file)))
    }

    /// An absent or all-zero card formatted first, as a new card is; any other image as it is.
    fn formatted(module: &Module, image: Vec<u8>) -> Vec<u8> {
        if image.iter().all(|&b| b == 0) { media::format(module.id).unwrap_or(image) } else { image }
    }

    /// One write of a card under the manager's own lock, refused where a game or another program holds it.
    fn edit(&self, file: &str, change: impl FnOnce(Vec<u8>) -> Result<Result<(Vec<u8>, String), String>, Thrown>) -> Done {
        if self.media.medium_of(file).is_none() {
            return Ok(no(format!("{} is of no medium a system pack names", show(file))));
        }
        if let Some(holder) = self.held_by(file) {
            return Ok(no(format!("{} is in use by {holder}; it can be changed once that ends", show(file))));
        }
        let Some(_held) = medium_lock::try_take(file, HOLDER, false) else { return Ok(no(self.in_use(file))) };
        let image = if file_exists(file) { self.try_read(file).unwrap_or_default() } else { Vec::new() };
        match change(image)? {
            Err(words) => Ok(no(words)),
            Ok((image, words)) => Ok(if self.write(file, &image) { ok(words) } else { no(format!("{} could not be written", show(file))) }),
        }
    }

    fn write_under_lock(&self, file: &str, bytes: &[u8], words: String) -> Outcome {
        let Some(_held) = medium_lock::try_take(file, HOLDER, false) else { return no(self.in_use(file)) };
        if self.write(file, bytes) { ok(words) } else { no(format!("{} could not be written", show(file))) }
    }

    /// `SaveMediaManager.CopySave`.
    pub fn copy_save(&self, from: &str, save: &str, to: &str) -> Done {
        self.edit(to, |target| {
            let (module, source) = match self.readable(from) {
                Ok(r) => r,
                Err(problem) => return Ok(Err(problem)),
            };
            if self.media.medium_of(to).map(|m| m.id) != Some(module.id.to_string()) {
                return Ok(Err(format!("{} is not a card of the same medium as {}", show(to), show(from))));
            }
            let listing = module_read(module, &source)?;
            let Some(entry) = find(&listing, save) else { return Ok(Err(format!("no save {save} is on {}", show(from)))) };
            Ok(match edited(media::copy(module.id, &source, &entry.id, &Self::formatted(module, target)))? {
                Edit::Image(image) => Ok((image, format!("{} ({}) copied onto {}", entry.title, entry.id, show(to)))),
                Edit::Refused(words) => Err(words),
            })
        })
    }

    /// `SaveMediaManager.MoveSave`: a copy, read back, then the delete; the first card loses the save only once the second holds it.
    pub fn move_save(&self, from: &str, save: &str, to: &str) -> Done {
        let (module, source) = match self.readable(from) {
            Ok(r) => r,
            Err(problem) => return Ok(no(problem)),
        };
        let listing = module_read(module, &source)?;
        let Some(entry) = find(&listing, save).cloned() else { return Ok(no(format!("no save {save} is on {}", show(from)))) };
        if let Some(holder) = self.held_by(from) {
            return Ok(no(format!("{} is in use by {holder}, so nothing is moved off it", show(from))));
        }
        let copied = self.copy_save(from, &entry.id, to)?;
        if !copied.done {
            return Ok(copied);
        }
        let back = match self.try_read(to) {
            Some(written) => module_read(module, &written)?.saves.iter().any(|s| s.id == entry.id),
            None => false,
        };
        if !back {
            return Ok(no(format!("{} did not read back from {}, so {} keeps it", entry.id, show(to), show(from))));
        }
        let deleted = self.delete_save(from, &entry.id)?;
        Ok(if deleted.done { ok(format!("{} ({}) moved from {} to {}", entry.title, entry.id, show(from), show(to))) } else { deleted })
    }

    /// `SaveMediaManager.DeleteSave`: as the console's manager deletes, which the medium's module says how.
    pub fn delete_save(&self, card: &str, save: &str) -> Done {
        self.edit(card, |image| {
            let Some(module) = self.media.medium_of(card).and_then(|m| m.module) else {
                return Ok(Err(format!("no format module reads {}'s medium, so its saves cannot be told apart", show(card))));
            };
            let listing = module_read(module, &image)?;
            let Some(entry) = find(&listing, save) else { return Ok(Err(format!("no save {save} is on {}", show(card)))) };
            Ok(match edited(media::delete(module.id, &image, &entry.id))? {
                Edit::Image(edited) => Ok((edited, format!("{} ({}) deleted from {}", entry.title, entry.id, show(card)))),
                Edit::Refused(words) => Err(words),
            })
        })
    }

    /// `SaveMediaManager.Import`: every save of the given cards onto one card, all or none (EmuSen_SaveMedia.md §6.6).
    pub fn import(&self, to: &str, from: &[String]) -> Done {
        self.edit(to, |target| {
            let Some(module) = self.media.medium_of(to).and_then(|m| m.module) else {
                return Ok(Err(format!("no format module reads {}'s medium; a game's whole card can be duplicated onto a new name instead", show(to))));
            };
            let mut image = Self::formatted(module, target);
            let there = module_read(module, &image)?;
            if !there.formatted {
                return Ok(Err(format!("{} is not a formatted card: {}", show(to), there.problem.as_deref().unwrap_or(""))));
            }
            let mut taken: Vec<(&str, Vec<u8>, Save)> = Vec::new();
            let mut present = 0usize;
            let whole_to = path::full_path(to);
            for card in from {
                if path::full_path(card) == whole_to {
                    continue;
                }
                if self.media.medium_of(card).map(|m| m.id) != Some(module.id.to_string()) {
                    return Ok(Err(format!("{} is not a card of the same medium as {}", show(card), show(to))));
                }
                if !file_exists(card) {
                    continue;
                }
                let Some(source) = self.try_read(card) else { return Ok(Err(format!("{} could not be read", show(card)))) };
                let listing = module_read(module, &source)?;
                if !listing.formatted {
                    continue;
                }
                for save in listing.saves {
                    if save.damaged {
                        return Ok(Err(format!("{} on {} is damaged and cannot be copied; nothing was imported", save.id, show(card))));
                    }
                    if there.saves.iter().any(|s| s.id == save.id) {
                        present += 1;
                        continue;
                    }
                    if let Some((other, _, _)) = taken.iter().find(|(_, _, s)| s.id == save.id) {
                        return Ok(Err(format!("{} and {} both hold a save named {}; copy the one wanted by itself. Nothing was imported", show(other), show(card), save.id)));
                    }
                    taken.push((card.as_str(), source.clone(), save));
                }
            }
            let needed: usize = taken.iter().map(|(_, _, s)| s.units).sum();
            if needed > there.free {
                let lines: Vec<String> = taken.iter().map(|(card, _, s)| format!("  {} {}(s)  {}  {}  (from {})", s.units, module.unit, s.id, s.title, show(card))).collect();
                return Ok(Err(format!(
                    "{} has {} free {}(s) of {} and these need {needed}, so nothing was imported:\n{}",
                    show(to),
                    there.free,
                    module.unit,
                    module.capacity,
                    lines.join("\n")
                )));
            }
            for (_, source, save) in &taken {
                match edited(media::copy(module.id, source, &save.id, &image))? {
                    Edit::Image(next) => image = next,
                    Edit::Refused(words) => return Ok(Err(format!("{}: {words}; nothing was imported", save.id))),
                }
            }
            let kept = if present == 0 { String::new() } else { format!("; {present} already on it left as they are") };
            Ok(Ok((image, format!("{} save(s), {needed} {}(s), imported onto {}{kept}", taken.len(), module.unit, show(to)))))
        })
    }

    /// `SaveMediaManager.New`: formatted where the medium's module can, else no file, which the game formats as a new card.
    pub fn new_card(&self, file: &str) -> Outcome {
        let Some(medium) = self.media.medium_of(file) else { return no(format!("{} is of no medium a system pack names", show(file))) };
        if file_exists(file) {
            return no(format!("{} already exists", show(file)));
        }
        match medium.module.and_then(|m| media::format(m.id)) {
            None => ok(format!("{} is named; with no file yet it is a new, blank {}, which the game formats", show(file), in_text(&medium.noun))),
            Some(blank) => self.write_under_lock(file, &blank, format!("{} made, a formatted {}", show(file), in_text(&medium.title))),
        }
    }

    fn one_medium(&self, from: &str, to: &str) -> Option<Medium> {
        let medium = self.media.medium_of(from)?;
        (self.media.medium_of(to).map(|m| m.id) == Some(medium.id.clone())).then_some(medium)
    }

    /// `SaveMediaManager.Duplicate`: a whole card to a new name, read as bytes, which needs no module.
    pub fn duplicate(&self, from: &str, to: &str) -> Outcome {
        if self.one_medium(from, to).is_none() {
            return no(format!("{} and {} are not files of one medium", show(from), show(to)));
        }
        if file_exists(to) {
            return no(format!("{} already exists; nothing is written over a card", show(to)));
        }
        let Some(bytes) = self.try_read(from) else { return no(format!("{} could not be read", show(from))) };
        self.write_under_lock(to, &bytes, format!("{} copied whole to {}", show(from), show(to)))
    }

    /// `SaveMediaManager.Rename`, under the manager's lock on the first name; the system's refusal is thrown, as `File.Move`'s was.
    pub fn rename(&self, from: &str, to: &str) -> Done {
        if self.one_medium(from, to).is_none() {
            return Ok(no(format!("{} and {} are not files of one medium", show(from), show(to))));
        }
        if !file_exists(from) {
            return Ok(no(format!("{} does not exist", show(from))));
        }
        if file_exists(to) {
            return Ok(no(format!("{} already exists", show(to))));
        }
        if let Some(holder) = self.held_by(from) {
            return Ok(no(format!("{} is in use by {holder}", show(from))));
        }
        let Some(_held) = medium_lock::try_take(from, HOLDER, false) else { return Ok(no(self.in_use(from))) };
        move_file(from, to).map_err(Thrown::Io)?;
        Ok(ok(format!("{} renamed {}", show(from), show(to))))
    }
}

/// `File.Move` without overwriting: a rename, or across file systems a copy and then the first file deleted; a folder
/// of the new name is a file that already exists, as .NET's link-then-unlink finds it.
fn move_file(from: &str, to: &str) -> std::io::Result<()> {
    if std::fs::symlink_metadata(to).is_ok() {
        return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, format!("The file '{to}' already exists.")));
    }
    match std::fs::rename(from, to) {
        Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices => {
            std::fs::copy(from, to)?;
            std::fs::remove_file(from)
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct ByExtension;

    impl Media for ByExtension {
        fn medium_of(&self, file: &str) -> Option<Medium> {
            let ext = file.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase())?;
            let ids: HashMap<&str, &str> = [("mcr", "psx.card"), ("mpk", "n64.pak"), ("brm", "mcd.backup"), ("crm", "mcd.ramcart")].into_iter().collect();
            Medium::by_id(ids.get(ext.as_str())?)
        }
    }

    struct Dir(std::path::PathBuf);

    impl Dir {
        fn new(label: &str) -> Dir {
            let d = std::env::temp_dir().join(format!("emusen-savemedia-{label}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&d);
            std::fs::create_dir_all(&d).unwrap();
            Dir(d)
        }
        fn file(&self, name: &str) -> String {
            self.0.join(name).to_string_lossy().into_owned()
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn words_inside_a_sentence_keep_acronyms() {
        assert_eq!(in_text("Backup RAM Cartridge"), "backup RAM cartridge");
        assert_eq!(in_text("Memory Card"), "memory card");
        assert_eq!(in_text("A"), "a");
    }

    #[test]
    fn a_new_card_is_formatted_listed_and_refused_twice_and_a_held_card_is_not_changed() {
        let d = Dir::new("new");
        let m = Manager::new(&ByExtension);
        let card = d.file("Blue.mcr");
        assert_eq!(m.new_card(&card), ok("Blue.mcr made, a formatted memory card".into()));
        assert_eq!(m.new_card(&card), no("Blue.mcr already exists".into()));
        let listed = m.open(&card).unwrap().unwrap();
        assert_eq!((listed.exists, listed.listing.unwrap().free), (true, 15));
        let game = medium_lock::try_take(&card, "Game X", false).unwrap();
        assert_eq!(m.delete_save(&card, "1").unwrap(), no("Blue.mcr is in use by Game X; it can be changed once that ends".into()));
        drop(game);
        assert_eq!(m.delete_save(&card, "1").unwrap(), no("no save 1 is on Blue.mcr".into()));
        assert_eq!(m.new_card(&d.file("x.txt")), no("x.txt is of no medium a system pack names".into()));
    }

    #[test]
    fn a_card_is_duplicated_and_renamed_but_never_written_over() {
        let d = Dir::new("dup");
        let m = Manager::new(&ByExtension);
        let (a, b, c) = (d.file("a.brm"), d.file("b.brm"), d.file("c.mcr"));
        assert!(m.new_card(&a).done);
        assert_eq!(m.duplicate(&a, &c), no("a.brm and c.mcr are not files of one medium".into()));
        assert_eq!(m.duplicate(&a, &b), ok("a.brm copied whole to b.brm".into()));
        assert_eq!(m.duplicate(&a, &b), no("b.brm already exists; nothing is written over a card".into()));
        assert_eq!(m.rename(&a, &b).unwrap(), no("b.brm already exists".into()));
        std::fs::remove_file(&b).unwrap();
        assert_eq!(m.rename(&a, &b).unwrap(), ok("a.brm renamed b.brm".into()));
        assert!(!file_exists(&a) && file_exists(&b));
    }
}
