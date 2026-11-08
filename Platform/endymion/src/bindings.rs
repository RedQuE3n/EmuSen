//! The four binding files and the one reader and writer of each: keybindings.json, gamepadbindings.json,
//! gamepadprofiles.json and hotkeybindings.json. A file is read by Galaxia's rules for a hand-edited file and then by the
//! rules each file has always had (a file from before consoles, a control added after a file was written, a profile's
//! checks); it is written in serde's spelling. What crosses the platform library is the model as a program starts with
//! it. See EmuSen_Input.md §5.2.

use crate::slots::MAX_PLAYERS;
use crate::vocabulary::{GamepadButtons, HotkeyActions, Keys, PadButtons, PadControls, Vocabulary, Word};
use emusen_galaxia::config_json;
use emusen_galaxia::tree::Tree;
use indexmap::IndexMap;
use serde::de::{self, DeserializeOwned, Deserializer, Visitor};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use std::hash::Hash;

pub type Control = Word<PadControls>;
pub type Button = Word<PadButtons>;
pub type GamepadButton = Word<GamepadButtons>;
pub type Key = Word<Keys>;
pub type Action = Word<HotkeyActions>;

/// A console's pad controls to keyboard keys.
pub type KeyMap = IndexMap<Control, Key>;
/// A console's (or a player's) pad buttons to a gamepad's buttons.
pub type PadMap = IndexMap<Button, GamepadButton>;
/// The emulator's actions to keyboard keys.
pub type HotkeyMap = IndexMap<Action, Key>;

/// keybindings.json as read: each console's map.
pub type KeyBindings = IndexMap<String, KeyMap>;

/// gamepadbindings.json as read: each console's map, and the maps of players past the first who have their own.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct GamepadBindings {
    pub consoles: IndexMap<String, PadMap>,
    pub players: IndexMap<String, IndexMap<Player, PadMap>>,
}

/// One console's named profiles: the one its games start with, each profile's maps by player, and the games that name their own.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct ConsoleProfiles {
    pub default: Option<String>,
    pub profiles: IndexMap<String, IndexMap<Player, PadMap>>,
    #[serde(serialize_with = "games_or_null", deserialize_with = "null_or_games")]
    pub games: IndexMap<String, String>,
}

/// gamepadprofiles.json as read; Default, the console's own maps, stays in gamepadbindings.json.
pub type GamepadProfiles = IndexMap<String, ConsoleProfiles>;

/// The profile every console has, which is the maps of gamepadbindings.json.
pub const DEFAULT_PROFILE: &str = "Default";

/// The first control and button added after the files were first written: a file may predate it and those after it.
const FIRST_ADDED: i32 = 12;

const KEY_DEFAULTS: [(&str, &str); 24] = [
    ("Up", "Up"), ("Down", "Down"), ("Left", "Left"), ("Right", "Right"), ("B", "Z"), ("A", "X"), ("Y", "A"), ("X", "S"),
    ("L", "Q"), ("R", "W"), ("Start", "Return"), ("Select", "RightShift"), ("L2", "E"), ("R2", "R"), ("L3", "C"), ("R3", "V"),
    ("LeftStickUp", "I"), ("LeftStickDown", "K"), ("LeftStickLeft", "J"), ("LeftStickRight", "L"), ("RightStickUp", "T"),
    ("RightStickDown", "G"), ("RightStickLeft", "F"), ("RightStickRight", "H"),
];

/// By the buttons' places, not their labels (EmuSen_Settings_Reference.md §4.6).
const PAD_DEFAULTS: [(&str, &str); 14] = [
    ("Up", "DPadUp"), ("Down", "DPadDown"), ("Left", "DPadLeft"), ("Right", "DPadRight"), ("B", "South"), ("A", "East"),
    ("Y", "West"), ("X", "North"), ("L", "LeftShoulder"), ("R", "RightShoulder"), ("Start", "Start"), ("Select", "Back"),
    ("L3", "LeftStick"), ("R3", "RightStick"),
];

