//! Eclipse's games view drawn on the graphics card, for the tester's pictures (EmuSen_Qt.md §3.4.10): with
//! `ECLIPSE_GAMES_PICTURES` naming a folder, the view over the covers' library, or with `ECLIPSE_GAMES_EMPTY` set the empty
//! library's board, saved there at 1280x800 and 1920x1200 through Vulkan in a compositor of the test's own. A card job:
//! run it through gpu.sh. With no folder named it stands down with a printed line.

mod common;

use common::{draw_covers, sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_models::themes;
use emusen_qt_test::compositor::Compositor;

/// The covers drawn, a tall, a wide and a square, as eclipse_games.rs draws them: constants of this test.
const COVERS: [(&str, (u32, u32)); 3] = [("Aurora Drift", (300, 420)), ("Amber Relay", (420, 300)), ("Basalt Kings", (360, 360))];
const PLAIN: [&str; 7] = ["Cinder Mile", "Cobalt Harbor", "Dune Signal", "Ember Circuit", "Fathom Nine", "Granite Pass", "Harbor Lights"];

#[test]
fn qt_3_4_10_eclipses_games_view_on_the_card_for_the_pictures() {
    let Some(dir) = std::env::var_os("ECLIPSE_GAMES_PICTURES").map(std::path::PathBuf::from) else {
        println!("ECLIPSE_GAMES_PICTURES names no folder: no pictures taken");
        return;
    };
    let empty = std::env::var_os("ECLIPSE_GAMES_EMPTY").is_some();
    let kwin = Compositor::start(3840, 2160);
    kwin.watch(600);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    let files: Vec<String> = if empty { Vec::new() } else { COVERS.iter().map(|(t, _)| *t).chain(PLAIN).map(|t| format!("{t}.sfc")).collect() };
    let games: Vec<(&str, &str)> = files.iter().map(|f| ("SNES", f.as_str())).collect();
    let world = sandbox("qt-app-eclipse-games-gpu", &games);
    let platform = Platform::load(library()).unwrap();
    let media = world.dir("Media");
    let mut harness = emusen_qt_test::Harness::new();
    if !empty {
        let pictures: Vec<_> = COVERS.iter().enumerate().map(|(i, (t, size))| (media.join("snes/covers").join(format!("{t}.png")), *size, i, t.to_string())).collect();
        draw_covers(&mut harness, &world.root, &pictures, |_| {});
        let mut settings = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
        settings["EsdeMediaDirectory"] = serde_json::json!(media.to_str().unwrap());
        platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    }
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(!emusen_qt_test::ffi::ffi::harness_software(), "drawn on the card");
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "coverStores", media.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "screens", "shown", "Home", 3000);
    for _ in 0..40 {
        let tree = scene.tree();
        if tree.find_named("place.0").is_some_and(|games| scene.focus(&tree) == Some(games)) {
            break;
        }
        scene.wait(50);
    }
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Consoles", 2000);
    let tree = scene.tree();
    assert!(scene.focus_item(&tree, tree.find_named("tile.0").expect("All games")));
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);
    assert!(scene.set_property("program", "theme", &themes::built_in_folder("eclipse")));
    wait_for(&mut scene, "host.theme", "ready", "true", 3000);
    scene.wait(500);
    std::fs::create_dir_all(&dir).unwrap();
    let state = if empty { "empty" } else { "games" };
    for (w, h) in [(1280, 800), (1920, 1200)] {
        scene.resize(w, h);
        scene.wait(400);
        assert!(scene.grab(&dir.join(format!("{state}-card-{w}x{h}.png")).display().to_string()));
    }
}
