//! Oracle for Eclipse's Controller Bindings (#12) and Keyboard Keys (#75) across software and card backends.
#![allow(dead_code)]

use crate::common::wait_for;
use crate::eclipse_board::{boxes, glyphs_in, held, kit_tokens, painted, picture, read, rows, same, words_under, Row};
use crate::eclipse_pad::{focused, press};
use crate::eclipse_settings_oracle::{at, settings, settle, shown};
use cxx_qt::casting::Upcast;
use emusen_client_fixture::library;
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::input::PadControl;
use emusen_kit_core::tokens::Colour;
use emusen_platform_client::bindings::consts as binding_files;
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_models::library_model::qobject::QAbstractListModel;
use emusen_qt_models::settings_model::qobject::SettingsModel;
use emusen_qt_models::themes;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::tree::Tree;
use emusen_qt_test::virtual_pad::{Sdl, VirtualPad};
use emusen_qt_test::Scene;
use std::collections::BTreeMap;

const WORLD_JSON: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../qt-app/tests/eclipse/settings_s6_world.json");
const BUTTONS_BOARD: &str = include_str!("../../../qt-app/tests/eclipse/pad_buttons.tsv");
const KEYS_BOARD: &str = include_str!("../../../qt-app/tests/eclipse/pad_keys.tsv");

fn until(scene: &mut Scene<'_>, what: &str, mut pred: impl FnMut(&mut Scene<'_>) -> bool) {
    for _ in 0..100 {
        if pred(scene) {
            return;
        }
        scene.wait(50);
    }
    panic!("timed out waiting for {what}");
}

fn press_crossing(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, control: PadControl, what: &str, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    let code = emusen_qt_endymion::reading::sdl_button(control).expect("pad button");
    sdl.press(pad, code, true);
    until(scene, what, &mut done);
    sdl.press(pad, code, false);
    scene.wait(80);
}

fn centre(scene: &mut Scene<'_>, name: &str) -> (f64, f64) {
    let tree = scene.tree();
    let n = tree.nodes.iter().rev().find(|n| n.name == name && n.visible).unwrap_or_else(|| panic!("{name} is drawn"));
    (n.x + n.w / 2.0, n.y + n.h / 2.0)
}

fn check_card_column(tree: &Tree, side: &str, scrolls: bool, k: f64, unit: f64, canvas_w: f64, tokens: &BTreeMap<String, String>, at: &str, differences: &mut Vec<String>) -> (usize, usize) {
    let mut count = 0;
    let mut focused_count = 0;
    let got = boxes(tree, unit);
    loop {
        let card_name = format!("{side}Card.{count}");
        let Some(card) = got.get(&card_name) else { break };
        if scrolls && !tree.nodes.iter().any(|n| n.name == card_name && n.visible) {
            count += 1;
            continue;
        }
        if !same(card.w, 344.0 * k) || !same(card.h, 56.0 * k) {
            differences.push(format!("{at}: {card_name} size {}x{} != {}x{}", card.w, card.h, 344.0 * k, 56.0 * k));
        }
        let want_x = if side == "left" { 96.0 * k } else { canvas_w - (96.0 + 344.0) * k };
        if !same(card.x, want_x) {
            differences.push(format!("{at}: {card_name} x={} != {want_x}", card.x));
        }
        if !scrolls && (card.y < 270.0 * k - 1e-4 || card.bottom() > 790.0 * k + 1e-4) {
            differences.push(format!("{at}: {card_name} y={}..{} outside {}..{}", card.y, card.bottom(), 270.0 * k, 790.0 * k));
        }
        let ring_name = format!("{side}CardRing.{count}");
        let fill_name = format!("{side}CardFill.{count}");
        let lit = tree.nodes.iter().any(|n| n.name == ring_name && n.visible);
        if lit {
            focused_count += 1;
        }
        let (fill_col, border_rule) = if lit { ("#FF151826", "line 0 -") } else { ("#FF0F1119", "line 1 #615E647B") };
        let (left_col, left_size, left_type, right_col, right_size, right_type) = if side == "left" {
            ("#FFB9BED0", "16", "400 0 none", "#FFF4F1EC", "19", "600 0 none")
        } else {
            ("#FFF4F1EC", "19", "600 0 none", "#FFB9BED0", "16", "400 0 none")
        };
        let mut synthetic = vec![
            Row {
                name: fill_name.clone(),
                part: "Rectangle".into(),
                x: format!("in:{card_name}:0"),
                y: format!("top:{card_name}:0"),
                w: "344".into(),
                h: "56".into(),
                radius: "14".into(),
                colour: fill_col.into(),
                size: "-".into(),
                status: "built".into(),
                kind: border_rule.into(),
            },
            Row {
                name: format!("{side}CardLeft.{count}"),
                part: "CardLine".into(),
                x: format!("in:{card_name}:16"),
                y: format!("mid:{card_name}"),
                w: "-".into(),
                h: "-".into(),
                radius: "-".into(),
                colour: left_col.into(),
                size: left_size.into(),
                status: "built".into(),
                kind: left_type.into(),
            },
            Row {
                name: format!("{side}CardRight.{count}"),
                part: "CardLine".into(),
                x: "-".into(),
                y: format!("mid:{card_name}"),
                w: format!("inright:{card_name}:16"),
                h: "-".into(),
                radius: "-".into(),
                colour: right_col.into(),
                size: right_size.into(),
                status: "built".into(),
                kind: right_type.into(),
            },
        ];
        if lit {
            synthetic.push(Row {
                name: ring_name,
                part: "FocusRing".into(),
                x: format!("in:{card_name}:0"),
                y: format!("top:{card_name}:0"),
                w: "344".into(),
                h: "56".into(),
                radius: "14".into(),
                colour: "-".into(),
                size: "-".into(),
                status: "built".into(),
                kind: "ring 3 3 34 8 20 #66F0B4C4 #FF07080F #FFF6E7D8".into(),
            });
        }
        held(tree, &synthetic, k, unit, canvas_w, tokens, at, &|_, _, _| false, differences);
        count += 1;
    }
    (count, focused_count)
}

