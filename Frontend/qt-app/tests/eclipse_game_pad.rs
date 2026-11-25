//! Every pill, tab, link and card of Eclipse's game page reached and acted on, by the pad and by the pointer, in the program
//! (EmuSen_Qt.md §3.4.12): the walk between them, Favourite turned, the tabs stepped by the shoulders and tapped, the trail's
//! links, and Resume, Start from the beginning and a card each starting the game from its state.

mod common;
mod eclipse_board;
#[path = "eclipse_board/game.rs"]
mod game;

use common::wait_for;
use cxx_qt::casting::Upcast;
use emusen_qt_models::game_page_model::qobject::GamePageModel;
use emusen_qt_models::library_model::qobject::QAbstractListModel;
use emusen_kit_core::input::UiButton;
use emusen_qt_test::{Harness, Scene};
use game::{board_states, centre, favourite, focused, open_page, program, started, state_file, states_tab, world, RESUME};

/// The program under Eclipse with the game's page at its States tab, the resume point's card focused.
fn page<'a>(harness: &'a mut Harness, world: &emusen_client_fixture::World) -> Scene<'a> {
    let mut scene = program(harness);
    open_page(&mut scene, world);
    states_tab(&mut scene);
    assert_eq!(focused(&mut scene), "card.0");
    scene
}

fn press(scene: &mut Scene<'_>, button: UiButton) -> String {
    scene.pad(button);
    scene.wait(120);
    focused(scene)
}

fn tap(scene: &mut Scene<'_>, name: &str) {
    let (x, y) = centre(scene, name);
    scene.click(x, y);
    scene.wait(200);
}

/// The tab the page's model shows, by its place among the board's, and whether Eclipse draws it.
fn shown(scene: &mut Scene<'_>) -> (i32, bool) {
    let object = scene.object("pageModel");
    assert!(!object.is_null(), "the page's model");
    // Safety: the window's model of that name is a GamePageModel, alive while the window is.
    let page = unsafe { &*GamePageModel::from_base_ptr(object as *const QAbstractListModel) };
    (page.tab(), scene.property("theme", "pageDrawn") == "true")
}

