//! Save media as the runtime places and lists them, the C# `MemoryCards`: a console's media grouped from its engine's
//! battery files, each medium's mode (per game, shared or named) and names as `consoles.json` stores them, the drawer's
//! cards and the games' own, and the card manager over them through `emusen-savemedia` (EmuSen_SaveMedia.md §3, §6).
//! Which medium a file is, by its extension, is core info's slots' (EmuSen_Backend.md F28).

use emusen_core_host::Discovery;
use emusen_dotnet::text::equals_ignore_case;
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::tree::Tree;
use emusen_savemedia::{Medium, Media};
use emusen_systems::media::Module;

use crate::info::{self, BatteryFile, BatteryKind, SystemDetail};
use crate::json::Json;

pub const PER_GAME: &str = "game";
pub const SHARED: &str = "shared";
pub const NAMED: &str = "named";
pub const MODE_KEY: &str = "Cards";
pub const CONSOLE_MODE_KEY: &str = "ConsoleMemory";
pub const NAMES_OFFERED: usize = 8;
pub const CARDS_FOLDER: &str = "Cards";
pub const CONSOLE_FOLDER: &str = "Console";

/// A medium's entry as the pack names it: its id, words and format module.
#[derive(Clone, Debug)]
pub struct Entry {
    pub id: String,
    pub title: String,
    pub noun: String,
    pub module: Option<&'static Module>,
}

/// The table's medium of this id, `SaveMediumFormats.Entry`.
pub fn entry(id: Option<&str>) -> Option<Entry> {
    let id = id?;
    let row = emusen_systems::table().ok()?.iter().flat_map(|s| s.media.iter()).find(|m| m.id == id)?;
    Some(Entry { id: row.id.clone(), title: row.title.clone(), noun: row.noun.clone(), module: emusen_systems::media::module(id) })
}

/// The extension of a suffix, its last part: ".mcr" of ".2.mcr", `ExtensionOf`.
pub fn extension_of(suffix: &str) -> String {
    let e = path::extension(Style::HOST, suffix);
    if e.len() > 1 { e.to_string() } else { suffix.to_string() }
}

/// One medium of a console, `SaveMedium`.
#[derive(Clone, Debug)]
pub struct SaveMedium {
    pub console: String,
    pub id: String,
    pub kind: BatteryKind,
    pub extension: String,
    pub title: String,
    pub noun: String,
    pub slots: Vec<BatteryFile>,
    pub ordinal: usize,
    pub entry: Option<Entry>,
}

/// The stored choice of a medium, `CardChoice`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    pub mode: String,
    pub names: Vec<String>,
}

impl Choice {
    pub fn apart(&self) -> bool {
        self.mode == SHARED || self.mode == NAMED
    }
}

impl SaveMedium {
    pub fn is_console(&self) -> bool {
        self.kind == BatteryKind::Console
    }

    pub fn mode_key(&self) -> String {
        if self.ordinal > 0 {
            format!("Media.{}", self.id)
        } else if self.is_console() {
            CONSOLE_MODE_KEY.to_string()
        } else {
            MODE_KEY.to_string()
        }
    }

    pub fn slot_key(&self, slot: usize) -> String {
        if self.ordinal > 0 {
            format!("Media.{}.{}", self.id, slot + 1)
        } else if self.is_console() {
            format!("{CONSOLE_MODE_KEY}{}", slot + 1)
        } else {
            format!("Card{}", slot + 1)
        }
    }

    pub fn default_name(&self, slot: usize) -> String {
        format!("{} {}", self.noun, slot + 1)
    }

    pub fn name_for(&self, choice: &Choice, slot: usize) -> String {
        if choice.mode == NAMED && slot < choice.names.len() { choice.names[slot].clone() } else { self.default_name(slot) }
    }

    pub fn slot_label(&self, slot: usize) -> String {
        self.slots[slot].label.clone().unwrap_or_else(|| if self.slots.len() == 1 { self.title.clone() } else { format!("Slot {}", slot + 1) })
    }

    pub fn folder(&self, tree: &Tree) -> String {
        medium_folder(tree, &self.console, self.is_console())
    }

    pub fn path_of(&self, tree: &Tree, name: &str) -> String {
        path::combine(Style::HOST, &self.folder(tree), &format!("{name}{}", self.extension))
    }

