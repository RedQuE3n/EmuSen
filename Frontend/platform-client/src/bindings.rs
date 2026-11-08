//! Endymion's binding files through the platform library: keybindings.json, gamepadbindings.json, gamepadprofiles.json
//! and hotkeybindings.json, each read as a program starts with it, saved, and a map's defaults. A model crosses as JSON
//! and stays Endymion's: its words and shapes are emusen_platform.h's, not restated here.

use crate::error::Result;
use crate::out::{json, to_json};
use crate::platform::Platform;
use serde_json::Value;

/// The four files, EMUSEN_ENDYMION_BINDINGS_*.
pub mod consts {
    pub use crate::raw::{
        EMUSEN_ENDYMION_BINDINGS_GAMEPAD as GAMEPAD, EMUSEN_ENDYMION_BINDINGS_HOTKEYS as HOTKEYS, EMUSEN_ENDYMION_BINDINGS_KEYS as KEYS,
        EMUSEN_ENDYMION_BINDINGS_PROFILES as PROFILES,
    };
}

impl Platform {
    /// A binding file's model for the consoles a program knows; a file that would not read is its defaults, and its
    /// diagnostic waits in `diagnostics`.
    pub fn bindings_load(&self, which: u32, consoles: &[&str]) -> Result<Value> {
        let names: Vec<u8> = consoles.iter().flat_map(|c| c.bytes().chain([0])).collect();
        let bytes =
            self.read(|o, l| unsafe { self.lib.emusen_endymion_bindings_load(which, names.as_ptr(), names.len(), o, l) })?.unwrap_or_default();
        json("bindings_load", &bytes)
    }

    /// A model saved as its binding file.
    pub fn bindings_save(&self, which: u32, model: &Value) -> Result<()> {
        let document = to_json(model);
        self.status(unsafe { self.lib.emusen_endymion_bindings_save(which, document.as_ptr(), document.len()) }.into()).map(drop)
    }

    /// One map at its defaults: a console's keys, a console's gamepad buttons, or the hotkeys.
    pub fn bindings_defaults(&self, which: u32) -> Result<Value> {
        let bytes = self.read(|o, l| unsafe { self.lib.emusen_endymion_bindings_defaults(which, o, l) })?.unwrap_or_default();
        json("bindings_defaults", &bytes)
    }
}
