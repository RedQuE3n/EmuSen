// EmuSen_Kit.md §7.113: on the software backend the glow's blur is made once a geometry: a change of colour recolours the kept
// shape and blurs nothing again, a change of size blurs once more, and a recoloured glow is the glow painted afresh in that
// colour. The time of 50 colour changes is printed. One test, as the harness is one per process.

use cxx_qt_lib::QString;
use emusen_qt_test::tree::Tree;
use emusen_qt_test::{Harness, Scene};
use std::time::Instant;

const URL: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/RecolourScene.qml");

/// A number of the canvas of the named glow.
fn canvas(scene: &mut Scene<'_>, glow: &str, property: &str) -> f64 {
    let tree: Tree = scene.tree();
    let parent = tree.nodes[tree.find_named(glow).expect("the glow") as usize].id.clone();
    let mut at = vec![parent];
    while let Some(id) = at.pop() {
        for n in tree.nodes.iter().filter(|n| n.parent == id) {
            if n.name == "glowCanvas" {
                let object = n.id.parse::<u64>().unwrap() as *mut cxx_qt::QObject;
                // Safety: the id was read from the live tree.
                let object = unsafe { core::pin::Pin::new_unchecked(&mut *object) };
                return emusen_qt_test::ffi::ffi::harness_property(object, &QString::from("glowCanvas"), &QString::from(property)).to_string().parse().unwrap();
            }
            at.push(n.id.clone());
        }
    }
    panic!("{glow} has no canvas")
}

/// Sets the first glow's colour and waits for its paint.
fn recolour(scene: &mut Scene<'_>, ink: &str) {
    let before = canvas(scene, "recoloured", "paints");
    assert!(scene.set_property("scene", "ink", ink));
    for _ in 0..200 {
        scene.wait(5);
        if canvas(scene, "recoloured", "paints") > before {
            return;
        }
    }
    panic!("the glow did not paint {ink}");
}

#[test]
fn kit_7_113_a_glow_recoloured_blurs_nothing_again() {
    let mut harness = Harness::new();
    let mut scene = harness.load(URL, 1400, 700);
    scene.wait(300);
    assert!(emusen_qt_test::ffi::ffi::harness_software(), "the software backend");
    assert_eq!(canvas(&mut scene, "recoloured", "blurs"), 1.0, "blurred once");
    let inks = ["#476A4CF0", "#47ECA0BA", "#471FC9A8", "#47D889A8"];
    // Fifty, or as many as EMUSEN_RECOLOURS names, for timing the glow before this part kept its blur.
    let changes: usize = std::env::var("EMUSEN_RECOLOURS").ok().and_then(|n| n.parse().ok()).unwrap_or(50);
    let started = Instant::now();
    for i in 0..changes {
        recolour(&mut scene, inks[i % inks.len()]);
    }
    println!("recolour\tsoftware\t{changes} colour changes\t{} ms", started.elapsed().as_millis());
    assert_eq!(canvas(&mut scene, "recoloured", "blurs"), 1.0, "fifty colours, no blur again");

    // Recoloured to the fresh glow's colour, it draws what the fresh one draws, beside it.
    recolour(&mut scene, "#476A4CF0");
    let mut worst = 0u8;
    for (dx, dy) in [(-20.0, 150.0), (-60.0, 150.0), (150.0, -40.0), (-10.0, -10.0), (320.0, 150.0), (150.0, 400.0)] {
        let got = scene.colours_at(&[(200.0 + dx, 200.0 + dy), (900.0 + dx, 200.0 + dy)]);
        let (a, b) = (got[0].unwrap(), got[1].unwrap());
        worst = worst.max(a.r.abs_diff(b.r)).max(a.g.abs_diff(b.g)).max(a.b.abs_diff(b.b));
    }
    assert!(worst <= 1, "the recoloured glow stands {worst} levels from the fresh one");
    let near = scene.colours_at(&[(190.0, 350.0)])[0].unwrap();
    assert!(near.b > near.r && near.b > near.g && near.b > 20, "the recoloured glow is drawn in its blue just past its box: {near:?}");

    // A change of size blurs once more.
    assert!(scene.set_property("scene", "side", "320"));
    scene.wait(300);
    assert_eq!(canvas(&mut scene, "recoloured", "blurs"), 2.0, "a new size, one blur");
    scene.assert_quiet();
}
