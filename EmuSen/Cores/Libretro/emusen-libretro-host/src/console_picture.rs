//! The console's own picture: the host's reserved setting `console_picture`, which a frontend sets while the console's CRT
//! filter is on, and each core's named exceptions it then applies and locks (EmuSen_LibretroHost.md §10i).

use emusen_native::core::{Effect, Scope, Setting, SettingKind};

/// The reserved key, outside every libretro core's options since none declares it.
pub const KEY: &str = "console_picture";

/// The note key listing the locked rows, space-separated; no setting can have it, `!` being outside a key's characters.
pub const LOCKED: &str = "!locked";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Exception {
    pub key: String,
    pub value: String,
    pub reason: String,
    pub decided: String,
}

const DATA: &str = include_str!("../data/console-picture.tsv");

/// The named exceptions of the core with this file stem, from the data file, in its order.
pub fn exceptions(stem: &str) -> Vec<Exception> {
    parse(DATA, stem)
}

fn parse(text: &str, stem: &str) -> Vec<Exception> {
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .filter_map(|l| match l.split('\t').collect::<Vec<_>>().as_slice() {
            [s, key, value, reason, decided] if *s == stem => Some(Exception { key: key.to_string(), value: value.to_string(), reason: reason.to_string(), decided: decided.to_string() }),
            _ => None,
        })
        .collect()
}

/// The reserved setting as the schema declares it.
pub fn setting() -> Setting {
    Setting {
        key: KEY.into(),
        label: "Console's own picture".into(),
        help: "On while the console's CRT filter is on: the core's options that would change the console's picture are held at the console's values, and given back when the filter is turned off.".into(),
        kind: SettingKind::Switch,
        default: "false".into(),
        scope: Scope::Run,
        category: Some("Screen filter".into()),
        effect: Effect::None,
        advanced: false,
        hidden: false,
        restart: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_data_names_pcee2s_two_exceptions_and_skips_comments() {
        let e = exceptions("pcee2");
        assert_eq!(e.iter().map(|x| (x.key.as_str(), x.value.as_str())).collect::<Vec<_>>(), [("pcsx2_upscale_multiplier", "1"), ("pcsx2_deinterlace_mode", "1")]);
        assert!(e.iter().all(|x| x.decided == "2026-11-01" && !x.reason.is_empty()));
        assert!(exceptions("stem").is_empty());
        assert!(setting().check().is_ok());
    }
}
