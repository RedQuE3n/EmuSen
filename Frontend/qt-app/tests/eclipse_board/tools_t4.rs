//! Shared oracle for Eclipse Scraper (#66), Scrape Status (#67), and Library Manager (#68).
#![allow(dead_code)]

use crate::common::wait_for;
use crate::eclipse_board::{glyphs_in, held, kit_tokens, painted, picture, read, rows, words_under};
use emusen_client_fixture::scraper::RecordedScraper;
use emusen_client_fixture::{library, World};
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::{Harness, Scene};
use serde_json::{json, Value};

const ECLIPSE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../themes/eclipse");
const TABLE_SCRAPER: &str = include_str!("../eclipse/scraper.tsv");
const TABLE_STATUS: &str = include_str!("../eclipse/scrape_status.tsv");
const TABLE_LIBRARY: &str = include_str!("../eclipse/library_manager.tsv");

const SHEET_MISSING: &str = "FILE \"MissingDisc.bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n";
const PNG_1X1: [u8; 67] = [
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

fn midnight() -> i64 {
    emusen_qt_models::words::now().div_euclid(86_400) * 86_400
}

fn until(scene: &mut Scene<'_>, what: &str, ms: i32, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    let mut waited = 0;
    while !done(scene) && waited < ms {
        scene.wait(50);
        waited += 50;
    }
    assert!(done(scene), "{what} within {ms} ms (eclipseScreen={})", scene.property("jobsModel", "eclipseScreen"));
}

fn focused_name(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).unwrap_or_default()
}

fn click_named(scene: &mut Scene<'_>, name: &str) {
    let tree = scene.tree();
    let i = tree.find_named(name).unwrap_or_else(|| panic!("{name} not found in tree"));
    let n = &tree.nodes[i as usize];
    scene.click(n.x + n.w / 2.0, n.y + n.h / 2.0);
}

fn confirm_question(scene: &mut Scene<'_>) {
    wait_for(scene, "jobsModel", "asking", "true", 2000);
    wait_for(scene, "jobsQuestion", "opened", "true", 3000);
    scene.wait(300);
    scene.pad(UiButton::Right);
    scene.wait(150);
    scene.pad(UiButton::Accept);
    wait_for(scene, "jobsModel", "asking", "false", 2000);
}

