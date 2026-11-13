// EmuSen_Kit.md §7.2, EmuSen_Qt.md §3.5.1: a theme's override of any number, whole number or switch token of the registry's
// Tokens reaches QML in its variant, as the colours' do; one test, as the harness is one per process.

use emusen_kit_core::tokens::Variant;
use emusen_qt_test::Harness;

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/TokensScene.qml");
const REGISTRY: &str = include_str!("../../../Testing/qmltypes/EmuSen.Kit-1.3.qmltypes");

/// The Tokens component's read-only properties with their registry types: the tokens a theme may override.
fn registry_tokens() -> Vec<(String, String)> {
    let at = REGISTRY.find("name: \"Tokens\"").expect("Tokens in the registry");
    let component = &REGISTRY[at..];
    let component = &component[..component.find("Component {").unwrap_or(component.len())];
    let mut out = Vec::new();
    for block in component.split("Property {").skip(1) {
        let block = &block[..block.find('}').unwrap_or(block.len())];
        let field = |key: &str| block.lines().map(str::trim).find_map(|l| l.strip_prefix(key)).map(|v| v.trim_matches('"').to_owned());
        if field("isReadonly: ").as_deref() == Some("true") {
            out.push((field("name: ").unwrap_or_default(), field("type: ").unwrap_or_default()));
        }
    }
    out
}

fn read(scene: &mut emusen_qt_test::Scene<'_>, name: &str) -> String {
    assert!(scene.set_property("scene", "asked", name));
    scene.property("scene", "answer")
}

#[test]
fn kit_7_2_every_number_and_switch_token_takes_a_theme_s_override_in_its_variant() {
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 320, 240);
    let tokens: Vec<(String, String)> =
        registry_tokens().into_iter().filter(|(_, t)| ["double", "int", "bool"].contains(&t.as_str())).collect();
    assert!(tokens.len() > 50, "the registry's number and switch tokens: {tokens:?}");
    let kit: Vec<String> = tokens.iter().map(|(name, _)| read(&mut scene, name)).collect();
    let given = |t: &str, kit: &str| match t {
        "double" => "7.25".to_owned(),
        "int" => "7".to_owned(),
        _ => (kit != "true").to_string(),
    };
    let mut overrides: Vec<(Variant, String, String)> =
        tokens.iter().zip(&kit).map(|((name, t), kit)| (Variant::Dark, name.clone(), given(t, kit))).collect();
    // A light value, a name the kit does not have, and a value not of its token's kind are left alone in the dark.
    overrides.push((Variant::Light, "rowHeight".to_owned(), "99".to_owned()));
    overrides.push((Variant::Dark, "radius".to_owned(), "3".to_owned()));
    overrides.push((Variant::Dark, "quick".to_owned(), "1.5".to_owned()));
    scene.kit_tokens().expect("the kit's tokens").set_overrides(overrides);
    for ((name, t), kit) in tokens.iter().zip(&kit) {
        assert_eq!(read(&mut scene, name), given(t, kit), "{name}, a {t}, is the theme's");
    }
    // Under reduced motion the theme's durations and its breathing are reduced as the kit's are; its radius stays.
    scene.kit_tokens().expect("the kit's tokens").set_reduced_motion(true);
    assert_eq!(read(&mut scene, "quick"), "0");
    assert_eq!(read(&mut scene, "quickFade"), "120");
    assert_eq!(read(&mut scene, "breathes"), "false");
    assert_eq!(read(&mut scene, "tileRadius"), "7.25");
    // None set: the kit's own again.
    scene.kit_tokens().expect("the kit's tokens").set_reduced_motion(false);
    scene.kit_tokens().expect("the kit's tokens").set_overrides(Vec::new());
    for ((name, _), kit) in tokens.iter().zip(&kit) {
        assert_eq!(&read(&mut scene, name), kit, "{name} is the kit's");
    }
}
