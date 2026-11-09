//! What the program reads before its window opens (EmuSen_Qt.md §7): where the platform library is, and Moonrise on or off.

use emusen_qt::start::{chosen_theme, cover_stores_from, game_mode_from, guide_chords_from, platform_library, reduced_motion_from, splash_from, theme, LIBRARY_FILE};
use serde_json::json;
use std::path::Path;

#[test]
fn the_platform_library_is_the_named_one_then_the_environment_s_then_beside_the_program() {
    let program = Path::new("/opt/emusen/emusen-qt");
    let named = vec!["emusen-qt".to_owned(), "--platform-library".to_owned(), "/a/lib.so".to_owned()];
    assert_eq!(platform_library(&named, Some("/b/lib.so".into()), program), Path::new("/a/lib.so"));
    assert_eq!(platform_library(&[], Some("/b/lib.so".into()), program), Path::new("/b/lib.so"));
    assert_eq!(platform_library(&[], None, program), Path::new("/opt/emusen").join(LIBRARY_FILE));
}

#[test]
fn moonrise_is_on_unless_the_player_turned_it_off() {
    assert!(splash_from(&serde_json::Value::Null), "no settings yet");
    assert!(splash_from(&json!({ "RomDirectory": "/r" })), "the key missing");
    assert!(!splash_from(&json!({ "StartupSplash": false })));
    assert!(splash_from(&json!({ "StartupSplash": true })));
}

#[test]
fn motion_is_reduced_only_when_the_player_chose_it() {
    assert!(!reduced_motion_from(&serde_json::Value::Null));
    assert!(reduced_motion_from(&json!({ "ReducedMotion": "on" })));
    assert!(!reduced_motion_from(&json!({ "ReducedMotion": "off" })));
    assert!(!reduced_motion_from(&json!({ "ReducedMotion": "system" })), "the desktop's preference is not read yet");
}

#[test]
fn steam_s_game_mode_is_read_as_mistress_reads_it() {
    assert!(game_mode_from(Some("gamescope"), None));
    assert!(game_mode_from(Some("KDE:Gamescope"), Some("0")), "gamescope among the desktop's names, in any case");
    assert!(!game_mode_from(Some("KDE"), Some("1")), "a Deck in Desktop Mode names its desktop");
    assert!(game_mode_from(None, Some("1")), "no desktop named, SteamDeck=1");
    assert!(game_mode_from(Some(""), Some("1")));
    assert!(!game_mode_from(None, None));
}

#[test]
fn the_guide_chords_are_on_off_or_auto_away_from_game_mode() {
    assert!(guide_chords_from(&serde_json::Value::Null, false), "Auto, the default, on the desktop");
    assert!(!guide_chords_from(&serde_json::Value::Null, true), "Auto under Game Mode, where Steam holds Guide");
    assert!(guide_chords_from(&json!({ "GuideChords": "On" }), true));
    assert!(!guide_chords_from(&json!({ "GuideChords": "Off" }), false));
    assert!(!guide_chords_from(&json!({ "GuideChords": "Auto" }), true));
}

#[test]
fn the_theme_is_the_named_one_then_the_environment_s_and_else_the_reference() {
    let named = vec!["emusen-qt".to_owned(), "--theme".to_owned(), "/t/Dawn".to_owned()];
    assert_eq!(theme(&named, Some("/t/Day".into())), "/t/Dawn");
    assert_eq!(theme(&[], Some("/t/Day".into())), "/t/Day");
    assert_eq!(theme(&[], None), "", "no theme: the reference");
}

#[test]
fn a_theme_s_pictures_come_from_the_media_store_and_the_artwork_folders_set() {
    let settings = json!({ "ArtworkDirectory": "/art", "EsdeMediaDirectory": "" });
    assert_eq!(cover_stores_from(Some(Path::new("/media")), &settings), ["/media", "/art"]);
    assert!(cover_stores_from(None, &serde_json::Value::Null).is_empty());
}

#[test]
fn the_settings_theme_id_resolves_through_the_store_and_a_missing_one_falls_back_with_words() {
    let store = std::env::temp_dir().join(format!("emusen-start-themes-{}", std::process::id()));
    std::fs::create_dir_all(store.join("dawn")).unwrap();
    std::fs::write(store.join("dawn/theme.json"), "{}").unwrap();
    assert_eq!(chosen_theme(&serde_json::Value::Null, Some(&store)), (String::new(), String::new()), "none stored: the reference");
    assert_eq!(chosen_theme(&json!({ "EmuSenTheme": "emusen" }), Some(&store)), (String::new(), String::new()));
    assert_eq!(chosen_theme(&json!({ "EmuSenTheme": "dawn" }), Some(&store)).0, store.join("dawn").to_str().unwrap());
    let (folder, words) = chosen_theme(&json!({ "EmuSenTheme": "gone" }), Some(&store));
    assert_eq!(folder, "", "a missing folder: the reference");
    assert!(words.contains("“gone”") && words.contains("EmuSen's own theme is shown"), "{words}");
    std::fs::remove_dir_all(store).unwrap();
}
