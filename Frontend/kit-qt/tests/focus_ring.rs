// EmuSen_Kit.md §7.113: one FocusRing marks the focus of every site the kit rings, each in the placement the table below
// gives it under the reference's tokens and Gapped under Eclipse's (R-20261114-21); a Button, a Switch, an ActionButton, a
// TextField and a lit CoverTile show one ring, its gap, ring and glow as the tokens say; under the reference each placement
// draws the pixels of the ring it replaced; the glow paints once at rest and follows the CSS rule. One test, as the harness
// is one per process.

mod common;

use common::Recipe;
use emusen_kit_core::tokens::Colour;
use emusen_qt_test::tree::Tree;
use emusen_qt_test::{Harness, Scene};

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/FocusRingScene.qml");
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

/// How far a sampled glow may stand from the rule, as an alpha: the largest difference measured on 2026-11-14 on the
/// desktop's software backend at these distances past each Eclipse ring was 0.0058 (EmuSen_Kit.md §7.113).
const GLOW_LEVELS: f64 = 0.006;

/// FocusRing's placements, as its `Placement` enumeration numbers them.
const GAPPED: f64 = 0.0;
const TIGHT: f64 = 1.0;
const EDGE: f64 = 2.0;

/// Every site the kit rings and the placement each keeps under the reference's tokens (R-20261114-21; §7.113): the scene's
/// item, and where the ring is drawn. The eleventh, a PageFrame's tile, Tight, is held by kit-gallery's `page_ring`, where a
/// page has tiles to ring.
const SITES: [(&str, f64); 10] = [
    ("button", EDGE),
    ("check", EDGE),
    ("combo", EDGE),
    ("radio", EDGE),
    ("slider", EDGE),
    ("switch", EDGE),
    ("tab", EDGE),
    ("field", EDGE),
    ("action", TIGHT),
    ("cover", GAPPED),
];

/// Each site's one FocusRing, shown or not, holds its placement and draws the look's: its own under the reference's
/// tokens, Gapped under a look that sets `ringGapped`.
fn holds_the_placements(scene: &mut Scene<'_>, gapped: bool) {
    let tree = scene.tree();
    for (name, own) in SITES {
        let at = tree.find_named(name).unwrap_or_else(|| panic!("{name} in the scene"));
        let mut under = tree.under(at);
        under.push(at);
        let rings: Vec<u32> = under.into_iter().filter(|&i| tree.nodes[i as usize].ty == "FocusRing").collect();
        assert_eq!(rings.len(), 1, "{name}: one FocusRing");
        let id: u64 = tree.nodes[rings[0] as usize].id.parse().expect("an id");
        assert_eq!(scene.number(id, "placement"), own, "{name}: its own placement");
        let drawn = if gapped { GAPPED } else { own };
        assert_eq!(scene.number(id, "form"), drawn, "{name}: drawn {}", if gapped { "Gapped under Eclipse" } else { "in its own placement" });
    }
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
fn holds_one_ring(scene: &mut Scene<'_>, name: &str, form: f64) {
    let edge = form == EDGE;
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
        let stands = n.y - tree.nodes[band as usize].y;
        assert!((stands - outer).abs() < 1e-9, "{name}: the ring's outer edge {stands} past the box, not {outer}");
        if inner >= 1.0 && gap.a == 255 {
            let seen = at(scene, centre, top - inner / 2.0 - 0.5);
            assert!(close(seen, gap, 1), "{name}: the gap is ringGapColor {gap:?}, not {seen:?}");
        }
        if form == GAPPED {
            assert_eq!(inner, offset, "{name}: the ring stands past a gap of ringOffset");
            assert_eq!(scene.number(id, "onEdge"), 0.0, "{name}: the box keeps its own border");
        } else {
            assert_eq!(inner, offset - thickness, "{name}: the ring's outer edge ringOffset past the box");
        }
    }
    let glows: Vec<u32> = tree.under(ring).into_iter().filter(|&i| tree.nodes[i as usize].name == "glowCanvas").collect();
    assert_eq!(glows.len(), usize::from(glow.a > 0), "{name}: a glow while its colour is not transparent, and only then");
    if let Some(&canvas) = glows.first() {
        let canvas_id: u64 = tree.nodes[canvas as usize].id.parse().expect("an id");
        scene.wait(100);
        let painted = scene.number(canvas_id, "paints");
        let frames = scene.frames();
        scene.wait(500);
        assert_eq!(painted, 1.0, "{name}: one paint for its size and colour");
        assert_eq!(scene.number(canvas_id, "paints"), painted, "{name}: no paint at rest over {} frames", scene.frames() - frames);
        // The rule, CSS Backgrounds 3 §6.1.2: the box grown by the spread, blurred by a Gaussian of sigma = blur / 2, outside
        // the box alone; integrated over the ring's own box and corners (tests/common).
        let (blur, spread) = (number(scene, "glowBlur"), number(scene, "glowSpread"));
        let rule = Recipe { width: n.w, height: n.h, radius: scene.number(id, "radius"), spread, blur, alpha: f64::from(glow.a) / 255.0 };
        let sigma = blur / 2.0;
        for d in [spread, spread + sigma, spread + 2.0 * sigma, spread + 3.0 * sigma].map(f64::floor) {
            if d < outer + 1.0 {
                continue;
            }
            let seen = share(at(scene, centre, top - d - 1.0), glow);
            let expected = rule.css_above(d + 0.5);
            println!("{name}: the glow {:.1} past the box: {seen:.4}, the rule gives {expected:.4}", d + 0.5);
            assert!((seen - expected).abs() <= GLOW_LEVELS, "{name}: the glow {d:.1} past the box is {seen:.4}, {expected:.4} by the rule");
        }
        let past = at(scene, centre, top - (spread + 2.0 * blur).ceil() - 2.0);
        assert_eq!(past, Colour::rgb(0, 0, 0), "{name}: nothing past the glow's reach");
    }
}

/// A focused Button, Switch, ActionButton and TextField and a lit CoverTile each show one ring in the look's placement.
fn holds_every_control(scene: &mut Scene<'_>, gapped: bool) {
    let form_of = |name: &str| if gapped { GAPPED } else { SITES.iter().find(|(n, _)| *n == name).map_or(f64::NAN, |(_, f)| *f) };
    for name in ["button", "switch", "action", "field"] {
        let tree = scene.tree();
        let item = tree.find_named(name).expect("the control");
        assert!(scene.focus_item(&tree, item), "{name} takes the focus");
        scene.wait(50);
        holds_one_ring(scene, name, form_of(name));
    }
    assert!(scene.set_property("cover", "lit", "true"));
    scene.wait(50);
    holds_one_ring(scene, "cover", form_of("cover"));
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
    holds_the_placements(&mut scene, false);
    holds_every_control(&mut scene, false);

    // Eclipse's tokens: the board's gap, ring and glow.
    scene.kit_tokens().expect("the kit's tokens").set_overrides(common::eclipse_dark());
    scene.wait(50);
    assert_eq!(colour(&mut scene, "gapColor").a, 255, "Eclipse's gap is drawn");
    assert!(colour(&mut scene, "glowColor").a > 0, "Eclipse glows");
    holds_the_placements(&mut scene, true);
    holds_every_control(&mut scene, true);
    scene.assert_quiet();
}
