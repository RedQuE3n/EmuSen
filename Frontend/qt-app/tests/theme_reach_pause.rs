//! The Pause Card held to the gate's fit and contrast rules under each in-repo theme that overrides a token, dark and light,
//! the theme's overrides set on its scene's `Tokens` as the program's ThemeStyle sets them (EmuSen_Qt.md §3.5.1).

// The modules the scene imports register through the program's crate, which a test must link.
extern crate emusen_qt;

use emusen_kit_core::audit::{self, contrast, fit, Rect};
use emusen_kit_core::input::WalkForm;
use emusen_kit_core::tokens::Variant;
use emusen_qt_test::harness::Measures;
use emusen_qt_test::Scene;

const THEMES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../themes");
const OVERRIDING: [&str; 4] = ["constellation", "horizon", "meridian", "orrery"];

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

const PAUSE: &str = r#"import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.App
Window {
    Kit.NormalizedCanvas {
        PauseCard {
            objectName: "card"
            anchors.fill: parent
            focus: true
            library: LibraryModel {}
            session: SessionModel { game: "/games/Cobalt Harbor (Synthetic).tst" }
        }
    }
}
"#;

#[test]
fn qt_3_5_1_the_pause_card_keeps_its_fit_and_contrast_under_each_theme_s_colours() {
    let dir = std::env::temp_dir().join("qt-app-theme-reach-pause");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("Pause.qml");
    std::fs::write(&file, PAUSE).unwrap();
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(&format!("file://{}", file.display()), 1280, 800);
    assert!(scene.set_property("card", "open", "true"));
    scene.wait(600);
    assert_eq!(scene.property("card", "slide"), "0", "the card in place");
    assert_eq!(scene.property("cardTitle", "text"), "Cobalt Harbor (Synthetic)");
    let mut report = Vec::new();
    for variant in [Variant::Dark, Variant::Light] {
        scene.set_variant(variant);
        scene.kit_tokens().expect("the kit's tokens").set_overrides(Vec::new());
        let kit = judged(&mut scene);
        for theme in OVERRIDING {
            let text = std::fs::read_to_string(format!("{THEMES}/{theme}/theme.json")).unwrap();
            let package = emusen_theme_host::package::read(&text).unwrap();
            // The table as the program's ThemeStyle sets it, through the kit's Tokens of this scene's engine.
            let table = package
                .overrides()
                .into_iter()
                .map(|(light, name, value)| (if light { Variant::Light } else { Variant::Dark }, name, value))
                .collect();
            scene.kit_tokens().expect("the kit's tokens").set_overrides(table);
            scene.wait(100);
            for line in judged(&mut scene).into_iter().filter(|l| !kit.contains(l)) {
                report.push(format!("PauseCard {theme} {variant:?}: {line}"));
            }
        }
    }
    assert!(report.is_empty(), "findings under a theme's colours that the kit's do not have:\n{}", report.join("\n"));
}
