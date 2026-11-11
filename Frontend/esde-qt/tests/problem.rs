//! A text the load cannot read is the model's problem, said in its words, never an empty view or a text read as nothing;
//! no text at all is the property's default (EmuSen_Qt.md §4.9): driven through `EsdeModel`'s own load, offscreen. One
//! test, as the harness is one per process.

use emusen_qt_test::{Harness, Scene};

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/Load.qml");

fn until_problem(scene: &mut Scene<'_>, words: &str) {
    for _ in 0..200 {
        if scene.property("model", "problem").contains(words) {
            return;
        }
        scene.wait(10);
    }
    panic!("no problem saying {words:?}: {:?}", scene.property("model", "problem"));
}

fn until_themed(scene: &mut Scene<'_>, themed: bool) {
    for _ in 0..200 {
        if scene.property("model", "themed") == themed.to_string() {
            return;
        }
        scene.wait(10);
    }
    panic!("themed never {themed}: {:?}", scene.property("model", "problem"));
}

/// A theme of one carousel and one text list, written for the test, nothing copied from a real theme.
struct Theme(std::path::PathBuf);

impl Theme {
    fn new() -> Theme {
        let root = std::env::temp_dir().join(format!("esde-qt-problem-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("capabilities.xml"), "<themeCapabilities></themeCapabilities>").unwrap();
        std::fs::write(
            root.join("theme.xml"),
            "<theme><view name=\"system\"><carousel name=\"systems\"><pos>0 0.3</pos><size>1 0.4</size></carousel></view>\
             <view name=\"gamelist\"><textlist name=\"games\"><pos>0.1 0.1</pos><size>0.8 0.8</size></textlist></view></theme>",
        )
        .unwrap();
        Theme(root)
    }
}

impl Drop for Theme {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const SHELF: &str = "[{\"name\":\"snes\",\"full_name\":\"Super Nintendo\",\"theme\":\"snes\",\"kind\":\"regular\",\"games\":[{\"file\":\"Cobalt Harbor.sfc\",\"name\":\"Cobalt Harbor\",\"esde_system\":\"snes\",\"favourite\":false,\"last_played\":0}]}]";

#[test]
fn qt_4_9_a_text_the_load_cannot_read_is_the_model_s_problem() {
    assert!(emusen_esde_qt::marks::mark_named("star").is_some(), "the module is linked into this test");
    let theme = Theme::new();
    let settings = serde_json::json!({ "theme": theme.0.to_str().unwrap() }).to_string();
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 320, 200);
    // A library that reads is installed, so each text that does not read below takes a view that was there away.
    assert!(scene.set_property("model", "media", "{}"));
    assert!(scene.set_property("model", "presence", "{}"));
    assert!(scene.set_property("model", "shelves", SHELF));
    assert!(scene.set_property("model", "settings", &settings));
    until_themed(&mut scene, true);
    assert_eq!(scene.property("model", "problem"), "", "the library read");

    assert!(scene.set_property("model", "shelves", "[{"));
    until_problem(&mut scene, "The library's shelves could not be read");
    assert_eq!(scene.property("model", "themed"), "false", "nothing installed");

    // Shelves that read, and a media presence that does not: said, not read as no media.
    assert!(scene.set_property("model", "shelves", SHELF));
    until_themed(&mut scene, true);
    assert!(scene.set_property("model", "presence", "{\"snes\":"));
    until_problem(&mut scene, "The library's media presence could not be read");
    assert_eq!(scene.property("model", "themed"), "false", "nothing installed");

    // Settings that do not read are said, not read as the defaults.
    assert!(scene.set_property("model", "presence", "{}"));
    until_themed(&mut scene, true);
    assert!(scene.set_property("model", "settings", "{\"theme\":"));
    until_problem(&mut scene, "The view's settings could not be read");
    assert_eq!(scene.property("model", "themed"), "false", "nothing installed");
    assert!(scene.set_property("model", "settings", &settings));
    until_themed(&mut scene, true);

    // A device status that does not read is said, and one that reads takes the words away; no text is no indicator.
    assert!(scene.set_property("model", "device", "{\"wifi\":"));
    until_problem(&mut scene, "The device's status could not be read");
    assert!(scene.set_property("model", "device", "{\"wifi\":true}"));
    assert_eq!(scene.property("model", "problem"), "", "a status that reads");
    assert!(scene.set_property("model", "device", "{\"wifi\":"));
    until_problem(&mut scene, "The device's status could not be read");
    assert!(scene.set_property("model", "device", ""));
    assert_eq!(scene.property("model", "problem"), "", "no status text is no indicator, not a problem");

    // No shelves text (the property's own default, a host that sets the settings first) is no shelves, not a problem.
    assert!(scene.set_property("model", "shelves", ""));
    until_themed(&mut scene, false);
    assert_eq!(scene.property("model", "problem"), "", "no shelves text is no shelves");
    assert!(scene.set_property("model", "shelves", SHELF));
    until_themed(&mut scene, true);

    // A media index that does not read is said; no media index text at all (a host that gives none, as the model's own
    // default) is no media: the load ends as it does with an empty index, `{}`.
    assert!(scene.set_property("model", "media", "{"));
    until_problem(&mut scene, "The library's media index could not be read");
    assert!(scene.set_property("model", "media", "{}"));
    for _ in 0..200 {
        if !scene.property("model", "problem").contains("could not be read") {
            break;
        }
        scene.wait(10);
    }
    let with_index = scene.property("model", "problem");
    assert!(!with_index.contains("could not be read"), "{with_index}");
    let revision = scene.property("model", "revision");
    assert!(scene.set_property("model", "media", ""));
    for _ in 0..200 {
        if scene.property("model", "revision") != revision {
            break;
        }
        scene.wait(10);
    }
    assert_ne!(scene.property("model", "revision"), revision, "the load with no media index came back");
    assert_eq!(scene.property("model", "problem"), with_index, "no media index text is no media, as an empty index");
    scene.assert_quiet();
}
