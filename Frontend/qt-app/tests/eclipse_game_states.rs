//! The states of Eclipse's game page that neither board draws, as the tester ruled them (R-20261123-33, -34, -36), in the
//! program (EmuSen_Qt.md §3.4.11): only a resume point; slots and no resume point; every slot full and the row following
//! the focus; Delete asking the program's question; a state gone since the page read it handed to the program as its own page
//! hands it; and every tab Eclipse does not name opening the program's page.

mod common;
mod eclipse_board;
#[path = "eclipse_board/game.rs"]
mod game;

use common::wait_for;
use cxx_qt::casting::Upcast;
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::input::UiButton;
use emusen_qt_models::game_page_model::qobject::GamePageModel;
use emusen_qt_models::library_model::qobject::QAbstractListModel;
use emusen_qt_test::Scene;
use game::{clear_states, focused, midnight, open_page, program, reopen, save_state, started, state_file, states_tab, when_words, world, RESUME};
use std::pin::Pin;

const DAY: i64 = 86_400;

fn model(scene: &mut Scene<'_>) -> Pin<&'static mut GamePageModel> {
    let object = scene.object("pageModel");
    assert!(!object.is_null(), "the page's model");
    // Safety: the window's model of that name is a GamePageModel, alive while the window is, and pinned where QML made it.
    unsafe { Pin::new_unchecked(&mut *(GamePageModel::from_base_ptr(object as *const QAbstractListModel) as *mut GamePageModel)) }
}

/// The drawn words of each card's name, in order, and of each badge.
fn cards(scene: &mut Scene<'_>) -> (Vec<String>, Vec<String>) {
    let tree = scene.tree();
    let text = |prefix: &str| -> Vec<String> {
        (0..12)
            .filter_map(|i| {
                tree.nodes.iter().rev().find(|n| n.name == format!("{prefix}.{i}") && n.ty.contains("Text") && n.visible).and_then(|n| n.text.clone())
            })
            .collect()
    };
    let mut names = text("cardName");
    if tree.nodes.iter().any(|n| n.name == "emptyName" && n.visible) {
        names.push(eclipse_board::read(&tree, "emptyName", "text"));
    }
    (names, text("cardBadgeWords"))
}

fn pills(scene: &mut Scene<'_>) -> Vec<String> {
    let tree = scene.tree();
    (0..3)
        .filter_map(|i| {
            ["buttonLabel", "pillLabel"]
                .iter()
                .find_map(|p| tree.nodes.iter().rev().find(|n| n.name == format!("{p}.{i}") && n.ty.contains("Text") && n.visible).and_then(|n| n.text.clone()))
        })
        .collect()
}

/// The program's question opened, its first answer focused, as the program's own page's tests wait for it.
fn question_opened(scene: &mut Scene<'_>) {
    wait_for(scene, "pageModel", "asking", "true", 2000);
    game::until(scene, "the program's question opened, its first answer focused", 3000, |s| {
        let tree = s.tree();
        s.property("pageQuestion", "opened") == "true" && s.focus(&tree).is_some_and(|i| tree.nodes[i as usize].name == "answer.0")
    });
}

fn help(scene: &mut Scene<'_>) -> Vec<String> {
    let tree = scene.tree();
    game::words_under(&tree, "help")
}

#[test]
fn qt_3_4_11_the_states_no_board_draws_are_as_the_tester_ruled() {
    let mut harness = emusen_qt_test::Harness::new();
    let today = midnight();
    // (1) Only a resume point, saved today: its card with the badge, then the empty slot; Resume leads (R-20261123-36).
    let resumed = (RESUME, words_now() - 60);
    let world = world("qt-app-eclipse-game-states", &[resumed], true, true);
    {
        let mut scene = program(&mut harness);
        open_page(&mut scene, &world);
        states_tab(&mut scene);
        assert_eq!(cards(&mut scene), (vec!["Resume point".to_owned(), "Empty slot".to_owned()], vec!["Resumes here".to_owned()]));
        assert_eq!(eclipse_board::read(&scene.tree(), "cardWhen.0", "text"), when_words(resumed.1), "today's form (R-20261123-34)");
        assert!(when_words(resumed.1).starts_with("Today, "));
        assert_eq!(pills(&mut scene), ["Resume", "Start from the beginning", "Favourite"]);
        assert_eq!(focused(&mut scene), "card.0");

        // (2) Slots and no resume point, one saved another year: no badge, and the never-played board's buttons (R-20261123-36).
        clear_states(&world);
        let old = (2, today - 400 * DAY + 19 * 3600 + 5 * 60);
        save_state(&world, (1, today - DAY + 20 * 3600 + 40 * 60));
        save_state(&world, old);
        reopen(&mut scene, &world);
        assert_eq!(cards(&mut scene), (vec!["Slot 1".to_owned(), "Slot 2".to_owned(), "Empty slot".to_owned()], Vec::new()));
        let year = eclipse_board::read(&scene.tree(), "cardWhen.1", "text");
        assert_eq!(year, when_words(old.1), "another year's form (R-20261123-34)");
        assert!(year.split(',').next().unwrap().split(' ').count() == 3, "the year written: {year}");
        assert_eq!(pills(&mut scene), ["Start", "Favourite"]);
        assert_eq!(help(&mut scene), ["Load this state", "Delete", "Back"], "a state's card");
        scene.pad(UiButton::Up);
        scene.wait(150);
        assert_eq!(focused(&mut scene), "pill.0");
        assert_eq!(help(&mut scene), ["Start", "Favourite", "Back", "Home"], "Start, in the never-played board's words");
        scene.pad(UiButton::Accept);
        assert_eq!(started(&mut scene), "", "Start starts with no state");
    }

    // (3) Every slot full and the resume point: nine cards at the board's size and no empty slot; the row follows the
    // focus sideways, a card the row has carried past either end not drawn (R-20261123-33).
    {
        let mut scene = program(&mut harness);
        save_state(&world, (RESUME, today - DAY + 21 * 3600 + 12 * 60));
        for slot in 1..=8 {
            save_state(&world, (slot, today - 2 * DAY - i64::from(slot) * 3600));
        }
        open_page(&mut scene, &world);
        states_tab(&mut scene);
        let (names, badges) = cards(&mut scene);
        assert!(!names.iter().any(|n| n == "Empty slot") && badges == ["Resumes here"], "{names:?} {badges:?}");
        for _ in 0..8 {
            scene.pad(UiButton::Right);
            scene.wait(120);
        }
        assert_eq!(focused(&mut scene), "card.8", "the last slot reached");
        let tree = scene.tree();
        let unit: f64 = eclipse_board::read(&tree, "gameCanvas", "unit").parse().unwrap();
        let k = DESIGN_HEIGHT / 1200.0;
        let drawn = |name: &str| tree.nodes.iter().rev().find(|n| n.name == name && n.ty != "QQuickText").map(|n| (n.visible, n.x / unit, n.w / unit));
        let (seen, x, w) = drawn("card.8").expect("the last card");
        assert!(seen && (w - 316.0 * k).abs() < 1e-6, "the last card drawn at the board's width: {seen} {w}");
        assert!(x + w <= 1280.0 / unit - 96.0 * k + 1e-6, "the last card inside the right margin: {}", x + w);
        assert!(drawn("card.0").map_or(true, |c| !c.0), "the first card carried past the left end is not drawn: gone or hidden");
        scene.pad(UiButton::Left);
        scene.wait(120);
        assert_eq!(focused(&mut scene), "card.7");

        // (4) Delete: X on a state's card asks the program's question, which the program draws; B keeps the state, and the
        // question's Delete removes it (R-20261123-36).
        let slot8 = state_file(&world, 8);
        scene.pad(UiButton::Right);
        scene.wait(120);
        scene.pad(UiButton::Screensaver);
        question_opened(&mut scene);
        assert_eq!(scene.property("theme", "pageDrawn"), "false", "the question is the program's");
        assert!(scene.property("pageModel", "questionTitle").starts_with("Delete Slot 8"), "{}", scene.property("pageModel", "questionTitle"));
        scene.pad(UiButton::Back);
        wait_for(&mut scene, "pageModel", "asking", "false", 2000);
        game::until(&mut scene, "Eclipse's screen again", 2000, |s| s.property("theme", "pageDrawn") == "true");
        assert!(slot8.exists(), "B keeps the state");
        game::until(&mut scene, "the focus on a card", 2000, |s| focused(s).starts_with("card."));
        while focused(&mut scene) != "card.8" {
            let before = focused(&mut scene);
            scene.pad(UiButton::Right);
            scene.wait(120);
            assert_ne!(focused(&mut scene), before, "the walk to slot 8");
        }
        scene.pad(UiButton::Screensaver);
        wait_for(&mut scene, "pageModel", "asking", "true", 2000);
        let choices: Vec<String> = model(&mut scene).question_choices().iter().map(|c| c.to_string()).collect();
        let delete = choices.iter().position(|c| c == "Delete").expect("Delete among the choices") as i32;
        assert!(model(&mut scene).answer(delete));
        game::until(&mut scene, "slot 8 deleted", 3000, |_| !slot8.exists());
        game::until(&mut scene, "slot 8's card gone and the empty slot last", 3000, |s| {
            let labels: Vec<String> = model(s).cards().iter().map(|c| c.label.clone()).collect();
            !labels.iter().any(|l| l == "Slot 8") && labels.last().is_some_and(|l| l == "Empty slot")
        });

        // (5) A state gone since the page read it: its card starts the game from it as the program's page does (R-20261123-36).
        let slot1 = state_file(&world, 1);
        reopen(&mut scene, &world);
        scene.pad(UiButton::Right);
        scene.wait(120);
        assert_eq!(eclipse_board::read(&scene.tree(), "cardName.1", "text"), "Slot 1");
        std::fs::remove_file(&slot1).unwrap();
        scene.pad(UiButton::Accept);
        assert_eq!(started(&mut scene), slot1.to_string_lossy(), "the card hands the program the state");
        // The program's notice, "Could not load the state, started from the beginning: it is gone", is the session's once a core
        // runs the game (qt-models/src/session_model.rs start_from); no core runs the synthetic image here, so it is not held.
    }

    // (6) Every tab Eclipse does not name opens the program's page; States is Eclipse's again (EmuSen_Qt.md §3.2, 1.11).
    let mut scene = program(&mut harness);
    open_page(&mut scene, &world);
    states_tab(&mut scene);
    let screen = game::theme_object(&mut scene, "gameScreen");
    for tab in [0, 2, 3, 4, 5, 1] {
        assert!(
            emusen_theme_qt::host::actions(eclipse_board::object(screen)).expect("Actions").page_tab(tab),
            "Actions.pageTab({tab}) taken on Eclipse's States"
        );
        game::until(&mut scene, &format!("tab {tab}"), 2000, |s| model(s).tab() == tab);
        let want = if tab == 1 { "true" } else { "false" };
        game::until(&mut scene, &format!("tab {tab} drawn by the right page"), 2000, |s| s.property("theme", "pageDrawn") == want);
        assert_eq!(scene.property("gamePage", "visible"), (tab != 1).to_string(), "tab {tab}: the program's page shown where Eclipse draws none");
        if tab != 1 {
            back_to_states(&mut scene);
        }
    }
    scene.assert_quiet();
}

fn words_now() -> i64 {
    emusen_qt_models::words::now()
}

/// From a tab the program draws back to States, Eclipse's, by the theme's own call while it is not shown: refused, so
/// through the program's page's shoulders.
fn back_to_states(scene: &mut Scene<'_>) {
    for _ in 0..8 {
        let tab = model(scene).tab();
        if tab == 1 && scene.property("theme", "pageDrawn") == "true" {
            return;
        }
        scene.pad(if tab == 5 {
            UiButton::Back
        } else if tab > 1 {
            UiButton::PageUp
        } else {
            UiButton::PageDown
        });
        scene.wait(200);
    }
    assert_eq!(scene.property("theme", "pageDrawn"), "true", "back to States");
}
