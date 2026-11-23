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
    // A game whose pictures are not looked for has no Cover art.
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

/// A card with the words the source gives it: its kind's for one of the game's own pictures, else `cover_words`'.
fn card(choice: CoverChoice, words: &str, picture: &str) -> CoverCard {
    let text = if words.is_empty() { cover_words(&choice) } else { words.into() };
    CoverCard { text, choice, picture: picture.into() }
}

fn covered() -> Read {
    let mut r = read();
    r.covers = vec![
        card(CoverChoice::Found, "", "/m/cover.png"),
        card(CoverChoice::Own("screenshot".into()), "Scraped screenshot", "/m/shot.png"),
        card(CoverChoice::Own("titlescreen".into()), "Title screen", "/m/title.png"),
    ];
    r.media = vec![
        Media { path: "/m/taken.png".into(), kind: MediaKind::Taken },
        Media { path: "/m/shot.png".into(), kind: MediaKind::Screenshot },
        Media { path: "/m/title.png".into(), kind: MediaKind::TitleScreen },
        Media { path: "/m/clip.mkv".into(), kind: MediaKind::Clip },
    ];
    r
}

/// Cover art's cards as (words, picture, in use).
fn cards(page: &GamePage) -> Vec<(String, String, bool)> {
    let Content::Tiles(tiles) = page.content() else { panic!("cards") };
    tiles.iter().map(|t| (t.label.clone(), t.picture.clone(), t.marked)).collect()
}

#[test]
fn qt_7_8_cover_art_shows_the_games_covers_as_cards_marks_the_one_in_use_and_a_chooses() {
    let mut page = GamePage::new(covered(), NOW);
    page.choose_section(4);
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    assert_eq!(rows.iter().map(|r| r.label.clone()).collect::<Vec<_>>(), ["Scrape This Game", "Edit Details", COVER_ART, "Open Folder", "Hide from Library"]);
    assert_eq!((rows[2].label.as_str(), rows[2].value.as_str(), rows[2].opens), ("Cover Art", "Its own", true), "the board's words");
    assert_eq!(page.choose_row(2), Outcome::CoverArt);
    page.covering = true;
    let pictures = ["/m/cover.png", "/m/shot.png", "/m/title.png"];
    let labels = ["Its own", "Scraped screenshot", "Title screen"];
    let marked = |at: usize| (0..labels.len()).map(|i| (labels[i].to_string(), pictures[i].to_string(), i == at)).collect::<Vec<_>>();
    assert_eq!(cards(&page), marked(0), "the cover found in use before any choice");
    let Content::Tiles(tiles) = page.content() else { panic!("cards") };
    assert_eq!(tiles[0].line, SHOWN, "the mark's words on the card in use");
    assert!(tiles[1].line.is_empty());
    assert_eq!(page.help(None, 1), Screen::GamePageCoverArt);
    assert_eq!(page.choose_tile(0), Outcome::Nothing, "the card in use is in use already");
    assert_eq!(page.choose_tile(2), Outcome::Changed(Change::Cover(Some("titlescreen".into()))));
    page.changed(&Change::Cover(Some("titlescreen".into())));
    assert_eq!(cards(&page), marked(2));
    assert_eq!(page.choose_tile(0), Outcome::Changed(Change::Cover(None)), "the first card puts the cover found back");
    assert!(page.back(), "B goes back to Details");
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    assert_eq!(rows[2].value, "Title screen");
    assert!(!page.back(), "and B again leaves the page");
    assert_eq!(page.cover_trail(), format!("{} / Details", page.read.title));
    assert_eq!(place(1, 8), "2 of 8");
}

