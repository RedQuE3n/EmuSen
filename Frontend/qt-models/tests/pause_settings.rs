//! The Pause Card's Game Options ▸ Display and Sound without Qt (settings_card.rs): Settings' own rows for the running
//! console, the same labels and values as its page in Settings, a step stored as Settings stores it, a door's page under
//! the card's closed by B, and the card's pages turning to them and back.

use emusen_qt_models::consoles::Console;
use emusen_qt_models::pause::{self, Card, Outcome as CardOutcome, Page};
use emusen_qt_models::settings::{self, Model, Outcome, SettingsSource, SettingsView};
use emusen_qt_models::settings_card::{CardPage, CardRows};
use serde_json::{json, Value};

const ROWS: &str = include_str!("../../../Platform/galaxia/data/frontend_settings.json");

struct Fake {
    app: Value,
    graphics: Value,
}

impl SettingsSource for Fake {
    fn rows(&self) -> Result<Value, String> {
        Ok(serde_json::from_str(ROWS).unwrap())
    }
    fn load(&self, model: Model) -> Result<Value, String> {
        Ok(if model == Model::App { self.app.clone() } else { self.graphics.clone() })
    }
    fn save(&self, _: Model, _: &Value) -> Result<(), String> {
        Ok(())
    }
    fn catalogue(&self) -> Result<Value, String> {
        let f = |v: f32| u64::from(v.to_bits());
        Ok(json!({
            "filters": [
                {"key": "CRT", "name": "CRT", "consoles": ["NES"], "settings": [
                    {"id": "quality", "description": "Quality", "initial": f(1.0), "minimum": f(0.0), "maximum": f(2.0), "step": f(1.0), "choices": ["Performance", "Balanced", "Accurate"]},
                    {"id": "gamma", "description": "Tube gamma", "initial": f(2.4), "minimum": f(2.0), "maximum": f(2.8), "step": f(0.05), "choices": null}]},
                {"key": "LCD", "name": "Game Boy LCD", "consoles": ["GB"], "settings": []},
                {"key": "effect.None.nearest", "name": "None", "consoles": null, "settings": []}
            ],
            "screens": [{"console": "NES", "kind": "Television"}, {"console": "GB", "kind": "Lcd"}]
        }))
    }
    fn outputs(&self) -> Vec<String> {
        vec!["Speakers".into()]
    }
    fn folders(&self) -> std::collections::BTreeMap<String, String> {
        Default::default()
    }
}

fn view(app: Value, graphics: Value) -> SettingsView {
    let mut view = SettingsView::new();
    view.read = settings::read(&Fake { app, graphics }).unwrap();
    view.read.consoles.consoles = ["NES", "GB"].map(|c| Console { console: c.into(), name: format!("{c} name"), engines: vec!["Engine".into()], ..Console::default() }).to_vec();
    view
}

fn labels(rows: &CardRows) -> Vec<(String, String)> {
    rows.rows.iter().map(|r| (r.label.clone(), r.value.clone())).collect()
}

/// Settings' own page of `console`, as Settings ▸ Consoles opens it: each row's id, label and value.
fn settings_page(view: &mut SettingsView, console: &str) -> Vec<(String, String, String)> {
    let at = view.sections().iter().position(|s| s == "Consoles").unwrap();
    view.choose_section(at);
    let row = view.rows().iter().position(|r| r.id == format!("console:{console}")).unwrap();
    assert_eq!(view.choose(row), Outcome::Opened);
    let rows = view.rows().into_iter().map(|r| (r.id, r.label, r.value)).collect();
    view.choose_section(0);
    rows
}