const HOTKEY_DEFAULTS: [(&str, &str); 10] = [
    ("FastForward", "Tab"), ("Rewind", "Back"), ("SaveState", "F5"), ("LoadState", "F8"), ("TogglePause", "P"),
    ("ToggleFullscreen", "F11"), ("ExitToLibrary", "Escape"), ("Screenshot", "F9"), ("ToggleBigPicture", "F10"),
    ("FrameAdvance", "OemPeriod"),
];

fn defaults<A: Vocabulary, B: Vocabulary>(table: &[(&str, &str)]) -> IndexMap<Word<A>, Word<B>> {
    table.iter().map(|&(a, b)| (Word::named(a).expect("a default's control"), Word::named(b).expect("a default's key"))).collect()
}

/// A console's keys as a program starts with them.
pub fn key_defaults() -> KeyMap {
    defaults(&KEY_DEFAULTS)
}

/// A console's gamepad buttons as a program starts with them.
pub fn pad_defaults() -> PadMap {
    defaults(&PAD_DEFAULTS)
}

/// The hotkeys as a program starts with them.
pub fn hotkey_defaults() -> HotkeyMap {
    defaults(&HOTKEY_DEFAULTS)
}

/// A map read from a file written before some of its entries existed: each default from `first` on that the map does not
/// have takes its default, unless that key or button is already bound to something else (EmuSen_Input.md §7.4).
fn add_new_defaults<A: Vocabulary, B: Vocabulary>(map: &mut IndexMap<Word<A>, Word<B>>, defaults: IndexMap<Word<A>, Word<B>>, first: i32) {
    for (entry, bound) in defaults {
        if entry.number() >= first && !map.contains_key(&entry) && !map.values().any(|v| *v == bound) {
            map.insert(entry, bound);
        }
    }
}

/// A player's number, a member's name in the files.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Player(pub i32);

impl Serialize for Player {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.0)
    }
}

struct PlayerVisitor;

impl Visitor<'_> for PlayerVisitor {
    type Value = Player;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("a player's number")
    }

    fn visit_str<E: de::Error>(self, text: &str) -> Result<Player, E> {
        text.parse().map(Player).map_err(|_| E::custom(format!("'{text}' is not a player's number.")))
    }
}

impl<'de> Deserialize<'de> for Player {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(PlayerVisitor)
    }
}

fn games_or_null<S: Serializer>(games: &IndexMap<String, String>, serializer: S) -> Result<S::Ok, S::Error> {
    if games.is_empty() { serializer.serialize_none() } else { games.serialize(serializer) }
}

fn null_or_games<'de, D: Deserializer<'de>>(deserializer: D) -> Result<IndexMap<String, String>, D::Error> {
    Option::<IndexMap<String, String>>::deserialize(deserializer).map(Option::unwrap_or_default)
}

/// Names compared as the C# compared them, without regard to case in any script.
fn same_name(a: &str, b: &str) -> bool {
    a.chars().flat_map(char::to_uppercase).eq(b.chars().flat_map(char::to_uppercase))
}

/// The entry of a name in a map keyed by names without regard to case, made at the end where there is none.
fn entry<'a, T: Default>(map: &'a mut IndexMap<String, T>, name: &str) -> &'a mut T {
    let index = match map.keys().position(|k| same_name(k, name)) {
        Some(index) => index,
        None => map.insert_full(name.to_string(), T::default()).0,
    };
    &mut map[index]
}

/// Each console named, at the defaults, once whatever its case.
fn seeded<T: Default>(consoles: &[&str], defaults: impl Fn() -> T) -> IndexMap<String, T> {
    let mut seeded = IndexMap::new();
    for console in consoles {
        if !seeded.keys().any(|k: &String| same_name(k, console)) {
            seeded.insert(console.to_string(), defaults());
        }
    }
    seeded
}

/// What a file holds by console: a file from before consoles, which every console inherits, or a map for each.
enum ByConsole<M> {
    Nothing,
    Flat(M),
    Each(IndexMap<String, Option<M>>),
}

/// A per-console file read by its shape: an object whose first member is not an object predates consoles.
fn by_console<M: DeserializeOwned>(text: &str) -> Result<ByConsole<M>, String> {
    let probe: Option<IndexMap<String, Value>> = config_json::read(text)?;
    Ok(match probe {
        None => ByConsole::Nothing,
        Some(probe) if probe.is_empty() => ByConsole::Nothing,
        Some(probe) if !probe[0].is_object() => ByConsole::Flat(config_json::read(text)?),
        Some(_) => ByConsole::Each(config_json::read(text)?),
    })
}

