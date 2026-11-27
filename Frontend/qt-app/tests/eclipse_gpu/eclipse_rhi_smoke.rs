// Single-pass Vulkan RHI smoke check across Eclipse screens and overlays at 1920x1200 (replacing 25 zero-pixel GPU clones).

use crate::common::{approved_console_core, draw_covers, sandbox, wait_for};
use crate::eclipse_board::read;
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_qt::MAIN_URL;
use emusen_qt_models::themes;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::{Harness, Scene};

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

fn check_rhi(scene: &mut Scene<'_>, label: &str, canvas: Option<&str>, scaled_items: &[&str], points: &[(f64, f64)]) {
    scene.wait(150);
    let tree = scene.tree();
    if let Some(canvas_name) = canvas {
        let unit: f64 = read(&tree, canvas_name, "unit").parse().unwrap_or_else(|_| panic!("{label}: {canvas_name}.unit missing"));
        for item in scaled_items {
            let scale: f64 = read(&tree, item, "pixelScale").parse().unwrap_or_else(|_| panic!("{label}: {item}.pixelScale missing"));
            assert!((scale - unit).abs() <= 1e-6, "{label}: {item}.pixelScale={scale} != {canvas_name}.unit={unit}");
        }
    }
    let colours = scene.colours_at(points);
    for (pt, c) in points.iter().zip(colours.iter()) {
        let px = c.unwrap_or_else(|| panic!("{label}: no pixel read at {pt:?}"));
        let lum = u32::from(px.r) + u32::from(px.g) + u32::from(px.b);
        assert!(lum > 0, "{label}: pixel at {pt:?} is pure black ({px:?})");
    }
}

#[test]
fn qt_3_4_eclipse_rhi_smoke_renders_all_screens_and_overlays_on_vulkan() {
    let kwin = Compositor::start(1920, 1200);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");

    let world = sandbox("qt-app-eclipse-rhi-smoke", &[("SNES", "Aurora Drift (Synthetic).sfc"), ("GB", "Laurel Crown (Synthetic).gb")]);
    let cores = world.dir("Cores");
    approved_console_core(&cores);
    let media = world.dir("Media");
    let mut harness = Harness::new();
    let cover = media.join("snes/covers/Aurora Drift (Synthetic).png");
    draw_covers(&mut harness, &world.root, &[(cover, (300, 420), 0, "Aurora Drift".into())], |_| {});

    let mut scene = harness.load(MAIN_URL, 1920, 1200);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("library", "cores", cores.to_str().unwrap()));
    assert!(scene.set_property("program", "coverStores", media.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("program", "theme", &themes::built_in_folder("eclipse")));
    wait_for(&mut scene, "library", "loading", "false", 15000);
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);
    let warnings = scene.property("host.theme", "warnings");
    assert!(warnings.is_empty() && scene.property("host.theme", "failed") == "false", "Eclipse loads clean: {warnings}");
    wait_for(&mut scene, "theme", "home", "true", 5000);

    // Home backdrop and corona disc, plus HomeCard overlay.
    check_rhi(&mut scene, "Home", Some("homeCanvas"), &["backdrop", "disc"], &[(260.0, 560.0), (1400.0, 320.0)]);
    scene.pad(UiButton::Menu);
    wait_for(&mut scene, "screens", "shown", "HomeCard", 3000);
    check_rhi(&mut scene, "HomeCard", None, &[], &[(320.0, 600.0)]);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 3000);

    // Search overlay.
    click_named(&mut scene, "search");
    wait_for(&mut scene, "screens", "shown", "Search", 3000);
    check_rhi(&mut scene, "Search", Some("searchCanvas"), &["searchBackdrop"], &[(960.0, 400.0)]);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 3000);

    // Consoles and Games views.
    click_named(&mut scene, "nav.consoles");
    wait_for(&mut scene, "screens", "shown", "Consoles", 3000);
    check_rhi(&mut scene, "Consoles", Some("consolesCanvas"), &["consolesBackdrop", "consoleDisc"], &[(300.0, 600.0), (1400.0, 500.0)]);
    click_named(&mut scene, "plate.0");
    wait_for(&mut scene, "screens", "shown", "Grid", 3000);
    check_rhi(&mut scene, "Games", Some("gridCanvas"), &["gamesBackdrop", "gamesDisc"], &[(260.0, 600.0), (1200.0, 500.0)]);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Consoles", 3000);

    // Collections view.
    scene.pad(UiButton::PageDown);
    wait_for(&mut scene, "screens", "shown", "Collections", 3000);
    check_rhi(&mut scene, "Collections", Some("collectionsCanvas"), &["collectionsBackdrop"], &[(960.0, 500.0)]);

    // Tools hub and sub-tools (Cores, Scraper, Packages, Library Manager, Themes).
    scene.pad(UiButton::PageDown);
    wait_for(&mut scene, "screens", "shown", "Tools", 3000);
    check_rhi(&mut scene, "Tools", Some("toolsCanvas"), &["toolsBackdrop"], &[(960.0, 500.0)]);

    click_named(&mut scene, "toolCard.0");
    wait_for(&mut scene, "screens", "shown", "Cores", 3000);
    check_rhi(&mut scene, "Cores", None, &[], &[(960.0, 500.0)]);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Tools", 3000);

    for (idx, tool) in [(1, "scraper"), (2, "packages"), (3, "library")] {
        click_named(&mut scene, &format!("toolCard.{idx}"));
        wait_for(&mut scene, "screens", "shown", "Tool", 3000);
        assert_eq!(scene.property("screens", "tool"), tool);
        check_rhi(&mut scene, tool, None, &[], &[(960.0, 500.0)]);
        scene.pad(UiButton::Back);
        wait_for(&mut scene, "screens", "shown", "Tools", 3000);
    }

    click_named(&mut scene, "toolCard.4");
    wait_for(&mut scene, "screens", "shown", "Themes", 3000);
    check_rhi(&mut scene, "Themes", None, &[], &[(960.0, 500.0)]);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Tools", 3000);

    // Settings and section sweep.
    click_named(&mut scene, "toolCard.5");
    wait_for(&mut scene, "screens", "shown", "Settings", 3000);
    check_rhi(&mut scene, "Settings", Some("settingsCanvas"), &["settingsBackdrop"], &[(1400.0, 500.0)]);
    for _ in 0..5 {
        scene.pad(UiButton::PageDown);
        scene.wait(80);
    }
    check_rhi(&mut scene, "Settings-s5", Some("settingsCanvas"), &["settingsBackdrop"], &[(1400.0, 500.0)]);

    scene.assert_quiet();
    drop(kwin);
}
