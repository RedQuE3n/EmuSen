//! Settings ▸ Consoles over the real runtime, built from this commit, in a scratch tree (EmuSen_Qt.md §7.7.E): a
//! synthetic core dropped in as a player's core is a console whose page is its declaration, with no code for it; its
//! firmware and a stored setting come back through the frontend API; and a hosted core's options, read
//! from a running session, are a page with its categories as headings.

use emusen_client_fixture::cores::{hosted, installation, libretro_entry, RING_GAME};
use emusen_client_fixture::{library, World, TEST_SYSTEM};
use emusen_crystal_client::{Api, Config, Open, Runtime, Scope};
use emusen_kit_core::settings::RowKind;
use emusen_qt_models::consoles::{self, ConsoleRead};
use emusen_qt_models::consoles_source::ClientConsoles;
use emusen_qt_models::settings::{Model, Outcome, SettingsSource, SettingsView};
use serde_json::{json, Value};
use std::collections::BTreeMap;

struct NoPlatform;

impl SettingsSource for NoPlatform {
    fn rows(&self) -> Result<Value, String> {
        Ok(serde_json::from_str(include_str!("../../../Platform/galaxia/data/frontend_settings.json")).unwrap())
    }
    fn load(&self, _: Model) -> Result<Value, String> {
        Ok(json!({}))
    }
    fn save(&self, _: Model, _: &Value) -> Result<(), String> {
        Ok(())
    }
    fn catalogue(&self) -> Result<Value, String> {
        Ok(json!({ "filters": [], "screens": [] }))
    }
    fn outputs(&self) -> Vec<String> {
        Vec::new()
    }
    fn folders(&self) -> BTreeMap<String, String> {
        BTreeMap::new()
    }
}

fn page(read: ConsoleRead, console: &str) -> SettingsView {
    let mut view = SettingsView::new();
    view.read = emusen_qt_models::settings::read(&NoPlatform).unwrap();
    view.read.consoles = read;
    view.choose_section(view.sections().iter().position(|s| s == "Consoles").unwrap());
    let at = view.rows().iter().position(|r| r.id == format!("console:{console}")).unwrap_or_else(|| panic!("no {console} in {:?}", view.rows()));
    assert_eq!(view.choose(at), Outcome::Opened);
    view
}

#[test]
fn qt_7_7_e_a_synthetic_core_dropped_in_is_a_console_whose_page_is_its_declaration() {
    let world = World::new("models-consoles");
    let player = world.dir("Cores");
    world.player_core(&player);
    let api = Api::load(library()).unwrap();
    let config = Config { development: false, cores: Some(Vec::new()), player_cores: Some(vec![player.to_str().unwrap().into()]) };
    let rt = Runtime::new(&api, &config).unwrap();
    let sha = rt.cores().unwrap().into_iter().find(|c| c.directory.as_deref() == Some("player")).and_then(|c| c.sha256).expect("the player's core");
    rt.core_approve(&sha).unwrap();
    let source = ClientConsoles { runtime: rt };

    let read = consoles::read(&source).unwrap();
    let test = read.consoles.iter().find(|c| c.systems.iter().any(|s| s == TEST_SYSTEM)).expect("the core's system is a console").clone();
    assert!(test.runs(), "{test:?}");
    let keys: Vec<&str> = test.schema.iter().map(|r| r.key.as_str()).collect();
    for key in ["Engine", "Wide", "Rate", "Ram", "Threads"] {
        assert!(keys.contains(&key), "the declaration's {key} in {keys:?}");
    }
    assert_eq!(test.firmware.iter().map(|(_, f)| f.file.as_str()).collect::<Vec<_>>(), ["boot.rom"]);
    // The card manager lists media for a console of the systems table alone (EmuSen_Qt.md §7.7.E names the gap), so the
    // test core's card is not a page yet; the page follows whatever the list says.
    assert_eq!(test.media, consoles::read(&source).unwrap().console(&test.console).unwrap().media);

    let console = test.console.clone();
    let mut view = page(read, &console);
    let rows = view.rows();
    let id = |key: &str| rows.iter().position(|r| r.id == key).unwrap_or_else(|| panic!("no row {key} in {rows:?}"));
    assert_eq!((rows[id("Wide")].value.as_str(), rows[id("Wide")].kind), ("Off", RowKind::Steps));
    assert_eq!(rows[id("Rate")].value, "32 kHz", "a choice by the label the core gives it");
    assert_eq!(rows[id("Ram")].value, "7");
    assert_eq!(rows[id("cards")].held, test.media.is_empty(), "Memory cards opens where the card manager lists a medium");
    assert_eq!(rows[id("firmware")].value, "⚠ Not found", "the boot ROM the core declares, required with no open replacement, and not in the tree");

    // A step stored through the runtime reads back as the console's value.
    let Outcome::Set { console: c, key, value } = view.step(id("Rate"), 1) else { panic!("a setting stored") };
    source.set(&c, &key, value.as_deref()).unwrap();
    assert_eq!(source.runtime.settings_get(0, Scope::Console, Some(&console), "Rate").unwrap().as_deref(), Some("48000"));
    let reread = consoles::read(&source).unwrap();
    assert_eq!(reread.console(&console).unwrap().value("Rate"), "48000");

}

