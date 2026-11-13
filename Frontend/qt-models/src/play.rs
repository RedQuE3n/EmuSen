//! What a running game and its window take from the player's settings (EmuSen_Qt.md §1.8.6): the game's Volume and
//! Output, Sync to display, Pause in background, and the lines over the game. Each key is read through the Settings
//! description's `applies`, as the program's other settings are, so it is named in one place; plain data, no Qt.

use crate::settings::{applied_in, applies_as_in, Model, SettingsSource, DESCRIPTION};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct PlaySettings {
    /// The game's volume, a share of full (the media viewer's clip reads it the same way).
    pub volume: f64,
    /// Output as stored: a device's name, or the setting's own initial value for the system default.
    pub output: String,
    pub sync_to_display: bool,
    pub pause_in_background: bool,
    pub show_fps: bool,
    pub show_status_text: bool,
    pub show_status_bar: bool,
}

impl PlaySettings {
    /// From the description and the two stored models.
    pub fn of(description: &Value, app: &Value, graphics: &Value) -> PlaySettings {
        let on = |name: &str| applies_as_in(description, app, graphics, name);
        PlaySettings {
            volume: crate::esde_shelves::volume_of(app),
            output: applied_in(description, app, graphics, "output").unwrap_or_default(),
            sync_to_display: on("sync_to_display"),
            pause_in_background: on("pause_in_background"),
            show_fps: on("show_fps"),
            show_status_text: on("show_status_text"),
            show_status_bar: on("show_status_bar"),
        }
    }

    /// Read from a source, as a game opens; a model with no file yet is at its initial values.
    pub fn read(source: &dyn SettingsSource) -> Result<PlaySettings, String> {
        let description: Value = serde_json::from_str(DESCRIPTION).map_err(|e| e.to_string())?;
        Ok(PlaySettings::of(&description, &source.load(Model::App)?, &source.load(Model::Graphics)?))
    }
}

/// The device a stored Output names for the game's player: none for the setting's initial value, the system default.
pub fn device(output: &str, initial: &str) -> Option<String> {
    (!output.is_empty() && output != initial).then(|| output.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn description() -> Value {
        serde_json::from_str(DESCRIPTION).unwrap()
    }

    #[test]
    fn each_setting_is_read_from_its_own_key() {
        let app = json!({"Volume": 0.25, "SoundOutputDevice": "Speakers", "PauseInBackground": false, "ShowFpsBar": true,
            "ShowStatusText": false, "ShowStatusBar": true});
        let graphics = json!({"SyncToDisplay": false});
        let read = PlaySettings::of(&description(), &app, &graphics);
        assert_eq!(
            read,
            PlaySettings {
                volume: 0.25,
                output: "Speakers".into(),
                sync_to_display: false,
                pause_in_background: false,
                show_fps: true,
                show_status_text: false,
                show_status_bar: true
            }
        );
        let turned = PlaySettings::of(&description(), &json!({"PauseInBackground": true, "ShowFpsBar": false, "ShowStatusText": true}), &json!({"SyncToDisplay": true}));
        assert!(turned.pause_in_background && !turned.show_fps && turned.show_status_text && turned.sync_to_display);
    }

    #[test]
    fn the_initial_output_is_the_system_default() {
        assert_eq!(device("System default", "System default"), None);
        assert_eq!(device("", "System default"), None);
        assert_eq!(device("Speakers", "System default"), Some("Speakers".into()));
    }
}
