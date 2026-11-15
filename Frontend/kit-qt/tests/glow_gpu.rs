// EmuSen_Kit.md §7.113: the kit's one glow on the graphics card, Qt's RectangularShadow masked to outside its box, in a
// compositor of the test's own: under Eclipse's tokens the ring's glow and the disc's two, nothing inside a box that draws
// nothing, nothing past the reach, the glow's hue, one cast glow a part and none while hidden or transparent, no canvas;
// the glow's alpha past each box and the frames of 200 focus moves printed for the manual's tables. A GPU job.

mod common;

use std::time::Instant;

use common::scene::{self, anywhere, box_of, colour, dark_inside, distances, past_reach, seen, under};
use common::{eclipse, eclipse_dark, number};
use emusen_kit_core::tokens::{Colour, Variant};
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::{Harness, Scene};

/// The focus moves timed, each way.
const MOVES: usize = 200;

#[test]
fn kit_7_113_the_cast_glow_on_the_card_stands_outside_its_box_alone() {
    let kwin = Compositor::start(scene::WINDOW.0, scene::WINDOW.1);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    let tokens = eclipse();
    let mut harness = Harness::new();
    let mut scene = harness.load(scene::URL, scene::WINDOW.0, scene::WINDOW.1);
    if scene.property("scene", "objectName").is_empty() {
        println!("no scene on the card: the test stands down");
        return;
    }

    let tree = scene.tree();
    assert_eq!(anywhere(&tree, "glowShadow"), 0, "no glow under the kit's tokens, its colour transparent");
    scene.kit_tokens().expect("the kit's tokens").set_overrides(eclipse_dark());
    scene.wait(200);
    let tree = scene.tree();
    assert_eq!(anywhere(&tree, "glowCanvas") + anywhere(&tree, "Canvas"), 0, "no canvas on the card");
    assert_eq!(under(&tree, "ring", "glowShadow"), 1, "one cast glow for the ring");
    assert_eq!(under(&tree, "disc", "glowShadow"), 1, "one cast glow for the disc");

    let ring_box = box_of(&tree, "ring");
    let glow = colour(&mut scene, "glowColor");
    let recipe = scene::ring(&tokens, glow, ring_box);
    let outer = number(&tokens, "ringOffset") + number(&tokens, "ringThickness");
    rows(&mut scene, "ring", ring_box, glow, &recipe, outer + 1.0);
    let inside = dark_inside(&mut scene, ring_box, recipe.radius);
    assert!(inside.is_empty(), "nothing inside the ring's box, which draws nothing: {inside:?}");
    assert_eq!(past_reach(&mut scene, ring_box, &recipe), Colour::rgb(0, 0, 0), "the ring's glow ends at its reach");
    hue(&mut scene, ring_box, glow, outer + 1.5, "ring");

    let disc_box = box_of(&tree, "disc");
    for halo in [false, true] {
        assert!(scene.set_property("scene", "halo", if halo { "1" } else { "0" }));
        scene.wait(200);
        let name = if halo { "disc's halo" } else { "disc's glow" };
        let glow = colour(&mut scene, "discColor");
        let recipe = scene::disc(&tokens, halo, glow, disc_box);
        rows(&mut scene, name, disc_box, glow, &recipe, 0.0);
        let inside = dark_inside(&mut scene, disc_box, recipe.radius);
        assert!(inside.is_empty(), "nothing inside the {name}'s disc: {inside:?}");
        assert_eq!(past_reach(&mut scene, disc_box, &recipe), Colour::rgb(0, 0, 0), "the {name} ends at its reach");
        hue(&mut scene, disc_box, glow, 1.5, name);
    }

    assert!(scene.set_property("scene", "ringOn", "false"));
    let tree = scene.tree();
    assert_eq!(under(&tree, "ring", "glowShadow"), 0, "the hidden ring casts no glow");

    // The focus walked over three controls, with the glow and with its colour transparent: each move's time to the frame
    // that shows it, printed; Qt's render loop prints each frame's own times where QSG_RENDER_TIMING is set.
    timed(&mut scene, "glow on");
    scene.assert_still("a focused control's glow at rest");
    let mut clear = eclipse_dark();
    clear.retain(|(_, n, _)| n != "glowColor");
    clear.push((Variant::Dark, "glowColor".into(), "#00000000".into()));
    scene.kit_tokens().expect("the kit's tokens").set_overrides(clear);
    scene.wait(200);
    timed(&mut scene, "glow transparent");
    let tree = scene.tree();
    assert_eq!(anywhere(&tree, "glowShadow"), 1, "a transparent ring glow casts nothing; the disc's stays");
    scene.assert_quiet();
}

/// The glow seen past a box beside the CSS rule, as the manual's table rows.
fn rows(scene: &mut Scene<'_>, name: &str, b: (f64, f64, f64, f64), glow: Colour, r: &common::Recipe, from: f64) {
    let at = distances(r, from);
    let got = seen(scene, b, glow, &at);
    for (d, s) in at.iter().zip(&got) {
        println!("glow\tcard\t{name}\t{d:.1}\t{s:.4}\t{:.4}", r.css_above(*d));
    }
    assert!(got[0] > 0.0, "{name}: the glow begins {:.1} past its box", at[0]);
}

/// The glow's colour where it is strongest past the box: each channel the glow's own times one share, within a level.
fn hue(scene: &mut Scene<'_>, b: (f64, f64, f64, f64), glow: Colour, d: f64, name: &str) {
    let c = scene.colours_at(&[(b.0 + (b.2 / 2.0).floor(), b.1 - d - 0.5)])[0].expect("a colour");
    let s = common::share((c.r, c.g, c.b), (glow.r, glow.g, glow.b));
    for (seen, own) in [(c.r, glow.r), (c.g, glow.g), (c.b, glow.b)] {
        let expected = f64::from(own) * s;
        assert!((f64::from(seen) - expected).abs() <= 1.5, "{name}: the glow's hue {c:?}, not the colour {glow:?} at {s:.3}");
    }
}

/// 200 focus moves over the three controls; each from the move to the next swapped frame, median and worst, printed.
fn timed(scene: &mut Scene<'_>, what: &str) {
    let tree = scene.tree();
    let controls: Vec<u32> = ["button", "switch", "action"].iter().map(|n| tree.find_named(n).expect("a control")).collect();
    eprintln!("timing {what}: start");
    let mut times = Vec::with_capacity(MOVES);
    for i in 0..MOVES {
        let before = scene.frames();
        let start = Instant::now();
        assert!(scene.focus_item(&tree, controls[i % controls.len()]));
        while scene.frames() == before && start.elapsed().as_millis() < 1000 {
            scene.wait(1);
        }
        times.push(start.elapsed().as_secs_f64() * 1000.0);
        scene.wait(20);
    }
    eprintln!("timing {what}: end");
    times.sort_by(f64::total_cmp);
    println!("timing\t{what}\t{MOVES} moves\tmedian {:.2} ms\tworst {:.2} ms", times[MOVES / 2], times[MOVES - 1]);
}
