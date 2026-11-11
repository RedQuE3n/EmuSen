//! EmuSen_Kit.md §7.100: an excuse names the finding's kind and the items it is about, read from the finding's own
//! structure and never from its words, at its size, and fails the test when it excused nothing.

use emusen_kit_core::audit::{Exempted, Fault, Side};
use emusen_kit_core::input::ElementId;
use emusen_qt_test::excuse::{names, Excuse, Excuses, Finding, SmallText, Verdict};
use emusen_qt_test::mockup::{compare_in, parse};
use emusen_qt_test::tree::Node;

fn nodes() -> Vec<Node> {
    let names = ["root", "title", "count", "ring", "value", "value", "list", "row"];
    let mut n: Vec<Node> = names.iter().map(|n| Node { name: (*n).into(), visible: true, ..Node::default() }).collect();
    n[4].text = Some("Dark".into());
    n[5].text = Some("/home/player/Roms".into());
    n
}

fn gate(fault: Fault) -> Finding {
    Finding::gate(&fault, &nodes())
}

fn excuse(finding: &'static str, item: &'static str, with: &'static str, words: &'static str) -> Excuse {
    Excuse { finding, item, with, words, size: None, why: "test" }
}

const SMALL_COUNT: Excuse = Excuse { finding: "too small to read", item: "count", with: "", words: "", size: Some((1280, 720)), why: "test" };

#[test]
fn kit_7_100_an_excuse_names_its_item_its_finding_and_its_size() {
    let small = |id| gate(Fault::TooSmall { item: ElementId(id), capitals: 10.72 });
    let mut e = Excuses::new(&[SMALL_COUNT]);
    assert!(!e.excuses(&small(1), (1280, 720)), "another item");
    assert!(!e.excuses(&gate(Fault::LowContrast { item: ElementId(2), ratio: 3.1, needed: 4.5 }), (1280, 720)), "another finding");
    assert!(!e.excuses(&small(2), (1280, 800)), "another size");
    assert!(e.stale().len() == 1, "nothing excused yet");
    assert!(e.excuses(&small(2), (1280, 720)));
    assert!(e.stale().is_empty());
}

#[test]
fn kit_7_100_a_mockup_s_own_figure_is_never_read_as_an_item() {
    // The tree's item 3 is the ring: a mockup line about the title whose figures hold a 3 names the title alone.
    let m = parse("mockup 1\ncanvas C 1280 800\nelement title y=3\n").unwrap();
    let mut n = nodes();
    n[1].y = 88.0;
    let found: Vec<Finding> = compare_in(&m, &n, 1.0, &|_| emusen_kit_core::tokens::Colour::hex("#000000")).iter().map(|d| Finding::mockup(d, &n)).collect();
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].line, "mockup: title: Y 88.0, the mockup's 3");
    assert_eq!(names(&excuse("Y", "ring", "", ""), &found[0]), Verdict::Other, "the mockup's 3 is no item");
    assert_eq!(names(&excuse("Y", "title", "", ""), &found[0]), Verdict::Names);
    assert_eq!(names(&excuse("X", "title", "", ""), &found[0]), Verdict::Other, "another edge");
}

#[test]
fn kit_7_100_an_edge_s_cut_names_the_item_cut_not_only_the_scroller() {
    let cut = |item| gate(Fault::CutAtEdge { item: ElementId(item), area: ElementId(6), side: Side::Bottom });
    let kind = "cut at an edge with no fade";
    assert_eq!(names(&excuse(kind, "list", "", ""), &cut(7)), Verdict::Other, "the scroller alone excuses nothing cut at it");
    assert_eq!(names(&excuse(kind, "row", "list", ""), &cut(7)), Verdict::Names);
    assert_eq!(names(&excuse(kind, "row", "list", ""), &cut(2)), Verdict::Other, "another item cut at the same scroller");
    let past = gate(Fault::PastItsLimit { item: ElementId(7), limit: Some(ElementId(6)) });
    assert_eq!(names(&excuse("past its scroller or clip", "list", "", ""), &past), Verdict::Other, "the limit alone");
    assert_eq!(names(&excuse("past its scroller or clip", "row", "list", ""), &past), Verdict::Names);
}

