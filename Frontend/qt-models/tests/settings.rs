//! The Settings screen's view without Qt (EmuSen_Qt.md §7.7), over a fake source holding Galaxia's committed rows: every
//! row the description names is Galaxia's or a door of its own; values, steps, consoles' rows, the CRT from Serenity's
//! catalogue, doors and pages, the panes, and what the settings ask of the program.

use emusen_kit_core::settings::RowKind;
use emusen_qt_models::settings::{self, Model, Outcome, Preview, SettingsSource, SettingsView};
use serde_json::{json, Value};
use std::cell::RefCell;

const ROWS: &str = include_str!("../../../Platform/galaxia/data/frontend_settings.json");

struct Fake {
    app: Value,
    graphics: Value,
    saved: RefCell<Vec<(Model, Value)>>,
}

impl SettingsSource for Fake {
    fn rows(&self) -> Result<Value, String> {
        Ok(serde_json::from_str(ROWS).unwrap())
    }
    fn load(&self, model: Model) -> Result<Value, String> {
        Ok(if model == Model::App { self.app.clone() } else { self.graphics.clone() })
    }
    fn save(&self, model: Model, document: &Value) -> Result<(), String> {
        self.saved.borrow_mut().push((model, document.clone()));
        Ok(())
    }
    fn catalogue(&self) -> Result<Value, String> {
        let balanced = 1.0f32.to_bits();
        Ok(json!({
            "filters": [
                {"key": "CRT", "name": "CRT", "settings": [{"id": "quality", "initial": balanced, "choices": ["Performance", "Balanced", "Accurate"]}]},
                {"key": "effect.None.nearest", "name": "Plain"}
            ],
            "screens": [{"console": "NES", "kind": "Television"}, {"console": "GB", "kind": "Lcd"}, {"console": "SNES", "kind": "Television"}]
        }))
    }
    fn outputs(&self) -> Vec<String> {
        vec!["Speakers".into()]
    }
    fn folders(&self) -> std::collections::BTreeMap<String, String> {
        [("firmware".to_owned(), "/emusen/Firmware".to_owned())].into()
    }
}

fn fake(app: Value, graphics: Value) -> Fake {
    Fake { app, graphics, saved: RefCell::new(Vec::new()) }
}

fn view(source: &Fake) -> SettingsView {
    let mut view = SettingsView::new();
    view.read = settings::read(source).unwrap();
    view
}

fn section(view: &mut SettingsView, name: &str) {
    let at = view.sections().iter().position(|s| s == name).unwrap();
    view.choose_section(at);
}

fn shown(view: &SettingsView) -> Vec<(String, String)> {
    view.rows().into_iter().map(|r| (r.label, r.value)).collect()
}

fn index(view: &SettingsView, id: &str) -> usize {
    view.rows().iter().position(|r| r.id == id).unwrap_or_else(|| panic!("no row {id}"))
}

#[test]
fn qt_7_7_every_row_is_galaxias_or_a_door_and_the_sections_are_the_six() {
    let source = fake(json!({}), json!({}));
    let mut view = view(&source);
    assert_eq!(view.sections(), ["Controllers", "Display", "Sound", "Consoles", "Library", "EmuSen"]);
    for s in 0..view.sections().len() {
        view.choose_section(s);
        for row in view.rows() {
            assert!(!row.label.is_empty(), "{}: row {} names no row of Galaxia's and no words of its own", view.sections()[s], row.id);
        }
    }
}

