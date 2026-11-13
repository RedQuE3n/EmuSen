//! A game's page without Qt (EmuSen_Qt.md §7.8): the facts line, the actions and Resume's stepping, the states as the
//! design orders them, the Options rows and their stepping, the doors, the help bar by the focus, and the screens' routes
//! into the page and back to where it was opened.

use emusen_qt_models::cheats::Switch;
use emusen_qt_models::display::Crop;
use emusen_qt_models::game_page::*;
use emusen_qt_models::help::Screen;
use emusen_qt_models::screens::{page_origin, page_step, step, step_in, Ask, Shown, GAME_PAGE};

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
        cheats: vec![switch("Infinite lives", true), switch("Moon jump", false), switch("All keys", false)],
        crop: Some(offer(None)),
        ..Read::default()
    }
}

/// Settings' overscan choices as Display crop offers them, the television's edge a share of 7 % past the glass.
fn tv() -> Crop {
    let edge = (1.0 - 1.0 / 1.07) / 2.0;
    Crop { left: edge, top: edge, right: edge, bottom: edge }
}

fn offer(own: Option<Crop>) -> CropOffer {
    CropOffer {
        label: "Crop overscan".into(),
        choices: vec![("Whole picture".into(), Crop::default()), ("TV overscan".into(), tv())],
        custom: "Custom crop".into(),
        own,
        most: 0.25,
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
    // X on a state asks first, No focused and B answering it, naming the state and when it was saved.
    let Outcome::Ask(q, change) = page.delete(1) else { panic!("a question") };
    assert_eq!(q.title, "Delete Slot 2?");
    assert_eq!(q.sentences, ["Cobalt Harbor (Synthetic)'s Slot 2, saved yesterday, is deleted. This cannot be undone."]);
    assert_eq!(q.choices, ["No", "Delete"]);
    assert_eq!(q.back, Some(0));
    assert_eq!(change, Change::DeleteState("s2".into()));
    assert_eq!(page.delete(4), Outcome::Nothing, "an empty slot has nothing to delete");
}

fn switch(description: &str, on: bool) -> Switch {
    Switch { description: description.into(), on }
}

#[test]
fn qt_7_8_options_rows_step_the_engine_and_hide_the_rows_that_open_nothing() {
    let mut page = GamePage::new(read(), NOW);
    page.choose_section(2);
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    let words: Vec<String> = rows.iter().map(|r| format!("{} | {}", r.label, r.value)).collect();
    // Package is shown only where its job can run.
    assert_eq!(words, ["Engine | Venus (default)", "Display crop | Console's", "Cheats | 1 of 3 on"]);
    assert!(rows[0].steps);
    assert_eq!(page.step_row(0, 1), Outcome::Changed(Change::Engine(Some("venus".into()))));
    assert_eq!(page.step_row(0, -1), Outcome::Changed(Change::Engine(Some("snes9x".into()))), "stepping wraps");
    assert_eq!(page.choose_row(0), Outcome::Nothing, "A on a stepped row does nothing");
    assert_eq!(page.help(None, 0), Screen::GamePageValue);
    // One engine runs the console: the row names it and steps nothing; no overscan, no crop row.
    let mut one = read();
    one.engines = vec![Choice { value: None, text: "Venus (the only one)".into() }];
    one.crop = None;
    one.cheats = Vec::new();
    let mut page = GamePage::new(one, NOW);
    page.choose_section(2);
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    assert_eq!(rows.iter().map(|r| r.label.clone()).collect::<Vec<_>>(), ["Engine"], "no cheats, no Cheats row");
    assert!(!rows[0].steps);
}

#[test]
fn qt_cheats_options_opens_the_games_cheats_each_a_switch_saved_as_it_turns() {
    let mut page = GamePage::new(read(), NOW);
    page.choose_section(2);
    assert_eq!(page.choose_row(2), Outcome::Cheats, "A on Cheats opens its page");
    page.cheating = true;
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    let words: Vec<String> = rows.iter().map(|r| format!("{} | {}", r.label, r.value)).collect();
    assert_eq!(words, ["Infinite lives | On", "Moon jump | Off", "All keys | Off"]);
    assert!(rows.iter().all(|r| r.steps && !r.opens));
    assert_eq!(page.choose_row(1), Outcome::Changed(Change::Cheat(1, true)), "A switches a cheat on");
    assert_eq!(page.step_row(0, 1), Outcome::Changed(Change::Cheat(0, false)), "Right switches one off");
    assert_eq!(page.step_row(0, -1), Outcome::Changed(Change::Cheat(0, false)), "Left too");
    assert_eq!(page.help(None, 0), Screen::GamePageValue);
    page.changed(&Change::Cheat(1, true));
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    assert_eq!(rows[1].value, "On", "taken into the page at once");
    assert!(page.back(), "B goes back to Options");
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    assert_eq!(rows[2].value, "2 of 3 on");
    assert!(!page.back(), "B on Options leaves the page");
    page.cheating = true;
    page.choose_section(0);
    assert!(!page.cheating, "another section leaves Cheats");
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
    // Cover art opens nothing yet, so it is not shown.
    assert_eq!(rows.iter().map(|r| r.label.clone()).collect::<Vec<_>>(), ["Scrape This Game", "Edit Details", "Open Folder", "Hide from Library"]);
    assert_eq!(page.choose_row(0), Outcome::Scrape);
    let Outcome::Ask(q, change) = page.choose_row(3) else { panic!("Hide asks first") };
    assert_eq!((q.title.as_str(), q.choices.as_slice(), change), ("Hide Cobalt Harbor (Synthetic)?", &["No".to_string(), "Hide".to_string()][..], Change::Hide));
}

#[test]
fn qt_7_8_the_page_opens_over_the_grid_home_and_the_cards_and_b_returns() {
    let page = Ask::Act(GAME_PAGE.into());
    for from in [Shown::Home, Shown::Grid, Shown::SearchGrid, Shown::HomeCard] {
        assert_eq!(step(from, &page), Some((Shown::Page, None)), "{from:?}");
    }
    assert_eq!(step_in(true, Shown::Esde, &page), Some((Shown::Page, None)));
    assert_eq!(step(Shown::Consoles, &page), None, "Choose a console has no game in hand");
    assert_eq!(page_step(false, page_origin(false, Shown::Grid), &Ask::Back), Some(Shown::Grid));
    assert_eq!(page_step(false, page_origin(false, Shown::HomeCard), &Ask::Back), Some(Shown::Home), "a card closed as it opened the page");
    assert_eq!(page_step(true, page_origin(true, Shown::HomeCard), &Ask::Back), Some(Shown::Esde));
    assert_eq!(page_step(true, Shown::Esde, &Ask::Act("home".into())), Some(Shown::Esde));
    assert_eq!(page_step(false, Shown::Grid, &Ask::Act("home".into())), Some(Shown::Home));
    assert_eq!(page_step(false, Shown::Grid, &Ask::Play), Some(Shown::Game));
}

#[test]
fn qt_7_8_g2_the_standard_the_package_the_players_screenshots_scrape_and_open_folder_are_mistresss() {
    // The television standard: the game's own first, then each standard by its name, stepped, wrapping, and saved.
    let mut r = read();
    r.standards = Some(standard_choices(&["ntsc".into(), "pal".into()]));
    r.standard = Some("pal".into());
    r.package = Package::Linked;
    r.layers = 2;
    r.crop = None;
    let mut page = GamePage::new(r, NOW);
    page.choose_section(2);
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    let words: Vec<String> = rows.iter().map(|r| format!("{} | {}", r.label, r.value)).collect();
    assert_eq!(words, ["Engine | Venus (default)", "Television standard | PAL", "Cheats | 1 of 3 on"]);
    assert!(rows[1].steps);
    assert_eq!(page.step_row(1, 1), Outcome::Changed(Change::Standard(None)), "PAL wraps to the game's own");
    assert_eq!(page.step_row(1, -1), Outcome::Changed(Change::Standard(Some("ntsc".into()))));
    // A step saved is the page's at once, so the next steps from it before the read that follows the save is back.
    page.changed(&Change::Standard(None));
    assert_eq!(page.step_row(1, -1), Outcome::Changed(Change::Standard(Some("pal".into()))), "back from the game's own to PAL");
    page.changed(&Change::Standard(Some("pal".into())));
    page.changed(&Change::Favourite);
    assert!(page.read.favourite != read().favourite, "the heart turned");
    assert_eq!(page.help(None, 1), Screen::GamePageValue);
    // The package's every state, as Mistress words it.
    assert_eq!(package_words(Package::None, 0), "Not a package");
    assert_eq!(package_words(Package::SelfContained, 1), "Self-contained · 1 layer");
    assert_eq!(package_words(Package::Changed, 0), "Changed: plays as a plain game");
    // The player's own screenshots first, numbered, then the scraper's pictures.
    let mut r = read();
    r.media = vec![
        Media { path: "/s/b.png".into(), kind: MediaKind::Taken },
        Media { path: "/s/a.png".into(), kind: MediaKind::Taken },
        Media { path: "/m/shot.png".into(), kind: MediaKind::Screenshot },
    ];
    r.scraping = true;
    let mut page = GamePage::new(r, NOW);
    page.choose_section(3);
    let Content::Tiles(tiles) = page.content() else { panic!("tiles") };
    assert_eq!(tiles.iter().map(|t| t.label.as_str()).collect::<Vec<_>>(), ["Screenshot 1", "Screenshot 2", "Scraped screenshot"]);
    assert_eq!(tiles[0].picture, "/s/b.png");
    // Another run going: Scrape This Game says so and waits; Open Folder is the program's folder action.
    page.choose_section(4);
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    assert_eq!(rows[0].value, "A run is going");
    assert_eq!(page.choose_row(0), Outcome::Nothing);
    assert_eq!(page.choose_row(2), Outcome::OpenFolder);
    // A state saved from another copy asks first, Cancel focused and B answering it.
    let q = other_copy_question("Cobalt Harbor (Europe).sfc", "Cobalt Harbor.sfc");
    assert_eq!(q.title, "A Different Copy of the Game");
    assert_eq!(q.choices, ["Cancel", "Load Anyway"]);
    assert_eq!(q.back, Some(0));
    assert!(q.sentences[0].starts_with("This state was saved from Cobalt Harbor (Europe).sfc, whose contents are not those of Cobalt Harbor.sfc."));
}

#[test]
fn qt_7_8_2_the_viewer_is_es_des_over_the_pages_media_the_manual_apart() {
    let mut r = read();
    r.cover = "/m/cover.png".into();
    r.media = vec![
        Media { path: "/s/a.png".into(), kind: MediaKind::Taken },
        Media { path: "/m/shot.png".into(), kind: MediaKind::Screenshot },
        Media { path: "/m/clip.mkv".into(), kind: MediaKind::Clip },
        Media { path: "/m/manual.pdf".into(), kind: MediaKind::Manual },
    ];
    let mut page = GamePage::new(r, NOW);
    page.choose_section(3);
    let Content::Tiles(tiles) = page.content() else { panic!("tiles") };
    assert_eq!(tiles.iter().map(|t| t.label.as_str()).collect::<Vec<_>>(), ["Screenshot 1", "Scraped screenshot", "Clip", "Manual"]);
    assert_eq!(tiles[3].picture, "/m/cover.png", "a manual's tile shows the cover");
    assert_eq!(page.choose_tile(1), Outcome::View(1));
    assert_eq!(page.choose_tile(9), Outcome::Nothing);
    // Opened on the second picture: the items are the pictures and the clip in the page's order, the manual apart.
    let mut viewer = page.viewer(1).expect("a viewer");
    assert_eq!(viewer.entries.iter().map(|e| e.source.as_str()).collect::<Vec<_>>(), ["/s/a.png", "/m/shot.png", "/m/clip.mkv"]);
    assert_eq!((viewer.index, viewer.manual.as_deref(), viewer.reading), (1, Some("/m/manual.pdf"), None));
    assert_eq!(page.caption(&viewer), "Scraped screenshot · 2 of 3");
    assert_eq!(viewer.shown().map(|e| e.kind.as_str()), Some("screenshot"));
    viewer.browse(1);
    assert_eq!(viewer.shown().map(|e| e.kind.as_str()), Some("video"), "the clip is ES-DE's video");
    assert_eq!(page.caption(&viewer), "Clip · 3 of 3");
    // The manual's tile opens the viewer reading it, its pages told by the drawing.
    let mut manual = page.viewer(3).expect("the manual's viewer");
    assert!(manual.reading.is_some());
    assert_eq!(page.caption(&manual), "Manual");
    manual.set_pages(12);
    assert_eq!(page.caption(&manual), "Manual · Page 1 of 12");
    // No media, no viewer.
    assert!(GamePage::new(read(), NOW).viewer(0).is_none());
}

#[test]
fn qt_7_8_2_a_scrape_is_started_stopped_and_worded_from_its_progress() {
    let mut page = GamePage::new(read(), NOW);
    page.choose_section(4);
    assert_eq!(page.choose_row(0), Outcome::Scrape);
    page.run = Some((true, "Looking it up".into()));
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    assert_eq!(rows[0].value, "Looking it up", "the row says the run's step");
    let Outcome::Stop(q) = page.choose_row(0) else { panic!("A on a run going asks to stop it") };
    assert_eq!((q.title.as_str(), q.choices.as_slice()), ("Stop the scrape?", &["No".to_string(), "Stop".to_string()][..]));
    page.run = Some((false, "Found: cover".into()));
    assert_eq!(page.choose_row(0), Outcome::Scrape, "an ended run starts again");
    let game = "/r/snes/a.sfc";
    let progress = |v: serde_json::Value| run_words(&v, game);
    assert_eq!(progress(serde_json::json!({ "state": "running", "current": null })), (true, "Starting".into()));
    assert_eq!(progress(serde_json::json!({ "state": "running", "current": { "step": "looking_up" } })), (true, "Looking it up".into()));
    assert_eq!(progress(serde_json::json!({ "state": "running", "current": { "step": "downloading", "kind": "cover" } })), (true, "Fetching the cover".into()));
    let done = |outcome: &str, kinds: serde_json::Value| progress(serde_json::json!({ "state": "done", "results": [{ "path": game, "outcome": outcome, "kinds": kinds, "detail": "" }] }));
    assert_eq!(done("found", serde_json::json!(["cover", "screenshot"])), (false, "Found: cover, screenshot".into()));
    assert_eq!(done("found", serde_json::json!([])), (false, "Found, nothing new".into()));
    assert_eq!(done("unknown", serde_json::json!([])), (false, "Not found".into()));
    assert_eq!(done("error", serde_json::json!([])), (false, "It failed".into()));
    assert_eq!(progress(serde_json::json!({ "state": "cancelled" })), (false, "Stopped".into()));
    assert_eq!(progress(serde_json::json!({ "state": "done", "results": [], "why": "no developer" })), (false, "It could not run: no developer".into()));
}

#[test]
fn qt_7_8_2_edit_details_types_each_detail_and_puts_back_what_was_scraped() {
    let mut r = read();
    r.details = DETAILS
        .iter()
        .map(|(field, _)| Detail { field, value: if *field == "name" { "Cobalt Harbor".into() } else { String::new() }, edited: *field == "name" })
        .collect();
    let mut page = GamePage::new(r, NOW);
    page.choose_section(4);
    assert_eq!(page.choose_row(1), Outcome::Edit);
    page.editing = true;
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    assert_eq!(rows.iter().map(|r| r.label.clone()).collect::<Vec<_>>(), ["Name", "Description", "Release date", "Developer", "Publisher", "Genre", "Players"]);
    assert_eq!(rows[0].value, "Cobalt Harbor");
    assert_eq!(page.help(None, 0), Screen::GamePageEdit);
    assert_eq!(page.choose_row(0), Outcome::Type { field: "name", title: "Cobalt Harbor (Synthetic) › Name".into(), start: "Cobalt Harbor".into() });
    assert_eq!(page.reset(0), Outcome::Changed(Change::Edit("name", None)), "Y puts back the scraped name");
    assert_eq!(page.reset(1), Outcome::Nothing, "a detail not edited has nothing to put back");
    page.changed(&Change::Edit("developer", Some("Lantern Works".into())));
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    assert_eq!(rows[3].value, "Lantern Works", "a detail saved is shown at once");
    assert_eq!(page.reset(3), Outcome::Changed(Change::Edit("developer", None)));
    assert!(page.back(), "B leaves Edit Details for Details");
    assert!(!page.back(), "and then the page");
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    assert_eq!(rows[0].label, "Scrape This Game");
    page.editing = true;
    page.choose_section(3);
    assert!(!page.editing, "another section leaves Edit Details");
}

#[test]
fn opened_at_cheats_the_page_shows_options_cheats_and_gives_way_to_options_without_any() {
    use emusen_qt_models::game_page::{GamePage, Section, CHEATS};
    let mut page = GamePage::new(Read::default(), 0);
    page.open_at(CHEATS);
    assert_eq!((page.shown(), page.cheating), (Section::Options, true));
    page.reread(Read::default());
    assert!(!page.cheating, "no cheats to show: Options");
    let mut other = GamePage::new(Read::default(), 0);
    other.open_at("");
    assert_eq!((other.shown(), other.cheating), (Section::Overview, false), "no part: the page's start");
}

#[test]
fn display_crop_steps_the_games_own_crop_through_settings_overscan_choices() {
    let mut page = GamePage::new(read(), NOW);
    page.choose_section(2);
    assert_eq!(page.choose_row(1), Outcome::Crop, "A on Display crop opens its page");
    page.cropping = true;
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    assert_eq!(rows.iter().map(|r| format!("{} | {}", r.label, r.value)).collect::<Vec<_>>(), ["Crop overscan | Console's"]);
    assert!(rows[0].steps && !rows[0].opens);
    assert_eq!(page.help(None, 0), Screen::GamePageValue);
    assert_eq!(page.step_row(0, 1), Outcome::Changed(Change::Crop(Some(Crop::default()))), "Right: the whole picture");
    page.changed(&Change::Crop(Some(Crop::default())));
    assert_eq!(page.step_row(0, 1), Outcome::Changed(Change::Crop(Some(tv()))), "Right again: TV overscan");
    page.changed(&Change::Crop(Some(tv())));
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    assert_eq!(rows.iter().map(|r| format!("{} | {}", r.label, r.value)).collect::<Vec<_>>(), ["Crop overscan | TV overscan"], "taken into the page at once; no edges");
    // Right once more: a crop of the player's own, Settings' last choice, starting from the crop shown, its four edges after it.
    assert_eq!(page.step_row(0, 1), Outcome::Changed(Change::Crop(Some(tv()))));
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    let edge = (tv().left * 1000.0).round() / 10.0;
    let words = rows.iter().map(|r| format!("{} | {}", r.label, r.value)).collect::<Vec<_>>();
    assert_eq!(words, ["Crop overscan | Custom crop".to_owned(), format!("Left edge | {edge}%"), format!("Right edge | {edge}%"), format!("Top edge | {edge}%"), format!("Bottom edge | {edge}%")]);
    assert!(rows[1..].iter().all(|r| r.steps && !r.sentence.is_empty()));
    // Right on the left edge: the next whole percent, the other edges kept, stored as the game's own crop.
    let stepped = Crop { left: 0.04, ..tv() };
    assert_eq!(page.step_row(1, 1), Outcome::Changed(Change::Crop(Some(stepped))));
    page.changed(&Change::Crop(Some(stepped)));
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    assert_eq!(rows[1].value, "4%");
    assert_eq!(page.step_row(3, -1), Outcome::Changed(Change::Crop(Some(Crop { top: 0.03, ..stepped }))), "Left on the top edge: the whole percent below");
    // An edge is held to the catalogue's most, and to none.
    page.changed(&Change::Crop(Some(Crop { left: 0.25, top: 0.0, ..stepped })));
    assert_eq!(page.step_row(1, 1), Outcome::Changed(Change::Crop(Some(Crop { left: 0.25, top: 0.0, ..stepped }))), "no more than a quarter");
    assert_eq!(page.step_row(3, -1), Outcome::Changed(Change::Crop(Some(Crop { left: 0.25, top: 0.0, ..stepped }))), "no less than none");
    // Every edge at none is still the player's own while it is chosen, so its rows stay under the focus.
    page.changed(&Change::Crop(Some(Crop::default())));
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    assert_eq!((rows.len(), rows[0].value.as_str()), (5, "Custom crop"));
    // Past the last, the console's again, and the edges gone.
    assert_eq!(page.step_row(0, 1), Outcome::Changed(Change::Crop(None)));
    page.changed(&Change::Crop(None));
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    assert_eq!(rows.len(), 1);
    assert_eq!(page.step_row(0, -1), Outcome::Changed(Change::Crop(Some(Crop::default()))), "Left from the console's: a crop of the player's own");
    // A crop of the player's own read from games.db, which no choice names, is said as Settings says one.
    let own = Crop { left: 0.1, top: 0.0, right: 0.0, bottom: 0.0 };
    page.custom = false;
    page.changed(&Change::Crop(Some(own)));
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    assert_eq!((rows[0].value.as_str(), rows[1].value.as_str(), rows.len()), ("Custom crop", "10%", 5));
    assert_eq!(page.step_row(0, 1), Outcome::Changed(Change::Crop(None)));
    page.custom = true;
    assert_eq!(page.step_row(0, -1), Outcome::Changed(Change::Crop(Some(tv()))));
    assert_eq!(change_failed(&Change::Crop(None), "locked"), "The game's crop could not be stored: locked");
    assert!(page.back(), "B goes back to Options");
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    assert_eq!(rows[1].value, "This game's");
    assert!(!page.back(), "B on Options leaves the page");
}

#[test]
fn package_shows_the_games_package_and_runs_its_job_with_its_progress() {
    let make = PackageJob { key: MAKE, request: serde_json::json!({ "op": "pkg-make", "paths": ["SNES/a.sfc"] }) };
    let mut r = read();
    r.crop = None;
    r.package_job = Some(make.clone());
    let mut page = GamePage::new(r, NOW);
    page.choose_section(2);
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    assert_eq!(rows.last().map(|r| format!("{} | {}", r.label, r.value)).as_deref(), Some("Package | Not a package"));
    assert_eq!(page.choose_row(rows.len() - 1), Outcome::Package, "A on Package opens its page");
    page.packaging = true;
    let words = |page: &GamePage| match page.content() {
        Content::Rows(rows) => rows.iter().map(|r| format!("{} | {}", r.label, r.value)).collect::<Vec<_>>(),
        _ => panic!("rows"),
    };
    assert_eq!(words(&page), ["Package | Not a package", "Make Package | …"]);
    assert_eq!(page.choose_row(0), Outcome::Nothing, "the package's state is only said");
    assert_eq!(page.choose_row(1), Outcome::Job(make), "A starts the job");
    // Its progress as the jobs tools say it, then how it ended.
    let mut run = emusen_qt_models::tools::jobs::Run::default();
    run.take(&serde_json::json!({ "event": "start", "op": "pkg-make", "packages": 1, "refused": [] }), &[]);
    page.job = Some((MAKE, true, run.clone()));
    assert_eq!(words(&page)[1], "Make Package | 0 of 1");
    assert_eq!(page.choose_row(1), Outcome::StopJob, "A while it runs stops it");
    run.take(&serde_json::json!({ "event": "package", "source": "SNES/a.sfc", "state": "made" }), &[]);
    run.take(&serde_json::json!({ "event": "end", "op": "pkg-make", "counts": { "made": 1 }, "seconds": 0.1 }), &[]);
    page.job = Some((MAKE, false, run));
    assert_eq!(words(&page)[1], "Make Package | Done");
    // Read again, the game is a linked package, and its job checks it.
    let mut r = page.read.clone();
    r.package = Package::Linked;
    r.package_job = Some(PackageJob { key: CHECK, request: serde_json::json!({ "op": "pkg-check", "paths": ["/p/a"] }) });
    page.reread(r);
    assert_eq!(words(&page), ["Package | Linked", "Check Package | …"], "the make's run is not the check's");
    assert!(page.back());
    // A game whose package job cannot run has no Package row.
    let mut r = read();
    r.package_job = None;
    let mut page = GamePage::new(r, NOW);
    page.choose_section(2);
    assert!(words(&page).iter().all(|w| !w.starts_with("Package")));
}
