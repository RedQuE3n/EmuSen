//! Shared oracle for Eclipse Cores (#10) and its three core subpages (#72-#74), plus pad/pointer reach and undrawn states.
#![allow(dead_code)]

use crate::common::{sandbox, wait_for};
use crate::eclipse_board::{glyphs_in, held, hold_card, kit_tokens, picture, read, rows, words_under};
use emusen_client_fixture::library;
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::input::UiButton;
use emusen_qt::MAIN_URL;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::{Harness, Scene};
use serde_json::{json, Value};
use std::path::PathBuf;

const ECLIPSE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../themes/eclipse");
const TABLE_CORES: &str = include_str!("../../../qt-app/tests/eclipse/cores.tsv");
const TABLE_PAGE: &str = include_str!("../../../qt-app/tests/eclipse/cores_page.tsv");
const TABLE_DETAILS: &str = include_str!("../../../qt-app/tests/eclipse/cores_details.tsv");
const TABLE_FIRMWARE: &str = include_str!("../../../qt-app/tests/eclipse/cores_firmware.tsv");

fn midnight() -> i64 {
    emusen_qt_models::words::now().div_euclid(86_400) * 86_400
}

fn write_board_world(dir: &std::path::Path) -> PathBuf {
    let year = emusen_qt_models::tools::cores::words::current_year();
    let have_date = format!("{year}-10-28");
    let newer_date = format!("{year}-11-09");
    let world = json!({
        "systems": [
            { "id": "nes", "console": "NES", "display_name": "Nintendo Entertainment System", "manufacturer": "Nintendo", "release_year": 1983, "engines": ["Moon"] },
            { "id": "snes", "console": "SNES", "display_name": "Super Nintendo", "manufacturer": "Nintendo", "release_year": 1990, "engines": ["VenusRT"] },
            { "id": "n64", "console": "N64", "display_name": "Nintendo 64", "manufacturer": "Nintendo", "release_year": 1996, "engines": ["Mars"] },
            { "id": "gb", "console": "GB", "display_name": "Game Boy", "manufacturer": "Nintendo", "release_year": 1998, "engines": ["Mercury"] },
            { "id": "sms", "console": "SMS", "display_name": "Master System", "manufacturer": "Sega", "release_year": 1986, "engines": ["Endou"] },
            { "id": "genesis", "console": "Genesis", "display_name": "Genesis", "manufacturer": "Sega", "release_year": 1988, "engines": ["Nephrite"] },
            { "id": "psx", "console": "PlayStation", "display_name": "PlayStation", "manufacturer": "Sony", "release_year": 1994, "engines": ["Diamond", "Beetle PSX HW", "SwanStation"] },
            { "id": "ps2", "console": "PlayStation 2", "display_name": "PlayStation 2", "manufacturer": "Sony", "release_year": 2000, "engines": ["Sapphire", "PCSX2"], "firmware_name": "PlayStation 2 BIOS" },
            { "id": "pce", "console": "PC Engine", "display_name": "PC Engine", "manufacturer": "NEC", "release_year": 1987, "engines": ["Beetle PCE"] },
            { "id": "atari2600", "console": "Atari 2600", "display_name": "Atari 2600", "manufacturer": "Atari", "release_year": 1977, "engines": ["Stella"] },
            { "id": "neogeo", "console": "Neo Geo", "display_name": "Neo Geo", "manufacturer": "SNK", "release_year": 1990, "engines": ["FBNeo"] }
        ],
        "cores": [
            { "library": "/opt/emusen/libmoon.so", "sha256": "01", "engine": "Moon", "directory": "installation", "trust": "manifest", "name": "Moon", "authors": ["EmuSen"], "license": "GPL-3.0-or-later", "systems": ["nes"] },
            { "library": "/opt/emusen/libvenusrt.so", "sha256": "02", "engine": "VenusRT", "directory": "installation", "trust": "manifest", "name": "VenusRT", "authors": ["EmuSen"], "license": "GPL-3.0-or-later", "systems": ["snes"] },
            { "library": "/opt/emusen/libmars.so", "sha256": "03", "engine": "Mars", "directory": "installation", "trust": "manifest", "name": "Mars", "authors": ["EmuSen"], "license": "GPL-3.0-or-later", "systems": ["n64"] },
            { "library": "/opt/emusen/libmercury.so", "sha256": "04", "engine": "Mercury", "directory": "installation", "trust": "manifest", "name": "Mercury", "authors": ["EmuSen"], "license": "GPL-3.0-or-later", "systems": ["gb"] },
            { "library": "/opt/emusen/libendou.so", "sha256": "05", "engine": "Endou", "directory": "installation", "trust": "manifest", "name": "Endou", "authors": ["EmuSen"], "license": "GPL-3.0-or-later", "systems": ["sms"] },
            { "library": "/opt/emusen/libnephrite.so", "sha256": "06", "engine": "Nephrite", "directory": "installation", "trust": "manifest", "name": "Nephrite", "authors": ["EmuSen"], "license": "GPL-3.0-or-later", "systems": ["genesis"] },
            { "library": "/opt/emusen/libdiamond.so", "sha256": "07", "engine": "Diamond", "directory": "installation", "trust": "manifest", "name": "Diamond", "authors": ["EmuSen"], "license": "GPL-3.0-or-later", "systems": ["psx"], "firmware": "optional" },
            { "library": "/opt/emusen/libsapphire.so", "sha256": "08", "engine": "Sapphire", "directory": "installation", "trust": "manifest", "name": "Sapphire", "authors": ["EmuSen"], "license": "GPL-3.0-or-later", "systems": ["ps2"] },
            { "library": "/home/player/Cores/libretro/mupen64plus_next/mupen64plus_next_libretro.so", "sha256": "11", "engine": "Mupen64Plus-Next", "directory": "player", "trust": "approved", "name": "Mupen64Plus-Next", "authors": ["m64p"], "license": "GPLv2", "systems": ["n64"], "libretro": { "sha256": "11" } },
            { "library": "/home/player/Cores/libretro/mednafen_psx_hw/mednafen_psx_hw_libretro.so", "sha256": "12", "engine": "Beetle PSX HW", "directory": "player", "trust": "approved", "name": "Beetle PSX HW", "authors": ["Mednafen"], "license": "GPLv2", "systems": ["psx"], "libretro": { "sha256": "12" } },
            { "library": "/home/player/Cores/libretro/pcsx2/pcsx2_libretro.so", "sha256": "13", "engine": "PCSX2", "directory": "player", "trust": "approved", "name": "PCSX2", "authors": ["PCSX2"], "license": "GPLv3", "systems": ["ps2"], "firmware": "required", "libretro": { "sha256": "13" } },
            { "library": "/home/player/Cores/libretro/mednafen_pce/mednafen_pce_libretro.so", "sha256": "14", "engine": "Beetle PCE", "directory": "player", "trust": "approved", "name": "Beetle PCE", "authors": ["Mednafen"], "license": "GPLv2", "systems": ["pce"], "libretro": { "sha256": "14" } }
        ],
        "space": [440234147840u64, 1099511627776u64],
        "jobs": {
            "core-ls": {
                "events": [
                    { "event": "core", "stem": "mupen64plus_next", "name": "Nintendo - N64 (Mupen64Plus-Next)", "corename": "Mupen64Plus-Next", "systems": ["n64"], "source": "libretro", "build": have_date, "offered": have_date, "state": "installed", "license": "GPLv2", "authors": ["m64p"], "size": 5242880 },
                    { "event": "core", "stem": "mednafen_psx_hw", "name": "Sony - PlayStation (Beetle PSX HW)", "corename": "Beetle PSX HW", "systems": ["psx"], "source": "libretro", "build": have_date, "offered": newer_date, "state": "update", "license": "GPLv2", "authors": ["Mednafen"], "size": 3250585 },
                    { "event": "core", "stem": "pcsx2", "name": "Sony - PlayStation 2 (PCSX2)", "corename": "PCSX2", "systems": ["ps2"], "source": "libretro", "build": have_date, "offered": have_date, "state": "installed", "license": "GPLv3", "authors": ["PCSX2"], "size": 41943040, "firmware": [{ "desc": "PlayStation 2 BIOS", "path": "pcsx2/bios/ps2.bin", "optional": false }] },
                    { "event": "core", "stem": "mednafen_pce", "name": "NEC - PC Engine (Beetle PCE)", "corename": "Beetle PCE", "systems": ["pce"], "source": "libretro", "build": have_date, "offered": newer_date, "state": "update", "license": "GPLv2", "authors": ["Mednafen"], "size": 2097152 },
                    { "event": "core", "stem": "swanstation", "name": "Sony - PlayStation (SwanStation)", "corename": "SwanStation", "systems": ["psx"], "source": "libretro", "build": newer_date, "offered": newer_date, "state": "available", "description": "Fast, with upscaling", "license": "GPLv3", "authors": ["stenzek"], "size": 5033165 },
                    { "event": "core", "stem": "pcsx_rearmed", "name": "Sony - PlayStation (PCSX ReARMed)", "corename": "PCSX ReARMed", "systems": ["psx"], "source": "libretro", "build": newer_date, "offered": newer_date, "state": "available", "description": "Light, for slower machines", "license": "GPLv2", "authors": ["notaz"], "size": 1992294 },
                    { "event": "core", "stem": "stella", "name": "Atari - 2600 (Stella)", "corename": "Stella", "systems": ["atari2600"], "source": "libretro", "build": newer_date, "offered": newer_date, "state": "available", "license": "GPLv2", "authors": ["Stella"], "size": null },
                    { "event": "core", "stem": "fbneo", "name": "Arcade (FinalBurn Neo)", "corename": "FBNeo", "systems": ["neogeo"], "source": "libretro", "build": newer_date, "offered": newer_date, "state": "available", "description": "Arcade & Neo Geo", "license": "FBNeo", "authors": ["FBNeo"], "size": 10485760 },
                    { "event": "done", "op": "core-ls", "catalogue": have_date }
                ],
                "ends": true
            },
            "core-update": {
                "events": [
                    { "event": "plan", "stem": "mednafen_psx_hw", "from": "https://buildbot.libretro.com/nightly/linux/x86_64/latest/mednafen_psx_hw_libretro.so.zip", "build": newer_date },
                    { "event": "installed", "stem": "mednafen_psx_hw", "sha256": "aa", "build": newer_date, "previous": "12", "revoke": [], "tested": { "match": "none" } },
                    { "event": "done", "op": "core-update", "catalogue": newer_date }
                ],
                "ends": true
            }
        },
        "firmware": {
            "ps2": [{ "label": "PlayStation 2 BIOS", "file": "ps2.bin", "required": true, "status": "missing", "need": "Required to start games." }]
        },
        "firmware_folder": "/home/player/EmuSen/Firmware"
    });
    let path = dir.join("board_world.json");
    std::fs::write(&path, world.to_string()).unwrap();
    path
}

