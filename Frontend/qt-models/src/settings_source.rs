//! The Settings screen's source over the platform library (EmuSen_Qt.md §7.7): Galaxia's rows and models, Serenity's
//! catalogue and Endymion's sound devices, each through `emusen-platform-client`.

use crate::bindings::{Bindings, Choice, File};
use crate::clients::app_settings;
use crate::settings::{Model, SettingsSource};
use crate::settings_controllers::{Chooser, ControlWord};
use emusen_platform_client::bindings::consts as files;
use emusen_platform_client::galaxia::consts::{DIR_FIRMWARE, DIR_SHADERS, FILE_GRAPHICS, FILE_SETTINGS, MODEL_APP_SETTINGS, MODEL_GRAPHICS_CONFIG};
use emusen_platform_client::{serenity, Platform};
use serde_json::Value;

pub struct ClientSettings {
    platform: std::sync::Arc<Platform>,
}

fn words(e: impl std::fmt::Display) -> String {
    e.to_string()
}

impl ClientSettings {
    pub fn open(platform_library: &str) -> Result<ClientSettings, String> {
        Ok(ClientSettings { platform: Platform::load(platform_library).map_err(words)? })
    }

    /// Over a platform library already loaded.
    pub fn on(platform: std::sync::Arc<Platform>) -> ClientSettings {
        ClientSettings { platform }
    }

    /// A model's file by Galaxia's number for it, as the name a model's load and save take: both sit in the config
    /// directory itself.
    fn name(&self, which: u32) -> Result<String, String> {
        let file = self.platform.file(which).map_err(words)?;
        file.file_name().and_then(|n| n.to_str()).map(str::to_owned).ok_or_else(|| format!("{} names no file", file.display()))
    }
}

impl ClientSettings {
    /// Endymion's four binding models for the consoles named, and the three maps' defaults.
    pub fn bindings(&self, consoles: &[&str]) -> Result<Bindings, String> {
        let p = &self.platform;
        Ok(Bindings {
            keys: p.bindings_load(files::KEYS, consoles).map_err(words)?,
            gamepad: p.bindings_load(files::GAMEPAD, consoles).map_err(words)?,
            profiles: p.bindings_load(files::PROFILES, consoles).map_err(words)?,
            hotkeys: p.bindings_load(files::HOTKEYS, consoles).map_err(words)?,
            key_defaults: p.bindings_defaults(files::KEYS).map_err(words)?,
            pad_defaults: p.bindings_defaults(files::GAMEPAD).map_err(words)?,
            hotkey_defaults: p.bindings_defaults(files::HOTKEYS).map_err(words)?,
            default_profile: default_profile(p)?,
        })
    }

    /// The binding files a change touched, saved through Endymion.
    pub fn save_bindings(&self, bindings: &Bindings, touched: &[File]) -> Result<(), String> {
        for file in touched {
            let which = crate::settings::SettingsView::bindings_files(&[*file])[0];
            self.platform.bindings_save(which, crate::settings::SettingsView::bindings_model(bindings, *file)).map_err(words)?;
        }
        Ok(())
    }

    /// The chooser over this platform library, with Endymion's controls' words; their failure fails the read.
    pub fn chooser(&self) -> Result<std::sync::Arc<dyn Chooser>, String> {
        let controls = ControlWord::read(&self.platform.bindings_controls().map_err(words)?);
        Ok(std::sync::Arc::new(Endymion(self.platform.clone(), controls)))
    }
}

/// The profile every console has, as Endymion names it: the one `bindings_choose` answers where no profile exists.
fn default_profile(p: &Platform) -> Result<String, String> {
    let none = serde_json::json!({});
    let answer = p.bindings_choose(Some(&none), Some(&none), "", 1, None, None).map_err(words)?;
    answer["profile"].as_str().map(str::to_owned).ok_or_else(|| format!("bindings_choose named no profile: {answer}"))
}

/// Endymion's `bindings_choose` over a platform library.
struct Endymion(std::sync::Arc<Platform>, Vec<ControlWord>);

impl Chooser for Endymion {
    fn choose(&self, gamepad: &Value, profiles: &Value, console: &str, player: u32, profile: Option<&str>, game: Option<&str>) -> Result<Choice, String> {
        let answer = self.0.bindings_choose(Some(gamepad), Some(profiles), console, player as i32, game, profile).map_err(words)?;
        match (answer["profile"].as_str(), answer["mapPlayer"].as_u64()) {
            (Some(profile), Some(map_player)) => Ok(Choice { profile: profile.to_owned(), map_player: map_player as u32 }),
            _ => Err(format!("bindings_choose named no profile and player: {answer}")),
        }
    }

