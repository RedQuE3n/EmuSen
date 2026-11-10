// The controller drawing (EmuSen_Kit.md §7.62): a drawing file's items, the regions a host's controls name, the generic
// drawing derived from the controls alone, the labels laid out beside the drawing, what is painted and the walk.

use emusen_kit_core::controller::{drawing_for, generic, lay_out, paints, walk, words, Drawing, Lit, Shape, Side};
use serde_json::json;

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}

const PAD: &str = r##"{
  "name": "A test pad",
  "design": [1000, 500],
  "items": [
    {"region": "L", "shape": {"box": [100, 20, 200, 40, 10]}, "fill": "#808080", "side": "top"},
    {"part": {"union": [{"circle": [250, 260, 200]}, {"circle": [750, 260, 200]}]}, "fill": "#C0C0C0", "stroke": "#404040", "width": 5},
    {"dpad": [250, 260, 60, 90], "controls": ["Up", "Down", "Left", "Right"], "fill": "#202020", "arrow": "#606060"},
    {"round": [700, 260, 36], "region": "B", "name": "Cross", "colour": "#3050C0", "caption": "B"},
    {"round": [800, 260, 36], "region": "A", "colour": "#C03030"},
    {"region": "Start", "shape": {"pill": [500, 300, 70, 22, -35]}, "fill": "#404040"},
    {"word": "START", "at": [500, 340], "size": 17, "colour": "#505050"},
    {"stick": [500, 420, 40, 24, 12], "controls": ["LeftY-", "LeftY+", "LeftX-", "LeftX+"], "click": "L3"}
  ]
}"##;

