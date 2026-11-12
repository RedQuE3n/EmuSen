//! The front end's own settings rows as data: each key of `AppSettings` and `GraphicsConfig` a settings screen sets,
//! with its label, sentence and choices, in the frontend API's `settings_schema` row shape, so that a front end draws a
//! core's rows and its own by one path. The file, `data/frontend_settings.json`, sits beside the models that own the
//! keys and is embedded when the crate is built. See EmuSen_Galaxia.md §3.15.

use serde::Deserialize;
use std::sync::OnceLock;

/// The rows, as committed.
pub const TEXT: &str = include_str!("../data/frontend_settings.json");

/// A row's kinds, the schema's three.
pub const KINDS: [&str; 3] = ["switch", "count", "choice"];

/// One row, every member of the schema's shape written; `note` is an engine's alone and never one of these.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    /// The model's key: a field by its file name, a nested model's as `Model.Field`; a console's own key in `GraphicsConfig.Consoles`.
    pub key: String,
    pub label: String,
    pub hint: String,
    pub kind: String,
    /// The value the model starts with, in its text form.
    pub default: String,
    pub min: i64,
    pub max: i64,
    /// The values stored, and the words shown for each; none for a switch.
    pub choices: Vec<String>,
    pub choice_labels: Vec<String>,
    /// The page the row is on.
    pub group: Option<String>,
    pub shown_with: Option<ShownWith>,
    pub locked: bool,
}

/// The key and value a row means something only with.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShownWith {
    pub key: String,
    pub value: String,
}

/// The rows by the model that owns their keys.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rows {
    #[serde(rename = "AppSettings")]
    pub app_settings: Vec<Row>,
    #[serde(rename = "GraphicsConfig")]
    pub graphics: Vec<Row>,
    /// A console's own keys, held in `GraphicsConfig`'s `Consoles` map.
    #[serde(rename = "GraphicsConfig.Consoles")]
    pub console_graphics: Vec<Row>,
}

impl Rows {
    /// Every row with the model section it is in.
    pub fn all(&self) -> impl Iterator<Item = (&'static str, &Row)> + '_ {
        let app = self.app_settings.iter().map(|r| ("AppSettings", r));
        app.chain(self.graphics.iter().map(|r| ("GraphicsConfig", r))).chain(self.console_graphics.iter().map(|r| ("GraphicsConfig.Consoles", r)))
    }
}

/// Text read as rows, refused with words where a row is not one: a kind the schema lacks, choices and their words that
/// differ in number, a switch with choices, a default the row cannot hold, or a key twice in a section.
pub fn parse(text: &str) -> Result<Rows, String> {
    let rows: Rows = serde_json::from_str(text).map_err(|e| e.to_string())?;
    for (section, list) in [("AppSettings", &rows.app_settings), ("GraphicsConfig", &rows.graphics), ("GraphicsConfig.Consoles", &rows.console_graphics)] {
        for (i, r) in list.iter().enumerate() {
            check(r).map_err(|what| format!("{section}.{}: {what}", r.key))?;
            if list[..i].iter().any(|o| o.key == r.key) {
                return Err(format!("{section}.{}: the key has a row already", r.key));
            }
        }
    }
    Ok(rows)
}

fn check(r: &Row) -> Result<(), String> {
    if r.key.trim().is_empty() || r.label.trim().is_empty() || r.hint.trim().is_empty() {
        return Err("a key, label or sentence is blank".into());
    }
    match r.kind.as_str() {
        "switch" if !r.choices.is_empty() || !r.choice_labels.is_empty() => Err("a switch has no choices".into()),
        "switch" if r.default != "true" && r.default != "false" => Err(format!("a switch's default is true or false, not {}", r.default)),
        "choice" if r.choices.len() != r.choice_labels.len() => Err("each choice has its words".into()),
        "choice" if !r.choices.contains(&r.default) => Err(format!("the default {} is none of its choices", r.default)),
        "count" if !r.default.parse::<i64>().is_ok_and(|d| (r.min..=r.max).contains(&d)) => Err("a count's default is a whole number in its range".into()),
        k if !KINDS.contains(&k) => Err(format!("the kind {k} is none of {}", KINDS.join(", "))),
        _ => Ok(()),
    }
}

/// The embedded rows, read once; an error only if the file was committed broken, which the tests refuse.
pub fn rows() -> Result<&'static Rows, &'static String> {
    static ROWS: OnceLock<Result<Rows, String>> = OnceLock::new();
    ROWS.get_or_init(|| parse(TEXT)).as_ref()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AppSettings, GraphicsConfig};
    use serde_json::Value;