#[test]
fn qt_7_7_sound_shows_the_stored_values_and_a_step_saves_them_in_their_own_types() {
    let source = fake(json!({"Volume": 1.0, "NavigationSounds": true, "BigPictureInterface": {"NavigationVolume": 60}, "SoundOutputDevice": "System default"}), json!({}));
    let mut view = view(&source);
    section(&mut view, "Sound");
    assert_eq!(
        shown(&view),
        [("Volume".into(), "100%".into()), ("Navigation sounds".into(), "On".into()), ("Sound volume".into(), "60%".into()), ("Output".into(), "System default".into())]
    );
    assert!(view.rows().iter().all(|r| r.kind == RowKind::Steps));
    let pane = view.pane(0);
    assert_eq!((pane.preview, pane.meter), (Preview::Meter, 1.0));
    assert_eq!(pane.sentence, "The game's volume. On the handheld this row defers to the device's quick menu.");

    assert_eq!(view.step(0, -1), Outcome::Saved(Model::App, view.read.app.clone()));
    assert_eq!(view.read.app["Volume"], json!(0.95), "a share stays a number");
    assert_eq!(view.rows()[0].value, "95%");
    view.step(1, 1);
    assert_eq!(view.read.app["NavigationSounds"], json!(false), "a switch stays a boolean");
    view.step(2, 1);
    assert_eq!(view.read.app["BigPictureInterface"]["NavigationVolume"], json!(65), "a nested model's whole number");
    assert_eq!(view.pane(2).meter, 0.65);

    // The machine's devices follow the fixed choice, and the pane says where the sound plays.
    view.step(3, 1);
    assert_eq!(view.rows()[3].value, "Speakers");
    assert_eq!(view.pane(3).sentence, "The game and the interface's sounds play on Speakers.");
}

#[test]
fn qt_7_7_a_consoles_row_is_set_on_each_and_says_consoles_while_they_differ() {
    let source = fake(json!({}), json!({"Consoles": {"NES": {"PictureShape": "square"}}}));
    let mut view = view(&source);
    section(&mut view, "Display");
    let shape = index(&view, "shape");
    assert_eq!(view.rows()[shape].value, "Console's", "NES square, the others at the default");
    let crop = index(&view, "crop");
    assert_eq!(view.rows()[crop].value, "Whole picture");
    view.step(shape, 1);
    for console in ["NES", "GB", "SNES"] {
        assert_eq!(view.read.graphics["Consoles"][console]["PictureShape"], "tv", "{console}");
    }
    assert_eq!(view.rows()[shape].value, "TV shape");
    view.step(crop, 1);
    assert_eq!(view.read.graphics["Consoles"]["NES"]["Overscan"], "tv");
    assert!(view.read.graphics["Consoles"]["GB"].get("Overscan").is_none(), "an LCD has no overscan to crop");
}

#[test]
fn qt_7_7_the_crt_row_takes_its_tiers_from_serenitys_catalogue() {
    let source = fake(json!({}), json!({}));
    let mut view = view(&source);
    section(&mut view, "Display");
    let crt = index(&view, "crt");
    assert_eq!(view.rows()[crt].value, "Off");
    let pane = view.pane(crt);
    assert_eq!(pane.preview, Preview::Picture);
    assert_eq!(pane.caption, "The last game · Off CRT");
    assert_eq!(pane.sentence, "Off: the console's pixels as they are, with no television.");
    view.step(crt, 1);
    assert_eq!(view.rows()[crt].value, "Performance");
    assert_eq!(view.read.graphics["Consoles"]["SNES"]["ScreenFilter"], "CRT");
    assert_eq!(view.read.graphics["ShaderParameters"]["SNES"]["CRT"]["quality"], "0");
    view.read.graphics["ShaderParameters"]["NES"]["CRT"] = json!({});
    assert_eq!(view.rows()[crt].value, "Console's", "NES at the setting's initial Balanced, SNES Performance");
    for _ in 0..4 {
        view.step(crt, -1);
    }
    assert_eq!(view.rows()[crt].value, "Off", "from Console's, Left starts at the last tier and steps down to Off");
    assert_eq!(view.read.graphics["Consoles"]["NES"]["ScreenFilter"], "Plain", "Off is the catalogue's plain picture by its name there");
    view.facts = json!({"running": "Aurora Drift"});
    assert_eq!(view.pane(crt).caption, "Aurora Drift · Off CRT");
}