    /// The mode a medium has when nothing is stored: shared where a format module gives it the import, `DefaultMode`.
    pub fn default_mode(&self) -> &'static str {
        if self.kind != BatteryKind::Game && self.entry.as_ref().is_some_and(|e| e.module.is_some()) { SHARED } else { PER_GAME }
    }

    pub fn offered_key(&self) -> String {
        format!("{}Offered", self.mode_key())
    }

    /// The stored choice, `MemoryCards.Read`.
    pub fn read(&self, stored: &dyn Fn(&str) -> Option<String>) -> Choice {
        let kept = stored(&self.mode_key());
        let mode = match kept.as_deref() {
            Some(m @ (SHARED | NAMED | PER_GAME)) => m.to_string(),
            None => self.default_mode().to_string(),
            Some(_) => PER_GAME.to_string(),
        };
        let names = (0..self.slots.len()).map(|slot| stored(&self.slot_key(slot)).filter(|n| !n.is_empty() && is_name(n)).unwrap_or_else(|| self.default_name(slot))).collect();
        Choice { mode, names }
    }
}

/// `SaveLibrary.MediumFolder`.
pub fn medium_folder(tree: &Tree, console: &str, console_kind: bool) -> String {
    path::combine_all(Style::HOST, &[&tree.saves(), console, if console_kind { CONSOLE_FOLDER } else { CARDS_FOLDER }])
}

/// A folder's own files, links to files among them, `Directory.EnumerateFiles` without its subfolders: each path and name.
fn files_in(folder: &str) -> Vec<(String, String)> {
    let Ok(read) = std::fs::read_dir(folder) else { return Vec::new() };
    read.flatten()
        .filter(|e| !emusen_galaxia::dotnet_files::listed_as_directory(e))
        .map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            (emusen_galaxia::dotnet_files::join(folder, &name), name)
        })
        .collect()
}

/// The names of a kind's files with this extension in a console's folder, in order ignoring case, `MediumNames`.
pub fn medium_names(tree: &Tree, console: &str, console_kind: bool, extension: &str) -> Vec<String> {
    let folder = medium_folder(tree, console, console_kind);
    let mut names: Vec<String> = files_in(&folder)
        .into_iter()
        .filter(|(_, name)| name.ends_with(extension) && equals_ignore_case(path::extension(Style::HOST, name), extension))
        .map(|(at, _)| path::file_name_without_extension(Style::HOST, &at).to_string())
        .collect();
    names.sort_by(|a, b| emusen_dotnet::text::compare_ignore_case(a, b));
    names
}

/// A name that is one file in the folder, `MemoryCards.IsName`.
pub fn is_name(name: &str) -> bool {
    name.trim() == name && !name.is_empty() && !name.starts_with('.') && !name.contains(['\0', '/', '\\'])
}

/// A console's media grouped from its system's battery files, `MediaOf`.
pub fn media_of(console: &str, system: Option<&SystemDetail>) -> Vec<SaveMedium> {
    let slots: Vec<&BatteryFile> = system.and_then(|s| s.battery.as_ref()).map(|b| b.iter().filter(|f| f.kind != BatteryKind::Game).collect()).unwrap_or_default();
    let mut groups: Vec<(String, Vec<BatteryFile>)> = Vec::new();
    for f in slots {
        let key = f.medium.clone().unwrap_or_else(|| format!("{}{}", f.kind.word(), extension_of(&f.suffix)));
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, list)) => list.push(f.clone()),
            None => groups.push((key, vec![f.clone()])),
        }
    }
    let mut media: Vec<SaveMedium> = Vec::new();
    for (key, list) in groups {
        let first = &list[0];
        let extension = extension_of(&first.suffix);
        let entry = entry(first.medium.as_deref());
        let inside = first.kind == BatteryKind::Console;
        let ordinal = media.iter().filter(|m| m.kind == first.kind).count();
        media.push(SaveMedium {
            console: console.to_string(),
            id: key,
            kind: first.kind,
            title: entry.as_ref().map(|e| e.title.clone()).unwrap_or_else(|| if inside { "Console memory" } else { "Memory card" }.to_string()),
            noun: entry.as_ref().map(|e| e.noun.clone()).unwrap_or_else(|| if inside { "Memory" } else { "Card" }.to_string()),
            extension,
            slots: list,
            ordinal,
            entry,
        });
    }
    media
}

/// A file of a medium the card manager lists, `MediumFile`.
pub struct MediumFile {
    pub path: String,
    pub name: String,
    pub label: String,
    pub games_own: bool,
}

