// EmuSen_Kit.md §7.117: the face tokens. The kit's faces are its fonts' own families and each is drawn as itself; the reference's
// roles are Barlow, Barlow Condensed and the application's font; a theme names a bundled face for a role and nothing else.

use emusen_kit_core::tokens::{BUNDLED, FaceRole, Variant, WeightRole};
use emusen_kit_qt::tokens::{bundled_families, platform_family};
use emusen_qt_test::Harness;

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/FacesScene.qml");

fn read(scene: &mut emusen_qt_test::Scene<'_>, name: &str) -> String {
    assert!(scene.set_property("scene", "asked", name));
    scene.property("scene", "answer")
}

/// The family a text of the scene asks for and the family Qt draws it in.
fn drawn(scene: &mut emusen_qt_test::Scene<'_>, name: &str) -> (String, String) {
    let tree = scene.tree();
    let node = tree.nodes.iter().find(|n| n.name == name).unwrap_or_else(|| panic!("{name} is drawn"));
    (node.face.clone().unwrap_or_default(), node.drawn.clone().unwrap_or_default())
}

#[test]
fn kit_7_117_the_faces_are_the_fonts_own_and_a_theme_names_one_per_role() {
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 320, 240);

    // The kit's list is the families its font files declare, every file read; each family is drawn as itself, no fallback.
    let mut loaded = bundled_families();
    loaded.sort();
    let mut listed: Vec<String> = BUNDLED.iter().map(|f| (*f).to_owned()).collect();
    listed.sort();
    assert_eq!(loaded, listed, "kit-core's BUNDLED is the families of kit-qt/fonts/");
    let files = std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/fonts")).unwrap().filter(|e| e.as_ref().is_ok_and(|e| e.path().extension().is_some_and(|x| x == "ttf"))).count();
    let qrc = include_str!("../fonts/Fonts.qrc");
    assert_eq!(qrc.matches(".ttf</file>").count(), files, "every font file in the folder is in the resource");
    assert!(scene.set_property("scene", "separator", "|"));
    assert!(scene.set_property("scene", "families", &BUNDLED.join("|")));
    for (i, family) in BUNDLED.iter().enumerate() {
        assert_eq!(drawn(&mut scene, &format!("bundled.{i}")), ((*family).to_owned(), (*family).to_owned()), "{family} is drawn as itself");
    }

    // The reference's roles: Barlow, Barlow Condensed, and the application's font for the rows and the codes.
    let platform = platform_family();
    assert!(!platform.is_empty());
    let kit = |role: FaceRole| role.kit().map_or(platform.clone(), str::to_owned);
    for role in FaceRole::ALL {
        assert_eq!(read(&mut scene, role.property()), kit(role), "{role:?} is the kit's");
    }
    assert_eq!(read(&mut scene, "faceText"), "Barlow");
    assert_eq!(read(&mut scene, "faceLabel"), "Barlow Condensed");
    assert_eq!(read(&mut scene, "faceCode"), platform);
    for role in WeightRole::ALL {
        assert_eq!(read(&mut scene, role.property()), role.kit().to_string(), "{role:?}'s weight is the kit's");
    }
    assert_eq!((read(&mut scene, "weightTitle"), read(&mut scene, "weightWordmark")), ("400".into(), "300".into()), "regular titles, the light wordmark");

    // A theme's faces in its variant; a family the kit does not bundle, and another variant's, leave the kit's.
    let pair = |v: Variant, n: &str, f: &str| (v, n.to_owned(), f.to_owned());
    scene.kit_tokens().expect("the kit's tokens").set_overrides(vec![
        pair(Variant::Dark, "faceText", "Sora"),
        pair(Variant::Dark, "faceInterface", "Sora"),
        pair(Variant::Dark, "faceCode", "JetBrains Mono"),
        pair(Variant::Dark, "faceLabel", "Comic Sans MS"),
        pair(Variant::Light, "faceLabel", "Sora"),
        pair(Variant::Dark, "weightTitle", "600"),
        pair(Variant::Dark, "weightSentence", "300"),
    ]);
    assert_eq!((read(&mut scene, "weightTitle"), read(&mut scene, "weightSentence")), ("600".into(), "300".into()), "a theme's weights");
    assert_eq!(drawn(&mut scene, "faceText"), ("Sora".into(), "Sora".into()));
    assert_eq!(drawn(&mut scene, "faceInterface"), ("Sora".into(), "Sora".into()));
    assert_eq!(drawn(&mut scene, "faceCode"), ("JetBrains Mono".into(), "JetBrains Mono".into()));
    assert_eq!(drawn(&mut scene, "faceLabel"), ("Barlow Condensed".into(), "Barlow Condensed".into()), "an unbundled family is not read");

    // None set: the kit's own again.
    scene.kit_tokens().expect("the kit's tokens").set_overrides(Vec::new());
    for role in FaceRole::ALL {
        assert_eq!(read(&mut scene, role.property()), kit(role), "{role:?} is the kit's again");
    }
    assert_eq!(read(&mut scene, "weightTitle"), "400", "the title's weight is the kit's again");
}