#[allow(clippy::too_many_arguments)]
fn check_pad_board(
    scene: &mut Scene<'_>,
    table_tsv: &str,
    keys: bool,
    want_controls: usize,
    want_help: &[&str],
    want_glyphs: &[&str],
    words: &[(&str, &str)],
    differences: &mut Vec<String>,
) {
    let (board_height, table) = rows(table_tsv);
    let k = DESIGN_HEIGHT / board_height;
    let tokens = kit_tokens(scene);
    for (width, height) in SIZES {
        scene.resize(width, height);
        scene.wait(300);
        let tree = scene.tree();
        let unit: f64 = read(&tree, "controllersEclipseCanvas", "unit").parse().expect("canvas unit");
        let canvas_w = f64::from(width) / unit;
        let at_size = format!("{} {width}x{height}", if keys { "#75" } else { "#12" });
        held(&tree, &table, k, unit, canvas_w, &tokens, &at_size, &|_, _, _| false, differences);
        let got = boxes(&tree, unit);
        // Verify responsive horizontal centering and 4-column widths across 16:10 and 16:9 viewports.
        if let Some(f) = got.get("backdropField.0") {
            let want = (canvas_w - 800.0 * k) / 2.0;
            if !same(f.x, want) {
                differences.push(format!("{at_size}: backdropField.0 x={} != {want}", f.x));
            }
        }
        if let Some(d) = got.get("detailCard") {
            let want = (canvas_w - 1040.0 * k) / 2.0;
            if !same(d.x, want) {
                differences.push(format!("{at_size}: detailCard x={} != {want}", d.x));
            }
        }
        if !keys {
            let want_w = (canvas_w - (192.0 + 48.0) * k) / 4.0;
            for i in 0..4 {
                let name = format!("wholeCard.{i}");
                if let Some(c) = got.get(&name) {
                    if !same(c.w, want_w) {
                        differences.push(format!("{at_size}: {name} width={} != {want_w}", c.w));
                    }
                }
            }
        } else if let (Some(p1), Some(cap)) = (got.get("keyConsolePill.1"), got.get("keysCaption")) {
            if !same(cap.x, p1.right() + 28.0 * k) {
                differences.push(format!("{at_size}: keysCaption x={} != {}", cap.x, p1.right() + 28.0 * k));
            }
        }
        let pic = read(&tree, "padPicture", "source");
        if !pic.starts_with("data:image/svg+xml;utf8,") {
            differences.push(format!("{at_size}: padPicture source missing SVG data URL: {pic:?}"));
        }
        let line = words_under(&tree, "help");
        if line != want_help {
            differences.push(format!("{at_size}: help words {line:?} != {want_help:?}"));
        }
        let glyphs = glyphs_in(&tree, "help");
        if glyphs != want_glyphs {
            differences.push(format!("{at_size}: help glyphs {glyphs:?} != {want_glyphs:?}"));
        }
        let (lc, lf) = check_card_column(&tree, "left", false, k, unit, canvas_w, &tokens, &at_size, differences);
        let (rc, rf) = check_card_column(&tree, "right", false, k, unit, canvas_w, &tokens, &at_size, differences);
        if lc + rc != want_controls || lf + rf != 1 {
            differences.push(format!("{at_size}: cards ({lc}+{rc})/{want_controls}, focused ({lf}+{rf})"));
        }
        let unlit_leaders = tree.nodes.iter().filter(|n| n.visible && (n.name.starts_with("leaderLeft.") || n.name.starts_with("leaderRight."))).count();
        let lit_leaders = tree.nodes.iter().filter(|n| n.visible && (n.name.starts_with("leaderFocusedLeft.") || n.name.starts_with("leaderFocusedRight."))).count();
        if unlit_leaders != want_controls - 1 || lit_leaders != 1 {
            differences.push(format!("{at_size}: leaders unlit={unlit_leaders} lit={lit_leaders} for {want_controls}"));
        }
        for n in tree.nodes.iter().filter(|n| n.visible && n.name.starts_with("leader") && n.ty.contains("Shape")) {
            let want = if n.name.starts_with("leaderFocused") { "#FFF6E7D8" } else { "#D95E647B" };
            if painted(&tree, &n.name) != Colour::parse(want) {
                differences.push(format!("{at_size}: {} colour {:?} != {want}", n.name, painted(&tree, &n.name)));
            }
        }
        for (name, want) in words {
            let have = read(&tree, name, "text");
            if have != *want {
                differences.push(format!("{at_size}: {name} is {have:?} != {want:?}"));
            }
        }
    }
    scene.resize(1280, 800);
    scene.wait(300);
}