#[test]
fn qt_7_7_doors_open_pages_and_screens_and_b_closes_a_page_first() {
    let source = fake(json!({"RomDirectory": "/games", "Scraping": true, "EmuSenTheme": "emusen"}), json!({}));
    let mut view = view(&source);
    view.facts = json!({"games": 3});
    section(&mut view, "Library");
    let rows = shown(&view);
    assert_eq!(rows[0], ("ROM folder".into(), "/games".into()));
    assert_eq!(rows[1], ("Artwork folder".into(), "beside the games".into()));
    assert_eq!(rows[2], ("Themes".into(), "EmuSen (built in)".into()));
    assert_eq!(rows[3], ("Scraping".into(), "ScreenScraper".into()));
    assert_eq!(rows[5], ("Hidden games".into(), "Not listed".into()));
    assert_eq!(rows[6].0, "Media viewer", "the viewer's settings, a page of their own (EmuSen_Qt.md §4.14)");
    assert_eq!(view.rows()[0].kind, RowKind::Opens);
    assert_eq!(view.pane(0).sentence, "3 games under /games, read in place and never moved.");
    assert_eq!(view.choose(index(&view, "themes")), Outcome::Door("themes".into()));
    // A folder cut on its row shows whole in the pane once focused.
    view.read.app["ArtworkDirectory"] = json!("/a/folder/long/enough/to/be/cut/on/its/row/of/seven/hundred/and/four/units/Art");
    assert_eq!(view.pane(1).lines, ["/a/folder/long/enough/to/be/cut/on/its/row/of/seven/hundred/and/four/units/Art"]);
    assert!(view.pane(0).sentence.contains("/games"));

    let index_of_scraping = index(&view, "scraping");
    assert_eq!(view.choose(index_of_scraping), Outcome::Opened);
    assert_eq!(view.trail(), ["Library", "Scraping"]);
    assert_eq!(view.rows()[0].label, "Use ScreenScraper");
    assert!(view.rows().iter().all(|r| r.kind == RowKind::Steps));
    view.choose(0);
    assert_eq!(view.read.app["Scraping"], json!(false));
    assert_eq!(view.back(), Some(index_of_scraping), "B closes the page, back to the row that opened it");
    assert!(view.trail().is_empty());
    assert_eq!(view.rows()[3].value, "Off");
    assert_eq!(view.back(), None, "B on a section is the screens'");
}

#[test]
fn qt_7_7_emusens_panes_and_values_name_other_keys_and_the_version() {
    let source = fake(json!({"ResumeOnLaunch": "Restart", "BigScreen": true}), json!({}));
    let mut view = view(&source);
    section(&mut view, "EmuSen");
    let pane = view.pane(0);
    assert_eq!(pane.sentence, "States and saves live beside the games, with one resume point kept per game.");
    assert_eq!(pane.muted, ["Resume · Always start again"]);
    // Big Picture is the program on every machine (the tester decided, 2026-11-13): no Big picture row, whatever the base holds.
    let labels: Vec<String> = shown(&view).into_iter().map(|(label, _)| label).collect();
    assert_eq!(labels, ["Saves and states", "Pause in the background", "About"]);
    assert_eq!(view.rows()[2].value, emusen_qt_models::words::version_line());
    assert_eq!(view.rows()[2].kind, RowKind::Action);
    assert_eq!(view.choose(2), Outcome::Nothing);
}

#[test]
fn emusen_lists_neither_logging_nor_shell_until_their_screens_are_built() {
    // Decided by the tester, 2026-11-13: the two doors wait for their screens.
    let source = fake(json!({}), json!({}));
    let mut view = view(&source);
    section(&mut view, "EmuSen");
    let ids: Vec<String> = view.rows().into_iter().map(|r| r.id).collect();
    assert_eq!(ids, ["saves", "background", "about"]);
}

#[test]
fn qt_7_7_text_size_light_and_motion_apply_to_the_program() {
    let source = fake(json!({"TextSize": "default", "Appearance": "dark", "ReducedMotion": "system"}), json!({}));
    let mut view = view(&source);
    section(&mut view, "Display");
    let size = index(&view, "textsize");
    let pane = view.pane(size);
    assert_eq!(pane.preview, Preview::Sizes);
    assert_eq!(
        pane.samples,
        [("Default · The quick brown fox".to_string(), 1.0), ("Large · The quick brown fox".to_string(), 1.15), ("Larger · The quick brown fox".to_string(), 1.3)]
    );
    assert_eq!((view.text_size().as_str(), view.light(), view.reduced_motion()), ("default", false, false));
    view.step(size, 1);
    assert_eq!(view.text_size(), "large");
    view.step(index(&view, "variant"), 1);
    assert!(view.light());
    assert_eq!(view.pane(index(&view, "variant")).samples.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(), ["Dark", "Light"]);
    view.step(index(&view, "motion"), 1);
    assert!(view.reduced_motion());
    assert_eq!(view.pane(index(&view, "motion")).sentence, "Nothing moves: a page appears at once, and a fade takes a moment.");
}

