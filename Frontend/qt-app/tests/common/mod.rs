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
use std::path::PathBuf;

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
    hold_in(scene, screen, m, named, report, Variant::Dark);
}

#[allow(dead_code)]
/// As `hold`, the state drawn in a variant of its own, its palette and contrast judged in it.
pub fn hold_in(scene: &mut Scene<'_>, screen: &str, m: &mockup::Mockup, named: Named, report: &mut Vec<String>, variant: Variant) {
    for (w, h) in SIZES {
        scene.resize(w, h);
        scene.wait(150);
        let mut tree = scene.tree();
        let focus = scene.focus(&tree);
        let name = format!("{screen}-{w}x{h}");
        let mut faults = Vec::new();
        if f64::from(w) / f64::from(h) == 1.6 {
            faults.extend(mockup::compare(m, &tree.nodes, f64::from(h) / 800.0, variant).into_iter().map(|f| format!("mockup: {f}")));
        }
        assert!(scene.grab(&out().join(format!("{name}.png")).display().to_string()), "grab {name}");
        scene.sample_behind(&mut tree, f64::from(w), f64::from(h));
        let walk = scene.walk(&tree);
        let walked = edges(&walk);
        let declared = palette(variant);
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
                .audit(&tree, WalkForm::BigScreen, variant, focus, &walked, f64::from(w), f64::from(h))
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

#[allow(unused_imports)]
pub use emusen_qt_test::covers::draw as draw_covers;
