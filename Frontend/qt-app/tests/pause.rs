//! The Pause Card through the harness on the software backend (EmuSen_Qt.md §7.1 step 7): its first page's rows and help
//! bar in Mistress's words, the slide in and out over the motion tokens and none under reduced motion, a page turn that
//! moves nothing, B and Start resuming, and Leave ▸ Back to Library. The real path, over a running game, is acceptance.rs's.

// The modules the scene imports register through the program's crate, which a test must link.
extern crate emusen_qt;

use emusen_kit_core::input::UiButton;

const SCENE: &str = r#"import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.App
Window {
    id: window
    objectName: "window"
    property bool reduced: false
    property int resumed: 0
    property int gone: 0
    property int library: 0
    Binding { target: Kit.Tokens; property: "reducedMotion"; value: window.reduced }
    Kit.NormalizedCanvas {
        PauseCard {
            id: card
            objectName: "card"
            anchors.fill: parent
            focus: true
            library: LibraryModel {}
            session: SessionModel { game: "/games/Cobalt Harbor (Synthetic).tst" }
            onResumed: { window.resumed += 1; card.open = false }
            onGone: window.gone += 1
            onBackToLibrary: window.library += 1
        }
    }
}
"#;

/// The card's rows as shown: each row's label and value words, in order.
fn rows(scene: &mut emusen_qt_test::Scene<'_>) -> Vec<String> {
    let tree = scene.tree();
    let words = |parent: &str, name: &str| {
        tree.nodes.iter().find(|n| n.parent == parent && n.name == name).and_then(|n| n.text.clone()).unwrap_or_default()
    };
    let mut rows: Vec<(f64, String)> = tree
        .nodes
        .iter()
        .filter(|n| n.visible && n.ty.starts_with("MenuRow"))
        .map(|n| (n.y, format!("{} | {}", words(&n.id, "label"), words(&n.id, "value"))))
        .collect();
    rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    rows.into_iter().map(|(_, r)| r).collect()
}

fn help(scene: &mut emusen_qt_test::Scene<'_>) -> Vec<String> {
    let tree = scene.tree();
    let mut entries: Vec<(f64, String)> =
        tree.nodes.iter().filter(|n| n.visible && n.name.starts_with("help-")).map(|n| (n.x, n.name[5..].to_string())).collect();
    entries.sort_by(|a, b| a.0.total_cmp(&b.0));
    entries.into_iter().map(|(_, e)| e).collect()
}

fn slide(scene: &mut emusen_qt_test::Scene<'_>) -> f64 {
    scene.property("card", "slide").parse().unwrap()
}

#[test]
fn the_card_slides_in_shows_mistresss_first_page_and_resumes() {
    let dir = std::env::temp_dir().join("qt-app-pause");
    std::fs::create_dir_all(&dir).unwrap();
    let qml = dir.join("card.qml");
    std::fs::write(&qml, SCENE).unwrap();
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(&format!("file://{}", qml.display()), 1280, 800);
    assert_eq!(slide(&mut scene), 1.0, "closed, the card is out of place");

    // Opened, it slides in over Enter (200 ms): part of the way at 60 ms, in place by 400.
    assert!(scene.set_property("card", "open", "true"));
    scene.wait(60);
    let part = slide(&mut scene);
    assert!(part > 0.0 && part < 1.0, "part of the way in at 60 ms: {part}");
    scene.wait(340);
    assert_eq!(slide(&mut scene), 0.0);
    assert_eq!(scene.property("pauseTitle", "text"), "Cobalt Harbor (Synthetic)");
    assert_eq!(
        rows(&mut scene),
        [
            "Resume | ",
            "Rewind | ",
            "Save State | Slot 1 (Empty)",
            "Load State | Slot 1 (Empty)",
            "Speed | 100%",
            "Game Options | ",
            "Leave | "
        ]
    );
    assert_eq!(help(&mut scene), ["Select", "Resume", "Screenshot", "Resume"]);
    assert_eq!(scene.property("PadMenuList", "currentIndex"), "0", "the card opens on Resume");
    scene.assert_still("the Pause Card in place");

    // Leave turns the page: no motion, its one row, the game as the way back; B returns to Leave's row.
    for _ in 0..6 {
        scene.pad(UiButton::Down);
    }
    scene.pad(UiButton::Accept);
    scene.wait(20);
    assert_eq!(slide(&mut scene), 0.0, "a page turn moves nothing");
    assert_eq!(rows(&mut scene), ["Back to Library | "]);
    assert_eq!((scene.property("pauseTitle", "text"), scene.property("pauseBreadcrumb", "text")), ("Leave".into(), "Cobalt Harbor (Synthetic)".into()));
    assert_eq!(help(&mut scene), ["Choose", "Back", "Screenshot", "Resume"]);
    scene.pad(UiButton::Back);
    scene.wait(20);
    assert_eq!(rows(&mut scene).len(), 7);
    assert_eq!(scene.property("PadMenuList", "currentIndex"), "6", "back on the row that opened the page");
    assert_eq!(scene.property("window", "resumed"), "0");
    scene.pad(UiButton::Accept);
    scene.pad(UiButton::Accept);
    assert_eq!(scene.property("window", "library"), "1", "Back to Library");

    // B on the first page resumes; the card slides out over Exit and says it is gone once out.
    scene.pad(UiButton::Back);
    scene.pad(UiButton::Back);
    assert_eq!(scene.property("window", "resumed"), "1");
    assert_eq!(scene.property("window", "gone"), "0", "not gone while it slides");
    scene.wait(400);
    assert_eq!((slide(&mut scene), scene.property("window", "gone")), (1.0, "1".into()));

    // Start's declared action resumes too, and under reduced motion the card comes and goes at once.
    assert!(scene.set_property("window", "reduced", "true"));
    assert!(scene.set_property("card", "open", "true"));
    scene.wait(10);
    assert_eq!(slide(&mut scene), 0.0, "in place at once under reduced motion");
    scene.pad(UiButton::Menu);
    scene.wait(10);
    assert_eq!((scene.property("window", "resumed"), scene.property("window", "gone")), ("2".into(), "2".into()), "Start resumes, gone at once");
    assert_eq!(slide(&mut scene), 1.0);
    scene.assert_quiet();
}