#[test]
fn qt_7_7_a_stored_value_no_choice_holds_is_shown_and_kept() {
    let source = fake(json!({"SoundOutputDevice": "Headphones"}), json!({}));
    let mut view = view(&source);
    section(&mut view, "Sound");
    assert_eq!(view.rows()[3].value, "Headphones");
    view.step(3, 1);
    assert_eq!(view.rows()[3].value, "System default", "a step past the kept device wraps to the first");
    assert_eq!(source.saved.borrow().len(), 0, "the view saves nothing itself; the model saves what a step hands it");
}

#[test]
fn qt_7_7_e_a_folder_row_asks_the_folder_dialog_and_saves_its_answer_in_its_model() {
    let source = fake(json!({"RomDirectory": "/games", "FirmwareDirectory": ""}), json!({}));
    let mut view = view(&source);
    section(&mut view, "Library");
    let rom = index(&view, "rom");
    assert_eq!(view.rows()[rom].kind, RowKind::Opens);
    assert_eq!(view.choose(rom), Outcome::ChooseFolder { key: "AppSettings RomDirectory".into(), start: "/games".into() });
    let Outcome::Saved(Model::App, saved) = view.set_folder("AppSettings RomDirectory", "/new/games") else { panic!("the folder saved") };
    assert_eq!(saved["RomDirectory"], json!("/new/games"));
    assert_eq!(view.rows()[rom].value, "/new/games");
    for id in ["art", "media", "themefolder"] {
        assert!(matches!(view.choose(index(&view, id)), Outcome::ChooseFolder { .. }), "{id} asks the dialog too");
    }
}

#[test]
fn qt_7_7_h5_a_stored_deadzone_that_is_not_a_number_is_said_and_the_other_pad_settings_still_apply() {
    let (settings, unread) = view(&fake(json!({ "StickDeadzone": 0.3, "MirrorPlayer1ToPlayer2": true }), json!({}))).pad_settings().expect("read");
    assert_eq!((settings.stick_deadzone, unread), (Some(0.3), None));
    let (settings, unread) = view(&fake(json!({ "StickDeadzone": "most", "MirrorPlayer1ToPlayer2": true }), json!({}))).pad_settings().expect("read");
    assert_eq!(settings.stick_deadzone, None, "Endymion's deadzone left as it was");
    assert!(settings.mirror, "the other three still reach the pads");
    assert_eq!(unread, Some(settings::unread_deadzone("most")));
}

#[test]
fn qt_7_7_h5_every_controller_type_galaxia_offers_but_automatic_is_a_family_of_the_kits() {
    let rows: Value = serde_json::from_str(ROWS).unwrap();
    let row = rows["AppSettings"].as_array().unwrap().iter().find(|r| r["key"] == "ControllerType").expect("Galaxia's Controller type");
    for choice in row["choices"].as_array().unwrap().iter().filter_map(Value::as_str) {
        let family = view(&fake(json!({ "ControllerType": choice }), json!({}))).controller_family();
        assert_eq!(family.is_none(), choice == "Automatic", "{choice}");
        assert_eq!(family.map(|f| f.name()), (choice != "Automatic").then_some(choice));
    }
}

#[test]
fn sound_volume_and_play_time_are_read_as_the_program_takes_them_and_follow_a_step() {
    let source = fake(json!({"BigPictureInterface": {"NavigationVolume": 60}, "MaxPlayTimeTracking": 3}), json!({}));
    let mut view = view(&source);
    assert_eq!((view.navigation_volume(), view.play_hours()), (0.6, Some(3)));
    section(&mut view, "Sound");
    view.step(index(&view, "soundvolume"), -1);
    assert_eq!(view.navigation_volume(), 0.55, "a step is the volume at once");
    // Nothing stored: each row's initial value, Galaxia's.
    let unset = self::view(&fake(json!({}), json!({})));
    assert_eq!((unset.navigation_volume(), unset.play_hours()), (0.6, Some(8)));
}

