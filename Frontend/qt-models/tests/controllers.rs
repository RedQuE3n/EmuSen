//! Settings ▸ Controllers without Qt (EmuSen_Qt.md §7.7.H): the drawings as data and the controls they are named by,
//! the section's rows and panes, Players over the seats, Profiles and a profile's page, Hotkeys, the bindings page for
//! the pad and the keyboard, and the Pause Card's Controllers page, over a fake source holding Galaxia's committed rows
//! and Endymion's models in the shapes emusen_platform.h gives them.

use emusen_crystal_client::shapes::Controllers;
use emusen_kit_core::controller::{drawing_for, Drawing};
use emusen_qt_endymion::seats::{Ask, Connected, Player};
use emusen_qt_models::bindings::{Bindings, Choice, File};
use emusen_qt_models::consoles::{Console, ConsoleRead};
use emusen_qt_models::{controllers, settings_controllers};
use emusen_qt_models::settings::{self, Model, Outcome, Page, Preview, SettingsSource, SettingsView};
use emusen_qt_models::settings_controllers::{Chooser, ControlWord, Seated};
use emusen_qt_models::settings_source::ClientSettings;
use serde_json::{json, Value};
use std::sync::Arc;

const ROWS: &str = include_str!("../../../Platform/galaxia/data/frontend_settings.json");

struct Fake;

impl SettingsSource for Fake {
    fn rows(&self) -> Result<Value, String> {
        Ok(serde_json::from_str(ROWS).unwrap())
    }
    fn load(&self, _: Model) -> Result<Value, String> {
        Ok(json!({ "Rumble": true }))
    }
    fn save(&self, _: Model, _: &Value) -> Result<(), String> {
        Ok(())
    }
    fn catalogue(&self) -> Result<Value, String> {
        Ok(json!({}))
    }
    fn outputs(&self) -> Vec<String> {
        Vec::new()
    }
    fn folders(&self) -> std::collections::BTreeMap<String, String> {
        Default::default()
    }
}

/// Endymion itself, from the platform library built from this commit: its choice of a map, its game names and its
/// controls' words. Only the pad's bindings are the test's own answer (Y on West), since `bindings_bound` reads the
/// player's own files.
struct Rule;

fn endymion() -> &'static Arc<dyn Chooser> {
    static ENDYMION: std::sync::OnceLock<Arc<dyn Chooser>> = std::sync::OnceLock::new();
    ENDYMION.get_or_init(|| ClientSettings::open(&emusen_client_fixture::library().display().to_string()).unwrap().chooser().unwrap())
}

/// The profile every console has, as Endymion names it.
fn standard() -> String {
    endymion().choose(&json!({}), &json!({}), "", 1, None, None).unwrap().profile
}

impl Chooser for Rule {
    fn choose(&self, gamepad: &Value, profiles: &Value, console: &str, player: u32, profile: Option<&str>, game: Option<&str>) -> Result<Choice, String> {
        endymion().choose(gamepad, profiles, console, player, profile, game)
    }
    fn bound(&self, _: &str, _: u32) -> Result<[i32; 16], String> {
        let mut out = [-1; 16];
        out[1] = 2; // Y on West
        Ok(out)
    }
    fn game_key(&self, path: &str) -> Result<String, String> {
        endymion().game_key(path)
    }
    fn controls(&self) -> &[ControlWord] {
        endymion().controls()
    }
}

fn pad(buttons: &[u32], axes: &[u32], players: &[&str]) -> Controllers {
    Controllers { buttons: buttons.to_vec(), axes: axes.to_vec(), ports: players.len() as u32, players: players.iter().map(|p| Some((*p).to_owned())).collect() }
}

fn console(name: &str, shown: &str, year: i32, c: Controllers, rumble: bool) -> Console {
    Console { console: name.into(), name: shown.into(), systems: vec![name.to_lowercase()], engines: vec!["Engine".into()], year, controllers: c, rumble, ..Console::default() }
}

