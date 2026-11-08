//! `consoles.json`, each console's core settings, the runtime's own file: read, and on its first load with no file
//! there made by copying `graphics.json`'s `Consoles` map whole, never over a file that exists (EmuSen_Backend.md
//! §14.12, SR12).

use crate::config::{self, Loaded};
use crate::models::{CONSOLES_FILE, ConsolesConfig, GRAPHICS_FILE, GraphicsConfig, Map, Text};
use crate::tree::Tree;

/// Each console's values by key, in the file's order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Consoles {
    pub consoles: Vec<(String, Vec<(String, String)>)>,
}

impl Consoles {
    /// A console's stored value, compared by name exactly as the C# dictionaries compare.
    pub fn value(&self, console: &str, key: &str) -> Option<&str> {
        self.consoles.iter().find(|(c, _)| c == console)?.1.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    /// `SetValue`: the value replaced where the key is held, else added after the console's others.
    pub fn set_value(&mut self, console: &str, key: &str, value: &str) {
        let at = match self.consoles.iter().position(|(c, _)| c == console) {
            Some(at) => at,
            None => {
                self.consoles.push((console.to_string(), Vec::new()));
                self.consoles.len() - 1
            }
        };
        let values = &mut self.consoles[at].1;
        match values.iter_mut().find(|(k, _)| k == key) {
            Some(slot) => slot.1 = value.to_string(),
            None => values.push((key.to_string(), value.to_string())),
        }
    }

    /// `Forget`: the key let go, and the console with its last value.
    pub fn forget(&mut self, console: &str, key: &str) {
        let Some(at) = self.consoles.iter().position(|(c, _)| c == console) else { return };
        self.consoles[at].1.retain(|(k, _)| k != key);
        if self.consoles[at].1.is_empty() {
            self.consoles.remove(at);
        }
    }

    /// A file's map as values by console: a null console holds none, and a null value is passed over.
    fn of(map: Map<Map<Text>>) -> Consoles {
        let consoles = map.unwrap_or_default().into_iter().map(|(console, values)| {
            let values = values.unwrap_or_default().into_iter().filter_map(|(key, value)| value.map(|value| (key, value))).collect();
            (console, values)
        });
        Consoles { consoles: consoles.collect() }
    }

    /// Written whole as `consoles.json`; false when it could not be.
    pub fn save(&self, tree: &Tree) -> bool {
        let consoles = self.consoles.iter().map(|(console, values)| (console.clone(), Some(values.iter().map(|(k, v)| (k.clone(), Some(v.clone()))).collect())));
        config::save_model(tree, None, CONSOLES_FILE, &ConsolesConfig { consoles: Some(consoles.collect()) }).is_ok()
    }
}

/// The file, or with none yet `graphics.json`'s map copied into a new one and saved where it holds anything; a file
/// that will not load reads as empty and is left as it is.
pub fn load(tree: &Tree) -> Consoles {
    match config::load_model::<ConsolesConfig>(tree, None, CONSOLES_FILE, false) {
        Ok(Loaded::Document(file)) => return Consoles::of(file.consoles),
        Ok(Loaded::Null) | Err(_) => return Consoles::default(),
        Ok(Loaded::Missing) => {}
    }
    if config::exists(tree, None, CONSOLES_FILE) || !config::exists(tree, None, GRAPHICS_FILE) {
        return Consoles::default();
    }
    let graphics = match config::load_model::<GraphicsConfig>(tree, None, GRAPHICS_FILE, false) {
        Ok(Loaded::Document(file)) => file,
        _ => GraphicsConfig::default(),
    };
    let made = Consoles::of(graphics.consoles);
    if !made.consoles.is_empty() {
        made.save(tree);
    }
    made
}
