//! Settings ▸ Consoles without Qt (EmuSen_Qt.md §7.7.E), over fake sources: the list and its panes, a console's page
//! derived from what its engines declare (a synthetic core's schema shown with no code for it), the Engine page of
//! choices, Firmware, Memory cards, the shoulders, a reset, and the Pause Card's Memory Cards over the same description.

use emusen_crystal_client::shapes::{CoreRow, FirmwareFile, FirmwareItem, SettingRow, ShownWith, SystemRow};
use emusen_kit_core::settings::RowKind;
use emusen_qt_models::consoles::{self, ConsoleSource};
use emusen_qt_models::pause::{self, Card, Page as PausePage};
use emusen_qt_models::settings::{self, Model, Outcome, SettingsSource, SettingsView};
use serde_json::{json, Value};
use std::collections::BTreeMap;

const ROWS: &str = include_str!("../../../Platform/galaxia/data/frontend_settings.json");

struct Platform;

impl SettingsSource for Platform {
    fn rows(&self) -> Result<Value, String> {
        Ok(serde_json::from_str(ROWS).unwrap())
    }
    fn load(&self, _: Model) -> Result<Value, String> {
        Ok(json!({}))
    }
    fn save(&self, _: Model, _: &Value) -> Result<(), String> {
        Ok(())
    }
    fn catalogue(&self) -> Result<Value, String> {
        Ok(json!({
            "filters": [
                {"key": "CRT", "name": "CRT", "consoles": ["SNES", "PlayStation"], "settings": [{"id": "quality", "initial": 1.0f64.to_bits(), "choices": ["Performance", "Balanced", "Accurate"]}]},
                {"key": "effect.None.nearest", "name": "None", "consoles": null},
                {"key": "effect.Scanlines.nearest", "name": "Scanlines", "consoles": null}
            ],
            "screens": [{"console": "SNES", "kind": "Television"}, {"console": "PlayStation", "kind": "Television"}]
        }))
    }
    fn outputs(&self) -> Vec<String> {
        Vec::new()
    }
    fn folders(&self) -> BTreeMap<String, String> {
        [("firmware".to_owned(), "/emusen/Firmware".to_owned())].into()
    }
}

fn choice(key: &str, label: &str, choices: &[&str], labels: &[&str], default: &str) -> SettingRow {
    SettingRow {
        key: key.into(),
        label: label.into(),
        hint: Some(format!("{label}'s hint.")),
        kind: "choice".into(),
        default: Some(default.into()),
        choices: Some(choices.iter().map(|c| c.to_string()).collect()),
        choice_labels: Some(labels.iter().map(|c| c.to_string()).collect()),
        ..SettingRow::default()
    }
}

fn kind(key: &str, kind: &str, default: &str, range: Option<(f64, f64)>, group: Option<&str>) -> SettingRow {
    SettingRow {
        key: key.into(),
        label: key.into(),
        hint: Some(format!("{key}'s hint.")),
        kind: kind.into(),
        default: Some(default.into()),
        min: range.map(|r| r.0),
        max: range.map(|r| r.1),
        group: group.map(str::to_owned),
        ..SettingRow::default()
    }
}

fn system(id: &str, console: &str, name: &str, year: i32, engines: &[&str], starts_empty: bool) -> SystemRow {
    SystemRow {
        id: id.into(),
        console: console.into(),
        display_name: name.into(),
        release_year: year,
        engines: engines.iter().map(|e| e.to_string()).collect(),
        starts_empty,
        ..SystemRow::default()
    }
}

/// The runtime as a test world: the Super Nintendo, the PlayStation with a BIOS and cards, a synthetic core's console,
/// and a console no engine runs.
struct World {
    synthetic: Vec<SettingRow>,
    stored: BTreeMap<(String, String), String>,
}

