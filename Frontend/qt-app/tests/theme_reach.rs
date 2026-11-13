//! The chosen EmuSen theme's token overrides on the program's own screens (decided by the tester, 2026-11-13; EmuSen_Qt.md
//! §3.5.1): a focused row of Settings and of the Home card drawn in the theme's accent, in its light and dark variants, and
//! the kit's again for a theme with none and with none chosen; then Settings, the Home card, the List card, the game page,
//! and a tool held to the gate's fit and contrast rules under each in-repo theme that overrides a token (the Pause Card in
//! theme_reach_pause.rs); one test, as the harness is one per process.

mod common;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::audit::{self, contrast, fit, Rect};
use emusen_kit_core::input::{UiButton, WalkForm};
use emusen_kit_core::tokens::{Colour, Variant};
use emusen_qt::MAIN_URL;
use emusen_qt_test::harness::Measures;
use emusen_qt_test::Scene;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../theme-host/tests/themes");
const THEMES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../themes");
/// The in-repo themes whose theme.json overrides a token.
const OVERRIDING: [&str; 4] = ["constellation", "horizon", "meridian", "orrery"];

fn choose(scene: &mut Scene<'_>, folder: &str) {
    assert!(scene.set_property("program", "theme", folder));
    scene.wait(300);
}

fn focused(scene: &mut Scene<'_>) -> Option<(String, f64, f64, f64, f64)> {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| {
        let n = &tree.nodes[i as usize];
        (n.name.clone(), n.x, n.y, n.w, n.h)
    })
}

/// The colour drawn at the focused row's left edge, where a kit row draws its accent bar.
fn bar(scene: &mut Scene<'_>) -> (u8, u8, u8) {
    let (name, x, y, _, h) = focused(scene).expect("a focused row");
    let c = scene.colours_at(&[(x + 1.0, y + h / 2.0)])[0].unwrap_or_else(|| panic!("a colour at {name}'s bar"));
    (c.r, c.g, c.b)
}

fn rgb(hex: &str) -> (u8, u8, u8) {
    let c = Colour::parse(hex).unwrap();
    (c.r, c.g, c.b)
}

fn home(scene: &mut Scene<'_>) {
    for _ in 0..40 {
        if focused(scene).is_some_and(|f| f.0 == "place.0") {
            return;
        }
        scene.wait(50);
    }
    panic!("Home's first place focused: {:?}", focused(scene));
}

fn place(scene: &mut Scene<'_>, index: usize, shown: &str) {
    home(scene);
    for _ in 0..index {
        scene.pad(UiButton::Right);
    }
    scene.pad(UiButton::Accept);
    wait_for(scene, "screens", "shown", shown, 2000);
    scene.wait(300);
}

fn back_home(scene: &mut Scene<'_>) {
    for _ in 0..6 {
        if scene.property("screens", "shown") == "Home" {
            break;
        }
        scene.pad(UiButton::Back);
        scene.wait(200);
    }
    wait_for(scene, "screens", "shown", "Home", 2000);
}

fn load(harness: &mut emusen_qt_test::Harness) -> Scene<'_> {
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "settingsModel", "ready", "true", 5000);
    scene
}

