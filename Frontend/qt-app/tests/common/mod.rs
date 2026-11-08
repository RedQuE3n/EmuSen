//! What the program's tests share: a sandbox home with synthetic games, and waiting on a property.

use emusen_client_fixture::{library, World};
use emusen_crystal_client::Config;
use emusen_qt_models::clients::ClientSource;
use emusen_qt_models::library::{world_line_of, LibrarySource};
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_kit_core::input::WalkForm;
use emusen_kit_core::tokens::Variant;
use emusen_qt_test::harness::{edges, palette, Measures, SIZES};
use emusen_qt_test::spec::{self, Header};
use emusen_qt_test::{mockup, Scene};
use cxx_qt::QObject;
use emusen_qt_test::Harness;
use std::path::{Path, PathBuf};

pub fn wait_for(scene: &mut emusen_qt_test::Scene<'_>, object: &str, property: &str, value: &str, ms: i32) {
    let mut waited = 0;
    while scene.property(object, property) != value && waited < ms {
        scene.wait(50);
        waited += 50;
    }
    assert_eq!(scene.property(object, property), value, "{object}.{property} after {ms} ms");
}

/// A scratch tree whose settings name its ROM folder, holding one synthetic image for each (folder, file) given.
pub fn sandbox(name: &str, games: &[(&str, &str)]) -> World {
    let world = World::new(name);
    for (folder, file) in games {
        let roms = world.dir(&format!("Roms/{folder}"));
        let mut image = b"NES\x1a\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
        image.resize(16 + 16384, 0);
        std::fs::write(roms.join(file), image).unwrap();
    }
    let platform = Platform::load(library()).unwrap();
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = serde_json::json!(world.root.join("Roms").to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    world
}

/// A scratch tree like `sandbox` at a library's scale: `count` games in each (folder, extension), each a hard link to one
/// synthetic image, so thousands of games take one file's room.
#[allow(dead_code)]
pub fn large_sandbox(name: &str, shelves: &[(&str, &str, usize)]) -> World {
    let world = sandbox(name, &[]);
    for (folder, ext, count) in shelves {
        let roms = world.dir(&format!("Roms/{folder}"));
        let first = roms.join(format!("Game 00000.{ext}"));
        let mut image = b"NES\x1a\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
        image.resize(16 + 16384, 0);
        std::fs::write(&first, image).unwrap();
        for n in 1..*count {
            std::fs::hard_link(&first, roms.join(format!("Game {n:05}.{ext}"))).unwrap();
        }
    }
    world
}

/// The courts canvas's consoles, as Mistress's picture states them (MoonrisePictureTool.Consoles).
pub const CONSOLES: [&str; 11] = ["nes", "snes", "n64", "gb", "gbc", "sms", "md", "mcd", "psx", "ps2", "dreamcast"];

/// The canvas's consoles' worlds as lines, from this commit's systems table in its order.
#[allow(dead_code)]
pub fn canvas_worlds() -> Vec<String> {
    let source = ClientSource::open(library().to_str().unwrap(), &Config::default()).unwrap();
    let worlds: Vec<String> =
        source.systems().unwrap().iter().filter(|s| CONSOLES.contains(&s.id.as_str())).map(world_line_of).collect();
    assert_eq!(worlds.len(), CONSOLES.len(), "every console of the canvas is in the table");
    worlds
}

/// Where states are recorded and grabbed: `EMUSEN_QT_COMPARE`, else the temporary folder.
#[allow(dead_code)]
pub fn out() -> PathBuf {
    let dir = std::env::var_os("EMUSEN_QT_COMPARE").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A test's differences named for the tester, which a held state does not count as faults: the fault, the tree, the size.
#[allow(dead_code)]
pub type Named = fn(&str, &emusen_qt_test::tree::Tree, (i32, i32)) -> bool;

#[allow(dead_code)]
/// Holds one state: the mockup where the size is the canvas's or its 1.5 times, the gate at every size, the spec and the grab.
pub fn hold(scene: &mut Scene<'_>, screen: &str, m: &mockup::Mockup, named: Named, report: &mut Vec<String>) {
    for (w, h) in SIZES {
        scene.resize(w, h);
        scene.wait(150);
        let mut tree = scene.tree();
        let focus = scene.focus(&tree);
        let name = format!("{screen}-{w}x{h}");
        let mut faults = Vec::new();
        if f64::from(w) / f64::from(h) == 1.6 {
            faults.extend(mockup::compare(m, &tree.nodes, f64::from(h) / 800.0, Variant::Dark).into_iter().map(|f| format!("mockup: {f}")));
        }
        assert!(scene.grab(&out().join(format!("{name}.png")).display().to_string()), "grab {name}");
        scene.sample_behind(&mut tree, f64::from(w), f64::from(h));
        let walk = scene.walk(&tree);
        let walked = edges(&walk);
        let declared = palette(Variant::Dark);
        let header = Header {
            state: &name,
            case: screen,
            kind: "menu",
            width: w,
            height: h,
            measures: Measures::of(WalkForm::BigScreen, f64::from(h)),
            palette: &declared,
        };
        std::fs::write(out().join(format!("{name}.spec")), spec::write(&tree, &tree.root, &header, focus, &walk)).unwrap();
        faults.extend(
            scene
                .audit(&tree, WalkForm::BigScreen, Variant::Dark, focus, &walked, f64::from(w), f64::from(h))
                .iter()
                .map(|f| format!("gate: {f}")),
        );
        faults.retain(|f| !named(f, &tree, (w, h)));
        let notes = out().join(format!("{name}.faults.txt"));
        if faults.is_empty() {
            let _ = std::fs::remove_file(&notes);
        } else {
            std::fs::write(&notes, format!("{}\n-- items:\n{}\n", faults.join("\n"), tree.legend())).unwrap();
            let who = |f: &str| -> String {
                let ids = f.rsplit(": ").next().unwrap_or("").split(|c: char| !c.is_ascii_digit()).filter_map(|n| n.parse::<usize>().ok());
                ids.filter_map(|i| tree.nodes.get(i)).map(|n| format!("{} {}", n.ty, n.name)).collect::<Vec<_>>().join(", ")
            };
            report.extend(faults.iter().map(|f| format!("{name}: {f} [{}]", who(f))));
        }
    }
    scene.resize(1280, 800);
    scene.wait(100);
}

/// The covers as Mistress's own-look fixture draws them, each a flat colour with a white rim and its words, grabbed from
/// one Qt Quick window resized to each box's shape (SyntheticLibrary.Label's drawing). `prepare` is handed the window
/// before it shows (on Vulkan, the program's device adopted), and the window is hidden once drawn: a second window's
/// swapchain released only when the engine was dropped faulted in Qt's release.
#[allow(dead_code)]
pub fn draw_covers(harness: &mut Harness, scratch: &Path, covers: &[(PathBuf, (u32, u32), usize, String)], prepare: impl FnOnce(*mut QObject)) {
    let qml = scratch.join("cover.qml");
    std::fs::write(
        &qml,
        "import QtQuick\nWindow {\n  objectName: \"covers\"\n  Rectangle { objectName: \"fill\"; anchors.fill: parent\n\
         Rectangle { x: 3; y: 3; width: parent.width - 6; height: parent.height - 6; color: \"transparent\"; border.color: \"white\"; border.width: 6 }\n\
         Text { objectName: \"words\"; anchors.centerIn: parent; width: parent.width - 20; horizontalAlignment: Text.AlignHCenter; color: \"white\"; font.pixelSize: Math.max(12, Math.floor(parent.height / 12)) } } }\n",
    )
    .unwrap();
    let mut scene = harness.load_prepared(&format!("file://{}", qml.display()), 64, 64, prepare);
    for (path, (w, h), index, words) in covers {
        // Green is the covers folder's: 60 + 6 * 23 % 160, the name's length in SyntheticLibrary.Colour.
        let (r, g, b) = (40 + 17 * index % 180, 198, 90 + 31 * index % 150);
        scene.resize(*w as i32, *h as i32);
        assert!(scene.set_property("fill", "color", &format!("#{r:02x}{g:02x}{b:02x}")));
        assert!(scene.set_property("words", "text", words));
        scene.wait(50);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        assert!(scene.grab(&path.display().to_string()), "cover {}", path.display());
    }
    // Hidden while its scene graph still holds its instance, so its swapchain is released before the window goes.
    assert!(scene.set_property("covers", "visible", "false"));
    scene.wait(50);
}
