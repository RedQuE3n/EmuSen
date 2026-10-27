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
    /// Whether `previous` is a game's own file a console medium adopts, rather than this save's earlier home.
    adopts: bool,
}

impl BatterySave {
    pub const NONE: BatterySave = BatterySave { path: None, previous: None, adopts: false };

    /// A game's own file, `BatterySave.Open`: in its package where it opened as one, the plain save then read until
    /// copied in, as an earlier place is (EmuSen_GamePackages.md §3.5); a patched game's by its list's key, with nothing
    /// read before it, so that a hack starts its own save and never reaches the original's (§5.5).
    pub fn open(tree: &Tree, rom: &str, console: &str, disabled: bool, extension: &str, package: Option<&str>, key: Option<&str>) -> BatterySave {
        if disabled || rom.is_empty() {
            return BatterySave::NONE;
        }
        let exists = emusen_galaxia::dotnet_files::file_exists;
        let plain = path::change_extension(Style::HOST, &saves::sram_path(tree, rom, console), extension);
        let Some(root) = package.filter(|p| !p.is_empty()) else {
            let previous = if exists(&plain) { None } else { previous_home(tree, rom, console, extension) };
            return BatterySave { path: Some(plain), previous, adopts: false };
        };
        let at = path::change_extension(Style::HOST, &saves::package_sram_path(root, key), extension);
        let previous = if exists(&at) || key.is_some_and(|k| !k.is_empty()) {
            None
        } else if exists(&plain) {
            Some(plain)
        } else {
            previous_home(tree, rom, console, extension)
        };
        BatterySave { path: Some(at), previous, adopts: false }
    }

    /// A shared or named medium's file, adopting a game's own file that `settle` copies in where it has none,
    /// `BatterySave.Medium` (EmuSen_SaveMedia.md §3.6).
    pub fn medium(tree: &Tree, console: &str, console_kind: bool, name: &str, extension: &str, disabled: bool, adopt: Option<String>) -> BatterySave {
        if disabled {
            return BatterySave::NONE;
        }
        let folder = crate::media::medium_folder(tree, console, console_kind);
        let adopts = adopt.is_some();
        BatterySave { path: Some(path::combine(Style::HOST, &folder, &format!("{name}{extension}"))), previous: adopt, adopts }
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
            if atomic::write(at, &saved).is_err() {
                return;
            }
            say(if self.adopts {
                format!("[BatterySave] Copied {previous}, the most recently written game's own, to {at}, the console's memory every game now shares; each game's own is untouched.")
            } else {
                format!("[BatterySave] Copied {previous} to {at}, where this game's battery save now lives; the original is untouched.")
            });
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
