//! The firmware store's folders; what a game or a system needs, and whether the library holds it, is the frontend API's.

use crate::error::Result;
use crate::out::{json, to_json};
use crate::platform::Platform;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Where firmware is searched, from the FirmwareDirectory setting, the player's home and the store.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct FoldersRequest {
    pub setting: Option<String>,
    pub home: String,
    pub store: String,
}

/// The folders searched, first to last, and the setting's standing: "Unset", "Usable", "Missing", "Relative" or "File".
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Folders {
    pub folders: Vec<String>,
    pub state: String,
    pub path: Option<String>,
    pub written: Option<Value>,
    pub problem: Option<String>,
}

impl Platform {
    /// The firmware folders for a setting.
    pub fn firmware_folders(&self, request: &FoldersRequest) -> Result<Folders> {
        let request = to_json(request);
        let bytes =
            self.read(|o, l| unsafe { self.lib.emusen_firmware_folders(request.as_ptr(), request.len(), o, l) })?.unwrap_or_default();
        json("firmware_folders", &bytes)
    }
}
