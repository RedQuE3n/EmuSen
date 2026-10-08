//! The config models Galaxia owns the files of, each a schema mirroring a C# class of `EmuSen.Galaxia.Models`.
//!
//! A field is listed in the order C# declares its property, which is the order of the file. What a setting means is
//! its owner's (EmuSen_Config_Reference.md §3); what is here is its name, its type and its value in a new instance.

use crate::dotnet_path::equals_ignore_case;
use crate::json::{Text, Value};
use crate::model::{Field, Initial, Schema, Type};

const fn text(name: &'static str, initial: &'static str) -> Field {
    Field::new(name, Type::String, Initial::Str(initial))
}

const fn no_text(name: &'static str) -> Field {
    Field::new(name, Type::String, Initial::Null)
}

const fn flag(name: &'static str, initial: bool) -> Field {
    Field::new(name, Type::Bool, Initial::Bool(initial))
}

const fn int(name: &'static str, initial: &'static str) -> Field {
    Field::new(name, Type::Int32, Initial::Number(initial))
}

const fn double(name: &'static str, initial: &'static str) -> Field {
    Field::new(name, Type::Double, Initial::Number(initial))
}

static STRING: Type = Type::String;
static LONG: Type = Type::Int64;
static STRINGS: Type = Type::List(&STRING);
static STRING_MAP: Type = Type::Map(&STRING);
static STRING_MAP_MAP: Type = Type::Map(&STRING_MAP);

/// `BigPictureChoices`: one ES-DE theme's choices; null is Automatic or the theme's own.
pub static BIG_PICTURE_CHOICES: Schema = Schema {
    name: "BigPictureChoices",
    fields: &[no_text("Variant"), no_text("ColorScheme"), no_text("FontSize"), no_text("AspectRatio"), no_text("Language"), no_text("Transitions")],
};
static BIG_PICTURE_CHOICES_TYPE: Type = Type::Object(&BIG_PICTURE_CHOICES);

/// `BigPictureCollections`.
pub static BIG_PICTURE_COLLECTIONS: Schema = Schema {
    name: "BigPictureCollections",
    fields: &[
        Field::new("AutoCollections", Type::List(&STRING), Initial::Strings(&["all", "favorites", "recent"])),
        Field::new("HiddenCustomCollections", Type::List(&LONG), Initial::Strings(&[])),
        text("GroupCustomCollections", "unthemed"),
        flag("FavoritesFirst", true),
        flag("FavoritesFirstCustom", false),
        flag("StarsCustom", false),
        text("DefaultSortOrder", "name, ascending"),
        text("RandomEntryButton", "games"),
        flag("FoldersOnTop", true),
        Field::new("FlattenedSystems", Type::List(&STRING), Initial::Strings(&[])),
    ],
};

/// `BigPictureInterface`.
pub static BIG_PICTURE_INTERFACE: Schema = Schema {
    name: "BigPictureInterface",
    fields: &[
        text("LaunchScreenDuration", "normal"),
        text("MenuOpeningEffect", "scale-up"),
        flag("DisplayClock", false),
        flag("DisplayHelp", true),
        flag("StatusBluetooth", true),
        flag("StatusWifi", true),
        flag("StatusBattery", true),
        flag("StatusBatteryPercentage", true),
        text("QuickSystemSelect", "leftrightshoulders"),
        text("StartupSystem", ""),
        text("StartupView", "system"),
        text("SystemsSorting", "release"),
        flag("ListScrollOverlay", false),
        int("NavigationVolume", "70"),
        int("ScreensaverTimer", "300000"),
        text("ScreensaverType", "dim"),
        flag("ScreensaverControls", true),
        int("ScreensaverSwapImageTimeout", "10000"),
        flag("ScreensaverSlideshowOnlyFavorites", false),
        flag("ScreensaverStretchImages", false),
        flag("ScreensaverSlideshowGameInfo", true),
        flag("ScreensaverSlideshowCustomImages", false),
        flag("ScreensaverSlideshowRecurse", false),
        text("ScreensaverSlideshowCustomDir", ""),
        flag("ScreensaverInGameMode", true),
    ],
};

