//! Shared oracle for Eclipse Packages (#22 Library, #23 Layers, #69 Package Page).
#![allow(dead_code)]

use crate::common::{approved_console_core, wait_for};
use crate::eclipse_board::{glyphs_in, held, kit_tokens, picture, read, rows, words_under};
use emusen_client_fixture::{library, World};
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::{DIR_PACKAGES, MODEL_APP_SETTINGS};
use emusen_platform_client::tools::{Step, ToolJob};
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_models::themes;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::lint;
use emusen_qt_test::{Harness, Scene};
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;

const TABLE_LIBRARY: &str = include_str!("../../../qt-app/tests/eclipse/packages_library.tsv");
const TABLE_LAYERS: &str = include_str!("../../../qt-app/tests/eclipse/packages_layers.tsv");
const TABLE_PAGE: &str = include_str!("../../../qt-app/tests/eclipse/package_page.tsv");

fn midnight() -> i64 {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    now - now.rem_euclid(86_400)
}

fn ips_patch(offset: u32, byte: u8) -> Vec<u8> {
    let mut out = b"PATCH".to_vec();
    out.push(((offset >> 16) & 0xff) as u8);
    out.push(((offset >> 8) & 0xff) as u8);
    out.push((offset & 0xff) as u8);
    out.extend_from_slice(&[0x00, 0x01, byte]);
    out.extend_from_slice(b"EOF");
    out
}

fn run_tool_job(platform: &Arc<Platform>, request: Value) {
    let job = ToolJob::start(platform, &request).expect("library-tool job starts");
    loop {
        match job.next(5000).expect("library-tool step") {
            Step::Event(ev) => {
                let kind = ev.get("event").and_then(Value::as_str);
                if kind == Some("start") && ev.get("refused").and_then(Value::as_array).is_some_and(|a| !a.is_empty()) {
                    panic!("library-tool job refused input: {ev}");
                }
                if matches!(kind, Some("outcome" | "package"))
                    && matches!(ev.get("state").and_then(Value::as_str), Some("failed" | "refused"))
                {
                    panic!("library-tool job refused: {ev}");
                }
            }
            Step::Waiting => {}
            Step::Ended => break,
        }
    }
}

fn click_named(scene: &mut Scene<'_>, name: &str) {
    let tree = scene.tree();
    let idx = tree
        .nodes
        .iter()
        .rposition(|n| n.name == name && n.visible)
        .or_else(|| tree.find_named(name).map(|i| i as usize))
        .unwrap_or_else(|| panic!("{name} is drawn"));
    let n = &tree.nodes[idx];
    scene.click(n.x + n.w / 2.0, n.y + n.h / 2.0);
}

fn wait_node(scene: &mut Scene<'_>, name: &str, property: &str, value: &str, ms: i32) {
    let mut waited = 0;
    let mut got = String::new();
    while waited < ms {
        let tree = scene.tree();
        got = read(&tree, name, property);
        if got == value {
            return;
        }
        scene.wait(50);
        waited += 50;
    }
    let tree = scene.tree();
    let act_lines = scene.property("jobsModel", "actionLines");
    let frame_focus = read(&tree, "packagesFrame", "activeFocus");
    let view_focus = read(&tree, "packagesLibraryScreen", "activeFocus");
    let screen_code = scene.property("jobsModel", "eclipseScreen");
    assert_eq!(
        read(&tree, name, property),
        value,
        "{name}.{property} after {ms} ms (last {got:?}, screen={screen_code:?}, viewFocus={view_focus:?}, frameFocus={frame_focus:?}, actionLines={act_lines:?})"
    );
}

fn wait_present(scene: &mut Scene<'_>, name: &str, ms: i32) {
    let mut waited = 0;
    while waited < ms {
        let tree = scene.tree();
        if tree.nodes.iter().any(|n| n.name == name && n.visible) {
            return;
        }
        scene.wait(200);
        waited += 200;
    }
    let busy = scene.property("jobsModel", "busy");
    let screen = scene.property("jobsModel", "eclipseScreen");
    let problem = scene.property("jobsModel", "problem");
    let cores = scene.property("jobsModel", "eclipseCores");
    panic!("{name} never became visible after {ms} ms (busy={busy:?}, eclipseScreen={screen:?}, problem={problem:?}, eclipseCores={cores:?})");
}

