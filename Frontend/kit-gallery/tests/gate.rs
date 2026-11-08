// EmuSen_Qt.md §7's stage 0 gate: the fit audit, pad reach, pointer reach, no chrome, names, palette reach and the style sweep, green on the gallery at the four sizes, in every form and variant; the grabs saved for a human look.

use std::path::PathBuf;

use emusen_kit_core::input::{UiButton, WalkForm};
use emusen_kit_core::tokens::Variant;
use emusen_kit_gallery::GALLERY_URL;
use emusen_qt_test::harness::{palette, SIZES};
use emusen_qt_test::lint::{frontend, qmllint, rules};
use emusen_qt_test::{Harness, Scene};

const FORMS: [(WalkForm, &str, &str); 3] =
    [(WalkForm::BigScreen, "big", "1"), (WalkForm::Desktop, "desktop", "0"), (WalkForm::Card, "card", "2")];
const SECTIONS: [&str; 3] = ["Menu", "Controls", "Moonrise"];
const SWEPT: [&str; 6] = ["play", "primary", "field", "combo", "progress", "tabs"];
const KEY_F1: i32 = 0x0100_0030;
/// The audit's slack at a scale above one: a pixel and a half.
const SLACK: f64 = 1.5;

fn grabs() -> PathBuf {
    let dir = PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cache").join("emusen").join("tmp").join("kit-gallery");
    std::fs::create_dir_all(&dir).expect("the grabs folder");
    dir
}

/// Waits for the splash's last frame, so its words are audited as they stay.
fn finish_moonrise(scene: &mut Scene<'_>) {
    for _ in 0..120 {
        if scene.property("moonrise", "finished") == "true" {
            return;
        }
        scene.wait(100);
    }
    panic!("the splash did not finish in 12 s");
}

/// The finished splash shows every star of the draft's sky.
fn moonrise_stars(scene: &mut Scene<'_>) {
    assert_eq!(scene.property("moonrise", "starsShown"), "220", "the splash's stars, which begin at no opacity");
}

/// The carousel's promise: nothing under it rests outside its box, so no renderer, clipping or not, draws a world over its neighbours.
fn inside_carousel(tree: &emusen_qt_test::tree::Tree) -> Vec<String> {
    let Some(at) = tree.find_named("carousel") else { return Vec::new() };
    let carousel = &tree.nodes[at as usize];
    let mut under = std::collections::HashSet::from([carousel.id.as_str()]);
    let mut faults = Vec::new();
    for (i, n) in tree.nodes.iter().enumerate() {
        if !under.contains(n.parent.as_str()) {
            continue;
        }
        under.insert(n.id.as_str());
        let outside = n.x < carousel.x - SLACK
            || n.y < carousel.y - SLACK
            || n.x + n.w > carousel.x + carousel.w + SLACK
            || n.y + n.h > carousel.y + carousel.h + SLACK;
        if n.visible && n.w > 0.0 && n.h > 0.0 && outside {
            faults.push(format!("rests outside the carousel {at}: {i}, {} at {:.0},{:.0} {:.0}x{:.0}", n.cls, n.x, n.y, n.w, n.h));
        }
    }
    faults
}

/// The style sweep: the colour inside each styled control's background, near its right end, is a token of the variant.
fn sweep(scene: &mut Scene<'_>, tree: &emusen_qt_test::tree::Tree, variant: Variant) -> Vec<String> {
    let tokens = palette(variant);
    let mut points = Vec::new();
    let mut names = Vec::new();
    for control in SWEPT {
        if let Some(i) = tree.find_named(control) {
            let n = &tree.nodes[i as usize];
            if n.visible {
                points.push((n.x + n.w - 6.0, n.y + n.h / 2.0));
                names.push(control);
            }
        }
    }
    let mut faults = Vec::new();
    for (control, colour) in names.iter().zip(scene.colours_at(&points)) {
        match colour {
            Some(c) if tokens.contains(&c) => {}
            other => faults.push(format!("style sweep: {control} paints {other:?}, not a token")),
        }
    }
    faults
}

