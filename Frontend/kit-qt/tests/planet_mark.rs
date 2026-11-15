// EmuSen_Kit.md §7.115: the kit's PlanetMark draws the mark its one owner, kit-core's planet_mark, gives a core, and each
// kind of core has the mark this test states: the sky-named cores, a shards and a gem console's core, an invented core on
// a planet's body, and a hosted libretro core; one test, as the harness is one per process.

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

/// A hosted core's name as the catalogue of hosted cores writes it: the first field of its first line.
fn hosted_core() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../Platform/systems/data/libretro-cores.tsv");
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let line = text.lines().find(|l| !l.starts_with('#') && !l.trim().is_empty()).expect("a hosted core");
    line.split('\t').next().expect("its name").to_owned()
}

#[test]
fn kit_7_115_a_planet_mark_draws_the_mark_its_owner_gives_a_core() {
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 200, 200);
    // Each kind of core and the mark this test states for it: a core named for the Moon or a planet its own symbol, for
    // every name of kit-core's SKY_NAMED; a core on a console drawn as shards or as a gem that body's glyph, the body read
    // from the console's own data (Platform/systems/data, the systems table's `look.body`, the words of
    // Platform/systems/src/lib.rs `BODIES`); an invented name on a planet-bodied console the plain planet.
    let mut kinds: Vec<(String, String, &str)> = planet_mark::SKY_NAMED.iter().map(|n| (n.to_string(), body_of("snes"), *n)).collect();
    kinds.push(("Nephrite".into(), body_of("md"), "shard"));
    kinds.push(("Diamond".into(), body_of("psx"), "gem"));
    kinds.push(("an invented core on a planet's body".into(), body_of("snes"), "planet"));
    let mut drawn = Vec::new();
    for (engine, body, expected) in &kinds {
        let mark = planet_mark::of(engine, false, body);
        assert_eq!(&mark, expected, "{engine} on a body of {body}");
        assert!(scene.set_property("mark", "mark", &mark));
        let path = scene.property("planetMarkPath", "path");
        assert!(!path.is_empty() && path == planet_mark::path(&mark), "{engine}: drew {path:?} for {mark}");
        assert_eq!(scene.property("planetMarkGlyph", "visible"), "true", "{engine}'s glyph is shown");
        assert_eq!(scene.property("planetMarkBadge", "visible"), "false", "{engine} has no libretro badge");
        drawn.push(path);
    }
    drawn.sort();
    drawn.dedup();
    assert_eq!(drawn.len(), kinds.len(), "every kind its own mark: {drawn:?}");
    let core = hosted_core();
    let hosted = planet_mark::of(&core, true, &body_of("md"));
    assert_eq!(hosted, "libretro", "{core}, a hosted core");
    assert!(scene.set_property("mark", "mark", &hosted));
    assert_eq!(scene.property("planetMarkBadge", "visible"), "true", "a hosted core shows libretro's badge");
    assert_eq!(scene.property("planetMarkGlyph", "visible"), "false", "and no glyph");
    assert_eq!(scene.property("planetMarkBadgePath", "path"), planet_mark::BADGE);
    assert_eq!(scene.property("planetMarkAlienPath", "path"), planet_mark::alien());
    scene.assert_quiet();
}
