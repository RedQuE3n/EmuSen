// EmuSen_Kit.md §7.36–§7.38: the settings frame with a picture, a meter and the two variants in its pane, the page of
// choices and the page of sliders pass the gate (fit, pad reach, pointer reach, no chrome, names, palette, contrast) at
// the four fit sizes, the frame where Mistress's recordings put it (SettingsSound and SettingsDisplay, 1280 x 800 and
// 1920 x 1200); each state recorded beside its grab.

use std::path::PathBuf;

use emusen_kit_core::input::WalkForm;
use emusen_kit_core::tokens::{ColourToken, Variant, colour};
use emusen_kit_gallery::SETTINGS_URL;
use emusen_qt_test::Harness;
use emusen_qt_test::harness::{Measures, SIZES, edges, palette};
use emusen_qt_test::spec::{self, Header};
use emusen_qt_test::tree::Tree;

const STATES: [(&str, &str); 5] = [("frame", "0"), ("meter", "1"), ("variants", "2"), ("choices", "3"), ("slider", "4")];
/// The fit audit's four sizes: the handheld's two panels and the two televisions' (EmuSen_Qt.md §8.1).
const FIT: usize = 4;
const SLACK: f64 = 1.5;

fn grabs() -> PathBuf {
    let dir = PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cache").join("emusen").join("tmp").join("kit-gallery");
    std::fs::create_dir_all(&dir).expect("the grabs folder");
    dir
}

/// A named item's box in window pixels.
fn boxed(tree: &Tree, name: &str) -> (f64, f64, f64, f64) {
    let at = tree.find_named(name).unwrap_or_else(|| panic!("{name} in the tree"));
    let n = &tree.nodes[at as usize];
    (n.x, n.y, n.w, n.h)
}

/// Mistress's boxes, in design units times the scale: the strip, the rows, the first row and the pane.
fn mistress_s_places(tree: &Tree, scale: f64, rows: f64) -> Vec<String> {
    let want = [
        ("settingsStrip", (64.0, 40.0, None, 44.0)),
        ("settingsRows", (64.0, 108.0, Some(704.0), rows * 54.0)),
        ("settingsPane", (800.0, 108.0, Some(416.0), 578.0)),
    ];
    let mut faults = Vec::new();
    for (name, (x, y, w, h)) in want {
        let (gx, gy, gw, gh) = boxed(tree, name);
        let near = |a: f64, b: f64| (a - b * scale).abs() <= SLACK;
        if !(near(gx, x) && near(gy, y) && w.is_none_or(|w| near(gw, w)) && near(gh, h)) {
            faults.push(format!("{name} at {gx:.1},{gy:.1} {gw:.1}x{gh:.1}, Mistress's {x},{y} {w:?}x{h} at {scale}"));
        }
    }
    faults
}

/// The rows' marks wholly below the window draw nothing there: the software backend paints a Shape past its clip
/// unclipped, which MenuRow's marks did until they were held to their clip (EmuSen_Kit.md §7.34, §7.36).
fn nothing_below_the_window(scene: &mut emusen_qt_test::Scene<'_>, tree: &Tree, variant: Variant) -> Vec<String> {
    let (_, y, _, h) = boxed(tree, "settingsList");
    let below: Vec<(f64, f64)> = tree
        .nodes
        .iter()
        .filter(|n| (n.name == "before" || n.name == "after") && n.y >= y + h)
        .map(|n| (n.x + n.w / 2.0, n.y + n.h / 2.0))
        .collect();
    let surface = colour(variant, ColourToken::Surface);
    scene
        .colours_at(&below)
        .into_iter()
        .zip(&below)
        .filter(|(c, _)| c.is_none_or(|c| (c.r, c.g, c.b) != (surface.r, surface.g, surface.b)))
        .map(|(c, (x, y))| format!("a row's mark drawn below the window at {x:.0},{y:.0}: {c:?}"))
        .collect()
}

#[test]
fn qt_5_a_the_settings_frame_the_choices_and_the_sliders_pass_the_gate_at_the_four_sizes() {
    let mut harness = Harness::new();
    let mut scene = harness.load(SETTINGS_URL, 1280, 800);
    let grabs = grabs();
    let mut report = Vec::new();
    let mut states = 0;
    for variant in [Variant::Dark, Variant::Light] {
        scene.set_variant(variant);
        for (state, value) in STATES {
            // The light variant once, on the state whose pane draws both.
            if variant == Variant::Light && state != "variants" {
                continue;
            }
            assert!(scene.set_property("scene", "shown", value));
            for &(w, h) in &SIZES[..FIT] {
                scene.resize(w, h);
                let mut tree = scene.tree();
                let focus = scene.focus(&tree);
                let name = format!("settings-{state}-{variant:?}-{w}x{h}");
                let mut faults = Vec::new();
                if focus.is_none() {
                    faults.push("no row holds the focus".to_owned());
                }
                if w * 10 == h * 16 {
                    let rows = match state {
                        "frame" | "variants" => 10.0,
                        "meter" => 4.0,
                        "choices" => 3.0,
                        _ => 5.0,
                    };
                    faults.extend(mistress_s_places(&tree, f64::from(h) / 800.0, rows));
                }
                faults.extend(nothing_below_the_window(&mut scene, &tree, variant));
                assert!(scene.grab(&grabs.join(format!("{name}.png")).display().to_string()), "grab {name}");
                scene.sample_behind(&mut tree, f64::from(w), f64::from(h));
                let walk = scene.walk(&tree);
                let edges = edges(&walk);
                let declared = palette(variant);
                let header = Header {
                    state: &name,
                    case: "SettingsGallery",
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
                    let _ = std::fs::write(&notes, format!("{}\n-- items:\n{}\n", faults.join("\n"), tree.legend()));
                    report.extend(faults.iter().map(|f| format!("{name}: {f}")));
                }
                states += 1;
            }
        }
    }
    scene.assert_quiet();
    assert!(report.is_empty(), "{states} states; faults:\n{}", report.join("\n"));
    assert_eq!(states, (STATES.len() + 1) * FIT);
}
