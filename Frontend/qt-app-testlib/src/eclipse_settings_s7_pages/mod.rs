//! The oracle of Eclipse's Settings on Shader levers (#59), Shader parameters (#60), Themes (#61) and Scraping account and run (#62) by pad, keyboard and pointer (EmuSen_Qt.md §3.4.11).
#![allow(dead_code)]

use crate::common::wait_for;
use crate::eclipse_board::{held, read, rows};
use crate::eclipse_pad::{focused, press};
use crate::eclipse_settings_oracle::{at, centre, help_line, lines_held, paused_frame, pictures, settings, settle, shown};
use cxx_qt_lib::{QString, QVariant};
use emusen_client_fixture::scraper::RecordedScraper;
use emusen_client_fixture::{library, World};
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::input::{PadControl, UiButton};
use emusen_kit_core::tokens::Colour;
use emusen_platform_client::galaxia::consts::{DIR_MEDIA, DIR_SHADERS, DIR_THEMES, FILE_GAMES, FILE_GRAPHICS, MODEL_APP_SETTINGS, MODEL_GRAPHICS_CONFIG};
use emusen_platform_client::scrape::ScraperConfig;
use emusen_platform_client::{Lent, Platform};
use emusen_qt::MAIN_URL;
use emusen_qt_endymion::reading::sdl_button;
use emusen_qt_models::clients::SQLITE;
use emusen_qt_models::themes;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::virtual_pad::{Sdl, VirtualPad};
use emusen_qt_test::Scene;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

const LEVERS_BOARD: &str = include_str!("../../../qt-app/tests/eclipse/settings_levers.tsv");
const LEVERS_LINES: &str = include_str!("../../../qt-app/tests/eclipse/settings_levers_lines.tsv");
const PARAMS_BOARD: &str = include_str!("../../../qt-app/tests/eclipse/settings_shader_params.tsv");
const PARAMS_LINES: &str = include_str!("../../../qt-app/tests/eclipse/settings_shader_params_lines.tsv");
const THEMES_BOARD: &str = include_str!("../../../qt-app/tests/eclipse/settings_themes.tsv");
const THEMES_LINES: &str = include_str!("../../../qt-app/tests/eclipse/settings_themes_lines.tsv");
const SCRAPE_BOARD: &str = include_str!("../../../qt-app/tests/eclipse/settings_scrape_run.tsv");
const SCRAPE_LINES: &str = include_str!("../../../qt-app/tests/eclipse/settings_scrape_run_lines.tsv");

const KEY_RETURN: i32 = 0x0100_0004;
const KEY_BACKSPACE: i32 = 0x0100_0003;
const KEY_ESCAPE: i32 = 0x0100_0000;

/// Waits until `pred` returns true or 5 seconds pass.
fn until(scene: &mut Scene<'_>, what: &str, mut pred: impl FnMut(&mut Scene<'_>) -> bool) {
    for _ in 0..100 {
        if pred(scene) {
            return;
        }
        scene.wait(50);
    }
    panic!("timed out waiting for {what}");
}

/// Checks a board's table, line metrics, glow, help bar, words and wrapped row line counts across every gate size.
#[allow(clippy::too_many_arguments)]
fn check_board(
    scene: &mut Scene<'_>,
    board_tsv: &str,
    lines_tsv: &str,
    glow: &str,
    want_help: &[&str],
    words: &[(String, String)],
    want_lines: &[(usize, i32)],
    differences: &mut Vec<String>,
) {
    let (board_height, table) = rows(board_tsv);
    let k = DESIGN_HEIGHT / board_height;
    for (width, height) in SIZES {
        crate::eclipse_board::resize_settle(scene, width, height);
        let tree = scene.tree();
        let at_size = format!("{width}x{height}");
        let unit: f64 = read(&tree, "settingsCanvas", "unit").parse().unwrap();
        let screen: u64 = tree.nodes[tree.find_named("settingsScreen").unwrap() as usize].id.parse().unwrap();
        let tokens = crate::eclipse_board::tokens(screen);
        held(&tree, &table, k, unit, f64::from(width) / unit, &tokens, &at_size, &|_, _, _| false, differences);
        lines_held(&tree, lines_tsv, k, &at_size, differences);
        if Colour::parse(&read(&tree, "backdropField.0", "paint")) != Colour::parse(glow) {
            differences.push(format!("{at_size}: glow is {} where board wants {glow}", read(&tree, "backdropField.0", "paint")));
        }
        for (name, want) in words {
            let got = read(&tree, name, "text");
            if got != *want {
                differences.push(format!("{at_size}: {name} is {got:?} where board wants {want:?}"));
            }
        }
        for &(row_i, lines) in want_lines {
            let name = format!("rowAbout.{row_i}");
            let lc: i32 = read(&tree, &name, "lineCount").parse().unwrap_or(-1);
            let cut = read(&tree, &name, "truncated") == "true";
            if lc != lines || cut {
                differences.push(format!("{at_size}: {name} has lineCount={lc} truncated={cut} where board wants {lines} lines uncut"));
            }
        }
        let (help, _) = help_line(scene);
        if help != want_help {
            differences.push(format!("{at_size}: help is {help:?} where board wants {want_help:?}"));
        }
    }
    crate::eclipse_board::resize_settle(scene, 1920, 1200);
}

/// Steps Down/Up until the focused row has `want_id`.
fn focus_id(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, want_id: &str) -> usize {
    let idx = settings(scene)
        .shown_rows()
        .iter()
        .position(|r| r.id == want_id)
        .unwrap_or_else(|| panic!("no row id {want_id:?} in {:?}", settings(scene).shown_rows().iter().map(|r| &r.id).collect::<Vec<_>>()));
    let mut cur = at(scene).0 as usize;
    while cur < idx {
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        cur = at(scene).0 as usize;
    }
    while cur > idx {
        press(scene, sdl, pad, PadControl::Up);
        settle(scene);
        cur = at(scene).0 as usize;
    }
    idx
}

