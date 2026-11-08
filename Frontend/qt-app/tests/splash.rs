//! The program's splash against Mistress's own (EmuSen_Kit.md §7.18), in a process of its own, as each harness wants.

mod common;

use common::{sandbox, wait_for};
use emusen_kit_core::input::UiButton;
use emusen_qt::MAIN_URL;
use emusen_qt_models::words::splash_caption;
use emusen_qt_test::Harness;


/// The program's finished splash against Mistress's own with the canvas's eleven consoles (EmuSen_Kit.md §7.18): each
/// console's world from this commit's systems table, in its order, whatever engines the build has. The test holds the
/// splash up, as the C# picture holds it with start-up work never done.
#[test]
fn the_finished_splash_is_mistress_s_with_each_console_s_body() {
    let _world = sandbox("qt-app-splash", &[("NES", "Synthetic.nes")]);
    let worlds = common::canvas_worlds();
    let mut harness = Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 720);
    assert!(scene.set_property("moonrise", "opacity", "1"));
    assert!(scene.set_property("moonrise", "worlds", &worlds.join("\n")));
    assert!(scene.set_property("moonrise", "caption", &splash_caption(worlds.len())));
    assert!(scene.set_property("moonrise", "running", "true"));
    scene.wait(100);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "moonrise", "finished", "true", 1000);
    scene.wait(200);
    // The loose box round the caption lets a digit through, so its words are held as the C# picture shows them.
    assert_eq!(scene.property("moonrise", "caption"), "11 CONSOLES");
    let tree = scene.tree();
    let grabs = std::env::temp_dir();
    assert!(scene.grab(&grabs.join("qt-app-splash.png").display().to_string()));
    let marks = grabs.join("qt-app-splash-difference.png").display().to_string();
    let faults = emusen_qt_test::moonrise::against(&mut scene, &tree, emusen_qt_test::moonrise::BODIES_REFERENCE, emusen_qt_test::moonrise::BODIES_SPARE, &marks);
    assert!(faults.is_empty(), "{}", faults.join("\n"));
    scene.assert_quiet();
}