impl World {
    fn new() -> World {
        // A synthetic core's declaration, as the crystal test core's: a switch, a choice, two counts, in two groups.
        let synthetic = vec![
            choice("Engine", "Engine", &["V1TestCore"], &[], "V1TestCore"),
            kind("Wide", "switch", "false", None, Some("Picture")),
            SettingRow { group: Some("Sound".into()), ..choice("Rate", "Rate", &["32000", "48000"], &["32 kHz", "48 kHz"], "32000") },
            kind("Ram", "count", "7", Some((1.0, 255.0)), Some("Machine")),
            kind("Threads", "count", "1", Some((1.0, 4.0)), Some("Machine")),
        ];
        World { synthetic, stored: BTreeMap::new() }
    }
}

impl ConsoleSource for World {
    fn systems(&self) -> Result<Vec<SystemRow>, String> {
        Ok(vec![
            system("saturn", "Saturn", "Saturn", 1994, &[], true),
            system("psx", "PlayStation", "PlayStation", 1994, &["Diamond"], true),
            SystemRow { firmware_name: None, ..system("snes", "SNES", "Super Nintendo", 1990, &["VenusRT", "Snes9x"], false) },
            system("test", "test", "Test system", 0, &["V1TestCore"], false),
        ]
        .into_iter()
        .map(|s| if s.id == "psx" { SystemRow { firmware_name: Some("PlayStation BIOS".into()), ..s } } else { s })
        .collect())
    }
    fn cores(&self) -> Vec<CoreRow> {
        Vec::new()
    }
    fn schema(&self, console: &str) -> Vec<SettingRow> {
        match console {
            "SNES" => vec![
                choice("Engine", "Engine", &["VenusRT", "Snes9x"], &[], "VenusRT"),
                choice("sa1_catch_up", "SA-1 catch-up", &["exact", "line"], &["After every S-CPU instruction (exact)", "Once a line"], "exact"),
            ],
            "PlayStation" => vec![
                choice("Engine", "Engine", &["Diamond"], &[], "Diamond"),
                choice("FirmwarePreference", "Firmware", &["auto", "replacement", "own"], &["Automatic", "Open replacement", "Own firmware"], "auto"),
                choice("Cards", "Memory Card", &["game", "shared", "named"], &["Per game", "Shared", "Named"], "shared"),
                SettingRow { shown_with: Some(ShownWith { key: "Cards".into(), value: "named".into() }), ..choice("Card1", "Memory Card: Slot 1", &["Card 1", "Card 2", "Card 3"], &[], "Card 1") },
                SettingRow { shown_with: Some(ShownWith { key: "Cards".into(), value: "named".into() }), ..choice("Card2", "Memory Card: Slot 2", &["Card 1", "Card 2", "Card 3"], &[], "Card 2") },
            ],
            "test" => self.synthetic.clone(),
            _ => Vec::new(),
        }
    }
    fn setting(&self, console: &str, key: &str) -> Option<String> {
        self.stored.get(&(console.to_owned(), key.to_owned())).cloned()
    }
    fn firmware(&self, system: &str) -> Option<Vec<FirmwareFile>> {
        let file = |label: &str, file: &str, status: &str, need: &str| FirmwareFile { label: label.into(), file: file.into(), required: status == "missing", status: status.into(), need: need.into() };
        Some(match system {
            "psx" => vec![
                file("PlayStation BIOS (Japan)", "scph5500.bin", "not_needed", "bios"),
                file("PlayStation BIOS (USA)", "scph5501.bin", "not_needed", "bios"),
                file("PlayStation BIOS (Europe)", "scph5502.bin", "not_needed", "bios"),
            ],
            "snes" => vec![file("DSP-1", "dsp1.rom", "missing", "")],
            _ => Vec::new(),
        })
    }
    fn overview(&self) -> Vec<FirmwareItem> {
        vec![FirmwareItem { file: "scph5501.bin".into(), effect: Some("accuracy".into()), cost: Some("Most games start and save as on a console. Some do not.".into()), ..FirmwareItem::default() }]
    }
    fn cards(&self, console: &str) -> Value {
        if console != "PlayStation" {
            return json!([]);
        }
        json!([{"medium": "psx.card", "title": "Memory Card", "noun": "Card", "extension": ".mcr", "kind": "removable", "mode": "shared",
                "mode_key": "Cards", "next_name": "Card 3",
                "slots": [{"label": "Slot 1", "name": "Card 1", "path": "/saves/PlayStation/Cards/Card 1.mcr"},
                          {"label": "Slot 2", "name": "Card 2", "path": "/saves/PlayStation/Cards/Card 2.mcr"}],
                "cards": [{"path": "/saves/PlayStation/Cards/Card 1.mcr", "name": "Card 1", "label": "Card 1", "games_own": false, "held_by": null},
                          {"path": "/saves/a.mcr", "name": "a", "label": "a", "games_own": true, "held_by": null}]}])
    }
}