#[test]
fn display_is_the_running_consoles_display_rows_as_its_settings_page_has_them() {
    let mut v = view(json!({}), json!({"Consoles": {"NES": {"Overscan": "tv"}}}));
    let page = settings_page(&mut v, "NES");
    v.card_open(CardPage::Display, "NES");
    let rows = v.card_rows();
    assert_eq!(rows.page, "display");
    assert_eq!(rows.title, "");
    assert_eq!(
        labels(&rows),
        [("CRT profile", "Off"), ("Shader levers", ""), ("Crop overscan", "TV overscan"), ("Picture shape", "TV shape")].map(|(l, v)| (l.to_owned(), v.to_owned())),
        "a television's rows"
    );
    for r in &rows.rows {
        assert!(page.contains(&(r.key.clone(), r.label.clone(), r.value.clone())), "{} as Settings ▸ Consoles ▸ NES has it", r.label);
    }
    let levers = &rows.rows[1];
    assert!(levers.opens && levers.dimmed, "Shader levers waits for the CRT, as on Settings' page");
    assert!(rows.rows[0].steps && rows.rows[2].steps && rows.rows[3].steps);
    assert_eq!(rows.rows[3].sentence, "The screen the console was made for, or each pixel square.", "the pane's words are Settings'");

    // A handheld's screen: its Screen row and its shape, no CRT.
    v.card_open(CardPage::Display, "GB");
    assert_eq!(labels(&v.card_rows()).iter().map(|(l, _)| l.as_str()).collect::<Vec<_>>(), ["Screen", "Picture shape"]);
}

#[test]
fn a_display_step_is_stored_as_settings_stores_it_and_leaves_settings_own_page_alone() {
    let mut v = view(json!({}), json!({}));
    let display = v.sections().iter().position(|s| s == "Display").unwrap();
    v.choose_section(display);
    v.card_open(CardPage::Display, "NES");
    let saved = v.card_settings_act(0, 1);
    let Outcome::Saved(Model::Graphics, document) = saved else { panic!("the CRT stepped is saved: {saved:?}") };
    assert_eq!(document["Consoles"]["NES"]["ScreenFilter"], "CRT");
    assert_eq!(document["ShaderParameters"]["NES"]["CRT"]["quality"], "0");
    assert_eq!(v.card_rows().rows[0].value, "Performance");
    assert_eq!(v.card_settings_act(3, 1), Outcome::Saved(Model::Graphics, v.read.graphics.clone()));
    assert_eq!(v.read.graphics["Consoles"]["NES"]["PictureShape"], "square");
    assert_eq!((v.section, v.page_open()), (display, false), "Settings' own section and pages as they were");
    assert_eq!(v.card_settings_act(9, 1), Outcome::Nothing, "no such row");
}

#[test]
fn shader_levers_opens_under_the_card_and_b_closes_it_on_its_door() {
    let mut v = view(json!({}), json!({"Consoles": {"NES": {"ScreenFilter": "CRT"}}}));
    v.card_open(CardPage::Display, "NES");
    assert!(!v.card_rows().rows[1].dimmed, "the CRT drawn: its levers open");
    assert_eq!(v.card_settings_act(1, 0), Outcome::Opened);
    let levers = v.card_rows();
    assert_eq!((levers.title.as_str(), levers.focus), ("Shader levers", 0));
    assert_eq!(labels(&levers), [("Quality", "Balanced"), ("Tube gamma", "2.4")].map(|(l, v)| (l.to_owned(), v.to_owned())));
    let stepped = v.card_settings_act(1, -1);
    let Outcome::All(parts) = stepped else { panic!("a lever is saved and set on the picture: {stepped:?}") };
    assert!(matches!(&parts[1], Outcome::Lever { console, id, value } if console == "NES" && id == "gamma" && (*value - 2.35).abs() < 1e-6));
    assert_eq!(v.card_rows().rows[1].value, "2.35");
    assert!(v.card_back(), "B closes the levers' page");
    let back = v.card_rows();
    assert_eq!((back.title.as_str(), back.focus, back.rows.len()), ("", 1, 4), "back on Display, on Shader levers");
    assert!(!v.card_back(), "on Display itself B is the card's");
    assert!(!v.page_open());
}

#[test]
fn sound_is_the_games_volume_and_output_stored_as_settings_stores_them() {
    let mut v = view(json!({"Volume": 0.5}), json!({}));
    v.card_open(CardPage::Sound, "NES");
    let rows = v.card_rows();
    assert_eq!(rows.page, "sound");
    assert_eq!(labels(&rows), [("Volume", "50%"), ("Output", "System default")].map(|(l, v)| (l.to_owned(), v.to_owned())), "the game's rows; the interface's sounds are not the game's");
    assert_eq!(rows.rows[0].sentence, "The game's volume. On the handheld this row defers to the device's quick menu.");
    let Outcome::Saved(Model::App, document) = v.card_settings_act(0, -1) else { panic!("the volume is saved") };
    assert_eq!(document["Volume"], json!(0.45), "stored as a number, as Settings stores it");
    assert_eq!(v.play().volume, 0.45, "the volume the session is handed");
    v.card_settings_act(1, 1);
    assert_eq!(v.play().output, "Speakers");
}

