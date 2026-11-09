// The tool screen and the page frame on the gallery: the fit audit, pad reach, pointer reach and no chrome at the four fit
// sizes in both variants, the pane spanning the list and the bar and the columns inside the list as drawn, and the walk
// between the parts (EmuSen_Kit.md §7.39, §7.40). The grabs and recordings are saved beside the gallery's for a look.

use std::path::PathBuf;

use emusen_kit_core::input::{UiButton, WalkForm};
use emusen_kit_core::tokens::Variant;
use emusen_kit_gallery::SCREENS_URL;
use emusen_qt_test::harness::{edges, palette, Measures, SIZES};
use emusen_qt_test::lint::{frontend, qmllint, rules};
use emusen_qt_test::spec::{self, Header};
use emusen_qt_test::tree::Tree;
use emusen_qt_test::{Harness, Scene};

/// The three states: the tool screen, the page with tiles, the page with rows.
const SCREENS: [&str; 3] = ["tool", "page-tiles", "page-rows"];
/// The fit sizes: every size the gate runs but the 4K one.
const FIT: usize = 4;
/// The slack of a pixel and a half at a scale above one, in design units at the smallest scale.
const SLACK: f64 = 1.5;

fn grabs() -> PathBuf {
    let dir = PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cache").join("emusen").join("tmp").join("kit-gallery");
    std::fs::create_dir_all(&dir).expect("the grabs folder");
    dir
}

fn rect(tree: &Tree, name: &str) -> (f64, f64, f64, f64) {
    let i = tree.find_named(name).unwrap_or_else(|| panic!("{name} in the tree")) as usize;
    let n = &tree.nodes[i];
    (n.x, n.y, n.w, n.h)
}

/// The pane spans the list and the bar as drawn: its top the list's, its foot the bar's, its left past the list's right.
fn pane_spans(tree: &Tree) -> Vec<String> {
    let (lx, ly, lw, _) = rect(tree, "toolList");
    let (_, by, _, bh) = rect(tree, "toolBar");
    let (px, py, _, ph) = rect(tree, "toolPane");
    let mut faults = Vec::new();
    if (py - ly).abs() > SLACK || (py + ph - (by + bh)).abs() > SLACK {
        faults.push(format!("the pane {py:.1}..{:.1} does not span the list's top {ly:.1} and the bar's foot {:.1}", py + ph, by + bh));
    }
    if px < lx + lw {
        faults.push(format!("the pane at {px:.1} stands over the list ending {:.1}", lx + lw));
    }
    faults
}

/// Every cell of every row lies inside the list, the table's trail kept clear: FitsWidth as drawn.
fn cells_inside(tree: &Tree) -> Vec<String> {
    let at = tree.find_named("toolRows").expect("the rows") as usize;
    let list = &tree.nodes[at];
    let limit = list.x + list.w - emusen_kit_core::table::TableMeasures::BIG_SCREEN.trail * list.scale + SLACK;
    let mut under = std::collections::HashSet::from([list.id.as_str()]);
    let mut faults = Vec::new();
    for (i, n) in tree.nodes.iter().enumerate() {
        if !under.contains(n.parent.as_str()) {
            continue;
        }
        under.insert(n.id.as_str());
        if n.visible && n.text.is_some() && n.x + n.w > limit {
            faults.push(format!("cell {i} ends at {:.1}, past the list's {limit:.1} less its trail", n.x + n.w));
        }
    }
    faults
}

/// The scene's walk between the parts, from where each press starts, by names in the tree.
fn walks(scene: &mut Scene<'_>, screen: usize) -> Vec<String> {
    let presses: &[(&str, UiButton, &str)] = match screen {
        0 => &[("toolSearch", UiButton::Down, "toolRows"), ("toolApply", UiButton::Up, "toolRows")],
        1 => &[("pageActions", UiButton::Down, "pageTiles"), ("pageTiles", UiButton::Up, "pageActions")],
        _ => &[("pageActions", UiButton::Down, "pageRows"), ("pageRows", UiButton::Up, "pageActions")],
    };
    let mut faults = Vec::new();
    for &(from, button, to) in presses {
        let tree = scene.tree();
        let start = tree.find_named(from).expect("the start");
        let start = tree.operable().into_iter().find(|&i| i == start || ancestors(&tree, i).contains(&start)).unwrap_or(start);
        scene.focus_item(&tree, start);
        scene.pad(button);
        let tree = scene.tree();
        let target = tree.find_named(to).expect("the target");
        let landed = scene.focus(&tree);
        if !landed.is_some_and(|f| f == target || ancestors(&tree, f).contains(&target)) {
            faults.push(format!("{button:?} from {from} landed at {landed:?}, not in {to}"));
        }
    }
    faults
}