    /// Every key the C# settings rows set (`SettingsScreen.Sections.cs` with the rows it draws from
    /// `SettingsScreen.Controllers.cs` and `.Keys.cs`, and `PlayerPreferencesRows.cs`), read on 2026-11-09.
    const CSHARP_KEYS: [(&str, &str); 59] = [
        ("AppSettings", "Rumble"),
        ("AppSettings", "SwapPadButtons"),
        ("AppSettings", "GuideChords"),
        ("AppSettings", "ControllerType"),
        ("AppSettings", "FirstControllerOnly"),
        ("AppSettings", "ControllerNotifications"),
        ("AppSettings", "OnScreenKeyboard"),
        ("AppSettings", "AnalogStickAsDpad"),
        ("AppSettings", "StickDeadzone"),
        ("AppSettings", "MirrorPlayer1ToPlayer2"),
        ("AppSettings", "KeyboardPlayer"),
        ("AppSettings", "ClaimPlayersByPress"),
        ("AppSettings", "ExternalControllerFirst"),
        ("AppSettings", "TextSize"),
        ("AppSettings", "HomeArt"),
        ("AppSettings", "ReducedMotion"),
        ("AppSettings", "StartupSplash"),
        ("AppSettings", "Appearance"),
        ("AppSettings", "PlayVideos"),
        ("AppSettings", "ShowStatusBar"),
        ("AppSettings", "ShowStatusText"),
        ("AppSettings", "ShowFpsBar"),
        ("AppSettings", "Volume"),
        ("AppSettings", "NavigationSounds"),
        ("AppSettings", "BigPictureInterface.NavigationVolume"),
        ("AppSettings", "SoundOutputDevice"),
        ("AppSettings", "ShowHiddenGames"),
        ("AppSettings", "Scraping"),
        ("AppSettings", "ScrapeRegion"),
        ("AppSettings", "ScrapeRegionFallback"),
        ("AppSettings", "ScrapeLanguage"),
        ("AppSettings", "ScrapeGameNames"),
        ("AppSettings", "ScrapeRefresh"),
        ("AppSettings", "OpenEmuFallback"),
        ("AppSettings", "ScrapeCovers"),
        ("AppSettings", "ScrapeScreenshots"),
        ("AppSettings", "ScrapeMarquees"),
        ("AppSettings", "ScrapeMiximages"),
        ("AppSettings", "ScrapeTitleScreens"),
        ("AppSettings", "ScrapeBackCovers"),
        ("AppSettings", "ScrapePhysicalMedia"),
        ("AppSettings", "ScrapeFanArt"),
        ("AppSettings", "ScrapeManuals"),
        ("AppSettings", "ScrapeVideos"),
        ("AppSettings", "Scrape3DBoxes"),
        ("AppSettings", "BigPictureCollections.GroupCustomCollections"),
        ("AppSettings", "BigPictureCollections.FavoritesFirst"),
        ("AppSettings", "BigPictureCollections.FavoritesFirstCustom"),
        ("AppSettings", "BigPictureCollections.StarsCustom"),
        ("AppSettings", "BigPictureCollections.DefaultSortOrder"),
        ("AppSettings", "BigPictureCollections.FoldersOnTop"),
        ("AppSettings", "BigPictureCollections.RandomEntryButton"),
        ("AppSettings", "BigScreen"),
        ("AppSettings", "PauseInBackground"),
        ("AppSettings", "ResumeOnLaunch"),
        ("AppSettings", "MaxPlayTimeTracking"),
        ("GraphicsConfig", "SyncToDisplay"),
        ("GraphicsConfig.Consoles", "PictureShape"),
        ("GraphicsConfig.Consoles", "Overscan"),
    ];

    /// The media viewer's settings, ES-DE's, which no C# row set: five of its media viewer settings menu and the video's
    /// audio, which ES-DE offers under its sound settings; their rows are the tester's of 2026-11-09, R-20261109-m2
    /// (EmuSen_Galaxia.md §3.15.1). The scanlines are not offered.
    const VIEWER_KEYS: [&str; 6] = [
        "BigPictureInterface.MediaViewerHelpPrompts",
        "BigPictureInterface.MediaViewerShowTypes",
        "BigPictureInterface.MediaViewerKeepVideoRunning",
        "BigPictureInterface.MediaViewerStretchVideos",
        "BigPictureInterface.MediaViewerVideoBlur",
        "BigPictureInterface.MediaViewerVideoAudio",
    ];

    /// Keys the C# rows set that have no row here, each with its owner (EmuSen_Galaxia.md §3.15).
    const NOT_ROWS: [(&str, &str); 14] = [
        ("RomDirectory", "a folder chosen through a picker"),
        ("ArtworkDirectory", "a folder chosen through a picker"),
        ("EsdeMediaDirectory", "a folder chosen through a picker"),
        ("LogDirectory", "a folder chosen through a picker"),
        ("StateDirectory", "a folder chosen through a picker"),
        ("BigPictureTheme", "the Themes tab"),
        ("LibraryStyle", "the Themes tab"),
        ("EmuSenTheme", "the Themes tab"),
        ("BigPictureCollections.AutoCollections", "a list, one switch for each member"),
        ("BigPictureCollections.HiddenCustomCollections", "a list, one switch for each collection"),
        ("BigPictureCollections.FlattenedSystems", "a list, one switch for each ES-DE system"),
        ("ScreenFilter", "Serenity's filter catalogue, emusen_serenity_filters"),
        ("ShaderParameters", "Serenity's filter catalogue and a preset's parameters"),
        ("CropLeft", "a console's page, with CropRight, CropTop and CropBottom"),
    ];