fn view(world: &World) -> SettingsView {
    let mut view = SettingsView::new();
    view.read = settings::read(&Platform).unwrap();
    view.read.consoles = consoles::read(world).unwrap();
    view.facts = json!({ "systems": { "snes": 5, "psx": 2 } });
    let at = view.sections().iter().position(|s| s == "Consoles").unwrap();
    view.choose_section(at);
    view
}

fn index(view: &SettingsView, id: &str) -> usize {
    view.rows().iter().position(|r| r.id == id).unwrap_or_else(|| panic!("no row {id} in {:?}", view.rows().iter().map(|r| r.id.clone()).collect::<Vec<_>>()))
}

fn open(view: &mut SettingsView, console: &str) {
    let at = index(view, &format!("console:{console}"));
    assert_eq!(view.choose(at), Outcome::Opened);
}

#[test]
fn qt_7_7_e_every_console_is_listed_those_an_engine_runs_first_by_year() {
    let world = World::new();
    let mut view = view(&world);
    let rows = view.rows();
    let names: Vec<&str> = rows.iter().map(|r| r.label.as_str()).collect();
    assert_eq!(names, ["Super Nintendo", "PlayStation", "Test system", "Saturn"], "engines first, by release year, an unknown year last");
    assert!(rows[..3].iter().all(|r| r.kind == RowKind::Opens && !r.held && r.value.is_empty()));
    assert_eq!((rows[3].value.as_str(), rows[3].held), ("Not supported yet", true), "a console no engine runs is listed, held");
    assert_eq!(view.choose(3), Outcome::Nothing);
    assert_eq!(view.pane(3).sentence, "EmuSen has no engine for the Saturn yet.");

    // Mistress's pane: games, engine, firmware, the CRT and the first choice of the console's own, then what they are.
    let snes = view.pane(0);
    assert_eq!(snes.sentence, "Super Nintendo · 5 games · engine VenusRT");
    assert_eq!(snes.lines, ["✓ No firmware needed", "CRT profile Off · SA-1 catch-up After every S-CPU instruction (exact)"]);
    assert_eq!(snes.muted, ["What every Super Nintendo game shares, unless the game's own options say otherwise."]);
    assert_eq!(view.pane(1).lines[0], "No BIOS of your own: the open replacement runs in its place", "a console that starts empty names its BIOS");
}