fn with_pad_world(tag: &str, f: impl FnOnce(&mut Scene<'_>, &Sdl, &VirtualPad, &str)) {
    std::env::set_var("LC_ALL", "en_US.UTF-8");
    let world = emusen_client_fixture::World::new(tag);
    let game_path = world.dir("Roms/snes").join("Hero.sfc");
    std::fs::write(&game_path, vec![0u8; 4096]).unwrap();
    let platform = Platform::load(library()).unwrap();
    let mut stored = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    stored["RomDirectory"] = serde_json::json!(world.root.join("Roms").to_str().unwrap());
    stored["SwapPadButtons"] = serde_json::json!(false);
    stored["AnalogStickAsDpad"] = serde_json::json!(true);
    stored["StickDeadzone"] = serde_json::json!(0.12);
    stored["Rumble"] = serde_json::json!(true);
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &stored).unwrap();
    let pad_defaults = platform.bindings_defaults(binding_files::GAMEPAD).unwrap();
    platform.bindings_save(binding_files::GAMEPAD, &serde_json::json!({ "Consoles": { "SNES": pad_defaults }, "Players": {} })).unwrap();
    let power = world.power_supply().join("BAT0");
    std::fs::create_dir_all(&power).unwrap();
    for (key, value) in [("type", "Battery"), ("capacity", "82"), ("status", "Discharging")] {
        std::fs::write(power.join(key), format!("{value}\n")).unwrap();
    }
    let sdl = Sdl::open();
    let pad = sdl.attach("Simulated pad", 0x045e, 0x028e);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("clock", "now", &emusen_qt_models::words::now().to_string()));
    assert!(scene.set_property("program", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("settingsModel", "world", WORLD_JSON));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "settingsModel", "ready", "true", 5000);
    wait_for(&mut scene, "pads", "connected", "1", 5000);
    scene.run_pad_clock();
    for _ in 0..40 {
        if focused(&mut scene) == "place.0" {
            break;
        }
        scene.wait(50);
    }
    for _ in 0..5 {
        press(&mut scene, &sdl, &pad, PadControl::Right);
    }
    press(&mut scene, &sdl, &pad, PadControl::South);
    wait_for(&mut scene, "screens", "shown", "Settings", 2000);
    assert!(scene.set_property("program", "theme", &themes::built_in_folder("eclipse")));
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);
    for _ in 0..40 {
        if scene.property("theme", "settings") == "true" {
            break;
        }
        scene.wait(50);
    }
    assert_eq!(scene.property("theme", "settings"), "true", "Eclipse draws Settings");
    let game_str = game_path.display().to_string();
    f(&mut scene, &sdl, &pad, &game_str);
    sdl.detach(pad);
}

