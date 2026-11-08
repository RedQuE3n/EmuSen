// EmuSen_Kit.md §7.15: SDL's virtual pads, read by Endymion through the gallery's PadReader, walk the gallery to the
// landings the router's queue gives, the help bar shows the family of the pad last used, and pads plugged in and pulled
// out while it runs are picked up. SDL's own virtual joysticks, as SdlVirtualPadsTests attaches them; no pad on the desk
// is opened, since the reader is shown these alone.

use std::time::Instant;

use emusen_client_fixture::{library, World};
use emusen_kit_core::input::UiButton;
use emusen_kit_gallery::GALLERY_URL;
use emusen_qt_endymion::reading::sdl_button;
use emusen_qt_test::virtual_pad::{Sdl, VirtualPad};
use emusen_qt_test::{Harness, Scene};

/// Waits up to two seconds for a condition the scene answers, letting Qt run between looks.
fn until(scene: &mut Scene<'_>, what: &str, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    let start = Instant::now();
    while !done(scene) {
        assert!(start.elapsed().as_secs_f64() < 2.0, "{what}: not within two seconds");
        scene.wait(8);
    }
}

/// A press on the virtual pad as a player makes one: down until the router holds it, then up until it lets go.
fn press(sdl: &Sdl, pad: &VirtualPad, scene: &mut Scene<'_>, button: UiButton) {
    let sdl_number = sdl_button(button.control(false)).expect("a direction is a button");
    sdl.press(pad, sdl_number, true);
    until(scene, "the router holds the press", |s| s.pad_held(button));
    sdl.press(pad, sdl_number, false);
    until(scene, "the router lets go", |s| !s.pad_held(button));
}

#[test]
fn a_virtual_pad_walks_the_gallery_as_the_router_s_queue_does_and_the_help_bar_follows_the_pad_last_used() {
    let _world = World::new("gallery-pads");
    let sdl = Sdl::open();
    let xbox = sdl.attach("EmuSen Test Xbox Pad", 0x045e, 0x028e);
    let mut harness = Harness::new();
    let mut scene = harness.load(GALLERY_URL, 1280, 800);
    assert_eq!(scene.property("scene", "padFamily"), "0", "the generic face before any pad is read");

    assert!(scene.set_property("scene", "onlyPads", &xbox.id.to_string()));
    assert!(scene.set_property("scene", "platformLibrary", library().to_str().unwrap()));
    until(&mut scene, "the virtual pad read", |s| s.property("scene", "padsConnected") == "1");
    assert_eq!(scene.property("scene", "padFamily"), "1", "the Xbox pad's family on the help bar");

    // The walk by the virtual pad, then the same walk by the router's queue, as the gate makes it.
    scene.run_pad_clock();
    let tree = scene.tree();
    let by_pad = scene.walk_by(&tree, |scene, button| press(&sdl, &xbox, scene, button));
    let by_queue = scene.walk(&tree);
    assert!(by_pad.len() >= 16, "the walk reached {} presses", by_pad.len());
    assert_eq!(by_pad, by_queue, "the virtual pad's landings are the queue's");
    scene.run_pad_clock();

    // A second pad plugged in while the gallery runs, used, then pulled out: the family follows the pad last used.
    let sony = sdl.attach("EmuSen Test DualShock Pad", 0x054c, 0x09cc);
    assert!(scene.set_property("scene", "onlyPads", &format!("{} {}", xbox.id, sony.id)));
    until(&mut scene, "the second pad read", |s| s.property("scene", "padsConnected") == "2");
    assert_eq!(scene.property("scene", "padFamily"), "1", "the Xbox pad was the last used");
    let random = sdl_button(emusen_kit_core::input::PadControl::RightStick).unwrap();
    sdl.press(&sony, random, true);
    until(&mut scene, "the PlayStation pad's family", |s| s.property("scene", "padFamily") == "2");
    sdl.press(&sony, random, false);
    sdl.detach(sony);
    until(&mut scene, "the second pad gone", |s| s.property("scene", "padsConnected") == "1");
    assert_eq!(scene.property("scene", "padFamily"), "1", "back to the pad still read");
    sdl.detach(xbox);
    until(&mut scene, "no pad read", |s| s.property("scene", "padsConnected") == "0");
    assert_eq!(scene.property("scene", "padFamily"), "0", "the generic face with no pad");
    assert_eq!(scene.property("pads", "problem"), "", "no failure said");
    scene.assert_quiet();
}