#[test]
fn the_cheats_door_is_held_until_the_game_last_played_has_cheats_and_the_discs_door_names_its_tool() {
    let source = fake(json!({}), json!({}));
    let mut view = view(&source);
    section(&mut view, "Library");
    let cheats = index(&view, "cheats");
    assert!(view.rows()[cheats].held, "no game played");
    assert_eq!(view.pane(cheats).sentence, "The cheats of the game last played, turned on or off.");
    view.facts = json!({"last": "/roms/a.sfc", "lastCheats": 0});
    assert!(view.rows()[cheats].held, "a game with no cheats");
    view.facts = json!({"last": "/roms/a.sfc", "lastCheats": 2});
    assert!(!view.rows()[cheats].held);
    assert_eq!(view.choose(cheats), Outcome::Door("cheats".into()));
    assert_eq!(view.choose(index(&view, "discs")), Outcome::Door("compressed".into()), "the Compressed Discs tool's key");
}

#[test]
fn qt_3_2_the_page_s_key_and_each_row_s_facets_a_theme_reads() {
    let source = fake(json!({}), json!({}));
    let mut view = view(&source);
    section(&mut view, "Controllers");
    assert_eq!(view.page_key(), "controllers", "a section is its id");
    let facets = view.facets();
    assert_eq!(facets.len(), view.rows().len(), "a facet for every row shown");
    let swap = index(&view, "swap");
    assert_eq!(facets[swap].switch, Some(false), "Swap A and B is a switch, off at its default");
    view.step(swap, 1);
    assert_eq!(view.facets()[swap].switch, Some(true), "turned on, its state follows the stored value, not its word");
    assert!(view.rows().iter().zip(view.facets()).all(|(r, f)| f.about.is_empty() && f.tile.is_none() && (r.kind == RowKind::Steps || f.switch.is_none())), "no description writes an about line yet, no row here is a console's, only a stepped row is a switch");
    section(&mut view, "Library");
    let scraping = index(&view, "scraping");
    assert_eq!(view.choose(scraping), Outcome::Opened);
    assert_eq!(view.page_key(), "group:Scraping", "a page is its kind and what it is of");
    assert_eq!(view.facets()[0].switch, Some(true), "Use ScreenScraper, a switch on by default");
    view.back();
    assert_eq!(view.page_key(), "library");
}

#[test]
fn qt_3_4_11_display_s_rows_carry_their_board_s_short_lines_and_the_others_none() {
    let source = fake(json!({}), json!({}));
    let mut view = view(&source);
    section(&mut view, "Display");
    // EclipseSettings.dc.html's renderVals (gen_settings.py:153-161), the board's words for the nine rows it draws (R-20261116-20).
    let board = [
        ("shape", "The screen each console was made for, or each of the frame's pixels square."),
        ("crt", "The television's picture for every console that had one."),
        ("levers", "The CRT's own levers for one console: the mask, the bloom, the scanlines and the glass."),
        ("shaders", "The shader each console draws with: the built-in filters, the recent ones and the slang pack's presets."),
        ("crop", "What is hidden at the picture's edges on a console that had a television."),
        ("textsize", "Every word on the big screen, and the desktop's, through the interface's type scale."),
        ("art", "The focused game's art behind Home, dimmed and blurred."),
        ("motion", "No movement, shorter fades. System follows the desktop's own setting."),
        ("splash", "The eclipse while big picture starts. Any button skips it."),
    ];
    let facets = view.facets();
    for (id, about) in board {
        assert_eq!(facets[index(&view, id)].about, about, "{id}'s short line, the board's");
    }
    let drawn: Vec<usize> = board.iter().map(|(id, _)| index(&view, id)).collect();
    assert!((0..facets.len()).filter(|i| !drawn.contains(i)).all(|i| facets[i].about.is_empty()), "a row no board draws has no short line until it is ruled on");
}
