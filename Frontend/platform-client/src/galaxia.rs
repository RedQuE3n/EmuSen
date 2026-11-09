//! Galaxia through the platform library: where the tree's directories and Galaxia's files are, a config file's path,
//! the config models (the player's settings) as the documents Galaxia binds, the saved cheat lists' names, and the front end's own settings rows. A model crosses as JSON and stays
//! Galaxia's: its fields are not restated here, so a model has one owner.

use crate::error::Result;
use crate::out::{json, string, text, to_json};
use crate::platform::Platform;
use serde_json::Value;
use std::path::PathBuf;

/// The directories of the tree, EMUSEN_GALAXIA_DIR_*, the files, EMUSEN_GALAXIA_FILE_*, and the models, EMUSEN_GALAXIA_MODEL_*.
pub mod consts {
    pub use crate::raw::{
        EMUSEN_GALAXIA_DIR_ARTWORK as DIR_ARTWORK, EMUSEN_GALAXIA_DIR_CHEATS as DIR_CHEATS,
        EMUSEN_GALAXIA_DIR_COMPRESSED as DIR_COMPRESSED, EMUSEN_GALAXIA_DIR_CONFIG as DIR_CONFIG, EMUSEN_GALAXIA_DIR_CORES as DIR_CORES,
        EMUSEN_GALAXIA_DIR_FIRMWARE as DIR_FIRMWARE, EMUSEN_GALAXIA_DIR_HOME as DIR_HOME, EMUSEN_GALAXIA_DIR_LIBRARY as DIR_LIBRARY,
        EMUSEN_GALAXIA_DIR_LOGS as DIR_LOGS, EMUSEN_GALAXIA_DIR_MEDIA as DIR_MEDIA, EMUSEN_GALAXIA_DIR_PACKAGES as DIR_PACKAGES,
        EMUSEN_GALAXIA_DIR_ROOT as DIR_ROOT, EMUSEN_GALAXIA_DIR_SAVE_STATES as DIR_SAVE_STATES, EMUSEN_GALAXIA_DIR_SAVES as DIR_SAVES,
        EMUSEN_GALAXIA_DIR_SCREENSHOTS as DIR_SCREENSHOTS, EMUSEN_GALAXIA_DIR_SHADERS as DIR_SHADERS,
        EMUSEN_GALAXIA_DIR_THEMES as DIR_THEMES, EMUSEN_GALAXIA_MODEL_APP_SETTINGS as MODEL_APP_SETTINGS,
        EMUSEN_GALAXIA_MODEL_AUDIO_CONFIG as MODEL_AUDIO_CONFIG, EMUSEN_GALAXIA_MODEL_CHEAT_FILE as MODEL_CHEAT_FILE,
        EMUSEN_GALAXIA_MODEL_GRAPHICS_CONFIG as MODEL_GRAPHICS_CONFIG, EMUSEN_GALAXIA_FILE_GAMES as FILE_GAMES,
        EMUSEN_GALAXIA_FILE_MEDIA as FILE_MEDIA, EMUSEN_GALAXIA_FILE_RECORDS as FILE_RECORDS, EMUSEN_GALAXIA_FILE_SETTINGS as FILE_SETTINGS,
        EMUSEN_GALAXIA_FILE_STATES as FILE_STATES, EMUSEN_GALAXIA_FILE_THEMES as FILE_THEMES,
        EMUSEN_GALAXIA_SAVE_PICTURE as SAVE_PICTURE, EMUSEN_GALAXIA_SAVE_RESUME_STATE as SAVE_RESUME_STATE,
        EMUSEN_GALAXIA_SAVE_SRAM as SAVE_SRAM, EMUSEN_GALAXIA_SAVE_STATE as SAVE_STATE,
    };
}

impl Platform {
    /// A directory of the process's tree, by its EMUSEN_GALAXIA_DIR_* number; none where the tree has none.
    pub fn directory(&self, which: u32) -> Result<Option<PathBuf>> {
        let bytes = self.read(|o, l| unsafe { self.lib.emusen_galaxia_directory(which, o, l) })?;
        bytes.map(|b| string("directory", b).map(PathBuf::from)).transpose()
    }

    /// A file of Galaxia's, by its EMUSEN_GALAXIA_FILE_* number: the settings file or a store.
    pub fn file(&self, which: u32) -> Result<PathBuf> {
        let bytes = self.read(|o, l| unsafe { self.lib.emusen_galaxia_file(which, o, l) })?.unwrap_or_default();
        string("file", bytes).map(PathBuf::from)
    }

