//! Start Console end to end in the own look (EmuSen_Qt.md §7.9), on Vulkan in a compositor of the test's own, as the
//! acceptance runs: the client fixture's console core, a PlayStation that starts empty, approved in the player's cores
//! folder. The grid's List card offers Start Console; its page names the console, its engine and its firmware line, A
//! says Start, and the page is held at the gate's five sizes; the pointer starts the console, whose picture a grab of
//! the window finds drawn and whose pad's B and Start reach it as themselves; the chord's Pause Card names the session
//! as Mistress named it, its Resume plays on and its Quit Game closes it to Home; then the pad starts it from the same
//! page. Start on Home opens the Home card at its first page (decided 2026-11-11), whose Start Console row the pad and
//! then the pointer take to the console; Back to Library, and the Home card's Continue names the console's session, is
//! held at the five sizes and resumes it. The ES-DE theme's walk is esde.rs's. A GPU job.

mod common;

use common::console::{back_to_library, choose_row, console_owns_the_pad, focused_words, grid_ready, pause, quit_game, row_centre, row_focused, row_shown, until};
use common::{approved_console_core, hold, sandbox, wait_for};

use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_play::host::Host;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::virtual_pad::Sdl;
use emusen_qt_test::{mockup, Harness};

/// No finding is named for the tester: the gate holds every state whole.
fn nothing_named(_: &str, _: &emusen_qt_test::tree::Tree, _: (i32, i32)) -> bool {
    false
}