fn write_subpage_world(dir: &std::path::Path) -> PathBuf {
    let world = json!({
        "systems": [
            { "id": "psx", "console": "PlayStation", "display_name": "PlayStation", "manufacturer": "Sony", "release_year": 1994, "engines": ["SwanStation"], "firmware_name": "PlayStation BIOS" }
        ],
        "cores": [
            { "library": "/home/player/Cores/libretro/swanstation/swanstation_libretro.so", "sha256": "abababababababababababababababababababababababababababababababab", "engine": "SwanStation", "directory": "player", "trust": "approved", "name": "SwanStation", "authors": ["stenzek"], "license": "GPLv3", "systems": ["psx"], "firmware": "required", "libretro": { "sha256": "abababababababababababababababababababababababababababababababab" } }
        ],
        "space": [440234147840u64, 1099511627776u64],
        "jobs": {
            "core-ls": {
                "events": [
                    {
                        "event": "core",
                        "stem": "swanstation",
                        "name": "Sony - PlayStation (SwanStation)",
                        "corename": "SwanStation",
                        "systems": ["psx"],
                        "source": "libretro",
                        "build": "2026-10-28",
                        "offered": "2026-11-09",
                        "fetched": "2026-10-28",
                        "state": "update",
                        "license": "GPLv3",
                        "authors": ["stenzek"],
                        "size": 5033165,
                        "from": "https://buildbot.libretro.com/nightly/linux/x86_64/latest/swanstation_libretro.so.zip",
                        "index_crc32": "3F9A61C2",
                        "previous": { "build": "2026-10-21", "sha256": "cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd" },
                        "firmware": [{ "desc": "BIOS", "path": "ps1_bios.bin", "optional": false }]
                    },
                    { "event": "done", "op": "core-ls", "catalogue": "2026-11-09" }
                ],
                "ends": true
            }
        },
        "schema": {
            "PlayStation/SwanStation": [
                { "key": "swan_renderer", "label": "Renderer", "kind": "choice", "default": "vulkan", "choices": ["vulkan", "software"], "choice_labels": ["Vulkan", "Software"], "group": "Video" }
            ]
        },
        "firmware": {
            "psx": [{ "label": "PlayStation BIOS", "file": "ps1_bios.bin", "required": true, "status": "present", "need": "Recognised by its content." }]
        },
        "firmware_folder": "/home/player/EmuSen/Firmware"
    });
    let path = dir.join("subpage_world.json");
    std::fs::write(&path, world.to_string()).unwrap();
    path
}