pub const ALL_CONSOLES: &str = "All consoles";
/// What `SelectedCore` was defaulted to while nothing read it.
pub const LEGACY_SELECTED_CORE_DEFAULT: &str = "SNES (Venus)";

/// `AppSettings`, the file `appsettings.json`.
pub static APP_SETTINGS: Schema = Schema {
    name: "AppSettings",
    fields: &[
        no_text("LogDirectory"),
        no_text("RomDirectory"),
        no_text("StateDirectory"),
        no_text("CheatDatabaseDirectory"),
        text("SelectedCore", ALL_CONSOLES),
        flag("SelectedCoreUpgraded", false),
        text("LibrarySearch", ""),
        text("CheatSearch", ""),
        flag("MirrorPlayer1ToPlayer2", false),
        int("KeyboardPlayer", "1"),
        flag("AnalogStickAsDpad", true),
        double("StickDeadzone", "0.5"),
        text("ControllerType", "Automatic"),
        flag("SwapPadButtons", false),
        flag("FirstControllerOnly", false),
        flag("ControllerNotifications", true),
        text("OnScreenKeyboard", "Automatic"),
        flag("BigScreen", false),
        flag("ShowStatusBar", true),
        flag("ShowStatusText", true),
        flag("ShowFpsBar", true),
        text("ResumeOnLaunch", "Ask"),
        text("LibraryView", "Grid"),
        double("LibraryTileScale", "1"),
        text("LibraryCollection", "all"),
        double("Volume", "1"),
        flag("PauseInBackground", true),
        Field::new("OnlineCovers", Type::NullableBool, Initial::Null).skipped_when_null(),
        flag("OpenEmuFallback", true),
        flag("Scraping", true),
        flag("ScrapeCovers", true),
        flag("ScrapeScreenshots", true),
        flag("ScrapeMarquees", true),
        flag("ScrapeTitleScreens", false),
        flag("ScrapeMiximages", true),
        text("ScrapeRegion", "auto"),
        text("ScrapeLanguage", "en"),
        flag("ScrapeRegionFallback", true),
        int("ScrapeThreads", "1"),
        flag("ScrapeBackCovers", false),
        flag("Scrape3DBoxes", false),
        flag("ScrapePhysicalMedia", false),
        flag("ScrapeFanArt", false),
        flag("ScrapeManuals", false),
        flag("ScrapeVideos", false),
        flag("ScrapeGameNames", false),
        text("ScrapeCriteria", "nocover"),
        flag("ScrapeRefresh", false),
        no_text("ArtworkDirectory"),
        text("LibraryStyle", "Theme"),
        no_text("BigPictureTheme"),
        no_text("EsdeMediaDirectory"),
        flag("NavigationSounds", true),
        int("MaxPlayTimeTracking", "8"),
        flag("ShowHiddenGames", false),
        Field::new("BigPicture", Type::Map(&BIG_PICTURE_CHOICES_TYPE), Initial::EmptyMap),
        Field::new("BigPictureCollections", Type::Object(&BIG_PICTURE_COLLECTIONS), Initial::New),
        Field::new("BigPictureInterface", Type::Object(&BIG_PICTURE_INTERFACE), Initial::New),
    ],
};

/// `AudioConfig`, the file `audio.json`.
pub static AUDIO_CONFIG: Schema = Schema {
    name: "AudioConfig",
    fields: &[
        int("SampleRate", "32000"),
        int("AudioBufferMaxSamples", "128000"),
        int("OutputTargetLatencyMs", "256"),
        double("RateControlMaxDeviation", "0.005"),
        Field::new("MasterVolume", Type::Single, Initial::Number("1")),
        flag("Muted", false),
        flag("AudioEnabled", true),
    ],
};

