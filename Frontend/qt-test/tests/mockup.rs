//! The mockups as numbers (EmuSen_Qt.md §8.2): the three approved canvases read, and the comparison holding an item to
//! its stated box within two design units, its token's colour exactly and its words.

use emusen_kit_core::tokens::Variant;
use emusen_qt_test::mockup::{compare, parse, Edge};
use emusen_qt_test::tree::Node;
use std::path::PathBuf;

fn mockups() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../Testing/mockups")
}

#[test]
fn the_three_canvases_read() {
    for (file, canvas, least) in [("Consoles", "Consoles", 60), ("Home", "Home", 25), ("Grid", "Grid", 18)] {
        let text = std::fs::read_to_string(mockups().join(format!("{file}.mockup.spec"))).unwrap();
        let m = parse(&text).unwrap_or_else(|e| panic!("{file}: {e}"));
        assert_eq!((m.canvas.as_str(), m.width, m.height), (canvas, 1280.0, 800.0));
        assert!(m.elements.len() >= least, "{file}: {} elements", m.elements.len());
        let mut names: Vec<&str> = m.elements.iter().map(|e| e.name.as_str()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), m.elements.len(), "{file}: each element once");
        let bar = m.elements.iter().find(|e| e.name == "helpBar").expect("the help bar");
        assert!(bar.edges.contains(&(Edge::Y, 742.0)), "{file}: the help bar on the foot, as decided on 2026-11-05");
    }
}

fn node(name: &str, at: [f64; 4], colors: &[&str], text: &str) -> Node {
    Node {
        name: name.into(),
        x: at[0],
        y: at[1],
        w: at[2],
        h: at[3],
        visible: true,
        opacity: 1.0,
        colors: colors.iter().map(|c| (*c).to_owned()).collect(),
        text: (!text.is_empty()).then(|| text.to_owned()),
        ..Node::default()
    }
}

#[test]
fn an_item_is_held_to_its_box_its_token_and_its_words() {
    let m = parse(
        "mockup 1\ncanvas T 1280 800\nelement title x=64 y=40 h=38.4 token=TextStrong text=\"GAMES\"\nelement tile x=64 y=100 w=372 h=72 token=SurfaceRaised alpha=0.55\nelement count right=1216 words=data\n",
    )
    .unwrap();
    // At 1920 x 1200 a design unit is 1.5 pixels.
    let good = [
        node("title", [96.0, 61.5, 120.0, 57.6], &["#FFEEEEF0"], "GAMES"),
        node("tile", [96.0, 150.0, 558.0, 108.0], &["#8C26262B"], ""),
        node("count", [1600.0, 0.0, 224.0, 36.0], &[], "3 consoles"),
    ];
    assert_eq!(compare(&m, &good, 1.5, Variant::Dark), Vec::<String>::new());
    let mut off = good.clone();
    off[0].y += 4.5;
    off[1].colors = vec!["#8C1B1B1E".into()];
    off[2].text = None;
    let faults = compare(&m, &off, 1.5, Variant::Dark);
    assert_eq!(faults.len(), 3, "{faults:?}");
    assert!(faults[0].starts_with("title: Y 44.0"), "three units down is past two: {faults:?}");
    assert!(faults[1].contains("not SurfaceRaised"), "{faults:?}");
    assert!(faults[2].contains("no words"), "{faults:?}");
    off[1].colors = vec!["#FF26262B".into()];
    assert!(compare(&m, &off[1..2], 1.5, Variant::Dark).iter().any(|f| f.starts_with("tile: alpha 1.000")), "the alpha is held");
    assert!(compare(&m, &[], 1.5, Variant::Dark).iter().all(|f| f.ends_with("not shown")));
}