#[test]
fn qt_7_7_e_a_synthetic_cores_declared_schema_is_its_page_with_no_code_for_it() {
    let mut world = World::new();
    let mut view = view(&world);
    open(&mut view, "test");
    assert_eq!(view.trail(), ["Consoles", "Test system"]);
    let rows = view.rows();
    let shown: Vec<(&str, &str, RowKind, bool)> = rows.iter().map(|r| (r.id.as_str(), r.value.as_str(), r.kind, r.held)).collect();
    assert_eq!(
        shown,
        [
            ("engine", "V1TestCore", RowKind::Steps, false),
            ("firmware", "none needed", RowKind::Opens, false),
            ("cards", "", RowKind::Opens, true),
            ("group:Picture", "", RowKind::Action, true),
            ("Wide", "Off", RowKind::Steps, false),
            ("group:Sound", "", RowKind::Action, true),
            ("Rate", "32 kHz", RowKind::Steps, false),
            ("group:Machine", "", RowKind::Action, true),
            ("Ram", "7", RowKind::Steps, false),
            ("Threads", "1", RowKind::Steps, false),
            ("screen", "None", RowKind::Steps, false),
            ("consoleshape", "TV shape", RowKind::Steps, false),
            ("reset", "", RowKind::Action, false),
        ],
        "each declared row by its kind, each group under a heading, the screen's rows for a console with no television"
    );
    assert_eq!(view.pane(index(&view, "group:Machine")).sentence, "The Test system's machine settings.");
    assert_eq!(view.pane(index(&view, "Ram")).sentence, "Ram's hint.");
    assert_eq!(view.pane(index(&view, "cards")).sentence, "The Test system's games keep their saves in the cartridge; there is no card to choose.");

    // Steps by kind, within the declaration: a switch turns, a choice wraps, a count stops at its end.
    let set = |o: Outcome| match o {
        Outcome::Set { console, key, value } => (console, key, value.unwrap_or_default()),
        other => panic!("not a setting stored: {other:?}"),
    };
    assert_eq!(set(view.step(index(&view, "Wide"), 1)), ("test".into(), "Wide".into(), "true".into()));
    assert_eq!(view.rows()[index(&view, "Wide")].value, "On", "the row shows it at once");
    assert_eq!(set(view.step(index(&view, "Rate"), -1)).2, "48000");
    let threads = index(&view, "Threads");
    for _ in 0..5 {
        view.step(threads, 1);
    }
    assert_eq!(view.rows()[threads].value, "4", "a count stops at its maximum");
    assert_eq!(view.step(index(&view, "group:Sound"), 1), Outcome::Nothing, "a heading takes no step");
    let screen = index(&view, "screen");
    assert!(matches!(view.step(screen, 1), Outcome::Saved(Model::Graphics, _)));
    assert_eq!(view.rows()[screen].value, "Scanlines", "the screen's filters are those of every console");

    // A setting the core declares tomorrow is a row tomorrow: the declaration alone changes.
    world.synthetic.push(SettingRow { group: Some("Machine".into()), ..kind("Turbo", "switch", "true", None, Some("Machine")) });
    let mut view = self::view(&world);
    open(&mut view, "test");
    assert_eq!(view.rows()[index(&view, "Turbo")].value, "On");
    assert_eq!(index(&view, "Turbo"), index(&view, "Threads") + 1, "under its group's heading");
}

#[test]
fn qt_7_7_e_engine_is_a_page_of_choices_on_the_value_cores_make_default_writes() {
    let world = World::new();
    let mut view = view(&world);
    open(&mut view, "SNES");
    let page = view.rows();
    let ids: Vec<&str> = page.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids, ["engine", "firmware", "cards", "sa1_catch_up", "crtprofile", "consolelevers", "consolecrop", "consoleshape", "reset"], "Mistress's page");
    assert_eq!(page[1].value, "only some games", "a cartridge's chip needed by some games");
    assert!(page[5].held, "Shader levers wait for the CRT to be chosen");
    let pane = view.pane(0);
    assert_eq!(pane.sentence, "VenusRT, the Super Nintendo's engine: every Super Nintendo game opens on it unless the game's own options choose another.");
    assert_eq!(pane.muted, ["Snes9x can be chosen here too; this is the same setting as Cores' Make Default."]);
    assert_eq!(view.help(), emusen_qt_models::help::Screen::SettingsConsole);

    assert_eq!(view.choose(0), Outcome::Opened);
    assert_eq!(view.trail(), ["Consoles", "Super Nintendo", "Engine"]);
    assert_eq!(view.help(), emusen_qt_models::help::Screen::SettingsChoices);
    let choices = view.rows();
    assert_eq!(choices.iter().map(|r| (r.label.as_str(), r.value.as_str(), r.marked, r.kind)).collect::<Vec<_>>(), [
        ("VenusRT", "Default", true, RowKind::Choice),
        ("Snes9x", "✓ Installed", false, RowKind::Choice)
    ]);
    assert_eq!(view.pane(1).sentence, "Snes9x would run every Super Nintendo game.");
    assert_eq!(view.pane(1).muted, ["A chooses; B keeps what was chosen."]);
    // A chooses and returns: the runtime's console setting Engine, the one Cores' Make Default writes.
    assert_eq!(view.choose(1), Outcome::Set { console: "SNES".into(), key: consoles::ENGINE_KEY.into(), value: Some("Snes9x".into()) });
    assert_eq!(view.trail(), ["Consoles", "Super Nintendo"]);
    assert_eq!(view.rows()[0].value, "Snes9x");
    // Left and Right still step it in place.
    assert!(matches!(view.step(0, 1), Outcome::Set { value: Some(v), .. } if v == "VenusRT"));
}