/// `GraphicsConfig`, the file `graphics.json`.
pub static GRAPHICS_CONFIG: Schema = Schema {
    name: "GraphicsConfig",
    fields: &[
        int("WindowWidth", "1060"),
        int("WindowHeight", "580"),
        int("TargetFps", "60"),
        flag("VSyncEnabled", true),
        flag("WindowResizable", true),
        flag("BilinearFiltering", true),
        flag("SyncToDisplay", true),
        Field::new("Consoles", Type::Map(&STRING_MAP), Initial::EmptyMap),
        Field::new("ShaderParameters", Type::Map(&STRING_MAP_MAP), Initial::EmptyMap),
        Field::new("RecentShaders", Type::Map(&STRINGS), Initial::EmptyMap),
    ],
};

/// `CheatFileWrite`: one write inside a saved cheat, its addresses and values hex text.
pub static CHEAT_FILE_WRITE: Schema = Schema {
    name: "CheatFileWrite",
    fields: &[
        text("Space", ""),
        text("Address", "0"),
        text("Value", "00"),
        int("Width", "1"),
        text("Type", "Set"),
        Field::new("BitPosition", Type::NullableInt32, Initial::Null),
        flag("BigEndian", false),
        int("RepeatCount", "1"),
        text("RepeatAddAddress", "0"),
        text("RepeatAddValue", "0"),
    ],
};
static CHEAT_FILE_WRITE_TYPE: Type = Type::Object(&CHEAT_FILE_WRITE);

/// `CheatFileEntry.IsRomPatch`.
fn is_rom_patch(entry: &Value) -> Result<Value, String> {
    Ok(Value::Bool(entry.get("Kind").and_then(Value::as_str).is_some_and(|kind| equals_ignore_case(kind, "RomPatch"))))
}

/// `CheatFileEntry.EffectiveWrites`: the writes as written, or the one write a file from before several could be listed describes.
fn effective_writes(entry: &Value) -> Result<Value, String> {
    match entry.get("Writes") {
        Some(Value::Array(writes)) if !writes.is_empty() => return Ok(Value::Array(writes.clone())),
        Some(Value::Array(_)) => {}
        // C# reads the count of a list that is not there, and the save fails.
        _ => return Err("A cheat has no list of writes.".to_string()),
    }
    let Some(address @ Value::String(_)) = entry.get("Address") else {
        return Ok(Value::Array(Vec::new()));
    };
    let or = |name: &str, fallback: &str| match entry.get(name) {
        Some(value @ Value::String(_)) => value.clone(),
        _ => Value::string(fallback),
    };
    let Value::Object(mut write) = CHEAT_FILE_WRITE.new_instance() else { unreachable!("a new instance is an object") };
    for (name, value) in [("Space", or("Space", "")), ("Address", address.clone()), ("Value", or("Value", "00"))] {
        write.iter_mut().find(|(key, _)| key.text == name).expect("the write has the field").1 = value;
    }
    Ok(Value::Array(vec![Value::Object(write)]))
}

/// `CheatFileEntry`: one cheat as it appears on disk, with the two values C# computes and writes beside it.
pub static CHEAT_FILE_ENTRY: Schema = Schema {
    name: "CheatFileEntry",
    fields: &[
        text("Kind", "RamPoke"),
        Field::new("Writes", Type::List(&CHEAT_FILE_WRITE_TYPE), Initial::Strings(&[])),
        no_text("Compare"),
        text("Description", ""),
        flag("Enabled", true),
        no_text("Space"),
        no_text("Address"),
        no_text("Value"),
        Field::derived("IsRomPatch", Type::Bool, is_rom_patch),
        Field::derived("EffectiveWrites", Type::List(&CHEAT_FILE_WRITE_TYPE), effective_writes),
    ],
};
static CHEAT_FILE_ENTRY_TYPE: Type = Type::Object(&CHEAT_FILE_ENTRY);

/// `CheatFile`, a file of the `cheats` category.
pub static CHEAT_FILE: Schema = Schema { name: "CheatFile", fields: &[Field::new("Cheats", Type::List(&CHEAT_FILE_ENTRY_TYPE), Initial::Strings(&[]))] };

/// A model with a file of its own: its schema, and what is done to a document after it is read.
#[derive(Debug)]
pub struct Model {
    pub schema: &'static Schema,
    /// The change a load makes to an older file's values, where C# makes one.
    pub upgrade: Option<fn(&mut Value)>,
}

