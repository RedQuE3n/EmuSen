//! Shared oracle for Eclipse's Pause Card across boards #6, #35, #36, #37 and #38 (R-20261113-5).
#![allow(dead_code)]

use crate::common::console::card_words;
use crate::common::{approved_console_core, draw_covers, sandbox, wait_for};
use crate::eclipse_board::{hold_card, kit_tokens, object, painted, picture, read, same};
use cxx_qt_lib::QString;
use emusen_client_fixture::library;
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::input::UiButton;
use emusen_kit_core::tokens::Colour;
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_models::cheats::Switch;
use emusen_qt_models::pause::{DiscFacts, Facts, SlotFact};
use emusen_qt_models::words;
use emusen_qt_test::tree::Tree;
use emusen_qt_test::{Harness, Scene};
use serde_json::json;
use std::collections::BTreeMap;

const ECLIPSE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../themes/eclipse");
const DAY: i64 = 86_400;
const TABLE_PAUSE: &str = include_str!("../eclipse/pause.tsv");
const TABLE_OPTIONS: &str = include_str!("../eclipse/pause_options.tsv");
const TABLE_LEAVE: &str = include_str!("../eclipse/pause_leave.tsv");
const TABLE_ASK: &str = include_str!("../eclipse/pause_ask.tsv");
const TABLE_DISC: &str = include_str!("../eclipse/pause_disc.tsv");

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
    facts.cards.console = "PlayStation".into();
    facts
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

/// Runs all 5 Eclipse Pause Card boards (#6, #35-#38), interactions, and undrawn states on the active backend.
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
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();

    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("library", "cores", cores.to_str().unwrap()));
    assert!(scene.set_property("program", "coverStores", media.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("program", "theme", &std::fs::canonicalize(ECLIPSE).unwrap().display().to_string()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
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
            ("PaneSlotName1", "Slot 1", false),
            ("PaneSlotWhen1", "8:40 PM", false),
            ("PaneSlotName2", "Slot 2", false),
            ("PaneSlotWhen2", "8 November", false),
            ("PaneSlotName3", "Slot 3", false),
            ("PaneSlotWhen3", "Empty", false),
            ("PaneSentence", "A loads Slot 2.", false),
            ("cardBattery", "82%", false),
            ("cardClock", "9:41 PM", false),
            ("cardPosition", "4 of 8", false),
        ],
        &["Select", "Resume", "Screenshot", "Resume"],
        &["A", "B", "X", "MENU"],
        &mut differences,
    );
    let tree = scene.tree();
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
    assert_eq!(read(&scene.tree(), "PaneSentence", "text"), "A saves to Slot 2, replacing it.");
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

    // Verify p2 subpages remain reachable without errors and hiding Cheats when empty works cleanly.
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    scene.wait(120);
    assert_eq!(scene.property("pause", "title"), "Controllers");
    scene.pad(UiButton::Back);
    scene.wait(100);
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    scene.wait(120);
    assert_eq!(scene.property("pause", "title"), "Display");
    scene.pad(UiButton::Back);
    scene.wait(100);
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    scene.wait(120);
    assert_eq!(scene.property("pause", "title"), "Sound");
    scene.pad(UiButton::Back);
    scene.wait(100);
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    scene.wait(120);
    assert_eq!(scene.property("pause", "title"), "Memory Cards");
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
    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
}
