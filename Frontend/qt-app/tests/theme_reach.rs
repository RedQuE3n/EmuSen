//! The chosen EmuSen theme's token overrides on the program's own screens (decided by the tester, 2026-11-13; EmuSen_Qt.md
//! §3.5.1): a focused row of Settings and of the Home card drawn in the theme's accent, and
//! the kit's again for a theme with none and with none chosen; one test, as the harness is one per process. The in-repo themes
//! that overrode tokens were retired 2026-11-23, so the fixtures' themes stand in.

mod common;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_kit_core::tokens::Colour;
use emusen_qt::MAIN_URL;
use emusen_qt_test::Scene;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../theme-host/tests/themes");

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

    // The Home card's focused row takes the fixture's accent once the theme is chosen, and gives it back for none.
    back_home(&mut scene);
    home(&mut scene);
    scene.pad(UiButton::Menu);
    wait_for(&mut scene, "screens", "shown", "HomeCard", 2000);
    scene.wait(500);
    assert_eq!(bar(&mut scene), kit, "the Home card's focused row in the kit's accent with no theme");
    choose(&mut scene, &format!("{FIXTURES}/accent"));
    assert_eq!(bar(&mut scene), rgb("#B4442A"), "the Home card's focused row in the theme's accent");
    choose(&mut scene, "");
    back_home(&mut scene);
}
