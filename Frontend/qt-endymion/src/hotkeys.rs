//! The emulator's actions a hotkey runs, by the words the binding files name them with (emusen_platform.h, the HOTKEYS
//! file; Mistress's `HotkeyAction`), and which of them last while the key is held (EmuSen_Qt.md §7.7.H6).

/// An action of the emulator's, bound to a key in hotkeybindings.json.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hotkey {
    FastForward,
    Rewind,
    SaveState,
    LoadState,
    TogglePause,
    ToggleFullscreen,
    ExitToLibrary,
    Screenshot,
    ToggleBigPicture,
    FrameAdvance,
}

/// Every action, in the header's order, by its word.
const WORDS: [(Hotkey, &str); 10] = [
    (Hotkey::FastForward, "FastForward"),
    (Hotkey::Rewind, "Rewind"),
    (Hotkey::SaveState, "SaveState"),
    (Hotkey::LoadState, "LoadState"),
    (Hotkey::TogglePause, "TogglePause"),
    (Hotkey::ToggleFullscreen, "ToggleFullscreen"),
    (Hotkey::ExitToLibrary, "ExitToLibrary"),
    (Hotkey::Screenshot, "Screenshot"),
    (Hotkey::ToggleBigPicture, "ToggleBigPicture"),
    (Hotkey::FrameAdvance, "FrameAdvance"),
];

impl Hotkey {
    /// The action a file's word names, in any case, as Endymion reads it.
    pub fn named(word: &str) -> Option<Hotkey> {
        WORDS.iter().find(|(_, w)| w.eq_ignore_ascii_case(word)).map(|(h, _)| *h)
    }

    /// The action's word in the files.
    pub fn word(self) -> &'static str {
        WORDS.iter().find(|(h, _)| *h == self).map_or("", |(_, w)| w)
    }

    /// Whether the action lasts while its key is held (Fast Forward, Rewind) rather than firing once as it goes down
    /// (Mistress's `IsHeld`).
    pub fn held(self) -> bool {
        matches!(self, Hotkey::FastForward | Hotkey::Rewind)
    }
}
