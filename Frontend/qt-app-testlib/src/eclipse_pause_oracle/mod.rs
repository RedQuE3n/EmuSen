//! Shared oracle for Eclipse's Pause Card across boards #6 and #35-#43 (R-20261113-5).
#![allow(dead_code)]

use crate::common::console::{card_words, until};
use crate::common::{approved_console_core, draw_covers, sandbox, wait_for};
use crate::eclipse_board::{hold_card, kit_tokens, object, painted, picture, read, same, words_under};
use cxx_qt_lib::QString;
use emusen_client_fixture::library;
use emusen_crystal_client::shapes::{SettingRow, ShownWith};
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::input::UiButton;
use emusen_kit_core::tokens::Colour;
use emusen_platform_client::galaxia::consts::{FILE_GRAPHICS, MODEL_APP_SETTINGS, MODEL_GRAPHICS_CONFIG};
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_models::cards::{Card as MediaCard, Medium, Slot};
use emusen_qt_models::cheats::Switch;
use emusen_qt_models::pause::{CardFacts, DiscFacts, Facts, SlotFact};
use emusen_qt_models::words;
use emusen_qt_test::tree::Tree;
use emusen_qt_test::virtual_pad::Sdl;
use emusen_qt_test::{Harness, Scene};
use serde_json::json;
use std::collections::BTreeMap;

const ECLIPSE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../themes/eclipse");
const DAY: i64 = 86_400;
const TABLE_PAUSE: &str = include_str!("../../../qt-app/tests/eclipse/pause.tsv");
const TABLE_OPTIONS: &str = include_str!("../../../qt-app/tests/eclipse/pause_options.tsv");
const TABLE_LEAVE: &str = include_str!("../../../qt-app/tests/eclipse/pause_leave.tsv");
const TABLE_ASK: &str = include_str!("../../../qt-app/tests/eclipse/pause_ask.tsv");
const TABLE_DISC: &str = include_str!("../../../qt-app/tests/eclipse/pause_disc.tsv");
const TABLE_CONTROLLERS: &str = include_str!("../../../qt-app/tests/eclipse/pause_controllers.tsv");
const TABLE_DISPLAY: &str = include_str!("../../../qt-app/tests/eclipse/pause_display.tsv");
const TABLE_SOUND: &str = include_str!("../../../qt-app/tests/eclipse/pause_sound.tsv");
const TABLE_CARDS: &str = include_str!("../../../qt-app/tests/eclipse/pause_cards.tsv");
const TABLE_DRAWER: &str = include_str!("../../../qt-app/tests/eclipse/pause_drawer.tsv");

fn midnight() -> i64 {
    words::now().div_euclid(DAY) * DAY
}

fn theme_object(scene: &mut Scene<'_>, name: &str) -> u64 {
    let tree = scene.tree();
    tree.nodes[tree.find_named(name).unwrap_or_else(|| panic!("{name} is placed")) as usize].id.parse().unwrap()
}

fn file_url(path: &std::path::Path) -> String {
    format!("file://{}", path.display())
}

fn base_facts(tower_path: &str, slot1: &str, slot2: &str) -> Facts {
    let held: Vec<u64> = (0..=56).map(|i| 1000 + i * 30).collect();
    let slots = vec![
        SlotFact { slot: 1, exists: true, when: "Sun 8 Nov, 20:40".into(), picture: slot1.to_owned() },
        SlotFact { slot: 2, exists: true, when: "Sun 8 Nov, 21:42".into(), picture: slot2.to_owned() },
    ];
    let mut slot_when = BTreeMap::new();
    slot_when.insert(1, "8:40 PM".into());
    slot_when.insert(2, "8 November".into());
    let mut facts = Facts {
        path: tower_path.to_owned(),
        system: "psx".into(),
        slot: 2,
        speed: 100,
        rewind_seconds: 42.0,
        held,
        now: 2690,
        slots,
        slot_when,
        discs: DiscFacts { count: 3, index: Some(0), lid_open: false, unread: false, can_change: true },
        cheats: vec![Switch { description: "Infinite lives".into(), on: false }],
        ..Facts::default()
    };
    facts.cards = card_facts(&["Card 1", "Card 2", "Card 3"], 2);
    facts
}

