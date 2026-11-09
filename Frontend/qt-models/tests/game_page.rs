//! A game's page without Qt (EmuSen_Qt.md §7.8): the facts line, the actions and Resume's stepping, the states as the
//! design orders them, the Options rows and their stepping, the doors, and the help bar by the focus.

use emusen_qt_models::game_page::*;
use emusen_qt_models::help::Screen;

const NOW: i64 = 1_762_700_000;

fn read() -> Read {
    let state = |slot, ago: i64, path: &str| State { path: path.into(), slot, picture: String::new(), when: NOW - ago };
    Read {
        title: "Cobalt Harbor (Synthetic)".into(),
        console: "SNES".into(),
        year: "1994".into(),
        developer: "Lantern Works".into(),
        publisher: "Publisher P".into(),
        states: vec![
            state(Slot::Auto, 12 * 60, "auto"),
            state(Slot::Number(1), 2 * 3600, "s1"),
            state(Slot::Number(2), 26 * 3600, "s2"),
            state(Slot::Number(3), 3 * 86_400, "s3"),
        ],
        engines: vec![
            Choice { value: None, text: "Venus (default)".into() },
            Choice { value: Some("venus".into()), text: "Venus (the console's)".into() },
            Choice { value: Some("snes9x".into()), text: "Snes9x".into() },
        ],
        cheats: Some((1, 3)),
        crop: Some(false),
        package: "Not a package".into(),
        ..Read::default()
    }
}

#[test]
fn qt_7_8_the_facts_line_and_the_actions_are_mistresss() {
    let page = GamePage::new(read(), NOW);
    assert_eq!(page.facts(), "SNES · 1994 · Lantern Works · Publisher P · Never played");
    let mut played = read();
    played.played_seconds = 3 * 3600 + 12 * 60;
    played.last_played = Some(NOW - 3 * 86_400);
    played.publisher = "Lantern Works".into();
    assert_eq!(GamePage::new(played, NOW).facts(), "SNES · 1994 · Lantern Works · 3 h 12 min · Last played 3 days ago");
    let actions = page.actions();
    assert_eq!(actions.iter().map(|a| a.label).collect::<Vec<_>>(), ["Play", "Resume", "Favourite"]);
    assert_eq!(actions[1].value, "Auto · 12 min ago");
    assert!(actions[0].filled && !actions[1].filled);
    assert_eq!(page.act(0), Outcome::Play(None));
    assert_eq!(page.act(1), Outcome::Play(Some("auto".into())));
    assert_eq!(page.act(2), Outcome::Changed(Change::Favourite));
    // A game with no state has no Resume.
    let mut none = read();
    none.states.clear();
    assert_eq!(GamePage::new(none, NOW).actions().len(), 2);
}

#[test]
fn qt_7_8_resume_steps_its_states_then_lets_the_focus_move_on() {
    let mut page = GamePage::new(read(), NOW);
    assert!(!page.step_action(1, -1), "Left at the first state moves to Play");
    for expected in ["Slot 1 · 2 h ago", "Slot 2 · yesterday", "Slot 3 · 3 days ago"] {
        assert!(page.step_action(1, 1));
        assert_eq!(page.actions()[1].value, expected);
    }
    assert!(!page.step_action(1, 1), "Right at the last state moves to Favourite");
    assert!(!page.step_action(0, 1), "Play does not step");
    assert_eq!(page.act(1), Outcome::Play(Some("s3".into())));
}