const SNES: [u32; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

fn bindings() -> Bindings {
    let pad_defaults = json!({"Up": "DPadUp", "Down": "DPadDown", "Left": "DPadLeft", "Right": "DPadRight", "B": "South", "A": "East", "Y": "West", "X": "North",
                              "L": "LeftShoulder", "R": "RightShoulder", "Start": "Start", "Select": "Back", "L3": "LeftStick", "R3": "RightStick"});
    let key_defaults = json!({"Up": "Up", "Down": "Down", "B": "Z", "A": "X", "Y": "A", "X": "S", "Start": "Return", "Select": "RightShift"});
    let hotkey_defaults = json!({"FastForward": "Tab", "Rewind": "Back", "SaveState": "F5", "LoadState": "F8", "TogglePause": "P"});
    Bindings {
        keys: json!({ "SNES": key_defaults, "PlayStation": key_defaults }),
        gamepad: json!({ "Consoles": { "SNES": pad_defaults, "PlayStation": pad_defaults }, "Players": {} }),
        profiles: json!({}),
        hotkeys: hotkey_defaults.clone(),
        key_defaults,
        pad_defaults,
        hotkey_defaults,
        default_profile: standard(),
    }
}

fn view() -> SettingsView {
    let mut view = SettingsView::new();
    view.read = settings::read(&Fake).unwrap();
    view.read.consoles = ConsoleRead {
        consoles: vec![
            console("SNES", "Super Nintendo", 1990, pad(&SNES, &[], &["snes.pad", "snes.pad"]), false),
            console("PlayStation", "PlayStation", 1994, pad(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15], &[0, 1, 2, 3], &["psx.dualshock", "psx.dualshock"]), true),
        ],
        ..ConsoleRead::default()
    };
    view.read.bindings = bindings();
    view.chooser = Some(Arc::new(Rule));
    view.seats = Seated {
        players: vec![Player { player: 1, pad: Some("Simulated pad".into()), kept: false }],
        pads: vec![Connected { key: 7, name: "Simulated pad".into(), player: 1, ..Connected::default() }],
        deadzone: 0.5,
    };
    view
}

fn index(view: &SettingsView, id: &str) -> usize {
    view.rows().iter().position(|r| r.id == id).unwrap_or_else(|| panic!("no row {id} in {:?}", view.rows().iter().map(|r| r.id.clone()).collect::<Vec<_>>()))
}

#[test]
fn qt_7_7_h_every_drawing_file_reads_and_covers_its_controllers_controls() {
    // The controls each in-tree core declares for the controller the file is named by (the cores' v1 descriptors).
    let declared: [(&str, &[u32], &[u32]); 12] = [
        ("snes.pad", &SNES, &[]),
        ("nes.pad", &[0, 2, 3, 4, 5, 6, 7, 8], &[]),
        ("gb.pad", &[0, 2, 3, 4, 5, 6, 7, 8], &[]),
        ("n64.pad", &[0, 3, 4, 5, 6, 7, 8, 10, 11, 12], &[0, 1, 2, 3]),
        ("md.pad3", &[0, 1, 3, 4, 5, 6, 7, 8], &[]),
        ("md.pad6", &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11], &[]),
        ("psx.pad", &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13], &[]),
        ("psx.dualshock", &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15], &[0, 1, 2, 3]),
        ("ps2.digital", &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13], &[]),
        ("ps2.dualshock2", &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15], &[0, 1, 2, 3]),
        ("sms.pad", &[0, 2, 3, 4, 5, 6, 7, 8], &[]),
        ("sg1000.pad", &[0, 3, 4, 5, 6, 7, 8], &[]),
    ];
    for (id, buttons, axes) in declared {
        let text = controllers::drawing(id).unwrap_or_else(|| panic!("{id} has a drawing"));
        Drawing::parse(text).unwrap_or_else(|e| panic!("{id}: {e}"));
        let controls = controllers::controls(&pad(buttons, axes, &[id]));
        let (_, generic) = drawing_for(Some(text), &controls);
        assert!(!generic, "{id}'s drawing has a region for each of {controls:?}");
    }
    assert_eq!(controllers::drawing("ps2.dualshock2"), controllers::drawing("psx.dualshock"), "one drawing for two ids");
    assert_eq!(controllers::drawing("test.pad"), None, "an id with no file");
    // A hosted core's RetroPad is drawn as its console's pad where the console has one (the tester's
    // R-20261111-113): the Saturn's and the Dreamcast's, each covering the controls their hosted cores give them; any other
    // console's RetroPad is the generic drawing.
    let hosted: [(&str, &[u32], &[u32]); 2] =
        [("saturn", &[0, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13], &[]), ("dreamcast", &[0, 1, 3, 4, 5, 6, 7, 8, 9, 12, 13], &[0, 1])];
    for (system, buttons, axes) in hosted {
        let c = Console { systems: vec![system.into()], controllers: pad(buttons, axes, &["retropad"]), ..Console::default() };
        let text = controllers::drawing_on(&c, 1).unwrap_or_else(|| panic!("{system}'s RetroPad has a drawing"));
        assert_eq!(Some(text), controllers::drawing(&format!("{system}.retropad")));
        Drawing::parse(text).unwrap_or_else(|e| panic!("{system}: {e}"));
        let (_, generic) = drawing_for(Some(text), &controllers::controls(&c.controllers));
        assert!(!generic, "{system}'s drawing has a region for each control");
    }
    let other = Console { systems: vec!["atari2600".into()], controllers: pad(&[0, 3, 4, 5, 6, 7], &[], &["retropad"]), ..Console::default() };
    assert_eq!(controllers::drawing_on(&other, 1), None, "another console's RetroPad is the generic drawing");
    assert_eq!(controllers::controls(&pad(&[0, 8], &[0, 1], &[])), ["B", "A", "LeftY-", "LeftY+", "LeftX-", "LeftX+"]);
}

