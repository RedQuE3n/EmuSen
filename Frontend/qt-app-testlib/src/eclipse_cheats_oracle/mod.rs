//! Oracle for Eclipse Cheats (#21) on software and GPU backends.
#![allow(dead_code)]

use crate::common::cheats::{database_folder, SNES};
use crate::common::wait_for;
use crate::eclipse_board::{glyphs_in, held, kit_tokens, picture, read, rows, words_under};
use emusen_client_fixture::{library, World};
use emusen_crystal_client::{consts, Api, Config, Open, Runtime};
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::{MODEL_APP_SETTINGS, MODEL_CHEAT_FILE};
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_models::screens::qobject::Screens;
use emusen_qt_models::tool_model::qobject::ToolModel;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::{Harness, Scene};
use serde_json::{json, Value};
use std::pin::Pin;
use std::sync::Arc;

const ECLIPSE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../themes/eclipse");
const TABLE_CHEATS: &str = include_str!("../../../qt-app/tests/eclipse/cheats.tsv");

fn midnight() -> i64 {
    emusen_qt_models::words::now().div_euclid(86_400) * 86_400
}

fn until(scene: &mut Scene<'_>, what: &str, ms: i32, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    let mut waited = 0;
    while !done(scene) && waited < ms {
        scene.wait(40);
        waited += 40;
    }
    assert!(
        done(scene),
        "{what} within {ms} ms (shown={}, eclipseScreen={}, view={})",
        scene.property("screens", "shown"),
        scene.property("jobsModel", "eclipseScreen"),
        scene.property("jobsModel", "eclipseCores")
    );
}

fn view_json(scene: &mut Scene<'_>) -> Value {
    serde_json::from_str(&scene.property("jobsModel", "eclipseCores")).unwrap_or(Value::Null)
}

fn freeze_header(scene: &mut Scene<'_>) {
    assert!(scene.set_property("clock", "now", &(midnight() + 21 * 3600 + 41 * 60).to_string()));
    assert!(scene.set_property("jobs", "battery", "82"));
    assert!(scene.set_property("jobs", "clockText", "9:41 PM"));
    scene.wait(200);
}

fn click_named(scene: &mut Scene<'_>, name: &str) {
    let tree = scene.tree();
    let n = tree
        .nodes
        .iter()
        .find(|n| n.name == name && n.visible)
        .unwrap_or_else(|| panic!("visible {name} not found in tree"));
    scene.click(n.x + n.w / 2.0, n.y + n.h / 2.0);
}

fn confirm_question(scene: &mut Scene<'_>) {
    wait_for(scene, "jobsModel", "asking", "true", 2000);
    wait_for(scene, "jobsQuestion", "opened", "true", 3000);
    scene.wait(250);
    scene.pad(UiButton::Right);
    scene.wait(120);
    scene.pad(UiButton::Accept);
    wait_for(scene, "jobsModel", "asking", "false", 2000);
    wait_for(scene, "jobsQuestion", "opened", "false", 2000);
    wait_for(scene, "jobsCheatsView", "sheetOpen", "false", 2000);
    scene.wait(80);
}

fn open_eclipse_tool(scene: &mut Scene<'_>, want_tool: &str) {
    scene.pad(UiButton::Menu);
    wait_for(scene, "screens", "shown", "HomeCard", 3000);
    scene.wait(300);
    let mut pick = |want: &str| {
        for _ in 0..14 {
            let tree = scene.tree();
            let focus = scene.focus(&tree).map(|i| tree.nodes[i as usize].id.clone()).unwrap_or_default();
            if tree.nodes.iter().any(|n| n.parent == focus && n.visible && n.text.as_deref() == Some(want)) {
                break;
            }
            scene.pad(UiButton::Down);
            scene.wait(60);
        }
        scene.pad(UiButton::Accept);
        scene.wait(300);
    };
    pick("Tools");
    pick(want_tool);
    wait_for(scene, "screens", "shown", "Tool", 4000);
    wait_for(scene, "jobsModel", "title", want_tool, 4000);
    scene.wait(200);
}

