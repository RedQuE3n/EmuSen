// EmuSen_Kit.md §7.36: a press returns once the queued calls it caused have run, so an animation it starts takes the held
// clock's first step, however long other work the press queued keeps the GUI thread.

use emusen_kit_core::input::UiButton;
use emusen_qt_test::Harness;

const SCENE: &str = "qrc:/qt/qml/EmuSen/Scenes/qml/QueuedStartScene.qml";
/// The animation's value is interpolated in double; it is the held time to well within this.
const ROUNDING_MS: f64 = 0.0005;

#[test]
fn qt_7_36_an_animation_a_press_starts_takes_the_held_clock_s_first_step() {
    // A held clock is refused under a threaded render loop, which installs its own animation driver (EmuSen_Kit.md §7.36).
    std::env::set_var("QSG_RENDER_LOOP", "basic");
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 320, 200);
    scene.hold_clock(Some(0));
    scene.pad(UiButton::Accept);
    assert_eq!(scene.property("scene", "moving"), "true", "the press reached the scene");
    scene.hold_clock(Some(16));
    let value: f64 = scene.property("probe", "value").parse().unwrap();
    assert!((value - 16.0).abs() < ROUNDING_MS, "the animation at the held 16 ms reads {value}");
    scene.hold_clock(Some(48));
    let value: f64 = scene.property("probe", "value").parse().unwrap();
    assert!((value - 48.0).abs() < ROUNDING_MS, "the animation at the held 48 ms reads {value}");
    scene.hold_clock(None);
}
