// EmuSen_Kit.md §7.116: the corona disc under Eclipse's tokens, read from its theme.json: its box, its rim's colour as
// given and as the look's where none is given, and its two glows at the tokens' sizes, drawn; one test, as the harness is
// one per process.

use emusen_kit_core::tokens::{Colour, Variant};
use emusen_qt_test::Harness;

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/CoronaDiscScene.qml");
const ECLIPSE: &str = include_str!("../../themes/eclipse/theme.json");
// The scene's box, its scale and its window, test inputs written in CoronaDiscScene.qml.
const BOX: (f64, f64) = (120.0, 80.0);
const SCALE: f64 = 3.0;
const CENTRE: f64 = 950.0;

/// Eclipse's dark overrides, each a token and its value as theme.json writes it.
fn eclipse() -> Vec<(String, String)> {
    let dark = ECLIPSE.split("\"dark\": {").nth(1).expect("Eclipse's dark tokens");
    let dark = &dark[..dark.find('}').expect("their end")];
    dark.lines()
        .filter_map(|l| l.trim().trim_end_matches(',').split_once(": "))
        .map(|(k, v)| (k.trim_matches('"').to_owned(), v.trim_matches('"').to_owned()))
        .collect()
}

fn number(tokens: &[(String, String)], name: &str) -> f64 {
    tokens.iter().find(|(k, _)| k == name).and_then(|(_, v)| v.parse().ok()).unwrap_or_else(|| panic!("Eclipse's {name}"))
}

fn colour(tokens: &[(String, String)], name: &str) -> Colour {
    tokens.iter().find(|(k, _)| k == name).and_then(|(_, v)| Colour::parse(v)).unwrap_or_else(|| panic!("Eclipse's {name}"))
}

fn read_colour(text: &str) -> Colour {
    Colour::parse(text).unwrap_or_else(|| panic!("a colour: {text}"))
}

/// How bright a drawn point is, over the scene's black.
fn light(c: Option<Colour>) -> u32 {
    c.map_or(0, |c| u32::from(c.r) + u32::from(c.g) + u32::from(c.b))
}

#[test]
fn kit_7_116_a_corona_disc_holds_its_box_its_rim_s_colour_and_its_glows() {
    let tokens = eclipse();
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 1900, 1900);
    let given: Vec<(Variant, String, String)> = tokens.iter().map(|(k, v)| (Variant::Dark, k.clone(), v.clone())).collect();
    scene.kit_tokens().expect("the kit's tokens").set_overrides(given);
    scene.wait(100);

    // The box: a disc as wide as the box's shorter side, at its centre.
    let side = BOX.0.min(BOX.1);
    assert_eq!(scene.property("coronaDisc", "width").parse::<f64>().unwrap(), side);
    assert_eq!(scene.property("coronaDisc", "height").parse::<f64>().unwrap(), side);
    assert_eq!(scene.property("coronaDisc", "x").parse::<f64>().unwrap(), (BOX.0 - side) / 2.0);
    let rim = number(&tokens, "coronaRim");
    assert_eq!(scene.property("coronaRim", "width").parse::<f64>().unwrap(), side + 2.0 * rim);

    // The rim's colour, the look's where none is given, as a property and as drawn at the rim's middle.
    let at_rim = (CENTRE + SCALE * (side / 2.0 + rim / 2.0), CENTRE);
    let look = colour(&tokens, "coronaColor");
    assert_eq!(read_colour(&scene.property("coronaRim", "color")), look);
    assert_eq!(scene.colours_at(&[at_rim])[0].map(|c| (c.r, c.g, c.b)), Some((look.r, look.g, look.b)), "the look's rim drawn");
    // A game's corona given: the rim takes it (a test input).
    let game = Colour::rgb(0x30, 0x80, 0xF0);
    assert!(scene.set_property("disc", "corona", "#3080F0"));
    assert_eq!(read_colour(&scene.property("coronaRim", "color")), game);
    assert_eq!(scene.colours_at(&[at_rim])[0].map(|c| (c.r, c.g, c.b)), Some((game.r, game.g, game.b)), "the game's rim drawn");

    // The glows, reached through the tree as a Repeater's delegates are: the glow's and the halo's sizes, each painted past
    // the disc by its spread and twice its blur.
    let tree = scene.tree();
    let id_of = |name: &str| tree.find_named(name).map(|i| tree.nodes[i as usize].id.clone()).unwrap_or_else(|| panic!("{name} in the tree"));
    for (index, which) in [(0, "Glow"), (1, "Halo")] {
        let (blur, spread) = (number(&tokens, &format!("corona{which}Blur")), number(&tokens, &format!("corona{which}Spread")));
        let glow = id_of(&format!("coronaGlow.{index}"));
        assert!((scene.item_number(&glow, "blur") - blur).abs() < 1e-9, "{which}'s blur");
        assert!((scene.item_number(&glow, "spread") - spread).abs() < 1e-9, "{which}'s spread");
        let painted = id_of(&format!("coronaGlowPainted.{index}"));
        assert_eq!(scene.item_number(&painted, "visible"), 1.0, "{which} is painted on the software backend");
        let width = scene.item_number(&painted, "width");
        assert!((width - (side + 2.0 * (spread + 2.0 * blur))).abs() < 1e-6, "{which} painted {width} wide");
    }
    // Drawn: just past the rim the glow, in its hue; past the glow's reach the halo alone, in its own; past the halo's, black.
    let (glow_reach, halo_reach) = (
        number(&tokens, "coronaGlowSpread") + 2.0 * number(&tokens, "coronaGlowBlur"),
        number(&tokens, "coronaHaloSpread") + 2.0 * number(&tokens, "coronaHaloBlur"),
    );
    let out = |d: f64| (CENTRE + SCALE * (side / 2.0 + d), CENTRE);
    let drawn = scene.colours_at(&[out(rim + 2.0), out(glow_reach + 4.0), out(halo_reach + 4.0)]);
    assert!(light(drawn[0]) > 60, "the glow past the rim: {:?}", drawn[0]);
    assert!(light(drawn[1]) > 6, "the halo past the glow's reach: {:?}", drawn[1]);
    assert_eq!(light(drawn[2]), 0, "nothing past the halo's reach: {:?}", drawn[2]);
    // Each glow in its own colour token: the glow green and the halo blue (test inputs over Eclipse's), the halo alone
    // past the glow's reach, the glow over it just past the rim.
    let mut tinted: Vec<(Variant, String, String)> = tokens
        .iter()
        .filter(|(k, _)| k != "coronaGlowColor" && k != "coronaHaloColor")
        .map(|(k, v)| (Variant::Dark, k.clone(), v.clone()))
        .collect();
    tinted.push((Variant::Dark, "coronaGlowColor".to_owned(), "#FF00FF00".to_owned()));
    tinted.push((Variant::Dark, "coronaHaloColor".to_owned(), "#FF0000FF".to_owned()));
    scene.kit_tokens().expect("the kit's tokens").set_overrides(tinted);
    scene.wait(100);
    let drawn = scene.colours_at(&[out(rim + 2.0), out(glow_reach + 4.0)]);
    let near = drawn[0].expect("a point past the rim");
    assert!(near.r == 0 && near.g > 100 && near.g > near.b, "the glow's green over the halo past the rim: {near:?}");
    let far = drawn[1].expect("a point past the glow's reach");
    assert!(far.r == 0 && far.g == 0 && far.b > 20, "the halo's blue alone past the glow's reach: {far:?}");
    scene.assert_quiet();
}