/// A player's file key, `<console> Player <n>` with n past 1, as its console and number.
fn player_key(key: &str) -> Option<(&str, i32)> {
    let (console, number) = key.rsplit_once(" Player ")?;
    let player = number.parse::<i32>().ok().filter(|_| !console.is_empty() && number.bytes().all(|b| b.is_ascii_digit()))?;
    (player > 1).then_some((console, player))
}

/// keybindings.json's text read for `consoles`; none is no file. A text that will not read gives the defaults and its words.
pub fn read_keys(text: Option<&str>, consoles: &[&str]) -> (KeyBindings, Option<String>) {
    let mut bindings = seeded(consoles, key_defaults);
    let Some(text) = text else { return (bindings, None) };
    match by_console::<KeyMap>(text) {
        Err(words) => return (bindings, Some(words)),
        Ok(ByConsole::Nothing) => {}
        Ok(ByConsole::Flat(shared)) if !shared.is_empty() => {
            for map in bindings.values_mut() {
                *map = shared.clone();
                add_new_defaults(map, key_defaults(), FIRST_ADDED);
            }
        }
        Ok(ByConsole::Flat(_)) => {}
        Ok(ByConsole::Each(each)) => {
            for (console, map) in each {
                let Some(mut map) = map.filter(|m| !m.is_empty()) else { continue };
                add_new_defaults(&mut map, key_defaults(), FIRST_ADDED);
                *entry(&mut bindings, &console) = map;
            }
        }
    }
    (bindings, None)
}

/// gamepadbindings.json's text read for `consoles`, as `read_keys`; a player's key keeps its own map.
pub fn read_gamepad(text: Option<&str>, consoles: &[&str]) -> (GamepadBindings, Option<String>) {
    let mut bindings = GamepadBindings { consoles: seeded(consoles, pad_defaults), players: IndexMap::new() };
    let Some(text) = text else { return (bindings, None) };
    match by_console::<PadMap>(text) {
        Err(words) => return (bindings, Some(words)),
        Ok(ByConsole::Nothing) => {}
        Ok(ByConsole::Flat(shared)) if !shared.is_empty() => {
            for map in bindings.consoles.values_mut() {
                *map = shared.clone();
                add_new_defaults(map, pad_defaults(), FIRST_ADDED);
            }
        }
        Ok(ByConsole::Flat(_)) => {}
        Ok(ByConsole::Each(each)) => {
            for (key, map) in each {
                let Some(mut map) = map.filter(|m| !m.is_empty()) else { continue };
                add_new_defaults(&mut map, pad_defaults(), FIRST_ADDED);
                match player_key(&key) {
                    Some((console, player)) => {
                        entry(&mut bindings.players, console).insert(Player(player), map);
                    }
                    None => *entry(&mut bindings.consoles, &key) = map,
                }
            }
        }
    }
    (bindings, None)
}

/// A profile's maps by player as a file may hold them, any of them null.
type ProfileMaps = IndexMap<Player, Option<PadMap>>;

/// A console's entry in gamepadprofiles.json as a file may hold it.
#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase", default)]
struct ProfilesEntry {
    default: Option<String>,
    profiles: Option<IndexMap<String, Option<ProfileMaps>>>,
    games: Option<IndexMap<String, Option<String>>>,
}

fn is_default(name: &str) -> bool {
    same_name(name, DEFAULT_PROFILE)
}

/// A profile's own spelling, by a name in any case.
fn named<'a>(all: &'a ConsoleProfiles, name: &str) -> Option<&'a str> {
    all.profiles.keys().find(|k| same_name(k, name)).map(String::as_str)
}

