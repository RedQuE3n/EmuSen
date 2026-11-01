//! How a core that took a game says it cannot run it, where libretro gives it no way to: a line of its log, named for each
//! core by its file stem as measured, which ends the machine with the core's own words (EmuSen_LibretroHost.md §10n).

use emusen_native::core::Level;

/// By a core's file stem, the lines that say its machine could not start. PCEE2 returns true from `retro_load_game`,
/// starts its machine in the first `retro_run`, and says it failed only in this line, at info level, measured 2026-11-01.
pub const FATAL_LINES: &[(&str, &[&str])] = &[("pcee2", &["VMManager::Initialize() failed"]), ("testcore", &["the test machine could not start"])];

/// What a call's log records said: the first fatal line of the core's profile, and the last error or warning.
#[derive(Default)]
pub struct Heard {
    pub fatal: Option<String>,
    pub last_error: Option<String>,
}

impl Heard {
    /// One record read against the profile of the core `stem`.
    pub fn hear(&mut self, stem: &str, level: Level, text: &str) {
        let lines = FATAL_LINES.iter().find(|p| p.0 == stem).map_or(&[][..], |p| p.1);
        if self.fatal.is_none() && lines.iter().any(|l| text.contains(l)) {
            self.fatal = Some(text.to_owned());
        }
        if matches!(level, Level::Error | Level::Warn) {
            self.last_error = Some(text.to_owned());
        }
    }
}

/// The words of a machine that could not start, quoting the core.
pub fn could_not_start(line: &str) -> String {
    format!("it could not start: {line}")
}