fn write_extra_world(dir: &std::path::Path) -> PathBuf {
    let sha = "4".repeat(64);
    let world = json!({
        "systems": [
            { "id": "psx", "console": "PlayStation", "display_name": "PlayStation", "manufacturer": "Sony", "release_year": 1994, "engines": ["CustomPsx"] }
        ],
        "cores": [
            { "library": "/home/player/Cores/custom_psx.so", "sha256": "55", "engine": "CustomPsx", "directory": "player", "trust": "approved", "name": "Custom PSX", "authors": ["Player"], "license": "MIT", "systems": ["psx"] },
            { "library": "/home/player/Cores/mednafen_psx.so", "sha256": sha, "engine": "Mednafen PSX", "directory": "player", "trust": "needs_approval", "name": "Mednafen PSX", "authors": ["Mednafen"], "license": "GPLv2", "systems": ["psx"], "libretro": { "sha256": sha } }
        ],
        "space": [440234147840u64, 1099511627776u64],
        "jobs": {
            "core-ls": {
                "events": [{ "event": "done", "op": "core-ls", "catalogue": null }],
                "ends": true
            },
            "core-fetch": {
                "events": [
                    { "event": "refused", "reason": "buildbot.libretro.com could not be reached" },
                    { "event": "done", "op": "core-fetch", "counts": {}, "catalogue": null }
                ],
                "ends": true
            }
        }
    });
    let path = dir.join("extra_world.json");
    std::fs::write(&path, world.to_string()).unwrap();
    path
}