#[test]
fn kit_7_62_a_drawing_files_items_become_its_layers_and_regions_named_by_controls() {
    let d = Drawing::parse(PAD).unwrap();
    assert_eq!((d.name.as_str(), d.width, d.height), ("A test pad", 1000.0, 500.0));
    let controls: Vec<&str> = d.regions.iter().map(|r| r.control.as_str()).collect();
    assert_eq!(controls, ["L", "Up", "Down", "Left", "Right", "B", "A", "Start", "LeftY-", "LeftY+", "LeftX-", "LeftX+", "L3"]);
    assert_eq!(d.region("B").unwrap().name, "Cross", "a region's name is the drawing's; its control the host's");
    assert_eq!(d.region("B").unwrap().stroke.as_deref(), Some("#1F347C"), "a round button's edge is its colour darkened");
    assert!(d.region("Up").unwrap().arm && d.region("Up").unwrap().mark.is_some(), "a direction pad's arm and its arrow");
    assert_eq!(d.sticks.len(), 1);
    assert_eq!(d.body.len(), 2, "the shell and the direction pad's cross");
    assert_eq!(words(&d).iter().map(|w| w.text.as_str()).collect::<Vec<_>>(), ["START", "B"]);
    assert!(Drawing::parse(r#"{"design": [0, 0]}"#).is_err());
    assert!(Drawing::parse(r#"{"design": [10, 10], "items": [{"nothing": 1}]}"#).is_err());
}

#[test]
fn kit_7_62_shapes_are_path_data_and_a_union_is_stroked_before_it_is_filled() {
    assert_eq!(Shape::Circle { cx: 10.0, cy: 10.0, r: 5.0 }.svg(), "M 5 10 A 5 5 0 1 1 15 10 A 5 5 0 1 1 5 10 Z");
    let turned = Shape::parse(&json!({"box": [0, 0, 10, 10, 0], "turn": [90, 0, 0]})).unwrap();
    let b = turned.bounds();
    assert!((b.x + 10.0).abs() < 1e-9 && b.w > 9.99 && b.y.abs() < 1e-9, "{b:?}");
    let d = Drawing::parse(PAD).unwrap();
    let painted = paints(&d, &|_| Lit::default(), &|_| (0.0, 0.0), "#4C9AE8");
    let shell: Vec<_> = painted.iter().take(4).collect();
    assert!(shell[0].fill.is_none() && shell[1].fill.is_none(), "both circles stroked first");
    assert!(shell[2].stroke.is_none() && shell[3].stroke.is_none(), "then both filled");
}

#[test]
fn kit_7_62_the_focused_region_is_ringed_a_held_one_lit_and_an_arm_washed() {
    let d = Drawing::parse(PAD).unwrap();
    let a = d.regions.iter().position(|r| r.control == "A").unwrap();
    let up = d.regions.iter().position(|r| r.control == "Up").unwrap();
    let none = paints(&d, &|_| Lit::default(), &|_| (0.0, 0.0), "#4C9AE8").len();
    let focused = paints(&d, &|i| Lit { focused: i == a, held: false }, &|_| (0.0, 0.0), "#4C9AE8");
    assert_eq!(focused.len(), none + 1);
    assert!(focused.iter().any(|p| p.stroke.as_deref() == Some("#4C9AE8") && p.width == 14.0));
    let held = paints(&d, &|i| Lit { focused: false, held: i == a }, &|_| (0.0, 0.0), "#4C9AE8");
    assert!(held.iter().any(|p| p.fill.as_deref() == Some("#4C9AE8")), "held: the accent fills it");
    assert!(held.iter().any(|p| p.stroke.as_deref() == Some("#5A4C9AE8")), "and a halo of the accent at 90");
    let arm = paints(&d, &|i| Lit { focused: i == up, held: false }, &|_| (0.0, 0.0), "#4C9AE8");
    assert!(arm.iter().any(|p| p.fill.as_deref() == Some("#824C9AE8")), "an arm is washed, not ringed");
}

#[test]
fn kit_7_62_a_drawing_stands_for_a_controller_only_if_it_has_every_declared_control() {
    let (d, generic_used) = drawing_for(Some(PAD), &names(&["B", "A", "Start"]));
    assert!(!generic_used && d.name == "A test pad");
    let (d, generic_used) = drawing_for(Some(PAD), &names(&["B", "A", "Select"]));
    assert!(generic_used && d.region("Select").is_some(), "Select has no region on the file's drawing");
    let (_, generic_used) = drawing_for(None, &names(&["B"]));
    assert!(generic_used, "no file");
    let (_, generic_used) = drawing_for(Some("{"), &names(&["B"]));
    assert!(generic_used, "a file that does not read");
}

#[test]
fn kit_7_62_the_generic_drawing_comes_from_the_declared_controls_alone() {
    let d = generic(&names(&["B", "A", "Up", "LeftY-", "Coin", "Turbo"]));
    for c in ["B", "A", "Up", "LeftY-", "Coin", "Turbo"] {
        assert!(d.region(c).is_some(), "{c} has a region");
    }
    let coin = d.region("Coin").unwrap();
    assert_eq!((coin.name.as_str(), coin.side), ("Coin", Some(Side::Bottom)), "a control the RetroPad lacks: a button under the shell");
    assert!(coin.shape.bounds().y > 528.0, "under the shell's middle, clear of its buttons");
    assert_eq!(generic(&names(&["B"])).regions.len(), generic(&names(&["B", "A"])).regions.len(), "the pad's own regions do not grow");
}

#[test]
fn kit_7_62_labels_stand_beside_the_drawing_without_overlapping_and_leaders_reach_them() {
    let d = Drawing::parse(PAD).unwrap();
    let controls = names(&["L", "Up", "Down", "Left", "Right", "B", "A", "Start", "LeftY-", "L3"]);
    let widths = vec![90.0; controls.len()];
    let laid = lay_out(&d, &controls, &widths, 30.0, 656.0, 380.0);
    assert!(laid.fits);
    assert_eq!(laid.sides[0], Side::Top, "the file puts L's label above");
    assert_eq!(laid.sides[1], Side::Left);
    assert_eq!(laid.sides[6], Side::Right);
    for (i, a) in laid.labels.iter().enumerate() {
        assert!(a.x >= 0.0 && a.y >= 0.0 && a.x + a.w <= 656.0 + 1e-9 && a.y + a.h <= 380.0 + 1e-9, "label {i} inside the box: {a:?}");
        assert!(a.x + a.w <= laid.drawing.x + 1e-9 || a.x >= laid.drawing.x + laid.drawing.w - 1e-9 || a.y + a.h <= laid.drawing.y + 1e-9, "label {i} beside the drawing");
        for (j, b) in laid.labels.iter().enumerate().skip(i + 1) {
            let apart = a.x + a.w <= b.x || b.x + b.w <= a.x || a.y + a.h <= b.y || b.y + b.h <= a.y;
            assert!(apart, "labels {i} and {j} overlap: {a:?} {b:?}");
        }
    }
    for (i, label) in laid.labels.iter().enumerate().filter(|(i, _)| matches!(laid.sides[*i], Side::Left | Side::Right)) {
        assert!(label.y >= 30.0, "label {i}'s column stands below the row of labels above: {label:?}");
    }
    assert_eq!(laid.leaders.matches('M').count(), controls.len());
    assert!(laid.leader(1).starts_with("M "));
    // Too many labels for the height: it says so.
    let many: Vec<String> = (0..20).map(|i| format!("L{i}")).collect();
    let crowded = lay_out(&generic(&many), &many, &vec![90.0; 20], 30.0, 656.0, 380.0);
    assert!(!crowded.fits);
}

#[test]
fn kit_7_62_the_walk_steps_the_controls_round_from_either_end() {
    assert_eq!(walk(0, 1, 3), 1);
    assert_eq!(walk(2, 1, 3), 0);
    assert_eq!(walk(0, -1, 3), 2);
    assert_eq!(walk(0, 1, 0), 0);
}

#[test]
fn kit_7_62_a_column_of_labels_stands_below_the_row_above_it() {
    let d = Drawing::parse(r#"{"design": [400, 400], "items": [
        {"region": "L", "shape": {"box": [150, 0, 100, 20]}, "side": "top"},
        {"region": "Select", "shape": {"box": [0, 0, 30, 30]}},
        {"region": "Start", "shape": {"box": [370, 0, 30, 30]}}
    ]}"#)
    .unwrap();
    let laid = lay_out(&d, &names(&["L", "Select", "Start"]), &[200.0, 120.0, 120.0], 30.0, 656.0, 380.0);
    assert!(laid.fits);
    assert!(laid.labels[1].y >= 32.0 && laid.labels[2].y >= 32.0, "{:?}", laid.labels);
}

#[test]
fn kit_7_62_a_full_column_does_not_climb_into_the_row_above() {
    let mut items = vec![r#"{"region": "L", "shape": {"box": [150, 0, 100, 20]}, "side": "top"}"#.to_owned()];
    let mut controls = vec!["L".to_owned()];
    for i in 0..11 {
        items.push(format!(r#"{{"region": "C{i}", "shape": {{"box": [0, 380, 20, 20]}}, "side": "left"}}"#));
        controls.push(format!("C{i}"));
    }
    let d = Drawing::parse(&format!(r#"{{"design": [400, 400], "items": [{}]}}"#, items.join(","))).unwrap();
    let laid = lay_out(&d, &controls, &vec![100.0; 12], 30.0, 656.0, 380.0);
    let below = laid.labels.iter().zip(&laid.sides).filter(|(_, s)| **s == Side::Left).all(|(b, _)| b.y >= 32.0);
    assert!(!laid.fits || below, "a column that fits stands below the row above: {:?}", laid.labels);
}

#[test]
fn kit_7_62_a_stick_s_knob_stands_where_its_directions_push_it() {
    let controls = names(&["A", "LeftY-", "LeftY+", "LeftX-", "LeftX+"]);
    let drawing = generic(&controls);
    let stick = drawing.sticks.first().expect("the generic drawing's left stick");
    let push = |c: &str| match c {
        "LeftX+" => 0.75,
        "LeftY-" => 0.5,
        "LeftY+" => 2.0,
        _ => 0.0,
    };
    // Right less left, down less up, each push held to 0 to 1.
    assert_eq!(emusen_kit_core::controller::knob(stick, &push), (0.75, 0.5));
    assert_eq!(emusen_kit_core::controller::knob(stick, &|_| 0.0), (0.0, 0.0), "at rest with nothing pushed");
}
