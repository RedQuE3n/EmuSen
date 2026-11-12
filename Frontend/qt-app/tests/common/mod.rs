//! What the program's tests share: a sandbox home with synthetic games, and waiting on a property.

pub mod console;
pub mod cores_fit;

use emusen_client_fixture::{library, World};
use emusen_crystal_client::Config;
use emusen_qt_models::clients::ClientSource;
use emusen_qt_models::library::{world_line_of, LibrarySource};
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_kit_core::input::WalkForm;
use emusen_kit_core::tokens::Variant;
use emusen_kit_core::input::UiButton;
use emusen_qt_test::harness::{edges, palette, Measures, DIRECTIONS, SIZES};
use emusen_qt_test::spec::{self, Header};
use emusen_qt_test::excuse::{Excuses, Finding};
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

#[allow(dead_code)]
/// Holds one state: the mockup where the size is the canvas's or its 1.5 times, the gate at every size, the spec and the grab;
/// a finding the test's excuses name is not counted (EmuSen_Kit.md §7.100).
pub fn hold(scene: &mut Scene<'_>, screen: &str, m: &mockup::Mockup, named: &mut Excuses, report: &mut Vec<String>) {
    hold_in(scene, screen, m, named, report, Variant::Dark);
}

#[allow(dead_code)]
/// As `hold`, the state drawn in a variant of its own, its palette and contrast judged in it.
pub fn hold_in(scene: &mut Scene<'_>, screen: &str, m: &mockup::Mockup, named: &mut Excuses, report: &mut Vec<String>, variant: Variant) {
    hold_walking(scene, screen, m, named, report, variant, &DIRECTIONS);
}

#[allow(dead_code)]
/// As `hold_in`, the gate's walk pressing `buttons` from every item: the shoulders too where the screen declares them.
pub fn hold_walking(scene: &mut Scene<'_>, screen: &str, m: &mockup::Mockup, named: &mut Excuses, report: &mut Vec<String>, variant: Variant, buttons: &[UiButton]) {
    for (w, h) in SIZES {
        scene.resize(w, h);
        scene.wait(150);
        let mut tree = scene.tree();
        let focus = scene.focus(&tree);
        let name = format!("{screen}-{w}x{h}");
        let mut faults: Vec<Finding> = Vec::new();
        if f64::from(w) / f64::from(h) == 1.6 {
            faults.extend(mockup::compare(m, &tree.nodes, f64::from(h) / 800.0, variant).iter().map(|d| Finding::mockup(d, &tree.nodes)));
        }
        assert!(scene.grab(&out().join(format!("{name}.png")).display().to_string()), "grab {name}");
        scene.sample_behind(&mut tree, f64::from(w), f64::from(h));
        theme_words(&mut tree.root);
        // What the screen showed and held before each press, kept beside the grab (EmuSen_Kit.md §7.103).
        let mut presses = Vec::new();
        let walk = scene.walk_pressing(&tree, buttons, |s, button| {
            let at = s.focus(&tree).map_or_else(|| "none".to_owned(), |i| tree.nodes[i as usize].name.clone());
            let (shown, keys) = (s.property("screens", "shown"), s.property("coresModel", "searchKeys"));
            presses.push(format!("shown={shown} coresKeys={keys} focus={at} press={button:?}"));
            s.pad(button);
        });
        std::fs::write(out().join(format!("{name}.presses.txt")), presses.join("\n")).unwrap();
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
                .map(|f| Finding::gate(f, &tree.nodes)),
        );
        let refused = named.sort(&name, &scene.exempted(&tree, WalkForm::BigScreen, variant, f64::from(h)));
        faults.extend(refused.iter().map(|e| Finding::exempted(e, &tree.nodes)));
        faults.retain(|f| !named.excuses(f, (w, h)));
        let notes = out().join(format!("{name}.faults.txt"));
        if faults.is_empty() {
            let _ = std::fs::remove_file(&notes);
        } else {
            let lines: Vec<&str> = faults.iter().map(|f| f.line.as_str()).collect();
            std::fs::write(&notes, format!("{}\n-- items:\n{}\n", lines.join("\n"), tree.legend())).unwrap();
            report.extend(faults.iter().map(|f| format!("{name}: {} [{}]", f.line, f.who())));
        }
    }
    scene.resize(1280, 800);
    scene.wait(100);
}