fn open_cores<'a>(harness: &'a mut Harness, world_path: &std::path::Path) -> Scene<'a> {
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "coresWorld", &world_path.display().to_string()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("program", "theme", &std::fs::canonicalize(ECLIPSE).unwrap().display().to_string()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    scene.wait(300);
    scene.pad(UiButton::Menu);
    wait_for(&mut scene, "screens", "shown", "HomeCard", 3000);
    scene.wait(300);
    let mut pick = |want: &str| {
        for _ in 0..10 {
            let tree = scene.tree();
            let focus = scene.focus(&tree).map(|i| tree.nodes[i as usize].id.clone()).unwrap_or_default();
            let hit = tree.nodes.iter().any(|n| n.parent == focus && n.visible && n.text.as_deref() == Some(want));
            if hit {
                break;
            }
            scene.pad(UiButton::Down);
            scene.wait(60);
        }
        scene.pad(UiButton::Accept);
        scene.wait(300);
    };
    pick("Tools");
    pick("Cores");
    wait_for(&mut scene, "screens", "shown", "Cores", 3000);
    wait_for(&mut scene, "coresModel", "busy", "false", 5000);
    assert!(scene.set_property("cores", "battery", "82"));
    assert!(scene.set_property("clock", "now", &(midnight() + 21 * 3600 + 41 * 60).to_string()));
    assert!(scene.set_property("cores", "clockText", "9:41 PM"));
    scene.wait(250);
    scene
}