/// The drawer's cards of a medium, then each game's own, `CardsOf`.
pub fn cards_of(tree: &Tree, m: &SaveMedium) -> Vec<MediumFile> {
    let mut cards: Vec<MediumFile> = medium_names(tree, &m.console, m.is_console(), &m.extension)
        .into_iter()
        .map(|n| MediumFile { path: m.path_of(tree, &n), label: n.clone(), name: n, games_own: false })
        .collect();
    let own = path::combine(Style::HOST, &tree.saves(), &m.console);
    if !emusen_galaxia::dotnet_files::directory_exists(&own) {
        return cards;
    }
    let mut suffixes: Vec<(String, usize)> = m.slots.iter().enumerate().map(|(n, s)| (s.suffix.clone(), n)).collect();
    suffixes.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    let mut files: Vec<String> = files_in(&own).into_iter().filter(|(_, name)| equals_ignore_case(path::extension(Style::HOST, name), &m.extension)).map(|(at, _)| at).collect();
    files.sort_by(|a, b| emusen_dotnet::text::compare_ignore_case(a, b));
    for file in files {
        let name = path::file_name(Style::HOST, &file).to_string();
        let found = suffixes.iter().find(|(s, _)| emusen_dotnet::text::ends_with_ignore_case(&name, s) && name.len() > s.len());
        let (game, slot) = match found {
            Some((s, slot)) => (name[..name.len() - s.len()].to_string(), *slot),
            None => (path::file_name_without_extension(Style::HOST, &file).to_string(), 0),
        };
        let label = if slot > 0 { format!("{game}, {} (the game's own)", m.slot_label(slot)) } else { format!("{game} (the game's own)") };
        cards.push(MediumFile { path: file, name: game, label, games_own: true });
    }
    cards
}

/// The first of the noun's names no file in the drawer has, `NextName`.
pub fn next_name(tree: &Tree, m: &SaveMedium) -> String {
    let taken = medium_names(tree, &m.console, m.is_console(), &m.extension);
    (0..).map(|n| m.default_name(n)).find(|d| !taken.iter().any(|t| equals_ignore_case(t, d))).unwrap_or_default()
}

/// The medium whose import a frontend offers before a game starts, `ImportToOffer`.
pub fn import_to_offer<'m>(tree: &Tree, media: &'m [SaveMedium], stored: &dyn Fn(&str) -> Option<String>) -> Option<&'m SaveMedium> {
    media.iter().find(|m| {
        stored(&m.mode_key()).is_none()
            && m.default_mode() == SHARED
            && stored(&m.offered_key()).is_none()
            && !emusen_galaxia::dotnet_files::file_exists(&m.path_of(tree, &m.default_name(0)))
            && cards_of(tree, m).iter().any(|c| c.games_own)
    })
}

/// Which medium a file is, by its extension among the media every discovered engine's slots name.
pub struct ByExtension<'d> {
    pub discovery: &'d Discovery,
}

impl Media for ByExtension<'_> {
    fn medium_of(&self, file: &str) -> Option<Medium> {
        let extension = path::extension(Style::HOST, file);
        for c in self.discovery.found() {
            for s in info::systems(&c.sidecar.info_json) {
                for b in s.battery.iter().flatten() {
                    if let Some(id) = &b.medium
                        && equals_ignore_case(&extension_of(&b.suffix), extension)
                        && let Some(m) = Medium::by_id(id)
                    {
                        return Some(m);
                    }
                }
            }
        }
        None
    }
}

/// One medium as `cards_op`'s LIST gives it.
pub fn list_json(j: &mut Json, tree: &Tree, m: &SaveMedium, stored: &dyn Fn(&str) -> Option<String>, offered: bool) {
    let choice = m.read(stored);
    j.begin_object()
        .str("medium", &m.id)
        .str("title", &m.title)
        .str("noun", &m.noun)
        .str("extension", &m.extension)
        .str("kind", if m.is_console() { "console" } else { "removable" });
    match m.entry.as_ref().and_then(|e| e.module) {
        Some(module) => {
            j.int("capacity", module.capacity as i64).str("unit", module.unit);
        }
        None => {
            j.null("capacity").null("unit");
        }
    }
    j.str("mode", &choice.mode).str("mode_key", &m.mode_key()).bool("import_due", offered).str("next_name", &next_name(tree, m)).key_array("slots");
    for slot in 0..m.slots.len() {
        let name = m.name_for(&choice, slot);
        j.begin_object().str("label", &m.slot_label(slot)).str("name", &name).str("path", &m.path_of(tree, &name)).end_object();
    }
    j.end_array().key_array("cards");
    for card in cards_of(tree, m) {
        j.begin_object()
            .str("path", &card.path)
            .str("name", &card.name)
            .str("label", &card.label)
            .bool("games_own", card.games_own)
            .str_or_null("held_by", emusen_galaxia::medium_lock::holder_of(&card.path).as_deref())
            .end_object();
    }
    j.end_array().end_object();
}

/// A refusal: its status and words.
pub type Refused = (i32, String);