#[test]
fn qt_7_7_h_the_section_counts_the_seats_names_the_profile_and_shows_rumble_where_a_console_shakes() {
    let mut view = view();
    view.choose_section(0);
    let rows = view.rows();
    let ids: Vec<&str> = rows.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(&ids[..8], ["players", "bindings", "profiles", "rumble", "swap", "chords", "keys", "hotkeys"]);
    assert_eq!(rows[0].value, "1 of 4 set");
    assert_eq!(rows[2].value, standard());
    let pane = view.pane(0);
    assert_eq!(pane.sentence, format!("Player 1 · Simulated pad · {} profile", standard()));
    assert_eq!(pane.muted, ["Players 2 and 3 and 4 · none", "Left and Right on a controller's row pick its player."]);
    let rumble = view.pane(3);
    assert_eq!(rumble.sentence, "Your controller shakes when the game drives its motor.");
    assert_eq!(rumble.muted, ["Games of these consoles can: PlayStation."]);
    // With no console's games driving a motor, there is no Rumble row.
    view.read.consoles.consoles[1].rumble = false;
    assert!(!view.rows().iter().any(|r| r.id == "rumble"));
}

#[test]
fn qt_7_7_h_players_seats_a_pad_forgets_a_kept_seat_and_ends_with_galaxias_player_rows() {
    let mut view = view();
    view.choose_section(0);
    view.seats.players.push(Player { player: 2, pad: Some("Old pad".into()), kept: true });
    assert_eq!(view.choose(0), Outcome::Opened);
    let rows = view.rows();
    assert_eq!((rows[0].label.as_str(), rows[0].value.as_str()), ("Simulated pad", "Player 1"));
    assert_eq!((rows[1].label.as_str(), rows[1].value.as_str()), ("Player 2: Old pad, disconnected", "Forget"));
    assert_eq!(rows.iter().skip(2).map(|r| r.id.as_str()).collect::<Vec<_>>(), ["keyboardplayer"], "the two seat rules hidden (R-20261112-14)");
    assert_eq!(view.step(0, 1), Outcome::Seat(Ask::Move { key: 7, player: 2 }));
    assert_eq!(view.step(0, -1), Outcome::Seat(Ask::Move { key: 7, player: 0 }), "Left from Player 1 is None");
    assert_eq!(view.choose(1), Outcome::Seat(Ask::Forget { player: 2 }));
    view.seats.pads[0].pressed = true;
    assert_eq!(view.rows()[0].value, "● Player 1", "a button held on it marks its row");
}

#[test]
fn qt_7_7_h_profiles_are_made_renamed_chosen_and_deleted_in_endymions_model() {
    let mut view = view();
    view.choose_section(0);
    let profiles = index(&view, "profiles");
    assert_eq!(view.pane(profiles).sentence, format!("Super Nintendo · 1 profile · {} starts its games", standard()));
    assert_eq!(view.choose(profiles), Outcome::Opened);
    let new = index(&view, "newprofile");
    let Outcome::Type { typed, prompt, .. } = view.choose(new) else { panic!("New profile asks for a name") };
    assert_eq!(prompt, "A name for the new Super Nintendo profile");
    assert_eq!(view.typed(&typed, "  Couch "), Outcome::Bindings(vec![File::Profiles]));
    assert_eq!(view.read.bindings.profiles("SNES"), [standard(), "Couch".into()]);
    assert_eq!(view.read.bindings.profiles["SNES"]["Profiles"]["Couch"]["1"]["Y"], "West", "a copy of the chosen profile's maps");
    let couch = index(&view, "profile:Couch");
    assert_eq!(view.choose(couch), Outcome::Opened);
    assert_eq!(view.trail(), ["Controllers", "Profiles", "Couch"]);
    assert_eq!(view.choose(index(&view, "pdefault")), Outcome::Bindings(vec![File::Profiles]));
    assert_eq!(view.read.bindings.console_default("SNES"), "Couch");
    let Outcome::Type { typed, .. } = view.choose(index(&view, "prename")) else { panic!("Rename asks") };
    view.typed(&typed, "Sofa");
    assert_eq!(view.read.bindings.console_default("SNES"), "Sofa", "the console's choice follows the name");
    assert_eq!(view.trail(), ["Controllers", "Profiles", "Sofa"]);
    assert_eq!(view.choose(index(&view, "pdelete")), Outcome::Bindings(vec![File::Profiles]));
    assert_eq!(view.read.bindings.profiles("SNES"), [standard()]);
    assert_eq!(view.read.bindings.console_default("SNES"), standard());
    assert_eq!(view.page(), Some(&Page::Profiles), "Delete closes the profile's page");
    // The profile every console has cannot be renamed or deleted.
    view.choose(index(&view, &format!("profile:{}", standard())));
    assert!(view.rows()[index(&view, "prename")].held && view.rows()[index(&view, "pdelete")].held);
}