pub fn oracle(backend: &str) {
    with_pad_world(&format!("qt-app-eclipse-pad-{backend}"), |scene, sdl, pad, game_path| {
        let mut differences = Vec::new();
        // Open Controller bindings (#12) from Controllers row 1.
        assert_eq!(settings(scene).shown_rows()[1].id, "bindings");
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        press_crossing(scene, sdl, pad, PadControl::South, "#12 bindings shown", |s| s.property("settingsModel", "bindingsShown") == "true" && shown(s, "controllersEclipseCanvas"));
        assert_eq!(scene.property("theme", "holdsFocus"), "false", "themeHost yields focus while Eclipse bindings runs");

        let board12: serde_json::Value = serde_json::from_str(&scene.property("settingsModel", "bindingsPad")).expect("#12 model");
        let b12_head = board12["detailHeading"].as_str().unwrap().to_owned();
        let b12_sub = board12["detailSubline"].as_str().unwrap().to_owned();
        let b12_cnt = board12["detailCounter"].as_str().unwrap().to_owned();
        let clock_now = scene.property("clock", "shortTime");
        let words12: Vec<(&str, &str)> = vec![
            ("crumbSettings", "Settings"),
            ("crumbSlash", "/"),
            ("crumbControllers", "Controllers"),
            ("battery", "82%"),
            ("clock", &clock_now),
            ("navTabWords.0", "Players"),
            ("navTabWords.1", "Buttons"),
            ("navTabWords.2", "Keyboard keys"),
            ("navTabWords.3", "Hotkeys"),
            ("navTabWords.4", "Profiles"),
            ("playerWords.0", "Player 1 · Simulated pad"),
            ("playerWords.1", "Player 2 · None"),
            ("forWords", "For"),
            ("consolePillWords", "Super Nintendo"),
            ("profilePre", "in the profile"),
            ("profileName", "Default"),
            ("detailHeading", &b12_head),
            ("detailSubline", &b12_sub),
            ("detailCounter", &b12_cnt),
            ("wholeCardName.0", "Swap A and B"),
            ("wholeCardValue.0", "Off"),
            ("wholeCardName.1", "Left stick as the D-pad"),
            ("wholeCardValue.1", "On"),
            ("wholeCardName.2", "Stick dead zone"),
            ("wholeCardValue.2", "12%"),
            ("wholeCardName.3", "Rumble"),
            ("wholeCardValue.3", "On, where a game asks"),
        ];
        check_pad_board(
            scene,
            BUTTONS_BOARD,
            false,
            12,
            &["Change", "Clear", "Back to the default", "Page", "Back"],
            &["A", "X", "Y", "LB", "B"],
            &words12,
            &mut differences,
        );
        picture(scene, &format!("eclipse-pad-buttons-1920x1200-{backend}.png"));

        // Rebind via pad, cancel 5s timeout, clear via X, and reset via Y on #12.
        press(scene, sdl, pad, PadControl::South);
        until(scene, "#12 pad listen", |s| s.property("settingsModel", "listeningPad") == "true");
        assert_eq!(scene.property("padListenTimeout", "interval"), "5000");
        scene.wait(5150);
        until(scene, "#12 5s timeout ends listen", |s| s.property("settingsModel", "listeningPad") == "false");
        assert_eq!(read(&scene.tree(), "detailHeading", "text"), b12_head, "timeout leaves binding untouched");

        press(scene, sdl, pad, PadControl::South);
        until(scene, "#12 pad listen again", |s| s.property("settingsModel", "listeningPad") == "true");
        let west = emusen_qt_endymion::reading::sdl_button(PadControl::West).unwrap();
        sdl.press(pad, west, true);
        until(scene, "#12 rebound", |s| s.property("settingsModel", "listening") == "false");
        sdl.press(pad, west, false);
        settle(scene);
        press(scene, sdl, pad, PadControl::West);
        until(scene, "#12 cleared", |s| read(&s.tree(), "detailHeading", "text") == "Not bound to a button.");
        press(scene, sdl, pad, PadControl::North);
        until(scene, "#12 reset", |s| read(&s.tree(), "detailHeading", "text") != "Not bound to a button.");

        // Step Up into #12 header pills, cycle Player & Console, then return Down; PlayStation 24-control overflow scrolls cleanly.
        for _ in 0..8 {
            press(scene, sdl, pad, PadControl::Up);
            if read(&scene.tree(), "playerPillRing.0", "visible") == "true" {
                break;
            }
        }
        assert_eq!(read(&scene.tree(), "playerPillRing.0", "visible"), "true");
        press(scene, sdl, pad, PadControl::Right);
        settle(scene);
        assert_eq!(read(&scene.tree(), "playerPillRing.1", "visible"), "true");
        press(scene, sdl, pad, PadControl::South);
        until(scene, "Player 2 active", |s| painted(&s.tree(), "playerPill.1") == Colour::parse("#FFF6E7D8"));
        press(scene, sdl, pad, PadControl::Right);
        press(scene, sdl, pad, PadControl::South);
        until(scene, "PlayStation chosen", |s| read(&s.tree(), "consolePillWords", "text") == "PlayStation");
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        assert_eq!(read(&scene.tree(), "leftCallouts", "clip"), "true");
        assert_eq!(read(&scene.tree(), "rightCallouts", "clip"), "true");
        assert_eq!(read(&scene.tree(), "detailCounter", "text"), "1 of 24");
        for _ in 0..10 {
            press(scene, sdl, pad, PadControl::Down);
        }
        settle(scene);
        // Focused PlayStation control keeps 56 px card inside visible column clip.
        let psx_tree = scene.tree();
        let unit: f64 = read(&psx_tree, "controllersEclipseCanvas", "unit").parse().unwrap();
        let k = DESIGN_HEIGHT / 1200.0;
        let col_left = boxes(&psx_tree, unit)["leftCallouts"];
        assert!(same(col_left.h, 520.0 * k), "col_left={col_left:?}");
        press(scene, sdl, pad, PadControl::LeftTrigger);
        until(scene, "LT cycles back to Super Nintendo", |s| read(&s.tree(), "consolePillWords", "text") == "Super Nintendo");

        // RB steps Page from #12 Buttons to #75 Keyboard keys.
        press(scene, sdl, pad, PadControl::RightShoulder);
        until(scene, "#75 keys shown", |s| s.property("settingsModel", "bindingsKeys") == "true");
        assert!(!shown(scene, "wholeStrip"));
        let board75: serde_json::Value = serde_json::from_str(&scene.property("settingsModel", "bindingsPad")).expect("#75 model");
        let k_head = board75["detailHeading"].as_str().unwrap().to_owned();
        let k_line1 = board75["detailLine1"].as_str().unwrap().to_owned();
        let k_line2 = board75["detailLine2"].as_str().unwrap().to_owned();
        let k_muted = board75["detailMuted"].as_str().unwrap().to_owned();
        let k_cap = board75["caption"].as_str().unwrap().to_owned();
        let clock_keys = scene.property("clock", "shortTime");
        let words75: Vec<(&str, &str)> = vec![
            ("crumbSettings", "Settings"),
            ("crumbSlash", "/"),
            ("crumbControllers", "Controllers"),
            ("battery", "82%"),
            ("clock", &clock_keys),
            ("navTabWords.0", "Players"),
            ("navTabWords.1", "Buttons"),
            ("navTabWords.2", "Keyboard keys"),
            ("navTabWords.3", "Hotkeys"),
            ("navTabWords.4", "Profiles"),
            ("keyboardPillWords", "Keyboard"),
            ("keyConsoleWords.0", "Super Nintendo"),
            ("keyConsoleWords.1", "PlayStation"),
            ("keysCaption", &k_cap),
            ("keysHeading", &k_head),
            ("keysLine1", &k_line1),
            ("keysLine2", &k_line2),
            ("keysMuted", &k_muted),
        ];
        check_pad_board(
            scene,
            KEYS_BOARD,
            true,
            12,
            &["Rebind", "Region", "Reset", "Console", "Clear", "Back"],
            &["A", "◀ ▶", "Y", "LB", "X", "B"],
            &words75,
            &mut differences,
        );
        picture(scene, &format!("eclipse-pad-keys-1920x1200-{backend}.png"));

        // #75 keyboard rebind, clear, reset, and RB console cycle.
        press(scene, sdl, pad, PadControl::South);
        until(scene, "#75 listening key", |s| s.property("settingsModel", "listening") == "true" && s.property("settingsModel", "listeningPad") == "false");
        scene.key(0x4b, 0);
        until(scene, "#75 rebound to K", |s| s.property("settingsModel", "listening") == "false" && read(&s.tree(), "keysLine2", "text") == "Bound to the key K.");
        press(scene, sdl, pad, PadControl::West);
        until(scene, "#75 cleared", |s| read(&s.tree(), "keysLine2", "text") == "Not bound to a key.");
        press(scene, sdl, pad, PadControl::North);
        until(scene, "#75 reset", |s| read(&s.tree(), "keysLine2", "text") != "Not bound to a key.");
        press(scene, sdl, pad, PadControl::RightShoulder);
        until(scene, "#75 RB rotates PlayStation first", |s| read(&s.tree(), "keyConsoleWords.0", "text") == "PlayStation");

        // Clicking navTab.0 returns directly to Settings -> Players and restores focus to chosenTheme.
        let (tx, ty) = centre(scene, "navTab.0");
        scene.click(tx, ty);
        until(scene, "navTab.0 opens Players", |s| s.property("settingsModel", "bindingsShown") == "false" && at(s) == (0, "players".to_owned()));
        assert_eq!(scene.property("theme", "holdsFocus"), "true", "focus restored to chosenTheme");

        // Pause Card bindings door locks nav2 away from bindings and hides Page help entry.
        press(scene, sdl, pad, PadControl::East);
        settle(scene);
        let facts = serde_json::json!({ "game": game_path, "system": "snes" }).to_string();
        assert!(scene.set_property("settingsModel", "facts", &facts));
        let door_row = settings(scene).view().game_rows().iter().position(|r| r.key == "gbindings").expect("gbindings row");
        // Safety: settingsModel lives pinned inside QQmlApplicationEngine for the life of `scene`.
        unsafe {
            let model = &mut *(<SettingsModel as Upcast<QAbstractListModel>>::from_base_ptr(scene.object("settingsModel") as *const QAbstractListModel) as *mut SettingsModel);
            std::pin::Pin::new_unchecked(model).card_act(door_row as i32, 0);
        }
        until(scene, "Pause Card door opens #12", |s| s.property("settingsModel", "bindingsShown") == "true" && s.property("settingsModel", "bindingsFromCard") == "true");
        assert_eq!(words_under(&scene.tree(), "help"), vec!["Change", "Clear", "Back to the default", "Back"]);
        let (locked_x, locked_y) = centre(scene, "navTab.0");
        scene.click(locked_x, locked_y);
        settle(scene);
        assert_eq!(scene.property("settingsModel", "bindingsShown"), "true", "fromCard blocks nav2 switch");
        press_crossing(scene, sdl, pad, PadControl::East, "B closes fromCard bindings", |s| s.property("settingsModel", "bindingsShown") == "false");

        // Built-in reference theme still uses Settings.qml untouched (R-20261114-21).
        assert!(scene.set_property("program", "theme", &themes::built_in_folder("emusen")));
        settle(scene);
        if scene.property("screens", "shown") != "Settings" {
            for _ in 0..5 {
                press(scene, sdl, pad, PadControl::Right);
            }
            press(scene, sdl, pad, PadControl::South);
            wait_for(scene, "screens", "shown", "Settings", 2000);
        }
        until(scene, "emusen SettingsSlot active", |s| s.property("settingsSlot", "active") == "true");
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        press(scene, sdl, pad, PadControl::South);
        until(scene, "emusen reference bindings open", |s| s.property("settingsModel", "bindingsShown") == "true");
        assert!(!shown(scene, "controllersEclipseCanvas"), "reference Settings.qml draws bindings, not ControllersEclipse");
        press(scene, sdl, pad, PadControl::East);
        until(scene, "reference bindings closed", |s| s.property("settingsModel", "bindingsShown") == "false");
        press(scene, sdl, pad, PadControl::East);
        wait_for(scene, "screens", "shown", "Home", 2000);
        assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
        scene.assert_quiet();
    });
}
