//! The config models Galaxia owns the files of, as serde types: each field in its file's order, with its type and its
//! value in a new instance. What a setting means is its owner's (EmuSen_Config_Reference.md §3); how a file is read and
//! written is `config_json`'s (EmuSen_RustPlatform.md §47.19).

use crate::config_json;
use indexmap::IndexMap;
use serde::de::{self, DeserializeOwned, Deserializer};
use serde::{Deserialize, Serialize};

/// Text a file may leave null.
pub type Text = Option<String>;
/// A list a file may leave null.
pub type List<T> = Option<Vec<T>>;
/// Values by name, in the file's order; a file may leave the map null.
pub type Map<T> = Option<IndexMap<String, T>>;

fn text(value: &str) -> Text {
    Some(value.to_string())
}

fn texts(values: &[&str]) -> List<Text> {
    Some(values.iter().map(|value| text(value)).collect())
}

fn empty<T>() -> Map<T> {
    Some(IndexMap::new())
}

/// A float as a file may give it: a number too large for a float is refused, since it could not be written back.
fn single<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    let single = f64::deserialize(deserializer)? as f32;
    if single.is_finite() { Ok(single) } else { Err(de::Error::custom("a number too large for a float")) }
}

/// A model and its new instance from one list: each field's name, type and initial value, in the file's order.
macro_rules! model {
    ($(#[$meta:meta])* $name:ident { $($(#[$field_meta:meta])* $field:ident: $ty:ty = $initial:expr,)* }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "PascalCase", default)]
        pub struct $name {
            $($(#[$field_meta])* pub $field: $ty,)*
        }

        impl Default for $name {
            fn default() -> Self {
                $name { $($field: $initial,)* }
            }
        }
    };
}

model! {
    /// `BigPictureChoices`: one ES-DE theme's choices; null is Automatic or the theme's own.
    BigPictureChoices {
        variant: Text = None,
        color_scheme: Text = None,
        font_size: Text = None,
        aspect_ratio: Text = None,
        language: Text = None,
        transitions: Text = None,
    }
}

model! {
    /// `BigPictureCollections`.
    BigPictureCollections {
        auto_collections: List<Text> = texts(&["all", "favorites", "recent"]),
        hidden_custom_collections: List<i64> = Some(Vec::new()),
        group_custom_collections: Text = text("unthemed"),
        favorites_first: bool = true,
        favorites_first_custom: bool = false,
        stars_custom: bool = false,
        default_sort_order: Text = text("name, ascending"),
        random_entry_button: Text = text("games"),
        folders_on_top: bool = true,
        flattened_systems: List<Text> = Some(Vec::new()),
    }
}

model! {
    /// `BigPictureInterface`.
    BigPictureInterface {
        launch_screen_duration: Text = text("normal"),
        menu_opening_effect: Text = text("scale-up"),
        display_clock: bool = false,
        display_help: bool = true,
        status_bluetooth: bool = true,
        status_wifi: bool = true,
        status_battery: bool = true,
        status_battery_percentage: bool = true,
        quick_system_select: Text = text("leftrightshoulders"),
        startup_system: Text = text(""),
        startup_view: Text = text("system"),
        systems_sorting: Text = text("release"),
        list_scroll_overlay: bool = false,
        navigation_volume: i32 = 60,
        screensaver_timer: i32 = 300000,
        screensaver_type: Text = text("dim"),
        screensaver_controls: bool = true,
        screensaver_swap_image_timeout: i32 = 10000,
        screensaver_slideshow_only_favorites: bool = false,
        screensaver_stretch_images: bool = false,
        screensaver_slideshow_game_info: bool = true,
        screensaver_slideshow_custom_images: bool = false,
        screensaver_slideshow_recurse: bool = false,
        screensaver_slideshow_custom_dir: Text = text(""),
        screensaver_in_game_mode: bool = true,
    }
}

pub const ALL_CONSOLES: &str = "All consoles";
/// What `SelectedCore` was defaulted to while nothing read it.
pub const LEGACY_SELECTED_CORE_DEFAULT: &str = "SNES (Venus)";

model! {
    /// `AppSettings`, the file `appsettings.json`.
    AppSettings {
        log_directory: Text = None,
        rom_directory: Text = None,
        state_directory: Text = None,
        cheat_database_directory: Text = None,
        selected_core: Text = text(ALL_CONSOLES),
        selected_core_upgraded: bool = false,
        library_search: Text = text(""),
        cheat_search: Text = text(""),
        mirror_player1_to_player2: bool = false,
        keyboard_player: i32 = 1,
        analog_stick_as_dpad: bool = true,
        stick_deadzone: f64 = 0.5,
        controller_type: Text = text("Automatic"),
        swap_pad_buttons: bool = false,
        first_controller_only: bool = false,
        controller_notifications: bool = true,
        guide_chords: Text = text("Auto"),
        claim_players_by_press: bool = true,
        external_controller_first: bool = true,
        rumble: bool = true,
        on_screen_keyboard: Text = text("Automatic"),
        big_screen: bool = false,
        show_status_bar: bool = true,
        show_status_text: bool = true,
        show_fps_bar: bool = true,
        appearance: Text = text("dark"),
        home_art: bool = true,
        play_videos: bool = true,
        text_size: Text = text("default"),
        reduced_motion: Text = text("system"),
        startup_splash: bool = true,
        resume_on_launch: Text = text("Ask"),
        library_view: Text = text("Grid"),
        library_tile_scale: f64 = 1.0,
        library_collection: Text = text("all"),
        sound_output_device: Text = text("System default"),
        volume: f64 = 1.0,
        mute: bool = false,
        pause_in_background: bool = true,
        /// Retired by the failover; left out of the file while null.
        online_covers: Option<bool> = None,
        open_emu_fallback: bool = true,
        scraping: bool = true,
        scrape_covers: bool = true,
        scrape_screenshots: bool = true,
        scrape_marquees: bool = true,
        scrape_title_screens: bool = true,
        scrape_miximages: bool = true,
        scrape_region: Text = text("auto"),
        scrape_language: Text = text("en"),
        scrape_region_fallback: bool = true,
        scrape_threads: i32 = 0,
        scrape_back_covers: bool = true,
        #[serde(rename = "Scrape3DBoxes")]
        scrape_3d_boxes: bool = true,
        scrape_physical_media: bool = true,
        scrape_fan_art: bool = true,
        scrape_manuals: bool = false,
        scrape_videos: bool = false,
        scrape_game_names: bool = false,
        scrape_criteria: Text = text("nocover"),
        scrape_refresh: bool = false,
        scrape_kinds_upgraded: bool = false,
        artwork_directory: Text = None,
        firmware_directory: Text = None,
        library_style: Text = text("Theme"),
        big_picture_theme: Text = None,
        esde_media_directory: Text = None,
        emu_sen_theme: Text = None,
        chd_output_directory: Text = None,
        navigation_sounds: bool = true,
        max_play_time_tracking: i32 = 8,
        show_hidden_games: bool = false,
        big_picture: Map<Option<BigPictureChoices>> = empty(),
        big_picture_collections: Option<BigPictureCollections> = Some(BigPictureCollections::default()),
        big_picture_interface: Option<BigPictureInterface> = Some(BigPictureInterface::default()),
    }
}

model! {
    /// `AudioConfig`, the file `audio.json`.
    AudioConfig {
        sample_rate: i32 = 32000,
        audio_buffer_max_samples: i32 = 128000,
        output_target_latency_ms: i32 = 256,
        rate_control_max_deviation: f64 = 0.005,
        #[serde(deserialize_with = "single")]
        master_volume: f32 = 1.0,
        muted: bool = false,
        audio_enabled: bool = true,
    }
}

model! {
    /// `GraphicsConfig`, the file `graphics.json`.
    GraphicsConfig {
        window_width: i32 = 1060,
        window_height: i32 = 580,
        target_fps: i32 = 60,
        #[serde(rename = "VSyncEnabled")]
        vsync_enabled: bool = true,
        window_resizable: bool = true,
        bilinear_filtering: bool = true,
        sync_to_display: bool = true,
        consoles: Map<Map<Text>> = empty(),
        shader_parameters: Map<Map<Map<Text>>> = empty(),
        recent_shaders: Map<List<Text>> = empty(),
    }
}

model! {
    /// `ConsolesConfig`, the runtime's file `consoles.json`: each console's core settings by key (EmuSen_Backend.md §14.12).
    ConsolesConfig {
        consoles: Map<Map<Text>> = empty(),
    }
}

pub const CONSOLES_FILE: &str = "consoles.json";
pub const GRAPHICS_FILE: &str = "graphics.json";

model! {
    /// `CheatFileWrite`: one write inside a saved cheat, its addresses and values hex text.
    CheatFileWrite {
        space: Text = text(""),
        address: Text = text("0"),
        value: Text = text("00"),
        width: i32 = 1,
        r#type: Text = text("Set"),
        bit_position: Option<i32> = None,
        big_endian: bool = false,
        repeat_count: i32 = 1,
        repeat_add_address: Text = text("0"),
        repeat_add_value: Text = text("0"),
    }
}

model! {
    /// `CheatFileEntry`: one cheat as it is read; the file also holds the two values `Written` computes.
    CheatFileEntry {
        kind: Text = text("RamPoke"),
        writes: List<CheatFileWrite> = Some(Vec::new()),
        compare: Text = None,
        description: Text = text(""),
        enabled: bool = true,
        space: Text = None,
        address: Text = None,
        value: Text = None,
    }
}

impl CheatFileEntry {
    /// `IsRomPatch`.
    pub fn is_rom_patch(&self) -> bool {
        self.kind.as_deref().is_some_and(|kind| kind.eq_ignore_ascii_case("RomPatch"))
    }

    /// `EffectiveWrites`, as the file holds it: a cheat with no list of writes cannot be written.
    pub fn effective_writes(&self) -> Result<Vec<CheatFileWrite>, String> {
        self.writes.as_ref().ok_or("A cheat has no list of writes.")?;
        Ok(self.listed_writes())
    }

    /// The writes as written, or the one write a file from before several could be listed describes.
    pub fn listed_writes(&self) -> Vec<CheatFileWrite> {
        if let Some(writes) = self.writes.as_ref().filter(|writes| !writes.is_empty()) {
            return writes.clone();
        }
        let Some(address) = &self.address else { return Vec::new() };
        let space = self.space.clone().or_else(|| text(""));
        let value = self.value.clone().or_else(|| text("00"));
        vec![CheatFileWrite { space, address: Some(address.clone()), value, ..CheatFileWrite::default() }]
    }
}

model! {
    /// `CheatFile`, a file of the `cheats` category.
    CheatFile {
        cheats: List<CheatFileEntry> = Some(Vec::new()),
    }
}

/// A cheat as its file holds it: the entry, then the two values computed from it, written and never read.
#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct WrittenEntry<'a> {
    #[serde(flatten)]
    entry: &'a CheatFileEntry,
    is_rom_patch: bool,
    effective_writes: Vec<CheatFileWrite>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct WrittenFile<'a> {
    cheats: Option<Vec<WrittenEntry<'a>>>,
}

/// A type with a file of its own: what a load does to an older file's values, and the file's text.
pub trait ConfigModel: Serialize + DeserializeOwned + Default {
    /// The change a load makes to an older file's values, where there is one.
    fn upgrade(&mut self) {}

    /// The text of the file that holds the model.
    fn file_text(&self) -> Result<String, String> {
        config_json::write(self)
    }
}

impl ConfigModel for AudioConfig {}
impl ConfigModel for GraphicsConfig {}
impl ConfigModel for ConsolesConfig {}

impl ConfigModel for CheatFile {
    fn file_text(&self) -> Result<String, String> {
        fn written(entry: &CheatFileEntry) -> Result<WrittenEntry<'_>, String> {
            Ok(WrittenEntry { entry, is_rom_patch: entry.is_rom_patch(), effective_writes: entry.effective_writes()? })
        }
        let cheats = self.cheats.as_ref().map(|cheats| cheats.iter().map(written).collect::<Result<Vec<_>, _>>()).transpose()?;
        config_json::write(&WrittenFile { cheats })
    }
}

/// The nine picture kinds as the old default stored them: four on, five off.
const OLD_SCRAPE_KINDS: [bool; 9] = [true, true, true, true, false, false, false, false, false];

impl AppSettings {
    fn scrape_kinds(&mut self) -> [&mut bool; 9] {
        [
            &mut self.scrape_covers,
            &mut self.scrape_screenshots,
            &mut self.scrape_marquees,
            &mut self.scrape_miximages,
            &mut self.scrape_title_screens,
            &mut self.scrape_back_covers,
            &mut self.scrape_3d_boxes,
            &mut self.scrape_physical_media,
            &mut self.scrape_fan_art,
        ]
    }
}

impl ConfigModel for AppSettings {
    fn file_text(&self) -> Result<String, String> {
        let mut file = serde_json::to_value(self).map_err(|error| error.to_string())?;
        if let Some(members) = file.as_object_mut().filter(|m| m.get("OnlineCovers").is_some_and(serde_json::Value::is_null)) {
            members.shift_remove("OnlineCovers");
        }
        config_json::write(&file)
    }

    /// `AppSettings.Upgraded`: a stored legacy default is not a choice, and the key the failover replaced goes. See EmuSen_Multicore.md §10.3.
    fn upgrade(&mut self) {
        if !self.selected_core_upgraded && self.selected_core.as_deref() == Some(LEGACY_SELECTED_CORE_DEFAULT) {
            self.selected_core = text(ALL_CONSOLES);
        }
        self.selected_core_upgraded = true;
        self.online_covers = None;
        // The old default of four kinds, stored by every save, is no choice: it becomes every picture a theme can draw (decided 2026-11-02 by the tester).
        if !self.scrape_kinds_upgraded {
            let mut kinds = self.scrape_kinds();
            if kinds.iter().zip(OLD_SCRAPE_KINDS).all(|(kind, was)| **kind == was) {
                kinds.iter_mut().for_each(|kind| **kind = true);
            }
            self.scrape_kinds_upgraded = true;
        }
    }
}

/// A model as the C interface and the corpus reach it, by number, each call on JSON text. Out of the library a model is
/// its transport: compact JSON with every field, which the host binds again.
pub struct Model {
    pub name: &'static str,
    /// Text read as the model, upgraded when asked: its transport, or none for a document that is `null`.
    pub bind: fn(&str, bool) -> Result<Option<String>, String>,
    /// A new instance's transport, upgraded when asked.
    pub new: fn(bool) -> String,
    /// A document given as JSON, as the text of its file; `null` for a document that is null.
    pub format: fn(&str) -> Result<String, String>,
}

impl Model {
    const fn of<M: ConfigModel>(name: &'static str) -> Model {
        Model { name, bind: bind::<M>, new: new::<M>, format: format::<M> }
    }
}

impl std::fmt::Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

/// A model as compact JSON, every field present.
pub fn transport<M: Serialize>(model: &M) -> String {
    serde_json::to_string(model).expect("a model is JSON")
}

fn bind<M: ConfigModel>(text: &str, upgrade: bool) -> Result<Option<String>, String> {
    let read: Option<M> = config_json::read(text)?;
    Ok(read.map(|mut model| {
        if upgrade {
            model.upgrade();
        }
        transport(&model)
    }))
}

fn new<M: ConfigModel>(upgrade: bool) -> String {
    let mut model = M::default();
    if upgrade {
        model.upgrade();
    }
    transport(&model)
}

fn format<M: ConfigModel>(document: &str) -> Result<String, String> {
    match config_json::read::<Option<M>>(document)? {
        Some(model) => model.file_text(),
        // Settings that are null are written as that word.
        None => Ok("null".to_string()),
    }
}

/// The models, numbered as `EMUSEN_GALAXIA_MODEL_*`.
pub static MODELS: [Model; 4] =
    [Model::of::<AppSettings>("AppSettings"), Model::of::<AudioConfig>("AudioConfig"), Model::of::<GraphicsConfig>("GraphicsConfig"), Model::of::<CheatFile>("CheatFile")];

/// The model of `consoles.json`, which a host binds itself and so has no `EMUSEN_GALAXIA_MODEL_*` number.
pub static CONSOLES_MODEL: Model = Model::of::<ConsolesConfig>("ConsolesConfig");

pub const APP_SETTINGS_ID: u32 = 0;
pub const AUDIO_CONFIG_ID: u32 = 1;
pub const GRAPHICS_CONFIG_ID: u32 = 2;
pub const CHEAT_FILE_ID: u32 = 3;

pub const APP_SETTINGS_FILE: &str = "appsettings.json";
pub const CHEATS_CATEGORY: &str = "cheats";

pub fn model(id: u32) -> Option<&'static Model> {
    MODELS.get(id as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read<M: ConfigModel>(text: &str) -> M {
        config_json::read(text).unwrap()
    }

    #[test]
    fn a_new_audio_config_is_written_in_serdes_spelling() {
        let expected = "{\n  \"SampleRate\": 32000,\n  \"AudioBufferMaxSamples\": 128000,\n  \"OutputTargetLatencyMs\": 256,\n  \"RateControlMaxDeviation\": 0.005,\n  \"MasterVolume\": 1.0,\n  \"Muted\": false,\n  \"AudioEnabled\": true\n}";
        assert_eq!(AudioConfig::default().file_text().unwrap(), expected);
    }

    #[test]
    fn a_volume_is_written_at_a_floats_width() {
        let config: AudioConfig = read(r#"{"MasterVolume":0.800000011920929,"RateControlMaxDeviation":0.800000011920929}"#);
        let text = config.file_text().unwrap();
        assert!(text.contains("\"MasterVolume\": 0.8,") && text.contains("\"RateControlMaxDeviation\": 0.800000011920929,"), "{text}");
        assert_eq!(config_json::read::<AudioConfig>(r#"{"MasterVolume":1e39}"#).unwrap_err(), "$.MasterVolume: a number too large for a float.");
        assert_eq!(config_json::read::<AudioConfig>(r#"{"SampleRate":5}"#).unwrap().master_volume, 1.0);
    }

    #[test]
    fn app_settings_has_every_property_and_skips_the_retired_key_while_it_is_null() {
        let text = AppSettings::default().file_text().unwrap();
        let names: Vec<String> = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&text).unwrap().keys().cloned().collect();
        assert_eq!(names.len(), 72);
        assert!(names.iter().any(|n| n == "Scrape3DBoxes") && names.iter().any(|n| n == "MirrorPlayer1ToPlayer2"));
        assert!(text.contains("\"SelectedCore\": \"All consoles\",") && text.contains("\"BigPicture\": {},") && !text.contains("OnlineCovers"));
        let old: AppSettings = read(r#"{"OnlineCovers":false}"#);
        assert!(old.file_text().unwrap().contains("\"PauseInBackground\": true,\n  \"OnlineCovers\": false,\n"));
    }

    #[test]
    fn the_old_four_kinds_become_the_nine_and_a_list_the_player_changed_is_kept() {
        let nine = |s: &mut AppSettings| s.scrape_kinds().iter().filter(|k| ***k).count();
        let old = r#"{"ScrapeCovers":true,"ScrapeScreenshots":SHOTS,"ScrapeMarquees":true,"ScrapeMiximages":true,"ScrapeTitleScreens":false,"ScrapeBackCovers":false,"Scrape3DBoxes":false,"ScrapePhysicalMedia":false,"ScrapeFanArt":false}"#;
        let mut fresh: AppSettings = read("{}");
        fresh.upgrade();
        assert_eq!(nine(&mut fresh), 9);
        let mut four: AppSettings = read(&old.replace("SHOTS", "true"));
        four.upgrade();
        assert_eq!((nine(&mut four), four.scrape_kinds_upgraded), (9, true));
        let mut changed: AppSettings = read(&old.replace("SHOTS", "false"));
        changed.upgrade();
        assert_eq!(nine(&mut changed), 3);
        let mut chosen_since: AppSettings = read(&old.replace("SHOTS", "true").replace('{', "{\"ScrapeKindsUpgraded\":true,"));
        chosen_since.upgrade();
        assert_eq!(nine(&mut chosen_since), 4);
    }

    #[test]
    fn a_stored_legacy_default_is_upgraded_once_and_a_choice_made_since_is_kept() {
        let mut old: AppSettings = read(r#"{"SelectedCore":"SNES (Venus)","OnlineCovers":false}"#);
        old.upgrade();
        assert_eq!((old.selected_core.as_deref(), old.selected_core_upgraded, old.online_covers), (Some(ALL_CONSOLES), true, None));
        let mut chosen: AppSettings = read(r#"{"SelectedCore":"SNES (Venus)","SelectedCoreUpgraded":true}"#);
        chosen.upgrade();
        assert_eq!(chosen.selected_core.as_deref(), Some(LEGACY_SELECTED_CORE_DEFAULT));
        let mut absent: AppSettings = read(r#"{"SelectedCore":null}"#);
        absent.upgrade();
        assert_eq!(absent.selected_core, None);
    }

    #[test]
    fn a_cheat_is_written_with_the_two_values_computed_and_they_are_never_read() {
        let file: CheatFile = read(r#"{"Cheats":[{"Kind":"rompatch","Writes":[{"Address":"7E0DBF","Value":"63"}],"IsRomPatch":"ignored"}]}"#);
        let text = file.file_text().unwrap();
        assert!(text.contains("\"IsRomPatch\": true,"), "{text}");
        assert_eq!(text.matches("\"Address\": \"7E0DBF\"").count(), 2);
        assert!(!transport(&file).contains("IsRomPatch"));
        assert!(transport(&AppSettings::default()).contains("\"OnlineCovers\":null"));
        let broken: CheatFile = read(r#"{"Cheats":[{"Writes":null}]}"#);
        assert!(broken.file_text().is_err());
    }

    #[test]
    fn a_cheat_from_before_several_writes_has_one_effective_write() {
        let file: CheatFile = read(r#"{"Cheats":[{"Space":"CpuBus","Address":"7E0019"},{"Value":"01"},{"Address":"1","Space":null,"Value":null}]}"#);
        let cheats = file.cheats.clone().unwrap();
        let effective = |i: usize| transport(&cheats[i].effective_writes().unwrap());
        assert_eq!(effective(0), r#"[{"Space":"CpuBus","Address":"7E0019","Value":"00","Width":1,"Type":"Set","BitPosition":null,"BigEndian":false,"RepeatCount":1,"RepeatAddAddress":"0","RepeatAddValue":"0"}]"#);
        assert_eq!(effective(1), "[]");
        assert!(effective(2).starts_with(r#"[{"Space":"","Address":"1","Value":"00","#));
        let written: serde_json::Value = serde_json::from_str(&file.file_text().unwrap()).unwrap();
        assert_eq!(written["Cheats"][0]["EffectiveWrites"][0]["Address"], "7E0019", "the file holds the write an old cheat describes");
        assert_eq!(written["Cheats"][0]["Writes"], serde_json::json!([]));
    }

    #[test]
    fn graphics_settings_nest_three_deep() {
        let config: GraphicsConfig = read(r#"{"ShaderParameters":{"SNES":{"crt":{"gain":"0.5"}}},"RecentShaders":{"SNES":["a",null]},"Consoles":{"N64":null}}"#);
        let text = config.file_text().unwrap();
        assert!(text.contains("\"gain\": \"0.5\"") && text.contains("\"N64\": null"));
        assert!(config_json::read::<GraphicsConfig>(r#"{"Consoles":{"N64":{"k":1}}}"#).is_err());
    }

    #[test]
    fn an_unknown_member_reads_as_if_absent_and_is_not_written_back_as_before() {
        let with: AppSettings = read(r#"{"Retired": {"x": [1, 2, {"deep": null}]}, "AlsoRetired": "é", "KeyboardPlayer": 2}"#);
        assert_eq!(with, read::<AppSettings>(r#"{"KeyboardPlayer": 2}"#));
        assert!(!with.file_text().unwrap().contains("Retired"));
        let cheats: CheatFile = read(r#"{"Cheats": [{"Description": "x", "Future": 1}], "Version": 2}"#);
        assert_eq!(cheats, read::<CheatFile>(r#"{"Cheats": [{"Description": "x"}]}"#));
    }

    #[test]
    fn each_model_by_number_binds_formats_and_makes_a_new_instance() {
        let app = &MODELS[APP_SETTINGS_ID as usize];
        assert!((app.new)(true).contains("\"SelectedCoreUpgraded\":true"));
        assert_eq!((app.bind)("null", true), Ok(None));
        assert!((app.bind)("[]", false).is_err());
        assert_eq!((MODELS[AUDIO_CONFIG_ID as usize].format)(" null "), Ok("null".to_string()));
    }
}