    fn model_value(model: &Value, key: &str) -> Option<Value> {
        key.split('.').try_fold(model.clone(), |v, part| v.get(part).cloned())
    }

    /// A model's value in a row's text form.
    fn text(v: &Value) -> String {
        match v {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.as_f64().map_or_else(|| n.to_string(), |f| f.to_string()),
            other => other.to_string(),
        }
    }

    #[test]
    fn the_committed_rows_read_and_each_is_a_settings_schema_row() {
        let rows = rows().expect("the committed file reads");
        let schema = ["key", "label", "hint", "kind", "default", "min", "max", "choices", "choice_labels", "group", "shown_with", "locked"];
        let doc: serde_json::Map<String, Value> = serde_json::from_str(TEXT).unwrap();
        for list in doc.values() {
            for row in list.as_array().unwrap() {
                assert_eq!(row.as_object().unwrap().keys().map(String::as_str).collect::<Vec<_>>(), schema, "{row}");
            }
        }
        assert!(rows.all().all(|(_, r)| r.group.is_some()), "every row names its page");
    }

    #[test]
    fn every_key_the_csharp_rows_set_and_the_viewer_s_has_exactly_one_row_and_no_other_has_one() {
        let rows = rows().unwrap();
        for (section, key) in CSHARP_KEYS.into_iter().chain(VIEWER_KEYS.map(|k| ("AppSettings", k))) {
            assert_eq!(rows.all().filter(|(s, r)| *s == section && r.key == key).count(), 1, "{section}.{key}");
        }
        assert_eq!(rows.all().count(), CSHARP_KEYS.len() + VIEWER_KEYS.len(), "a row for a key neither a C# row nor the viewer sets");
        assert!(rows.all().filter(|(_, r)| r.group.as_deref() == Some("Media viewer")).all(|(_, r)| VIEWER_KEYS.contains(&r.key.as_str())));
        for (key, owner) in NOT_ROWS {
            assert!(rows.all().all(|(_, r)| r.key != key), "{key} is {owner}'s");
        }
    }

    #[test]
    fn each_key_is_its_models_and_each_default_the_models_initial_value() {
        let app = serde_json::to_value(AppSettings::default()).unwrap();
        let graphics = serde_json::to_value(GraphicsConfig::default()).unwrap();
        let rows = rows().unwrap();
        for (model, list) in [(&app, &rows.app_settings), (&graphics, &rows.graphics)] {
            for r in list {
                let initial = model_value(model, &r.key).unwrap_or_else(|| panic!("{} is no field of its model", r.key));
                assert_eq!(text(&initial), r.default, "{}", r.key);
            }
        }
        for r in &rows.console_graphics {
            assert!(graphics.get(&r.key).is_none(), "{} is a console's key, not a field", r.key);
        }
    }

    #[test]
    fn a_row_that_is_none_is_refused_with_words() {
        let one = |row: &str| format!(r#"{{"AppSettings": [{row}], "GraphicsConfig": [], "GraphicsConfig.Consoles": []}}"#);
        let row = r#"{"key": "HomeArt", "label": "Art", "hint": "Art.", "kind": "switch", "default": "true", "min": 0, "max": 0, "choices": [], "choice_labels": [], "group": "Display", "shown_with": null, "locked": false}"#;
        assert!(parse(&one(row)).is_ok());
        assert_eq!(parse(&one(&row.replace("\"switch\"", "\"slider\""))).unwrap_err(), "AppSettings.HomeArt: the kind slider is none of switch, count, choice");
        assert_eq!(parse(&one(&row.replace("\"true\"", "\"yes\""))).unwrap_err(), "AppSettings.HomeArt: a switch's default is true or false, not yes");
        let choice = row.replace("\"switch\"", "\"choice\"").replace("\"choices\": []", "\"choices\": [\"true\", \"false\"]");
        assert_eq!(parse(&one(&choice)).unwrap_err(), "AppSettings.HomeArt: each choice has its words");
        let worded = choice.replace("\"choice_labels\": []", "\"choice_labels\": [\"On\", \"Off\"]");
        assert!(parse(&one(&worded)).is_ok());
        assert_eq!(parse(&one(&worded.replace("\"default\": \"true\"", "\"default\": \"maybe\""))).unwrap_err(), "AppSettings.HomeArt: the default maybe is none of its choices");
        assert_eq!(parse(&one(&format!("{row}, {row}"))).unwrap_err(), "AppSettings.HomeArt: the key has a row already");
        assert!(parse(&one(&row.replace("\"locked\": false", "\"locked\": false, \"note\": null"))).unwrap_err().contains("unknown field `note`"));
    }
}