fn ancestors(tree: &Tree, index: u32) -> Vec<u32> {
    let mut chain = Vec::new();
    let mut at = tree.nodes[index as usize].parent.clone();
    while let Some(p) = tree.nodes.iter().position(|n| n.id == at) {
        chain.push(p as u32);
        at = tree.nodes[p].parent.clone();
    }
    chain
}

#[test]
fn qt_5_b_the_tool_screen_and_the_page_frame_on_the_gallery() {
    let gallery = frontend().join("kit-gallery").join("qml");
    let findings = rules(&[(&gallery, true)]);
    assert!(findings.is_empty(), "qml-rules on the gallery:\n{}", findings.join("\n"));
    let complaints = qmllint(&[&gallery]);
    assert!(complaints.is_empty(), "qmllint on the gallery:\n{}", complaints.join("\n"));

    let mut harness = Harness::new();
    let mut scene = harness.load(SCREENS_URL, 1280, 800);
    scene.set_token_form(WalkForm::BigScreen);
    let grabs = grabs();
    let mut report = Vec::new();
    let mut states = 0;
    for variant in [Variant::Dark, Variant::Light] {
        scene.set_variant(variant);
        for (screen, screen_name) in SCREENS.iter().enumerate() {
            assert!(scene.set_property("scene", "shown", &screen.to_string()));
            for &(w, h) in &SIZES[..FIT] {
                scene.resize(w, h);
                let tree = scene.tree();
                if let Some(first) = tree.operable().first().copied() {
                    scene.focus_item(&tree, first);
                }
                let mut tree = scene.tree();
                let focus = scene.focus(&tree);
                let name = format!("{screen_name}-{variant:?}-{w}x{h}");
                let mut faults = if screen == 0 { pane_spans(&tree).into_iter().chain(cells_inside(&tree)).collect() } else { Vec::new() };
                if variant == Variant::Dark && (w, h) == (1280, 800) {
                    faults.extend(walks(&mut scene, screen));
                    tree = scene.tree();
                }
                assert!(scene.grab(&grabs.join(format!("{name}.png")).display().to_string()), "grab {name}");
                scene.sample_behind(&mut tree, f64::from(w), f64::from(h));
                let walk = scene.walk(&tree);
                let edges = edges(&walk);
                let declared = palette(variant);
                let header = Header {
                    state: &name,
                    case: "Gallery",
                    kind: "menu",
                    width: w,
                    height: h,
                    measures: Measures::of(WalkForm::BigScreen, f64::from(h)),
                    palette: &declared,
                };
                std::fs::write(grabs.join(format!("{name}.spec")), spec::write(&tree, &tree.root, &header, focus, &walk))
                    .expect("the recording");
                faults.extend(
                    scene
                        .audit(&tree, WalkForm::BigScreen, variant, focus, &edges, f64::from(w), f64::from(h))
                        .iter()
                        .map(ToString::to_string),
                );
                let notes = grabs.join(format!("{name}.faults.txt"));
                if faults.is_empty() {
                    let _ = std::fs::remove_file(&notes);
                } else {
                    let walked: Vec<String> = edges.iter().map(|e| format!("{} {:?} {}", e.from.0, e.button, e.to.0)).collect();
                    let _ = std::fs::write(
                        &notes,
                        format!("{}\n-- items:\n{}\n-- edges:\n{}\n", faults.join("\n"), tree.legend(), walked.join("\n")),
                    );
                    report.extend(faults.iter().map(|f| format!("{name}: {f}")));
                }
                states += 1;
            }
        }
    }
    // A, a click and the apply bar's control each reach the host.
    scene.set_variant(Variant::Dark);
    assert!(scene.set_property("scene", "shown", "0"));
    scene.resize(1280, 800);
    let tree = scene.tree();
    let row = tree.operable().into_iter().find(|&i| tree.nodes[i as usize].heard == "Sapphire").expect("Sapphire's row");
    scene.focus_item(&tree, row);
    scene.pad(UiButton::Accept);
    assert_eq!(scene.property("scene", "activated"), "1", "A on Sapphire's row");
    let apply = tree.find_named("toolApply").expect("the apply control");
    let (x, y, w, h) = {
        let n = &tree.nodes[apply as usize];
        (n.x, n.y, n.w, n.h)
    };
    scene.click(x + w / 2.0, y + h / 2.0);
    assert_eq!(scene.property("scene", "applied"), "1", "a click on Apply…");

    scene.assert_quiet();
    assert!(report.is_empty(), "{states} states; faults:\n{}", report.join("\n"));
    assert_eq!(states, 2 * SCREENS.len() * FIT);
}