/// The theme's screen judged as theme-audit judges it: its words that are not the program's own (`words::is_own`) are
/// data, which may elide (R-20261111-44, EmuSen_Qt.md §3.7.2); words outside the theme host are the program's own, held whole.
fn theme_words(item: &mut emusen_kit_core::audit::Item) {
    if item.name == "reference" {
        item.mark_data(&emusen_qt_models::words::is_own);
        return;
    }
    item.children.iter_mut().for_each(theme_words);
}

#[allow(unused_imports)]
pub use emusen_qt_test::covers::draw as draw_covers;

#[allow(dead_code)]
/// The client fixture's console core, a PlayStation that starts empty, in the player's cores folder (Galaxia's), approved
/// by its hash on the configuration the program makes with `cores` as the installation's folder, as Cores approves one.
pub fn approved_console_core(cores: &std::path::Path) {
    let platform = Platform::load(library()).unwrap();
    let player = platform.directory(emusen_platform_client::galaxia::consts::DIR_CORES).unwrap().expect("the player's cores folder");
    std::fs::create_dir_all(&player).unwrap();
    let core = player.join(emusen_client_fixture::console_core().file_name().unwrap());
    std::fs::copy(emusen_client_fixture::console_core(), &core).unwrap();
    let config = emusen_qt_models::clients::runtime_config(&platform, cores.to_str().unwrap()).unwrap();
    let api = emusen_crystal_client::Api::load(library()).unwrap();
    let rt = emusen_crystal_client::Runtime::new(&api, &config).unwrap();
    rt.core_approve(&emusen_client_fixture::cores::sha256(&core)).unwrap_or_else(|e| panic!("{e}; the runtime lists {:?}", rt.cores()));
}

#[allow(dead_code)]
/// A clip of one colour, magenta, 64 × 48 at 30 frames a second for two seconds: a YUV4MPEG2 stream, the simplest a
/// test can write by hand, under the name the media store looks for. FFmpeg finds the format by its content.
pub fn write_clip(path: &std::path::Path) {
    let (w, h) = (64usize, 48usize);
    let mut bytes = b"YUV4MPEG2 W64 H48 F30:1 Ip A1:1 C420jpeg\n".to_vec();
    for _ in 0..60 {
        bytes.extend_from_slice(b"FRAME\n");
        bytes.extend(std::iter::repeat_n(107u8, w * h));
        bytes.extend(std::iter::repeat_n(202u8, w * h / 4));
        bytes.extend(std::iter::repeat_n(222u8, w * h / 4));
    }
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
}

/// One instant the program's clock is held at, in seconds since 1970 (Clock.now), for a test whose states must not
/// depend on the minute they are read in.
#[allow(dead_code)]
pub const HELD_NOW: &str = "1793966400";

#[allow(dead_code)]
/// What the program holds at a failure, for the message that reports it: the screen shown, each model's problem, and the
/// item holding the focus with those it sits in.
pub fn state(scene: &mut Scene<'_>) -> String {
    let problems: Vec<String> = ["library", "session", "pageModel", "settingsModel"].iter().map(|m| format!("{m} {:?}", scene.property(m, "problem"))).collect();
    format!("shown {:?}; problems: {}; the focus: {:?}", scene.property("screens", "shown"), problems.join(", "), console::focus_chain(scene))
}

#[allow(dead_code)]
/// `assert_still`, its failure saying what the program held then (`state`).
#[track_caller]
pub fn assert_still_saying(scene: &mut Scene<'_>, what: &str) {
    let still = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| scene.assert_still(what)));
    if let Err(failure) = still {
        let said = failure.downcast_ref::<String>().cloned().or_else(|| failure.downcast_ref::<&str>().map(|s| (*s).to_owned())).unwrap_or_default();
        panic!("{said}\nat the failure: {}", state(scene));
    }
}
