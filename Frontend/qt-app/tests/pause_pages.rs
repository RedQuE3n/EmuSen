//! The Pause Card's pages through the harness on the software backend (EmuSen_Qt.md §7.3): the context pane on Rewind,
//! Save and Load from the session's facts, Left and Right choosing the slot, and the reel's page with its strip, its
//! steps and B back to Rewind. The pictures are judged on Vulkan, over a running game, in acceptance.rs.

extern crate emusen_qt;

use emusen_kit_core::input::UiButton;
use emusen_qt_models::pause::{Facts, SlotFact};

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
            session: SessionModel { objectName: "session"; game: "/games/Cobalt Harbor (Synthetic).tst" }
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

/// The pane's tiles as shown: each one's words under its picture, left to right.
fn tiles(scene: &mut emusen_qt_test::Scene<'_>) -> Vec<String> {
    let tree = scene.tree();
    let said = |parent: &str| -> Vec<(f64, String)> {
        tree.nodes
            .iter()
            .filter(|t| t.parent == parent && t.visible)
            .filter_map(|t| t.text.clone().filter(|w| !w.is_empty()).map(|w| (t.y, w)))
            .collect()
    };
    let mut shown: Vec<(f64, String)> = tree
        .nodes
        .iter()
        .filter(|n| n.visible && n.name.starts_with("PaneSlot") && n.name != "PaneSlots")
        .map(|n| {
            let mut under = said(&n.id);
            under.extend(said(&n.parent));
            (n.x, under.into_iter().map(|(_, w)| w).collect::<Vec<_>>().join(" / "))
        })
        .collect();
    shown.sort_by(|a, b| a.0.total_cmp(&b.0));
    shown.into_iter().map(|(_, t)| t).collect()
}

fn facts() -> String {
    let held: Vec<u64> = (0..=56).map(|i| 1000 + i * 30).collect();
    let slots = vec![SlotFact { slot: 2, exists: true, when: "Sun 8 Nov, 21:42".into(), picture: String::new() }];
    Facts { held, now: 2690, rewind_seconds: 28.0, slots, ..Facts::default() }.to_json()
}

#[test]
fn the_pane_follows_the_row_and_the_reel_turns_in_place() {
    let dir = std::env::temp_dir().join("qt-app-pause-pages");
    std::fs::create_dir_all(&dir).unwrap();
    let qml = dir.join("card.qml");
    std::fs::write(&qml, SCENE).unwrap();
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(&format!("file://{}", qml.display()), 1280, 800);
    assert!(scene.set_property("card", "open", "true"));
    assert!(scene.set_property("pause", "facts", &facts()));
    scene.wait(400);
    assert_eq!(scene.property("AdaptiveCardPane", "visible"), "false", "Resume shows no pane");

    scene.pad(UiButton::Down);
    scene.wait(20);
    assert_eq!(scene.property("AdaptiveCardPane", "visible"), "true");
    assert_eq!(tiles(&mut scene), ["1 s ago", "0.5 s ago", "Now"], "the reel's three newest");
    assert_eq!(scene.property("PaneSentence", "text"), "28 s available. A opens the reel; Left and Right step through it.");

    scene.pad(UiButton::Down);
    scene.wait(20);
    assert_eq!(tiles(&mut scene), ["Empty / Slot 1 / Empty", "Empty / Slot 2 / Sun 8 Nov, 21:42", "Empty / Slot 3 / Empty"], "a slot whose picture cannot be had is drawn empty, as Mistress draws it");
    assert_eq!(scene.property("PaneSentence", "text"), "A saves to Slot 1.");
    assert_eq!(help(&mut scene), ["Select", "Resume", "Change slot", "Screenshot", "Resume"]);
    scene.pad(UiButton::Right);
    scene.wait(20);
    assert_eq!(scene.property("session", "slot"), "2", "Right chooses the next slot through the session");
    scene.pad(UiButton::Down);
    scene.wait(20);
    assert_eq!(scene.property("PaneSentence", "text"), "Slot 1 is empty.", "Load on an empty slot");
    scene.pad(UiButton::Down);
    scene.wait(20);
    assert_eq!(scene.property("AdaptiveCardPane", "visible"), "false", "Speed shows no pane");
    scene.pad(UiButton::Right);
    scene.wait(20);
    assert_eq!(scene.property("session", "speed"), "300", "Right on Speed chooses turbo through the session");
    scene.pad(UiButton::Left);
    scene.pad(UiButton::Left);
    scene.wait(20);
    assert_eq!(scene.property("session", "speed"), "25", "Left twice slow motion");

    // Rewind's reel: a page turned in place, on the stop nearest 1.5 s back, the strip of six with it ringed.
    scene.pad(UiButton::Up);
    scene.pad(UiButton::Up);
    scene.pad(UiButton::Up);
    scene.pad(UiButton::Accept);
    scene.wait(20);
    assert_eq!(scene.property("card", "slide"), "0", "a page turn moves nothing");
    assert_eq!(rows(&mut scene), ["Go back to 1.7 s ago | ", "Moment | 1.7 s ago", "Hold ◂ to scrub | "]);
    assert_eq!((scene.property("cardTitle", "text"), scene.property("cardBreadcrumb", "text")), ("Rewind".into(), "Cobalt Harbor (Synthetic)".into()));
    assert_eq!(tiles(&mut scene), ["Now", "1.7 s", "4.2 s", "8.2 s", "12 s", "28 s"]);
    assert_eq!(help(&mut scene), ["Go back", "Back", "Moment", "Screenshot"]);
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Right);
    scene.wait(20);
    assert_eq!(rows(&mut scene)[1], "Moment | 4.2 s ago", "Right goes to the next stop back");
    assert_eq!(scene.property("PaneSentence", "text"), "4.2 s ago. A goes back there; the moments after it are kept until you play on.");
    scene.pad(UiButton::Back);
    scene.wait(20);
    assert_eq!(rows(&mut scene).len(), 7);
    assert_eq!(scene.property("PadMenuList", "currentIndex"), "1", "B goes back to Rewind's row");
    assert_eq!(scene.property("window", "resumed"), "0");
    scene.assert_quiet();
}
