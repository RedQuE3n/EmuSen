//! The program's splash against Mistress's own (EmuSen_Kit.md §7.17), in a process of its own, as each harness wants.

use emusen_client_fixture::{library, World};
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_crystal_client::Config;
use emusen_qt::MAIN_URL;
use emusen_qt_models::clients::ClientSource;
use emusen_qt_models::library::{world_line_of, LibrarySource};
use emusen_qt_models::words::splash_caption;
use emusen_qt_test::Harness;

fn wait_for(scene: &mut emusen_qt_test::Scene<'_>, object: &str, property: &str, value: &str, ms: i32) {
    let mut waited = 0;
    while scene.property(object, property) != value && waited < ms {
        scene.wait(50);
        waited += 50;
    }
    assert_eq!(scene.property(object, property), value, "{object}.{property} after {ms} ms");
}

/// A scratch tree whose settings name its ROM folder, holding one synthetic NES image.
fn sandbox(name: &str) -> World {
    let world = World::new(name);
    let roms = world.dir("Roms/NES");
    let mut image = b"NES\x1a\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
    image.resize(16 + 16384, 0);
    std::fs::write(roms.join("Synthetic.nes"), image).unwrap();
    let platform = Platform::load(library()).unwrap();
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = serde_json::json!(world.root.join("Roms").to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    world
}

/// The courts canvas's consoles, as Mistress's picture states them (MoonrisePictureTool.Consoles).
const CONSOLES: [&str; 11] = ["nes", "snes", "n64", "gb", "gbc", "sms", "md", "mcd", "psx", "ps2", "dreamcast"];

/// The program's finished splash against Mistress's own with the canvas's eleven consoles (EmuSen_Kit.md §7.17): each
/// console's world from this commit's systems table, in its order, whatever engines the build has. The test holds the
/// splash up, as the C# picture holds it with start-up work never done.
#[test]
fn the_finished_splash_is_mistress_s_with_each_console_s_body() {
    let _world = sandbox("qt-app-splash");
    let source = ClientSource::open(library().to_str().unwrap(), &Config::default()).unwrap();
    let worlds: Vec<String> =
        source.systems().unwrap().iter().filter(|s| CONSOLES.contains(&s.id.as_str())).map(world_line_of).collect();
    assert_eq!(worlds.len(), CONSOLES.len(), "every console of the canvas is in the table");
    let mut harness = Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 720);
    assert!(scene.set_property("moonrise", "opacity", "1"));
    assert!(scene.set_property("moonrise", "worlds", &worlds.join("\n")));
    assert!(scene.set_property("moonrise", "caption", &splash_caption(worlds.len())));
    assert!(scene.set_property("moonrise", "running", "true"));
    scene.wait(100);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "moonrise", "finished", "true", 1000);
    scene.wait(200);
    // The loose box round the caption lets a digit through, so its words are held as the C# picture shows them.
    assert_eq!(scene.property("moonrise", "caption"), "11 CONSOLES");
    let tree = scene.tree();
    let grabs = std::env::temp_dir();
    assert!(scene.grab(&grabs.join("qt-app-splash.png").display().to_string()));
    let marks = grabs.join("qt-app-splash-difference.png").display().to_string();
    let faults = emusen_qt_test::moonrise::against(&mut scene, &tree, emusen_qt_test::moonrise::BODIES_REFERENCE, emusen_qt_test::moonrise::BODIES_SPARE, &marks);
    assert!(faults.is_empty(), "{}", faults.join("\n"));
    scene.assert_quiet();
}
