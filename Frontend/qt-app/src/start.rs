//! What the program reads before its window opens: the platform library's path and the player's settings that shape the
//! start, as plain values handed to the window's root as its initial properties (EmuSen_Qt.md §1.2.1: no static carries them).

use cxx_qt_lib::{QMap, QMapPair_QString_QVariant, QString, QStringList, QVariant};
use emusen_platform_client::galaxia::consts::{DIR_MEDIA, DIR_THEMES};
use emusen_qt_models::{esde_shelves, themes};
use emusen_platform_client::Platform;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// The platform library's file name, found beside the program when nothing names another.
pub const LIBRARY_FILE: &str = "libemusen_platform.so";

/// The start's facts, each an initial property of the window.
#[derive(Clone, Debug, PartialEq)]
pub struct Start {
    pub platform_library: String,
    /// Moonrise at the start: AppSettings.StartupSplash, on unless the player turned it off (EmuSen_Interface.md §22).
    pub splash: bool,
    /// AppSettings.ReducedMotion: "on" reduces it; "off" and the desktop's own preference ("system") do not yet (EmuSen_Qt.md §7.1).
    pub reduced_motion: bool,
    /// Steam's Game Mode holds the session, so Power holds Quit alone (Mistress's `InGameModeSession`).
    pub game_mode: bool,
    /// Guide held with Up, Down or the right shoulder over a game (AppSettings.GuideChords, EmuSen_BigPicture.md §48.7).
    pub guide_chords: bool,
    /// The EmuSen theme's folder: `--theme DIR`, else `EMUSEN_THEME`, else the settings' id resolved through the themes store;
    /// empty for the reference (EmuSen_Qt.md §3.5, §3.8).
    pub theme: String,
    /// Why the reference is shown in place of the id the settings hold; empty when it is not.
    pub theme_words: String,
    /// The folders a theme may show pictures from: the media store, the artwork and ES-DE media folders (EmuSen_Qt.md §3.5).
    pub cover_stores: Vec<String>,
    /// An ES-DE theme is the library's look: `LibraryStyle` is Theme and `BigPictureTheme` names a folder, read as the library
    /// model reads it (`esde_shelves::settings_of`), so the own look's screens are not made at the start (EmuSen_Qt.md §4.15).
    pub esde: bool,
    /// AppSettings.NavigationSounds: the ES-DE theme's navigation sounds, on unless the player turned them off.
    pub navigation_sounds: bool,
}

/// The platform library: `--platform-library PATH`, else `EMUSEN_PLATFORM_LIBRARY`, else the file beside the program.
pub fn platform_library(args: &[String], env: Option<String>, program: &Path) -> PathBuf {
    let named = args.iter().position(|a| a == "--platform-library").and_then(|i| args.get(i + 1)).cloned();
    named.or(env).map(PathBuf::from).unwrap_or_else(|| program.parent().unwrap_or(Path::new(".")).join(LIBRARY_FILE))
}

/// Moonrise on or off from the settings as Galaxia's model reads them; on when the file or the key is missing.
pub fn splash_from(settings: &Value) -> bool {
    settings.get("StartupSplash").and_then(Value::as_bool).unwrap_or(true)
}

/// The navigation sounds from the settings; on when the key is missing, as Galaxia's model has it.
pub fn navigation_sounds_from(settings: &Value) -> bool {
    settings.get("NavigationSounds").and_then(Value::as_bool).unwrap_or(true)
}

/// Reduced motion from the settings: on only when the player chose it.
pub fn reduced_motion_from(settings: &Value) -> bool {
    settings.get("ReducedMotion").and_then(Value::as_str) == Some("on")
}

/// The Guide chords from the settings, as Mistress's `GuideChordsOn` reads them: "On", "Off", or Auto (the default),
/// off under Steam's Game Mode, where Steam holds Guide's chords.
pub fn guide_chords_from(settings: &Value, game_mode: bool) -> bool {
    emusen_qt_models::settings::guide_chords_on(settings.get("GuideChords").and_then(Value::as_str), game_mode)
}