fn check_board(
    scene: &mut Scene<'_>,
    canvas: &str,
    unit_item: &str,
    table: &str,
    words: &[(&str, &str, bool)],
    help_bar: &str,
    help: &[&str],
    help_glyphs: &[&str],
    differences: &mut Vec<String>,
) {
    let (board_h, all_rows) = rows(table);
    let k = DESIGN_HEIGHT / board_h;
    let tokens = kit_tokens(scene);
    for (width, height) in SIZES {
        scene.resize(width, height);
        scene.wait(300);
        let tree = scene.tree();
        let unit: f64 = read(&tree, unit_item, "unit").parse().expect("canvas unit");
        let cw = width as f64 / unit;
        let at = format!("{canvas} {width}x{height}");
        let (present, missing): (Vec<_>, Vec<_>) = all_rows
            .iter()
            .cloned()
            .partition(|r| r.status != "built" || tree.find_named(&r.name).is_some());
        differences.extend(missing.iter().map(|r| format!("{at}: {} is not drawn", r.name)));
        held(&tree, &present, k, unit, cw, &tokens, &at, &|_, _, _| false, differences);
        for (name, want, upper) in words {
            let have = read(&tree, name, "text");
            let drawn = if *upper { have.to_uppercase() } else { have.clone() };
            if drawn != *want {
                differences.push(format!("{at}: {name} says {drawn:?} where board expects {want:?}"));
            }
        }
        let line = words_under(&tree, help_bar);
        if line != help {
            differences.push(format!("{at}: {help_bar} says {line:?} where board expects {help:?}"));
        }
        let letters = glyphs_in(&tree, help_bar);
        if letters != help_glyphs {
            differences.push(format!("{at}: {help_bar} glyphs read {letters:?} where board draws {help_glyphs:?}"));
        }
    }
    scene.resize(1280, 800);
    scene.wait(250);
}

fn verify_qml_token_purity() {
    let qml_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../qt-app/qml");
    for file in ["PackagesEclipse.qml", "PackageLayersEclipse.qml", "PackagePageEclipse.qml"] {
        let p = qml_dir.join(file);
        let src = std::fs::read_to_string(&p).expect("read qml");
        assert!(!src.contains('#'), "{file} contains forbidden hex literal '#'");
    }
    let issues = lint::rules(&[(&qml_dir, true)]);
    assert!(issues.is_empty(), "qml-rules failed:\n{}", issues.join("\n"));
}