#[test]
fn qt_7_9_start_console_starts_a_console_whose_screen_owns_the_pad() {
    unsafe { std::env::set_var("SDL_AUDIODRIVER", "dummy") };
    let kwin = Compositor::start(3840, 2160);
    kwin.watch(600);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    let mut harness = Harness::new();
    let platform = Platform::load(library()).unwrap();
    let gpu = std::env::var("EMUSEN_GPU_DEVICE").ok();
    let host = Host::new(&platform, "wayland", gpu.as_deref(), false, &std::env::temp_dir().join("qt-app-console-presets")).expect("a Vulkan device");
    let adoption = host.adoption();

    let world = sandbox("qt-app-console", &[("nes", "Brass Lantern.nes")]);
    let cores = world.dir("lib");
    approved_console_core(&cores);

    let sdl = Sdl::open();
    let pad = sdl.attach("EmuSen Test Xbox Pad", 0x045e, 0x028e);
    let mut scene = harness.load_prepared(MAIN_URL, 1280, 800, |window| unsafe { adoption.adopt(window.cast()) }.expect("Qt adopts the device"));
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("clock", "now", common::HELD_NOW));
    assert!(scene.set_property("program", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("program", "feed", &host.feed.address().to_string()));
    assert!(scene.set_property("library", "cores", cores.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 20000);
    until(&mut scene, 10, "the console listed", |s| s.property("library", "startable").contains("\"PlayStation\""));
    until(&mut scene, 20, "the virtual pad read", |s| s.property("pads", "connected") == "1");
    let mut report = Vec::new();

    // The grid's Select opens the List card, whose Start Console turns to its page: the console by its name with its
    // engine, the pane's firmware line, A's word Start; held at the five sizes.
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    list_card_start_console(&mut scene);
    assert_eq!(scene.property("listCardModel", "help"), emusen_qt_models::help_model::shown_screen(emusen_qt_models::help::Screen::CardStart).to_string());
    assert!(scene.tree().nodes.iter().any(|n| n.visible && n.text.as_deref() == Some("Start")), "A's word is Start");
    let shown: Vec<String> = scene.tree().nodes.iter().filter(|n| n.visible).filter_map(|n| n.text.clone()).collect();
    assert!(shown.iter().any(|t| t.starts_with("Starts a PlayStation with no disc in it, on V1ConsoleCore")), "the pane's sentence: {shown:?}");
    assert!(shown.iter().any(|t| t == "✗ Not found: add it in Settings → Firmware"), "the pane's firmware line: {shown:?}");
    hold(&mut scene, "ListCardStartConsole", &mockup::Mockup::default(), nothing_named, &mut report);

    // The pointer's click on the console starts it; its screen is shown and owns the pad.
    let (x, y) = row_centre(&mut scene, "PlayStation");
    scene.click(x, y);
    console_owns_the_pad(&mut scene, &sdl, &pad, "PlayStation");

    // The chord's Pause Card names it "PlayStation (no disc)"; its Resume plays on; the Pause Card's Quit Game closes it,
    // and the library comes back.
    pause(&mut scene, &sdl, &pad, "PlayStation (no disc)");
    choose_row(&mut scene, "Resume");
    until(&mut scene, 5, "Resume playing on", |s| s.property("screens", "shown") == "Game" && s.property("session", "paused") == "false");
    quit_game(&mut scene, &sdl, &pad, "PlayStation (no disc)", "Home");

    // The pad: the List card's Start Console again, A on the PlayStation; Quit Game to Home again.
    list_card_start_console(&mut scene);
    choose_row(&mut scene, "PlayStation");
    console_owns_the_pad(&mut scene, &sdl, &pad, "PlayStation");
    quit_game(&mut scene, &sdl, &pad, "PlayStation (no disc)", "Home");

    // Start on Home opens the Home card at its first page; its Start Console row by the pad, A on the PlayStation.
    home_card(&mut scene);
    let labels: Vec<String> = scene.tree().nodes.iter().filter(|n| n.visible && n.name == "label").filter_map(|n| n.text.clone()).collect();
    assert!(row_shown(&mut scene, "Start Console"), "the first page offers Start Console: {labels:?}; startable {}", scene.property("library", "startable"));
    choose_row(&mut scene, "Start Console");
    until(&mut scene, 5, "Start Console's page", |s| s.property("homeCardModel", "page") == emusen_qt_models::home_card::START_CONSOLE);
    row_focused(&mut scene, "PlayStation");
    choose_row(&mut scene, "PlayStation");
    console_owns_the_pad(&mut scene, &sdl, &pad, "PlayStation");

    // Back to Library is Home; Start's Home card leads with Continue naming the console's session, held at the five
    // sizes, and A resumes it.
    back_to_library(&mut scene, &sdl, &pad, "PlayStation (no disc)", "Home");
    home_card(&mut scene);
    assert!(focused_words(&mut scene).starts_with("Continue"), "the focus on Continue: {}", focused_words(&mut scene));
    assert!(scene.tree().nodes.iter().any(|n| n.visible && n.text.as_deref() == Some("PlayStation (no disc)")), "Continue names the console's session");
    hold(&mut scene, "HomeCardContinueConsole", &mockup::Mockup::default(), nothing_named, &mut report);
    scene.pad(UiButton::Accept);
    until(&mut scene, 5, "Continue resuming the console", |s| s.property("screens", "shown") == "Game" && s.property("session", "paused") == "false");
    console_owns_the_pad(&mut scene, &sdl, &pad, "PlayStation");
    quit_game(&mut scene, &sdl, &pad, "PlayStation (no disc)", "Home");

    // The pointer: the Home card's Start Console row clicked, then the PlayStation's; Quit Game to Home.
    home_card(&mut scene);
    let (x, y) = row_centre(&mut scene, "Start Console");
    scene.click(x, y);
    until(&mut scene, 5, "Start Console's page", |s| s.property("homeCardModel", "page") == emusen_qt_models::home_card::START_CONSOLE);
    row_focused(&mut scene, "PlayStation");
    let (x, y) = row_centre(&mut scene, "PlayStation");
    scene.click(x, y);
    console_owns_the_pad(&mut scene, &sdl, &pad, "PlayStation");
    quit_game(&mut scene, &sdl, &pad, "PlayStation (no disc)", "Home");

    assert!(report.is_empty(), "{} faults:\n{}", report.len(), report.join("\n"));
    scene.assert_quiet();
    sdl.detach(pad);
    assert!(scene.set_property("program", "visible", "false"));
    scene.wait(50);
    drop(harness);
    drop(host);
}

/// Home's focus on Games, as Home gives it.
fn games_place(scene: &mut emusen_qt_test::Scene<'_>) {
    until(scene, 5, "Home's Games holding the focus", |s| {
        let tree = s.tree();
        tree.find_named("place.0").is_some_and(|games| s.focus(&tree) == Some(games) || s.focus_item(&tree, games))
    });
}

/// From Home: Games, All Games, the grid, Select's List card and its Start Console page, on the PlayStation's row.
fn list_card_start_console(scene: &mut emusen_qt_test::Scene<'_>) {
    games_place(scene);
    scene.pad(UiButton::Accept);
    wait_for(scene, "screens", "shown", "Consoles", 1000);
    let tree = scene.tree();
    let all = tree.find_named("tile.0").expect("All Games");
    assert!(scene.focus_item(&tree, all));
    scene.pad(UiButton::Accept);
    wait_for(scene, "screens", "shown", "Grid", 1000);
    grid_ready(scene);
    scene.pad(UiButton::Options);
    until(scene, 5, "the List card in place", |s| s.property("screens", "shown") == "List" && s.property("listCardCanvas", "slide") == "0");
    choose_row(scene, "Start Console");
    until(scene, 5, "Start Console's page", |s| s.property("listCardModel", "page") == emusen_qt_models::list_card::START_CONSOLE);
    row_focused(scene, "PlayStation");
}

/// Start on Home: the Home card at its first page, in place.
fn home_card(scene: &mut emusen_qt_test::Scene<'_>) {
    until(scene, 5, "Home holding the focus", |s| s.property("screens", "shown") == "Home" && s.property("home", "activeFocus") == "true");
    scene.pad(UiButton::Menu);
    until(scene, 5, "the Home card in place", |s| s.property("screens", "shown") == "HomeCard" && s.property("homeCardCanvas", "slide") == "0");
    assert_eq!(scene.property("homeCardModel", "page"), emusen_qt_models::home_card::FIRST, "Start opens its first page");
    until(scene, 5, "the card holding the focus", |s| common::console::focus_within(s, "homeCardCanvas"));
}
