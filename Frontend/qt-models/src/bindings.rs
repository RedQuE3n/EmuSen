//! Endymion's four binding models as Settings ▸ Controllers reads and changes them (EmuSen_Qt.md §7.7.H,
//! EmuSen_Input.md §5.2, §10.5, §10.6): each crosses as its file's JSON (emusen_platform.h's shapes), is changed here in
//! that shape and saved through Endymion, whose reading rules keep a profile's name and a map's members; the files are
//! read again after each save, so what is shown is what Endymion kept. A button's gamepad button is written by its SDL
//! number, which Endymion reads as it reads a name. No Qt here.

use serde_json::{json, Map, Value};

/// The profile every console has: the maps of gamepadbindings.json (Endymion's `DEFAULT_PROFILE`, emusen_platform.h).
pub const DEFAULT: &str = "Default";

/// The four models and the three maps' defaults, as `bindings_load` and `bindings_defaults` give them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bindings {
    pub keys: Value,
    pub gamepad: Value,
    pub profiles: Value,
    pub hotkeys: Value,
    pub key_defaults: Value,
    pub pad_defaults: Value,
    pub hotkey_defaults: Value,
}

/// Which map a player plays with, as Endymion's `bindings_choose` answers: the profile in its own spelling and the
/// player whose map it is in that profile.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Choice {
    pub profile: String,
    pub map_player: u32,
}

/// Which of the four a change touched, for saving.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum File {
    Keys,
    Gamepad,
    Profiles,
    Hotkeys,
}

fn same(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// A member of an object by a name in any case.
fn get<'a>(v: &'a Value, name: &str) -> Option<&'a Value> {
    v.as_object()?.iter().find(|(k, _)| same(k, name)).map(|(_, v)| v)
}

/// The member's own spelling of a name in any case, or the name.
fn spelling(v: &Value, name: &str) -> String {
    v.as_object().and_then(|o| o.keys().find(|k| same(k, name)).cloned()).unwrap_or_else(|| name.to_owned())
}

fn object(v: &mut Value) -> &mut Map<String, Value> {
    if !v.is_object() {
        *v = json!({});
    }
    v.as_object_mut().expect("an object")
}

/// A member made an object if it is not one, by a name in any case.
fn entry<'a>(v: &'a mut Value, name: &str) -> &'a mut Value {
    let key = spelling(v, name);
    let o = object(v);
    let e = o.entry(key).or_insert_with(|| json!({}));
    if !e.is_object() {
        *e = json!({});
    }
    e
}

