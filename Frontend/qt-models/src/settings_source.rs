//! The Settings screen's source over the platform library (EmuSen_Qt.md §7.7): Galaxia's rows and models, Serenity's
//! catalogue and Endymion's sound devices, each through `emusen-platform-client`.

use crate::clients::app_settings;
use crate::settings::{Model, SettingsSource};
use emusen_platform_client::galaxia::consts::{FILE_SETTINGS, MODEL_APP_SETTINGS, MODEL_GRAPHICS_CONFIG};
use emusen_platform_client::{serenity, Platform};
use serde_json::Value;

/// GraphicsConfig's file, as `emusen_platform.h` names it beside EMUSEN_GALAXIA_MODEL_GRAPHICS_CONFIG, which has no file number.
const GRAPHICS_FILE: &str = "graphics.json";

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

    fn settings_name(&self) -> Result<String, String> {
        let file = self.platform.file(FILE_SETTINGS).map_err(words)?;
        file.file_name().and_then(|n| n.to_str()).map(str::to_owned).ok_or_else(|| format!("{} names no settings file", file.display()))
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
            Model::Graphics => (MODEL_GRAPHICS_CONFIG, self.platform.model_load(MODEL_GRAPHICS_CONFIG, None, GRAPHICS_FILE).map_err(words)?.unwrap_or(Value::Null)),
        };
        match read {
            Value::Null => self.platform.model_new(number).map_err(words),
            read => Ok(read),
        }
    }

    fn save(&self, model: Model, document: &Value) -> Result<(), String> {
        match model {
            Model::App => self.platform.model_save(MODEL_APP_SETTINGS, None, &self.settings_name()?, document).map_err(words),
            Model::Graphics => self.platform.model_save(MODEL_GRAPHICS_CONFIG, None, GRAPHICS_FILE, document).map_err(words),
        }
    }

    fn catalogue(&self) -> Result<Value, String> {
        serenity::catalogue(&self.platform).map_err(words)
    }

    fn outputs(&self) -> Vec<String> {
        self.platform.audio_devices().unwrap_or_default()
    }
}
