//! EmuSen_Kit.md §7.100: an excuse names the one item and the finding it excuses, at its size, and fails the test when it
//! excused nothing.

use emusen_qt_test::excuse::{names, Excuse, Excuses};
use emusen_qt_test::tree::Node;

fn nodes() -> Vec<Node> {
    let mut n: Vec<Node> = ["root", "title", "count", "ring", "value", "value"].iter().map(|n| Node { name: (*n).into(), ..Node::default() }).collect();
    n[4].text = Some("Dark".into());
    n[5].text = Some("/home/player/Roms".into());
    n
}

const SMALL_COUNT: Excuse = Excuse { finding: "too small to read", item: "count", words: "", size: Some((1280, 720)), why: "test" };

#[test]
fn kit_7_100_an_excuse_names_its_item_its_finding_and_its_size() {
    let n = nodes();
    let mut e = Excuses::new(&[SMALL_COUNT]);
    assert!(!e.excuses("gate: too small to read: 1 with capitals of 10.72", &n, (1280, 720)), "another item");
    assert!(!e.excuses("gate: low contrast: 2 at 3.10:1, needs 4.5:1", &n, (1280, 720)), "another finding");
    assert!(!e.excuses("gate: too small to read: 2 with capitals of 10.72", &n, (1280, 800)), "another size");
    assert!(e.stale().len() == 1, "nothing excused yet");
    assert!(e.excuses("gate: too small to read: 2 with capitals of 10.72", &n, (1280, 720)));
    assert!(e.stale().is_empty());
}

#[test]
fn kit_7_100_the_gate_s_numbers_and_the_mockup_s_names_are_read() {
    let n = nodes();
    assert!(names("mockup: ring: Y 88.0, the mockup's 96", "Y", "ring", "", &n));
    assert!(!names("mockup: title: Y 88.0, the mockup's 96", "Y", "ring", "", &n));
    assert!(names("gate: overlap: 1 and 3", "overlap", "ring", "", &n), "an overlap's second item");
    assert!(names("gate: past its scroller or clip 1: 2", "past its scroller", "count", "", &n), "what passes the limit");
    assert!(!names("gate: low contrast: 1 at 3.00:1, needs 4.5:1", "low contrast", "ring", "", &n), "a ratio is no item: 3");
    assert!(!names("gate: scrolls in too little room: 1 at 3", "scrolls in too little room", "ring", "", &n), "a height is no item");
}

#[test]
fn kit_7_100_words_tell_one_item_from_others_of_its_name() {
    let n = nodes();
    assert!(!names("gate: cut with an ellipsis: 4", "cut with an ellipsis", "value", "/Roms", &n), "another row's value");
    assert!(names("gate: cut with an ellipsis: 5", "cut with an ellipsis", "value", "/Roms", &n));
}

#[test]
#[should_panic(expected = "1 stale excuses:\ntoo small to read on count at 1280x720: excused nothing")]
fn kit_7_100_an_excuse_that_excused_nothing_fails_the_test() {
    let _e = Excuses::new(&[SMALL_COUNT]);
}

#[test]
fn kit_7_100_no_excuse_excuses_nothing_and_passes() {
    let mut e = Excuses::none();
    assert!(!e.excuses("gate: too small to read: 2 with capitals of 10.72", &nodes(), (1280, 720)));
}