fn bad(words: String) -> Refused {
    (emusen_session_service::abi::status::BAD_ARGUMENT, words)
}

/// What the C# facade threw for the manager's failures, as the C# implementation answers it.
fn thrown(t: emusen_savemedia::Thrown) -> Refused {
    match t {
        emusen_savemedia::Thrown::Range => (emusen_session_service::abi::status::FAULT, "ArgumentOutOfRangeException: Specified argument was out of the range of valid values.".to_string()),
        emusen_savemedia::Thrown::Io(e) => (crate::abi::status::IO, e.to_string()),
    }
}

fn outcome_json(o: &emusen_savemedia::Outcome) -> String {
    let mut j = Json::new();
    j.begin_object().bool("done", o.done).str("words", &o.words).end_object();
    j.finish()
}

/// A console's media, as its engine's core info lists them, `MediaFor`.
pub fn media_for(catalog: &crate::catalog::Catalog, console: &str) -> Vec<SaveMedium> {
    media_of(console, catalog.engine_system(console).as_ref())
}

/// The card manager over a console's cards, `ManagedRuntime.CardsOp`.
pub fn cards_op(catalog: &crate::catalog::Catalog, tree: &Tree, op: u32, args: &str) -> Result<String, Refused> {
    let doc = emusen_dotnet::json::parse(if args.is_empty() { "{}" } else { args }).map_err(|e| bad(format!("the argument is not JSON: {e}")))?;
    let text = |k: &str| doc.get(k).and_then(|v| v.as_str()).map(str::to_string).ok_or_else(|| bad(format!("\"{k}\" must be given as a string")));
    let media = ByExtension { discovery: catalog.discovery };
    let manager = emusen_savemedia::Manager::new(&media);
    use crate::abi as a;
    let done = |d: emusen_savemedia::Done| d.map(|o| outcome_json(&o)).map_err(thrown);
    match op {
        a::CARDS_LIST => {
            let console = text("console")?;
            let stored = emusen_galaxia::consoles::load(tree);
            let value = |key: &str| stored.value(&console, key).map(str::to_string);
            let media = media_for(catalog, &console);
            let offer = import_to_offer(tree, &media, &value).map(|m| m.id.clone());
            let mut j = Json::new();
            j.begin_array();
            for m in &media {
                list_json(&mut j, tree, m, &value, offer.as_deref() == Some(m.id.as_str()));
            }
            j.end_array();
            Ok(j.finish())
        }
        a::CARDS_OPEN => {
            let at = text("path")?;
            let card = manager.open(&at).map_err(thrown)?.ok_or_else(|| (emusen_session_service::abi::status::ABSENT, format!("{} is of no medium a system pack names", path::file_name(Style::HOST, &at))))?;
            let mut j = Json::new();
            j.begin_object()
                .str("medium", &card.medium.id)
                .bool("exists", card.exists)
                .int("length", card.length as i64)
                .bool("formatted", card.listing.as_ref().is_some_and(|l| l.formatted))
                .int("free", card.listing.as_ref().map_or(0, |l| l.free as i64))
                .str_or_null("problem", match &card.listing {
                    None => Some("no format module reads this medium"),
                    Some(l) => l.problem.as_deref(),
                })
                .key_array("saves");
            for s in card.listing.iter().flat_map(|l| l.saves.iter()) {
                j.begin_object().str("id", &s.id).str("title", &s.title).int("units", s.units as i64).str_or_null("game", s.game.as_deref()).bool("damaged", s.damaged).end_object();
            }
            j.end_array().end_object();
            Ok(j.finish())
        }
        a::CARDS_COPY => done(manager.copy_save(&text("from")?, &text("save")?, &text("to")?)),
        a::CARDS_MOVE => done(manager.move_save(&text("from")?, &text("save")?, &text("to")?)),
        a::CARDS_DELETE => done(manager.delete_save(&text("card")?, &text("save")?)),
        a::CARDS_IMPORT => {
            let to = text("to")?;
            let from: Vec<String> = match doc.get("from") {
                Some(emusen_dotnet::json::Value::Array(items)) => items.iter().map(|v| v.as_str().unwrap_or("").to_string()).collect(),
                _ => return Err(bad("\"from\" must be given as a list of paths".to_string())),
            };
            done(manager.import(&to, &from))
        }
        a::CARDS_NEW => Ok(outcome_json(&manager.new_card(&text("path")?))),
        a::CARDS_DUPLICATE => Ok(outcome_json(&manager.duplicate(&text("from")?, &text("to")?))),
        a::CARDS_RENAME => done(manager.rename(&text("from")?, &text("to")?)),
        _ => Err((emusen_session_service::abi::status::NOT_SUPPORTED, format!("no cards op {op} in 1.{}", crate::abi::MINOR))),
    }
}