#[test]
fn qt_7_8_a_borrowed_cover_is_a_card_in_use_and_any_choice_ends_it() {
    let mut r = covered();
    let borrowed = CoverChoice::Borrowed("/roms/snes/Other Game.sfc".into());
    r.covers.insert(0, card(borrowed.clone(), "", "/m/other.png"));
    r.cover_in_use = borrowed;
    let mut page = GamePage::new(r, NOW);
    page.choose_section(4);
    page.covering = true;
    assert_eq!(cards(&page)[0], ("Borrowed from Other Game".to_string(), "/m/other.png".to_string(), true), "the borrowed cover, in use");
    assert_eq!(cards(&page).iter().filter(|c| c.2).count(), 1, "one card in use");
    assert_eq!(page.choose_tile(0), Outcome::Nothing);
    assert_eq!(page.choose_tile(1), Outcome::Changed(Change::Cover(None)), "the cover found is a choice while another game's is shown");
    page.changed(&Change::Cover(None));
    assert_eq!(cards(&page).iter().map(|c| c.0.as_str()).collect::<Vec<_>>(), ["Its own", "Scraped screenshot", "Title screen"], "the borrowed card gone");
    assert!(cards(&page)[0].2, "the cover found in use");
}

/// The page on Cover art for a game whose found cover is there or not, with the cover in use given.
fn none_case(found: bool, in_use: CoverChoice) -> GamePage {
    let mut r = covered();
    if !found {
        r.covers.retain(|c| c.choice != CoverChoice::Found);
    }
    if let CoverChoice::Borrowed(_) = &in_use {
        r.covers.insert(0, card(in_use.clone(), "", "/m/other.png"));
    }
    r.cover_in_use = in_use;
    let mut page = GamePage::new(r, NOW);
    page.choose_section(4);
    page.covering = true;
    page
}

fn card_words(page: &GamePage) -> Vec<String> {
    cards(page).into_iter().map(|c| c.0).collect()
}

#[test]
fn qt_7_8_5_none_is_a_card_only_where_no_cover_was_found_and_a_choice_is_in_use_and_it_clears_the_choice() {
    let title = CoverChoice::Own("titlescreen".into());
    // No cover found and the title screen chosen: None stands in the found cover's place, unmarked, and A clears the choice.
    let mut page = none_case(false, title.clone());
    assert_eq!(cards(&page), [
        ("None".to_string(), String::new(), false),
        ("Scraped screenshot".to_string(), "/m/shot.png".to_string(), false),
        ("Title screen".to_string(), "/m/title.png".to_string(), true),
    ]);
    assert_eq!(page.choose_tile(0), Outcome::Changed(Change::Cover(None)), "None clears the choice");
    page.changed(&Change::Cover(None));
    assert_eq!(card_words(&page), ["Scraped screenshot", "Title screen"], "with no choice the None card goes");
    // No cover found and another game's borrowed: None after the borrowed card, and A ends the borrowing.
    let mut page = none_case(false, CoverChoice::Borrowed("/roms/snes/Other Game.sfc".into()));
    assert_eq!(card_words(&page), ["Borrowed from Other Game", "None", "Scraped screenshot", "Title screen"]);
    assert_eq!(page.choose_tile(1), Outcome::Changed(Change::Cover(None)));
    page.changed(&Change::Cover(None));
    assert_eq!(card_words(&page), ["Scraped screenshot", "Title screen"]);
    // A cover found and no choice; a cover found and a choice; no cover found and no choice: no None card.
    assert_eq!(card_words(&none_case(true, CoverChoice::Found)), ["Its own", "Scraped screenshot", "Title screen"]);
    assert_eq!(card_words(&none_case(true, title)), ["Its own", "Scraped screenshot", "Title screen"]);
    assert_eq!(card_words(&none_case(false, CoverChoice::Found)), ["Scraped screenshot", "Title screen"]);
}

#[test]
fn qt_7_8_the_viewer_sets_a_scraped_picture_as_the_cover_but_not_a_taken_one_a_clip_or_the_cover_shown() {
    let mut page = GamePage::new(covered(), NOW);
    let at = |page: &GamePage, index: usize| page.viewer(index).and_then(|v| page.viewer_cover(&v));
    assert_eq!(at(&page, 0), None, "the player's own screenshot is no medium of the store");
    assert_eq!(at(&page, 1).as_deref(), Some("screenshot"));
    assert_eq!(at(&page, 2).as_deref(), Some("titlescreen"));
    assert_eq!(at(&page, 3), None, "a clip is not a picture");
    page.changed(&Change::Cover(Some("titlescreen".into())));
    assert_eq!(at(&page, 2), None, "the picture already the cover");
    assert_eq!(change_failed(&Change::Cover(None), "no"), "The game's cover could not be chosen: no");
}