/// gamepadprofiles.json's text read by the profiles' rules: a name trimmed, never Default's and never taken twice, with
/// player 1's map; players 1 to 8; a chosen profile and a game's that exist. A text that will not read gives no profiles.
pub fn read_profiles(text: Option<&str>) -> (GamepadProfiles, Option<String>) {
    let mut profiles = GamepadProfiles::new();
    let Some(text) = text else { return (profiles, None) };
    let file: Option<IndexMap<String, Option<ProfilesEntry>>> = match config_json::read(text) {
        Ok(file) => file,
        Err(words) => return (profiles, Some(words)),
    };
    for (console, file) in file.unwrap_or_default() {
        let Some(file) = file else { continue };
        let all = entry(&mut profiles, &console);
        for (name, maps) in file.profiles.unwrap_or_default() {
            let trimmed = name.trim();
            let Some(maps) = maps else { continue };
            if trimmed.is_empty() || is_default(trimmed) || named(all, trimmed).is_some() || !maps.get(&Player(1)).is_some_and(Option::is_some) {
                continue;
            }
            let mut profile = IndexMap::new();
            for (player, map) in maps {
                let Some(mut map) = map.filter(|_| (1..=MAX_PLAYERS as i32).contains(&player.0)) else { continue };
                add_new_defaults(&mut map, pad_defaults(), FIRST_ADDED);
                profile.insert(player, map);
            }
            all.profiles.insert(trimmed.to_string(), profile);
        }
        all.default = file.default.and_then(|chosen| named(all, &chosen).map(str::to_string));
        for (game, name) in file.games.unwrap_or_default() {
            let Some(name) = name.filter(|_| !game.is_empty()) else { continue };
            let kept = if is_default(&name) { Some(DEFAULT_PROFILE.to_string()) } else { named(all, &name).map(str::to_string) };
            if let Some(kept) = kept {
                all.games.insert(game, kept);
            }
        }
    }
    (profiles, None)
}

/// hotkeybindings.json's text read; an action added after the file was written takes its default key where that is free.
pub fn read_hotkeys(text: Option<&str>) -> (HotkeyMap, Option<String>) {
    let Some(text) = text else { return (hotkey_defaults(), None) };
    match config_json::read::<Option<HotkeyMap>>(text) {
        Err(words) => (hotkey_defaults(), Some(words)),
        Ok(Some(mut map)) if !map.is_empty() => {
            add_new_defaults(&mut map, hotkey_defaults(), 0);
            (map, None)
        }
        Ok(_) => (hotkey_defaults(), None),
    }
}

/// gamepadbindings.json's file: each console's map, then each player's under its key.
fn gamepad_file(bindings: &GamepadBindings) -> IndexMap<String, &PadMap> {
    let mut file: IndexMap<String, &PadMap> = bindings.consoles.iter().map(|(console, map)| (console.clone(), map)).collect();
    for (console, players) in &bindings.players {
        for (player, map) in players {
            file.insert(format!("{console} Player {}", player.0), map);
        }
    }
    file
}

/// The binding files, by their `EMUSEN_ENDYMION_BINDINGS_*` number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum File {
    Keys,
    Gamepad,
    Profiles,
    Hotkeys,
}

impl File {
    pub const ALL: [File; 4] = [File::Keys, File::Gamepad, File::Profiles, File::Hotkeys];

    pub fn from_u32(which: u32) -> Option<File> {
        File::ALL.get(which as usize).copied()
    }

