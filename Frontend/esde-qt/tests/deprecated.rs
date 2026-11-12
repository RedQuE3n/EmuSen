//! EmuSen.Esde 1.1 keeps `hover(row)`, deprecated for `hoverAt` until the next major (EmuSen_Qt.md §4.8.3, §5.3),
//! doing what it did in 1.0: the row selected whether or not the pointer moves. Offscreen, on the software backend; one
//! test, as the harness is one per process.

use emusen_kit_core::input::UiButton;
use emusen_qt_test::{Harness, Scene};

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/Deprecated.qml");

/// A text carousel of systems and a text list of games, written for the test, nothing copied from a real theme.
struct Theme(std::path::PathBuf);

impl Theme {
    fn new() -> Theme {
        let root = std::env::temp_dir().join(format!("esde-qt-deprecated-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("capabilities.xml"), "<themeCapabilities></themeCapabilities>").unwrap();
        std::fs::write(
            root.join("theme.xml"),
            "<theme><view name=\"system\"><carousel name=\"systems\"><pos>0 0.3</pos><size>1 0.4</size><maxItemCount>3</maxItemCount>\
             <text>${system.fullName}</text><textColor>FFFFFF</textColor><fontSize>0.05</fontSize></carousel></view>\
             <view name=\"gamelist\"><textlist name=\"games\"><pos>0.05 0.1</pos><size>0.4 0.8</size><fontSize>0.04</fontSize>\
             <primaryColor>FFFFFF</primaryColor><selectedColor>FFFF00</selectedColor></textlist></view></theme>",
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

fn shelf(name: &str, full: &str, games: usize) -> String {
    let games: Vec<String> = (0..games)
        .map(|i| format!("{{\"file\":\"Game {i:04}.bin\",\"name\":\"Game {i:04}\",\"esde_system\":\"{name}\",\"favourite\":false,\"last_played\":0}}"))
        .collect();
    format!("{{\"name\":\"{name}\",\"full_name\":\"{full}\",\"theme\":\"{name}\",\"kind\":\"regular\",\"games\":[{}]}}", games.join(","))
}

fn until(scene: &mut Scene<'_>, what: &str, ok: impl Fn(&mut Scene<'_>) -> bool) {
    for _ in 0..300 {
        if ok(scene) {
            return;
        }
        scene.wait(10);
    }
    panic!("{what}: not within 3 s; the model's problem {:?}", scene.property("esdeModel", "problem"));
}

#[test]
fn qt_5_3_the_deprecated_hover_selects_its_row_as_in_1_0() {
    assert!(emusen_esde_qt::marks::mark_named("star").is_some(), "the module is linked into this test");
    let theme = Theme::new();
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 1280, 800);
    assert!(scene.set_property("view", "shelves", &format!("[{}]", shelf("nes", "Nintendo", 12))));
    let settings = serde_json::json!({ "theme": theme.0.to_str().unwrap() }).to_string();
    assert!(scene.set_property("view", "settings", &settings));
    until(&mut scene, "the theme loaded", |s| s.property("esdeModel", "themed") == "true");
    scene.pad(UiButton::Accept);
    until(&mut scene, "the gamelist", |s| s.property("esdeModel", "view") == "gamelist");
    until(&mut scene, "the view at rest", |s| s.property("esdeModel", "moving") == "false");
    assert_eq!(scene.property("esdeModel", "selected"), "0");

    // No pointer moves in this test: hover(row) selects the row all the same, as it did in 1.0, where hoverAt would not.
    assert!(scene.set_property("scene", "hoverRow", "3"));
    scene.wait(50);
    assert_eq!(scene.property("esdeModel", "selected"), "3", "hover(3) selects the fourth row with the pointer at rest");
    assert!(scene.set_property("scene", "hoverRow", "7"));
    scene.wait(50);
    assert_eq!(scene.property("esdeModel", "selected"), "7", "hover(7) selects the eighth");
    assert!(scene.set_property("scene", "hoverRow", "-1"));
    scene.wait(50);
    assert_eq!(scene.property("esdeModel", "selected"), "7", "hover(-1), off every row, leaves the selection");
    scene.assert_quiet();
}