fn setup_world(backend: &str) -> (World, std::path::PathBuf, std::path::PathBuf) {
    let world = World::new(&format!("qt-app-eclipse-packages-{backend}"));
    let cores = world.dir("Cores");
    approved_console_core(&cores);

    let roms = world.dir("Roms");
    let packages = world.dir("Packages");
    let compressed = world.dir("Compressed");
    let downloads = world.dir("Downloads");

    let snes_dir = roms.join("snes");
    let saturn_dir = roms.join("saturn");
    let gb_dir = roms.join("gb");
    let nes_dir = roms.join("nes");
    for d in [&snes_dir, &saturn_dir, &gb_dir, &nes_dir] {
        std::fs::create_dir_all(d).unwrap();
    }

    let mut nes = b"NES\x1a\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
    nes.resize(16 + 16384, 0);
    std::fs::write(nes_dir.join("Contra.nes"), &nes).unwrap();
    std::fs::write(snes_dir.join("Super Metroid.sfc"), vec![0u8; 32768]).unwrap();
    std::fs::write(gb_dir.join("Super Mario Land.gb"), vec![0u8; 32768]).unwrap();
    std::fs::write(
        saturn_dir.join("Panzer Dragoon II Zwei (USA).cue"),
        "FILE \"Panzer Dragoon II Zwei (USA).bin\" BINARY\n  TRACK 01 MODE1/2352\n    INDEX 01 00:00:00\n",
    )
    .unwrap();
    std::fs::write(saturn_dir.join("Panzer Dragoon II Zwei (USA).bin"), vec![0u8; 2352 * 4]).unwrap();

    let patch1 = downloads.join("Super Metroid Redux v1.8.ips");
    let patch2 = downloads.join("MSU Audio Hook.ips");
    let patch3 = downloads.join("extra_fix.ips");
    std::fs::write(&patch1, ips_patch(0x000010, 0x42)).unwrap();
    std::fs::write(&patch2, ips_patch(0x000020, 0x7a)).unwrap();
    std::fs::write(&patch3, ips_patch(0x000030, 0x99)).unwrap();

    let platform = Arc::new(Platform::load(library()).unwrap());
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = json!(roms.to_str().unwrap());
    settings["PackagesDirectory"] = json!(packages.to_str().unwrap());
    settings["CompressedDirectory"] = json!(compressed.to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();

    run_tool_job(&platform, json!({ "op": "pkg-make", "paths": ["snes/Super Metroid.sfc"], "self_contained": false, "zip": false }));
    run_tool_job(&platform, json!({ "op": "pkg-patch-add", "paths": ["SNES/Super Metroid.sfc"], "patch": patch1.to_str().unwrap() }));
    run_tool_job(&platform, json!({ "op": "pkg-patch-add", "paths": ["SNES/Super Metroid.sfc"], "patch": patch2.to_str().unwrap() }));
    run_tool_job(&platform, json!({ "op": "pkg-identity", "paths": ["SNES/Super Metroid.sfc"], "identity": "patched" }));
    run_tool_job(&platform, json!({ "op": "pkg-make", "paths": ["saturn/Panzer Dragoon II Zwei (USA).cue"], "self_contained": false, "zip": false }));
    run_tool_job(&platform, json!({ "op": "pkg-make", "paths": ["nes/Contra.nes"], "self_contained": true, "zip": false }));

    let power = world.power_supply().join("BAT0");
    std::fs::create_dir_all(&power).unwrap();
    for (k, v) in [("type", "Battery"), ("capacity", "82"), ("status", "Discharging")] {
        std::fs::write(power.join(k), format!("{v}\n")).unwrap();
    }
    let packages_dir = platform.directory(DIR_PACKAGES).unwrap().unwrap();
    (world, patch3, packages_dir)
}

fn find_row_index_by_title(scene: &mut Scene<'_>, target: &str) -> Option<usize> {
    let tree = scene.tree();
    for i in 0..16 {
        if read(&tree, &format!("packageRowTitle.{i}"), "text") == target {
            return Some(i);
        }
    }
    None
}

pub fn packages(backend: &str) {
    verify_qml_token_purity();
    std::env::set_var("TZ", "UTC");
    let (world, extra_patch, packages_dir) = setup_world(backend);
    let cores = world.dir("Cores");

    let mut harness = Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("clock", "now", &(midnight() + 21 * 3600 + 41 * 60).to_string()));
    assert!(scene.set_property("library", "cores", cores.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("program", "theme", &themes::built_in_folder("eclipse")));
    wait_for(&mut scene, "library", "loading", "false", 15000);
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);
    wait_for(&mut scene, "theme", "home", "true", 5000);
    scene.wait(300);

    // Navigate Home -> Consoles -> RB -> Collections -> RB -> Tools -> Packages.
    click_named(&mut scene, "nav.consoles");
    wait_for(&mut scene, "screens", "shown", "Consoles", 3000);
    scene.pad(UiButton::PageDown);
    wait_for(&mut scene, "screens", "shown", "Collections", 3000);
    scene.pad(UiButton::PageDown);
    wait_for(&mut scene, "screens", "shown", "Tools", 3000);
    click_named(&mut scene, "toolCard.2");
    wait_for(&mut scene, "screens", "shown", "Tool", 4000);
    assert_eq!(scene.property("screens", "tool"), "packages");
    wait_present(&mut scene, "packageRow.0", 10000);
    scene.wait(300);

    let mut differences = Vec::new();

    // Board #22: Packages Library.
    check_board(
        &mut scene,
        "packagesLibraryScreen",
        "jobs",
        TABLE_LIBRARY,
        &[
            ("trailWords.0", "Tools", false),
            ("trailWords.1", "Packages", false),
            ("battery", "82%", false),
            ("clock", "9:41 PM", false),
            ("centerEyebrow", "ALL GAMES · 4", false),
            ("positionCounterText", "1 of 4", false),
        ],
        "packagesHelpBar",
        &["Open package", "Mark for batch", "Queue job", "Filter", "Console", "Review queue", "Search", "Back"],
        &["A", "Y", "X", "LB", "LT", "MENU", "VIEW", "B"],
        &mut differences,
    );
    if backend == "card" {
        picture(&mut scene, "packages_library.png");
    }

    // Filter switching (RB -> Packaged -> LB -> All games) and batch mark + confirm execution.
    scene.pad(UiButton::PageDown);
    wait_node(&mut scene, "centerEyebrow", "text", "PACKAGED · 3", 3000);
    scene.pad(UiButton::PageUp);
    wait_node(&mut scene, "centerEyebrow", "text", "ALL GAMES · 4", 3000);

    let metroid_idx = find_row_index_by_title(&mut scene, "Super Metroid").expect("Super Metroid row");
    click_named(&mut scene, &format!("packageRow.{metroid_idx}"));
    wait_node(&mut scene, "inspectorTitle", "text", "Super Metroid", 3000);

    // Mark Super Metroid for batch, verify queued state, then unmark before opening page.
    scene.pad(UiButton::Search);
    wait_node(&mut scene, "batchChip", "text", "1 marked", 3000);
    scene.pad(UiButton::Search);
    wait_node(&mut scene, "batchChip", "text", "0 marked", 3000);

    // Open #69 Package Page (Super Metroid).
    scene.pad(UiButton::Accept);
    wait_present(&mut scene, "packagePageScreen", 4000);
    wait_node(&mut scene, "packagePageTitle", "text", "Super Metroid", 3000);
    scene.wait(250);

    // AUDIT.md:180 + AUDIT.md:183: packaged games must omit Make Package, Unmake, From the mods library, Export patched game.
    let tree = scene.tree();
    let page_labels: Vec<String> = (0..12)
        .map(|i| read(&tree, &format!("packageActionLabel.{i}"), "text"))
        .filter(|s| !s.is_empty())
        .collect();
    assert!(!page_labels.iter().any(|l| l.contains("Make Package") || l.contains("Make linked")), "AUDIT.md:180 violated: {page_labels:?}");
    assert!(!page_labels.iter().any(|l| l.contains("Unmake") || l.contains("mods library") || l.contains("Export the patched game")));
    assert!(page_labels.contains(&"Soft patches".to_owned()));
    assert!(page_labels.contains(&"Check Package".to_owned()));
    assert!(page_labels.contains(&"Stores".to_owned()));
    assert!(page_labels.contains(&"Open Folder".to_owned()));

    check_board(
        &mut scene,
        "packagePageScreen",
        "jobs",
        TABLE_PAGE,
        &[
            ("trailWords.0", "Tools", false),
            ("trailWords.1", "Packages", false),
            ("trailWords.2", "Super Metroid", false),
            ("battery", "82%", false),
            ("clock", "9:41 PM", false),
            ("packagePageTitle", "Super Metroid", false),
            ("packageFactLabel.0", "Package", false),
            ("packageFactValue.0", "Linked", false),
            ("packageActionLabel.0", "Soft patches", false),
            ("packageActionValue.0", "2", false),
        ],
        "packagePageHelpBar",
        &["Choose", "Back", "Apply…"],
        &["A", "B", "MENU"],
        &mut differences,
    );
    if backend == "card" {
        picture(&mut scene, "package_page.png");
    }

    // Open #23 Package Layers via Soft patches action row 0.
    scene.pad(UiButton::Accept);
    wait_present(&mut scene, "packageLayersScreen", 4000);
    wait_node(&mut scene, "layersTitle", "text", "Super Metroid", 3000);
    scene.wait(250);

    // AUDIT.md:149: cartridge-only rail hides Saturn disc packages.
    let layers_tree = scene.tree();
    assert!(layers_tree.find_named("layersRailRow.0").is_some());
    assert!(layers_tree.find_named("layersRailRow.1").is_none(), "disc packages must stay hidden on #23");

    check_board(
        &mut scene,
        "packageLayersScreen",
        "jobs",
        TABLE_LAYERS,
        &[
            ("trailWords.0", "Tools", false),
            ("trailWords.1", "Packages", false),
            ("trailWords.2", "Patch Layers", false),
            ("battery", "82%", false),
            ("clock", "9:41 PM", false),
            ("layersTitle", "Super Metroid", false),
            ("baseCardTitle", "Base cartridge dump (read-only)", false),
            ("positionCounterText", "1 of 3", false),
        ],
        "layersHelpBar",
        &["Verify chain", "Move patch down / up", "Add patch file…", "Remove patch…", "Game", "Back"],
        &["A", "LB", "X", "Y", "LT", "B"],
        &mut differences,
    );
    if backend == "card" {
        picture(&mut scene, "packages_layers.png");
    }

    // Real reorder (#2 <-> #1 via Down onto patch 1 then RB move up) + Add patch via FileDialog seam + Remove patch.
    scene.pad(UiButton::Down);
    wait_node(&mut scene, "positionCounterText", "text", "2 of 3", 2000);
    scene.pad(UiButton::PageDown);
    wait_node(&mut scene, "patchName.0", "text", "Super Metroid Redux v1.8.ips", 8000);

    assert!(scene.set_property("jobsModel", "fileCatching", "true"));
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "jobsModel", "fileWaiting", "true", 3000);
    assert!(scene.set_property("jobsModel", "fileAnswer", extra_patch.to_str().unwrap()));
    wait_present(&mut scene, "patchCard.2", 8000);
    wait_node(&mut scene, "positionCounterText", "text", "1 of 4", 3000);

    scene.pad(UiButton::Search);
    wait_for(&mut scene, "jobsModel", "asking", "true", 3000);
    wait_for(&mut scene, "jobsQuestion", "opened", "true", 3000);
    scene.wait(300);
    scene.pad(UiButton::Right);
    scene.wait(150);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "jobsModel", "asking", "false", 3000);
    wait_node(&mut scene, "positionCounterText", "text", "1 of 3", 8000);

    // Verify Open Folder from #69 cleanly pops back to #69 -> Folders.catching.
    scene.pad(UiButton::Back);
    wait_present(&mut scene, "packagePageScreen", 4000);
    let open_folder_idx = {
        let t = scene.tree();
        (0..12)
            .find(|i| read(&t, &format!("packageActionLabel.{i}"), "text") == "Open Folder")
            .expect("Open Folder row")
    };
    let metroid_pkg_dir = packages_dir.join("SNES/Super Metroid.sfc");
    assert!(metroid_pkg_dir.exists(), "missing {}", metroid_pkg_dir.display());
    assert!(scene.set_property("folders", "catching", "true"));
    click_named(&mut scene, &format!("packageActionRow.{open_folder_idx}"));
    wait_for(&mut scene, "folders", "caught", &format!("file://{}", metroid_pkg_dir.display()), 3000);

    // Unpackaged GB game on #22 offers Make linked package + Make self-contained; batch runs pkg-make cleanly.
    scene.pad(UiButton::Back);
    wait_present(&mut scene, "packagesLibraryScreen", 4000);
    let gb_idx = find_row_index_by_title(&mut scene, "Super Mario Land").expect("Super Mario Land row");
    click_named(&mut scene, &format!("packageRow.{gb_idx}"));
    wait_node(&mut scene, "inspectorTitle", "text", "Super Mario Land", 3000);
    scene.pad(UiButton::Search);
    wait_node(&mut scene, "batchChip", "text", "1 marked", 3000);
    scene.pad(UiButton::Menu);
    wait_for(&mut scene, "jobsModel", "asking", "true", 3000);
    wait_for(&mut scene, "jobsQuestion", "opened", "true", 3000);
    scene.wait(300);
    scene.pad(UiButton::Right);
    scene.wait(150);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "jobsModel", "asking", "false", 3000);
    wait_node(&mut scene, "centerEyebrow", "text", "ALL GAMES · 4", 3000);
    wait_node(&mut scene, "batchChip", "text", "0 marked", 10000);

    // Confirm Super Mario Land package page now omits Make Package after packaging.
    let new_gb_idx = find_row_index_by_title(&mut scene, "Super Mario Land").expect("Super Mario Land packaged row");
    click_named(&mut scene, &format!("packageRow.{new_gb_idx}"));
    wait_node(&mut scene, "inspectorTitle", "text", "Super Mario Land", 3000);
    scene.pad(UiButton::Accept);
    wait_present(&mut scene, "packagePageScreen", 4000);
    let post_tree = scene.tree();
    let post_labels: Vec<String> = (0..10)
        .map(|i| read(&post_tree, &format!("packageActionLabel.{i}"), "text"))
        .filter(|s| !s.is_empty())
        .collect();
    assert!(!post_labels.iter().any(|l| l.contains("Make Package") || l.contains("Make linked")), "post-make AUDIT.md:180 violated: {post_labels:?}");

    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
}