#[test]
fn qt_7_8_5_cover_art_whose_pictures_could_not_be_looked_for_says_why() {
    let mut r = read();
    r.covers_unread = Some("the scraper could not be opened".into());
    let mut page = GamePage::new(r, NOW);
    page.choose_section(4);
    let Content::Rows(rows) = page.content() else { panic!("rows") };
    let at = rows.iter().position(|r| r.label == COVER_ART).expect("the row stays");
    assert!(!rows[at].opens, "it opens nothing");
    assert_eq!(page.choose_row(at), Outcome::Said(format!("{COVER_UNREAD}: the scraper could not be opened")));
}

/// Eclipse's page as its boards word it (EclipseGame.dc.html; EmuSen_Qt.md §3.4.11).
fn detailed(mut r: Read) -> Read {
    r.details = vec![
        Detail { field: "genre", value: "Action".into(), edited: false },
        Detail { field: "players", value: "1-2".into(), edited: false },
    ];
    r
}

#[test]
fn qt_3_4_11_a_themes_trail_opens_choose_a_console_and_the_consoles_games() {
    assert_eq!(page_step(false, Shown::Home, &Ask::Section(emusen_qt_models::screens::CONSOLES_KEY.into())), Some(Shown::Consoles));
    assert_eq!(page_step(false, Shown::Home, &Ask::Open("snes".into())), Some(Shown::Grid));
    assert_eq!(page_step(false, Shown::Home, &Ask::Section("library".into())), None, "a section that opens at a key is not a trail's");
}

#[test]
fn qt_3_4_11_the_tabs_are_the_boards_six_in_its_order_cheats_only_with_cheats() {
    let mut page = GamePage::new(read(), NOW);
    let names = |p: &GamePage| p.tabs().iter().map(|t| t.name()).collect::<Vec<_>>();
    assert_eq!(names(&page), ["Overview", "States", "Media", "Details", "Options", "Cheats"]);
    assert_eq!(page.tab(), 0);
    assert!(page.choose_tab(1));
    assert_eq!((page.shown(), page.tab(), page.tab_key()), (Section::States, 1, Some("states")));
    assert!(page.choose_tab(2));
    assert_eq!((page.shown(), page.tab()), (Section::Media, 2), "Media is the board's third, the program's fourth section");
    assert!(page.choose_tab(5));
    assert_eq!((page.shown(), page.cheating, page.tab(), page.tab_key()), (Section::Options, true, 5, Some("cheats")));
    assert!(page.choose_tab(4));
    assert_eq!((page.cheating, page.tab()), (false, 4), "Options leaves Cheats");
    assert!(!page.choose_tab(6), "no seventh tab");
    let mut none = read();
    none.cheats.clear();
    let page = GamePage::new(none, NOW);
    assert_eq!(names(&page), ["Overview", "States", "Media", "Details", "Options"]);
}

#[test]
fn qt_3_4_11_a_theme_draws_the_tabs_it_names_and_never_a_deeper_page_or_a_question() {
    let mut page = GamePage::new(read(), NOW);
    let states = vec!["states".to_owned()];
    assert!(page.drawn_by(&[], false) && page.drawn_by(&[], true), "a screen that names none draws everything");
    assert!(!page.drawn_by(&states, false), "the Overview is the program's");
    page.choose_tab(1);
    assert!(page.drawn_by(&states, false));
    assert!(!page.drawn_by(&states, true), "a question is the program's");
    page.choose_tab(3);
    page.editing = true;
    assert_eq!(page.tab_key(), None, "Edit Details is a deeper page");
    assert!(!page.drawn_by(&["details".to_owned()], false));
}

