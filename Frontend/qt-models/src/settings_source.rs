//! The Settings screen's source over the platform library (EmuSen_Qt.md §7.7): Galaxia's rows and models, Serenity's
//! catalogue and Endymion's sound devices, each through `emusen-platform-client`.

use crate::bindings::{Bindings, Choice, File};
use crate::clients::app_settings;
use crate::settings::{Model, SettingsSource};
use crate::settings_controllers::{Chooser, ControlWord};
use emusen_platform_client::bindings::consts as files;
use emusen_platform_client::galaxia::consts::{DIR_FIRMWARE, DIR_MEDIA, DIR_SCREENSHOTS, DIR_SHADERS, DIR_THEMES, FILE_GAMES, FILE_GRAPHICS, FILE_SETTINGS, MODEL_APP_SETTINGS, MODEL_GRAPHICS_CONFIG};
use emusen_platform_client::scrape::ScraperConfig;
use emusen_platform_client::{serenity, Lent, Platform};
use serde_json::{json, Value};

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

    /// Galaxia's installed themes folder, for Settings -> Library -> Themes (#61).
    pub fn themes_store(&self) -> Option<std::path::PathBuf> {
        self.platform.directory(DIR_THEMES).ok().flatten()
    }

    /// Live ScreenScraper account, queue, run and quota snapshot from emusen.scrape (#62).
    pub fn scrape_snapshot(&self) -> crate::settings_scrape::ScrapeState {
        let mut out = crate::settings_scrape::ScrapeState::default();
        let Ok(service) = self.platform.scrape_service() else { return out };
        if let Ok(account) = service.get(&json!({ "what": "account" })) {
            out.signed_in = account["signed_in"].as_bool().unwrap_or(false);
            out.account_name = account["name"].as_str().unwrap_or_default().to_owned();
            out.verified = account["verified"].as_str().unwrap_or_default().to_owned();
        }
        if let Ok(queue) = service.get(&json!({ "what": "queue" })) {
            out.queued = queue.as_array().map_or(0, Vec::len);
        }
        if let Ok(quota) = service.get(&json!({ "what": "quota" })) {
            out.requests_today = quota["requests"].as_u64().unwrap_or(0);
            out.max_per_day = quota["max_per_day"].as_u64();
            out.threads = quota["threads"].as_u64().unwrap_or(0);
            out.gate = quota["gate"].as_str().unwrap_or_default().to_owned();
        }
        if let Ok(progress) = service.progress(0) {
            out.queued = out.queued.max(progress["interrupted"].as_u64().unwrap_or(0) as usize);
            out.run = crate::scrape_run::Run::of(&progress);
        }
        out
    }

    /// Signs into a ScreenScraper member account through emusen.scrape (#62).
    pub fn scrape_sign_in(&self, user: &str, password: &str) -> Result<Value, String> {
        let service = self.platform.scrape_service().map_err(words)?;
        let _ = service.change(&json!({ "op": "settings", "developer_files": true }));
        service.change(&json!({ "op": "sign_in", "user": user, "password": password })).map_err(words)
    }

    /// Signs out of the saved ScreenScraper member account through emusen.scrape (#62).
    pub fn scrape_sign_out(&self) -> Result<Value, String> {
        let service = self.platform.scrape_service().map_err(words)?;
        service.change(&json!({ "op": "sign_out" })).map_err(words)
    }

    /// Re-verifies the saved ScreenScraper member account through emusen.scrape (#62).
    pub fn scrape_check(&self) -> Result<Value, String> {
        let service = self.platform.scrape_service().map_err(words)?;
        let _ = service.change(&json!({ "op": "settings", "developer_files": true }));
        service.change(&json!({ "op": "check" })).map_err(words)
    }

    /// Resumes queued games from a stopped scrape run, opening the media store if idle (#62).
    pub fn scrape_resume(&self, settings: &Value) -> Result<i64, String> {
        let service = self.platform.scrape_service().map_err(words)?;
        let kinds = service.get(&json!({ "what": "kinds" })).map_err(words)?;
        let mut req = crate::scrape_run::request(&[], false, settings, &kinds);
        req["resume"] = json!(true);
        match service.start(&req) {
            Ok(n) => Ok(n),
            Err(_) => {
                let root = self.platform.directory(DIR_MEDIA).map_err(words)?.ok_or("the media folder is not known")?;
                let games_db = self.platform.file(FILE_GAMES).ok().map(|p| p.to_string_lossy().into_owned());
                let sqlite = Lent::open(crate::clients::SQLITE).map_err(words)?;
                let text = |key: &str| settings.get(key).and_then(Value::as_str).filter(|v| !v.is_empty());
                let config = ScraperConfig {
                    root: root.to_string_lossy().into_owned(),
                    games_db,
                    settings: json!({
                        "roms": text("RomDirectory"), "art": text("ArtworkDirectory"), "esde_media": text("EsdeMediaDirectory"),
                        "openemu": settings.get("OpenEmuFallback").and_then(Value::as_bool).unwrap_or(true),
                        "developer_files": true, "sources_alone": false,
                    }),
                    ..ScraperConfig::default()
                };
                self.platform.scraper_open(&config, Some(&sqlite)).map_err(words)?;
                self.platform.scrape_service().map_err(words)?.start(&req).map_err(words)
            }
        }
    }

    /// Cancels an active scrape run through emusen.scrape (#62).
    pub fn scrape_cancel(&self) -> Result<bool, String> {
        self.platform.scrape_service().map_err(words)?.cancel().map_err(words)
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

    fn shot(&self) -> Option<String> {
        crate::screenshots::newest(&self.platform.directory(DIR_SCREENSHOTS).ok().flatten()?)
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