#[test]
fn qt_7_7_e_a_hosted_cores_options_are_a_page_with_its_categories_as_headings() {
    let world = World::new("models-hosted");
    // SAFETY: set before any thread reads the environment; the proxy starts the runner it names.
    unsafe { std::env::set_var("EMUSEN_CORE_RUNNER", &hosted().runner) };
    let cores = installation(&world.dir("lib"));
    let core = libretro_entry(&world.dir("Home/Cores"));
    let config = Config { cores: Some(vec![cores.to_str().unwrap().into()]), player_cores: Some(vec![world.root.join("Home/Cores").to_str().unwrap().into()]), ..Config::default() };
    let rt = Runtime::new(&Api::load(library()).unwrap(), &config).unwrap();
    rt.core_approve(&emusen_client_fixture::cores::sha256(&core)).unwrap();
    let game = world.root.join("game.tcg");
    std::fs::write(&game, RING_GAME).unwrap();
    let session = rt.open(&Open { path: Some(game.to_str().unwrap().into()), ..Open::default() }).unwrap();
    let info = rt.session_info(session).unwrap();
    // The console's picture on, as Mistress's hosted audit sets it: the host then holds the core's deinterlacing.
    rt.settings_set(session, Scope::Console, None, "console_picture", Some("true")).unwrap();
    let rows = rt.settings_schema(session, None, None).unwrap();
    rt.close(session).unwrap();

    // The console as the runtime lists it, its schema the running core's options, as the console's page reads them.
    let source = ClientConsoles { runtime: rt };
    let mut read = consoles::read(&source).unwrap();
    let console = read.consoles.iter_mut().find(|c| c.console.eq_ignore_ascii_case(&info.console)).unwrap_or_else(|| panic!("{} listed", info.console));
    console.schema = rows;
    let name = console.console.clone();
    let view = page(read, &name);
    let shown = view.rows();
    let headings: Vec<&str> = shown.iter().filter(|r| r.id.starts_with("group:")).map(|r| r.label.as_str()).collect();
    assert_eq!(headings, ["Picture", "Sound", "Memory", "States", "Probe"], "the core's categories, in its order");
    assert!(shown.iter().filter(|r| r.id.starts_with("group:")).all(|r| r.held && r.kind == RowKind::Action));
    for key in ["tc_format", "tc_rate", "tc_hw"] {
        let row = shown.iter().find(|r| r.id == key).unwrap_or_else(|| panic!("{key} in {shown:?}"));
        assert_eq!(row.kind, RowKind::Steps, "{key}");
    }
    let held = shown.iter().position(|r| r.id == "tc_deinterlace").expect("the held option is a row");
    assert!(shown[held].held, "a row the host holds is drawn held, as Mistress's Locked");
    assert!(view.pane(held).sentence.contains("while the CRT filter is on"), "and its note says why: {}", view.pane(held).sentence);
    let format = shown.iter().position(|r| r.id == "tc_format").unwrap();
    assert_eq!(view.pane(format).sentence, "The format the picture is drawn in.", "the option's information is its sentence");
}
