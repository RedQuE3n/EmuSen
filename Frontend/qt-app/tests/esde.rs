//! An ES-DE theme as the library's look (EmuSen_Qt.md §4.15): the program over a sandbox whose settings choose Art Book Next
//! from the corpus, its library two synthetic NES images and a synthetic Super Nintendo image, played by VenusRT built
//! from this commit with its sidecar (the client fixture's; no player's game and no published build is read). Moonrise
//! gives way to the system view; a NES game with no engine fails, said as Mistress said it; Right steps the carousel; A
//! opens the gamelist and A on the game plays it through the ring; the chord pauses it under the Pause Card, and Back to Library is
//! the gamelist at the same game, where the Home card, made under a pointer left resting since the List card, holds the focus on Continue and resumes it and its Tools ▸ Cores and ▸ Themes open those
//! screens and come back, the card, Cores and Themes held at the gate's five sizes over the theme; Start Console starts
//! the client fixture's PlayStation from the Home card by the pad and by the pointer, and the PlayStation's gamelist's
//! Select menu starts it by the pointer, its picture found on a grab of the window and its pad's B and Start turning
//! that picture blue, until Quit Game brings the theme back (EmuSen_Qt.md §7.9); Start is the Home card, Select
//! the List card, which sorts and filters the theme's gamelist, Y the favourite, each step's sound heard; a theme that is not there gives way to the own look with a notice; nothing is logged. On Vulkan in a compositor of the test's own, as the
//! acceptance runs. Run it with the ROM library bound read-only (`bwrap --ro-bind`), as the gates do.

mod common;

use common::console::{console_owns_the_pad, guide_alone, pause, quit_game, row_centre, row_focused, row_middles};
use common::{approved_console_core, hold, sandbox, wait_for};
use cxx_qt_lib::QString;

use emusen_client_fixture::cores::{installation_with_snes, snes_image};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt::{start, MAIN_URL};
use emusen_qt_play::host::Host;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::excuse::Excuses;
use emusen_qt_test::virtual_pad::Sdl;
use emusen_qt_test::{mockup, Harness, Scene};
use serde_json::json;
use std::path::PathBuf;

/// Art Book Next with the options the oracle and the C# baseline gave it (Testing/esde-baseline/states.tsv).
const THEME: &str = "art-book-next-es-de";
const CHOICES: [(&str, &str); 3] = [("Variant", "gamelist-list-metadata-cover"), ("ColorScheme", "dark-screenshots"), ("FontSize", "medium")];

fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap())
}


/// Waits up to `seconds` for a condition the scene answers, letting Qt run between looks; at the timeout it names where the program stands.
fn until(scene: &mut Scene<'_>, seconds: u64, what: &str, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    let start = std::time::Instant::now();
    while !done(scene) {
        if start.elapsed() >= std::time::Duration::from_secs(seconds) {
            let state = format!(
                "shown {}, session running {} paused {} problem {:?}, the pad's chord only {}",
                scene.property("screens", "shown"),
                scene.property("session", "running"),
                scene.property("session", "paused"),
                scene.property("session", "problem"),
                scene.pad_chord_only()
            );
            panic!("{what}: not within {seconds} s ({state})");
        }
        scene.wait(8);
    }
}

