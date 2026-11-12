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

/// Every action, in the header's order.
const ALL: [Hotkey; 10] = [
    Hotkey::FastForward,
    Hotkey::Rewind,
    Hotkey::SaveState,
    Hotkey::LoadState,
    Hotkey::TogglePause,
    Hotkey::ToggleFullscreen,
    Hotkey::ExitToLibrary,
    Hotkey::Screenshot,
    Hotkey::ToggleBigPicture,
    Hotkey::FrameAdvance,
];

impl Hotkey {
    /// The action a file's word names, in any case, as Endymion reads it: the word is the action's own name, held
    /// against Endymion's `bindings_defaults` by the tests (EmuSen_Qt.md §7.7.H7).
    pub fn named(word: &str) -> Option<Hotkey> {
        ALL.into_iter().find(|h| h.word().eq_ignore_ascii_case(word))
    }

    /// The action's word in the files.
    pub fn word(self) -> String {
        format!("{self:?}")
    }

    /// Whether the action lasts while its key is held (Fast Forward, Rewind) rather than firing once as it goes down
    /// (Mistress's `IsHeld`).
    pub fn held(self) -> bool {
        matches!(self, Hotkey::FastForward | Hotkey::Rewind)
    }
}
