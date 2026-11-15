// EmuSen_Kit.md §7.113: the kit's one glow on the software backend, where Qt's Context2D paints it: under Eclipse's tokens
// the ring's glow and the disc's two, each read past its box against the CSS rule, nothing inside the box, nothing past its
// reach, one painted glow a part and none while hidden or transparent. One test, as the harness is one per process.

mod common;

use common::scene::{self, anywhere, box_of, colour, dark_inside, distances, past_reach, seen, under};
use common::{eclipse, eclipse_dark};
use emusen_kit_core::tokens::Colour;
use emusen_qt_test::Harness;

/// How far the painted glow may stand from the CSS rule, as an alpha: the largest difference measured on 2026-11-14 on the
/// desktop's software backend at the distances this test reads, over the ring's and the disc's recipes, was 0.0119
/// (EmuSen_Kit.md §7.113).
const MEASURED: f64 = 0.012;

#[test]
fn kit_7_113_the_painted_glow_follows_the_css_rule_outside_its_box_alone() {
    let tokens = eclipse();
    let mut harness = Harness::new();
    let mut scene = harness.load(scene::URL, scene::WINDOW.0, scene::WINDOW.1);

    // The kit's own tokens: no glow is made anywhere.
    let tree = scene.tree();
    assert_eq!(anywhere(&tree, "glowCanvas"), 0, "no glow under the kit's tokens, its colour transparent");

    scene.kit_tokens().expect("the kit's tokens").set_overrides(eclipse_dark());
    scene.wait(100);
    let tree = scene.tree();
    assert_eq!(under(&tree, "ring", "glowCanvas"), 1, "one painted glow for the ring");
    assert_eq!(under(&tree, "disc", "glowCanvas"), 1, "one painted glow for the disc");
    assert_eq!(anywhere(&tree, "glowShadow"), 0, "no RectangularShadow on the software backend");

    let mut worst: f64 = 0.0;
    let ring_box = box_of(&tree, "ring");
    let glow = colour(&mut scene, "glowColor");
    let recipe = scene::ring(&tokens, glow, ring_box);
    worst = worst.max(table(&mut scene, "ring", ring_box, glow, &recipe, tokens_outer(&tokens) + 1.0));
    assert!(dark_inside(&mut scene, ring_box, recipe.radius).is_empty(), "nothing inside the ring's box: {:?}", dark_inside(&mut scene, ring_box, recipe.radius));
    assert_eq!(past_reach(&mut scene, ring_box, &recipe), Colour::rgb(0, 0, 0), "the ring's glow ends at its reach");

    let disc_box = box_of(&tree, "disc");
    for halo in [false, true] {
        assert!(scene.set_property("scene", "halo", if halo { "1" } else { "0" }));
        scene.wait(100);
        let glow = colour(&mut scene, "discColor");
        let recipe = scene::disc(&tokens, halo, glow, disc_box);
        let name = if halo { "disc's halo" } else { "disc's glow" };
        worst = worst.max(table(&mut scene, name, disc_box, glow, &recipe, 0.0));
        assert!(dark_inside(&mut scene, disc_box, recipe.radius).is_empty(), "nothing inside the {name}'s disc");
        assert_eq!(past_reach(&mut scene, disc_box, &recipe), Colour::rgb(0, 0, 0), "the {name} ends at its reach");
    }
    println!("the largest difference from the CSS rule: {worst:.4}");
    assert!(worst <= MEASURED, "the painted glow stands {worst:.4} from the CSS rule, past the {MEASURED} measured");

    // Hidden: the ring's glow goes with it.
    assert!(scene.set_property("scene", "ringOn", "false"));
    let tree = scene.tree();
    assert_eq!(anywhere(&tree, "glowCanvas"), 1, "the hidden ring's glow is gone, the disc's stays");
    scene.assert_quiet();
}

/// The ring's outer edge under Eclipse: its offset and its thickness past the box.
fn tokens_outer(tokens: &[(String, String)]) -> f64 {
    common::number(tokens, "ringOffset") + common::number(tokens, "ringThickness")
}

/// The glow seen at each distance beside the CSS rule, printed as a table's rows; the largest difference.
fn table(scene: &mut emusen_qt_test::Scene<'_>, name: &str, b: (f64, f64, f64, f64), glow: Colour, r: &common::Recipe, from: f64) -> f64 {
    let at = distances(r, from);
    let got = seen(scene, b, glow, &at);
    let mut worst: f64 = 0.0;
    for (d, s) in at.iter().zip(&got) {
        let css = r.css_above(*d);
        println!("glow\tsoftware\t{name}\t{d:.1}\t{s:.4}\t{css:.4}");
        worst = worst.max((s - css).abs());
    }
    worst
}
