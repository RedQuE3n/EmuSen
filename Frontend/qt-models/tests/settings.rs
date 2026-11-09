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
        let balanced = 1.0f64.to_bits();
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
    assert_eq!(view.rows()[0].kind, RowKind::Opens);
    assert_eq!(view.pane(0).sentence, "3 games under /games, read in place and never moved.");
    assert_eq!(view.choose(index(&view, "themes")), Outcome::Door("themes".into()));
    // A folder cut on its row shows whole in the pane once focused.
    view.read.app["ArtworkDirectory"] = json!("/a/folder/long/enough/to/be/cut/on/its/row/of/seven/hundred/and/four/units/Art");
    assert_eq!(view.pane(1).lines, ["/a/folder/long/enough/to/be/cut/on/its/row/of/seven/hundred/and/four/units/Art"]);
    assert!(view.pane(0).sentence.contains("/games"));

    assert_eq!(view.choose(index(&view, "scraping")), Outcome::Opened);
    assert_eq!(view.trail(), ["Library", "Scraping"]);
    assert_eq!(view.rows()[0].label, "Use ScreenScraper");
    assert!(view.rows().iter().all(|r| r.kind == RowKind::Steps));
    view.choose(0);
    assert_eq!(view.read.app["Scraping"], json!(false));
    assert!(view.back(), "B closes the page");
    assert!(view.trail().is_empty());
    assert_eq!(view.rows()[3].value, "Off");
    assert!(!view.back(), "B on a section is the screens'");
}

#[test]
fn qt_7_7_emusens_panes_and_values_name_other_keys_and_the_version() {
    let source = fake(json!({"ResumeOnLaunch": "Restart", "BigScreen": true}), json!({}));
    let mut view = view(&source);
    section(&mut view, "EmuSen");
    let pane = view.pane(0);
    assert_eq!(pane.sentence, "States and saves live beside the games, with one resume point kept per game.");
    assert_eq!(pane.muted, ["Resume · Always start again"]);
    assert_eq!(shown(&view)[1], ("Big picture".into(), "On startup".into()));
    assert_eq!(view.rows()[5].value, emusen_qt_models::words::version_line());
    assert_eq!(view.rows()[5].kind, RowKind::Action);
    assert_eq!(view.choose(5), Outcome::Nothing);
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
    assert_eq!((view.text_scale(), view.light(), view.reduced_motion()), (1.0, false, false));
    view.step(size, 1);
    assert_eq!(view.text_scale(), 1.15);
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
