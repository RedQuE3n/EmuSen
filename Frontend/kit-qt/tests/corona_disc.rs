// EmuSen_Kit.md §7.116: the corona disc under Eclipse's tokens, read from its theme.json: its box, its rim's colour as
// given and as the look's where none is given, and its two glows at the tokens' sizes, drawn; one test, as the harness is
// one per process.

mod common;

use common::{eclipse, number, Recipe};
use emusen_kit_core::tokens::{Colour, Variant};
use emusen_qt_test::Harness;

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/CoronaDiscScene.qml");
// The scene's box, its scale and its window, test inputs written in CoronaDiscScene.qml.
const BOX: (f64, f64) = (120.0, 80.0);
const SCALE: f64 = 3.0;
const CENTRE: f64 = 950.0;
/// How far each painted glow may stand from the CSS rule, as an alpha: the largest difference measured on 2026-11-14 on the
/// desktop's software backend at the distances below was 0.0192 (EmuSen_Kit.md §7.116).
const MEASURED: f64 = 0.02;

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
    // Decided 2026-11-14 by the tester (R-20261114-38): the glows take the game's corona too, each at its own token's
    // strength; with none given they are the tokens' colours.
    for (part, token) in [("glowColor", "coronaGlowColor"), ("haloColor", "coronaHaloColor")] {
        let strength = colour(&tokens, token).a;
        let seen = read_colour(&scene.property("disc", part));
        assert_eq!((seen.r, seen.g, seen.b), (game.r, game.g, game.b), "{token}'s glow takes the game's corona");
        assert!((i32::from(seen.a) - i32::from(strength)).abs() <= 1, "{token}'s glow keeps its token's strength {strength}: {seen:?}");
    }
    let near = scene.colours_at(&[(CENTRE + SCALE * (side / 2.0 + rim + 2.0), CENTRE)])[0].expect("a point past the rim");
    assert!(near.r < near.g && near.g < near.b, "the glows drawn in the game's blue past the rim: {near:?}");
    assert!(scene.set_property("disc", "corona", "transparent"));
    let near = scene.colours_at(&[(CENTRE + SCALE * (side / 2.0 + rim + 2.0), CENTRE)])[0].expect("a point past the rim");
    assert!(near.r > near.b, "the glows drawn in the look's warm colours again: {near:?}");
    for (part, token) in [("glowColor", "coronaGlowColor"), ("haloColor", "coronaHaloColor")] {
        let seen = read_colour(&scene.property("disc", part));
        assert_eq!(seen, colour(&tokens, token), "with no game's corona, {token}'s own colour");
    }
    assert_eq!(read_colour(&scene.property("coronaRim", "color")), look, "with no game's corona, the look's rim");

    // The glows, reached through the tree as a Repeater's delegates are: the glow's and the halo's sizes, each painted past
    // the disc by its spread and twice its blur.
    let tree = scene.tree();
    let id_of = |name: &str| tree.find_named(name).map(|i| tree.nodes[i as usize].id.clone()).unwrap_or_else(|| panic!("{name} in the tree"));
    for (index, which) in [(0, "Glow"), (1, "Halo")] {
        let (blur, spread) = (number(&tokens, &format!("corona{which}Blur")), number(&tokens, &format!("corona{which}Spread")));
        let glow = id_of(&format!("coronaGlow.{index}"));
        assert!((scene.item_number(&glow, "blur") - blur).abs() < 1e-9, "{which}'s blur");
        assert!((scene.item_number(&glow, "spread") - spread).abs() < 1e-9, "{which}'s spread");
        let at = tree.find_named(&format!("coronaGlow.{index}")).expect("the glow");
        let painted: Vec<u32> = tree.under(at).into_iter().filter(|&i| tree.nodes[i as usize].name == "glowCanvas").collect();
        assert_eq!(painted.len(), 1, "{which} is painted once on the software backend");
        let width = tree.nodes[painted[0] as usize].w;
        let reach = (spread + 2.0 * blur).ceil();
        assert!((width - SCALE * (side + 2.0 * reach)).abs() < 1e-6, "{which} painted {width} wide on the window");
    }
    // Drawn, each glow in a colour of its own (test inputs over Eclipse's): the glow green and the halo blue, opaque, so a
    // pixel's green is the glow's alpha and its blue the halo's under it; each at stated distances past the disc beside the
    // CSS rule, and black past the halo's reach.
    let mut tinted: Vec<(Variant, String, String)> = tokens
        .iter()
        .filter(|(k, _)| k != "coronaGlowColor" && k != "coronaHaloColor")
        .map(|(k, v)| (Variant::Dark, k.clone(), v.clone()))
        .collect();
    tinted.push((Variant::Dark, "coronaGlowColor".to_owned(), "#FF00FF00".to_owned()));
    tinted.push((Variant::Dark, "coronaHaloColor".to_owned(), "#FF0000FF".to_owned()));
    scene.kit_tokens().expect("the kit's tokens").set_overrides(tinted);
    scene.wait(100);
    let recipe = |which: &str| Recipe {
        width: side,
        height: side,
        radius: side / 2.0,
        spread: number(&tokens, &format!("corona{which}Spread")),
        blur: number(&tokens, &format!("corona{which}Blur")),
        alpha: 1.0,
    };
    let (glow, halo) = (recipe("Glow"), recipe("Halo"));
    let out = |d: f64| (CENTRE + SCALE * (side / 2.0 + d), CENTRE);
    let at: Vec<f64> = [rim + 1.0, glow.spread, glow.spread + glow.blur / 2.0, glow.spread + glow.blur, halo.spread, halo.spread + halo.blur / 2.0, halo.spread + halo.blur]
        .iter()
        .map(|d| d.max(rim + 1.0))
        .collect();
    let drawn = scene.colours_at(&at.iter().map(|d| out(*d)).collect::<Vec<_>>());
    let mut worst: f64 = 0.0;
    for (d, c) in at.iter().zip(&drawn) {
        let c = c.expect("a point past the rim");
        assert_eq!(c.r, 0, "no red in either glow: {c:?}");
        let a_glow = f64::from(c.g) / 255.0;
        let a_halo = if a_glow < 1.0 { f64::from(c.b) / 255.0 / (1.0 - a_glow) } else { 0.0 };
        let (e_glow, e_halo) = (glow.css_above(*d), halo.css_above(*d));
        println!("corona\tsoftware\t{d:.2}\tglow {a_glow:.4} css {e_glow:.4}\thalo {a_halo:.4} css {e_halo:.4}");
        worst = worst.max((a_glow - e_glow).abs()).max((a_halo - e_halo).abs());
    }
    println!("the largest difference from the CSS rule: {worst:.4}");
    assert!(worst <= MEASURED, "the disc's glows stand {worst:.4} from the CSS rule, past the {MEASURED} measured");
    let halo_reach = halo.spread + 2.0 * halo.blur;
    assert_eq!(light(scene.colours_at(&[out(halo_reach + 4.0)])[0]), 0, "nothing past the halo's reach");
    scene.assert_quiet();
}
