//! The Pause Card's Game Options ▸ Controllers (EmuSen_Interface.md §20.3, EmuSen_Qt.md §7.7.H): a page of Settings'
//! one description drawn on the card, over a scripted world and an SDL virtual pad, held at the gate's sizes and
//! recorded for the comparison.

extern crate emusen_qt;

mod bindings;
mod common;

use bindings::nothing_named;
use common::{hold, sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_qt_test::mockup;
use emusen_qt_test::virtual_pad::Sdl;

const CARD: &str = r#"import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.Endymion
import EmuSen.App
Window {
    id: window
    objectName: "window"
    property string library
    property string pad
    Kit.NormalizedCanvas {
        PadReader {
            id: reader
            objectName: "pads"
            platformLibrary: window.library
            onlyPads: window.pad
            onSeatsChanged: settings.seatsChanged()
        }
        SettingsModel {
            id: settings
            objectName: "settingsModel"
            platformLibrary: window.library
            open: true
            facts: '{"game": "/games/Hero.sfc", "system": "snes"}'
            gamePads: reader.gamePads
        }
        PauseCard {
            id: card
            objectName: "card"
            anchors.fill: parent
            focus: true
            library: LibraryModel {}
            settings: settings
            session: SessionModel { objectName: "session"; game: "/games/Hero.sfc" }
        }
    }
}
"#;

#[test]
fn qt_7_7_h_the_pause_cards_controllers_page_seats_the_pad_and_names_the_profile() {
    let _world = sandbox("qt-app-card-controllers", &[("snes", "Hero.sfc")]);
    let sdl = Sdl::open();
    let pad = sdl.attach("Simulated pad", 0x045e, 0x028e);
    let dir = std::env::temp_dir().join("qt-app-card-controllers");
    std::fs::create_dir_all(&dir).unwrap();
    let qml = dir.join("card.qml");
    std::fs::write(&qml, CARD).unwrap();
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(&format!("file://{}", qml.display()), 1280, 800);
    assert!(scene.set_property("settingsModel", "world", concat!(env!("CARGO_MANIFEST_DIR"), "/tests/settings_controllers.json")));
    assert!(scene.set_property("window", "pad", &pad.id.to_string()));
    assert!(scene.set_property("window", "library", library().to_str().unwrap()));
    wait_for(&mut scene, "settingsModel", "ready", "true", 5000);
    wait_for(&mut scene, "pads", "connected", "1", 5000);
    assert!(scene.set_property("card", "open", "true"));
    scene.wait(400);
    // Game Options, the sixth row with no disc to change and no second player, then Controllers.
    for _ in 0..5 {
        scene.pad(UiButton::Down);
    }
    wait_for(&mut scene, "pause", "current", "5", 1000);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "pause", "title", "Game Options", 1000);
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "pause", "title", "Controllers", 1000);
    let mut waited = 0;
    while !scene.property("PaneSentence", "text").starts_with("Player 1 plays on the Simulated pad") && waited < 3000 {
        scene.wait(50);
        waited += 50;
    }
    assert_eq!(scene.property("PaneSentence", "text"), "Player 1 plays on the Simulated pad, with the Default profile.");
    assert_eq!(scene.property("PaneMuted", "text"), "Left and Right pick another connected controller; Bindings opens the drawing for this console.");
    let mut report = Vec::new();
    hold(&mut scene, "PauseCardControllers", &mockup::Mockup::default(), nothing_named, &mut report);
    // Right on Player 2 seats the pad there, trading with player 1.
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Right);
    let mut waited = 0;
    while !scene.property("PaneSentence", "text").starts_with("Player 2 plays on the Simulated pad") && waited < 3000 {
        scene.wait(50);
        waited += 50;
    }
    assert!(scene.property("PaneSentence", "text").starts_with("Player 2 plays on the Simulated pad"), "{}", scene.property("PaneSentence", "text"));
    sdl.detach(pad);
    assert!(report.is_empty(), "{}", report.join("\n"));
}