#[test]
fn qt_7_stage_0_gate_on_the_gallery() {
    let gallery = frontend().join("kit-gallery").join("qml");
    let findings = rules(&[(&gallery, true)]);
    assert!(findings.is_empty(), "qml-rules on the gallery:\n{}", findings.join("\n"));
    let complaints = qmllint(&[&gallery]);
    assert!(complaints.is_empty(), "qmllint on the gallery:\n{}", complaints.join("\n"));

    let mut harness = Harness::new();
    let mut scene = harness.load(GALLERY_URL, 1280, 800);
    let grabs = grabs();
    let mut report = Vec::new();
    let mut states = 0;
    for variant in [Variant::Dark, Variant::Light] {
        scene.set_variant(variant);
        for (form, form_name, form_value) in FORMS {
            assert!(scene.set_property("scene", "form", form_value));
            scene.set_token_form(form);
            for (page, section) in SECTIONS.iter().enumerate() {
                assert!(scene.set_property("frame", "controller.selectedSection", section));
                assert_eq!(scene.property("scene", "page"), page.to_string(), "the section turned the page");
                for (w, h) in SIZES {
                    scene.resize(w, h);
                    scene.wait(60);
                    if page == 2 {
                        finish_moonrise(&mut scene);
                        moonrise_stars(&mut scene);
                    }
                    let tree = scene.tree();
                    if let Some(first) = tree.operable().first().copied() {
                        scene.focus_item(&tree, first);
                    }
                    let mut tree = scene.tree();
                    let focus = scene.focus(&tree);
                    let name = format!("{form_name}-{variant:?}-{}-{w}x{h}", section.to_lowercase());
                    let mut faults: Vec<String> = match page {
                        0 => inside_carousel(&tree),
                        1 => sweep(&mut scene, &tree, variant),
                        _ => Vec::new(),
                    };
                    assert!(scene.grab(&grabs.join(format!("{name}.png")).display().to_string()), "grab {name}");
                    scene.sample_behind(&mut tree, f64::from(w), f64::from(h));
                    let edges = scene.record_edges(&tree);
                    faults.extend(
                        scene.audit(&tree, form, variant, focus, &edges, f64::from(w), f64::from(h)).iter().map(ToString::to_string),
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
    }
    // The question open, at 1280 x 800 on the big screen, dark: judged alone, as a player sees it.
    scene.set_variant(Variant::Dark);
    assert!(scene.set_property("scene", "form", "1"));
    scene.set_token_form(WalkForm::BigScreen);
    assert!(scene.set_property("frame", "controller.selectedSection", "Menu"));
    scene.resize(1280, 800);
    scene.key(KEY_F1, 0);
    scene.wait(400);
    assert_eq!(scene.property("question", "opened"), "true", "F1 opened the question");
    let mut tree = scene.tree();
    scene.sample_behind(&mut tree, 1280.0, 800.0);
    let popup = tree.nodes.iter().position(|n| n.popup && n.visible).expect("the question's popup item") as u32;
    let sheet = tree.root.find(emusen_kit_core::input::ElementId(popup)).expect("the popup in the tree").clone();
    let focus = scene.focus(&tree);
    assert!(scene.grab(&grabs.join("question-1280x800.png").display().to_string()));
    let edges = scene.record_edges(&tree);
    let faults = scene.audit_item(&sheet, WalkForm::BigScreen, Variant::Dark, focus, &edges, 1280.0, 800.0);
    report.extend(faults.iter().map(|f| format!("question: {f}")));
    scene.pad(UiButton::Back);
    scene.wait(400);
    assert_eq!(scene.property("question", "opened"), "false", "B closed it");
    states += 1;

    assert!(report.is_empty(), "{} states; faults:\n{}", states, report.join("\n"));
    assert_eq!(states, 2 * 3 * 3 * 4 + 1);
}