fn card_facts(labels: &[&str], held_count: usize) -> CardFacts {
    let drawer_cards: Vec<MediaCard> = labels
        .iter()
        .enumerate()
        .map(|(i, name)| MediaCard {
            path: format!("/cards/{name}.mcd"),
            name: (*name).into(),
            label: (*name).into(),
            games_own: false,
            held_by: (i < held_count).then(|| "Tower Set".to_owned()),
        })
        .collect();
    let choices: Vec<String> = labels.iter().map(|s| (*s).to_owned()).collect();
    CardFacts {
        console: "PlayStation".into(),
        media: vec![Medium {
            medium: "psx.card".into(),
            title: "Memory Card".into(),
            noun: "Card".into(),
            noun_in_text: "card".into(),
            extension: ".mcd".into(),
            kind: "card".into(),
            capacity: Some(15),
            unit: Some("block".into()),
            mode: "named".into(),
            mode_key: "Cards".into(),
            slots: vec![
                Slot { label: "Slot 1".into(), name: "Card 1".into(), path: "/cards/Card 1.mcd".into() },
                Slot { label: "Slot 2".into(), name: "Card 2".into(), path: "/cards/Card 2.mcd".into() },
            ],
            cards: drawer_cards,
            next_name: format!("Card {}", labels.len() + 1),
            opened: Vec::new(),
        }],
        schema: vec![
            SettingRow {
                key: "Cards".into(),
                label: "Mode".into(),
                kind: "choice".into(),
                default: Some("named".into()),
                choices: Some(vec!["shared".into(), "named".into(), "per_game".into()]),
                choice_labels: Some(vec!["Shared".into(), "Named".into(), "Per game".into()]),
                ..SettingRow::default()
            },
            SettingRow {
                key: "Card1".into(),
                label: "Slot 1".into(),
                kind: "choice".into(),
                default: Some("Card 1".into()),
                choices: Some(choices.clone()),
                shown_with: Some(ShownWith { key: "Cards".into(), value: "named".into() }),
                ..SettingRow::default()
            },
            SettingRow {
                key: "Card2".into(),
                label: "Slot 2".into(),
                kind: "choice".into(),
                default: Some("Card 2".into()),
                choices: Some(choices),
                shown_with: Some(ShownWith { key: "Cards".into(), value: "named".into() }),
                ..SettingRow::default()
            },
        ],
        values: BTreeMap::from([("Cards".into(), "named".into()), ("Card1".into(), "Card 1".into()), ("Card2".into(), "Card 2".into())]),
    }
}

fn check_resume_glyphs(tree: &Tree, unit: f64, differences: &mut Vec<String>) {
    let resumes: Vec<_> = tree.nodes.iter().filter(|n| n.visible && n.name == "ownGlyph-Resume").collect();
    if resumes.len() != 2 {
        differences.push(format!("expected 2 visible ownGlyph-Resume glyphs, found {}", resumes.len()));
        return;
    }
    let mut ordered = resumes;
    ordered.sort_by(|a, b| a.x.total_cmp(&b.x));
    let k = DESIGN_HEIGHT / 1200.0;
    let (b_w, b_h) = (ordered[0].w / unit, ordered[0].h / unit);
    let m_h = ordered[1].h / unit;
    if !same(b_w, 38.0 * k) || !same(b_h, 38.0 * k) {
        differences.push(format!("B Resume glyph size ({b_w}, {b_h}) where the board gives ({}, {})", 38.0 * k, 38.0 * k));
    }
    if !same(m_h, 34.0 * k) {
        differences.push(format!("Menu Resume glyph height {m_h} where the board gives {}", 34.0 * k));
    }
}

