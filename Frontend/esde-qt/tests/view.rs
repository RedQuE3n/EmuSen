//! The view on the GPU tier (EmuSen_Qt.md §8.1, tier 2): the bridge and `View.qml` over a synthetic theme the test
//! writes, Vulkan in a compositor of the test's own with the validation layer on (EmuSen_Qt.md §4.8.4).

use std::path::PathBuf;
use std::time::Instant;

use emusen_kit_core::input::UiButton;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::{Harness, Scene};

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/View.qml");
const W: i32 = 1280;
const H: i32 = 800;

// A text carousel of systems and a text list of games with the game's name beside it; nothing from a real theme.
fn theme() -> PathBuf {
    let root = std::env::temp_dir().join(format!("esde-qt-view-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("capabilities.xml"), "<themeCapabilities></themeCapabilities>").unwrap();
    std::fs::write(
        root.join("theme.xml"),
        "<theme><view name=\"system\"><carousel name=\"systems\"><pos>0 0.3</pos><size>1 0.4</size><maxItemCount>3</maxItemCount>\
         <itemScale>1.5</itemScale><text>${system.fullName}</text><textColor>FFFFFF</textColor><fontSize>0.05</fontSize></carousel></view>\
         <view name=\"gamelist\"><textlist name=\"games\"><pos>0.05 0.1</pos><size>0.4 0.8</size><fontSize>0.04</fontSize>\
         <primaryColor>FFFFFF</primaryColor><selectedColor>FFFF00</selectedColor><selectorColor>404080</selectorColor></textlist>\
         <text name=\"title\"><pos>0.55 0.1</pos><size>0.4 0.1</size><metadata>name</metadata><fontSize>0.05</fontSize><color>FFFFFF</color></text>\
         </view></theme>",
    )
    .unwrap();
    root
}

fn shelf(name: &str, full: &str, games: usize) -> String {
    let rows: Vec<String> = (0..games)
        .map(|i| format!("{{\"file\":\"{name}-{i:04}.rom\",\"name\":\"Game {i:04}\",\"esde_system\":\"{name}\",\"favourite\":false,\"last_played\":0}}"))
        .collect();
    format!("{{\"name\":\"{name}\",\"full_name\":\"{full}\",\"theme\":\"{name}\",\"kind\":\"regular\",\"games\":[{}]}}", rows.join(","))
}

fn settings(theme: &PathBuf) -> String {
    format!("{{\"theme\":\"{}\",\"interface\":{{\"DisplayClock\":false}}}}", theme.display())
}

fn until(scene: &mut Scene<'_>, what: &str, ok: impl Fn(&mut Scene<'_>) -> bool) {
    let mut waited = 0;
    while !ok(scene) {
        assert!(waited < 5000, "{what} within 5 s");
        scene.wait(20);
        waited += 20;
    }
}

fn prop(scene: &mut Scene<'_>, name: &str, property: &str) -> String {
    scene.property(name, property)
}

// The words drawn under the item of this objectName: Repeater's delegates are reached through the item tree only.
fn words(scene: &mut Scene<'_>, name: &str) -> String {
    let tree = scene.tree();
    let Some(top) = tree.nodes.iter().find(|n| n.name == name).map(|n| n.id.clone()) else { return String::new() };
    let under = |start: &str| {
        let mut id = start.to_owned();
        loop {
            if id == top {
                return true;
            }
            match tree.nodes.iter().find(|n| n.id == id) {
                Some(n) if !n.parent.is_empty() => id = n.parent.clone(),
                _ => return false,
            }
        }
    };
    let found: Vec<String> = tree.nodes.iter().filter(|n| n.visible && n.text.as_deref().is_some_and(|t| !t.is_empty()) && under(&n.id)).filter_map(|n| n.text.clone()).collect();
    found.join("|")
}

fn box_of(scene: &mut Scene<'_>, name: &str) -> (f64, f64, f64, f64) {
    scene.tree().nodes.iter().find(|n| n.name == name).map(|n| (n.x, n.y, n.w, n.h)).unwrap_or_else(|| panic!("{name} drawn"))
}

fn rows_drawn(scene: &mut Scene<'_>) -> usize {
    scene.tree().nodes.iter().filter(|n| n.name.starts_with("row-") && n.visible).count()
}

fn start() -> Compositor {
    let kwin = Compositor::start(3840, 2160);
    kwin.watch(180);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    std::env::set_var("QSG_RHI_DEBUG_LAYER", "1");
    std::env::set_var("VK_LAYER_SYNCVAL_SHADER_ACCESSES_HEURISTIC", "1");
    std::env::set_var("QT_VK_PHYSICAL_DEVICE_INDEX", std::env::var("EMUSEN_GPU_INDEX").unwrap_or_else(|_| "0".into()));
    kwin
}

#[test]
fn the_view_binds_the_scene_and_asks_for_frames_only_while_it_moves() {
    assert!(emusen_esde_qt::marks::mark_named("star").is_some(), "the module is linked into this test");
    let _kwin = start();
    let theme = theme();
    let scratch = std::env::temp_dir().join(format!("esde-qt-view-grabs-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, W, H);
    let shelves = format!("[{},{},{}]", shelf("nes", "Nintendo", 3), shelf("snes", "Super Nintendo", 4000), shelf("n64", "Nintendo 64", 2));
    assert!(scene.set_property("view", "shelves", &shelves));
    assert!(scene.set_property("view", "settings", &settings(&theme)));
    until(&mut scene, "the theme loaded", |s| s.property("esdeModel", "themed") == "true");
    assert_eq!(prop(&mut scene, "esdeModel", "problem"), "");
    assert_eq!(prop(&mut scene, "esdeModel", "view"), "system");

    // At rest the system view swaps no frame and asks for no wake.
    scene.assert_still("the system view at rest");
    assert_eq!(prop(&mut scene, "esdeModel", "nextChangeMs"), "0");

    // A carousel step moves for its 400 ms, then rests on the static frame of the new state, pixel for pixel.
    scene.pad(UiButton::Right);
    scene.wait(150);
    assert_eq!(prop(&mut scene, "esdeModel", "moving"), "true", "the carousel step glides");
    let before = scene.frames();
    scene.wait(100);
    assert!(scene.frames() > before + 2, "frames swapped while the carousel glides");
    until(&mut scene, "the step settled", |s| s.property("esdeModel", "moving") == "false");
    scene.assert_still("the carousel after its step");
    let stepped = scratch.join("stepped.png");
    assert!(scene.grab(&stepped.to_string_lossy()));
    assert!(scene.set_property("view", "settings", &format!("{} ", settings(&theme))), "a reload installs the new state as a static frame");
    scene.wait(500);
    assert_eq!(scene.difference(&stepped.to_string_lossy(), 0, 0, &[], ""), Some(0), "the step's last frame is the static frame of the new state");

    // A opens the selected system's gamelist (SNES, 4,000 games); this theme names no transition.
    scene.pad(UiButton::Accept);
    until(&mut scene, "the slide settled", |s| s.property("esdeModel", "moving") == "false");
    assert_eq!(prop(&mut scene, "esdeModel", "view"), "gamelist");
    assert_eq!(words(&mut scene, "element-title"), "Game 0000");

    // The list draws its visible rows only, and a step jumps in one frame: no glide, the name at once.
    let drawn = rows_drawn(&mut scene);
    let row_h = box_of(&mut scene, "row-0").3;
    let shown = (box_of(&mut scene, "element-games").3 / row_h).ceil() as usize;
    assert!(drawn <= shown + 1 && drawn >= shown - 1, "{drawn} row delegates for {shown} visible rows of 4,000");
    let t = Instant::now();
    scene.pad(UiButton::Down);
    let step_ms = t.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(words(&mut scene, "element-title"), "Game 0001", "the selection and its metadata move in the same frame");
    assert_eq!(prop(&mut scene, "esdeModel", "moving"), "false", "a list step does not glide");
    eprintln!("a step over 4,000 games: {step_ms:.2} ms including the harness's 5 ms wait");

    // The pointer makes the pad's calls: a hover selects the row, a tap plays it, the wheel steps.
    let row3 = box_of(&mut scene, "row-3");
    let games = (row3.0 + row3.2 / 2.0, row3.1 + row3.3 / 2.0);
    scene.hover(games.0, games.1);
    scene.wait(50);
    assert_eq!(words(&mut scene, "element-title"), "Game 0003", "hover selects as the pad's step does");
    scene.wheel(games.0, games.1, -1);
    scene.wait(50);
    assert_eq!(words(&mut scene, "element-title"), "Game 0004", "a notch down steps on");
    assert!(scene.watch("esdeModel", "view"));
    scene.click(games.0, games.1);
    scene.wait(50);
    assert_eq!(words(&mut scene, "element-title"), "Game 0003", "the tap selects its row");
    scene.right_click(games.0, games.1);
    until(&mut scene, "B went back", |s| s.property("esdeModel", "view") == "system");

    scene.assert_quiet();
    let _ = std::fs::remove_dir_all(&theme);
    let _ = std::fs::remove_dir_all(&scratch);
}