    /// The file's name in the config directory.
    pub fn name(self) -> &'static str {
        match self {
            File::Keys => "keybindings.json",
            File::Gamepad => "gamepadbindings.json",
            File::Profiles => "gamepadprofiles.json",
            File::Hotkeys => "hotkeybindings.json",
        }
    }

    /// A file's text read as its model, as compact JSON, for `consoles`; none is no file. A text that will not read gives
    /// what no file gives, and its words.
    pub fn read(self, text: Option<&str>, consoles: &[&str]) -> (String, Option<String>) {
        fn as_json<T: Serialize>((model, words): (T, Option<String>)) -> (String, Option<String>) {
            (compact(&model), words)
        }
        match self {
            File::Keys => as_json(read_keys(text, consoles)),
            File::Gamepad => as_json(read_gamepad(text, consoles)),
            File::Profiles => as_json(read_profiles(text)),
            File::Hotkeys => as_json(read_hotkeys(text)),
        }
    }

    /// A model, as any JSON its type binds by a hand-edited file's rules, as the file's text: a console with no profile
    /// and no game's choice is left out of gamepadprofiles.json.
    pub fn text(self, document: &str) -> Result<String, String> {
        match self {
            File::Keys => config_json::write(&config_json::read::<KeyBindings>(document)?),
            File::Gamepad => config_json::write(&gamepad_file(&config_json::read::<GamepadBindings>(document)?)),
            File::Profiles => {
                let mut profiles = config_json::read::<GamepadProfiles>(document)?;
                profiles.retain(|_, all| !all.profiles.is_empty() || !all.games.is_empty());
                config_json::write(&profiles)
            }
            File::Hotkeys => config_json::write(&config_json::read::<HotkeyMap>(document)?),
        }
    }

    /// One map at its defaults, as compact JSON: a console's keys, a console's gamepad buttons, or the hotkeys; profiles have none.
    pub fn default_map(self) -> Option<String> {
        match self {
            File::Keys => Some(compact(&key_defaults())),
            File::Gamepad => Some(compact(&pad_defaults())),
            File::Profiles => None,
            File::Hotkeys => Some(compact(&hotkey_defaults())),
        }
    }
}

/// A model as compact JSON, as it crosses the platform library.
fn compact<T: Serialize>(model: &T) -> String {
    serde_json::to_string(model).expect("a binding model is JSON")
}

/// A file's model as a program starts with it, from the config directory of `tree`, with the diagnostic for a file that
/// would not read: `<path>: <words> Falling back to defaults.`, as every config file says it.
pub fn load(tree: &Tree, file: File, consoles: &[&str]) -> (String, Option<String>) {
    let (text, unread) = match emusen_galaxia::config::read_text(tree, None, file.name()) {
        Ok(text) => (text, None),
        Err(words) => (None, Some(words)),
    };
    let (model, words) = file.read(text.as_deref(), consoles);
    let said = unread.or(words).map(|words| emusen_galaxia::config::load_diagnostic(tree, None, file.name(), &words));
    (model, said)
}

/// Why a model was not saved.
#[derive(Debug, PartialEq)]
pub enum Unsaved {
    /// The document is not a model of the file, with the words.
    Document(String),
    /// The file could not be written, with the words.
    Write(String),
}

/// A model saved as its file in the config directory of `tree`, through a temp file and a rename.
pub fn save(tree: &Tree, file: File, document: &str) -> Result<(), Unsaved> {
    let text = file.text(document).map_err(Unsaved::Document)?;
    if emusen_galaxia::config::write_text(tree, None, file.name(), Some(&text)) {
        Ok(())
    } else {
        Err(Unsaved::Write(format!("{} could not be written.", tree.config_path(None, file.name()).display())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_default_is_a_word_of_its_vocabulary() {
        assert_eq!(key_defaults().len(), 24);
        assert_eq!(pad_defaults().len(), 14);
        assert_eq!(hotkey_defaults().len(), 10);
        assert_eq!(key_defaults()[&Control::named("Start").unwrap()].name(), "Return");
    }

    #[test]
    fn a_players_key_is_a_console_and_a_number_past_one() {
        assert_eq!(player_key("SNES Player 2"), Some(("SNES", 2)));
        assert_eq!(player_key("Neo Geo Player 4"), Some(("Neo Geo", 4)));
        for key in ["SNES Player 1", "SNES Player x", "SNES Player -2", "SNES Player +2", " Player 2", "SNES"] {
            assert_eq!(player_key(key), None, "{key}");
        }
    }

    #[test]
    fn a_word_reads_from_a_name_in_any_case_or_a_number_and_writes_its_first_name() {
        assert_eq!(Key::parse("enter").unwrap().name(), "Return");
        assert_eq!(Key::parse(" 24 ").unwrap().name(), "Up");
        assert_eq!(GamepadButton::parse("dpadupp").unwrap_err(), "'dpadupp' is not a valid GamepadButton. Did you mean 'DPadUp'?");
        assert_eq!(Action::parse("Pause").unwrap_err(), "'Pause' is not a valid HotkeyAction.");
        assert!(Key::parse("999").is_err());
        assert_eq!(serde_json::to_string(&Button::named("select").unwrap()).unwrap(), "\"Select\"");
    }
}
