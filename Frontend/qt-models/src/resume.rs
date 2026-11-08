//! Where a game was left off (EmuSen_Interface.md §18.3, Mistress's `ChooseResumeAsync`): the player's setting, what a
//! game's start does with it and the runtime's resume state, and what each answer to the Resume question comes to.

/// The setting Resume (`AppSettings.ResumeOnLaunch`): ask each time, always resume, never resume.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setting {
    Ask,
    Always,
    Never,
}

/// The setting's key in the player's settings.
pub const KEY: &str = "ResumeOnLaunch";

impl Setting {
    /// The stored value, Galaxia's words: `Ask`, `Resume` or `Restart`; anything else asks, as the default does.
    pub fn read(stored: &str) -> Setting {
        match stored {
            "Resume" => Setting::Always,
            "Restart" => Setting::Never,
            _ => Setting::Ask,
        }
    }
}

/// What a game's start does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Start {
    /// From the beginning: no resume state, or the setting says never.
    Fresh,
    /// Where it was left off, without asking.
    Load,
    /// The Resume question first, the game held until it is answered.
    Ask,
}

/// What the start does, given the setting and whether the runtime holds a resume state for the game (slot 0).
pub fn start(setting: Setting, left_off: bool) -> Start {
    match (setting, left_off) {
        (_, false) | (Setting::Never, _) => Start::Fresh,
        (Setting::Always, true) => Start::Load,
        (Setting::Ask, true) => Start::Ask,
    }
}

/// An answer to the Resume question, by the index the kit's `Question` gives: Continue, Start again, or B (-1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    Load,
    Fresh,
    /// Back to the library, the game closed unplayed.
    Back,
}

pub fn answer(index: i32) -> Answer {
    match index {
        0 => Answer::Load,
        1 => Answer::Fresh,
        _ => Answer::Back,
    }
}

/// The game's name in the question, as Mistress's `ResumeWindow` names it: its file name without the extension.
pub fn subject(path: &str) -> String {
    std::path::Path::new(path).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}