#[test]
fn qt_7_7_h_the_bindings_page_draws_the_consoles_controller_and_rebinds_resets_and_tests_a_button() {
    let mut view = view();
    view.choose_section(0);
    view.choose(index(&view, "bindings"));
    let drawn = view.drawn().expect("the bindings page");
    assert_eq!(drawn.tabs, ["Player 1", "Super Nintendo", "PlayStation"]);
    assert_eq!(drawn.drawing, controllers::drawing("snes.pad").unwrap());
    assert_eq!(drawn.controls[..4], ["B", "Y", "Select", "Start"]);
    assert_eq!(drawn.texts[1], "West");
    // Left from B is Y, the nearest control that way on the drawing; nothing lies below B.
    assert!(!view.binding_move(0, 1));
    assert!(view.binding_move(-1, 0));
    let pane = view.pane(0);
    assert_eq!((pane.preview, pane.caption.as_str()), (Preview::Heading, "Y"));
    assert_eq!(pane.sentence, "Super Nintendo · Y");
    assert_eq!(pane.lines, ["Bound to the pad's West button."]);
    assert_eq!(pane.muted, ["A listens for a press and rebinds it; Y resets this button; the profile saves at once."]);
    assert!(view.binding_accept() && view.listening_pad());
    assert!(view.pane(0).lines[0].starts_with("● Press the button for Y on the pad."));
    assert_eq!(view.heard_down(1 << 3), Outcome::Bindings(vec![File::Gamepad]), "North pressed");
    assert_eq!(view.read.bindings.gamepad["Consoles"]["SNES"]["Y"], 3);
    assert_eq!(view.binding_act("reset"), Outcome::Bindings(vec![File::Gamepad]));
    assert_eq!(view.read.bindings.gamepad["Consoles"]["SNES"]["Y"], "West");
    // X: player 2, whose map follows player 1's until one of theirs is changed.
    assert_eq!(view.binding_act("player"), Outcome::Shown);
    assert_eq!(view.drawn().unwrap().tabs[0], "Player 2");
    view.binding_accept();
    view.heard_down(1);
    assert_eq!(view.read.bindings.gamepad["Players"]["SNES"]["2"]["Y"], 0, "player 2's own map, made from player 1's");
    assert_eq!(view.read.bindings.gamepad["Consoles"]["SNES"]["Y"], "West", "player 1's untouched");
    // The shoulders: the next console, its first control, its default profile.
    assert!(view.binding_shoulder(1));
    assert_eq!(view.drawn().unwrap().tabs[1], "PlayStation");
    assert_eq!(view.drawn().unwrap().focus, 0);
    // A trigger pulled for L2 binds it alone: the button beside it let go.
    let at = view.drawn().unwrap().controls.iter().position(|c| c == "L2").unwrap();
    assert!(view.binding_focus(at));
    assert_eq!(view.pane(0).lines[0], "Bound to the pad's Left Trigger.");
    view.binding_accept();
    view.heard_down(emusen_qt_models::settings_controllers::LEFT_TRIGGER_BIT);
    let own = &view.read.bindings.gamepad["Players"]["PlayStation"]["2"];
    assert!(own.is_object() && own.get("L2").is_none(), "{own}");
    // A stick's direction has no button to bind.
    let at = view.drawn().unwrap().controls.iter().position(|c| c.ends_with('-')).unwrap();
    assert!(view.binding_focus(at));
    assert_eq!(view.pane(0).lines[0], "Read from the pad's own stick, so it has no button to bind.");
    assert!(!view.binding_accept());
    // The tester lights the bound button held on the player's pad, and B stops it.
    view.binding_shoulder(-1);
    view.binding_act("player");
    assert_eq!(view.drawn().unwrap().tabs[0], "Player 1");
    assert_eq!(view.binding_act("test"), Outcome::Shown);
    let mut seated = view.seats.clone();
    seated.pads[0].buttons = 1 << 2;
    view.held = view.held_from(&seated).unwrap();
    assert_eq!(view.held, ["Y"]);
    assert_eq!(view.pane(0).lines, ["Held: Y"]);
    assert!(view.back_taken() && !view.testing());
    assert_eq!(view.back(), Some(1), "B closes the page onto its row");
}