/// Presses A on the pad when the action transfers focus out of the theme engine (e.g. typing overlay or theme switch).
fn press_handoff(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, what: &str, pred: impl FnMut(&mut Scene<'_>) -> bool) {
    let south = sdl_button(PadControl::South).unwrap();
    sdl.press(pad, south, true);
    until(scene, what, pred);
    sdl.press(pad, south, false);
    scene.wait(60);
}

/// Types ASCII text into the active typing overlay and presses X (Done).
fn type_and_submit(scene: &mut Scene<'_>, text: &str) {
    until(scene, "settingsKeyboard focused", |s| s.property("settingsKeyboard", "activeFocus") == "true");
    for ch in text.chars() {
        scene.key(ch.to_ascii_uppercase() as i32, 0);
        scene.wait(20);
    }
    scene.pad(UiButton::Screensaver);
    settle(scene);
}

/// Opens the process scraper with its real media.db offline and queues 12 ROMs, then cancels so all 12 stay queued.
fn seed_queued_scrape(platform: &Arc<Platform>, roms: &[PathBuf]) {
    let root = platform.directory(DIR_MEDIA).unwrap().expect("media dir");
    let games_db = platform.file(FILE_GAMES).ok().map(|p| p.to_string_lossy().into_owned());
    let sqlite = Lent::open(SQLITE).expect("libsqlite3");
    let config = ScraperConfig {
        root: root.to_string_lossy().into_owned(),
        games_db,
        settings: json!({
            "openemu": false,
            "developer_files": true,
            "sources_alone": false,
            "test": { "offline": true },
        }),
        ..ScraperConfig::default()
    };
    platform.scraper_open(&config, Some(&sqlite)).expect("open scraper with store");
    let service = platform.scrape_service().expect("scrape service");
    let kinds = service.get(&json!({ "what": "kinds" })).expect("scrape kinds");
    let first_kind = kinds[0]["type"].as_str().expect("first scrape kind");
    let listed: Vec<_> = roms.iter().map(|p| json!({ "path": p.to_string_lossy(), "system": "snes" })).collect();
    let req = json!({
        "games": listed,
        "one": false,
        "refresh": false,
        "enabled": true,
        "choices": { "kinds": [first_kind], "region": "auto", "language": "en", "region_fallback": true, "threads": 1 },
    });
    let queued = service.start(&req).expect("start offline scrape to populate queue");
    assert_eq!(queued, 12, "all 12 games enqueued");
    service.cancel().expect("cancel scrape");
    for _ in 0..100 {
        if service.progress(0).ok().and_then(|p| p["state"].as_str().map(str::to_owned)).as_deref() != Some("running") {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    let queue = service.get(&json!({ "what": "queue" })).expect("read queue");
    assert_eq!(queue.as_array().map_or(0, Vec::len), 12, "all 12 games remain queued after cancel");
}

/// The program in a made-up s7 world with 12 SNES ROMs, CRT levers, a slang preset and Eclipse active on Settings.
fn with_s7(harness: &mut emusen_qt_test::Harness, tag: &str, f: impl FnOnce(&mut Scene<'_>, &Sdl, &VirtualPad, &Arc<Platform>, &[PathBuf])) {
    std::env::set_var("LC_ALL", "en_US.UTF-8");
    let world = World::new(tag);
    let snes = world.dir("Roms/snes");
    let mut roms = Vec::new();
    for i in 0..12u8 {
        let name = if i == 0 { "Aurora Drift.sfc".to_owned() } else { format!("Game {i:02}.sfc") };
        let path = snes.join(name);
        let mut bytes = vec![0u8; 4096];
        bytes[0] = i.wrapping_add(1);
        std::fs::write(&path, bytes).unwrap();
        roms.push(path);
    }
    let platform = Arc::new(Platform::load(library()).unwrap());
    let mut app = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    app["RomDirectory"] = json!(world.root.join("Roms").to_str().unwrap());
    app["EmuSenTheme"] = json!("eclipse");
    app["OpenEmuFallback"] = json!(false);
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &app).unwrap();
    RecordedScraper::developer(&world.root.join("Config"));

    // Write CRT Royale pack preset declaring board #60's four parameters.
    let pack = platform.directory(DIR_SHADERS).unwrap().expect("shaders dir");
    std::fs::create_dir_all(pack.join("crt")).unwrap();
    let slang = concat!(
        "#version 450\n",
        "#pragma parameter scanline_weight \"Scanline weight\" 0.3 0.0 1.0 0.05\n",
        "#pragma parameter mask_strength \"Mask strength\" 0.5 0.0 1.0 0.05\n",
        "#pragma parameter curvature \"Curvature\" 0.0 0.0 1.0 1.0\n",
        "#pragma parameter brightness_boost \"Brightness boost\" 1.0 0.5 2.0 0.05\n",
        "#pragma stage vertex\nvoid main() {}\n",
        "#pragma stage fragment\nvoid main() {}\n",
    );
    std::fs::write(pack.join("crt/royale.slang"), slang).unwrap();
    std::fs::write(pack.join("crt/CRT_Royale.slangp"), "shaders = 1\nshader0 = royale.slang\n").unwrap();

    // Seed SNES CRT levers (#59) and CRT Royale parameter overrides (#60) in graphics.json.
    let graphics_file = platform.file(FILE_GRAPHICS).unwrap();
    let graphics = json!({
        "Consoles": { "SNES": { "ScreenFilter": "CRT" } },
        "ShaderParameters": {
            "SNES": {
                "CRT": { "signal": "1", "standard": "2", "curvature": "0.12", "level": "0", "mask": "0", "maskDepth": "0.6", "glare": "0.2" },
                "slang:crt/CRT_Royale.slangp": { "scanline_weight": "0.45", "curvature": "1", "brightness_boost": "1.2" }
            }
        }
    });
    platform.model_save(MODEL_GRAPHICS_CONFIG, None, graphics_file.file_name().unwrap().to_str().unwrap(), &graphics).unwrap();

    // Machine battery at the board's 82%, discharging.
    let power = world.power_supply().join("BAT0");
    std::fs::create_dir_all(&power).unwrap();
    for (key, value) in [("type", "Battery"), ("capacity", "82"), ("status", "Discharging")] {
        std::fs::write(power.join(key), format!("{value}\n")).unwrap();
    }

    let sdl = Sdl::open();
    let pad = sdl.attach("EmuSen Test Xbox Pad", 0x045e, 0x028e);
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "settingsModel", "ready", "true", 5000);
    wait_for(&mut scene, "pads", "connected", "1", 5000);
    scene.run_pad_clock();

    // Open Settings from Home via the pad, then activate Eclipse over it.
    until(&mut scene, "Home place.0 focused", |s| focused(s) == "place.0");
    for _ in 0..5 {
        press(&mut scene, &sdl, &pad, PadControl::Right);
    }
    press(&mut scene, &sdl, &pad, PadControl::South);
    wait_for(&mut scene, "screens", "shown", "Settings", 2000);
    assert!(scene.set_property("program", "theme", &themes::built_in_folder("eclipse")));
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);
    until(&mut scene, "Eclipse draws Settings", |s| s.property("theme", "settings") == "true");
    until(&mut scene, "row.0 focused", |s| focused(s) == "row.0");
    assert_eq!(at(&mut scene), (0, "controllers".to_owned()));
    assert!(scene.set_property("settingsModel", "facts", r#"{"running":"Aurora Drift"}"#));
    settle(&mut scene);

    f(&mut scene, &sdl, &pad, &platform, &roms);
    sdl.detach(pad);
}

/// Boards #59 (Shader levers), #60 (Shader parameters), #62 (Scraping account & run) and #61 (Themes).
pub fn s7(backend: &str) {
    let mut harness = emusen_qt_test::Harness::new();
    s7_with(&mut harness, backend);
}

/// Runs boards #59-#62 checks on an existing test harness.
pub fn s7_with(harness: &mut emusen_qt_test::Harness, backend: &str) {
    with_s7(harness, "qt-app-eclipse-settings-s7", |scene, sdl, pad, platform, roms| {
        let mut differences = Vec::new();
        let page_help = &["Value", "Change", "Back", "Home"];

        // Board #59: Display -> Shader levers -> Super Nintendo.
        press(scene, sdl, pad, PadControl::RightShoulder);
        settle(scene);
        assert_eq!(at(scene).1, "display");
        focus_id(scene, sdl, pad, "levers");
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        assert_eq!(at(scene).1, "leverConsoles");
        focus_id(scene, sdl, pad, "shaders:SNES");
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        assert_eq!(at(scene), (0, "levers:SNES".to_owned()));

        // Verify all 28 Serenity CRT settings (7 board levers first) + Reset at index 28.
        let lever_rows = settings(scene).shown_rows().to_vec();
        assert_eq!(lever_rows.len(), 29, "28 Serenity CRT settings + Reset to the tier's values");
        let first_eight: Vec<&str> = lever_rows.iter().take(8).map(|r| r.label.as_str()).collect();
        assert_eq!(first_eight, ["Signal", "Television standard", "Curvature", "Picture brightness", "Mask", "Mask depth", "Glare in the glass", "Quality"]);
        assert_eq!(lever_rows[28].label, "Reset to the tier's values");

        paused_frame(scene, true);
        press(scene, sdl, pad, PadControl::Down);
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        assert_eq!(at(scene).0, 2);

        let levers_words = vec![
            ("settingsCrumb".into(), "Display".into()),
            ("settingsHeading".into(), "Shader levers".into()),
            ("settingsPlace".into(), "3 of 29".into()),
            ("asideCaption".into(), "Aurora Drift · live preview".into()),
            ("asideSentence".into(), "Curvature at 12%. Each lever redraws the preview; the tier's own values come back with Reset.".into()),
            ("rowName.0".into(), "Signal".into()),
            ("rowAbout.0".into(), "Signal, for the Super Nintendo.".into()),
            ("rowWords.0".into(), "S-Video".into()),
            ("rowName.1".into(), "Television standard".into()),
            ("rowAbout.1".into(), "Television standard, for the Super Nintendo.".into()),
            ("rowWords.1".into(), "The game's".into()),
            ("rowName.2".into(), "Curvature".into()),
            ("rowAbout.2".into(), "Curvature, for the Super Nintendo.".into()),
            ("rowWords.2".into(), "12%".into()),
            ("rowName.3".into(), "Picture brightness".into()),
            ("rowAbout.3".into(), "Picture brightness, for the Super Nintendo.".into()),
            ("rowWords.3".into(), "Bright".into()),
            ("rowName.4".into(), "Mask".into()),
            ("rowAbout.4".into(), "Mask, for the Super Nintendo.".into()),
            ("rowWords.4".into(), "The screen's own".into()),
            ("rowName.5".into(), "Mask depth".into()),
            ("rowAbout.5".into(), "Mask depth, for the Super Nintendo.".into()),
            ("rowWords.5".into(), "60%".into()),
            ("rowName.6".into(), "Glare in the glass".into()),
            ("rowAbout.6".into(), "Glare in the glass, for the Super Nintendo.".into()),
            ("rowWords.6".into(), "20%".into()),
            ("rowName.7".into(), "Quality".into()),
            ("rowAbout.7".into(), "Quality, for the Super Nintendo.".into()),
            ("rowWords.7".into(), "Balanced".into()),
        ];
        check_board(
            scene,
            LEVERS_BOARD,
            LEVERS_LINES,
            "#6A4CF0",
            page_help,
            &levers_words,
            &[(0, 1), (1, 1), (2, 1), (3, 1), (4, 1), (5, 1), (6, 1), (7, 1)],
            &mut differences,
        );
        pictures(scene, backend, "levers");

        // Stepping Curvature live updates value + aside; Reset to the tier's values restores levers while keeping Quality.
        press(scene, sdl, pad, PadControl::Right);
        settle(scene);
        if read(&scene.tree(), "rowWords.2", "text") != "15%" || !read(&scene.tree(), "asideSentence", "text").starts_with("Curvature at 15%.") {
            differences.push(format!("Curvature Right gave {:?} / {:?}", read(&scene.tree(), "rowWords.2", "text"), read(&scene.tree(), "asideSentence", "text")));
        }
        focus_id(scene, sdl, pad, "lever:quality");
        press(scene, sdl, pad, PadControl::Right);
        settle(scene);
        assert_eq!(read(&scene.tree(), "rowWords.7", "text"), "Accurate");
        focus_id(scene, sdl, pad, "levers:reset");
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        focus_id(scene, sdl, pad, "lever:curvature");
        if read(&scene.tree(), "rowWords.2", "text") == "15%" || read(&scene.tree(), "rowWords.7", "text") != "Accurate" {
            differences.push(format!("Reset levers mismatch: curvature={:?} quality={:?}", read(&scene.tree(), "rowWords.2", "text"), read(&scene.tree(), "rowWords.7", "text")));
        }
        press(scene, sdl, pad, PadControl::East);
        settle(scene);
        press(scene, sdl, pad, PadControl::East);
        settle(scene);
        assert_eq!(at(scene).1, "display");

        // Board #60: Display -> Shaders (#20) -> CRT Royale -> All 4 settings -> Parameters (#60).
        focus_id(scene, sdl, pad, "shaders");
        press_handoff(scene, sdl, pad, "#20 shaders opens", |s| s.property("settingsModel", "shadersShown") == "true" && shown(s, "shadersEclipseCanvas"));
        let (sx, sy) = centre(scene, "sourceTab.1");
        scene.click(sx, sy);
        until(scene, "libretro CRT_Royale shown", |s| read(&s.tree(), "shaderRowName.0", "text") == "CRT Royale");
        let (rx, ry) = centre(scene, "shaderRow.0");
        scene.click(rx, ry);
        until(scene, "All 4 parameters link ready", |s| read(&s.tree(), "allSettingsWords", "text") == "All 4 parameters \u{203A}");
        let (ax, ay) = centre(scene, "allSettingsWords");
        scene.click(ax, ay);
        until(scene, "#60 Parameters page open", |s| s.property("settingsModel", "shadersShown") == "false" && at(s) == (0, "shaderParams:SNES".to_owned()));
        assert_eq!(at(scene), (0, "shaderParams:SNES".to_owned()));

        press(scene, sdl, pad, PadControl::Down);
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        assert_eq!(at(scene).0, 2);

        let params_words = vec![
            ("settingsCrumb".into(), "Display  ›  Shaders  ›  Super Nintendo".into()),
            ("settingsHeading".into(), "Parameters".into()),
            ("settingsPlace".into(), "3 of 6".into()),
            ("asideCaption".into(), "Aurora Drift · live preview".into()),
            ("asideSentence".into(), "The parameters of CRT Royale, for the Super Nintendo.".into()),
            ("rowName.0".into(), "Search parameters".into()),
            ("rowAbout.0".into(), "Words in a parameter's name.".into()),
            ("rowWords.0".into(), "…".into()),
            ("rowName.1".into(), "Reset all".into()),
            ("rowAbout.1".into(), "Every parameter back to the shader's own value.".into()),
            ("rowName.2".into(), "Scanline weight".into()),
            ("rowAbout.2".into(), "Scanline weight, for the Super Nintendo; 0.3 is the shader's own.".into()),
            ("rowWords.2".into(), "0.45".into()),
            ("rowName.3".into(), "Mask strength".into()),
            ("rowAbout.3".into(), "Mask strength, for the Super Nintendo; 0.5 is the shader's own.".into()),
            ("rowWords.3".into(), "0.5".into()),
            ("rowName.4".into(), "Curvature".into()),
            ("rowAbout.4".into(), "Curvature, for the Super Nintendo; Off is the shader's own.".into()),
            ("rowWords.4".into(), "On".into()),
            ("rowName.5".into(), "Brightness boost".into()),
            ("rowAbout.5".into(), "Brightness boost, for the Super Nintendo; 1 is the shader's own.".into()),
            ("rowWords.5".into(), "1.2".into()),
        ];
        check_board(
            scene,
            PARAMS_BOARD,
            PARAMS_LINES,
            "#6A4CF0",
            page_help,
            &params_words,
            &[(0, 1), (1, 1), (2, 1), (3, 1), (4, 1), (5, 1)],
            &mut differences,
        );
        pictures(scene, backend, "shader-params");

        // Interactive #60: bool toggle On <-> Off, live Search parameters typing overlay, and Reset all.
        focus_id(scene, sdl, pad, "lever:curvature");
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        if read(&scene.tree(), "rowWords.4", "text") != "Off" {
            differences.push(format!("ShaderParams bool toggle gave {:?}", read(&scene.tree(), "rowWords.4", "text")));
        }
        focus_id(scene, sdl, pad, "params:search");
        press_handoff(scene, sdl, pad, "Search parameters typing opens", |s| s.property("settingsModel", "typing") == "true" && shown(s, "settingsTyping"));
        type_and_submit(scene, "mask");
        until(scene, "Search filters to Mask strength", |s| settings(s).shown_rows().len() == 3 && read(&s.tree(), "rowName.2", "text") == "Mask strength");
        until(scene, "Settings row regains focus", |s| focused(s).starts_with("row."));
        press_handoff(scene, sdl, pad, "Search reopen", |s| s.property("settingsModel", "typing") == "true");
        until(scene, "settingsKeyboard focused on reopen", |s| s.property("settingsKeyboard", "activeFocus") == "true");
        for _ in 0..4 {
            scene.key(KEY_BACKSPACE, 0);
            scene.wait(15);
        }
        scene.pad(UiButton::Screensaver);
        until(scene, "Search cleared to 6 rows", |s| settings(s).shown_rows().len() == 6);
        until(scene, "Settings row regains focus after clear", |s| focused(s).starts_with("row."));
        focus_id(scene, sdl, pad, "params:reset");
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        if read(&scene.tree(), "rowWords.2", "text") != "0.3" || read(&scene.tree(), "rowWords.5", "text") != "1" {
            differences.push(format!("ShaderParams Reset all gave scanline={:?} brightness={:?}", read(&scene.tree(), "rowWords.2", "text"), read(&scene.tree(), "rowWords.5", "text")));
        }
        paused_frame(scene, false);
        scene.run_pad_clock();
        let back = sdl_button(PadControl::East).unwrap();
        sdl.press(pad, back, true);
        until(scene, "B returns from #60 to #20", |s| s.property("settingsModel", "shadersShown") == "true");
        sdl.press(pad, back, false);
        settle(scene);
        sdl.press(pad, back, true);
        until(scene, "B returns from #20 to display", |s| s.property("settingsModel", "shadersShown") == "false" && at(s).1 == "display");
        sdl.press(pad, back, false);
        settle(scene);
        assert_eq!(at(scene).1, "display");

        // Board #62: Library -> Account and run (seed 12 real queued games in emusen.scrape first).
        seed_queued_scrape(platform, roms);
        for _ in 0..3 {
            press(scene, sdl, pad, PadControl::RightShoulder);
            settle(scene);
        }
        assert_eq!(at(scene).1, "library");
        focus_id(scene, sdl, pad, "scraperun");
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        assert_eq!(at(scene), (0, "scrapeRun".to_owned()));

        let scrape_words = vec![
            ("settingsCrumb".into(), "Library".into()),
            ("settingsHeading".into(), "Scraping".into()),
            ("settingsPlace".into(), "1 of 6".into()),
            ("asideHeading".into(), "Member account".into()),
            ("asideSentence".into(), "Not signed in: runs use EmuSen's developer credentials alone.".into()),
            ("rowName.0".into(), "Member account".into()),
            ("rowAbout.0".into(), "Your ScreenScraper account, for more requests a day and faster ones.".into()),
            ("rowWords.0".into(), "Not signed in".into()),
            ("rowName.1".into(), "Log in".into()),
            ("rowAbout.1".into(), "Your user name and password, checked with one request and kept only if accepted, in a file only you can read.".into()),
            ("rowName.2".into(), "Resume".into()),
            ("rowAbout.2".into(), "Goes on with the games a stopped run left queued; nothing resumes by itself.".into()),
            ("rowWords.2".into(), "12 games queued".into()),
            ("rowName.3".into(), "Cancel scraping".into()),
            ("rowAbout.3".into(), "Stops the run once you say so; what it has not reached stays queued for Resume.".into()),
            ("rowName.4".into(), "Status".into()),
            ("rowAbout.4".into(), "The run's progress, the account's limits and today's requests.".into()),
            ("rowName.5".into(), "Scraper".into()),
            ("rowAbout.5".into(), "The tool that chooses which games a run fetches for and starts it.".into()),
        ];
        check_board(
            scene,
            SCRAPE_BOARD,
            SCRAPE_LINES,
            "#1FA890",
            page_help,
            &scrape_words,
            &[(0, 2), (1, 2), (2, 2), (3, 1), (4, 1), (5, 1)],
            &mut differences,
        );
        pictures(scene, backend, "scrape-run");

        // Interactive #62: Log in 2-step typing (username -> masked password), Status refresh, Scraper door.
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        press_handoff(scene, sdl, pad, "Log in username prompt", |s| s.property("settingsModel", "typing") == "true" && s.property("settingsModel", "typingSecret") == "false");
        type_and_submit(scene, "member");
        until(scene, "Log in masked password prompt", |s| s.property("settingsModel", "typing") == "true" && s.property("settingsModel", "typingSecret") == "true");
        scene.pad(UiButton::Back);
        until(scene, "Log in prompt closed", |s| s.property("settingsModel", "typing") == "false");
        until(scene, "Settings row regains focus after Log in", |s| focused(s).starts_with("row."));
        focus_id(scene, sdl, pad, "scrape:status");
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        if read(&scene.tree(), "asideSentence", "text") != "Idle · 12 games queued. Today: 0 requests." {
            differences.push(format!("Scrape Status aside is {:?}", read(&scene.tree(), "asideSentence", "text")));
        }
        focus_id(scene, sdl, pad, "scrape:scraper");
        press(scene, sdl, pad, PadControl::South);
        wait_for(scene, "screens", "shown", "Tool", 2000);
        wait_for(scene, "jobsModel", "title", "Scraper", 3000);
        scene.wait(200);
        scene.pad(UiButton::Back);
        wait_for(scene, "screens", "shown", "Settings", 2000);
        until(scene, "Settings row regains focus from Tool", |s| focused(s).starts_with("row."));
        settle(scene);
        assert_eq!(at(scene).1, "scrapeRun");
        press(scene, sdl, pad, PadControl::East);
        settle(scene);
        assert_eq!(at(scene).1, "library");

        // Board #61: Library -> Themes (real installed themes only: EmuSen and Eclipse).
        focus_id(scene, sdl, pad, "themes");
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        assert_eq!(at(scene), (0, "themes".to_owned()));
        let theme_rows = settings(scene).shown_rows().to_vec();
        assert_eq!(theme_rows.iter().map(|r| r.label.as_str()).collect::<Vec<_>>(), ["EmuSen", "Eclipse"]);
        let (emusen_help, _) = help_line(scene);
        if emusen_help != ["Use", "Check", "Kind", "Back", "Home"] {
            differences.push(format!("Themes help on EmuSen is {emusen_help:?}"));
        }
        if !shown(scene, "asideThemeImage") || shown(scene, "asideThemeSwatch") {
            differences.push("EmuSen theme preview PNG is not drawn cleanly".into());
        }
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        assert_eq!(at(scene).0, 1);

        let themes_words = vec![
            ("settingsCrumb".into(), "Library".into()),
            ("inlinePillWords.0".into(), "EmuSen".into()),
            ("inlinePillWords.1".into(), "ES-DE".into()),
            ("settingsHeading".into(), "Themes".into()),
            ("settingsPlace".into(), "2 of 2".into()),
            ("rowName.0".into(), "EmuSen".into()),
            ("rowWords.0".into(), "Checked".into()),
            ("rowName.1".into(), "Eclipse".into()),
            ("rowWords.1".into(), "In use".into()),
            ("asideHeading".into(), "Eclipse".into()),
            ("asideMeta".into(), "By The EmuSen project · EmuSen.Theme 1.24 · dark".into()),
            ("asideSentence".into(), "In use.".into()),
        ];
        check_board(
            scene,
            THEMES_BOARD,
            THEMES_LINES,
            "#1FA890",
            &["Check", "Kind", "Back", "Home"],
            &themes_words,
            &[],
            &mut differences,
        );
        if !shown(scene, "asideThemeSwatch") {
            differences.push("Eclipse theme preview swatch card is not drawn".into());
        }
        pictures(scene, backend, "themes");

        // Interactive #61: RB/LB and pointer clicks switch inline kind pills; choosing EmuSen transitions back to reference Settings.
        press(scene, sdl, pad, PadControl::RightShoulder);
        settle(scene);
        if settings(scene).shown_rows().len() != 0 {
            differences.push(format!("ES-DE tab had {} rows", settings(scene).shown_rows().len()));
        }
        press(scene, sdl, pad, PadControl::LeftShoulder);
        settle(scene);
        assert_eq!(settings(scene).shown_rows().len(), 2);
        let (px, py) = centre(scene, "inlinePill.1");
        scene.click(px, py);
        settle(scene);
        assert_eq!(settings(scene).shown_rows().len(), 0);
        let (px, py) = centre(scene, "inlinePill.0");
        scene.click(px, py);
        settle(scene);
        assert_eq!(settings(scene).shown_rows().len(), 2);

        assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));

        // Choosing EmuSen transitions cleanly back to the reference theme's Settings on row 2 (Themes).
        focus_id(scene, sdl, pad, "theme:emusen");
        press_handoff(scene, sdl, pad, "reference Settings active after choosing EmuSen", |s| s.property("settingsSlot", "active") == "true" && s.property("program", "theme").is_empty());
        assert_eq!(scene.property("screens", "shown"), "Settings");
        assert_eq!(scene.property("settingsModel", "focused"), "2", "focus lands on Themes (row 2) after switching back from Eclipse Page::Themes");
        scene.assert_quiet();
    });
}

/// Opens the offline scraper so login attempts fail offline without touching the network.
fn open_offline_scraper(platform: &Arc<Platform>) {
    let root = platform.directory(DIR_MEDIA).unwrap().expect("media dir");
    let games_db = platform.file(FILE_GAMES).ok().map(|p| p.to_string_lossy().into_owned());
    let sqlite = Lent::open(SQLITE).expect("libsqlite3");
    let config = ScraperConfig {
        root: root.to_string_lossy().into_owned(),
        games_db,
        settings: json!({ "openemu": false, "developer_files": true, "sources_alone": false, "test": { "offline": true } }),
        ..ScraperConfig::default()
    };
    platform.scraper_open(&config, Some(&sqlite)).expect("open offline scraper");
}

/// Opens Settings from Eclipse Home via Start -> Home Card -> Settings.
fn open_eclipse_settings(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad) {
    press(scene, sdl, pad, PadControl::Start);
    wait_for(scene, "screens", "shown", "HomeCard", 2000);
    scene.wait(300);
    for _ in 0..12 {
        let tree = scene.tree();
        let is_settings = scene.focus(&tree).is_some_and(|i| {
            let row_id = &tree.nodes[i as usize].id;
            tree.nodes.iter().any(|n| n.parent == *row_id && n.name == "label" && n.text.as_deref() == Some("Settings"))
        });
        if is_settings {
            break;
        }
        scene.pad(UiButton::Down);
        scene.wait(40);
    }
    scene.pad(UiButton::Accept);
    scene.run_pad_clock();
    wait_for(scene, "screens", "shown", "Settings", 2000);
    until(scene, "Eclipse draws Settings", |s| s.property("theme", "settings") == "true");
    until(scene, "Eclipse Settings row focused", |s| focused(s).starts_with("row."));
}

/// Software regression tests for store-installed theme preview PNGs, X Check, password masking, clearing, notice X routing, and Themes focus.
pub fn s7_regressions(harness: &mut emusen_qt_test::Harness) {
    std::env::set_var("LC_ALL", "en_US.UTF-8");
    let world = World::new("qt-app-eclipse-s7-regressions");
    let snes = world.dir("Roms/snes");
    std::fs::write(snes.join("Aurora Drift.sfc"), vec![1u8; 4096]).unwrap();
    let platform = Arc::new(Platform::load(library()).unwrap());
    let mut app = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    app["RomDirectory"] = json!(world.root.join("Roms").to_str().unwrap());
    app["EmuSenTheme"] = json!("eclipse");
    app["OpenEmuFallback"] = json!(false);
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &app).unwrap();
    RecordedScraper::developer(&world.root.join("Config"));
    open_offline_scraper(&platform);

    let themes_dir = platform.directory(DIR_THEMES).unwrap().expect("themes dir");
    let user_theme = themes_dir.join("aurora");
    std::fs::create_dir_all(user_theme.join("screens")).unwrap();
    std::fs::create_dir_all(user_theme.join("assets/previews")).unwrap();
    std::fs::write(
        user_theme.join("theme.json"),
        serde_json::to_string_pretty(&json!({
            "schema": 1,
            "api": "1.21",
            "name": "Aurora",
            "author": "EmuSen Project",
            "licence": "GPL-3.0-or-later",
            "variants": ["dark"],
            "fontSizes": [1],
            "screens": ["home"],
            "previews": ["assets/previews/home.png"]
        }))
        .unwrap(),
    )
    .unwrap();
    std::fs::write(user_theme.join("screens/Home.qml"), "import QtQuick\nimport EmuSen.Theme 1.0\n\nThemeScreen {\n    objectName: \"Home\"\n}\n").unwrap();
    std::fs::write(user_theme.join("assets/previews/home.png"), include_bytes!("../../../themes/emusen/assets/previews/home.png")).unwrap();

    let audit = world.root.join("audit.sh");
    std::fs::write(&audit, "#!/bin/bash\nexit 0\n").unwrap();
    assert!(std::process::Command::new("chmod").arg("+x").arg(&audit).status().unwrap().success());
    let prev_audit = std::env::var_os("EMUSEN_THEME_AUDIT");
    std::env::set_var("EMUSEN_THEME_AUDIT", &audit);

    let sdl = Sdl::open();
    let pad = sdl.attach("EmuSen Test Xbox Pad", 0x045e, 0x028e);
    let start = emusen_qt::start::read(library());
    assert!(start.cover_stores.iter().any(|s| s == themes_dir.to_str().unwrap()), "start::read appends DIR_THEMES to cover_stores");
    let mut initial = emusen_qt::start::initial(&start);
    initial.insert(QString::from("splash"), QVariant::from(&false));
    initial.insert(QString::from("onlyPads"), QVariant::from(&QString::from(&pad.id.to_string())));

    let mut scene = harness.load_initial(MAIN_URL, 1280, 800, &initial, |_| {});
    assert!(scene.set_property("folders", "catching", "true"));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "settingsModel", "ready", "true", 5000);
    wait_for(&mut scene, "pads", "connected", "1", 5000);
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);
    until(&mut scene, "Eclipse theme active", |s| s.property("theme", "home") == "true");
    scene.run_pad_clock();

    // Open Settings -> Library -> Themes and inspect the store-installed Aurora theme preview + X Check.
    until(&mut scene, "Eclipse Home focused", |s| !focused(s).is_empty());
    open_eclipse_settings(&mut scene, &sdl, &pad);
    for _ in 0..4 {
        press(&mut scene, &sdl, &pad, PadControl::RightShoulder);
        settle(&mut scene);
    }
    assert_eq!(at(&mut scene).1, "library");
    focus_id(&mut scene, &sdl, &pad, "themes");
    press(&mut scene, &sdl, &pad, PadControl::South);
    settle(&mut scene);
    focus_id(&mut scene, &sdl, &pad, "theme:aurora");
    until(&mut scene, "Aurora preview PNG rendered", |s| shown(s, "asideThemeImage") && !shown(s, "asideThemeSwatch"));
    assert!(scene.property("theme", "warnings").is_empty(), "{}", scene.property("theme", "warnings"));
    scene.assert_quiet();

    press(&mut scene, &sdl, &pad, PadControl::West);
    until(&mut scene, "Aurora X Check completes", |s| read(&s.tree(), "rowWords.2", "text") == "Checked");
    assert_eq!(read(&scene.tree(), "asideSentence", "text"), "Checked: nothing found.");

    // Move to Library -> Scraping: verify actionable notice does not steal pad X (Done), masking, and clearing.
    press(&mut scene, &sdl, &pad, PadControl::East);
    settle(&mut scene);
    focus_id(&mut scene, &sdl, &pad, "scraperun");
    press(&mut scene, &sdl, &pad, PadControl::South);
    settle(&mut scene);
    focus_id(&mut scene, &sdl, &pad, "scrape:login");
    press_handoff(&mut scene, &sdl, &pad, "Log in username prompt", |s| s.property("settingsModel", "typing") == "true" && s.property("settingsModel", "typingSecret") == "false");
    until(&mut scene, "settingsKeyboard focused", |s| s.property("settingsKeyboard", "activeFocus") == "true");
    for ch in "member".chars() {
        scene.key(ch.to_ascii_uppercase() as i32, 0);
        scene.wait(20);
    }

    let snes_dir = snes.to_string_lossy().into_owned();
    let presenter_ptr = scene.object("noticePresenter");
    let presenter = unsafe { std::pin::Pin::new_unchecked(&mut *(presenter_ptr as *mut emusen_qt_models::notice_presenter::qobject::NoticePresenter)) };
    presenter.present(&QString::from("Save failed"), &QString::from("Folder action attached."), true, &QString::from("Open its folder"), &QString::from(snes_dir.as_str()));
    let notices_ptr = scene.object("notices");
    let notices = unsafe { std::pin::Pin::new_unchecked(&mut *(notices_ptr as *mut emusen_kit_qt::notices::qobject::NoticeModel)) };
    notices.raise(&QString::from("Save failed\nFolder action attached."), true, false);
    wait_for(&mut scene, "notices", "shown", "true", 2000);
    assert_eq!(scene.property("noticePresenter", "hasAction"), "true");

    let west = sdl_button(PadControl::West).unwrap();
    sdl.press(&pad, west, true);
    until(&mut scene, "pad X advances ScrapeUser to ScrapePassword", |s| s.property("settingsModel", "typingSecret") == "true");
    sdl.press(&pad, west, false);
    assert_eq!(scene.property("settingsModel", "typing"), "true");
    let folders = unsafe { &*(scene.object("folders") as *const emusen_qt_models::folders::ffi::Folders) };
    assert!(folders.caught().is_empty(), "actionable notice must not intercept X while typing");
    let notices = unsafe { std::pin::Pin::new_unchecked(&mut *(notices_ptr as *mut emusen_kit_qt::notices::qobject::NoticeModel)) };
    notices.expire();
    wait_for(&mut scene, "notices", "shown", "false", 2000);

    until(&mut scene, "deferred typeWords clears username", |s| s.property("settingsThemeTypist", "text").is_empty());
    for ch in "pw".chars() {
        scene.key(ch.to_ascii_uppercase() as i32, 0);
        scene.wait(20);
    }
    until(&mut scene, "password typed into KeyboardModel", |s| s.property("settingsThemeTypist", "text") == "pw");
    assert_eq!(read(&scene.tree(), "settingsTyped", "text"), "••", "Eclipse typing overlay masks password");
    sdl.press(&pad, west, true);
    until(&mut scene, "pad X closes ScrapePassword", |s| s.property("settingsModel", "typing") == "false");
    sdl.press(&pad, west, false);
    until(&mut scene, "Eclipse KeyboardModel text cleared after Done", |s| s.property("settingsThemeTypist", "text").is_empty());
    until(&mut scene, "Settings row regains focus after Done", |s| focused(s).starts_with("row."));

    // Closing Settings mid-password clears typing, pending_user, and KeyboardModel.text.
    press_handoff(&mut scene, &sdl, &pad, "Log in username prompt reopen", |s| s.property("settingsModel", "typing") == "true" && s.property("settingsModel", "typingSecret") == "false");
    type_and_submit(&mut scene, "member");
    until(&mut scene, "Log in password prompt reopen", |s| s.property("settingsModel", "typingSecret") == "true" && s.property("settingsThemeTypist", "text").is_empty());
    scene.key('P' as i32, 0);
    until(&mut scene, "password char entered", |s| s.property("settingsThemeTypist", "text") == "p");
    let screens = unsafe { std::pin::Pin::new_unchecked(&mut *(scene.object("screens") as *mut emusen_qt_models::screens::qobject::Screens)) };
    screens.home();
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    wait_for(&mut scene, "settingsModel", "typing", "false", 2000);
    until(&mut scene, "Eclipse KeyboardModel text cleared on close", |s| s.property("settingsThemeTypist", "text").is_empty());
    assert!(settings(&mut scene).view().scrape.pending_user.is_none());

    // Reopen Settings -> Library -> Themes, switch to EmuSen, verify Reference Settings focuses row 2 (Themes), then test Reference masking & clearing.
    until(&mut scene, "Eclipse Home focused again", |s| !focused(s).is_empty());
    open_eclipse_settings(&mut scene, &sdl, &pad);
    if at(&mut scene).1 == "scrapeRun" {
        press(&mut scene, &sdl, &pad, PadControl::East);
        settle(&mut scene);
    }
    assert_eq!(at(&mut scene).1, "library");
    focus_id(&mut scene, &sdl, &pad, "themes");
    press(&mut scene, &sdl, &pad, PadControl::South);
    settle(&mut scene);
    focus_id(&mut scene, &sdl, &pad, "theme:emusen");
    press_handoff(&mut scene, &sdl, &pad, "Reference Settings active on row 2", |s| s.property("settingsSlot", "active") == "true" && s.property("program", "theme").is_empty());
    assert_eq!(scene.property("settingsModel", "focused"), "2", "focus lands on Themes (row 2)");

    scene.pad(UiButton::Down);
    scene.pad(UiButton::Down);
    wait_for(&mut scene, "settingsModel", "focused", "4", 1000);
    scene.pad(UiButton::Accept);
    until(&mut scene, "Reference ScrapeRun opens", |s| settings(s).view().page_key() == "scrapeRun");
    scene.pad(UiButton::Down);
    wait_for(&mut scene, "settingsModel", "focused", "1", 1000);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "settingsModel", "typing", "true", 2000);
    assert_eq!(scene.property("settingsModel", "typingSecret"), "false");
    type_and_submit(&mut scene, "member");
    until(&mut scene, "Reference ScrapePassword prompt", |s| s.property("settingsModel", "typingSecret") == "true" && s.property("settingsTypist", "text").is_empty());
    for ch in "pw".chars() {
        scene.key(ch.to_ascii_uppercase() as i32, 0);
        scene.wait(20);
    }
    until(&mut scene, "Reference password typed", |s| s.property("settingsTypist", "text") == "pw");
    assert_eq!(read(&scene.tree(), "settingsTyped", "text"), "••", "Reference Settings masks password with bullets");
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "settingsModel", "typing", "false", 2000);
    until(&mut scene, "Reference KeyboardModel text cleared after Done", |s| s.property("settingsTypist", "text").is_empty());

    scene.assert_quiet();
    sdl.detach(pad);
    match prev_audit {
        Some(v) => std::env::set_var("EMUSEN_THEME_AUDIT", v),
        None => std::env::remove_var("EMUSEN_THEME_AUDIT"),
    }
}