/// The models, numbered as `EMUSEN_GALAXIA_MODEL_*`.
pub static MODELS: [Model; 4] = [
    Model { schema: &APP_SETTINGS, upgrade: Some(upgrade_app_settings) },
    Model { schema: &AUDIO_CONFIG, upgrade: None },
    Model { schema: &GRAPHICS_CONFIG, upgrade: None },
    Model { schema: &CHEAT_FILE, upgrade: None },
];

pub const APP_SETTINGS_ID: u32 = 0;
pub const AUDIO_CONFIG_ID: u32 = 1;
pub const GRAPHICS_CONFIG_ID: u32 = 2;
pub const CHEAT_FILE_ID: u32 = 3;

pub const APP_SETTINGS_FILE: &str = "appsettings.json";
pub const CHEATS_CATEGORY: &str = "cheats";

pub fn model(id: u32) -> Option<&'static Model> {
    MODELS.get(id as usize)
}

/// Sets a field of a bound object.
pub fn set(object: &mut Value, name: &str, value: Value) {
    if let Value::Object(members) = object {
        match members.iter_mut().find(|(key, _)| key.text == name) {
            Some(slot) => slot.1 = value,
            None => members.push((Text::new(name), value)),
        }
    }
}

/// `AppSettings.Upgraded`: a stored legacy default is not a choice, and the key the failover replaced goes. See EmuSen_Multicore.md §10.3.
fn upgrade_app_settings(settings: &mut Value) {
    let upgraded = settings.get("SelectedCoreUpgraded").and_then(Value::as_bool).unwrap_or(false);
    if !upgraded && settings.get("SelectedCore").and_then(Value::as_str) == Some(LEGACY_SELECTED_CORE_DEFAULT) {
        set(settings, "SelectedCore", Value::string(ALL_CONSOLES));
    }
    set(settings, "SelectedCoreUpgraded", Value::Bool(true));
    set(settings, "OnlineCovers", Value::Null);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json;
    use crate::model::{format, transport};

    fn read(schema: &'static Schema, text: &str) -> Value {
        schema.bind(&json::parse(text).unwrap()).unwrap().unwrap()
    }

    #[test]
    fn a_new_audio_config_is_the_file_csharp_writes() {
        let expected = "{\n  \"SampleRate\": 32000,\n  \"AudioBufferMaxSamples\": 128000,\n  \"OutputTargetLatencyMs\": 256,\n  \"RateControlMaxDeviation\": 0.005,\n  \"MasterVolume\": 1,\n  \"Muted\": false,\n  \"AudioEnabled\": true\n}";
        assert_eq!(format(&AUDIO_CONFIG, &AUDIO_CONFIG.new_instance()).unwrap(), expected.replace('\n', crate::model::NEW_LINE));
    }

    #[test]
    fn a_volume_is_written_at_a_floats_width() {
        let config = read(&AUDIO_CONFIG, r#"{"MasterVolume":0.8,"RateControlMaxDeviation":0.8}"#);
        let text = format(&AUDIO_CONFIG, &config).unwrap();
        assert!(text.contains("\"MasterVolume\": 0.8,") && text.contains("\"RateControlMaxDeviation\": 0.8,"));
        let config = read(&AUDIO_CONFIG, r#"{"MasterVolume":0.800000011920929,"RateControlMaxDeviation":0.800000011920929}"#);
        let text = format(&AUDIO_CONFIG, &config).unwrap();
        assert!(text.contains("\"MasterVolume\": 0.8,") && text.contains("\"RateControlMaxDeviation\": 0.800000011920929,"));
    }

    #[test]
    fn app_settings_has_every_property_and_skips_the_retired_key_while_it_is_null() {
        let new = APP_SETTINGS.new_instance();
        let text = format(&APP_SETTINGS, &new).unwrap();
        assert_eq!(APP_SETTINGS.fields.len(), 58);
        assert!(text.contains("\"SelectedCore\": \"All consoles\","));
        assert!(text.contains("\"BigPicture\": {},"));
        assert!(text.contains("\"NavigationVolume\": 70,"));
        assert!(!text.contains("OnlineCovers"));
        let old = read(&APP_SETTINGS, r#"{"OnlineCovers":false}"#);
        assert!(format(&APP_SETTINGS, &old).unwrap().contains("\"PauseInBackground\": true,\n  \"OnlineCovers\": false,\n".replace('\n', crate::model::NEW_LINE).as_str()));
    }

    #[test]
    fn a_stored_legacy_default_is_upgraded_once_and_a_choice_made_since_is_kept() {
        let upgrade = MODELS[APP_SETTINGS_ID as usize].upgrade.unwrap();
        let mut old = read(&APP_SETTINGS, r#"{"SelectedCore":"SNES (Venus)","OnlineCovers":false}"#);
        upgrade(&mut old);
        assert_eq!(old.get("SelectedCore").and_then(Value::as_str), Some(ALL_CONSOLES));
        assert_eq!(old.get("SelectedCoreUpgraded"), Some(&Value::Bool(true)));
        assert_eq!(old.get("OnlineCovers"), Some(&Value::Null));
        let mut chosen = read(&APP_SETTINGS, r#"{"SelectedCore":"SNES (Venus)","SelectedCoreUpgraded":true}"#);
        upgrade(&mut chosen);
        assert_eq!(chosen.get("SelectedCore").and_then(Value::as_str), Some(LEGACY_SELECTED_CORE_DEFAULT));
        let mut absent = read(&APP_SETTINGS, r#"{"SelectedCore":null}"#);
        upgrade(&mut absent);
        assert_eq!(absent.get("SelectedCore"), Some(&Value::Null));
    }

    #[test]
    fn a_cheat_is_written_with_the_two_values_csharp_computes() {
        let file = read(&CHEAT_FILE, r#"{"Cheats":[{"Kind":"rompatch","Writes":[{"Address":"7E0DBF","Value":"63"}],"IsRomPatch":"ignored"}]}"#);
        let text = format(&CHEAT_FILE, &file).unwrap();
        assert!(text.contains("\"IsRomPatch\": true,"));
        assert_eq!(text.matches("\"Address\": \"7E0DBF\"").count(), 2);
        assert!(!transport(&file).contains("IsRomPatch"));
    }

    #[test]
    fn a_cheat_from_before_several_writes_has_one_effective_write() {
        let file = read(&CHEAT_FILE, r#"{"Cheats":[{"Space":"CpuBus","Address":"7E0019"},{"Value":"01"},{"Address":"1","Space":null,"Value":null}]}"#);
        let Value::Array(cheats) = file.get("Cheats").unwrap().clone() else { panic!() };
        let effective = |i: usize| transport(&effective_writes(&cheats[i]).unwrap());
        assert_eq!(effective(0), r#"[{"Space":"CpuBus","Address":"7E0019","Value":"00","Width":1,"Type":"Set","BitPosition":null,"BigEndian":false,"RepeatCount":1,"RepeatAddAddress":"0","RepeatAddValue":"0"}]"#);
        assert_eq!(effective(1), "[]");
        assert!(effective(2).starts_with(r#"[{"Space":"","Address":"1","Value":"00","#));
        let broken = read(&CHEAT_FILE, r#"{"Cheats":[{"Writes":null}]}"#);
        assert!(format(&CHEAT_FILE, &broken).is_err());
    }

    #[test]
    fn graphics_settings_nest_three_deep() {
        let config = read(&GRAPHICS_CONFIG, r#"{"ShaderParameters":{"SNES":{"crt":{"gain":"0.5"}}},"RecentShaders":{"SNES":["a",null]},"Consoles":{"N64":null}}"#);
        let text = format(&GRAPHICS_CONFIG, &config).unwrap();
        assert!(text.contains("\"gain\": \"0.5\""));
        assert!(text.contains("\"N64\": null"));
        assert!(GRAPHICS_CONFIG.bind(&json::parse(r#"{"Consoles":{"N64":{"k":1}}}"#).unwrap()).is_err());
    }
}