#[test]
fn qt_7_7_h_keyboard_keys_and_hotkeys_take_a_key_from_each_other_and_name_a_clash() {
    let mut view = view();
    view.choose_section(0);
    view.choose(index(&view, "keys"));
    assert_eq!(view.drawn().unwrap().tabs[0], "Keyboard");
    assert_eq!(view.drawn().unwrap().texts[0], "Z");
    assert!(view.binding_accept());
    assert_eq!(view.heard_key(Some("Tab")), Outcome::Bindings(vec![File::Keys, File::Hotkeys]), "Tab taken from Fast Forward");
    assert_eq!(view.read.bindings.key("SNES", "B").as_deref(), Some("Tab"));
    assert_eq!(view.read.bindings.hotkey("FastForward"), None);
    assert_eq!(view.binding_act("clear"), Outcome::Bindings(vec![File::Keys]));
    assert_eq!(view.read.bindings.key("SNES", "B"), None);
    assert_eq!(view.binding_act("reset"), Outcome::Bindings(vec![File::Keys]));
    assert_eq!(view.read.bindings.key("SNES", "B").as_deref(), Some("Z"));
    view.binding_accept();
    assert_eq!(view.heard_key(None), Outcome::Shown, "Escape changes nothing");
    view.back();
    // Hotkeys: a key a console's button holds is marked, and taking it takes it from the console.
    view.choose(index(&view, "hotkeys"));
    let rows = view.rows();
    assert_eq!((rows[0].label.as_str(), rows[0].value.as_str()), ("Fast Forward", "Not bound"), "its Tab went to the button");
    assert_eq!((rows[1].label.as_str(), rows[1].value.as_str()), ("Rewind", "Back"));
    view.read.bindings.hotkeys["TogglePause"] = json!("Z");
    assert_eq!(view.rows()[4].value, "⚠ Z");
    assert!(view.pane(4).lines.contains(&"⚠ Conflict: the Super Nintendo also has Z for a button.".to_owned()));
    view.choose(0);
    assert_eq!(view.rows()[0].value, "Press a key…");
    assert_eq!(view.heard_key(Some("Z")), Outcome::Bindings(vec![File::Hotkeys, File::Keys]));
    assert_eq!(view.read.bindings.key("SNES", "B"), None);
    assert_eq!(view.rows()[0].value, "⚠ Z", "Pause / Resume holds Z as well");
    assert_eq!(view.clear_hotkey(0), Outcome::Bindings(vec![File::Hotkeys]));
    assert_eq!(view.rows()[0].value, "Not bound");
    // Reset every binding: keys, pads and hotkeys at their defaults, a named profile kept.
    view.back();
    view.read.bindings.create("SNES", "Couch", &standard());
    view.choose(index(&view, "resetbindings"));
    assert_eq!(view.read.bindings.hotkey("FastForward").as_deref(), Some("Tab"));
    assert_eq!(view.read.bindings.key("SNES", "B").as_deref(), Some("Z"));
    assert_eq!(view.read.bindings.profiles("SNES"), [standard(), "Couch".into()]);
}

#[test]
fn qt_7_7_h_keyboard_keys_shows_and_binds_a_sticks_directions_by_the_files_words() {
    let mut view = view();
    view.read.bindings.keys["PlayStation"]["LeftStickUp"] = json!("I");
    view.choose_section(0);
    view.choose(index(&view, "keys"));
    // To the PlayStation, whose DualShock's sticks are controls; Left stick up is bound to I in the file's words.
    assert!(view.binding_shoulder(1));
    let drawn = view.drawn().unwrap();
    let up = drawn.controls.iter().position(|c| c == "LeftY-").expect("the left stick's up");
    assert_eq!(drawn.texts[up], "I");
    assert!(view.binding_focus(up));
    assert_eq!(view.pane(0).lines[0], "Bound to the key I.", "the pane names its key");
    assert!(view.binding_accept());
    assert_eq!(view.heard_key(Some("K")), Outcome::Bindings(vec![File::Keys]));
    assert_eq!(view.read.bindings.key("PlayStation", "LeftStickUp").as_deref(), Some("K"), "written under the file's word");
    assert_eq!(view.read.bindings.key("PlayStation", "LeftY-"), None, "never under the drawing's");
    assert_eq!(view.binding_act("clear"), Outcome::Bindings(vec![File::Keys]));
    assert_eq!(view.read.bindings.key("PlayStation", "LeftStickUp"), None);
}