fn options(console: &str) -> Card {
    let mut card = Card::default();
    card.facts.cards.console = console.to_owned();
    card.open();
    let at = card.rows().iter().position(|r| r.key == pause::OPTIONS).unwrap();
    card.choose(at);
    card
}

fn on_display() -> (Card, SettingsView) {
    let mut card = options("NES");
    let at = card.rows().iter().position(|r| r.key == pause::DISPLAY).unwrap();
    assert_eq!(card.choose(at), CardOutcome::SettingsPage { page: CardPage::Display, console: "NES".into() });
    let mut v = view(json!({}), json!({"Consoles": {"NES": {"ScreenFilter": "CRT"}}}));
    v.card_open(CardPage::Display, "NES");
    card.take_settings(v.card_rows());
    (card, v)
}

#[test]
fn the_cards_display_draws_settings_rows_and_asks_settings_to_carry_them_out() {
    let (mut card, mut v) = on_display();
    assert_eq!((card.page, card.title(""), card.breadcrumb("Game")), (Page::Settings(CardPage::Display), "Display".to_owned(), "Game".to_owned()));
    let rows: Vec<_> = card.rows().into_iter().map(|r| (r.label, r.value, r.steps, r.opens)).collect();
    assert_eq!(rows[0], ("CRT profile".to_owned(), "Balanced".to_owned(), true, false));
    assert_eq!(rows[1], ("Shader levers".to_owned(), String::new(), false, true));
    assert_eq!(card.step(0, 1, false), CardOutcome::Settings { index: 0, by: 1 });
    assert_eq!(card.step(1, 1, false), CardOutcome::Nothing, "a door does not step");
    assert_eq!(card.choose(1), CardOutcome::Settings { index: 1, by: 0 });
    assert_eq!(card.help(), emusen_qt_models::help::Screen::PausePage, "the help bar of the card's other pages");
    assert_eq!(card.pane().sentence, "The CRT's own levers for the NES name.", "the pane is the row's in Settings");

    // The door's page: titled by Settings, B asks Settings to close it, and the focus comes back to the door.
    assert_eq!(v.card_settings_act(1, 0), Outcome::Opened);
    assert!(card.take_settings(v.card_rows()));
    assert_eq!((card.title(""), card.current), ("Shader levers".to_owned(), 0));
    assert_eq!(card.back(), CardOutcome::SettingsBack);
    assert!(v.card_back());
    assert!(card.take_settings(v.card_rows()));
    assert_eq!((card.title(""), card.current), ("Display".to_owned(), 1));

    // B on Display: Game Options, on Display's row.
    assert_eq!(card.back(), CardOutcome::Shown);
    assert_eq!(card.page, Page::Options);
    assert_eq!(card.rows()[card.current].key, pause::DISPLAY);
}

#[test]
fn the_cards_sound_turns_to_its_page_and_never_draws_another_pages_rows() {
    let (_, mut v) = on_display();
    let mut card = options("NES");
    // Display's rows still held as Sound opens: none are drawn until Settings gives Sound's.
    card.take_settings(v.card_rows());
    let at = card.rows().iter().position(|r| r.key == pause::SOUND).unwrap();
    assert_eq!(card.choose(at), CardOutcome::SettingsPage { page: CardPage::Sound, console: "NES".into() });
    assert!(card.rows().is_empty());
    v.card_open(CardPage::Sound, "NES");
    card.take_settings(v.card_rows());
    assert_eq!(card.title(""), "Sound");
    assert_eq!(card.rows().iter().map(|r| r.label.as_str()).collect::<Vec<_>>(), ["Volume", "Output"]);
    assert_eq!(card.back(), CardOutcome::Shown);
    assert_eq!(card.rows()[card.current].key, pause::SOUND);
}
