//! What the program reads before its window opens: the platform library's path and the player's settings that shape the
//! start, as plain values handed to the window's root as its initial properties (EmuSen_Qt.md §1.2.1: no static carries them).

use emusen_platform_client::Platform;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// The platform library's file name, found beside the program when nothing names another.
pub const LIBRARY_FILE: &str = "libemusen_platform.so";

/// The start's facts, each an initial property of the window.
#[derive(Clone, Debug, PartialEq)]
pub struct Start {
    pub platform_library: String,
    /// Moonrise at the start: AppSettings.StartupSplash, on unless the player turned it off (EmuSen_Interface.md §22).
    pub splash: bool,
}

/// The platform library: `--platform-library PATH`, else `EMUSEN_PLATFORM_LIBRARY`, else the file beside the program.
pub fn platform_library(args: &[String], env: Option<String>, program: &Path) -> PathBuf {
    let named = args.iter().position(|a| a == "--platform-library").and_then(|i| args.get(i + 1)).cloned();
    named.or(env).map(PathBuf::from).unwrap_or_else(|| program.parent().unwrap_or(Path::new(".")).join(LIBRARY_FILE))
}

/// Moonrise on or off from the settings as Galaxia's model reads them; on when the file or the key is missing.
pub fn splash_from(settings: &Value) -> bool {
    settings.get("StartupSplash").and_then(Value::as_bool).unwrap_or(true)
}

/// The start read through the platform library at `library`; its settings unread (the defaults) when it cannot load.
pub fn read(library: &Path) -> Start {
    let settings =
        Platform::load(library).ok().and_then(|platform| emusen_qt_models::clients::app_settings(&platform).ok()).unwrap_or(Value::Null);
    Start { platform_library: library.to_string_lossy().into_owned(), splash: splash_from(&settings) }
}