#[test]
fn qt_7_7_h_the_cards_controllers_page_seats_chooses_the_games_profile_and_opens_the_drawing() {
    let mut view = view();
    view.facts = json!({ "game": "/roms/snes/Hero.sfc", "system": "snes" });
    let rows = view.game_rows();
    assert_eq!(rows.iter().map(|r| (r.label.as_str(), r.value.as_str())).collect::<Vec<_>>(),
        [("Player 1", "Simulated pad"), ("Player 2", "None"), ("Profile", standard().as_str()), ("Bindings", ""), ("Swap Players", "")]);
    assert_eq!(rows[0].sentence, format!("Player 1 plays on the Simulated pad, with the {} profile.", standard()));
    assert_eq!(rows[0].muted, "Left and Right pick another connected controller; Bindings opens the drawing for this console.");
    assert!(rows[4].dimmed, "no player 2");
    assert_eq!(view.game_act(1, 1), Outcome::Seat(Ask::Move { key: 7, player: 2 }));
    view.read.bindings.create("SNES", "Couch", &standard());
    assert_eq!(view.game_act(2, 1), Outcome::Bindings(vec![File::Profiles]));
    assert_eq!(view.read.bindings.profiles["SNES"]["Games"]["Hero"], "Couch");
    assert_eq!(view.game_act(3, 0), Outcome::Door("cardBindings".into()));
    assert_eq!(view.drawn().unwrap().tabs[1], "Super Nintendo");
    assert_eq!(view.back(), None, "B from the card's drawing leaves Settings");
}

#[test]
fn qt_7_7_h_the_tester_lights_keys_and_pad_moves_the_sticks_and_holds_b_a_second() {
    let mut view = view();
    view.choose_section(0);
    view.choose(index(&view, "bindings"));
    view.binding_shoulder(1);
    assert_eq!(view.drawn().unwrap().tabs[1], "PlayStation");
    assert!(!view.tester_key("X", true).unwrap(), "a key is the tester's only while it runs");
    assert_eq!(view.binding_act("test"), Outcome::Shown);
    assert_eq!(view.tester_hold_ms(), 1000, "B held a second leaves, as Mistress's TestExitHold");
    // A key held lights the control it is bound to on the console's keyboard map, and lets it go when released.
    assert!(view.tester_key("x", true).unwrap());
    assert_eq!(view.held, ["A"]);
    assert!(view.tester_key("X", false).unwrap());
    assert!(view.held.is_empty());
    // The left stick pushed right and up: its knob stands there, and up and right light past Endymion's deadzone.
    let mut seated = view.seats.clone();
    seated.pads[0].axes = [0.75, -0.25, 0.0, 0.0, 0.0, 0.0];
    view.held = view.held_from(&seated).unwrap();
    view.pushes = view.pushes_from(&seated);
    let drawn = view.drawn().unwrap();
    let push = |c: &str| drawn.pushes[drawn.controls.iter().position(|x| x == c).unwrap()];
    assert_eq!((push("LeftX+"), push("LeftX-"), push("LeftY-"), push("LeftY+")), (0.75, 0.0, 0.25, 0.0));
    assert_eq!(view.held, ["LeftX+"], "up at a quarter is under the deadzone");
    seated.deadzone = 0.2;
    assert_eq!(view.held_from(&seated).unwrap(), ["LeftY-", "LeftX+"], "the deadzone is Endymion's, not a number of the page's");
    // Stopped, nothing is held or pushed and the knobs are at rest.
    assert!(view.back_taken() && !view.testing());
    assert!(view.held.is_empty() && view.pushes.is_empty() && view.drawn().unwrap().pushes.is_empty());
}

/// Endymion failing each call, for the failures' tests: its controls' words still Endymion's own.
struct Failing {
    choose: bool,
    game_key: bool,
}

impl Chooser for Failing {
    fn choose(&self, gamepad: &Value, profiles: &Value, console: &str, player: u32, profile: Option<&str>, game: Option<&str>) -> Result<Choice, String> {
        if self.choose {
            return Err("no map".into());
        }
        endymion().choose(gamepad, profiles, console, player, profile, game)
    }
    fn bound(&self, _: &str, _: u32) -> Result<[i32; 16], String> {
        Err("no files".into())
    }
    fn game_key(&self, path: &str) -> Result<String, String> {
        if self.game_key {
            return Err("no name".into());
        }
        endymion().game_key(path)
    }
    fn controls(&self) -> &[ControlWord] {
        endymion().controls()
    }
}

