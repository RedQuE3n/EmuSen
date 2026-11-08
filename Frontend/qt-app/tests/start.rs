//! What the program reads before its window opens (EmuSen_Qt.md §7): where the platform library is, and Moonrise on or off.

use emusen_qt::start::{platform_library, reduced_motion_from, splash_from, LIBRARY_FILE};
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
