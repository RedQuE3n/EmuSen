//! `consoles.json`, each console's core settings, the runtime's own file: read, and on its first load with no file
//! there made by copying `graphics.json`'s `Consoles` map whole, never over a file that exists, as the C#
//! `ConsolesConfig.Load` does (EmuSen_Backend.md §14.12, SR12).

use crate::config::{self, Loaded};
use crate::json::{Text, Value};
use crate::models::{CONSOLES_FILE, CONSOLES_MODEL, GRAPHICS_CONFIG, GRAPHICS_FILE, MODELS, GRAPHICS_CONFIG_ID};
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

    fn of(map: Option<&Value>) -> Consoles {
        let mut out = Consoles::default();
        if let Some(Value::Object(members)) = map {
            for (console, settings) in members {
                let values = match settings {
                    Value::Object(pairs) => pairs.iter().filter_map(|(k, v)| v.as_str().map(|v| (k.text.clone(), v.to_string()))).collect(),
                    _ => Vec::new(),
                };
                match out.consoles.iter_mut().find(|(c, _)| *c == console.text) {
                    Some(slot) => slot.1 = values,
                    None => out.consoles.push((console.text.clone(), values)),
                }
            }
        }
        out
    }

    /// The document as the C# serialiser shapes it, for the model's formatter.
    fn canonical(&self) -> String {
        let mut members = Vec::new();
        for (console, values) in &self.consoles {
            members.push((Text::new(console.clone()), Value::Object(values.iter().map(|(k, v)| (Text::new(k.clone()), Value::string(v.clone()))).collect())));
        }
        let mut out = String::new();
        crate::json::write_compact(&mut out, &Value::Object(vec![(Text::new("Consoles"), Value::Object(members))]));
        out
    }

    /// Written whole as the C# `ConsolesConfig.Save` writes it; false when it could not be.
    pub fn save(&self, tree: &Tree) -> bool {
        config::save(tree, &CONSOLES_MODEL, None, CONSOLES_FILE, Some(&self.canonical())).is_ok()
    }
}

/// The file, or with none yet `graphics.json`'s map copied into a new one and saved where it holds anything; a file
/// that will not load reads as empty and is left as it is.
pub fn load(tree: &Tree) -> Consoles {
    match config::load(tree, &CONSOLES_MODEL, None, CONSOLES_FILE, false) {
        Ok(Loaded::Document(doc)) => return Consoles::of(doc.get("Consoles")),
        Ok(Loaded::Null) | Err(_) => return Consoles::default(),
        Ok(Loaded::Missing) => {}
    }
    if config::exists(tree, None, CONSOLES_FILE) || !config::exists(tree, None, GRAPHICS_FILE) {
        return Consoles::default();
    }
    let graphics = match config::load(tree, &MODELS[GRAPHICS_CONFIG_ID as usize], None, GRAPHICS_FILE, false) {
        Ok(Loaded::Document(doc)) => doc,
        _ => GRAPHICS_CONFIG.new_instance(),
    };
    let made = Consoles::of(graphics.get("Consoles"));
    if !made.consoles.is_empty() {
        made.save(tree);
    }
    made
}