#[test]
fn qt_3_4_11_the_kicker_the_line_and_the_buttons_are_the_boards() {
    let mut r = detailed(read());
    r.console = "Super Nintendo".into();
    r.played_seconds = 3 * 3600 + 20 * 60;
    r.last_played = Some(NOW - 30 * 3600);
    r.favourite = true;
    let page = GamePage::new(r.clone(), NOW);
    assert_eq!(page.kicker(), "Super Nintendo · 1994 · Action");
    assert_eq!(page.line(), "Last played yesterday · 3 h 20 min played · 1 to 2 players");
    let pills = page.pills();
    let words: Vec<_> = pills.iter().map(|p| (p.key, p.label, p.button, p.filled)).collect();
    assert_eq!(words, [(RESUME, "Resume", emusen_kit_core::input::UiButton::Accept as i32, true), (PLAY, "Start from the beginning", -1, false), (FAVOURITE, "Favourite", -1, false)]);
    assert_eq!(pills[2].value, HEART_ON);
    let mut never = detailed(read());
    never.states.clear();
    never.year.clear();
    let page = GamePage::new(never, NOW);
    assert_eq!(page.kicker(), "SNES · Action", "a part the game lacks is left out");
    assert_eq!(page.line(), "Never played · 1 to 2 players");
    let words: Vec<_> = page.pills().iter().map(|p| (p.key, p.label, p.filled)).collect();
    assert_eq!(words, [(PLAY, "Start", true), (FAVOURITE, "Favourite", false)]);
    assert_eq!(page.pills()[1].value, HEART_OFF);
    assert_eq!(page.cards_note(), NO_STATES);
}

#[test]
fn qt_3_4_11_the_cards_are_the_resume_point_the_slots_in_order_and_one_empty_slot() {
    let mut page = GamePage::new(read(), NOW);
    page.choose_tab(1);
    let when = |at: i64| format!("at {}", NOW - at);
    let cards = page.state_cards(&when);
    let shown: Vec<_> = cards.iter().map(|c| (c.label.as_str(), c.line.as_str(), c.marked, c.badge.as_str(), c.row)).collect();
    let Content::Tiles(tiles) = page.content() else { panic!("States shows tiles") };
    let row = |key: &str| tiles.iter().position(|t| t.key == key);
    assert_eq!(
        shown,
        [
            ("Resume point", "at 720", true, RESUMES_HERE, row("auto")),
            ("Slot 1", "at 7200", false, "", row("s1")),
            ("Slot 2", "at 93600", false, "", row("s2")),
            ("Slot 3", "at 259200", false, "", row("s3")),
            (EMPTY_SLOT, "", false, "", None),
        ]
    );
    assert!(cards[..4].iter().all(|c| c.row.is_some()), "every state's card is a tile the page's calls take");
    assert_eq!(page.cards_note(), "");
}

#[test]
fn qt_3_4_11_a_states_when_line_is_the_boards_and_the_testers_forms() {
    let (today, yesterday) = ((2026, 11, 23), (2026, 11, 22));
    let when = |day| state_when(day, today, yesterday, "9:12 PM");
    assert_eq!(when(yesterday), "Yesterday, 9:12 PM");
    assert_eq!(when((2026, 11, 8)), "8 November, 9:12 PM");
    assert_eq!(when(today), "Today, 9:12 PM", "R-20261123-34");
    assert_eq!(when((2025, 11, 8)), "8 November 2025, 9:12 PM", "R-20261123-34");
    assert_eq!(state_when((2027, 1, 1), (2027, 1, 2), (2027, 1, 1), "0:05"), "Yesterday, 0:05", "yesterday across a year's turn");
}

