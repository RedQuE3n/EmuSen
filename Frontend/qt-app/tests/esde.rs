//! An ES-DE theme as the library's look (EmuSen_Qt.md §4.9): the program over a sandbox whose settings choose Art Book Next
//! from the corpus, its library two synthetic NES images and a real Super Nintendo game read in place from the player's
//! library (`EMUSEN_ESDE_GAME`, else Super Mario World), its engines the published installation's (`EMUSEN_ESDE_CORES`,
//! else `out/linux-x64/Mistress/lib/EmuSen`). Moonrise gives way to the system view; Right steps the carousel; A opens the
//! gamelist and A on the game plays it through the ring; the chord pauses it under the Pause Card, and Back to Library is
//! the gamelist at the same game; Start is the Home card, Select the List card, Y the favourite; a theme that is not there
//! gives way to the own look with a notice; nothing is logged. On Vulkan in a compositor of the test's own, as the
//! acceptance runs. Run it with the ROM library bound read-only (`bwrap --ro-bind`), as the gates do.

mod common;

use common::{sandbox, wait_for};

use emusen_client_fixture::library;
use emusen_kit_core::input::{PadControl, UiButton};
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt::{start, MAIN_URL};
use emusen_qt_play::host::Host;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::virtual_pad::Sdl;
use emusen_qt_test::{Harness, Scene};
use serde_json::json;
use std::path::PathBuf;

/// Art Book Next with the options the oracle and the C# baseline gave it (Testing/esde-baseline/states.tsv).
const THEME: &str = "art-book-next-es-de";
const CHOICES: [(&str, &str); 3] = [("Variant", "gamelist-list-metadata-cover"), ("ColorScheme", "dark-screenshots"), ("FontSize", "medium")];

fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap())
}

fn named_or(variable: &str, default: PathBuf) -> PathBuf {
    std::env::var_os(variable).map(PathBuf::from).unwrap_or(default)
}

/// Waits up to `seconds` for a condition the scene answers, letting Qt run between looks.
fn until(scene: &mut Scene<'_>, seconds: u64, what: &str, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    let start = std::time::Instant::now();
    while !done(scene) {
        assert!(start.elapsed() < std::time::Duration::from_secs(seconds), "{what}: not within {seconds} s");
        scene.wait(8);
    }
}

/// The words of the item holding the focus.
fn focused_words(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    let Some(focus) = scene.focus(&tree) else { return String::new() };
    tree.nodes.iter().skip(focus as usize).find_map(|n| n.text.clone()).unwrap_or_default()
}

/// Down on the Pause Card until the focused row reads `words`, then A.
fn choose_row(scene: &mut Scene<'_>, words: &str) {
    for _ in 0..12 {
        if focused_words(scene).starts_with(words) {
            scene.pad(UiButton::Accept);
            return;
        }
        scene.pad(UiButton::Down);
    }
    panic!("no row reads {words}; the focus reads {}", focused_words(scene));
}