#[test]
fn qt_3_4_12_every_pill_tab_link_and_card_is_reached_and_acted_on_by_pad_and_pointer() {
    let mut harness = Harness::new();
    let world = world("qt-app-eclipse-game-pad", &board_states(), true, true);
    let file = |slot| state_file(&world, slot).to_string_lossy().into_owned();

    // The pad's walk: along the cards to the last state (the empty slot is never reached), up to the buttons, along them,
    // up to the trail's links, along them, and down again to the buttons and the cards.
    {
        let mut scene = page(&mut harness, &world);
        let walked: Vec<String> = [UiButton::Right, UiButton::Right, UiButton::Right, UiButton::Right, UiButton::Left, UiButton::Up, UiButton::Right, UiButton::Right,
            UiButton::Right, UiButton::Up, UiButton::Left, UiButton::Left, UiButton::Down, UiButton::Down]
            .into_iter()
            .map(|b| press(&mut scene, b))
            .collect();
        assert_eq!(
            walked,
            ["card.1", "card.2", "card.3", "card.3", "card.2", "pill.0", "pill.1", "pill.2", "pill.2", "trailLink.1", "trailLink.0", "trailLink.0", "pill.0", "card.0"],
            "the pad's walk"
        );
        // Favourite turned off and on by A, saved at once and drawn.
        for (b, on) in [(UiButton::Up, true), (UiButton::Right, true), (UiButton::Right, true)] {
            assert_eq!(favourite(&world), on);
            press(&mut scene, b);
        }
        assert_eq!(focused(&mut scene), "pill.2");
        scene.pad(UiButton::Accept);
        game::until(&mut scene, "Favourite turned off", 2000, |_| !favourite(&world));
        game::until(&mut scene, "the heart drawn empty", 2000, |s| eclipse_board::read(&s.tree(), "pill.2", "on") == "false");
        scene.pad(UiButton::Accept);
        game::until(&mut scene, "Favourite turned on again", 2000, |_| favourite(&world));
        // The shoulders step all six Eclipse tabs in board order: States -> Media -> Details -> Options -> Cheats, then back to Overview -> States.
        assert_eq!(shown(&mut scene), (1, true), "States, Eclipse's");
        scene.pad(UiButton::PageDown);
        game::until(&mut scene, "RB opening Media, Eclipse's", 2000, |s| shown(s) == (2, true));
        scene.pad(UiButton::PageDown);
        game::until(&mut scene, "RB opening Details, Eclipse's", 2000, |s| shown(s) == (3, true));
        scene.pad(UiButton::PageDown);
        game::until(&mut scene, "RB opening Options, Eclipse's", 2000, |s| shown(s) == (4, true));
        scene.pad(UiButton::PageDown);
        game::until(&mut scene, "RB opening Cheats, Eclipse's", 2000, |s| shown(s) == (5, true));
        tap(&mut scene, "tab.1");
        game::until(&mut scene, "tapping States back to States, Eclipse's", 2000, |s| shown(s) == (1, true));
        scene.pad(UiButton::PageUp);
        game::until(&mut scene, "LB opening Overview, Eclipse's", 2000, |s| shown(s) == (0, true));
        scene.pad(UiButton::PageDown);
        game::until(&mut scene, "RB back to States", 2000, |s| shown(s) == (1, true));
        // A on slot 1's card starts the game from slot 1.
        game::until(&mut scene, "focus on card.0", 2000, |s| focused(s) == "card.0");
        assert_eq!(press(&mut scene, UiButton::Right), "card.1", "along to slot 1");
        scene.pad(UiButton::Accept);
        assert_eq!(started(&mut scene), file(1), "slot 1's card loads slot 1");
    }
    // A on Resume loads the resume point, not the most recent state (R-20261123-35); A on Start from the beginning starts
    // with no state.
    for (pill, want) in [(0, file(RESUME)), (1, String::new())] {
        let mut scene = page(&mut harness, &world);
        press(&mut scene, UiButton::Up);
        for _ in 0..pill {
            press(&mut scene, UiButton::Right);
        }
        assert_eq!(focused(&mut scene), format!("pill.{pill}"));
        scene.pad(UiButton::Accept);
        assert_eq!(started(&mut scene), want, "pill {pill} by A");
    }
    // The trail's console link by the pad opens the console's games; by the pointer, Consoles opens Choose a console.
    {
        let mut scene = page(&mut harness, &world);
        press(&mut scene, UiButton::Up);
        assert_eq!(press(&mut scene, UiButton::Up), "trailLink.1");
        scene.pad(UiButton::Accept);
        wait_for(&mut scene, "screens", "shown", "Grid", 3000);
        assert_eq!(scene.property("screens", "key"), "snes", "the game's console's games");
    }
    {
        let mut scene = page(&mut harness, &world);
        tap(&mut scene, "trailLink.0");
        wait_for(&mut scene, "screens", "shown", "Consoles", 3000);
    }
    {
        let mut scene = page(&mut harness, &world);
        tap(&mut scene, "trailLink.1");
        wait_for(&mut scene, "screens", "shown", "Grid", 3000);
        assert_eq!(scene.property("screens", "key"), "snes", "the game's console's games");
    }
    // The pointer: Favourite tapped off and on, every tab tapped (the program's page for each but States), and a card
    // tapped, which starts the game from its state.
    {
        let mut scene = page(&mut harness, &world);
        tap(&mut scene, "pill.2");
        game::until(&mut scene, "Favourite tapped off", 2000, |_| !favourite(&world));
        tap(&mut scene, "pill.2");
        game::until(&mut scene, "Favourite tapped on", 2000, |_| favourite(&world));
        for tab in [0, 2, 3, 4, 5] {
            tap(&mut scene, &format!("tab.{tab}"));
            game::until(&mut scene, &format!("tab {tab} tapped"), 2000, |s| shown(s) == (tab, true));
            states_back(&mut scene);
        }
        tap(&mut scene, "tab.1");
        assert_eq!(shown(&mut scene), (1, true), "States tapped stays Eclipse's");
        tap(&mut scene, "card.2");
        assert_eq!(started(&mut scene), file(2), "slot 2's card tapped loads slot 2");
    }
    for (pill, want) in [(0, file(RESUME)), (1, String::new())] {
        let mut scene = page(&mut harness, &world);
        tap(&mut scene, &format!("pill.{pill}"));
        assert_eq!(started(&mut scene), want, "pill {pill} tapped");
    }
}

/// From any tab back to States by tapping or stepping through the program's page.
fn states_back(scene: &mut Scene<'_>) {
    for _ in 0..8 {
        let (tab, drawn) = shown(scene);
        if (tab, drawn) == (1, true) {
            return;
        }
        if drawn {
            tap(scene, "tab.1");
            return;
        }
        // Cheats is Options' deeper page, which B leaves for Options.
        scene.pad(if tab == 5 { UiButton::Back } else if tab > 1 { UiButton::PageUp } else { UiButton::PageDown });
        scene.wait(200);
    }
    assert_eq!(shown(scene), (1, true), "back to States");
}