fn hold_cores_board(scene: &mut Scene<'_>, differences: &mut Vec<String>) {
    let (board_h, all_rows) = rows(TABLE_CORES);
    let k = DESIGN_HEIGHT / board_h;
    let tokens = kit_tokens(scene);
    for (width, height) in SIZES {
        scene.resize(width, height);
        scene.wait(300);
        let tree = scene.tree();
        let unit: f64 = read(&tree, "cores", "unit").parse().expect("cores unit");
        let cw = width as f64 / unit;
        let at = format!("cores {width}x{height}");
        let (present, missing): (Vec<_>, Vec<_>) = all_rows.iter().cloned().partition(|r| r.status != "built" || tree.find_named(&r.name).is_some());
        differences.extend(missing.iter().map(|r| format!("{at}: {} is not drawn", r.name)));
        held(&tree, &present, k, unit, cw, &tokens, &at, &|_, _, _| false, differences);
        let expected_words = [
            ("trailWords.0", "Tools", false),
            ("trailSlash.0", "/", false),
            ("trailWords.1", "Cores", false),
            ("battery", "82%", false),
            ("clock", "9:41 PM", false),
            ("filterWords.ready", "9 consoles ready", false),
            ("filterWords.updates", "2 newer builds", false),
            ("filterWords.firmware", "1 needs your firmware", false),
            ("viewLabel", "Get cores for 2 more consoles", false),
            ("asideEyebrow", "SONY · 88 GAMES", true),
            ("asideHeading", "PlayStation", false),
            ("asideInstalledHeading", "ON THIS MACHINE · 2", true),
            ("asideCoreName.0", "Diamond", false),
            ("asideCoreName.1", "Beetle PSX HW", false),
            ("asideYouHave.1", "28 Oct", false),
            ("asideNewer.1", "9 Nov · 3.1 MB", false),
            ("asideAvailableHeading", "MORE FROM LIBRETRO · 2 AVAILABLE", true),
            ("asideAvailName.0", "SwanStation", false),
            ("asideAvailDesc.0", "Fast, with upscaling", false),
            ("asideAvailSize.0", "4.8 MB", false),
            ("coresSummary", "PlayStation · 2 of 4", false),
        ];
        for (name, want, upper) in expected_words {
            let have = read(&tree, name, "text");
            let drawn = if upper { have.to_uppercase() } else { have };
            if drawn != want {
                differences.push(format!("{at}: {name} says {drawn:?} where the board says {want:?}"));
            }
        }
        let help = words_under(&tree, "coresHelp");
        let want_help = ["Update…", "Use for console", "Remove…", "Filter", "Update all · 2", "Back"];
        if help != want_help {
            differences.push(format!("{at}: coresHelp says {help:?} where the board says {want_help:?}"));
        }
        let glyphs = glyphs_in(&tree, "coresHelp");
        let want_glyphs = ["A", "X", "Y", "LB", "MENU", "B"];
        if glyphs != want_glyphs {
            differences.push(format!("{at}: coresHelp glyphs read {glyphs:?} where the board draws {want_glyphs:?}"));
        }
        if let (Some(fw), Some(vb)) = (tree.find_named("filterPill.firmware"), tree.find_named("coresViewButton")) {
            let fw_node = &tree.nodes[fw as usize];
            let vb_node = &tree.nodes[vb as usize];
            if fw_node.x + fw_node.w >= vb_node.x {
                differences.push(format!("{at}: filterPill.firmware right {} overlaps coresViewButton left {}", fw_node.x + fw_node.w, vb_node.x));
            }
        }
    }
    scene.resize(1280, 800);
    scene.wait(200);
}