    /// A save file's path by its EMUSEN_GALAXIA_SAVE_* kind: `rom` the game's path (a state's, for `SAVE_PICTURE`),
    /// `extra` what the kind names (a console, a directory override), `slot` a state's slot.
    pub fn save_path(&self, kind: u32, rom: &str, extra: Option<&str>, slot: i32) -> Result<PathBuf> {
        let (e, el) = text(extra);
        let bytes = self
            .read(|o, l| unsafe { self.lib.emusen_galaxia_save_path(kind, rom.as_ptr(), rom.len(), e, el, slot, o, l) })?
            .unwrap_or_default();
        string("save_path", bytes).map(PathBuf::from)
    }

    /// A config file's path: a category is a subdirectory of the config directory, none the directory itself.
    pub fn config_path(&self, category: Option<&str>, file: &str) -> Result<PathBuf> {
        let (c, cl) = text(category);
        let bytes =
            self.read(|o, l| unsafe { self.lib.emusen_galaxia_config_path(c, cl, file.as_ptr(), file.len(), o, l) })?.unwrap_or_default();
        string("config_path", bytes).map(PathBuf::from)
    }

    /// A model's file as its bound document, upgraded as settings for use are; none for no file, `Value::Null` for a
    /// file whose document is null.
    pub fn model_load(&self, model: u32, category: Option<&str>, file: &str) -> Result<Option<Value>> {
        let (c, cl) = text(category);
        let flags = crate::raw::EMUSEN_GALAXIA_MODEL_UPGRADE;
        let bytes =
            self.read(|o, l| unsafe { self.lib.emusen_galaxia_model_load(model, c, cl, file.as_ptr(), file.len(), flags, o, l) })?;
        bytes.map(|b| json("model_load", &b)).transpose()
    }

    /// A document saved as a model's file.
    pub fn model_save(&self, model: u32, category: Option<&str>, file: &str, document: &Value) -> Result<()> {
        let (c, cl) = text(category);
        let document = to_json(document);
        let n = unsafe { self.lib.emusen_galaxia_model_save(model, c, cl, file.as_ptr(), file.len(), document.as_ptr(), document.len()) };
        self.status(n.into()).map(drop)
    }

    /// The saved cheat lists by name, in the order Galaxia lists them.
    pub fn cheat_names(&self) -> Result<Vec<String>> {
        let bytes = self.read(|o, l| unsafe { self.lib.emusen_galaxia_cheat_names(o, l) })?.unwrap_or_default();
        Ok(bytes.split(|b| *b == 0).filter(|n| !n.is_empty()).map(|n| String::from_utf8_lossy(n).into_owned()).collect())
    }

    /// Whether a cheat list may have this name; it is checked, never rewritten.
    pub fn cheat_name_valid(&self, name: &str) -> Result<bool> {
        self.status(unsafe { self.lib.emusen_galaxia_cheat_name_valid(name.as_ptr(), name.len()) }.into()).map(|n| n == 1)
    }

    /// A game's own cheat list as a front end keeps it: its name and its category, the file `<name>.json` there; none
    /// for a path with no usable name.
    pub fn cheat_game_list(&self, console: Option<&str>, rom_path: &str) -> Result<Option<(String, String)>> {
        let (c, cl) = text(console);
        let bytes = self.read(|o, l| unsafe { self.lib.emusen_galaxia_cheat_game_list(c, cl, rom_path.as_ptr(), rom_path.len(), o, l) })?;
        Ok(bytes.map(|b| {
            let mut parts = b.split(|b| *b == 0).map(|p| String::from_utf8_lossy(p).into_owned());
            (parts.next().unwrap_or_default(), parts.next().unwrap_or_default())
        }))
    }

    /// The front end's own settings rows (since 45): `{"AppSettings": [...], "GraphicsConfig": [...],
    /// "GraphicsConfig.Consoles": [...]}`, each row in the settings schema's shape, the rows Galaxia's to describe.
    pub fn settings_rows(&self) -> Result<Value> {
        let bytes = self.read(|o, l| unsafe { self.lib.emusen_galaxia_settings_rows(o, l) })?.unwrap_or_default();
        json("settings_rows", &bytes)
    }

    /// A new instance of a model, every field at its initial value.
    pub fn model_new(&self, model: u32) -> Result<Value> {
        let flags = crate::raw::EMUSEN_GALAXIA_MODEL_UPGRADE;
        let bytes = self.read(|o, l| unsafe { self.lib.emusen_galaxia_model_new(model, flags, o, l) })?.unwrap_or_default();
        json("model_new", &bytes)
    }
}