    fn bound(&self, console: &str, player: u32) -> Result<[i32; 16], String> {
        self.0.bindings_bound(console, player as i32, None).map_err(words)
    }

    fn game_key(&self, path: &str) -> Result<String, String> {
        match self.0.bindings_game_key(path).map_err(words)? {
            key if key.is_empty() => Err(format!("Endymion named {path} by no name")),
            key => Ok(key),
        }
    }

    fn controls(&self) -> &[ControlWord] {
        &self.1
    }
}

impl SettingsSource for ClientSettings {
    fn rows(&self) -> Result<Value, String> {
        self.platform.settings_rows().map_err(words)
    }

    fn load(&self, model: Model) -> Result<Value, String> {
        // A model with no file yet is a new one, so a save writes every key at its initial value beside the one changed.
        let (number, read) = match model {
            Model::App => (MODEL_APP_SETTINGS, app_settings(&self.platform)?),
            Model::Graphics => (MODEL_GRAPHICS_CONFIG, self.platform.model_load(MODEL_GRAPHICS_CONFIG, None, &self.name(FILE_GRAPHICS)?).map_err(words)?.unwrap_or(Value::Null)),
        };
        match read {
            Value::Null => self.platform.model_new(number).map_err(words),
            read => Ok(read),
        }
    }

    fn save(&self, model: Model, document: &Value) -> Result<(), String> {
        match model {
            Model::App => self.platform.model_save(MODEL_APP_SETTINGS, None, &self.name(FILE_SETTINGS)?, document).map_err(words),
            Model::Graphics => self.platform.model_save(MODEL_GRAPHICS_CONFIG, None, &self.name(FILE_GRAPHICS)?, document).map_err(words),
        }
    }

    fn catalogue(&self) -> Result<Value, String> {
        serenity::catalogue(&self.platform).map_err(words)
    }

    fn outputs(&self) -> Vec<String> {
        self.platform.audio_devices().unwrap_or_default()
    }

    fn folders(&self) -> std::collections::BTreeMap<String, String> {
        let firmware = self.platform.directory(DIR_FIRMWARE).ok().flatten();
        firmware.into_iter().map(|d| ("firmware".to_owned(), d.display().to_string())).collect()
    }

    fn pack(&self) -> Vec<String> {
        let Some(root) = self.platform.directory(DIR_SHADERS).ok().flatten() else { return Vec::new() };
        let mut out = Vec::new();
        presets_under(&root, &root, &mut out);
        out.sort_by_key(|p| p.to_lowercase());
        out
    }

    fn preset_parameters(&self, path: &str) -> Result<Vec<Value>, String> {
        let root = self.platform.directory(DIR_SHADERS).map_err(words)?.ok_or("the shader pack's folder is not known")?;
        let preset = serenity::preset_load(&self.platform, &root.join(path).to_string_lossy()).map_err(words)?;
        failed(&preset)?;
        let mut sources = Vec::new();
        for pass in preset["passes"].as_array().into_iter().flatten() {
            let source = serenity::source_load(&self.platform, &text(&pass["shader"])).map_err(words)?;
            failed(&source)?;
            sources.push(source["parameters"].clone());
        }
        let merged = serenity::parameters_merge(&self.platform, &serde_json::json!({ "sources": sources, "overrides": preset["parameters"] })).map_err(words)?;
        Ok(merged.as_array().cloned().unwrap_or_default())
    }
}

fn text(value: &Value) -> String {
    value.as_str().unwrap_or_default().to_owned()
}

/// A reader's document of failure as its words.
fn failed(document: &Value) -> Result<(), String> {
    match document.get("fail") {
        Some(_) => Err(format!("{}", document["message"].as_str().unwrap_or("the preset could not be read"))),
        None => Ok(()),
    }
}

/// Every `.slangp` file under `folder`, by its path from `root` with `/` between its folders; a link is not followed.
fn presets_under(root: &std::path::Path, folder: &std::path::Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(folder) else { return };
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else { continue };
        let path = entry.path();
        if kind.is_dir() {
            presets_under(root, &path, out);
        } else if kind.is_file() && path.extension().is_some_and(|e| e.eq_ignore_ascii_case("slangp")) {
            if let Ok(relative) = path.strip_prefix(root) {
                out.push(relative.components().map(|c| c.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/"));
            }
        }
    }
}