/// The files of a game's save media at one load, the flags its machine is made with, and the locks held on them,
/// `MediaPlacement`; the locks go when it is dropped.
#[derive(Default)]
pub struct Placement {
    pub files: Vec<(u32, crate::battery::BatterySave)>,
    /// The console media's slots a new machine finds formatted where no file exists yet, by their module.
    pub fresh: Vec<(u32, &'static str)>,
    pub removable_apart: bool,
    pub console_apart: bool,
    pub notices: Vec<String>,
    locks: Vec<emusen_galaxia::medium_lock::MediumLock>,
}

impl Placement {
    pub fn file(&self, which: u32) -> Option<&crate::battery::BatterySave> {
        self.files.iter().find(|(w, _)| *w == which).map(|(_, s)| s)
    }

    fn set(&mut self, which: u32, save: crate::battery::BatterySave) {
        match self.files.iter_mut().find(|(w, _)| *w == which) {
            Some(slot) => slot.1 = save,
            None => self.files.push((which, save)),
        }
    }
}

/// Each slot's file at a load: under shared or named its medium's file, locked for `holder`, else the game's own;
/// none with no battery, `MemoryCards.Place`.
pub fn place(
    tree: &Tree,
    console: &str,
    system: Option<&SystemDetail>,
    choices: &[(String, Choice)],
    holder: &str,
    own: &dyn Fn(&BatteryFile) -> crate::battery::BatterySave,
    disabled: bool,
) -> Placement {
    let mut placed = Placement::default();
    let mut taken: Vec<String> = Vec::new();
    for medium in media_of(console, system) {
        let choice = choices.iter().find(|(id, _)| *id == medium.id).map(|(_, c)| c.clone()).unwrap_or(Choice { mode: PER_GAME.to_string(), names: Vec::new() });
        for slot in 0..medium.slots.len() {
            let file = medium.slots[slot].clone();
            if medium.is_console()
                && let Some(module) = medium.entry.as_ref().and_then(|e| e.module)
            {
                match placed.fresh.iter_mut().find(|(w, _)| *w == file.which) {
                    Some(f) => f.1 = module.id,
                    None => placed.fresh.push((file.which, module.id)),
                }
            }
            if choice.apart() {
                let name = medium.name_for(&choice, slot);
                if let Some(save) = apart(tree, &medium, &name, slot, holder, &mut taken, &mut placed, disabled) {
                    placed.set(file.which, save);
                    if medium.is_console() {
                        placed.console_apart = true;
                    } else {
                        placed.removable_apart = true;
                    }
                    continue;
                }
            }
            let save = own(&file);
            if let Some(at) = &save.path
                && let Some(held) = emusen_galaxia::medium_lock::try_take(at, holder, true)
            {
                placed.locks.push(held);
            }
            placed.set(file.which, save);
        }
    }
    placed
}

#[allow(clippy::too_many_arguments)]
fn apart(tree: &Tree, medium: &SaveMedium, name: &str, slot: usize, holder: &str, taken: &mut Vec<String>, placed: &mut Placement, disabled: bool) -> Option<crate::battery::BatterySave> {
    let save = crate::battery::BatterySave::medium(tree, &medium.console, medium.is_console(), name, &medium.extension, disabled);
    let key = medium.path_of(tree, name);
    if taken.iter().any(|t| equals_ignore_case(t, &key)) {
        placed.notices.push(format!("{name} is already in another slot; {} keeps the game's own {}.", medium.slot_label(slot), emusen_savemedia::in_text(&medium.noun)));
        return None;
    }
    taken.push(key);
    let Some(at) = save.path.clone() else { return Some(save) };
    match emusen_galaxia::medium_lock::try_take(&at, holder, true) {
        Some(held) => {
            placed.locks.push(held);
            Some(save)
        }
        None => {
            let by = emusen_galaxia::medium_lock::holder_of(&at).unwrap_or_else(|| "another program".to_string());
            placed.notices.push(format!("{name} is in use by {by}; {} keeps the game's own {} until it is free.", medium.slot_label(slot), emusen_savemedia::in_text(&medium.noun)));
            None
        }
    }
}

/// Every medium's stored choice, by medium id, `MemoryCards.Stored`.
pub fn stored_choices(media: &[SaveMedium], stored: &dyn Fn(&str) -> Option<String>) -> Vec<(String, Choice)> {
    media.iter().map(|m| (m.id.clone(), m.read(stored))).collect()
}
