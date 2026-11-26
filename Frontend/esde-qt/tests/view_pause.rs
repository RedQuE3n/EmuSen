// Software test verifying View.qml gates its animation driver and idle timer on root.visible.

use std::path::{Path, PathBuf};

use emusen_kit_core::input::UiButton;
use emusen_qt_test::{Harness, Scene};

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/View.qml");

struct TempTheme(PathBuf);

impl Drop for TempTheme {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn write_theme(root: &Path) {
    std::fs::create_dir_all(root).unwrap();
    std::fs::write(root.join("capabilities.xml"), "<themeCapabilities></themeCapabilities>").unwrap();
    std::fs::write(
        root.join("theme.xml"),
        "<theme><view name=\"system\">\
         <carousel name=\"systems\"><pos>0 0.3</pos><size>1 0.4</size><maxItemCount>3</maxItemCount>\
         <text>${system.fullName}</text><textColor>FFFFFF</textColor><fontSize>0.05</fontSize></carousel>\
         <clock name=\"clock\"><pos>0.05 0.05</pos><size>0.2 0.1</size><format>%H:%M:%S</format>\
         <color>FFFFFF</color><fontSize>0.04</fontSize></clock>\
         </view></theme>",
    )
    .unwrap();
}

fn shelf(name: &str, full: &str) -> String {
    format!("{{\"name\":\"{name}\",\"full_name\":\"{full}\",\"theme\":\"{name}\",\"kind\":\"regular\",\"games\":[{{\"file\":\"{name}.rom\",\"name\":\"Sample\",\"esde_system\":\"{name}\"}}]}}")
}

fn until(scene: &mut Scene<'_>, what: &str, ok: impl Fn(&mut Scene<'_>) -> bool) {
    let mut waited = 0;
    while !ok(scene) {
        assert!(waited < 5000, "{what} within 5 s (problem: {:?})", scene.property("esdeModel", "problem"));
        scene.wait(20);
        waited += 20;
    }
}

#[test]
fn view_gates_animation_driver_and_timer_on_visible() {
    assert!(emusen_esde_qt::marks::mark_named("star").is_some(), "the module is linked into this test");
    let theme = TempTheme(std::env::temp_dir().join(format!("esde-qt-view-pause-{}", std::process::id())));
    write_theme(&theme.0);
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 1280, 800);
    let shelves = format!("[{},{},{}]", shelf("nes", "Nintendo"), shelf("snes", "Super Nintendo"), shelf("n64", "Nintendo 64"));
    let settings = format!("{{\"theme\":\"{}\",\"interface\":{{\"DisplayClock\":true}}}}", theme.0.display());
    assert!(scene.set_property("view", "shelves", &shelves));
    assert!(scene.set_property("view", "settings", &settings));
    until(&mut scene, "theme loaded", |s| s.property("esdeModel", "themed") == "true");
    until(&mut scene, "initial settle", |s| s.property("esdeModel", "moving") == "false");

    // Hiding View pauses the running animation driver and stops frames.
    scene.pad(UiButton::Right);
    scene.wait(60);
    assert_eq!(scene.property("esdeModel", "moving"), "true");
    assert_eq!(scene.property("viewClock", "driver.running"), "true");
    assert!(!scene.running().is_empty(), "animation runs while View is visible");

    assert!(scene.set_property_now("view", "visible", "false"));
    scene.wait(30);
    assert_eq!(scene.property("viewClock", "driver.running"), "false");
    assert!(scene.running().is_empty(), "hidden View pauses animation driver");
    let frames_hidden = scene.frames();
    scene.wait(200);
    assert_eq!(scene.frames(), frames_hidden, "no frames swap while View is hidden");

    // Showing View resumes the driver and lets the transition settle.
    assert!(scene.set_property_now("view", "visible", "true"));
    scene.wait(30);
    assert_eq!(scene.property("viewClock", "driver.running"), "true");
    until(&mut scene, "glide settled after resume", |s| s.property("esdeModel", "moving") == "false");

    // Hiding View also pauses the idle Timer driven by nextChangeMs > 0.
    assert_eq!(scene.property("esdeModel", "nextChangeMs"), "1000");
    assert!(scene.set_property_now("view", "visible", "false"));
    scene.wait(50);
    let rev_hidden = scene.property("esdeModel", "revision");
    scene.wait(1200);
    assert_eq!(scene.property("esdeModel", "revision"), rev_hidden, "hidden View must not tick idle Timer");

    assert!(scene.set_property_now("view", "visible", "true"));
    until(&mut scene, "visible View ticks idle Timer", |s| s.property("esdeModel", "revision") != rev_hidden);
    scene.assert_quiet();
}