#[test]
fn qt_7_7_e_firmware_is_one_row_a_need_with_the_open_replacement_and_the_folders() {
    let world = World::new();
    let mut view = view(&world);
    open(&mut view, "PlayStation");
    let fw = index(&view, "firmware");
    assert_eq!(view.rows()[fw].value, "Not found");
    assert_eq!(view.choose(fw), Outcome::Opened);
    let rows: Vec<(String, String)> = view.rows().into_iter().map(|r| (r.label, r.value)).collect();
    let rows: Vec<(&str, &str)> = rows.iter().map(|(l, v)| (l.as_str(), v.as_str())).collect();
    assert_eq!(rows, [
        ("PlayStation BIOS", "Not found"),
        ("Open replacement", "In use when none is found"),
        ("Folder", "EmuSen's folder"),
        ("Use EmuSen's folder alone", ""),
        ("Open EmuSen's folder", ""),
        ("Check again", "")
    ], "the three regional files one need, named by the systems table");
    let need = view.pane(0);
    assert_eq!(need.sentence, "No PlayStation BIOS of your own was found, so Diamond runs its open replacement in its place.");
    assert_eq!(need.muted, [
        "Your own copy (scph5500.bin, scph5501.bin, scph5502.bin) is used once it is found. EmuSen never downloads firmware.",
        "The open version: Most games start and save as on a console."
    ]);
    assert_eq!(view.pane(1).sentence, "Your own firmware runs where it is found; where it is not, the open replacement runs in its place.");
    assert!(matches!(view.step(1, 1), Outcome::Set { key, value: Some(v), .. } if key == "FirmwarePreference" && v == "replacement"));
    assert_eq!(view.rows()[1].value, "Always");
    assert!(view.rows()[3].held, "no folder of the player's to forget");
    assert_eq!(view.choose(3), Outcome::Nothing);
    assert_eq!(view.choose(4), Outcome::OpenFolder("/emusen/Firmware".into()), "EmuSen's folder, through the program's one Open Folder");
    assert_eq!(view.choose(5), Outcome::Reread);
    assert_eq!(view.back(), Some(fw), "B returns to the Firmware row");
}

#[test]
fn qt_7_7_e_memory_cards_are_the_mode_the_slots_the_drawer_and_a_new_card() {
    let world = World::new();
    let mut view = view(&world);
    open(&mut view, "PlayStation");
    let ids: Vec<String> = view.rows().into_iter().map(|r| r.id).collect();
    assert_eq!(ids, ["engine", "firmware", "cards", "crtprofile", "consolelevers", "consolecrop", "consoleshape", "reset"], "the cards' and the firmware's keys on their own pages");
    assert_eq!(view.choose(index(&view, "cards")), Outcome::Opened);
    let rows = view.rows();
    let shown: Vec<(&str, &str, &str, RowKind, bool)> = rows.iter().map(|r| (r.id.as_str(), r.label.as_str(), r.value.as_str(), r.kind, r.held)).collect();
    assert_eq!(shown, [
        ("mode:psx.card", "Mode", "Shared", RowKind::Steps, false),
        ("slot:psx.card:0", "Slot 1", "Card 1", RowKind::Steps, true),
        ("slot:psx.card:1", "Slot 2", "Card 2", RowKind::Steps, true),
        ("cards:psx.card", "Cards", "1 card", RowKind::Opens, false),
        ("saves:psx.card", "Saves on the cards", "", RowKind::Opens, false),
        ("new:psx.card", "New card", "…", RowKind::Asks, false)
    ], "a slot the mode places is held until Named");
    let pane = view.pane(0);
    assert_eq!(pane.sentence, "Shared: every PlayStation game sees the same cards when they are shared, as the console did; per game gives each game its own.");
    assert_eq!(pane.muted, ["1 card in the drawer"]);
    assert_eq!(view.step(1, 1), Outcome::Nothing, "a held slot takes no step");
    assert!(matches!(view.step(0, 1), Outcome::Set { key, value: Some(v), .. } if key == "Cards" && v == "named"));
    assert!(!view.rows()[1].held, "under Named the slots are chosen");
    assert!(matches!(view.step(1, 1), Outcome::Set { key, value: Some(v), .. } if key == "Card1" && v == "Card 2"));
    assert_eq!(view.choose(5), Outcome::NewCard("/saves/PlayStation/Cards/Card 3.mcr".into()));
    assert_eq!(view.choose(4), Outcome::Door("cardsaves:PlayStation".into()));
    assert_eq!(view.choose(3), Outcome::Opened);
    assert_eq!(view.trail(), ["Consoles", "PlayStation", "Memory cards", "Memory Cards"]);
    assert_eq!(view.rows().iter().map(|r| r.label.as_str()).collect::<Vec<_>>(), ["Card 1"], "the drawer: its own cards, not a game's");
}

