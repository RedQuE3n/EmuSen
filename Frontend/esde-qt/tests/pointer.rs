//! The ES-DE view's pointer selects only as it moves (EmuSen_Kit.md §7.108, EmuSen_Qt.md §4.8.5): a gamelist made under a
//! pointer at rest keeps its selection, and the pointer's own move onto a row selects it. Offscreen, on the software
//! backend; one test, as the harness is one per process.

use emusen_kit_core::input::UiButton;
use emusen_qt_test::{Harness, Scene};

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/View.qml");

/// A text carousel of systems and a text list of games, written for the test, nothing copied from a real theme.
struct Theme(std::path::PathBuf);

impl Theme {
    fn new() -> Theme {
        let root = std::env::temp_dir().join(format!("esde-qt-pointer-{}", std::process::id()));
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

/// The middle of the gamelist's row `index`, by its name.
fn row_middle(scene: &mut Scene<'_>, index: usize) -> (f64, f64) {
    let tree = scene.tree();
    let row = tree.nodes.iter().find(|n| n.visible && n.name == format!("row-{index}")).unwrap_or_else(|| panic!("the row row-{index}"));
    (row.x + row.w / 2.0, row.y + row.h / 2.0)
}

fn settled(scene: &mut Scene<'_>) {
    until(scene, "the view at rest", |s| s.property("esdeModel", "moving") == "false");
}

#[test]
fn kit_7_108_the_es_de_view_selects_only_as_the_pointer_moves() {
    assert!(emusen_esde_qt::marks::mark_named("star").is_some(), "the module is linked into this test");
    let theme = Theme::new();
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 1280, 800);
    let shelves = format!("[{},{}]", shelf("nes", "Nintendo", 12), shelf("snes", "Super Nintendo", 12));
    assert!(scene.set_property("view", "shelves", &shelves));
    let settings = serde_json::json!({ "theme": theme.0.to_str().unwrap() }).to_string();
    assert!(scene.set_property("view", "settings", &settings));
    until(&mut scene, "the theme loaded", |s| s.property("esdeModel", "themed") == "true");
    assert_eq!(scene.property("esdeModel", "view"), "system");

    // A opens the NES gamelist; the pointer moves onto its fourth row, which it selects.
    scene.pad(UiButton::Accept);
    until(&mut scene, "the gamelist", |s| s.property("esdeModel", "view") == "gamelist");
    settled(&mut scene);
    assert_eq!(scene.property("esdeModel", "selected"), "0");
    let fourth = row_middle(&mut scene, 3);
    scene.hover(fourth.0, fourth.1 - 4.0);
    scene.hover(fourth.0, fourth.1);
    scene.wait(50);
    assert_eq!(scene.property("esdeModel", "selected"), "3", "the pointer's own move onto the fourth row selects it");

    // Down to the fifth row, the pointer left over the fourth. B to the systems, Right to the SNES and A: its gamelist
    // opens on the fifth row, as the view carries its row from list to list, and is made under the pointer at rest over
    // its fourth row; it keeps the fifth selected.
    scene.pad(UiButton::Down);
    until(&mut scene, "Down to the fifth row", |s| s.property("esdeModel", "selected") == "4");
    scene.pad(UiButton::Back);
    until(&mut scene, "the systems", |s| s.property("esdeModel", "view") == "system");
    settled(&mut scene);
    scene.pad(UiButton::Right);
    until(&mut scene, "the SNES", |s| s.property("esdeModel", "shelf") == "snes");
    settled(&mut scene);
    scene.pad(UiButton::Accept);
    until(&mut scene, "the SNES gamelist", |s| s.property("esdeModel", "view") == "gamelist");
    settled(&mut scene);
    assert_eq!(row_middle(&mut scene, 3), fourth, "the new list's fourth row under the resting pointer");
    scene.wait(200);
    assert_eq!(scene.property("esdeModel", "selected"), "4", "the gamelist made under the resting pointer keeps its selection, not the fourth row under it");

    // The pointer's own move onto the sixth row selects it.
    let sixth = row_middle(&mut scene, 5);
    scene.hover(sixth.0, sixth.1);
    scene.wait(50);
    assert_eq!(scene.property("esdeModel", "selected"), "5", "the pointer's own move onto the sixth row selects it");
    scene.assert_quiet();
}