#[test]
fn qt_3_5_1_the_chosen_theme_s_tokens_draw_the_program_s_screens_and_none_gives_the_kit_s() {
    let _world = sandbox("qt-app-theme-reach", &[("NES", "One.nes"), ("NES", "Two.nes")]);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = load(&mut harness);
    let kit = rgb("#4C9AE8");

    // Settings, opened under the kit's look, takes the fixture's accent once the theme is chosen, and gives it back for none.
    place(&mut scene, 5, "Settings");
    assert_eq!(bar(&mut scene), kit, "Settings' focused row in the kit's accent with no theme");
    choose(&mut scene, &format!("{FIXTURES}/accent"));
    assert_eq!(scene.property("chosenTheme", "status"), "Ready");
    assert_eq!(bar(&mut scene), rgb("#B4442A"), "Settings' focused row in the theme's accent");
    choose(&mut scene, &format!("{FIXTURES}/stub"));
    assert_eq!(bar(&mut scene), kit, "a theme with no overrides gives the kit's back");
    choose(&mut scene, &format!("{FIXTURES}/accent"));
    choose(&mut scene, "");
    assert_eq!(bar(&mut scene), kit, "no theme chosen gives the kit's back");

    // Meridian's two accents, each in its variant, on Settings and on the Home card.
    choose(&mut scene, &format!("{THEMES}/meridian"));
    assert_eq!(bar(&mut scene), rgb("#5FD0B5"), "Meridian's night accent");
    scene.set_variant(Variant::Light);
    assert_eq!(bar(&mut scene), rgb("#0B7A64"), "Meridian's day accent");
    scene.set_variant(Variant::Dark);
    back_home(&mut scene);
    choose(&mut scene, "");
    home(&mut scene);
    scene.pad(UiButton::Menu);
    wait_for(&mut scene, "screens", "shown", "HomeCard", 2000);
    scene.wait(500);
    assert_eq!(bar(&mut scene), kit, "the Home card's focused row in the kit's accent with no theme");
    choose(&mut scene, &format!("{THEMES}/meridian"));
    assert_eq!(bar(&mut scene), rgb("#5FD0B5"), "the Home card's focused row in Meridian's accent");
    scene.set_variant(Variant::Light);
    assert_eq!(bar(&mut scene), rgb("#0B7A64"), "the Home card's focused row in Meridian's day accent");
    scene.set_variant(Variant::Dark);
    choose(&mut scene, "");
    back_home(&mut scene);

    // Each program screen under each overriding theme's colours: the gate's fit and contrast as under the kit's.
    let mut report = Vec::new();

    place(&mut scene, 5, "Settings");
    under_each_theme(&mut scene, "Settings", &mut report);
    back_home(&mut scene);

    place(&mut scene, 3, "Tool");
    under_each_theme(&mut scene, "Tool", &mut report);
    back_home(&mut scene);

    home(&mut scene);
    scene.pad(UiButton::Menu);
    wait_for(&mut scene, "screens", "shown", "HomeCard", 2000);
    scene.wait(500);
    under_each_theme(&mut scene, "HomeCard", &mut report);
    back_home(&mut scene);

    place(&mut scene, 0, "Consoles");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);
    wait_for(&mut scene, "gridCells", "count", "2", 2000);
    scene.pad(UiButton::Options);
    wait_for(&mut scene, "screens", "shown", "List", 2000);
    scene.wait(500);
    under_each_theme(&mut scene, "ListCard", &mut report);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "screens", "shown", "Page", 2000);
    wait_for(&mut scene, "pageModel", "ready", "true", 10000);
    scene.wait(500);
    under_each_theme(&mut scene, "GamePage", &mut report);

    assert!(report.is_empty(), "findings under a theme's colours that the kit's do not have:\n{}", report.join("\n"));
}


/// The gate's fit and contrast findings of the window as shown, at 1280 by 800, each with the items it names.
fn judged(scene: &mut Scene<'_>) -> Vec<String> {
    let (w, h) = (1280.0, 800.0);
    scene.wait(200);
    let mut tree = scene.tree();
    scene.sample_behind(&mut tree, w, h);
    let measures = Measures::of(WalkForm::BigScreen, h);
    let smallest = measures.floor * audit::DESKTOP_CAPS * measures.unit;
    let mut found: Vec<String> = fit(&tree.root, Rect::new(0.0, 0.0, w, h), measures.slack, smallest)
        .into_iter()
        .chain(contrast(&tree.root))
        .map(|f| {
            let who: Vec<String> = f.items().into_iter().map(|i| tree.nodes[i.0 as usize].name.clone()).collect();
            format!("{f} ({})", who.join("; "))
        })
        .collect();
    found.sort();
    found
}

/// Each overriding theme's `theme.json` with one plain Home screen in place of its own, whose screens take the focus as they
/// load: only the theme's tokens reach the program's screen judged under it.
fn colours_only() -> Vec<(&'static str, String)> {
    let root = std::env::temp_dir().join("qt-app-theme-reach-packages");
    OVERRIDING
        .iter()
        .map(|theme| {
            let dir = root.join(theme);
            std::fs::create_dir_all(dir.join("screens")).unwrap();
            let mut manifest: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(format!("{THEMES}/{theme}/theme.json")).unwrap()).unwrap();
            manifest["screens"] = serde_json::json!(["home"]);
            std::fs::write(dir.join("theme.json"), manifest.to_string()).unwrap();
            std::fs::copy(format!("{FIXTURES}/accent/screens/Home.qml"), dir.join("screens/Home.qml")).unwrap();
            (*theme, dir.display().to_string())
        })
        .collect()
}

/// Each overriding theme over the screen shown, dark and light: what the gate finds that it does not under the kit's.
fn under_each_theme(scene: &mut Scene<'_>, screen: &str, report: &mut Vec<String>) {
    let held = focused(scene).expect("the screen's focus");
    for variant in [Variant::Dark, Variant::Light] {
        choose(scene, "");
        scene.set_variant(variant);
        let kit = judged(scene);
        for (theme, folder) in colours_only() {
            choose(scene, &folder);
            assert_eq!(scene.property("chosenTheme", "status"), "Ready", "{theme}");
            assert_eq!(focused(scene), Some(held.clone()), "{screen} keeps the focus under {theme}");

            for line in judged(scene).into_iter().filter(|l| !kit.contains(l)) {
                report.push(format!("{screen} {theme} {variant:?}: {line}"));
            }
        }
    }
    choose(scene, "");
    scene.set_variant(Variant::Dark);
}