#[test]
fn an_esde_theme_stands_for_the_library_and_a_game_comes_back_to_its_gamelist() {
    let theme = home().join(".cache/emusen/bigpicture/corpus/whole").join(THEME);
    let game = named_or("EMUSEN_ESDE_GAME", home().join("Documents/Roms/SNES/SMW.smc"));
    let cores = named_or("EMUSEN_ESDE_CORES", emusen_qt_test::lint::frontend().join("../../out/linux-x64/Mistress/lib/EmuSen"));
    for (what, path) in [("the theme", &theme), ("the game", &game), ("the installation's engines", &cores)] {
        assert!(path.exists(), "{what} at {}", path.display());
    }
    // Set before SDL is first loaded: the interface player and the game play to no speaker.
    unsafe { std::env::set_var("SDL_AUDIODRIVER", "dummy") };
    let kwin = Compositor::start(3840, 2160);
    kwin.watch(600);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    let mut harness = Harness::new();
    let platform = Platform::load(library()).unwrap();
    let gpu = std::env::var("EMUSEN_GPU_DEVICE").ok();
    let host = Host::new(&platform, "wayland", gpu.as_deref(), false, &std::env::temp_dir().join("qt-app-esde-presets")).expect("a Vulkan device");
    let adoption = host.adoption();

    // The library: two NES images and the real game, linked into the sandbox's folder and read where it lies.
    let world = sandbox("qt-app-esde", &[("nes", "Brass Lantern.nes"), ("nes", "Cobalt Harbor.nes")]);
    let snes = world.dir("Roms/snes").join("Super Mario World.smc");
    std::os::unix::fs::symlink(&game, &snes).unwrap();
    let mut settings = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    let folder = theme.to_str().unwrap();
    settings["LibraryStyle"] = json!("Theme");
    settings["BigPictureTheme"] = json!(folder);
    settings["BigPicture"] = json!({ folder: CHOICES.iter().map(|(k, v)| (k.to_string(), json!(v))).collect::<serde_json::Map<_, _>>() });
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    let read = start::read(library());
    assert!(read.esde && read.navigation_sounds, "the start reads the ES-DE theme as the library's look");

    let sdl = Sdl::open();
    let pad = sdl.attach("EmuSen Test Xbox Pad", 0x045e, 0x028e);
    let mut scene = harness.load_prepared(MAIN_URL, 1280, 800, |window| unsafe { adoption.adopt(window.cast()) }.expect("Qt adopts the device"));
    assert!(scene.set_property("program", "esdeChosen", "true"));
    assert!(scene.set_property("program", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("program", "feed", &host.feed.address().to_string()));
    assert!(scene.set_property("library", "cores", cores.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));

    // Moonrise, then the theme's system view in place of Home, the first system selected; the own look's screens unmade.
    wait_for(&mut scene, "library", "loading", "false", 20000);
    until(&mut scene, 20, "Moonrise giving way to the theme", |s| s.property("moonrise", "visible") == "false");
    assert_eq!(scene.property("screens", "shown"), "Esde");
    assert_eq!((scene.property("esdeModel", "view"), scene.property("esdeModel", "problem")), ("system".into(), String::new()));
    assert_eq!(scene.property("esdeModel", "shelf"), "nes", "the systems in release order");
    assert_eq!(scene.property("chosenTheme", "active"), "false");
    for slot in ["home.reference", "consoles.reference", "games.reference"] {
        assert_eq!(scene.property(slot, "active"), "false", "the own look's {slot} is not made");
    }
    until(&mut scene, 10, "the theme's sounds decoded", |s| s.property("cues", "problem").is_empty() && s.property("esdeModel", "sounds").contains("select"));

    // Right steps the carousel; A opens the gamelist on the game.
    scene.pad(UiButton::Right);
    wait_for(&mut scene, "esdeModel", "shelf", "snes", 1000);
    until(&mut scene, 5, "the step at rest", |s| s.property("esdeModel", "moving") == "false");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "esdeModel", "view", "gamelist", 1000);
    until(&mut scene, 5, "the slide at rest", |s| s.property("esdeModel", "moving") == "false");
    let selected = scene.property("esdeModel", "selected");

    // Y turns the game a favourite through the library.
    assert!(!scene.property("library", "esdeShelves").contains("\"favourite\":true"));
    scene.pad(UiButton::Search);
    until(&mut scene, 10, "the favourite written and read again", |s| s.property("library", "esdeShelves").contains("\"favourite\":true"));

    // Start opens the Home card over the theme and B closes it; Select opens the List card and B closes it.
    scene.pad(UiButton::Menu);
    until(&mut scene, 5, "the Home card in place", |s| s.property("screens", "shown") == "HomeCard" && s.property("homeCardCanvas", "slide") == "0");
    assert_eq!(scene.property("esde", "visible"), "true", "the theme under the card");
    scene.pad(UiButton::Back);
    until(&mut scene, 5, "the Home card closed to the theme", |s| s.property("screens", "shown") == "Esde" && s.property("homeCardCanvas", "visible") == "false");
    scene.pad(UiButton::Options);
    until(&mut scene, 5, "the List card in place", |s| s.property("screens", "shown") == "List" && s.property("listCardCanvas", "slide") == "0");
    scene.pad(UiButton::Back);
    until(&mut scene, 5, "the List card closed to the theme", |s| s.property("screens", "shown") == "Esde" && s.property("listCardCanvas", "visible") == "false");
    assert_eq!((scene.property("esdeModel", "view"), scene.property("esdeModel", "selected")), ("gamelist".into(), selected.clone()));

    // A on the game plays it through the ring, its first picture drawn.
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Game", 2000);
    until(&mut scene, 20, "the game running", |s| {
        let problem = s.property("session", "problem");
        assert!(problem.is_empty() || problem.starts_with("The game plays without sound"), "the session: {problem}");
        s.property("session", "running") == "true"
    });
    until(&mut scene, 20, "the first picture", |s| s.property("session", "firstPictureMs").parse::<f64>().is_ok_and(|ms| ms > 0.0));
    assert_eq!(scene.property("esde", "visible"), "false", "the theme hidden under the game");

    // The chord pauses it under the Pause Card; Leave, Back to Library is the gamelist at the same game.
    let guide = emusen_qt_endymion::reading::sdl_button(PadControl::Guide).unwrap();
    assert!(scene.pad_chord_only(), "the game owns the pad");
    scene.run_pad_clock();
    sdl.press(&pad, guide, true);
    until(&mut scene, 10, "the chord pausing the game under the Pause Card", |s| s.property("session", "paused") == "true" && s.property("screens", "shown") == "Pause");
    sdl.press(&pad, guide, false);
    until(&mut scene, 5, "the card in place", |s| s.property("cardCanvas", "slide") == "0");
    choose_row(&mut scene, "Leave");
    wait_for(&mut scene, "pause", "title", "Leave", 1000);
    choose_row(&mut scene, "Back to Library");
    until(&mut scene, 5, "Back to Library showing the theme", |s| s.property("screens", "shown") == "Esde" && s.property("cardCanvas", "visible") == "false");
    assert_eq!(scene.property("session", "paused"), "true", "the game kept suspended");
    assert_eq!(
        (scene.property("esdeModel", "view"), scene.property("esdeModel", "shelf"), scene.property("esdeModel", "selected")),
        ("gamelist".into(), "snes".into(), selected),
        "the gamelist at the same game"
    );
    assert_eq!(scene.property("esde", "activeFocus"), "true", "the theme takes the pad again");
    assert!(scene.set_property("game", "game", ""));
    until(&mut scene, 10, "the game closed to the theme", |s| s.property("session", "running") == "false" && s.property("screens", "shown") == "Esde");

    // A theme that is not there gives way to the own look, with the loader's words as a notice.
    settings["BigPictureTheme"] = json!(world.root.join("no-such-theme").to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    assert!(scene.set_property("library", "platformLibrary", ""));
    assert!(scene.set_property("library", "platformLibrary", library().to_str().unwrap()));
    until(&mut scene, 20, "the own look in place of the missing theme", |s| s.property("screens", "shown") == "Home");
    assert!(scene.property("esdeModel", "problem").contains("theme.xml"), "{}", scene.property("esdeModel", "problem"));
    wait_for(&mut scene, "notices", "shown", "true", 3000);

    scene.assert_quiet();
    sdl.detach(pad);
    assert!(scene.set_property("program", "visible", "false"));
    scene.wait(50);
    drop(harness);
    drop(host);
}
