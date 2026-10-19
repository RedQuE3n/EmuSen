//! A cartridge's battery save, the C# `BatterySave`: its console's folder in Saves, the no-battery latch, the atomic
//! write, the flush period, and a save kept in its earlier place read there and copied in once a load keeps the file
//! (EmuSen_Settings_Reference.md §4.85.6, §4.85.11, EmuSen_Backend.md §17.10).

use emusen_galaxia::atomic;
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::saves;
use emusen_galaxia::tree::Tree;

/// How often a save is written without being asked, about five seconds at sixty frames.
pub const FLUSH_EVERY: i64 = 300;

pub fn is_flush_frame(frame: i64) -> bool {
    frame % FLUSH_EVERY == 0
}

/// One battery file of a machine: where it is kept, none when nothing is read or written.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BatterySave {
    pub path: Option<String>,
    previous: Option<String>,
}

impl BatterySave {
    pub const NONE: BatterySave = BatterySave { path: None, previous: None };

    /// A game's own file, `BatterySave.Open`.
    pub fn open(tree: &Tree, rom: &str, console: &str, disabled: bool, extension: &str) -> BatterySave {
        if disabled || rom.is_empty() {
            return BatterySave::NONE;
        }
        let at = path::change_extension(Style::HOST, &saves::sram_path(tree, rom, console), extension);
        let previous = if emusen_galaxia::dotnet_files::file_exists(&at) { None } else { previous_home(tree, rom, console, extension) };
        BatterySave { path: Some(at), previous }
    }

    /// A shared or named medium's file, `BatterySave.Medium`.
    pub fn medium(tree: &Tree, console: &str, console_kind: bool, name: &str, extension: &str, disabled: bool) -> BatterySave {
        if disabled {
            return BatterySave::NONE;
        }
        let folder = crate::media::medium_folder(tree, console, console_kind);
        BatterySave { path: Some(path::combine(Style::HOST, &folder, &format!("{name}{extension}"))), previous: None }
    }

    /// The save kept before copied into its own place, once a load knows the machine keeps the file.
    pub fn settle(&self, say: &dyn Fn(String)) {
        if let (Some(at), Some(previous)) = (&self.path, &self.previous)
            && !emusen_galaxia::dotnet_files::file_exists(at)
        {
            if path::full_path(previous) == path::full_path(at) {
                return;
            }
            let Ok(Some(saved)) = atomic::try_read(previous) else { return };
            if atomic::write(at, &saved).is_ok() {
                say(format!("[BatterySave] Copied {previous} to {at}, where this game's battery save now lives; the original is untouched."));
            }
        }
    }

    pub fn read(&self) -> Option<Vec<u8>> {
        let at = self.path.as_ref()?;
        if let Ok(Some(bytes)) = atomic::try_read(at) {
            return Some(bytes);
        }
        match &self.previous {
            Some(previous) if !emusen_galaxia::dotnet_files::file_exists(at) => atomic::try_read(previous).ok().flatten(),
            _ => None,
        }
    }

    pub fn write(&self, contents: &[u8]) -> bool {
        self.path.as_ref().is_some_and(|at| atomic::write(at, contents).is_ok())
    }
}

/// The one place each console's saves were kept before, `PreviousHome`.
pub fn previous_home(tree: &Tree, rom: &str, console: &str, extension: &str) -> Option<String> {
    let change = |p: &str| path::change_extension(Style::HOST, p, extension);
    match console {
        "SNES" | "N64" => Some(change(&saves::flat_sram_path(tree, rom))),
        "NES" | "GB" | "GBC" => Some(change(rom)),
        "Genesis" => Some(change(&saves::sram_path(tree, rom, "MD"))),
        "Sega CD" => Some(change(&saves::sram_path(tree, rom, "MCD"))),
        _ => None,
    }
}