/// Endymion naming no control's word, for a key that must not be written under the drawing's name.
struct Wordless;

impl Chooser for Wordless {
    fn choose(&self, gamepad: &Value, profiles: &Value, console: &str, player: u32, profile: Option<&str>, game: Option<&str>) -> Result<Choice, String> {
        endymion().choose(gamepad, profiles, console, player, profile, game)
    }
    fn bound(&self, console: &str, player: u32) -> Result<[i32; 16], String> {
        Rule.bound(console, player)
    }
    fn game_key(&self, path: &str) -> Result<String, String> {
        endymion().game_key(path)
    }
    fn controls(&self) -> &[ControlWord] {
        &[]
    }
}

#[test]
fn qt_7_7_h5_a_map_endymion_cannot_choose_is_said_and_nothing_is_written() {
    let mut view = view();
    view.chooser = Some(Arc::new(Failing { choose: true, game_key: false }));
    view.choose_section(0);
    view.choose(index(&view, "bindings"));
    let words = settings_controllers::unchosen("no map");
    assert_eq!(view.controllers_problem().as_deref(), Some(words.as_str()), "the page says the failure");
    let y = view.drawn().unwrap().controls.iter().position(|c| c == "Y").unwrap();
    assert_eq!(view.drawn().unwrap().texts[y], "", "no binding shown in place of Endymion's map");
    assert!(view.binding_focus(y));
    let before = view.read.bindings.gamepad.clone();
    assert_eq!(view.binding_act("reset"), Outcome::Failed(words.clone()));
    assert!(view.binding_accept());
    assert_eq!(view.heard_pad(Some(3), None), Outcome::Failed(words.clone()));
    assert_eq!(view.read.bindings.gamepad, before, "nothing written under a fallback map");
}

#[test]
fn qt_7_7_h5_the_pads_bindings_unread_light_nothing_and_are_said() {
    let mut view = view();
    view.chooser = Some(Arc::new(Failing { choose: false, game_key: false }));
    view.choose_section(0);
    view.choose(index(&view, "bindings"));
    assert_eq!(view.binding_act("test"), Outcome::Shown);
    let mut seated = view.seats.clone();
    seated.pads[0].buttons = 1 << 2;
    assert_eq!(view.held_from(&seated), Err(settings_controllers::unbound("no files")));
}

#[test]
fn qt_7_7_h5_the_cards_profile_fails_in_words_where_endymion_cannot_choose_or_name_the_game() {
    let mut view = view();
    view.facts = json!({ "game": "/roms/snes/Hero.sfc", "system": "snes" });
    view.read.bindings.create("SNES", "Couch", &standard());
    view.chooser = Some(Arc::new(Failing { choose: false, game_key: true }));
    let before = view.read.bindings.profiles.clone();
    assert_eq!(view.game_act(2, 1), Outcome::Failed(settings_controllers::unnamed("no name")));
    assert_eq!(view.read.bindings.profiles, before, "nothing written under an empty key");
    view.chooser = Some(Arc::new(Failing { choose: true, game_key: false }));
    let words = settings_controllers::unchosen("no map");
    let keys: Vec<String> = view.game_rows().into_iter().map(|r| r.key).collect();
    assert!(!keys.iter().any(|k| k == "gprofile" || k == "gbindings"), "no profile shown or opened on a fallback: {keys:?}");
    assert_eq!(view.controllers_problem(), Some(words.clone()), "and the failure is said");
    assert_eq!(view.open_card_bindings("SNES"), Err(words), "the drawing is not opened on a fallback profile");
}

#[test]
fn qt_7_7_h5_a_key_endymion_names_no_word_for_is_said_and_not_written() {
    let mut view = view();
    view.chooser = Some(Arc::new(Wordless));
    view.choose_section(0);
    view.choose(index(&view, "keys"));
    let before = view.read.bindings.keys.clone();
    assert!(view.binding_accept());
    let control = view.drawn().unwrap().controls[0].clone();
    assert_eq!(view.heard_key(Some("K")), Outcome::Failed(settings_controllers::unworded(&control)));
    assert_eq!(view.binding_act("clear"), Outcome::Failed(settings_controllers::unworded(&control)));
    assert_eq!(view.binding_act("reset"), Outcome::Failed(settings_controllers::unworded(&control)));
    assert_eq!(view.read.bindings.keys, before, "nothing written under the drawing's name");
}

