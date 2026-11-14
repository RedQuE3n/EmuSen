// EmuSen_Kit.md §7.113: one FocusRing marks the focus of a Button, an ActionButton, a TextField and a lit CoverTile, its gap,
// ring and glow drawn as the tokens say under the reference's tokens and under Eclipse's; under the reference each placement
// draws the pixels of the ring it replaced; the glow paints once at rest and falls off as a canvas's shadowBlur. One test, as
// the harness is one per process.

use emusen_kit_core::tokens::{Colour, Variant};
use emusen_qt_test::tree::Tree;
use emusen_qt_test::{Harness, Scene};

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/FocusRingScene.qml");
const ECLIPSE: &str = include_str!("../../themes/eclipse/theme.json");

/// Eclipse's dark overrides as the host hands them to the tokens: each `"name": value` line of its `tokens.dark`, the value as written.
fn eclipse() -> Vec<(Variant, String, String)> {
    let dark = &ECLIPSE[ECLIPSE.find("\"dark\": {").expect("Eclipse's dark tokens")..];
    let dark = &dark[..dark.find('}').unwrap_or(dark.len())];
    dark.lines()
        .skip(1)
        .filter_map(|l| l.trim().trim_end_matches(',').split_once(": "))
        .map(|(n, v)| (Variant::Dark, n.trim_matches('"').to_owned(), v.trim_matches('"').to_owned()))
        .collect()
}

fn number(scene: &mut Scene<'_>, name: &str) -> f64 {
    scene.property("scene", name).parse().unwrap_or(f64::NAN)
}

fn colour(scene: &mut Scene<'_>, name: &str) -> Colour {
    let text = scene.property("scene", name);
    Colour::parse(&text).unwrap_or_else(|| panic!("{name} is a colour: {text}"))
}

/// The visible FocusRings in the named item's subtree.
fn rings_under(tree: &Tree, name: &str) -> Vec<u32> {
    let at = tree.find_named(name).unwrap_or_else(|| panic!("{name} in the scene"));
    let mut under = tree.under(at);
    under.push(at);
    under.into_iter().filter(|&i| tree.nodes[i as usize].ty == "FocusRing" && tree.nodes[i as usize].visible).collect()
}

/// The named child of an item, by objectName, among the items under it.
fn part(tree: &Tree, of: u32, name: &str) -> Option<u32> {
    tree.under(of).into_iter().find(|&i| tree.nodes[i as usize].name == name && tree.nodes[i as usize].visible)
}

/// The colour drawn at a point, opaque, as one grab sees it.
fn at(scene: &mut Scene<'_>, x: f64, y: f64) -> Colour {
    scene.colours_at(&[(x, y)])[0].expect("a colour")
}

fn close(a: Colour, b: Colour, levels: i32) -> bool {
    [(a.r, b.r), (a.g, b.g), (a.b, b.b)].iter().all(|(x, y)| (i32::from(*x) - i32::from(*y)).abs() <= levels)
}

/// The share of the glow's colour over black at a pixel, read on its strongest channel.
fn share(c: Colour, glow: Colour) -> f64 {
    f64::from(c.r) / f64::from(glow.r)
}

/// How far a sampled glow may stand from the rule, as a share: measured on 2026-11-14 within 0.006 of it at the four distances
/// past each Eclipse ring on the software backend, the grab's 8-bit channels being 0.004 a level.
const GLOW_LEVELS: f64 = 0.02;

/// The standard normal's distribution function, by the Abramowitz and Stegun 7.1.26 form of erf (error under 1.5e-7).
fn phi(z: f64) -> f64 {
    let x = z.abs() / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + 0.327_591_1 * x);
    let erf = 1.0 - (((((1.061_405_429 * t - 1.453_152_027) * t) + 1.421_413_741) * t - 0.284_496_736) * t + 0.254_829_592) * t * (-x * x).exp();
    if z >= 0.0 { 0.5 * (1.0 + erf) } else { 0.5 * (1.0 - erf) }
}

/// Every pixel of a before and after pair's ring box equal: the after is 300 pixels right of the before.
fn same_pixels(scene: &mut Scene<'_>, tree: &Tree, before: &str, reach: f64) -> usize {
    let b = &tree.nodes[tree.find_named(before).expect("the before") as usize];
    let (x0, y0, w, h) = (b.x - reach, b.y - reach, b.w + 2.0 * reach, b.h + 2.0 * reach);
    let mut left = Vec::new();
    let mut right = Vec::new();
    for row in 0..h as i64 {
        for column in 0..w as i64 {
            left.push((x0 + column as f64, y0 + row as f64));
            right.push((x0 + column as f64 + 300.0, y0 + row as f64));
        }
    }
    let (l, r) = (scene.colours_at(&left), scene.colours_at(&right));
    assert!(l.iter().any(|c| c.is_some_and(|c| c.r > 0)), "{before}: the ring was drawn");
    l.iter().zip(&r).filter(|(a, b)| a != b).count()
}