/// Steam's Game Mode from the environment, as Mistress reads it: gamescope among `XDG_CURRENT_DESKTOP`'s names, or,
/// with no desktop named, `SteamDeck=1`.
pub fn game_mode_from(desktop: Option<&str>, steam_deck: Option<&str>) -> bool {
    match desktop.filter(|d| !d.is_empty()) {
        Some(d) => d.split(':').any(|name| name.eq_ignore_ascii_case("gamescope")),
        None => steam_deck == Some("1"),
    }
}

/// The theme's folder: `--theme DIR`, else the environment's; empty for none.
pub fn theme(args: &[String], env: Option<String>) -> String {
    let named = args.iter().position(|a| a == "--theme").and_then(|i| args.get(i + 1)).cloned();
    named.or(env).unwrap_or_default()
}

/// The theme the settings choose, by its id, as the host's folder and the words when the reference stands in for it.
pub fn chosen_theme(settings: &Value, store: Option<&Path>) -> (String, String) {
    let id = themes::chosen_of(settings);
    let resolved = themes::resolve(&id, store);
    (resolved.host_folder(), resolved.words(&id))
}

/// The folders a theme's pictures may come from: the media store, then the settings' artwork and ES-DE media folders where set.
pub fn cover_stores_from(media: Option<&Path>, settings: &Value) -> Vec<String> {
    let set = |key: &str| settings.get(key).and_then(Value::as_str).filter(|v| !v.is_empty()).map(str::to_owned);
    media.map(|m| m.to_string_lossy().into_owned()).into_iter().chain(set("ArtworkDirectory")).chain(set("EsdeMediaDirectory")).collect()
}

/// The start read through the platform library at `library`; its settings unread (the defaults) when it cannot load.
pub fn read(library: &Path) -> Start {
    let platform = Platform::load(library).ok();
    let settings = platform.as_ref().and_then(|p| emusen_qt_models::clients::app_settings(p).ok()).unwrap_or(Value::Null);
    let media = platform.as_ref().and_then(|p| p.directory(DIR_MEDIA).ok().flatten());
    let args: Vec<String> = std::env::args().collect();
    let store = platform.as_ref().and_then(|p| p.directory(DIR_THEMES).ok().flatten());
    let named = theme(&args, std::env::var("EMUSEN_THEME").ok());
    let (theme, theme_words) = if named.is_empty() { chosen_theme(&settings, store.as_deref()) } else { (named, String::new()) };
    let game_mode = game_mode_from(std::env::var("XDG_CURRENT_DESKTOP").ok().as_deref(), std::env::var("SteamDeck").ok().as_deref());
    Start { platform_library: library.to_string_lossy().into_owned(), splash: splash_from(&settings), reduced_motion: reduced_motion_from(&settings),
        game_mode, guide_chords: guide_chords_from(&settings, game_mode),
        theme, theme_words, cover_stores: cover_stores_from(media.as_deref(), &settings),
        esde: esde_shelves::settings_of(&settings).themed, navigation_sounds: navigation_sounds_from(&settings),
    }
}

/// The start's facts as the window's initial properties, by the root's property names; the program adds the device's
/// feed and the gate's pad, and a test loads the window with the same map (EmuSen_Qt.md §4.15).
pub fn initial(start: &Start) -> QMap<QMapPair_QString_QVariant> {
    let mut initial = QMap::<QMapPair_QString_QVariant>::default();
    initial.insert(QString::from("platformLibrary"), QVariant::from(&QString::from(&start.platform_library)));
    initial.insert(QString::from("splash"), QVariant::from(&start.splash));
    initial.insert(QString::from("reducedMotion"), QVariant::from(&start.reduced_motion));
    initial.insert(QString::from("gameMode"), QVariant::from(&start.game_mode));
    initial.insert(QString::from("guideChords"), QVariant::from(&start.guide_chords));
    initial.insert(QString::from("theme"), QVariant::from(&QString::from(&start.theme)));
    initial.insert(QString::from("themeWords"), QVariant::from(&QString::from(&start.theme_words)));
    initial.insert(QString::from("esdeChosen"), QVariant::from(&start.esde));
    initial.insert(QString::from("navigationSounds"), QVariant::from(&start.navigation_sounds));
    let mut stores = QStringList::default();
    start.cover_stores.iter().for_each(|s| stores.append(QString::from(s)));
    initial.insert(QString::from("coverStores"), QVariant::from(&stores));
    initial
}