/// How many sounds the interface player was asked to play.
fn rings(scene: &mut Scene<'_>) -> i32 {
    scene.property("cues", "rings").parse().unwrap()
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
    let mut excuses = Excuses::none().listing("qt-app-esde");
    let theme = home().join(".cache/emusen/bigpicture/corpus/whole").join(THEME);
    assert!(theme.exists(), "the theme at {}", theme.display());
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

    // The library: two NES images and a Super Nintendo image the test makes; the installation's folder holds VenusRT
    // built from this commit with its sidecar, vouched for by its manifest, as a publish leaves it.
    let world = sandbox("qt-app-esde", &[("nes", "Brass Lantern.nes"), ("nes", "Cobalt Harbor.nes")]);
    std::fs::write(world.dir("Roms/snes").join("Aurora Drift.sfc"), snes_image("AURORA DRIFT")).unwrap();
    let cores = installation_with_snes(&world.dir("lib"));
    // The client fixture's console core, a PlayStation that starts empty, approved; and a PlayStation disc, so the theme
    // has the console's gamelist, whose Select menu offers it.
    approved_console_core(&cores);
    std::fs::write(world.dir("Roms/psx").join("Basalt Run.iso"), b"not a disc").unwrap();
    let mut settings = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    let folder = theme.to_str().unwrap();
    settings["LibraryStyle"] = json!("Theme");
    settings["BigPictureTheme"] = json!(folder);
    settings["BigPicture"] = json!({ folder: CHOICES.iter().map(|(k, v)| (k.to_string(), json!(v))).collect::<serde_json::Map<_, _>>() });
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    let read = start::read(library());
    assert!(read.esde && read.navigation_sounds, "the start reads the ES-DE theme as the library's look");
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
    // The window made with the start's own initial properties, as main.rs makes it; the library named after the cores.
    let mut initial = start::initial(&read);
    assert!(initial.remove(&QString::from("platformLibrary")));
    let mut scene = harness.load_initial(MAIN_URL, 1280, 800, &initial, |window| unsafe { adoption.adopt(window.cast()) }.expect("Qt adopts the device"));
    assert_eq!(scene.property("program", "esdeChosen"), "true", "the start's choice, as main.rs hands it in");
    assert!(scene.set_property("program", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("program", "feed", &host.feed.address().to_string()));
    assert!(scene.set_property("library", "cores", cores.to_str().unwrap()));
    // The splash skipped before the library is named: finished, Moonrise still stands, the theme not yet drawn.
    scene.pad(UiButton::Accept);
    until(&mut scene, 5, "the splash skipped", |s| s.property("moonrise", "finished") == "true");
    // `done` is a binding, so it is read as `finished` turns: no wait is needed for it to have gone true.
    assert_eq!(scene.property("moonrise", "done"), "false", "Moonrise stays while the theme is not drawn");
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));

    // Moonrise, then the theme's system view in place of Home, the first system selected; the own look's screens unmade.
    wait_for(&mut scene, "library", "loading", "false", 20000);
    // Moonrise goes only once the theme is drawn (`lookSettled`): never with the theme still loading behind it.
    let mut settled = None;
    for _ in 0..1250 {
        if scene.property("moonrise", "done") == "true" {
            settled = Some(scene.property("esdeModel", "themed"));
            break;
        }
        scene.wait(16);
    }
    assert_eq!(settled.as_deref(), Some("true"), "Moonrise gave way with the theme drawn");
    until(&mut scene, 20, "Moonrise giving way to the theme", |s| s.property("moonrise", "visible") == "false");
    assert_eq!(scene.property("screens", "shown"), "Esde");
    assert_eq!((scene.property("esdeModel", "view"), scene.property("esdeModel", "problem")), ("system".into(), String::new()));
    assert_eq!(scene.property("esdeModel", "shelf"), "nes", "the systems in release order");
    assert_eq!(scene.property("chosenTheme", "active"), "false");
    for slot in ["home.reference", "consoles.reference", "games.reference"] {
        assert_eq!(scene.property(slot, "active"), "false", "the own look's {slot} is not made");
    }
    until(&mut scene, 10, "the theme's sounds decoded", |s| s.property("cues", "problem").is_empty() && s.property("esdeModel", "sounds").contains("select"));
    // Each sound the view asks for is played: the theme's file where it gave one, else the kit's cue (cues.rs).
    let sounds: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&scene.property("esdeModel", "sounds")).unwrap();
    let themed: Vec<String> = sounds.keys().cloned().collect();
    let key = |name: &str| emusen_qt::cues::key_of(name, &themed).unwrap();
    let heard = |s: &mut Scene<'_>, name: &str, before: i32| {
        until(s, 3, &format!("the {name} sound"), |s| rings(s) > before);
        assert_eq!(s.property("cues", "rung"), key(name), "the {name} sound");
    };

    // A game that fails to open, a NES image with no engine for it here: said in Mistress's form, the theme back in place
    // of a game screen with nothing on it (EmuSen_Qt.md §7.9); B goes back to the systems.
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "esdeModel", "view", "gamelist", 1000);
    until(&mut scene, 5, "the slide at rest", |s| s.property("esdeModel", "moving") == "false");
    scene.pad(UiButton::Accept);
    until(&mut scene, 20, "the failure said", |s| s.property("notices", "shown") == "true" && s.property("notices", "words").starts_with("Failed to load Brass Lantern: "));
    assert_eq!(scene.property("notices", "attention"), "true");
    until(&mut scene, 5, "the theme back", |s| s.property("screens", "shown") == "Esde" && s.property("session", "running") == "false");
    until(&mut scene, 10, "the notice gone", |s| s.property("notices", "shown") == "false");
    scene.pad(UiButton::Back);
    until(&mut scene, 5, "the systems again", |s| s.property("esdeModel", "view") == "system" && s.property("esdeModel", "moving") == "false");

    // Right steps the carousel; A opens the gamelist on the game.
    let before = rings(&mut scene);
    scene.pad(UiButton::Right);
    wait_for(&mut scene, "esdeModel", "shelf", "snes", 1000);
    heard(&mut scene, "systembrowse", before);
    until(&mut scene, 5, "the step at rest", |s| s.property("esdeModel", "moving") == "false");
    let before = rings(&mut scene);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "esdeModel", "view", "gamelist", 1000);
    heard(&mut scene, "select", before);
    until(&mut scene, 5, "the slide at rest", |s| s.property("esdeModel", "moving") == "false");
    let selected = scene.property("esdeModel", "selected");

    // Y turns the game a favourite through the library.
    assert!(!scene.property("library", "esdeShelves").contains("\"favourite\":true"));
    let before = rings(&mut scene);
    scene.pad(UiButton::Search);
    until(&mut scene, 10, "the favourite written and read again", |s| s.property("library", "esdeShelves").contains("\"favourite\":true"));
    heard(&mut scene, "favorite", before);

    // Start opens the Home card over the theme and B closes it; Select opens the List card and B closes it.
    scene.pad(UiButton::Menu);
    until(&mut scene, 5, "the Home card in place", |s| s.property("screens", "shown") == "HomeCard" && s.property("homeCardCanvas", "slide") == "0");
    assert_eq!(scene.property("esde", "visible"), "true", "the theme under the card");
    scene.pad(UiButton::Back);
    until(&mut scene, 5, "the Home card closed to the theme", |s| s.property("screens", "shown") == "Esde" && s.property("homeCardCanvas", "visible") == "false");
    scene.pad(UiButton::Options);
    until(&mut scene, 5, "the List card in place", |s| s.property("screens", "shown") == "List" && s.property("listCardCanvas", "slide") == "0");
    // The pointer moves over the card's second row and is left resting there through the game's session, so the cards
    // that open after it are made under a resting pointer (EmuSen_Kit.md §7.107, §7.108).
    let first = focused_words(&mut scene);
    let middles = row_middles(&mut scene);
    scene.hover(middles[1].0, middles[1].1 - 4.0);
    scene.hover(middles[1].0, middles[1].1);
    until(&mut scene, 5, "the moving pointer focusing the second row", |s| focused_words(s) != first);
    scene.pad(UiButton::Back);
    until(&mut scene, 5, "the List card closed to the theme", |s| s.property("screens", "shown") == "Esde" && s.property("listCardCanvas", "visible") == "false");
    assert_eq!((scene.property("esdeModel", "view"), scene.property("esdeModel", "selected")), ("gamelist".into(), selected.clone()));

    // A on the game plays it through the ring, its first picture drawn.
    let before = rings(&mut scene);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Game", 2000);
    heard(&mut scene, "launch", before);
    until(&mut scene, 20, "the game running", |s| {
        let problem = s.property("session", "problem");
        assert!(problem.is_empty() || problem.starts_with("The game plays without sound"), "the session: {problem}");
        s.property("session", "running") == "true"
    });
    until(&mut scene, 20, "the first picture", |s| s.property("session", "firstPictureMs").parse::<f64>().is_ok_and(|ms| ms > 0.0));
    assert_eq!(scene.property("esde", "visible"), "false", "the theme hidden under the game");

    // Guide alone pauses it under the Pause Card; Leave, Back to Library is the gamelist at the same game.
    assert!(scene.pad_chord_only(), "the game owns the pad");
    scene.run_pad_clock();
    guide_alone(&mut scene, &sdl, &pad);
    until(&mut scene, 10, "the chord pausing the game under the Pause Card", |s| s.property("session", "paused") == "true" && s.property("screens", "shown") == "Pause");
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

    // Start's Home card leads with Continue over the suspended game, held at the gate's five sizes; A resumes the game.
    scene.pad(UiButton::Menu);
    until(&mut scene, 5, "the Home card in place", |s| s.property("screens", "shown") == "HomeCard" && s.property("homeCardCanvas", "slide") == "0");
    assert!(focused_words(&mut scene).starts_with("Continue"), "the focus on Continue: {}", focused_words(&mut scene));
    // Made under the resting pointer, the card holds the focus: Down moves it and Up brings it back to Continue.
    scene.pad(UiButton::Down);
    until(&mut scene, 5, "Down moving the focus on the card", |s| !focused_words(s).starts_with("Continue") && common::console::focus_within(s, "homeCardCanvas"));
    scene.pad(UiButton::Up);
    until(&mut scene, 5, "Up back on Continue", |s| focused_words(s).starts_with("Continue"));
    let mut report = Vec::new();
    hold(&mut scene, "EsdeHomeCardContinue", &mockup::Mockup::default(), &mut excuses, &mut report);
    scene.resize(1280, 800);
    scene.wait(150);
    scene.pad(UiButton::Accept);
    until(&mut scene, 5, "Continue resuming the game", |s| s.property("screens", "shown") == "Game" && s.property("session", "paused") == "false");
    until(&mut scene, 5, "the game owning the pad again", |s| s.pad_chord_only());
    scene.run_pad_clock();
    guide_alone(&mut scene, &sdl, &pad);
    until(&mut scene, 10, "Guide pausing it again", |s| s.property("session", "paused") == "true" && s.property("screens", "shown") == "Pause");
    until(&mut scene, 5, "the card in place", |s| s.property("cardCanvas", "slide") == "0");
    choose_row(&mut scene, "Leave");
    wait_for(&mut scene, "pause", "title", "Leave", 1000);
    choose_row(&mut scene, "Back to Library");
    until(&mut scene, 5, "the theme again", |s| s.property("screens", "shown") == "Esde" && s.property("cardCanvas", "visible") == "false");
    until(&mut scene, 5, "the theme holding the pad", |s| s.property("esde", "activeFocus") == "true");

    // Tools ▸ Cores, chosen by the pointer, opens Cores, and B comes back to the theme; Tools ▸ Themes, by the pad, the
    // Themes tab, and B the same; each held at the gate's five sizes over the theme.
    for (tool, by_pointer, screen) in [("Cores", true, "Cores"), ("Themes", false, "Themes")] {
        scene.pad(UiButton::Menu);
        until(&mut scene, 5, "the Home card in place", |s| s.property("screens", "shown") == "HomeCard" && s.property("homeCardCanvas", "slide") == "0");
        choose_row(&mut scene, "Tools");
        until(&mut scene, 5, "the Tools page", |s| s.property("homeCardModel", "page") == "tools");
        if by_pointer {
            let row = row_centre(&mut scene, tool);
            scene.click(row.0, row.1);
        } else {
            choose_row(&mut scene, tool);
        }
        until(&mut scene, 5, "the tool's screen", |s| s.property("screens", "shown") == screen);
        scene.wait(300);
        hold(&mut scene, &format!("Esde{screen}"), &mockup::Mockup::default(), &mut excuses, &mut report);
        scene.pad(UiButton::Back);
        until(&mut scene, 5, "B back to the theme", |s| s.property("screens", "shown") == "Esde");
    }

    until(&mut scene, 5, "the theme holding the pad after the tools", |s| s.property("esde", "activeFocus") == "true");
    // A on the suspended game resumes it; the chord, Leave, Quit Game and Yes close it, and the theme comes back.
    scene.pad(UiButton::Accept);
    until(&mut scene, 10, "the game resumed", |s| s.property("screens", "shown") == "Game" && s.property("session", "paused") == "false");
    until(&mut scene, 5, "the game owning the pad again", |s| s.pad_chord_only());
    scene.run_pad_clock();
    guide_alone(&mut scene, &sdl, &pad);
    until(&mut scene, 10, "the chord pausing the game again", |s| s.property("screens", "shown") == "Pause");
    until(&mut scene, 5, "the card in place", |s| s.property("cardCanvas", "slide") == "0");
    choose_row(&mut scene, "Leave");
    wait_for(&mut scene, "pause", "title", "Leave", 1000);
    choose_row(&mut scene, "Quit Game");
    choose_row(&mut scene, "Yes");
    until(&mut scene, 10, "the game closed to the theme", |s| s.property("session", "running") == "false" && s.property("screens", "shown") == "Esde");

    // The List card under the theme acts on the theme's gamelist alone: B to the systems, Left to the NES, A on its gamelist.
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "esdeModel", "view", "system", 2000);
    until(&mut scene, 5, "the slide at rest", |s| s.property("esdeModel", "moving") == "false");
    // With NavigationSounds off a step plays nothing.
    assert!(scene.set_property("program", "navigationSounds", "false"));
    let before = rings(&mut scene);
    scene.pad(UiButton::Left);
    wait_for(&mut scene, "esdeModel", "shelf", "nes", 1000);
    until(&mut scene, 5, "the step at rest", |s| s.property("esdeModel", "moving") == "false");
    assert_eq!(rings(&mut scene), before, "no sound with NavigationSounds off");
    assert!(scene.set_property("program", "navigationSounds", "true"));
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "esdeModel", "view", "gamelist", 1000);
    // The sounds are rung in order on the GUI thread, so a ring of the step taken with them off would be counted before A's.
    until(&mut scene, 3, "the select sound", |s| rings(s) > before);
    assert_eq!(rings(&mut scene), before + 1, "A's sound alone: the step taken with NavigationSounds off rang nothing");
    until(&mut scene, 5, "the slide at rest", |s| s.property("esdeModel", "moving") == "false");
    let nes = |s: &mut Scene<'_>| -> Vec<String> {
        let shelves: serde_json::Value = serde_json::from_str(&s.property("library", "esdeShelves")).unwrap();
        let shelf = shelves.as_array().unwrap().iter().find(|x| x["name"] == "nes").unwrap().clone();
        shelf["games"].as_array().unwrap().iter().map(|g| g["name"].as_str().unwrap_or_default().to_owned()).collect()
    };
    assert_eq!(nes(&mut scene), ["Brass Lantern", "Cobalt Harbor"]);
    assert_eq!(scene.property("esdeModel", "selected"), "0");
    // Down to the second entry, so that the sort's answer, the first entry, is a selection the act has to make.
    scene.pad(UiButton::Down);
    until(&mut scene, 5, "the second entry selected", |s| s.property("esdeModel", "selected") == "1" && s.property("esdeModel", "moving") == "false");
    let own = (scene.property("gridGames", "listFacts"), scene.property("gridGames", "focused"), scene.property("gridGames", "count"));
    assert!(!own.0.is_empty() && !own.2.is_empty(), "the own grid read by its name: {own:?}");
    // Select, Down to Sort, A steps it to name, descending: the new order shown, the selection on the first entry the act answered.
    scene.pad(UiButton::Options);
    until(&mut scene, 5, "the List card in place", |s| s.property("screens", "shown") == "List" && s.property("listCardCanvas", "slide") == "0");
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    until(&mut scene, 10, "the gamelist sorted again", |s| nes(s) == ["Cobalt Harbor", "Brass Lantern"]);
    scene.pad(UiButton::Back);
    until(&mut scene, 5, "the List card closed to the theme", |s| s.property("screens", "shown") == "Esde" && s.property("listCardCanvas", "visible") == "false");
    until(&mut scene, 5, "the view at rest on the new order", |s| s.property("esdeModel", "moving") == "false");
    assert_eq!(scene.property("esdeModel", "selected"), "0", "the entry the act answered, moved to from the second");
    let tree = scene.tree();
    // The theme draws its words glyph by glyph; a reader hears them whole on the item that draws them.
    let top = |name: &str| tree.nodes.iter().filter(|n| n.visible && n.heard == name).map(|n| n.y).reduce(f64::min);
    let (cobalt, brass) = (top("Cobalt Harbor"), top("Brass Lantern"));
    assert!(matches!((cobalt, brass), (Some(c), Some(b)) if c < b), "the view lists both, Cobalt Harbor first: {cobalt:?} {brass:?}");
    // Y makes Cobalt Harbor a favourite; Down to Brass Lantern; Filter, Favourites leaves Cobalt Harbor alone in the
    // gamelist, the selection moved back to it by the act.
    scene.pad(UiButton::Search);
    until(&mut scene, 10, "the favourite read again", |s| s.property("library", "esdeShelves").matches("\"favourite\":true").count() >= 2);
    scene.pad(UiButton::Down);
    until(&mut scene, 5, "Brass Lantern selected", |s| s.property("esdeModel", "selected") == "1" && s.property("esdeModel", "moving") == "false");
    scene.pad(UiButton::Options);
    until(&mut scene, 5, "the List card in place", |s| s.property("screens", "shown") == "List" && s.property("listCardCanvas", "slide") == "0");
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    choose_row(&mut scene, "Favourites");
    until(&mut scene, 10, "the favourites alone", |s| nes(s) == ["Cobalt Harbor"] && s.property("screens", "shown") == "Esde");
    until(&mut scene, 5, "the view at rest on the favourites", |s| s.property("esdeModel", "moving") == "false");
    assert_eq!(scene.property("esdeModel", "selected"), "0", "the entry the act answered, moved to from the second");
    assert_eq!((scene.property("gridGames", "listFacts"), scene.property("gridGames", "focused"), scene.property("gridGames", "count")), own, "the own look's listing untouched");

    // A sounds text that does not read is said as a notice.
    assert!(scene.set_property("cues", "sounds", "{"));
    until(&mut scene, 5, "the sounds' problem as a notice", |s| s.property("notices", "words").contains("sounds could not be read"));

    // Start Console by the pad: the Home card's row and its page, A on the PlayStation; the console's screen is shown and
    // owns the pad; the Pause Card's Quit Game closes it to the theme.
    scene.pad(UiButton::Menu);
    until(&mut scene, 5, "the Home card in place", |s| s.property("screens", "shown") == "HomeCard" && s.property("homeCardCanvas", "slide") == "0");
    choose_row(&mut scene, "Start Console");
    until(&mut scene, 5, "Start Console's page", |s| s.property("homeCardModel", "page") == emusen_qt_models::home_card::START_CONSOLE);
    row_focused(&mut scene, "PlayStation");
    choose_row(&mut scene, "PlayStation");
    console_owns_the_pad(&mut scene, &sdl, &pad, "PlayStation");
    assert_eq!(scene.property("esde", "visible"), "false", "the theme hidden under the console");
    quit_game(&mut scene, &sdl, &pad, "PlayStation (no disc)", "Esde");

    // Start Console by the pointer: the Home card's row clicked, then the PlayStation's; Quit Game to the theme again.
    scene.pad(UiButton::Menu);
    until(&mut scene, 5, "the Home card in place", |s| s.property("screens", "shown") == "HomeCard" && s.property("homeCardCanvas", "slide") == "0");
    let (x, y) = row_centre(&mut scene, "Start Console");
    scene.click(x, y);
    until(&mut scene, 5, "Start Console's page", |s| s.property("homeCardModel", "page") == emusen_qt_models::home_card::START_CONSOLE);
    row_focused(&mut scene, "PlayStation");
    let (x, y) = row_centre(&mut scene, "PlayStation");
    scene.click(x, y);
    console_owns_the_pad(&mut scene, &sdl, &pad, "PlayStation");
    quit_game(&mut scene, &sdl, &pad, "PlayStation (no disc)", "Esde");

    // The PlayStation's gamelist: Select's menu leads with Start PlayStation With No Disc, which the pointer chooses.
    scene.pad(UiButton::Back);
    until(&mut scene, 5, "the systems", |s| s.property("esdeModel", "view") == "system" && s.property("esdeModel", "moving") == "false");
    for _ in 0..4 {
        if scene.property("esdeModel", "shelf") == "psx" {
            break;
        }
        scene.pad(UiButton::Right);
        until(&mut scene, 5, "the step at rest", |s| s.property("esdeModel", "moving") == "false");
    }
    assert_eq!(scene.property("esdeModel", "shelf"), "psx");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "esdeModel", "view", "gamelist", 1000);
    until(&mut scene, 5, "the slide at rest", |s| s.property("esdeModel", "moving") == "false");
    scene.pad(UiButton::Options);
    until(&mut scene, 5, "the List card in place", |s| s.property("screens", "shown") == "List" && s.property("listCardCanvas", "slide") == "0");
    hold(&mut scene, "EsdeListCardStartConsole", &mockup::Mockup::default(), &mut excuses, &mut report);
    let (x, y) = row_centre(&mut scene, "Start PlayStation With No Disc");
    let top = row_middles(&mut scene)[0].1;
    assert_eq!(y, top, "the console's row leads the menu");
    scene.click(x, y);
    console_owns_the_pad(&mut scene, &sdl, &pad, "PlayStation");
    // Back to Library, the Home card's Continue names the console's session, held at the five sizes, and resumes it.
    pause(&mut scene, &sdl, &pad, "PlayStation (no disc)");
    choose_row(&mut scene, "Leave");
    wait_for(&mut scene, "pause", "title", "Leave", 1000);
    choose_row(&mut scene, "Back to Library");
    until(&mut scene, 5, "Back to Library showing the theme", |s| s.property("screens", "shown") == "Esde" && s.property("cardCanvas", "visible") == "false");
    scene.pad(UiButton::Menu);
    until(&mut scene, 5, "the Home card in place", |s| s.property("screens", "shown") == "HomeCard" && s.property("homeCardCanvas", "slide") == "0");
    assert!(focused_words(&mut scene).starts_with("Continue"), "the focus on Continue: {}", focused_words(&mut scene));
    assert!(scene.tree().nodes.iter().any(|n| n.visible && n.text.as_deref() == Some("PlayStation (no disc)")), "Continue names the console's session");
    hold(&mut scene, "EsdeHomeCardContinueConsole", &mockup::Mockup::default(), &mut excuses, &mut report);
    scene.pad(UiButton::Accept);
    until(&mut scene, 5, "Continue resuming the console", |s| s.property("screens", "shown") == "Game" && s.property("session", "paused") == "false");
    quit_game(&mut scene, &sdl, &pad, "PlayStation (no disc)", "Esde");
    assert!(report.is_empty(), "{} faults:\n{}", report.len(), report.join("\n"));

    // A theme that is not there gives way to the own look, with the loader's words as a notice.
    settings["BigPictureTheme"] = json!(world.root.join("no-such-theme").to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    assert!(scene.set_property("library", "platformLibrary", ""));
    assert!(scene.set_property("library", "platformLibrary", library().to_str().unwrap()));
    until(&mut scene, 20, "the own look in place of the missing theme", |s| s.property("screens", "shown") == "Home");
    assert!(scene.property("esdeModel", "problem").contains("theme.xml"), "{}", scene.property("esdeModel", "problem"));
    wait_for(&mut scene, "notices", "shown", "true", 3000);

    // The theme again, then an empty library: a load with nothing to show gives way to the own look, with no problem.
    settings["BigPictureTheme"] = json!(folder);
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    assert!(scene.set_property("library", "platformLibrary", ""));
    assert!(scene.set_property("library", "platformLibrary", library().to_str().unwrap()));
    until(&mut scene, 20, "the theme back", |s| s.property("screens", "shown") == "Esde");
    // Every game the sandbox holds, the PlayStation's disc with them, so the theme has no shelf left to show.
    for (folder, file) in [("nes", "Brass Lantern.nes"), ("nes", "Cobalt Harbor.nes"), ("snes", "Aurora Drift.sfc"), ("psx", "Basalt Run.iso")] {
        std::fs::remove_file(world.dir(&format!("Roms/{folder}")).join(file)).unwrap();
    }
    assert!(scene.set_property("library", "platformLibrary", ""));
    assert!(scene.set_property("library", "platformLibrary", library().to_str().unwrap()));
    let no_game = |s: &mut Scene<'_>| {
        let shelves: serde_json::Value = serde_json::from_str(&s.property("library", "esdeShelves")).unwrap_or_default();
        shelves.as_array().is_some_and(|all| all.iter().all(|x| x["games"].as_array().is_none_or(|g| g.is_empty())))
    };
    until(&mut scene, 20, "the library read with no game on any shelf", |s| s.property("library", "loading") == "false" && no_game(s));
    until(&mut scene, 20, "the own look in place of the empty theme", |s| s.property("screens", "shown") == "Home");
    assert_eq!((scene.property("esdeModel", "themed"), scene.property("esdeModel", "problem")), ("false".into(), String::new()));

    scene.assert_quiet();
    sdl.detach(pad);
    assert!(scene.set_property("program", "visible", "false"));
    scene.wait(50);
    drop(harness);
    drop(host);
}