fn setup_world(tag: &str) -> (World, String, String, String, Arc<Platform>) {
    let world = World::new(tag);
    let snes = world.dir("Roms/snes");
    let nes = world.dir("Roms/nes");
    std::fs::create_dir_all(&snes).unwrap();
    std::fs::create_dir_all(&nes).unwrap();

    let ct_path = snes.join("Chrono Trigger (USA).sfc");
    let nes_path = nes.join("Cobalt Harbor.nes");
    std::fs::write(&ct_path, emusen_client_fixture::play::image("snes").unwrap().1).unwrap();
    std::fs::write(&nes_path, emusen_client_fixture::play::image("nes").unwrap().1).unwrap();

    let platform = Platform::load(library()).unwrap();
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = json!(world.dir("Roms").to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();

    // Seed local .cht database file inside sandboxed CheatsDatabase folder.
    let db_dir = database_folder().join(SNES);
    std::fs::create_dir_all(&db_dir).unwrap();
    let cht_text = concat!(
        "cheats = 6\n",
        "cheat0_desc = \"Infinite lives\"\ncheat0_code = \"7E00F204\"\ncheat0_enable = false\n",
        "cheat1_desc = \"999 coins\"\ncheat1_code = \"7E2C50E7\"\ncheat1_enable = false\n",
        "cheat2_desc = \"Max level\"\ncheat2_code = \"7E281063\"\ncheat2_enable = false\n",
        "cheat3_desc = \"Start with all keys\"\ncheat3_code = \"DD62-6DAD\"\ncheat3_enable = false\n",
        "cheat4_desc = \"Walk through walls\"\ncheat4_code = \"7E011001\"\ncheat4_enable = false\n",
        "cheat5_desc = \"Infinite MP\"\ncheat5_code = \"7E284063\"\ncheat5_enable = false\n",
    );
    std::fs::write(db_dir.join("Chrono Trigger (USA).cht"), cht_text).unwrap();

    let ct_str = ct_path.to_str().unwrap().to_owned();
    let nes_str = nes_path.to_str().unwrap().to_owned();
    let rt = Runtime::new(&Api::load(library()).unwrap(), &Config::default()).unwrap();
    let snes_console = rt.systems().unwrap().into_iter().find(|s| s.id == "snes").expect("snes system").console;
    drop(rt);
    let list = platform.cheat_game_list(Some(&snes_console), &ct_str).unwrap().expect("cheat list key");
    let cheats_json = json!({
        "Cheats": [
            { "Kind": "RamPoke", "Description": "Infinite lives", "Enabled": true, "Writes": [{ "Space": "CpuBus", "Address": "7E00F2", "Value": "04", "Width": 1, "Type": "Set" }] },
            { "Kind": "RamPoke", "Description": "999 coins", "Enabled": true, "Writes": [{ "Space": "CpuBus", "Address": "7E2C50", "Value": "E7", "Width": 1, "Type": "Set" }] },
            { "Kind": "RomPatch", "Description": "Start with all keys", "Enabled": false, "Writes": [{ "Space": "CpuBus", "Address": "00B41C", "Value": "FF", "Width": 1, "Type": "Set" }] },
            { "Kind": "RamPoke", "Description": "Walk through walls", "Enabled": false, "Writes": [{ "Space": "CpuBus", "Address": "7E0110", "Value": "01", "Width": 1, "Type": "Set" }] },
            { "Kind": "RamPoke", "Description": "Fast dash", "Enabled": false, "Writes": [{ "Space": "CpuBus", "Address": "7E00A4", "Value": "02", "Width": 1, "Type": "Set" }] },
            { "Kind": "RamPoke", "Description": "Double speed", "Enabled": false, "Writes": [{ "Space": "CpuBus", "Address": "7E00A4", "Value": "08", "Width": 1, "Type": "Set" }] }
        ]
    });
    platform.model_save(MODEL_CHEAT_FILE, Some(&list.1), &format!("{}.json", list.0), &cheats_json).unwrap();
    (world, ct_str, nes_str, snes_console, platform)
}

fn hold_cheats_board(scene: &mut Scene<'_>, backend: &str, differences: &mut Vec<String>) {
    let (board_h, all_rows) = rows(TABLE_CHEATS);
    let k = DESIGN_HEIGHT / board_h;
    let tokens = kit_tokens(scene);
    for (width, height) in SIZES {
        scene.resize(width, height);
        scene.wait(200);
        let tree = scene.tree();
        let unit: f64 = read(&tree, "jobs", "unit").parse().expect("jobs unit");
        let cw = width as f64 / unit;
        let at = format!("cheats {width}x{height}");
        let (present, missing): (Vec<_>, Vec<_>) = all_rows.iter().cloned().partition(|r| r.status != "built" || tree.find_named(&r.name).is_some());
        differences.extend(missing.iter().map(|r| format!("{at}: {} is not drawn", r.name)));
        held(&tree, &present, k, unit, cw, &tokens, &at, &|_, _, _| false, differences);
        for (name, want, upper) in [
            ("trailWords.0", "Consoles", false),
            ("trailWords.3", "Cheats", false),
            ("cheatsBattery", "82%", false),
            ("cheatsClock", "9:41 PM", false),
            ("sortCaption", "Active first, then A\u{2013}Z", false),
            ("groupLabel.database", "FROM THE DATABASE \u{00B7} 4", true),
            ("inspectorName", "999 coins", false),
        ] {
            let have = read(&tree, name, "text");
            let drawn = if upper { have.to_uppercase() } else { have };
            if drawn != want {
                differences.push(format!("{at}: {name} says {drawn:?} where the board says {want:?}"));
            }
        }
        let line = words_under(&tree, "cheatsHelp");
        let want_help = vec!["Turn off", "Edit or remove\u{2026}", "Browse file", "Add code", "Filter", "Back"];
        if line != want_help {
            differences.push(format!("{at}: cheatsHelp says {line:?} where the board says {want_help:?}"));
        }
        let seen_glyphs = glyphs_in(&tree, "cheatsHelp");
        let want_glyphs = vec!["A", "X", "Y", "VIEW", "LB", "B"];
        if seen_glyphs != want_glyphs {
            differences.push(format!("{at}: cheatsHelp glyphs read {seen_glyphs:?} where the board draws {want_glyphs:?}"));
        }
    }
    scene.resize(1920, 1200);
    scene.wait(200);
    let pic = if backend == "gpu" { "cheats.png".to_owned() } else { format!("cheats-1920x1200-{backend}.png") };
    picture(scene, &pic);
    scene.resize(1280, 800);
    scene.wait(150);
}

pub fn cheats_oracle(backend: &str) {
    std::env::set_var("TZ", "UTC");
    let (world, ct_path, _nes_path, snes_console, platform) = setup_world(&format!("qt-app-eclipse-cheats-{backend}"));
    let mut harness = Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("program", "theme", &std::fs::canonicalize(ECLIPSE).unwrap().display().to_string()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);

    // 1. Entry Point A: Home -> Tools -> Cheats (#21), board verification, and functional checks.
    open_eclipse_tool(&mut scene, "Cheats");
    wait_for(&mut scene, "jobsModel", "eclipseScreen", "8", 3000);
    until(&mut scene, "cheats view loaded", 4000, |s| view_json(s)["ready"] == true && view_json(s)["hasGame"] == true);
    freeze_header(&mut scene);

    let v0 = view_json(&mut scene);
    assert_eq!(v0["positionCounter"], "1 of 6");
    assert_eq!(v0["activeSummary"], "2 of 6 active");
    assert_eq!(v0["appliedSub"], "Applied on next launch");
    assert!(!v0["inspector"]["facts"].as_array().unwrap().iter().any(|f| f["label"] == "Not with"), "unconflicted cheat omits Not with");
    assert!(v0["dbCard"]["copy"].as_str().unwrap().contains("4 of 6 codes"));

    let mut differences = Vec::new();
    hold_cheats_board(&mut scene, backend, &mut differences);
    assert!(differences.is_empty(), "board diffs:\n{}", differences.join("\n"));

    // 2. Filtering via LB / RB.
    scene.pad(UiButton::PageDown);
    until(&mut scene, "From database filter active", 2000, |s| view_json(s)["positionCounter"] == "1 of 4");
    scene.pad(UiButton::PageDown);
    until(&mut scene, "Yours filter active", 2000, |s| view_json(s)["positionCounter"] == "1 of 2");
    // Conditional Not with appears on Double speed (shares $7E00A4 with Fast dash).
    let yours_facts = view_json(&mut scene)["inspector"]["facts"].as_array().unwrap().clone();
    let conflict = yours_facts.iter().find(|f| f["label"] == "Not with").expect("shared address conflict row");
    assert_eq!(conflict["value"], "Fast dash");
    scene.pad(UiButton::PageDown);
    until(&mut scene, "Active filter active", 2000, |s| view_json(s)["positionCounter"] == "1 of 2");
    scene.pad(UiButton::PageDown);
    until(&mut scene, "All filter active again", 2000, |s| view_json(s)["positionCounter"] == "1 of 6");

    // 3. Toggle row (A) -> saved to CheatFile + updated count.
    scene.pad(UiButton::Down);
    scene.wait(100);
    scene.pad(UiButton::Down);
    until(&mut scene, "focus moved to Start with all keys", 1500, |s| view_json(s)["inspector"]["name"] == "Start with all keys");
    scene.pad(UiButton::Accept);
    until(&mut scene, "cheat toggled on", 2000, |s| view_json(s)["activeSummary"] == "3 of 6 active");
    let list_key = platform.cheat_game_list(Some(&snes_console), &ct_path).unwrap().unwrap();
    let disk = platform.model_load(MODEL_CHEAT_FILE, Some(&list_key.1), &format!("{}.json", list_key.0)).unwrap().unwrap();
    assert_eq!(disk["Cheats"][2]["Enabled"], true, "toggle saved cleanly to CheatFile");

    // 4. Master toggle (MENU).
    scene.pad(UiButton::Menu);
    until(&mut scene, "master switch off", 2000, |s| view_json(s)["masterOn"] == false && view_json(s)["appliedSub"] == "Master switch is off");
    scene.pad(UiButton::Menu);
    until(&mut scene, "master switch back on", 2000, |s| view_json(s)["masterOn"] == true);

    // 5. Add cheat modal (VIEW) with invalid rejection then valid save (`Moon jump`, `7E00F6:02`).
    scene.pad(UiButton::Options);
    until(&mut scene, "editor opened", 2000, |s| view_json(s)["editorOpen"] == true);
    let set_field = |s: &mut Scene<'_>, idx: i32, text: &str| {
        let obj = s.object("jobsModel") as *mut ToolModel;
        assert!(!obj.is_null());
        unsafe { Pin::new_unchecked(&mut *obj) }.eclipse_set_field(idx, &cxx_qt_lib::QString::from(text));
        s.wait(100);
    };
    set_field(&mut scene, 0, "Moon jump");
    set_field(&mut scene, 1, "BADCODE");
    click_named(&mut scene, "cheatEditorSave");
    until(&mut scene, "invalid code error shown", 2000, |s| {
        !view_json(s)["editorError"].as_str().unwrap_or("").is_empty() && view_json(s)["editorOpen"] == true
    });
    set_field(&mut scene, 1, "7E00F6:02");
    click_named(&mut scene, "cheatEditorSave");
    until(&mut scene, "added cheat appears in list", 2000, |s| view_json(s)["activeSummary"] == "4 of 7 active" && view_json(s)["editorOpen"] == false);

    // 6. Browse .cht file (Y) -> list all 6 entries -> close overlay without leaving #21.
    scene.pad(UiButton::Search);
    until(&mut scene, "browse overlay opened", 2000, |s| view_json(s)["browseOpen"] == true);
    assert_eq!(view_json(&mut scene)["browseRows"].as_array().unwrap().len(), 6);
    click_named(&mut scene, "browseCloseButton");
    until(&mut scene, "browse overlay closed cleanly", 2000, |s| view_json(s)["browseOpen"] == false && s.property("screens", "shown") == "Tool");

    // 7. Edit & Remove via X -> MenuCard -> #14 Question confirm.
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "jobsModel", "cardOpen", "true", 2000);
    scene.wait(150);
    scene.pad(UiButton::Accept);
    until(&mut scene, "edit modal opened", 2000, |s| view_json(s)["editorOpen"] == true && view_json(s)["editorTitle"] == "Edit cheat code");
    set_field(&mut scene, 0, "High jump");
    click_named(&mut scene, "cheatEditorSave");
    until(&mut scene, "edited cheat saved", 2000, |s| view_json(s)["editorOpen"] == false && view_json(s)["inspector"]["name"] == "High jump");
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "jobsModel", "cardOpen", "true", 2000);
    scene.wait(150);
    scene.pad(UiButton::Down);
    scene.wait(100);
    scene.pad(UiButton::Accept);
    confirm_question(&mut scene);
    until(&mut scene, "cheat removed", 2000, |s| view_json(s)["activeSummary"] == "3 of 6 active");

    // Leave Tools -> Cheats cleanly.
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 3000);

    // 8. Entry Point B: Game Page -> Cheats tab -> X All cheats -> #21 -> B returns to Page tab 5.
    let screens_obj = |s: &mut Scene<'_>| unsafe { Pin::new_unchecked(&mut *(s.object("screens") as *mut Screens)) };
    screens_obj(&mut scene).open_game(&cxx_qt_lib::QString::from(ct_path.as_str()));
    wait_for(&mut scene, "screens", "shown", "Page", 3000);
    until(&mut scene, "Cheats tab button shown", 3000, |s| s.tree().nodes.iter().any(|n| n.name == "tab.5" && n.visible));
    click_named(&mut scene, "tab.5");
    until(&mut scene, "Game Page Cheats tab shown", 3000, |s| s.tree().nodes.iter().any(|n| n.name == "pageRow.0" && n.visible));
    scene.wait(120);
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "screens", "shown", "Tool", 3000);
    wait_for(&mut scene, "jobsModel", "eclipseScreen", "8", 3000);
    until(&mut scene, "Chrono Trigger loaded from GamePage", 3000, |s| view_json(s)["gameTitle"] == "Chrono Trigger (USA)");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Page", 3000);
    assert_eq!(scene.property("screens", "pageAt"), "cheats");

    // 9. Entry Point C: Pause Card -> Cheats + live runtime session reload verification.
    let player_cores = world.dir("Home/Cores");
    let core_path = world.player_core(&player_cores);
    let tst_path = world.dir("Roms").join("SessionTest.tst");
    std::fs::write(&tst_path, emusen_client_fixture::image(&[1, 2, 3, 4])).unwrap();
    let rt_cfg = Config { player_cores: Some(vec![player_cores.to_str().unwrap().into()]), ..Config::default() };
    let api = Api::load(library()).unwrap();
    let rt = Runtime::new(&api, &rt_cfg).unwrap();
    rt.core_approve(&emusen_client_fixture::cores::sha256(&core_path)).unwrap();
    let tst_str = tst_path.to_string_lossy().into_owned();
    let opened = rt.open(&Open { path: Some(tst_str.clone()), ..Open::default() }).unwrap();
    let tst_console = rt.session_info(opened).unwrap().console;
    let tst_key = platform.cheat_game_list(Some(&tst_console), &tst_str).unwrap().unwrap();
    platform.model_save(MODEL_CHEAT_FILE, Some(&tst_key.1), &format!("{}.json", tst_key.0), &json!({
        "Cheats": [
            { "Kind": "RamPoke", "Description": "Hold byte", "Enabled": true, "Writes": [{ "Space": "RAM", "Address": "2", "Value": "5A", "Width": 1, "Type": "Set" }] },
            { "Kind": "RamPoke", "Description": "Extra code", "Enabled": false, "Writes": [{ "Space": "RAM", "Address": "3", "Value": "10", "Width": 1, "Type": "Set" }] }
        ]
    })).unwrap();
    let given = emusen_qt_models::cheats::give(&platform, &rt, opened, &tst_str).unwrap().expect("given");
    assert_eq!(given.list.entries().len(), 2);
    assert_eq!(emusen_qt_models::cheats::master(&rt, opened).unwrap(), true);
    emusen_qt_models::cheats::set_master(&rt, opened, false).unwrap();
    assert_eq!(emusen_qt_models::cheats::master(&rt, opened).unwrap(), false);
    platform.model_save(MODEL_CHEAT_FILE, Some(&tst_key.1), &format!("{}.json", tst_key.0), &json!({
        "Cheats": [
            { "Kind": "RamPoke", "Description": "Hold byte", "Enabled": true, "Writes": [{ "Space": "RAM", "Address": "2", "Value": "5A", "Width": 1, "Type": "Set" }] }
        ]
    })).unwrap();
    let reloaded = emusen_qt_models::cheats::reload(&platform, &rt, opened, &tst_str).unwrap().unwrap();
    assert_eq!(reloaded.list.entries().len(), 1);
    let svc = rt.session_service(opened, 0).unwrap();
    let listed = svc.cheat_op(opened, consts::CHEAT_LIST, &json!({})).unwrap();
    assert_eq!(listed["cheats"].as_array().unwrap().len(), 1);
    rt.close(opened).unwrap();

    // Verify PauseCard screen transition in QML (`Pause` -> `Tool("cheats")` -> `Pause` back).
    assert!(scene.set_property("screens", "pauseGame", &ct_path));
    screens_obj(&mut scene).play();
    wait_for(&mut scene, "screens", "shown", "Game", 2000);
    screens_obj(&mut scene).menu();
    wait_for(&mut scene, "screens", "shown", "Pause", 2000);
    screens_obj(&mut scene).open_cheats();
    wait_for(&mut scene, "screens", "shown", "Tool", 3000);
    wait_for(&mut scene, "jobsModel", "eclipseScreen", "8", 3000);
    until(&mut scene, "Pause opens Chrono Trigger cheats", 3000, |s| view_json(s)["gameTitle"] == "Chrono Trigger (USA)");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Pause", 2000);
}
