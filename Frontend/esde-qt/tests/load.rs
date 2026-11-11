//! The view's load at a library's scale (EmuSen_Qt.md §4.15), through `EsdeModel`'s own load, offscreen: the GUI thread's
//! part of a load hands the four texts to the load's thread without turning them into Rust text. A timing test, run on
//! a quiet machine. One test, as the harness is one per process.

use emusen_qt_test::Harness;

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/Load.qml");

#[test]
fn the_gui_thread_s_part_of_a_load_stays_under_two_milliseconds_at_nine_thousand_games() {
    // The texts' sizes at 9,007 games (§4.15): the shelves 6.6 MB, the media index 2.2 MB.
    let row = r#"{"file":"/roms/snes/Game 0000.sfc","name":"Game 0000","esde_system":"snes","description":"A game invented for EmuSen's tests, long enough to wrap.","developer":"Studio A","publisher":"Publisher P","genre":"Puzzle","players":"1","release_date":"19900101T000000","favourite":false},"#;
    let shelves = format!("[{{\"name\":\"snes\",\"games\":[{}]}}]", row.repeat(9000).trim_end_matches(','));
    let media = format!("{{\"m\":\"{}\"}}", "m".repeat(2_200_000));
    assert!(shelves.len() > 2_000_000, "a library's scale");
    assert!(emusen_esde_qt::marks::mark_named("star").is_some(), "the module is linked into this test");
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 320, 200);
    assert!(scene.set_property("model", "media", &media));
    assert!(scene.set_property("model", "presence", "{}"));
    assert!(scene.set_property("model", "settings", "{\"theme\":\"t\"}"));
    let mut best = std::time::Duration::MAX;
    for round in 0..3 {
        // Each round a new shelves text, set without a wait, so the load it asks for runs in the timed turn of the loop.
        let text = shelves.replace("Game 0000\"", &format!("Game 000{round}\""));
        let revision = scene.property("model", "revision");
        assert!(scene.set_property_now("model", "shelves", &text));
        let started = std::time::Instant::now();
        scene.wait(0);
        best = best.min(started.elapsed());
        for _ in 0..1000 {
            if scene.property("model", "revision") != revision {
                break;
            }
            scene.wait(10);
        }
        assert_ne!(scene.property("model", "revision"), revision, "the load came back and was installed");
    }
    eprintln!("the GUI thread's part of a load: {best:?}");
    assert!(best < std::time::Duration::from_millis(2), "{best:?}");
}