/// Runs all 10 Eclipse Pause Card boards (#6, #35-#43), interactions, and undrawn states on the active backend.
pub fn pause(backend: &str) {
    std::env::set_var("LC_ALL", "en_US.UTF-8");
    std::env::set_var("TZ", "UTC");
    let roms = [
        ("psx", "Tower Set.cue"),
        ("psx", "Glass Orchard.cue"),
        ("psx", "Ninth Lantern.cue"),
        ("psx", "Saltwater Kings.cue"),
    ];
    let world = sandbox(&format!("qt-app-eclipse-pause-{backend}"), &roms);
    let cores = world.dir("Cores");
    approved_console_core(&cores);
    let tower_path = world.root.join("Roms/psx/Tower Set.cue").to_string_lossy().into_owned();

    let mut harness = Harness::new();
    let media = world.dir("EsdeMedia");
    let shots = world.dir("Shots");
    let slot1_path = shots.join("slot1.png");
    let slot2_path = shots.join("slot2.png");
    let pictures = vec![
        (media.join("psx/covers/Tower Set.png"), (400, 500), 1, String::new()),
        (media.join("psx/covers/Glass Orchard.png"), (400, 500), 2, String::new()),
        (media.join("psx/covers/Ninth Lantern.png"), (400, 500), 3, String::new()),
        (slot1_path.clone(), (320, 240), 4, String::new()),
        (slot2_path.clone(), (320, 240), 5, String::new()),
    ];
    draw_covers(&mut harness, &world.root, &pictures, |_| {});
    let platform = Platform::load(library()).unwrap();
    let mut settings = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    settings["EsdeMediaDirectory"] = json!(media.to_str().unwrap());
    settings["OpenEmuFallback"] = json!(false);
    settings["Volume"] = json!(0.6);
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    let gfx_file = platform.file(FILE_GRAPHICS).unwrap();
    let gfx_name = gfx_file.file_name().unwrap().to_str().unwrap();
    let mut gfx = platform.model_load(MODEL_GRAPHICS_CONFIG, None, gfx_name).unwrap().unwrap_or_else(|| platform.model_new(MODEL_GRAPHICS_CONFIG).unwrap());
    gfx["Consoles"]["PlayStation"]["ScreenFilter"] = json!("CRT");
    gfx["Consoles"]["PlayStation"]["Overscan"] = json!("tv");
    gfx["Consoles"]["PlayStation"]["PictureShape"] = json!("tv");
    gfx["ShaderParameters"]["PlayStation"]["CRT"]["quality"] = json!("1");
    platform.model_save(MODEL_GRAPHICS_CONFIG, None, gfx_name, &gfx).unwrap();

    let sdl = Sdl::open();
    let pad = sdl.attach("Wireless Controller", 0x045e, 0x028e);
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("library", "cores", cores.to_str().unwrap()));
    assert!(scene.set_property("program", "coverStores", media.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("program", "theme", &std::fs::canonicalize(ECLIPSE).unwrap().display().to_string()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "settingsModel", "ready", "true", 5000);
    until(&mut scene, 10, "virtual pad connected", |s| s.property("pads", "connected") == "1");
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);
    assert_eq!(scene.property("theme", "failed"), "false", "{}", scene.property("theme", "warnings"));
    assert!(scene.set_property("clock", "now", &(midnight() + 21 * 3600 + 41 * 60).to_string()));
    let family = emusen_kit_qt::kinds::family_of(emusen_kit_core::glyph::PadFamily::Xbox).repr;
    assert!(scene.set_property("theme", "family", &family.to_string()));
    assert!(scene.set_property("cardCanvas", "battery", "82"));
    scene.wait(300);

    // Launch Tower Set into Game screen (feed == 0 keeps SessionModel quiet), then open Pause Card via Guide.
    let home_id = theme_object(&mut scene, "homeScreen");
    assert!(emusen_theme_qt::host::actions(object(home_id)).expect("Actions").play(&QString::from(&tower_path)));
    wait_for(&mut scene, "screens", "shown", "Game", 5000);
    scene.wait(100);
    scene.pad(UiButton::Guide);
    wait_for(&mut scene, "screens", "shown", "Pause", 5000);
    wait_for(&mut scene, "cardCanvas", "slide", "0", 3000);
    scene.wait(200);

    let facts = base_facts(&tower_path, &file_url(&slot1_path), &file_url(&slot2_path));
    assert!(scene.set_property("pause", "facts", &facts.to_json()));
    let settings_facts = json!({ "game": tower_path, "system": "psx" }).to_string();
    assert!(scene.set_property("settingsModel", "facts", &settings_facts));
    scene.wait(200);
    let tokens = kit_tokens(&mut scene);
    let mut differences = Vec::new();

    // Board #6: Pause Card, in a game, Load State focused (4 of 8) with Save/Load slots detail card.
    for _ in 0..3 {
        scene.pad(UiButton::Down);
    }
    scene.wait(200);
    assert_eq!(
        card_words(&mut scene),
        vec![
            ("Resume".into(), "".into()),
            ("Rewind".into(), "42 s".into()),
            ("Save State".into(), "Slot 2".into()),
            ("Load State".into(), "Slot 2".into()),
            ("Speed".into(), "100%".into()),
            ("Change Disc".into(), "".into()),
            ("Game Options".into(), "".into()),
            ("Leave".into(), "".into()),
        ]
    );
    hold_card(
        &mut scene,
        "cardCanvas",
        &tokens,
        TABLE_PAUSE,
        &[
            ("cardBreadcrumb", "PAUSED · PLAYSTATION", true),
            ("cardTitle", "Tower Set", false),
            ("PaneSlotHeader", "LOAD STATE PREVIEW · SLOTS 1\u{2013}3 OF 8", false),
            ("PaneSlotHint", "◀ ▶ steps slot", false),
            ("PaneSlotSelectedLabel2", "SELECTED", false),
            ("PaneSlotName1", "Slot 1", false),
            ("PaneSlotWhen1", "8:40 PM", false),
            ("PaneSlotName2", "Slot 2", false),
            ("PaneSlotWhen2", "8 November", false),
            ("PaneSlotName3", "Slot 3", false),
            ("PaneSlotWhen3", "Empty", false),
            ("PaneSentence", "Press A to load Slot 2 immediately.", false),
            ("PaneSlotOccupied", "2 of 8 slots occupied", false),
            ("cardBattery", "82%", false),
            ("cardClock", "9:41 PM", false),
            ("cardPosition", "4 of 8", false),
        ],
        &["Select", "Resume", "Screenshot", "Resume"],
        &["A", "B", "X", "MENU"],
        &mut differences,
    );
    let tree = scene.tree();
    assert!(!tree.nodes.iter().any(|n| n.visible && n.name == "PaneSlotRing2"), "active slot wears selected border without second FocusRing");
    let unit: f64 = read(&tree, "cardCanvas", "unit").parse().expect("cardCanvas unit");
    check_resume_glyphs(&tree, unit, &mut differences);
    picture(&mut scene, &format!("pause-1920x1200-{backend}.png"));

    // Interactions on #6: stepping slot on Load State and Save State, plus Speed stepping.
    scene.pad(UiButton::Right);
    scene.wait(60);
    assert_eq!(scene.property("session", "slot"), "3", "Right on Load State chooses Slot 3");
    let mut slot3_facts = facts.clone();
    slot3_facts.slot = 3;
    assert!(scene.set_property("pause", "facts", &slot3_facts.to_json()));
    scene.wait(80);
    assert_eq!(read(&scene.tree(), "PaneSentence", "text"), "Slot 3 is empty.");
    assert!(scene.set_property("pause", "facts", &facts.to_json()));
    scene.pad(UiButton::Up);
    scene.wait(80);
    assert_eq!(read(&scene.tree(), "PaneSentence", "text"), "Press A to save to Slot 2, replacing it.");
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Down);
    scene.wait(80);
    assert_eq!(scene.property("AdaptiveCardPane", "visible"), "false", "Speed shows no pane");
    scene.pad(UiButton::Right);
    scene.wait(60);
    assert_eq!(scene.property("session", "speed"), "300", "Right on Speed selects 300%");

    // Board #38: Pause Card, Change Disc (Tower Set Disc 2 focused, 2 of 7, cover detail card).
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    scene.wait(200);
    assert_eq!(scene.property("pause", "title"), "Change Disc");
    scene.pad(UiButton::Down);
    scene.wait(200);
    assert_eq!(
        card_words(&mut scene),
        vec![
            ("Tower Set (Disc 1)".into(), "Inserted".into()),
            ("Tower Set (Disc 2)".into(), "".into()),
            ("Tower Set (Disc 3)".into(), "".into()),
            ("Glass Orchard".into(), "".into()),
            ("Ninth Lantern".into(), "".into()),
            ("Saltwater Kings".into(), "".into()),
            ("Eject disc".into(), "".into()),
        ]
    );
    hold_card(
        &mut scene,
        "cardCanvas",
        &tokens,
        TABLE_DISC,
        &[
            ("cardBreadcrumb", "PAUSED · PLAYSTATION · TOWER SET", true),
            ("cardTitle", "Change Disc", false),
            ("PaneTitle", "Tower Set, Disc 2 of 3", false),
            ("PaneHint", "A swaps to this disc, as the console sees it.", false),
            ("cardBattery", "82%", false),
            ("cardClock", "9:41 PM", false),
            ("cardPosition", "2 of 7", false),
        ],
        &["Put in", "Back", "Screenshot"],
        &["A", "B", "X"],
        &mut differences,
    );
    assert_eq!(scene.property("PaneCoverFallback", "visible"), "false", "covered disc hides fallback title");
    picture(&mut scene, &format!("pause-disc-1920x1200-{backend}.png"));

    // Undrawn states on #38: Inserted disc sentence, coverless disc fallback, and Eject disc sentence-only pane.
    scene.pad(UiButton::Up);
    scene.wait(80);
    assert_eq!(read(&scene.tree(), "PaneTitle", "text"), "Tower Set, Disc 1 of 3");
    assert_eq!(read(&scene.tree(), "PaneHint", "text"), "In the console now. B goes back.");
    for _ in 0..5 {
        scene.pad(UiButton::Down);
    }
    scene.wait(100);
    assert_eq!(read(&scene.tree(), "PaneTitle", "text"), "Saltwater Kings");
    assert_eq!(scene.property("PaneCoverFallback", "visible"), "true", "coverless disc shows fallback title");
    scene.pad(UiButton::Down);
    scene.wait(100);
    assert_eq!(scene.property("PaneCoverRow", "visible"), "false", "Eject disc hides cover row");
    assert_eq!(read(&scene.tree(), "PaneSentence", "text"), "A takes the disc out and leaves none in, as the console sees it.");
    scene.pad(UiButton::Back);
    scene.wait(150);
    assert_eq!(scene.property("pause", "title"), "Tower Set");

    // Board #35: Pause Card, Game Options (5 rows, 1 of 5, no detail pane).
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    scene.wait(200);
    assert_eq!(
        card_words(&mut scene),
        vec![
            ("Cheats".into(), "".into()),
            ("Controllers".into(), "".into()),
            ("Display".into(), "".into()),
            ("Sound".into(), "".into()),
            ("Memory Cards".into(), "".into()),
        ]
    );
    assert_eq!(scene.property("AdaptiveCardPane", "visible"), "false", "Game Options shows no pane");
    hold_card(
        &mut scene,
        "cardCanvas",
        &tokens,
        TABLE_OPTIONS,
        &[
            ("cardBreadcrumb", "PAUSED · PLAYSTATION · TOWER SET", true),
            ("cardTitle", "Game Options", false),
            ("cardBattery", "82%", false),
            ("cardClock", "9:41 PM", false),
            ("cardPosition", "1 of 5", false),
        ],
        &["Choose", "Back", "Screenshot", "Resume"],
        &["A", "B", "X", "MENU"],
        &mut differences,
    );
    picture(&mut scene, &format!("pause-options-1920x1200-{backend}.png"));

    // Board #39: Pause Card, Game Options, Controllers.
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    until(&mut scene, 5, "Controllers opened", |s| s.property("pause", "title") == "Controllers" && !card_words(s).is_empty());
    scene.wait(120);
    assert_eq!(
        card_words(&mut scene),
        vec![
            ("Player 1".into(), "Wireless Controller".into()),
            ("Player 2".into(), "None".into()),
            ("Profile".into(), "Default".into()),
            ("Bindings".into(), "".into()),
            ("Swap Players".into(), "".into()),
        ]
    );
    hold_card(
        &mut scene,
        "cardCanvas",
        &tokens,
        TABLE_CONTROLLERS,
        &[
            ("cardBreadcrumb", "TOWER SET · GAME OPTIONS", true),
            ("cardTitle", "Controllers", false),
            ("PaneSentence", "Player 1 plays on the Wireless Controller, with the Default profile.", false),
            ("PaneMuted", "The controller player 1 plays on.", false),
            ("cardBattery", "82%", false),
            ("cardClock", "9:41 PM", false),
            ("cardPosition", "1 of 5", false),
        ],
        &["Value", "Choose", "Back", "Screenshot", "Resume"],
        &["◀ ▶", "A", "B", "X", "MENU"],
        &mut differences,
    );
    picture(&mut scene, &format!("pause-controllers-1920x1200-{backend}.png"));
    // Bindings and Swap Players omit Value in the help bar; Swap Players renders in full textStrong ink unfocused.
    for _ in 0..3 {
        scene.pad(UiButton::Down);
    }
    scene.wait(60);
    assert_eq!(words_under(&scene.tree(), "cardHelp"), ["Choose", "Back", "Screenshot", "Resume"]);
    let tree = scene.tree();
    let row4 = tree.nodes.iter().find(|n| n.visible && n.name == "cardRow.4").expect("Swap Players row");
    let swap_ink = tree.nodes.iter().find(|n| n.visible && n.parent == row4.id && n.name == "label").and_then(|n| n.colors.first()).and_then(|c| Colour::parse(c));
    assert_eq!(swap_ink, Colour::parse("#FFF4F1EC"), "Swap Players keeps full cream-white ink unfocused");
    scene.pad(UiButton::Down);
    scene.wait(60);
    assert_eq!(words_under(&scene.tree(), "cardHelp"), ["Choose", "Back", "Screenshot", "Resume"]);
    scene.pad(UiButton::Back);
    scene.wait(100);

    // Board #40: Pause Card, Game Options, Display (R-20261125-2).
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    until(&mut scene, 5, "Display opened", |s| s.property("pause", "title") == "Display" && !card_words(s).is_empty());
    scene.wait(120);
    assert_eq!(
        card_words(&mut scene),
        vec![
            ("CRT profile".into(), "Balanced".into()),
            ("Shader levers".into(), "".into()),
            ("Crop overscan".into(), "TV overscan".into()),
            ("Picture shape".into(), "TV shape".into()),
        ]
    );
    assert!(!scene.tree().nodes.iter().any(|n| n.visible && n.name == "AdaptiveCardPane"), "Display shows no aside box");
    hold_card(
        &mut scene,
        "cardCanvas",
        &tokens,
        TABLE_DISPLAY,
        &[
            ("cardBreadcrumb", "TOWER SET · GAME OPTIONS", true),
            ("cardTitle", "Display", false),
            ("cardSubtitle", "The PlayStation's CRT, drawn on the paused frame behind the card: Balanced.", false),
            ("cardBadgeWords", "Tower Set · Balanced", false),
            ("cardBattery", "82%", false),
            ("cardClock", "9:41 PM", false),
            ("cardPosition", "1 of 4", false),
        ],
        &["Value", "Choose", "Back", "Screenshot", "Resume"],
        &["◀ ▶", "A", "B", "X", "MENU"],
        &mut differences,
    );
    picture(&mut scene, &format!("pause-display-1920x1200-{backend}.png"));
    // Undrawn states on #40: stepping CRT updates subtitle + badge; Shader levers uses {GAME} · DISPLAY and sentence pane.
    scene.pad(UiButton::Down);
    scene.wait(60);
    assert_eq!(words_under(&scene.tree(), "cardHelp"), ["Choose", "Back", "Screenshot", "Resume"]);
    assert_eq!(read(&scene.tree(), "cardBadgeWords", "text"), "Tower Set · Balanced");
    scene.pad(UiButton::Accept);
    until(&mut scene, 5, "Shader levers opened", |s| s.property("pause", "title") == "Shader levers");
    assert_eq!(read(&scene.tree(), "cardBreadcrumb", "text").to_uppercase(), "TOWER SET · DISPLAY");
    assert_eq!(read(&scene.tree(), "AdaptiveCardPane", "visible"), "true");
    assert!(!scene.tree().nodes.iter().any(|n| n.visible && n.name == "cardBadge"));
    scene.pad(UiButton::Back);
    until(&mut scene, 5, "back on Display", |s| s.property("pause", "title") == "Display");
    scene.pad(UiButton::Up);
    scene.pad(UiButton::Left);
    until(&mut scene, 5, "CRT Performance", |s| read(&s.tree(), "cardBadgeWords", "text") == "Tower Set · Performance");
    assert_eq!(read(&scene.tree(), "cardSubtitle", "text"), "The PlayStation's CRT, drawn on the paused frame behind the card: Performance.");
    scene.pad(UiButton::Left);
    until(&mut scene, 5, "CRT Off", |s| read(&s.tree(), "cardBadgeWords", "text") == "Tower Set · Off");
    assert_eq!(read(&scene.tree(), "cardSubtitle", "text"), "The PlayStation's CRT, drawn on the paused frame behind the card: Off.");
    scene.pad(UiButton::Right);
    scene.pad(UiButton::Right);
    until(&mut scene, 5, "CRT Balanced restored", |s| read(&s.tree(), "cardBadgeWords", "text") == "Tower Set · Balanced");
    scene.pad(UiButton::Back);
    scene.wait(100);

    // Board #41: Pause Card, Game Options, Sound (R-20261125-3).
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    until(&mut scene, 5, "Sound opened", |s| s.property("pause", "title") == "Sound" && !card_words(s).is_empty());
    scene.wait(120);
    assert_eq!(card_words(&mut scene), vec![("Volume".into(), "60%".into()), ("Output".into(), "System default".into())]);
    hold_card(
        &mut scene,
        "cardCanvas",
        &tokens,
        TABLE_SOUND,
        &[
            ("cardBreadcrumb", "TOWER SET · GAME OPTIONS", true),
            ("cardTitle", "Sound", false),
            ("PaneSentence", "The game's volume. On the handheld this row defers to the device's quick menu.", false),
            ("PaneMuted", "The ring fills as the volume rises; the next sound plays at the level you chose.", false),
            ("cardBattery", "82%", false),
            ("cardClock", "9:41 PM", false),
            ("cardPosition", "1 of 2", false),
        ],
        &["Value", "Choose", "Back", "Screenshot", "Resume"],
        &["◀ ▶", "A", "B", "X", "MENU"],
        &mut differences,
    );
    picture(&mut scene, &format!("pause-sound-1920x1200-{backend}.png"));
    // Stepping Volume fills bar 3 at 80%; Output hides the 5-bar meter and shows plain sentence + muted aside.
    for _ in 0..4 {
        scene.pad(UiButton::Right);
        scene.wait(40);
    }
    until(&mut scene, 5, "Volume at 80%", |s| card_words(s)[0].1 == "80%");
    assert_eq!(painted(&scene.tree(), "PaneMeterBar.3"), Colour::parse("#FFF6E7D8"), "4th bar fills at 80%");
    scene.pad(UiButton::Down);
    scene.wait(60);
    assert!(!scene.tree().nodes.iter().any(|n| n.visible && n.name == "PaneMeter"), "Output hides the 5-bar meter");
    assert_eq!(words_under(&scene.tree(), "cardHelp"), ["Value", "Choose", "Back", "Screenshot", "Resume"]);
    scene.pad(UiButton::Back);
    scene.wait(100);

    // Board #42: Pause Card, Game Options, Memory Cards.
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    until(&mut scene, 5, "Memory Cards opened", |s| s.property("pause", "title") == "Memory Cards" && !card_words(s).is_empty());
    scene.wait(120);
    assert_eq!(
        card_words(&mut scene),
        vec![
            ("Slot 1".into(), "Card 1".into()),
            ("Slot 2".into(), "Card 2".into()),
            ("Mode".into(), "Named".into()),
            ("Cards".into(), "3 cards".into()),
        ]
    );
    hold_card(
        &mut scene,
        "cardCanvas",
        &tokens,
        TABLE_CARDS,
        &[
            ("cardBreadcrumb", "TOWER SET · GAME OPTIONS", true),
            ("cardTitle", "Memory Cards", false),
            ("PaneSentence", "Card 1 is in Slot 1.", false),
            ("PaneMuted", "Left and Right choose another card; it goes in when the game next starts.", false),
            ("cardBattery", "82%", false),
            ("cardClock", "9:41 PM", false),
            ("cardPosition", "1 of 4", false),
        ],
        &["Value", "Choose", "Back", "Screenshot", "Resume"],
        &["◀ ▶", "A", "B", "X", "MENU"],
        &mut differences,
    );
    picture(&mut scene, &format!("pause-cards-1920x1200-{backend}.png"));

    // Board #43: Pause Card, Memory Cards, the drawer (Card 3 focused).
    for _ in 0..3 {
        scene.pad(UiButton::Down);
    }
    scene.pad(UiButton::Accept);
    until(&mut scene, 5, "Drawer opened", |s| card_words(s).len() == 3 && card_words(s)[0].0 == "Card 1");
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Down);
    scene.wait(120);
    assert_eq!(card_words(&mut scene), vec![("Card 1".into(), "in Tower Set".into()), ("Card 2".into(), "in Tower Set".into()), ("Card 3".into(), "".into())]);
    hold_card(
        &mut scene,
        "cardCanvas",
        &tokens,
        TABLE_DRAWER,
        &[
            ("cardBreadcrumb", "TOWER SET · MEMORY CARDS", true),
            ("cardTitle", "Memory Cards", false),
            ("PaneDrawerCardName.0", "Card 1", false),
            ("PaneDrawerCardName.1", "Card 2", false),
            ("PaneDrawerCardName.2", "Card 3", false),
            ("PaneSentence", "A card in the drawer.", false),
            ("cardBattery", "82%", false),
            ("cardClock", "9:41 PM", false),
            ("cardPosition", "3 of 3", false),
        ],
        &["Choose", "Back", "Screenshot", "Resume"],
        &["A", "B", "X", "MENU"],
        &mut differences,
    );
    picture(&mut scene, &format!("pause-drawer-1920x1200-{backend}.png"));
    // Undrawn >3 card drawer sliding window check.
    let mut five = facts.clone();
    five.cards = card_facts(&["Card 1", "Card 2", "Card 3", "Card 4", "Card 5"], 2);
    assert!(scene.set_property("pause", "facts", &five.to_json()));
    scene.wait(60);
    assert_eq!(read(&scene.tree(), "PaneDrawerCardName.1", "text"), "Card 3");
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Down);
    scene.wait(60);
    assert_eq!(
        [read(&scene.tree(), "PaneDrawerCardName.0", "text"), read(&scene.tree(), "PaneDrawerCardName.1", "text"), read(&scene.tree(), "PaneDrawerCardName.2", "text")],
        ["Card 3", "Card 4", "Card 5"]
    );
    assert!(scene.set_property("pause", "facts", &facts.to_json()));
    scene.pad(UiButton::Back);
    scene.wait(100);
    scene.pad(UiButton::Back);
    scene.wait(100);
    let mut no_cheats = facts.clone();
    no_cheats.cheats.clear();
    assert!(scene.set_property("pause", "facts", &no_cheats.to_json()));
    scene.wait(80);
    assert_eq!(card_words(&mut scene).len(), 4, "Game Options hides Cheats when none exist");
    assert!(scene.set_property("pause", "facts", &facts.to_json()));
    scene.pad(UiButton::Back);
    scene.wait(150);

    // Board #36: Pause Card, Leave (Back to Library focused with divider, 1 of 4, no detail pane).
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    scene.wait(200);
    assert_eq!(
        card_words(&mut scene),
        vec![
            ("Back to Library".into(), "".into()),
            ("Restart Game...".into(), "".into()),
            ("Quit Game...".into(), "".into()),
            ("Quit EmuSen...".into(), "".into()),
        ]
    );
    assert_eq!(scene.property("AdaptiveCardPane", "visible"), "false", "Leave shows no pane");
    hold_card(
        &mut scene,
        "cardCanvas",
        &tokens,
        TABLE_LEAVE,
        &[
            ("cardBreadcrumb", "PAUSED · PLAYSTATION · TOWER SET", true),
            ("cardTitle", "Leave", false),
            ("cardBattery", "82%", false),
            ("cardClock", "9:41 PM", false),
            ("cardPosition", "1 of 4", false),
        ],
        &["Choose", "Back", "Screenshot", "Resume"],
        &["A", "B", "X", "MENU"],
        &mut differences,
    );
    picture(&mut scene, &format!("pause-leave-1920x1200-{backend}.png"));

    // Board #37: Pause Card, Leave's question (Quit Game?, subtitle under title, No focused, 1 of 2).
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    scene.wait(200);
    assert_eq!(card_words(&mut scene), vec![("No".into(), "".into()), ("Yes".into(), "".into())]);
    assert_eq!(scene.property("AdaptiveCardPane", "visible"), "false", "Ask shows no pane");
    assert_eq!(painted(&scene.tree(), "rowBg.0"), Colour::parse("#FFF6E7D8"), "No row focused cream fill");
    hold_card(
        &mut scene,
        "cardCanvas",
        &tokens,
        TABLE_ASK,
        &[
            ("cardBreadcrumb", "PAUSED · PLAYSTATION · TOWER SET", true),
            ("cardTitle", "Quit Game?", false),
            ("cardSubtitle", "The game closes and the library comes back.", false),
            ("cardBattery", "82%", false),
            ("cardClock", "9:41 PM", false),
            ("cardPosition", "1 of 2", false),
        ],
        &["Choose", "Back", "Screenshot", "Resume"],
        &["A", "B", "X", "MENU"],
        &mut differences,
    );
    picture(&mut scene, &format!("pause-ask-1920x1200-{backend}.png"));

    // Choosing No and pressing B return to the Leave row that asked; Restart and Quit EmuSen show their subtitles.
    scene.pad(UiButton::Accept);
    scene.wait(100);
    assert_eq!(scene.property("pause", "title"), "Leave");
    assert_eq!(scene.property("pause", "current"), "2", "No returns to Quit Game row");
    scene.pad(UiButton::Up);
    scene.pad(UiButton::Accept);
    scene.wait(100);
    assert_eq!(scene.property("pause", "title"), "Restart Game?");
    assert_eq!(read(&scene.tree(), "cardSubtitle", "text"), "The game starts again from the beginning. Progress since your last save is lost.");
    scene.pad(UiButton::Back);
    scene.wait(100);
    assert_eq!(scene.property("pause", "current"), "1", "B returns to Restart Game row");
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    scene.wait(100);
    assert_eq!(scene.property("pause", "title"), "Quit EmuSen?");
    assert_eq!(read(&scene.tree(), "cardSubtitle", "text"), "The game closes, then EmuSen.");
    scene.pad(UiButton::Back);
    scene.wait(100);
    assert_eq!(scene.property("pause", "current"), "3", "B returns to Quit EmuSen row");

    scene.assert_quiet();
    sdl.detach(pad);
    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
}