#[test]
fn qt_7_8_states_stand_slot_by_slot_with_auto_after_the_last_in_use() {
    let mut page = GamePage::new(read(), NOW);
    page.choose_section(1);
    let Content::Tiles(tiles) = page.content() else { panic!("tiles") };
    let words: Vec<String> = tiles.iter().take(6).map(|t| format!("{} | {}", t.label, t.line)).collect();
    assert_eq!(
        words,
        ["Slot 1 | 2 h ago", "Slot 2 | Yesterday", "Slot 3 | 3 days ago", "Auto | Where you left off · 12 min ago", "Slot 4 | Empty", "Slot 5 | Empty"]
    );
    assert_eq!(tiles.len(), 9, "eight slots and Auto");
    assert!(tiles[3].marked, "the state Resume loads is marked");
    assert_eq!(page.choose_tile(0), Outcome::Play(Some("s1".into())));
    assert_eq!(page.choose_tile(4), Outcome::Nothing, "an empty slot loads nothing");
    assert_eq!(page.help(None, 0), Screen::GamePageLoad);
    assert_eq!(page.help(None, 4), Screen::GamePageNone);
}

#[test]
fn qt_7_8_options_rows_step_the_engine_and_open_their_doors() {
    let mut page = GamePage::new(read(), NOW);
    page.choose_section(2);
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    let words: Vec<String> = rows.iter().map(|r| format!("{} | {}", r.label, r.value)).collect();
    assert_eq!(words, ["Engine | Venus (default)", "Cheats | 1 of 3 on", "Display crop | Console's", "Package | Not a package"]);
    assert!(rows[0].steps && rows[1].opens);
    assert_eq!(page.step_row(0, 1), Outcome::Changed(Change::Engine(Some("venus".into()))));
    assert_eq!(page.step_row(0, -1), Outcome::Changed(Change::Engine(Some("snes9x".into()))), "stepping wraps");
    assert_eq!(page.choose_row(1), Outcome::Door(CHEATS));
    assert_eq!(page.choose_row(2), Outcome::Door(CROP));
    assert_eq!(page.choose_row(0), Outcome::Nothing, "A on a stepped row does nothing");
    assert_eq!(page.help(None, 0), Screen::GamePageValue);
    assert_eq!(page.help(None, 1), Screen::GamePageChange);
    // One engine runs the console: the row names it and steps nothing; no overscan, no crop row.
    let mut one = read();
    one.engines = vec![Choice { value: None, text: "Venus (the only one)".into() }];
    one.crop = None;
    one.cheats = None;
    let mut page = GamePage::new(one, NOW);
    page.choose_section(2);
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    assert_eq!(rows.iter().map(|r| r.label).collect::<Vec<_>>(), ["Engine", "Package"]);
    assert!(!rows[0].steps);
}

#[test]
fn qt_7_8_the_overview_media_and_details() {
    let mut r = read();
    r.hero = "/m/fanart.png".into();
    r.media = vec![Media { path: "/m/shot.png".into(), kind: MediaKind::Screenshot }, Media { path: "/m/clip.mkv".into(), kind: MediaKind::Clip }];
    let mut page = GamePage::new(r, NOW);
    assert_eq!(page.content(), Content::Text { words: NO_DESCRIPTION.into(), picture: "/m/fanart.png".into() });
    assert_eq!(page.help(Some(0), 0), Screen::GamePage);
    assert_eq!(page.help(Some(1), 0), Screen::GamePageResumeSlots);
    assert_eq!(page.help(Some(2), 0), Screen::GamePageFavourite);
    page.choose_section(3);
    let Content::Tiles(tiles) = page.content() else { panic!("tiles") };
    assert_eq!(tiles.iter().map(|t| t.label.as_str()).collect::<Vec<_>>(), ["Scraped screenshot", "Clip"]);
    assert_eq!(tiles[1].picture, "/m/fanart.png", "a clip's tile shows the hero's picture");
    assert_eq!(page.help(None, 0), Screen::GamePageView);
    page.choose_section(4);
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    assert_eq!(rows.iter().map(|r| r.label).collect::<Vec<_>>(), ["Scrape This Game", "Edit Details", "Cover art", "Open Folder", "Hide from Library"]);
    assert_eq!(rows[2].value, "Generated");
    assert_eq!(page.choose_row(0), Outcome::Door(SCRAPE));
}