#[test]
fn qt_3_4_11_resume_loads_the_resume_point_and_is_never_stepped() {
    let mut r = read();
    r.states[0].when = NOW - 5 * 86_400;
    let mut page = GamePage::new(r, NOW);
    page.choose_tab(1);
    let Content::Tiles(tiles) = page.content() else { panic!("tiles") };
    let row = page.resume_point_row().expect("a resume point");
    assert_eq!(tiles[row].key, "auto", "the resume point, not the most recent state");
    assert!(page.pills().iter().all(|p| !p.stepped), "Eclipse offers no step (R-20261123-35)");
    let cards = page.state_cards(&|_| String::new());
    assert_eq!(cards.iter().filter(|c| c.marked).map(|c| c.key.as_str()).collect::<Vec<_>>(), ["auto"], "the badge on the resume point alone");
}

#[test]
fn qt_3_4_11_the_states_no_board_draws_as_the_tester_ruled() {
    let state = |slot, path: &str| State { path: path.into(), slot, picture: String::new(), when: NOW - 60 };
    // Only a resume point: its card, then the empty slot (R-20261123-36, 1).
    let mut only = read();
    only.states = vec![state(Slot::Auto, "auto")];
    let page = GamePage::new(only, NOW);
    let labels = |p: &GamePage| p.state_cards(&|_| String::new()).iter().map(|c| c.label.clone()).collect::<Vec<_>>();
    assert_eq!(labels(&page), ["Resume point", EMPTY_SLOT]);
    assert_eq!(page.pills()[0].label, "Resume");
    // Slots but no resume point: no badge, and the never-played board's buttons (R-20261123-36, 2).
    let mut slots = read();
    slots.states = vec![state(Slot::Number(2), "s2"), state(Slot::Number(1), "s1")];
    let page = GamePage::new(slots, NOW);
    assert_eq!(labels(&page), ["Slot 1", "Slot 2", EMPTY_SLOT]);
    assert!(page.state_cards(&|_| String::new()).iter().all(|c| !c.marked && c.badge.is_empty()));
    assert_eq!(page.pills().iter().map(|p| p.label).collect::<Vec<_>>(), ["Start", "Favourite"]);
    assert_eq!(page.resume_point_row(), None);
    assert_eq!(page.cards_note(), "", "the words beside the cards are the never-played board's alone");
    assert_eq!(page.board_help(Some(0), 0), Some(Screen::GamePageStart));
    // More than fit: every slot and the resume point, no empty card (R-20261123-33).
    let mut full = read();
    full.states = (1..=8).map(|n| state(Slot::Number(n), &format!("s{n}"))).chain([state(Slot::Auto, "auto")]).collect();
    let page = GamePage::new(full, NOW);
    assert_eq!(labels(&page).len(), 9);
    assert!(!labels(&page).iter().any(|l| l == EMPTY_SLOT));
}

#[test]
fn qt_3_4_11_the_help_in_the_boards_words_on_a_state_and_on_start() {
    let mut page = GamePage::new(read(), NOW);
    page.choose_tab(1);
    let Content::Tiles(tiles) = page.content() else { panic!("tiles") };
    let s1 = tiles.iter().position(|t| t.key == "s1").unwrap();
    let empty = tiles.iter().position(|t| t.key.starts_with("empty.")).unwrap();
    assert_eq!(page.board_help(None, s1), Some(Screen::GamePageState));
    assert_eq!(page.board_help(None, empty), None, "an empty slot has the program's");
    assert_eq!(page.board_help(Some(0), 0), None, "Play with a resume point: the boards draw none");
    assert_eq!(page.board_help(Some(1), 0), Some(Screen::GamePageResume), "Resume without its step");
    let mut none = read();
    none.states.clear();
    let page = GamePage::new(none, NOW);
    assert_eq!(page.board_help(Some(0), 0), Some(Screen::GamePageStart));
    assert_eq!(page.board_help(Some(1), 0), None, "Favourite: the boards draw none");
    let words = |s: Screen| emusen_qt_models::help::entries(s).iter().map(|e| e.1).collect::<Vec<_>>();
    assert_eq!(words(Screen::GamePageState), ["Load this state", "Delete", "Back"]);
    assert_eq!(words(Screen::GamePageStart), ["Start", "Favourite", "Back", "Home"]);
    assert_eq!(words(Screen::GamePageSections), ["Section"]);
}