fn open_eclipse_tool(scene: &mut Scene<'_>, want_tool: &str) {
    scene.pad(UiButton::Menu);
    wait_for(scene, "screens", "shown", "HomeCard", 3000);
    scene.wait(300);
    let mut pick = |want: &str| {
        for _ in 0..12 {
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

fn freeze_header(scene: &mut Scene<'_>) {
    assert!(scene.set_property("clock", "now", &(midnight() + 21 * 3600 + 41 * 60).to_string()));
    assert!(scene.set_property("jobs", "battery", "82"));
    assert!(scene.set_property("jobs", "clockText", "9:41 PM"));
    scene.wait(200);
}

fn hold_board(scene: &mut Scene<'_>, tag: &str, table: &str, words: &[(&str, &str, bool)], glyphs: &[&str], help_name: &str, help_words: &[&str], differences: &mut Vec<String>) {
    let (board_h, all_rows) = rows(table);
    let k = DESIGN_HEIGHT / board_h;
    let tokens = kit_tokens(scene);
    for (width, height) in SIZES {
        scene.resize(width, height);
        scene.wait(200);
        let tree = scene.tree();
        let unit: f64 = read(&tree, "jobs", "unit").parse().expect("jobs unit");
        let cw = width as f64 / unit;
        let at = format!("{tag} {width}x{height}");
        let (present, missing): (Vec<_>, Vec<_>) = all_rows.iter().cloned().partition(|r| r.status != "built" || tree.find_named(&r.name).is_some());
        differences.extend(missing.iter().map(|r| format!("{at}: {} is not drawn", r.name)));
        held(&tree, &present, k, unit, cw, &tokens, &at, &|_, _, _| false, differences);
        for (name, want, upper) in words {
            let have = read(&tree, name, "text");
            let drawn = if *upper { have.to_uppercase() } else { have };
            if drawn != *want {
                differences.push(format!("{at}: {name} says {drawn:?} where the board says {want:?}"));
            }
        }
        let line = words_under(&tree, help_name);
        if line != help_words {
            differences.push(format!("{at}: {help_name} says {line:?} where the board says {help_words:?}"));
        }
        let seen_glyphs = glyphs_in(&tree, help_name);
        if seen_glyphs != glyphs {
            differences.push(format!("{at}: {help_name} glyphs read {seen_glyphs:?} where the board draws {glyphs:?}"));
        }
    }
    scene.resize(1280, 800);
    scene.wait(200);
}

fn setup_world(tag: &str, server: &RecordedScraper) -> World {
    let world = World::new(tag);
    let roms = world.dir("Roms");
    let nes = roms.join("nes");
    let gbc = roms.join("gbc");
    let snes = roms.join("snes");
    let psx = roms.join("psx");
    std::fs::create_dir_all(&nes).unwrap();
    std::fs::create_dir_all(&gbc).unwrap();
    std::fs::create_dir_all(&snes).unwrap();
    std::fs::create_dir_all(&psx).unwrap();

    std::fs::write(nes.join("Cobalt Harbor.nes"), emusen_client_fixture::play::image("nes").unwrap().1).unwrap();
    std::fs::write(gbc.join("Lumen Garden.gbc"), emusen_client_fixture::play::image("gbc").unwrap().1).unwrap();
    let mut blob = emusen_client_fixture::play::image("snes").unwrap().1;
    std::fs::write(snes.join("Starlight Drift (USA).sfc"), &blob).unwrap();
    std::fs::write(snes.join("Starlight Drift (Copy).sfc"), &blob).unwrap();
    blob[0] ^= 0x01;
    std::fs::write(snes.join("Starlight Drift (Europe).sfc"), &blob).unwrap();
    for i in 0..8 {
        blob[0] = 0x50 + i;
        std::fs::write(snes.join(format!("Valley {i}.sfc")), &blob).unwrap();
    }
    std::fs::write(psx.join("Tower Set (Disc 2).cue"), SHEET_MISSING).unwrap();

    let media = world.dir("Home/Media");
    for folder in ["covers", "screenshots", "marquees", "titlescreens", "miximages"] {
        std::fs::create_dir_all(media.join("nes").join(folder)).unwrap();
        std::fs::write(media.join("nes").join(folder).join("Cobalt Harbor.png"), PNG_1X1).unwrap();
    }
    std::fs::create_dir_all(media.join("nes/manuals")).unwrap();
    std::fs::write(media.join("nes/manuals/Cobalt Harbor.pdf"), b"%PDF-1.4\n%%EOF\n").unwrap();
    for folder in ["covers", "screenshots", "marquees"] {
        std::fs::create_dir_all(media.join("gbc").join(folder)).unwrap();
        std::fs::write(media.join("gbc").join(folder).join("Lumen Garden.png"), PNG_1X1).unwrap();
    }

    let platform = Platform::load(library()).unwrap();
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = json!(roms.to_str().unwrap());
    settings["OpenEmuFallback"] = json!(false);
    settings["ScrapeVideos"] = json!(false);
    for on in ["ScrapeCovers", "ScrapeScreenshots", "ScrapeMarquees", "ScrapeTitleScreens", "ScrapeMiximages", "ScrapeManuals"] {
        settings[on] = json!(true);
    }
    for off in ["ScrapeBackcovers", "Scrape3dboxes", "ScrapePhysicalmedia", "ScrapeFanart"] {
        settings[off] = json!(false);
    }
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
    RecordedScraper::developer(&world.root.join("Config"));
    let _ = server;
    world
}

fn view_json(scene: &mut Scene<'_>) -> Value {
    serde_json::from_str(&scene.property("jobsModel", "eclipseCores")).unwrap_or(Value::Null)
}

pub fn tools_t4(backend: &str) {
    std::env::set_var("TZ", "UTC");
    let server = RecordedScraper::start();
    let world = setup_world(&format!("qt-app-eclipse-tools-t4-{backend}"), &server);
    let mut harness = Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "coverStores", world.dir("Home/Media").to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("program", "scraperTest", &server.test));
    assert!(scene.set_property("program", "theme", &std::fs::canonicalize(ECLIPSE).unwrap().display().to_string()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);
    let mut differences = Vec::new();

    // 1. Open Scraper (#66) via Home card -> Tools -> Scraper, check board, mark, toggles, filters, cards, then start run -> #67.
    open_eclipse_tool(&mut scene, "Scraper");
    wait_for(&mut scene, "jobsModel", "eclipseScreen", "2", 3000);
    freeze_header(&mut scene);

    // Focus row 1 (Lumen Garden, Game Boy Advance) and mark it so #66 matches board states cleanly.
    scene.pad(UiButton::Down);
    scene.wait(150);
    scene.pad(UiButton::Accept);
    scene.wait(150);
    let v66 = view_json(&mut scene);
    assert!(v66["rows"].as_array().is_some_and(|r| r.len() >= 2), "scraper populated rows");
    assert_eq!(v66["aside"]["title"], "Lumen Garden");
    assert!(v66["markBarText"].as_str().unwrap_or("").starts_with("1 game marked to scrape"));

    hold_board(
        &mut scene,
        "scraper",
        TABLE_SCRAPER,
        &[
            ("trailWords.0", "Tools", false),
            ("trailWords.1", "Scraper", false),
            ("battery", "82%", false),
            ("clock", "9:41 PM", false),
            ("toolTitle", "Scraper", false),
            ("filterWords.all", "All", false),
            ("asideHeading", "Lumen Garden", false),
            ("asideConsole", "Console: Game Boy Color", false),
            ("mediaLabel.0", "Cover", false),
            ("markToggleWords.0", "Videos · Off", false),
            ("markToggleWords.1", "Overwrite · Off", false),
        ],
        &["A", "B", "LB", "X", "VIEW", "MENU"],
        "scraperHelp",
        &["Mark", "Back", "Console", "More", "List", "Apply…"],
        &mut differences,
    );
    picture(&mut scene, &format!("scraper-1920x1200-{backend}.png"));

    // Pointer click on Videos & Overwrite toggle pills flips state live.
    click_named(&mut scene, "markToggle.0");
    until(&mut scene, "Videos toggle flipped On", 2000, |s| read(&s.tree(), "markToggleWords.0", "text") == "Videos · On");
    click_named(&mut scene, "markToggle.1");
    until(&mut scene, "Overwrite toggle flipped On", 2000, |s| read(&s.tree(), "markToggleWords.1", "text") == "Overwrite · On");
    click_named(&mut scene, "markToggle.0");
    click_named(&mut scene, "markToggle.1");
    until(&mut scene, "toggles restored Off", 2000, |s| read(&s.tree(), "markToggleWords.0", "text") == "Videos · Off");

    // LB/RB filter stepping and X More / SELECT List cards.
    scene.pad(UiButton::PageDown);
    scene.wait(150);
    assert_ne!(view_json(&mut scene)["filters"][0]["active"], true, "RB steps past All");
    scene.pad(UiButton::PageUp);
    scene.wait(150);
    assert_eq!(view_json(&mut scene)["filters"][0]["active"], true, "LB steps back to All");

    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "jobsModel", "cardOpen", "true", 2000);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "jobsModel", "cardOpen", "false", 2000);

    // Clear single mark and mark games without art via List card so >4 games enter #67.
    scene.pad(UiButton::Options);
    wait_for(&mut scene, "jobsModel", "cardOpen", "true", 2000);
    scene.wait(300);
    scene.pad(UiButton::Down);
    scene.wait(100);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "jobsModel", "cardOpen", "false", 2000);
    scene.wait(150);

    // 2. Apply -> confirm #14 -> Scrape Status (#67).
    scene.pad(UiButton::Menu);
    confirm_question(&mut scene);
    wait_for(&mut scene, "jobsModel", "eclipseScreen", "3", 3000);

    // Wait until at least 3 results appear while run is still active, verify Pause/Resume while active, then check #67 board.
    until(&mut scene, "3 recent results on #67", 30000, |s| {
        let v = view_json(s);
        v["recentLines"].as_array().is_some_and(|r| r.len() >= 3) && v["actions"][0]["enabled"] == true
    });
    scene.pad(UiButton::Accept);
    until(&mut scene, "Pause -> Resume", 4000, |s| read(&s.tree(), "statusActionWords.0", "text") == "Resume");
    scene.pad(UiButton::Accept);
    until(&mut scene, "Resume -> Pause", 4000, |s| read(&s.tree(), "statusActionWords.0", "text") == "Pause");
    freeze_header(&mut scene);
    hold_board(
        &mut scene,
        "scrape_status",
        TABLE_STATUS,
        &[
            ("trailWords.0", "Tools", false),
            ("trailWords.1", "Scraper", false),
            ("trailWords.2", "Status", false),
            ("battery", "82%", false),
            ("clock", "9:41 PM", false),
            ("statusActionWords.1", "Cancel...", false),
            ("statusActionWords.2", "Hide", false),
            ("statusCounter", "1 of 3", false),
        ],
        &["◀ ▶", "A", "B"],
        "statusHelp",
        &["Move", "Choose", "Back"],
        &mut differences,
    );
    picture(&mut scene, &format!("scrape_status-1920x1200-{backend}.png"));

    // Move to Hide, return to #66, let scrape finish so media.db has SameGame entries too.
    scene.pad(UiButton::Right);
    scene.wait(120);
    scene.pad(UiButton::Right);
    scene.wait(120);
    assert_eq!(read(&scene.tree(), "statusCounter", "text"), "3 of 3");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "jobsModel", "eclipseScreen", "2", 2000);
    wait_for(&mut scene, "jobsModel", "busy", "false", 45000);

    // Return to Home before opening Library Manager (#68).
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 3000);

    // 3. Open Library Manager (#68) via Home card -> Tools -> Library Manager.
    open_eclipse_tool(&mut scene, "Library Manager");
    wait_for(&mut scene, "jobsModel", "eclipseScreen", "4", 3000);
    wait_for(&mut scene, "jobsModel", "busy", "false", 15000);
    freeze_header(&mut scene);

    // Move to row 2 (focused) for #68 board assertion across all 5 gate sizes.
    scene.pad(UiButton::Down);
    scene.wait(100);
    scene.pad(UiButton::Down);
    scene.wait(100);
    hold_board(
        &mut scene,
        "library_manager",
        TABLE_LIBRARY,
        &[
            ("trailWords.0", "Tools", false),
            ("trailWords.1", "Library Manager", false),
            ("battery", "82%", false),
            ("clock", "9:41 PM", false),
            ("toolTitle", "Library Manager", false),
            ("filterWords.all", "All", false),
            ("filterWords.same_dump", "Same Dump", false),
            ("asideContractHeading", "Read-only library guarantee", false),
        ],
        &["A", "X", "LB", "VIEW", "MENU", "B"],
        "libraryHelp",
        &["Hide…", "Mark", "Filter", "Sort & jobs", "Apply marked…", "Back"],
        &mut differences,
    );
    picture(&mut scene, &format!("library_manager-1920x1200-{backend}.png"));

    // Verify findings filter pills, A Hide… -> games.db hide + Show Again…, and X More mark + START Apply… batch.
    scene.pad(UiButton::PageDown);
    scene.wait(120);
    assert_eq!(view_json(&mut scene)["filters"][1]["active"], true, "RB activates Same Dump filter");
    scene.pad(UiButton::PageDown);
    scene.wait(120);
    assert_eq!(view_json(&mut scene)["filters"][2]["active"], true, "RB activates Same Game filter");
    scene.pad(UiButton::PageDown);
    scene.wait(120);
    assert_eq!(view_json(&mut scene)["filters"][3]["active"], true, "RB activates Broken or Suspect filter");
    assert!(view_json(&mut scene)["rows"].as_array().is_some_and(|r| !r.is_empty()), "broken filter shows failed disc cue");

    // A Hide… -> confirms and marks row hidden in games.db.
    scene.pad(UiButton::Accept);
    confirm_question(&mut scene);
    until(&mut scene, "row becomes hidden", 2000, |s| view_json(s)["rows"][0]["cells"][5] == "hidden");

    // A Show again… -> confirms and removes hidden in games.db.
    scene.pad(UiButton::Accept);
    confirm_question(&mut scene);
    until(&mut scene, "row unhidden", 2000, |s| view_json(s)["rows"][0]["cells"][5] == "");

    // Switch back to All, mark via X More, and batch apply via START Apply….
    click_named(&mut scene, "filterPill.all");
    scene.wait(150);
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "jobsModel", "cardOpen", "true", 2000);
    scene.wait(300);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "jobsModel", "cardOpen", "false", 2000);
    until(&mut scene, "mark bar shows 1 to hide", 2000, |s| {
        view_json(s)["markBarText"].as_str().unwrap_or("").starts_with("1 to hide")
    });
    scene.pad(UiButton::Menu);
    confirm_question(&mut scene);
    until(&mut scene, "batch apply hides game and clears marks", 2000, |s| {
        let v = view_json(s);
        v["markBarText"].as_str().unwrap_or("").starts_with("Nothing marked") && v["rows"][0]["cells"][5] == "hidden"
    });

    assert!(painted(&scene.tree(), "backdropGround").is_some());
    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
}