fn text(v: Option<&Value>) -> Option<String> {
    match v? {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

impl Bindings {
    /// A console's named profiles, Default first.
    pub fn profiles(&self, console: &str) -> Vec<String> {
        let mut out = vec![DEFAULT.to_owned()];
        if let Some(named) = get(&self.profiles, console).and_then(|c| get(c, "Profiles")).and_then(Value::as_object) {
            out.extend(named.keys().filter(|k| !same(k, DEFAULT)).cloned());
        }
        out
    }

    /// The profile a console's games start with.
    pub fn console_default(&self, console: &str) -> String {
        let chosen = get(&self.profiles, console).and_then(|c| get(c, "Default")).and_then(Value::as_str);
        chosen.and_then(|c| self.profiles(console).into_iter().find(|p| same(p, c))).unwrap_or_else(|| DEFAULT.to_owned())
    }

    /// A game's own profile chosen, or the console's again for none.
    pub fn set_game_profile(&mut self, console: &str, game: &str, profile: Option<&str>) -> File {
        let c = entry(&mut self.profiles, console);
        let games = entry(c, "Games");
        let key = spelling(games, game_key(game));
        match profile {
            Some(p) => {
                object(games).insert(key, json!(p));
            }
            None => {
                object(games).remove(&key);
            }
        }
        File::Profiles
    }

    /// Starts every game of the console with this profile.
    pub fn choose_default(&mut self, console: &str, profile: &str) -> File {
        let c = entry(&mut self.profiles, console);
        object(c).insert("Default".into(), if same(profile, DEFAULT) { Value::Null } else { json!(profile) });
        File::Profiles
    }

    /// A new profile, a copy of `from`'s maps, under a name Endymion keeps if it is a new one.
    pub fn create(&mut self, console: &str, name: &str, from: &str) -> File {
        let name = name.trim().to_owned();
        let maps = self.maps(console, from);
        let c = entry(&mut self.profiles, console);
        object(entry(c, "Profiles")).insert(name, maps);
        File::Profiles
    }

    /// A profile renamed; the console's choice and the games that chose it follow it.
    pub fn rename(&mut self, console: &str, from: &str, to: &str) -> File {
        let to = to.trim().to_owned();
        let c = entry(&mut self.profiles, console);
        let profiles = entry(c, "Profiles");
        let key = spelling(profiles, from);
        if let Some(maps) = object(profiles).remove(&key) {
            object(profiles).insert(to.clone(), maps);
        }
        if get(c, "Default").and_then(Value::as_str).is_some_and(|d| same(d, from)) {
            object(c).insert("Default".into(), json!(to));
        }
        if let Some(games) = c.get_mut("Games").and_then(Value::as_object_mut) {
            for chosen in games.values_mut() {
                if chosen.as_str().is_some_and(|g| same(g, from)) {
                    *chosen = json!(to);
                }
            }
        }
        File::Profiles
    }

    /// A profile deleted: its games start with the console's again.
    pub fn delete(&mut self, console: &str, profile: &str) -> File {
        let c = entry(&mut self.profiles, console);
        let profiles = entry(c, "Profiles");
        let key = spelling(profiles, profile);
        object(profiles).remove(&key);
        if get(c, "Default").and_then(Value::as_str).is_some_and(|d| same(d, profile)) {
            object(c).insert("Default".into(), Value::Null);
        }
        if let Some(games) = c.get_mut("Games").and_then(Value::as_object_mut) {
            games.retain(|_, chosen| !chosen.as_str().is_some_and(|g| same(g, profile)));
        }
        File::Profiles
    }

    /// A profile's maps by player, as a new profile copies them: Default's are the console's and its players'.
    fn maps(&self, console: &str, profile: &str) -> Value {
        if same(profile, DEFAULT) {
            let mut out = json!({ "1": get(&self.gamepad, "Consoles").and_then(|c| get(c, console)).cloned().unwrap_or_else(|| self.pad_defaults.clone()) });
            if let Some(players) = get(&self.gamepad, "Players").and_then(|p| get(p, console)).and_then(Value::as_object) {
                for (player, map) in players {
                    out[player] = map.clone();
                }
            }
            return out;
        }
        get(&self.profiles, console).and_then(|c| get(c, "Profiles")).and_then(|p| get(p, profile)).cloned().unwrap_or_else(|| json!({}))
    }

    /// The map a choice of Endymion's names (`bindings_choose`: the profile and the player whose map it is there).
    pub fn pad_map(&self, console: &str, choice: &Choice) -> Value {
        if same(&choice.profile, DEFAULT) {
            if choice.map_player > 1 {
                if let Some(own) = get(&self.gamepad, "Players").and_then(|p| get(p, console)).and_then(|c| get(c, &choice.map_player.to_string())) {
                    return own.clone();
                }
            }
            return get(&self.gamepad, "Consoles").and_then(|c| get(c, console)).cloned().unwrap_or_else(|| self.pad_defaults.clone());
        }
        let maps = self.maps(console, &choice.profile);
        get(&maps, &choice.map_player.to_string()).cloned().unwrap_or_else(|| self.pad_defaults.clone())
    }

    /// The map a player's change is written to, made the player's own from the one they play with first.
    fn own(&mut self, console: &str, player: u32, choice: &Choice) -> (&mut Value, File) {
        let start = self.pad_map(console, choice);
        let profile = choice.profile.as_str();
        if same(profile, DEFAULT) {
            let (place, name) = if player > 1 { (entry(entry(&mut self.gamepad, "Players"), console), player.to_string()) } else { (entry(&mut self.gamepad, "Consoles"), console.to_owned()) };
            let key = spelling(place, &name);
            let map = object(place).entry(key).or_insert(start);
            return (map, File::Gamepad);
        }
        let c = entry(&mut self.profiles, console);
        let p = entry(entry(c, "Profiles"), profile);
        let key = spelling(p, &player.to_string());
        let map = object(p).entry(key).or_insert(start);
        (map, File::Profiles)
    }

    /// The gamepad button a pad button is bound to for a player in a profile, in the file's word; none for none.
    pub fn bound(&self, console: &str, choice: &Choice, button: &str) -> Option<String> {
        text(get(&self.pad_map(console, choice), button)).filter(|b| !same(b, "Invalid"))
    }

    /// A button bound to the gamepad button of SDL's number `pad`.
    pub fn rebind(&mut self, console: &str, player: u32, choice: &Choice, button: &str, pad: i32) -> File {
        let (map, file) = self.own(console, player, choice);
        let key = spelling(map, button);
        object(map).insert(key, json!(pad));
        file
    }

    /// A button bound to nothing, as a trigger bound alone leaves the button beside it (EmuSen_Input.md §7.3).
    pub fn unbind(&mut self, console: &str, player: u32, choice: &Choice, button: &str) -> File {
        let (map, file) = self.own(console, player, choice);
        let key = spelling(map, button);
        object(map).remove(&key);
        file
    }

    /// A button back to its default.
    pub fn reset_button(&mut self, console: &str, player: u32, choice: &Choice, button: &str) -> File {
        let default = get(&self.pad_defaults, button).cloned();
        let (map, file) = self.own(console, player, choice);
        let key = spelling(map, button);
        match default {
            Some(d) => object(map).insert(key, d),
            None => object(map).remove(&key),
        };
        file
    }

    /// The key a control is bound to on a console's keyboard, in the file's word.
    pub fn key(&self, console: &str, control: &str) -> Option<String> {
        text(get(&self.keys, console).and_then(|m| get(m, control)))
    }

    /// A control bound to a key, the key taken from a hotkey holding it, since the hotkeys are every console's.
    pub fn rebind_key(&mut self, console: &str, control: &str, key: &str) -> Vec<File> {
        let mut files = vec![File::Keys];
        if let Some(action) = self.hotkey_of(key) {
            let name = spelling(&self.hotkeys, &action);
            object(&mut self.hotkeys).remove(&name);
            files.push(File::Hotkeys);
        }
        let map = entry(&mut self.keys, console);
        let name = spelling(map, control);
        object(map).insert(name, json!(key));
        files
    }

    pub fn reset_key(&mut self, console: &str, control: &str) -> File {
        let default = get(&self.key_defaults, control).cloned();
        let map = entry(&mut self.keys, console);
        let name = spelling(map, control);
        match default {
            Some(d) => object(map).insert(name, d),
            None => object(map).remove(&name),
        };
        File::Keys
    }

    pub fn clear_key(&mut self, console: &str, control: &str) -> File {
        let map = entry(&mut self.keys, console);
        let name = spelling(map, control);
        object(map).remove(&name);
        File::Keys
    }

    /// The emulator's actions, in Endymion's order.
    pub fn actions(&self) -> Vec<String> {
        self.hotkey_defaults.as_object().map(|o| o.keys().cloned().collect()).unwrap_or_default()
    }

    pub fn hotkey(&self, action: &str) -> Option<String> {
        text(get(&self.hotkeys, action))
    }

    /// The action a key fires, if any.
    pub fn hotkey_of(&self, key: &str) -> Option<String> {
        self.hotkeys.as_object()?.iter().find(|(_, k)| text(Some(k)).is_some_and(|k| same(&k, key))).map(|(a, _)| a.clone())
    }

    /// A hotkey bound to a key, taken from every console's controls first.
    pub fn rebind_hotkey(&mut self, action: &str, key: &str) -> Vec<File> {
        let mut files = vec![File::Hotkeys];
        if let Some(consoles) = self.keys.as_object_mut() {
            for map in consoles.values_mut().filter_map(Value::as_object_mut) {
                let before = map.len();
                map.retain(|_, k| !text(Some(k)).is_some_and(|k| same(&k, key)));
                if map.len() != before && !files.contains(&File::Keys) {
                    files.push(File::Keys);
                }
            }
        }
        let name = spelling(&self.hotkeys, action);
        object(&mut self.hotkeys).insert(name, json!(key));
        files
    }

    pub fn clear_hotkey(&mut self, action: &str) -> File {
        let name = spelling(&self.hotkeys, action);
        object(&mut self.hotkeys).remove(&name);
        File::Hotkeys
    }

    /// The consoles whose controls hold a key, of those whose controls are given.
    pub fn consoles_holding<'a>(&self, key: &str, consoles: &'a [(String, Vec<String>)]) -> Vec<&'a str> {
        consoles
            .iter()
            .filter(|(console, controls)| controls.iter().any(|c| self.key(console, c).is_some_and(|k| same(&k, key))))
            .map(|(c, _)| c.as_str())
            .collect()
    }

    /// A console's clashes among its controls' keys and the hotkeys: each key held twice, with what holds it, by the
    /// names given for its controls and actions.
    pub fn conflicts(&self, console: &str, controls: &[(String, String)], actions: &dyn Fn(&str) -> String) -> Vec<(String, Vec<String>)> {
        let mut owners: Vec<(String, Vec<String>)> = Vec::new();
        let mut add = |key: String, owner: String| match owners.iter_mut().find(|(k, _)| same(k, &key)) {
            Some((_, list)) => list.push(owner),
            None => owners.push((key, vec![owner])),
        };
        for (control, name) in controls {
            if let Some(k) = self.key(console, control) {
                add(k, name.clone());
            }
        }
        for action in self.actions() {
            if let Some(k) = self.hotkey(&action) {
                add(k, actions(&action));
            }
        }
        owners.into_iter().filter(|(_, o)| o.len() > 1).collect()
    }

    /// Every console's keys and gamepad buttons and the hotkeys back to their defaults; named profiles stay.
    pub fn reset_all(&mut self) -> Vec<File> {
        if let Some(consoles) = self.keys.as_object_mut() {
            for map in consoles.values_mut() {
                *map = self.key_defaults.clone();
            }
        }
        let gamepad = object(&mut self.gamepad);
        if let Some(consoles) = gamepad.get_mut("Consoles").and_then(Value::as_object_mut) {
            for map in consoles.values_mut() {
                *map = self.pad_defaults.clone();
            }
        }
        gamepad.insert("Players".into(), json!({}));
        self.hotkeys = self.hotkey_defaults.clone();
        vec![File::Keys, File::Gamepad, File::Hotkeys]
    }
}

/// The name a game is known by in a console's profiles: its file's name without the extension (Endymion's `game_key`).
pub fn game_key(path: &str) -> &str {
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    match name.rfind('.') {
        Some(dot) if dot > 0 => &name[..dot],
        _ => name,
    }
}

/// A gamepad button's file word in the words a pane says: `DPadUp` as "D-pad Up", `LeftShoulder` as "Left Shoulder".
pub fn pad_words(word: &str) -> String {
    let word = word.replace("DPad", "D-pad ");
    let mut out = String::new();
    let mut prev: Option<char> = None;
    for ch in word.chars() {
        if ch.is_ascii_uppercase() && prev.is_some_and(|p| p.is_ascii_lowercase()) {
            out.push(' ');
        }
        out.push(ch);
        prev = Some(ch);
    }
    out
}
