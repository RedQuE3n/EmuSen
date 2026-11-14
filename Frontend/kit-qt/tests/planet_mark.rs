// EmuSen_Kit.md §7.115: the kit's PlanetMark draws the mark its one owner, kit-core's planet_mark, gives a core: three
// cores of the house and a hosted libretro core; one test, as the harness is one per process.

use emusen_kit_core::planet_mark;
use emusen_qt_test::Harness;

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/PlanetMarkScene.qml");

/// A console's body as the systems table, its owner, writes it.
fn body_of(console: &str) -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../Platform/systems/data/").to_owned() + console + ".json";
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let at = text.find("\"body\"").unwrap_or_else(|| panic!("{path} has a body"));
    text[at..].split('"').nth(3).expect("the body's word").to_owned()
}

#[test]
fn kit_7_115_a_planet_mark_draws_the_mark_its_owner_gives_a_core() {
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 200, 200);
    // Three cores of the house, each on its console's body, and a hosted core: the engines' names are test inputs.
    let cores = [("Venus", "snes"), ("Diamond", "psx"), ("Nephrite", "md")];
    let mut drawn = Vec::new();
    for (engine, console) in cores {
        let mark = planet_mark::of(engine, false, &body_of(console));
        assert!(scene.set_property("mark", "mark", &mark));
        let path = scene.property("planetMarkPath", "path");
        assert!(!path.is_empty() && path == planet_mark::path(&mark), "{engine}: drew {path:?} for {mark}");
        assert_eq!(scene.property("planetMarkGlyph", "visible"), "true", "{engine}'s glyph is shown");
        assert_eq!(scene.property("planetMarkBadge", "visible"), "false", "{engine} has no libretro badge");
        drawn.push(path);
    }
    drawn.dedup();
    assert_eq!(drawn.len(), cores.len(), "three cores, three marks: {drawn:?}");
    let hosted = planet_mark::of("a hosted core", true, &body_of("md"));
    assert!(scene.set_property("mark", "mark", &hosted));
    assert_eq!(scene.property("planetMarkBadge", "visible"), "true", "a hosted core shows libretro's badge");
    assert_eq!(scene.property("planetMarkGlyph", "visible"), "false", "and no glyph");
    assert_eq!(scene.property("planetMarkBadgePath", "path"), planet_mark::BADGE);
    assert_eq!(scene.property("planetMarkAlienPath", "path"), planet_mark::alien());
    scene.assert_quiet();
}