#[test]
fn kit_7_100_an_overlap_s_excuse_names_both_items() {
    let overlap = |a, b| gate(Fault::Overlap { a: ElementId(a), b: ElementId(b) });
    let kind = "overlap";
    assert_eq!(names(&excuse(kind, "ring", "", ""), &overlap(1, 3)), Verdict::Other, "one side alone excuses none of its overlaps");
    assert_eq!(names(&excuse(kind, "ring", "title", ""), &overlap(1, 3)), Verdict::Names, "either way round");
    assert_eq!(names(&excuse(kind, "ring", "title", ""), &overlap(3, 1)), Verdict::Names);
    assert_eq!(names(&excuse(kind, "ring", "title", ""), &overlap(3, 2)), Verdict::Other, "the ring over another");
    assert_eq!(names(&excuse(kind, "count", "", ""), &gate(Fault::TooSmall { item: ElementId(2), capitals: 3.0 })), Verdict::Other);
}

#[test]
fn kit_7_100_a_shared_name_with_no_words_excuses_none_and_says_why() {
    let cut = |id| gate(Fault::Ellipsis { item: ElementId(id) });
    assert_eq!(names(&excuse("cut with an ellipsis", "value", "", ""), &cut(4)), Verdict::Ambiguous);
    let mut e = Excuses::new(&[excuse("cut with an ellipsis", "value", "", "")]);
    assert!(!e.excuses(&cut(4), (1280, 800)) && !e.excuses(&cut(5), (1280, 800)), "neither row's value");
    let stale = e.stale();
    std::mem::forget(e);
    assert_eq!(stale, ["cut with an ellipsis on value: excused nothing, its item's name shared by others and no words given"]);
}

#[test]
fn kit_7_100_words_tell_one_item_from_others_of_its_name() {
    let cut = |id| gate(Fault::Ellipsis { item: ElementId(id) });
    let rom = excuse("cut with an ellipsis", "value", "", "/Roms");
    assert_eq!(names(&rom, &cut(4)), Verdict::Other, "another row's value");
    assert_eq!(names(&rom, &cut(5)), Verdict::Names);
}

#[test]
fn kit_7_100_an_exemption_is_excused_by_its_own_kind_and_item() {
    let spare = Finding::exempted(&Exempted::Spare { item: ElementId(1), worst: 3.0, needed: 4.5 }, &nodes());
    assert_eq!(spare.line, "gate: low contrast in the spare tenth: 1 at 3.00:1, needs 4.5:1");
    assert_eq!(names(&excuse("low contrast in the spare tenth", "title", "", ""), &spare), Verdict::Names);
    assert_eq!(names(&excuse("low contrast", "title", "", ""), &spare), Verdict::Other, "the rule's own finding is another");
}

#[test]
#[should_panic(expected = "1 stale excuses:\ntoo small to read on count at 1280x720: excused nothing")]
fn kit_7_100_an_excuse_that_excused_nothing_fails_the_test() {
    let _e = Excuses::new(&[SMALL_COUNT]);
}

#[test]
fn kit_7_100_no_excuse_excuses_nothing_and_passes() {
    let mut e = Excuses::none();
    assert!(!e.excuses(&gate(Fault::TooSmall { item: ElementId(2), capitals: 10.72 }), (1280, 720)));
}

#[test]
#[should_panic(expected = "1 stale small-text exceptions:\nSMALL_TEXT_EXCEPTIONS' moonriseCaption: let nothing pass")]
fn kit_7_100_a_small_text_exception_that_let_nothing_pass_fails_the_gate() {
    let _seen = SmallText::new();
}

#[test]
fn kit_7_100_a_small_text_exception_seen_letting_its_item_pass_is_not_stale() {
    let mut n = nodes();
    n[2].name = "moonriseCaption".into();
    let mut seen = SmallText::new();
    seen.saw(&[Exempted::Spare { item: ElementId(2), worst: 3.0, needed: 4.5 }], &n);
    assert_eq!(seen.stale().len(), 1, "a spare-tenth exemption is not the small text's");
    seen.saw(&[Exempted::SmallText { item: ElementId(2), capitals: 10.0 }], &n);
    assert!(seen.stale().is_empty());
}
