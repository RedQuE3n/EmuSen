//! What a running game and its window take from the player's settings (EmuSen_Qt.md §1.8.6): the game's Volume and
//! Output, Sync to display, Pause in background, and the lines over the game. Each key is read through the Settings
//! description's `applies`, as the program's other settings are, so it is named in one place; plain data, no Qt.

use crate::settings::{applied_in, applies_as_in, Model, SettingsSource, DESCRIPTION};
use cxx_qt_lib::{QMap, QMapPair_QString_QVariant, QString, QVariant};
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

/// The window's size and whether the player may resize it while it is not full screen: graphics.json's WindowWidth,
/// WindowHeight and WindowResizable (Galaxia's `GraphicsConfig`, EmuSen_Qt.md §7.7.H7). A full-screen window never takes
/// them: its screen sizes it. Read whole or not at all: where any of the three is missing, or a size is not a positive
/// whole number, there is nothing to give the window, and Qt sizes it and leaves it free, as for a window given nothing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Windowed {
    pub width: i32,
    pub height: i32,
    pub resizable: bool,
}

impl Windowed {
    /// From the description and the two stored models; None where they do not hold all three.
    pub fn of(description: &Value, app: &Value, graphics: &Value) -> Option<Windowed> {
        let stored = |name: &str| applied_in(description, app, graphics, name).filter(|v| !v.is_empty());
        let size = |name: &str| stored(name).and_then(|v| v.parse::<i32>().ok()).filter(|n| *n > 0);
        stored("window_resizable")?;
        Some(Windowed { width: size("window_width")?, height: size("window_height")?, resizable: applies_as_in(description, app, graphics, "window_resizable") })
    }

    /// Read from a source; a model with no file yet is at its initial values (the base's `model_new`).
    pub fn read(source: &dyn SettingsSource) -> Result<Option<Windowed>, String> {
        let description: Value = serde_json::from_str(DESCRIPTION).map_err(|e| e.to_string())?;
        Ok(Windowed::of(&description, &source.load(Model::App)?, &source.load(Model::Graphics)?))
    }

    /// As a window's initial properties, by the names both programs' windows declare.
    pub fn insert_into(&self, initial: &mut QMap<QMapPair_QString_QVariant>) {
        initial.insert(QString::from("windowedWidth"), QVariant::from(&self.width));
        initial.insert(QString::from("windowedHeight"), QVariant::from(&self.height));
        initial.insert(QString::from("windowResizable"), QVariant::from(&self.resizable));
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
    fn the_window_size_is_read_from_graphics_json() {
        let graphics = json!({"WindowWidth": 1500, "WindowHeight": 900, "WindowResizable": false});
        assert_eq!(Windowed::of(&description(), &json!({}), &graphics), Some(Windowed { width: 1500, height: 900, resizable: false }));
        let turned = Windowed::of(&description(), &json!({}), &json!({"WindowWidth": 700, "WindowHeight": 500, "WindowResizable": true}));
        assert_eq!(turned, Some(Windowed { width: 700, height: 500, resizable: true }));
    }

    #[test]
    fn a_window_size_not_read_whole_is_nothing() {
        let of = |graphics: Value| Windowed::of(&description(), &json!({}), &graphics);
        assert_eq!(of(Value::Null), None, "no model");
        assert_eq!(of(json!({"WindowWidth": 700, "WindowHeight": 500})), None, "no WindowResizable");
        assert_eq!(of(json!({"WindowHeight": 500, "WindowResizable": false})), None, "no WindowWidth");
        assert_eq!(of(json!({"WindowWidth": 0, "WindowHeight": 500, "WindowResizable": false})), None, "a width of none");
        assert_eq!(of(json!({"WindowWidth": 700, "WindowHeight": -5, "WindowResizable": false})), None, "a height below none");
    }

    #[test]
    fn the_initial_output_is_the_system_default() {
        assert_eq!(device("System default", "System default"), None);
        assert_eq!(device("", "System default"), None);
        assert_eq!(device("Speakers", "System default"), Some("Speakers".into()));
    }
}