#[test]
fn qt_7_7_e_the_shoulders_step_the_consoles_an_engine_runs_and_reset_forgets_the_console() {
    let world = World::new();
    let mut view = view(&world);
    assert!(!view.shoulder(1), "on a section the shoulders are the strip's");
    open(&mut view, "PlayStation");
    assert!(view.shoulder(1));
    assert_eq!(view.trail(), ["Consoles", "Test system"]);
    assert!(view.shoulder(1), "round, past the console no engine runs");
    assert_eq!(view.trail(), ["Consoles", "Super Nintendo"]);
    assert!(view.shoulder(-1));
    assert_eq!(view.trail(), ["Consoles", "Test system"]);

    view.read.graphics = json!({ "Consoles": { "test": { "PictureShape": "square" }, "SNES": { "PictureShape": "square" } } });
    let Outcome::All(all) = view.choose(index(&view, "reset")) else { panic!("a reset is several changes") };
    assert!(all.contains(&Outcome::Set { console: "test".into(), key: "Wide".into(), value: None }), "every declared key forgotten");
    let Some(Outcome::Saved(Model::Graphics, graphics)) = all.last() else { panic!("the picture forgotten too") };
    assert_eq!(graphics["Consoles"], json!({ "SNES": { "PictureShape": "square" } }), "this console's alone");
}

#[test]
fn qt_7_7_e_the_pause_cards_memory_cards_are_the_same_description_slots_first() {
    let world = World::new();
    let read = consoles::read(&world).unwrap();
    let psx = read.console("PlayStation").unwrap();
    let mut card = Card::default();
    card.facts.discs.can_change = true;
    card.facts.cards = pause::CardFacts { console: "PlayStation".into(), media: psx.media.clone(), schema: psx.schema.clone(), values: BTreeMap::new() };
    // The facts cross from SessionModel to PauseModel as plain data.
    card.facts = pause::Facts::from_json(&card.facts.to_json());
    card.act(pause::OPTIONS);
    let at = card.rows().iter().position(|r| r.key == pause::MEMORY_CARDS).unwrap();
    card.choose(at);
    assert_eq!(card.page, PausePage::Cards);
    let rows: Vec<(String, String)> = card.rows().into_iter().map(|r| (r.label, r.value)).collect();
    assert_eq!(rows, [("Slot 1".into(), "Card 1".into()), ("Slot 2".into(), "Card 2".into()), ("Mode".into(), "Shared".into()), ("Cards".into(), "1 card".into())]);
    assert_eq!(card.pane().sentence, "Card 1 is in Slot 1. Left and Right choose another card; it goes in when the game next starts.");
    assert_eq!(card.step(2, 1, false), pause::Outcome::Setting { key: "Cards".into(), value: "named".into() });
    assert_eq!(card.step(0, 1, false), pause::Outcome::Setting { key: "Card1".into(), value: "Card 2".into() });
    card.back();
    assert_eq!(card.page, PausePage::Options);
}