/// The focused control's one ring, with its gap, ring and glow as the tokens give them: sampled upwards from the middle of
/// the box's top edge, a straight edge in every control, one pixel row at a time (a point's pixel is the one it rounds to).
fn holds_one_ring(scene: &mut Scene<'_>, name: &str, edge: bool) {
    let tree = scene.tree();
    let rings = rings_under(&tree, name);
    assert_eq!(rings.len(), 1, "{name}: exactly one FocusRing shows");
    let ring = rings[0];
    let n = tree.nodes[ring as usize].clone();
    let id: u64 = n.id.parse().expect("an id");
    let (offset, thickness) = (number(scene, "ringOffset"), number(scene, "ringThickness"));
    let (accent, gap, glow) = (colour(scene, "accent"), colour(scene, "gapColor"), colour(scene, "glowColor"));
    let (inner, outer) = (scene.number(id, "inner"), scene.number(id, "outer"));
    assert_eq!(outer - inner, thickness, "{name}: the ring is ringThickness wide");
    let (centre, top) = (n.x + (n.w / 2.0).floor(), n.y);
    let band = at(scene, centre, top - inner - thickness / 2.0 - 0.5);
    assert!(close(band, accent, 1), "{name}: the ring is the accent {accent:?}, not {band:?}");
    if edge {
        assert_eq!(scene.number(id, "edgeWidth"), thickness, "{name}: the box's border is the ring");
        assert_eq!(inner, -thickness, "{name}: on the box's edge");
    } else {
        let band = part(&tree, ring, "focusBand").expect("the ring's band");
        assert_eq!(n.y - tree.nodes[band as usize].y, outer, "{name}: the ring's outer edge");
        if inner >= 1.0 && gap.a == 255 {
            let seen = at(scene, centre, top - inner / 2.0 - 0.5);
            assert!(close(seen, gap, 1), "{name}: the gap is ringGapColor {gap:?}, not {seen:?}");
        }
        if name == "cover" {
            assert_eq!(inner, offset, "the cover's ring stands past a gap of ringOffset");
        }
    }
    let glows: Vec<u32> = tree.under(ring).into_iter().filter(|&i| tree.nodes[i as usize].name == "focusGlowCanvas").collect();
    assert_eq!(glows.len(), usize::from(glow.a > 0), "{name}: a glow while its colour is not transparent, and only then");
    if let Some(&canvas) = glows.first() {
        let canvas_id: u64 = tree.nodes[canvas as usize].id.parse().expect("an id");
        scene.wait(100);
        let painted = scene.number(canvas_id, "paints");
        let frames = scene.frames();
        scene.wait(500);
        assert_eq!(painted, 1.0, "{name}: one paint for its size and colour");
        assert_eq!(scene.number(canvas_id, "paints"), painted, "{name}: no paint at rest over {} frames", scene.frames() - frames);
        // The rule: Qt's Context2D page gives none for shadowBlur; its source (qquickcontext2dcommandbuffer.cpp, Qt 6.11) blurs
        // with three passes of a box 2 round(blur / 2) + 1 wide, close to a Gaussian of sigma = blur / 2, the board's CSS rule.
        // At a straight edge the glow's alpha d past the box is its colour's alpha times Phi((spread - d) / sigma).
        let (blur, spread) = (number(scene, "glowBlur"), number(scene, "glowSpread"));
        let sigma = blur / 2.0;
        let alpha = f64::from(glow.a) / 255.0;
        for d in [spread, spread + sigma, spread + 2.0 * sigma, spread + 3.0 * sigma].map(f64::floor) {
            if d < outer + 1.0 {
                continue;
            }
            let seen = share(at(scene, centre, top - d - 1.0), glow);
            let expected = alpha * phi((spread - (d + 0.5)) / sigma);
            println!("{name}: the glow {:.1} past the box: {seen:.3}, Phi gives {expected:.3}", d + 0.5);
            assert!((seen - expected).abs() <= GLOW_LEVELS, "{name}: the glow {d:.1} past the box is {seen:.3}, {expected:.3} by the rule");
        }
        let past = at(scene, centre, top - (spread + 2.0 * blur).ceil() - 2.0);
        assert_eq!(past, Colour::rgb(0, 0, 0), "{name}: nothing past the glow's reach");
    }
}

fn holds_every_control(scene: &mut Scene<'_>) {
    for (name, edge) in [("button", true), ("action", false), ("field", true)] {
        let tree = scene.tree();
        let item = tree.find_named(name).expect("the control");
        assert!(scene.focus_item(&tree, item), "{name} takes the focus");
        scene.wait(50);
        holds_one_ring(scene, name, edge);
    }
    assert!(scene.set_property("cover", "lit", "true"));
    scene.wait(50);
    holds_one_ring(scene, "cover", false);
    assert!(scene.set_property("cover", "lit", "false"));
}

#[test]
fn kit_7_113_one_focus_ring_its_gap_ring_and_glow_from_the_tokens() {
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 1280, 800);

    // The reference's tokens: each placement draws the ring it replaced to the pixel, and no glow is made.
    let tree = scene.tree();
    let reach = number(&mut scene, "ringOffset") + number(&mut scene, "ringThickness") + 2.0;
    for before in ["tightBefore", "gappedBefore", "edgeBefore"] {
        assert_eq!(same_pixels(&mut scene, &tree, before, reach), 0, "{before}: the FocusRing draws the same pixels");
    }
    holds_every_control(&mut scene);

    // Eclipse's tokens: the board's gap, ring and glow.
    scene.kit_tokens().expect("the kit's tokens").set_overrides(eclipse());
    scene.wait(50);
    assert_eq!(colour(&mut scene, "gapColor").a, 255, "Eclipse's gap is drawn");
    assert!(colour(&mut scene, "glowColor").a > 0, "Eclipse glows");
    holds_every_control(&mut scene);
    scene.assert_quiet();
}