/// Runs all 4 Eclipse Cores boards (#10, #72-#74), pad navigation, and undrawn states on the active backend.
pub fn cores(backend: &str) {
    std::env::set_var("TZ", "UTC");
    let world = sandbox(&format!("qt-app-eclipse-cores-{backend}"), &[("SNES", "Alpha.sfc")]);
    let dir = world.dir("worlds");
    let board_path = write_board_world(&dir);
    let subpage_path = write_subpage_world(&dir);
    let mut harness = Harness::new();
    let mut differences = Vec::new();

    // Board #10: Eclipse Cores grid & aside.
    {
        let mut scene = open_cores(&mut harness, &board_path);
        // Initially game counts are omitted until consoleCounts is supplied (R-20261125-7).
        assert_eq!(read(&scene.tree(), "consoleMaker.0", "text"), "Nintendo");
        let counts = json!({ "NES": 188, "SNES": 214, "N64": 72, "GB": 160, "SMS": 44, "Genesis": 143, "PlayStation": 88, "PlayStation 2": 56, "PC Engine": 25 });
        assert!(scene.set_property("coresModel", "consoleCounts", &counts.to_string()));
        scene.wait(200);

        // Pad navigation: Up from row 0 enters top strip, Down returns, Down x2 to row 2 PlayStation, Right/Accept into aside -> Down to Beetle PSX HW.
        scene.pad(UiButton::Up);
        scene.wait(150);
        let state: Value = serde_json::from_str(&scene.property("coresModel", "eclipseCores")).unwrap();
        assert_eq!(state["focusZone"], "top");
        scene.pad(UiButton::Down);
        scene.wait(150);
        scene.pad(UiButton::Down);
        scene.pad(UiButton::Down);
        scene.wait(150);
        let state: Value = serde_json::from_str(&scene.property("coresModel", "eclipseCores")).unwrap();
        assert_eq!(state["focusedConsole"], "PlayStation");
        scene.pad(UiButton::Accept);
        scene.pad(UiButton::Down);
        scene.wait(200);
        let state: Value = serde_json::from_str(&scene.property("coresModel", "eclipseCores")).unwrap();
        assert_eq!(state["focusZone"], "aside");
        assert_eq!(state["focusedAsideIndex"], 1);

        hold_cores_board(&mut scene, &mut differences);
        picture(&mut scene, &format!("cores-1920x1200-{backend}.png"));

        // Immediate #14 confirmation on A (Update) -> B cancels cleanly.
        scene.pad(UiButton::Accept);
        wait_for(&mut scene, "coresModel", "asking", "true", 2000);
        wait_for(&mut scene, "coresQuestion", "opened", "true", 2000);
        assert_eq!(scene.property("coresModel", "questionTitle"), "Update Beetle PSX HW?");
        scene.pad(UiButton::Back);
        wait_for(&mut scene, "coresModel", "asking", "false", 2000);
        wait_for(&mut scene, "coresQuestion", "opened", "false", 2000);
        wait_for(&mut scene, "coresEclipse", "sheetOpen", "false", 2000);

        // Left returns from aside to grid; VIEW (Options) toggles remaining consoles (size omitted when null).
        scene.pad(UiButton::Left);
        scene.wait(150);
        assert_eq!(serde_json::from_str::<Value>(&scene.property("coresModel", "eclipseCores")).unwrap()["focusZone"], "grid");
        scene.pad(UiButton::Options);
        scene.wait(200);
        let rem: Value = serde_json::from_str(&scene.property("coresModel", "eclipseCores")).unwrap();
        assert_eq!(rem["showingRemaining"], true);
        assert_eq!(rem["aside"]["available"][0]["size"], "");
        assert_eq!(read(&scene.tree(), "asideAvailSize.0", "visible"), "false", "omitted size hides size label");
        scene.pad(UiButton::Options);
        scene.wait(150);
        // Menu (Update all · 2) asks immediate #14 dialog; Right enters aside, X switches default, Y asks Remove.
        scene.pad(UiButton::Menu);
        wait_for(&mut scene, "coresModel", "asking", "true", 2000);
        wait_for(&mut scene, "coresQuestion", "opened", "true", 2000);
        assert_eq!(scene.property("coresModel", "questionTitle"), "Update 2 cores?");
        scene.pad(UiButton::Back);
        wait_for(&mut scene, "coresModel", "asking", "false", 2000);
        wait_for(&mut scene, "coresQuestion", "opened", "false", 2000);
        wait_for(&mut scene, "coresEclipse", "sheetOpen", "false", 2000);
        scene.pad(UiButton::Down);
        scene.pad(UiButton::Down);
        scene.wait(100);
        assert_eq!(serde_json::from_str::<Value>(&scene.property("coresModel", "eclipseCores")).unwrap()["focusedConsole"], "PlayStation");
        scene.pad(UiButton::Accept);
        scene.pad(UiButton::Down);
        scene.wait(150);
        scene.pad(UiButton::Search);
        wait_for(&mut scene, "coresModel", "asking", "true", 2000);
        wait_for(&mut scene, "coresQuestion", "opened", "true", 2000);
        assert_eq!(scene.property("coresModel", "questionTitle"), "Remove Beetle PSX HW?");
        scene.pad(UiButton::Back);
        wait_for(&mut scene, "coresModel", "asking", "false", 2000);
        wait_for(&mut scene, "coresQuestion", "opened", "false", 2000);
        wait_for(&mut scene, "coresEclipse", "sheetOpen", "false", 2000);
        scene.pad(UiButton::Screensaver);
        scene.wait(150);
        let switched: Value = serde_json::from_str(&scene.property("coresModel", "eclipseCores")).unwrap();
        assert_eq!(switched["aside"]["installed"][0]["name"], "Beetle PSX HW");
        assert_eq!(switched["aside"]["installed"][0]["badge"], "Plays this console");
        scene.pad(UiButton::PageDown);
        scene.wait(120);
        let filtered: Value = serde_json::from_str(&scene.property("coresModel", "eclipseCores")).unwrap();
        assert_eq!(filtered["filters"][1]["active"], true, "RB steps Cores filter to newer builds");
        scene.pad(UiButton::PageUp);
        scene.wait(120);
        let restored: Value = serde_json::from_str(&scene.property("coresModel", "eclipseCores")).unwrap();
        assert_eq!(restored["filters"][0]["active"], true, "LB restores ready filter");
        scene.assert_quiet();
    }

    // Boards #72, #73, #74: Core Page, Core Details, and Core Firmware.
    {
        let mut scene = open_cores(&mut harness, &subpage_path);
        scene.pad(UiButton::Accept);
        scene.wait(150);
        scene.pad(UiButton::Random);
        wait_for(&mut scene, "coresModel", "cardOpen", "true", 2000);
        scene.wait(350);
        let tokens = kit_tokens(&mut scene);

        // #72 Core Page.
        hold_card(
            &mut scene,
            "cores",
            &tokens,
            TABLE_PAGE,
            &[
                ("cardBreadcrumb", "CORES · PLAYSTATION", true),
                ("cardTitle", "SwanStation", false),
                ("paneLead.0", "Runs PlayStation games through EmuSen's libretro host, in a sandbox of its own.", false),
                ("paneMuted.0", "Needs a BIOS of your own, which EmuSen never downloads.", false),
                ("paneStatus", "✓ You have one.", false),
                ("cardBattery", "82%", false),
                ("cardClock", "9:41 PM", false),
                ("cardPosition", "1 of 8", false),
            ],
            &["Choose", "Back", "Close"],
            &["A", "B", "MENU"],
            &mut differences,
        );
        picture(&mut scene, &format!("core-page-1920x1200-{backend}.png"));

        // #73 Core Details (row 7 on #72 -> Details; row 4 -> From).
        for _ in 0..7 {
            scene.pad(UiButton::Down);
            scene.wait(50);
        }
        scene.pad(UiButton::Accept);
        scene.wait(300);
        for _ in 0..4 {
            scene.pad(UiButton::Down);
            scene.wait(50);
        }
        scene.wait(200);
        hold_card(
            &mut scene,
            "cores",
            &tokens,
            TABLE_DETAILS,
            &[
                ("cardBreadcrumb", "CORES · PLAYSTATION", true),
                ("cardTitle", "SwanStation · details", false),
                ("detailPaneLabel", "FROM", true),
                ("detailPaneValue", "buildbot.libretro.com, index CRC-32 3F9A61C2", false),
                ("cardBattery", "82%", false),
                ("cardClock", "9:41 PM", false),
                ("cardPosition", "5 of 9", false),
            ],
            &["Choose", "Back", "Close"],
            &["A", "B", "MENU"],
            &mut differences,
        );
        picture(&mut scene, &format!("core-details-1920x1200-{backend}.png"));

        // #74 Core Firmware (Back to #72, navigate to Firmware at row 2).
        scene.pad(UiButton::Back);
        scene.wait(250);
        for _ in 0..5 {
            scene.pad(UiButton::Up);
            scene.wait(50);
        }
        scene.pad(UiButton::Accept);
        scene.wait(300);
        hold_card(
            &mut scene,
            "cores",
            &tokens,
            TABLE_FIRMWARE,
            &[
                ("cardBreadcrumb", "CORES · PLAYSTATION", true),
                ("cardTitle", "SwanStation · Firmware", false),
                ("paneLead.0", "Your own: PS1_BIOS", false),
                ("paneMuted.0", "Recognised by its content.", false),
                ("cardBattery", "82%", false),
                ("cardClock", "9:41 PM", false),
                ("cardPosition", "1 of 2", false),
            ],
            &["Choose", "Back", "Close"],
            &["A", "B", "MENU"],
            &mut differences,
        );
        picture(&mut scene, &format!("core-firmware-1920x1200-{backend}.png"));

        // Open Folder on #74 and on #72 both emit OPEN_FOLDER through ToolScreen -> Folders.
        let folders_caught = |s: &mut Scene<'_>| -> Vec<String> {
            let ptr = s.object("folders");
            assert!(!ptr.is_null(), "folders");
            unsafe { &*(ptr as *const emusen_qt_models::folders::ffi::Folders) }.caught().iter().map(|u| u.to_string()).collect()
        };
        assert!(scene.set_property("folders", "catching", "true"));
        scene.pad(UiButton::Down);
        scene.wait(100);
        scene.pad(UiButton::Accept);
        scene.wait(150);
        let fw_url = emusen_qt_models::folders::url_of("/home/player/EmuSen/Firmware");
        assert_eq!(folders_caught(&mut scene), [fw_url.clone()]);
        scene.pad(UiButton::Back);
        wait_for(&mut scene, "coresModel", "cardTitle", "SwanStation", 2000);
        for _ in 0..4 {
            scene.pad(UiButton::Down);
            scene.wait(50);
        }
        scene.pad(UiButton::Accept);
        scene.wait(150);
        let core_url = emusen_qt_models::folders::url_of("/home/player/Cores/libretro/swanstation");
        assert_eq!(folders_caught(&mut scene), [fw_url, core_url]);
        scene.assert_quiet();
    }

    // Offline status pill, Origin::Added ("Added by you"), and "N to approve" flow.
    {
        let extra_path = write_extra_world(&dir);
        let mut scene = open_cores(&mut harness, &extra_path);
        wait_for(&mut scene, "coresModel", "busy", "false", 5000);
        scene.wait(200);
        let tree = scene.tree();
        assert_eq!(read(&tree, "coresStatusPill", "visible"), "true");
        assert_eq!(read(&tree, "coresStatusText", "text"), "Buildbot.libretro.com could not be reached.");
        assert_eq!(read(&tree, "filterWords.approval", "text"), "1 to approve");
        assert_eq!(read(&tree, "asideCoreKind.0", "text"), "Added by you");
        scene.pad(UiButton::Accept);
        scene.pad(UiButton::Down);
        scene.wait(150);
        scene.pad(UiButton::Accept);
        wait_for(&mut scene, "coresModel", "asking", "true", 2000);
        wait_for(&mut scene, "coresQuestion", "opened", "true", 2000);
        assert!(scene.property("coresModel", "questionTitle").contains("Approve"));
        scene.pad(UiButton::Back);
        wait_for(&mut scene, "coresModel", "asking", "false", 2000);
        wait_for(&mut scene, "coresQuestion", "opened", "false", 2000);
        wait_for(&mut scene, "coresEclipse", "sheetOpen", "false", 2000);
        scene.assert_quiet();
    }

    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
}