/// The acts list: each row a page may show, with the test that shows it acts (tests/controllers_acts.tsv).
fn acts(page: &str) -> Vec<(String, String)> {
    include_str!("controllers_acts.tsv")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .map(|l| l.split('\t').collect::<Vec<_>>())
        .filter(|f| f[0] == page)
        .map(|f| (f[1].to_owned(), f[2].to_owned()))
        .collect()
}

/// Each row the page shows is on the acts list, so none only saves.
#[track_caller]
fn every_row_acts(view: &SettingsView, page: &str) {
    let listed = acts(page);
    for row in view.rows() {
        let found = listed.iter().any(|(id, _)| id.strip_suffix('*').map_or(*id == row.id, |stem| row.id.starts_with(stem)));
        assert!(found, "{page}: the row {} ({}) is on no acts list: it would only save", row.id, row.label);
    }
}

#[test]
fn qt_7_7_h8_no_controllers_page_shows_a_row_that_only_saves() {
    let mut view = view();
    // Endymion's own hotkeys, so an action it adds is a row here.
    let platform = emusen_platform_client::Platform::load(emusen_client_fixture::library()).unwrap();
    view.read.bindings.hotkey_defaults = platform.bindings_defaults(emusen_platform_client::bindings::consts::HOTKEYS).unwrap();
    view.seats.players.push(Player { player: 2, pad: Some("Old pad".into()), kept: true });
    view.choose_section(0);
    every_row_acts(&view, "controllers");
    assert!(!view.rows().iter().any(|r| r.id == "keyboard"), "On-screen keyboard hidden (R-20261112-13)");
    view.choose(index(&view, "players"));
    every_row_acts(&view, "players");
    view.back();
    view.choose(index(&view, "hotkeys"));
    every_row_acts(&view, "hotkeys");
    assert!(!view.rows().iter().any(|r| r.id == "hk:ToggleBigPicture"), "Big Picture hidden (R-20261112-12)");
    assert_eq!(view.rows().len(), 9, "Endymion's ten hotkeys but Big Picture");
}

#[test]
fn qt_7_7_h8_every_test_the_acts_list_names_is_in_the_tree() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut sources = String::new();
    for dir in ["qt-models/tests", "qt-app/tests", "qt-endymion/tests"] {
        for file in std::fs::read_dir(root.join(dir)).unwrap().flatten().filter(|f| f.path().extension().is_some_and(|e| e == "rs")) {
            sources.push_str(&std::fs::read_to_string(file.path()).unwrap());
        }
    }
    for page in ["controllers", "players", "hotkeys"] {
        for (row, test) in acts(page) {
            assert!(sources.contains(&format!("fn {test}(")), "{page} {row}: no test {test}");
        }
    }
}

#[test]
fn qt_7_7_h8_a_hidden_rows_stored_value_is_left_as_the_file_holds_it() {
    let mut view = view();
    view.read.app = json!({ "Rumble": true, "SwapPadButtons": false, "OnScreenKeyboard": "Steam", "ClaimPlayersByPress": true, "ExternalControllerFirst": true });
    view.choose_section(0);
    let Outcome::Saved(_, document) = view.step(index(&view, "swap"), 1) else { panic!("Swap A and B saves") };
    assert_eq!(document["SwapPadButtons"], true);
    for (key, kept) in [("OnScreenKeyboard", json!("Steam")), ("ClaimPlayersByPress", json!(true)), ("ExternalControllerFirst", json!(true))] {
        assert_eq!(document[key], kept, "{key} as the older file holds it");
    }
}

#[test]
fn qt_7_7_h8_the_screens_hotkey_words_are_endymions_actions() {
    let platform = emusen_platform_client::Platform::load(emusen_client_fixture::library()).unwrap();
    let defaults = platform.bindings_defaults(emusen_platform_client::bindings::consts::HOTKEYS).unwrap();
    let description: Value = serde_json::from_str(include_str!("../data/settings_screen.json")).unwrap();
    let described: Vec<&str> = description["controllers"]["actions"].as_array().unwrap().iter().map(|p| p[0].as_str().unwrap()).collect();
    let hidden: Vec<&str> = description["controllers"]["hidden"].as_array().unwrap().iter().filter_map(|h| h["hotkey"].as_str()).collect();
    for action in &described {
        assert!(defaults.get(*action).is_some(), "the screen words {action}, which Endymion has no hotkey for");
    }
    for action in defaults.as_object().unwrap().keys() {
        assert!(described.contains(&action.as_str()) != hidden.contains(&action.as_str()), "{action}: worded on the screen or hidden, one of the two");
    }
}
