// EmuSen_Kit.md §7.102: every item a rule's exemption lets pass is reported by itself, and only where the strict rule fails it.

use emusen_kit_core::audit::{ACCEPTED, MOONRISE_SOURCE, Axes, Class, Edges, Exempted, Fault, Item, Kind, Legibility, Rect, SMALL_TEXT_EXCEPTIONS, Text, accepted, exempted, fit};
use emusen_kit_core::input::ElementId;
use emusen_kit_core::tokens::{Colour, ColourToken, Variant, colour};

const PANEL: Rect = Rect::new(0.0, 0.0, 1280.0, 800.0);

fn words(id: u32, x: f64, ink: Colour, behind: Vec<Colour>) -> Item {
    Item::new(id, Kind::Words, Rect::new(x, 300.0, 200.0, 30.0)).legible(Legibility { ink, size: 16.0, bold: false, behind, disabled: false })
}

#[test]
fn kit_7_102_words_passing_by_the_spare_tenth_are_named() {
    let (black, white) = (Colour::hex("#000000"), Colour::hex("#FFFFFF"));
    let mut sky = vec![black; 19];
    sky.push(white);
    let state = Item::new(1, Kind::Panel, PANEL).child(words(2, 10.0, white, sky)).child(words(3, 300.0, white, vec![black; 20]));
    let found = exempted(&state, Some(&[]), 0.0);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(matches!(found[0], Exempted::Spare { item: ElementId(2), .. }), "the words over a star alone: {found:?}");
}

#[test]
fn kit_7_102_words_under_their_contrast_at_a_faded_edge_are_named() {
    let (faint, white) = (Colour::hex("#949494"), Colour::hex("#FFFFFF"));
    let strip = |x: f64| {
        Item::new(1, Kind::Panel, PANEL).child(
            Item::new(2, Kind::Scroller, Rect::new(0.0, 200.0, 1280.0, 300.0))
                .scrolling(Axes { horizontal: true, vertical: false })
                .fading(Edges { left: true, right: true, ..Edges::default() })
                .child(words(3, x, faint, vec![white])),
        )
    };
    assert!(matches!(exempted(&strip(1180.0), Some(&[]), 0.0)[..], [Exempted::Fading { item: ElementId(3), .. }]), "crossing the faded edge");
    assert!(exempted(&strip(600.0), Some(&[]), 0.0).is_empty(), "inside the strip the rule holds it, and nothing is exempted");
}

#[test]
fn kit_7_102_a_scene_s_colour_outside_the_palette_is_named() {
    let declared = [colour(Variant::Dark, ColourToken::Surface)];
    let stray = Colour::hex("#007ACC");
    let state = Item::new(1, Kind::Panel, PANEL).child(
        Item::new(90, Kind::Panel, PANEL)
            .scene()
            .child(Item::new(91, Kind::Picture, Rect::new(0.0, 0.0, 100.0, 100.0)).paints(&[stray, declared[0]])),
    );
    assert_eq!(exempted(&state, Some(&declared), 0.0), vec![Exempted::Scene { item: ElementId(91), colour: stray }]);
    assert!(exempted(&state, None, 0.0).is_empty(), "where no palette rule runs, a scene's colour is exempted from none");
}

#[test]
fn kit_7_102_a_named_small_text_under_the_floor_is_named() {
    // Each item is written in a file: the kit's Moonrise.qml, or a theme's Home with a wrapper it names "moonrise".
    let small = |file: &str, name: &str| {
        let source = if file == "moonrise" { MOONRISE_SOURCE } else { "file:///themes/t/screens/Home.qml" };
        Item::new(1, Kind::Panel, PANEL).named("moonrise").written_in(source).child(
            Item::new(2, Kind::Words, Rect::new(10.0, 10.0, 200.0, 20.0))
                .named(name)
                .written_in(source)
                .words(Text { cap_height: 9.0, width: 100.0, height: 18.0, ellipsis: false, data: false, own: None }),
        )
    };
    assert!(matches!(exempted(&small("moonrise", "moonriseCaption"), Some(&[]), 11.64)[..], [Exempted::SmallText { item: ElementId(2), .. }]));
    assert!(exempted(&small("moonrise", "other"), Some(&[]), 11.64).is_empty(), "another item is the floor's finding, not an exemption");
    assert!(exempted(&small("Home", "moonriseCaption"), Some(&[]), 11.64).is_empty(), "a theme's caption under a wrapper named moonrise is the floor's finding");
    assert!(exempted(&small("moonrise", "moonriseCaption"), Some(&[]), 0.0).is_empty(), "no floor, nothing exempted");
}

#[test]
fn kit_7_109_data_cut_with_an_ellipsis_is_an_exemption_and_a_fixed_word_a_fault() {
    let cut = |data: bool| {
        Item::new(1, Kind::Panel, PANEL).child(
            Item::new(2, Kind::Words, Rect::new(10.0, 10.0, 200.0, 20.0))
                .named("title")
                .words(Text { cap_height: 14.0, width: 200.0, height: 18.0, ellipsis: true, data, own: None }),
        )
    };
    assert_eq!(exempted(&cut(true), None, 11.64), vec![Exempted::Elided { item: ElementId(2) }]);
    assert_eq!(Exempted::Elided { item: ElementId(2) }.to_string(), "data cut with an ellipsis: 2");
    assert!(exempted(&cut(false), None, 11.64).is_empty(), "a fixed word cut is the fit rule's fault, no exemption");
    assert_eq!(fit(&cut(false), PANEL, 1.0, 11.64), vec![Fault::Ellipsis { item: ElementId(2) }]);
    assert!(fit(&cut(true), PANEL, 1.0, 11.64).is_empty(), "data is left to the table (R-20261111-44)");
}

#[test]
fn kit_7_102_an_exemption_reads_as_one_line() {
    let line = Exempted::Scene { item: ElementId(91), colour: Colour::hex("#007ACC") }.to_string();
    assert_eq!(line, "a colour outside the palette in a scene: 91 #007ACC");
}

#[test]
fn kit_7_100_a_finding_names_its_kind_and_its_items_apart_from_its_numbers() {
    use emusen_kit_core::audit::{Fault, Side};
    let cut = Fault::CutAtEdge { item: ElementId(7), area: ElementId(3), side: Side::Bottom };
    assert_eq!((cut.kind(), cut.items()), ("cut at an edge with no fade", vec![ElementId(7), ElementId(3)]), "the item first, its area after");
    let past = Fault::PastItsLimit { item: ElementId(9), limit: Some(ElementId(2)) };
    assert_eq!((past.kind(), past.items()), ("past its scroller or clip", vec![ElementId(9), ElementId(2)]));
    let low = Fault::LowContrast { item: ElementId(4), ratio: 3.0, needed: 4.5 };
    assert_eq!((low.kind(), low.items()), ("low contrast", vec![ElementId(4)]), "a ratio is no item");
    let room = Fault::ScrollsInTooLittleRoom { item: ElementId(5), height: 3.0 };
    assert_eq!(room.items(), vec![ElementId(5)], "a height is no item");
    assert!(low.to_string().starts_with(low.kind()) && past.to_string().starts_with(past.kind()));
    let spare = Exempted::Spare { item: ElementId(6), worst: 3.0, needed: 4.5 };
    assert_eq!((spare.kind(), spare.item()), ("low contrast in the spare tenth", ElementId(6)));
    assert!(spare.to_string().starts_with(spare.kind()));
}

#[test]
fn kit_7_102_the_table_holds_the_classes_the_tester_accepted_and_no_other() {
    let rows: Vec<(Class, &str)> = ACCEPTED.iter().map(|a| (a.class, a.ruling)).collect();
    assert_eq!(
        rows,
        [
            (Class::RomFolderCutAtStart, "R-20261111-36"),
            (Class::SpareTenth, "R-20261111-37"),
            (Class::FadedEdge, "R-20261111-38"),
            (Class::SceneColour, "R-20261111-39"),
            (Class::MoonriseCaption, "R-20261111-40"),
            (Class::CpuConversionLine, "R-20261111-41"),
            (Class::DimmedAndMoving, "R-20261111-43"),
            (Class::DataWordsElided, "R-20261111-44"),
            (Class::MainThreadLineInTests, "R-20261111-110"),
        ]
    );
    assert!(ACCEPTED.iter().all(|a| !a.covers.is_empty()), "each row says what it covers");
    let lines: Vec<(Class, &[&str])> = ACCEPTED.iter().filter(|a| !a.lines.is_empty()).map(|a| (a.class, a.lines)).collect();
    assert_eq!(
        lines,
        [
            (Class::CpuConversionLine, &["No RHI backend. Using CPU conversion."][..]),
            (Class::MainThreadLineInTests, &["QApplication was not created in the main() thread"][..]),
        ],
        "the log lines a row accepts are its own, word for word: a line added under a class changes this table"
    );
}

#[test]
fn kit_7_102_each_exemption_is_of_its_class_and_accepted_by_its_ruling() {
    let id = ElementId(1);
    let found = [
        (Exempted::Spare { item: id, worst: 3.0, needed: 4.5 }, "R-20261111-37"),
        (Exempted::Fading { item: id, ratio: 2.0, needed: 4.5 }, "R-20261111-38"),
        (Exempted::Scene { item: id, colour: Colour::hex("#2E8C96") }, "R-20261111-39"),
        (Exempted::SmallText { item: id, capitals: 10.0, class: SMALL_TEXT_EXCEPTIONS[0].2 }, "R-20261111-40"),
        (Exempted::Elided { item: id }, "R-20261111-44"),
        (Exempted::Disabled { item: id, ratio: 2.0, needed: 4.5 }, "R-20261111-43"),
    ];
    for (e, ruling) in found {
        assert_eq!(e.accepted().map(|a| a.ruling), Some(ruling), "{e}");
        assert_eq!(accepted(e.class()).map(|a| a.ruling), Some(ruling));
    }
}

#[test]
fn kit_7_109_a_disabled_control_s_words_under_their_contrast_are_named_not_faults() {
    let (faint, white) = (Colour::hex("#C8C8C8"), Colour::hex("#FFFFFF"));
    let state = |disabled: bool| {
        let words = Item::new(2, Kind::Words, Rect::new(10.0, 300.0, 200.0, 30.0))
            .legible(Legibility { ink: faint, size: 16.0, bold: false, behind: vec![white; 20], disabled });
        Item::new(1, Kind::Panel, PANEL).child(words)
    };
    assert!(matches!(emusen_kit_core::audit::contrast(&state(false))[..], [Fault::LowContrast { item: ElementId(2), .. }]), "enabled, a fault");
    assert!(exempted(&state(false), None, 0.0).is_empty());
    assert!(emusen_kit_core::audit::contrast(&state(true)).is_empty(), "disabled, left to exempted");
    let found = exempted(&state(true), None, 0.0);
    assert!(matches!(found[..], [Exempted::Disabled { item: ElementId(2), .. }]), "{found:?}");
    assert_eq!(found[0].accepted().map(|a| a.ruling), Some("R-20261111-43"));
    assert!(found[0].to_string().starts_with("low contrast on a disabled control: 2 at "));
    let passing = Item::new(1, Kind::Panel, PANEL).child(
        Item::new(2, Kind::Words, Rect::new(10.0, 300.0, 200.0, 30.0))
            .legible(Legibility { ink: Colour::hex("#000000"), size: 16.0, bold: false, behind: vec![white; 20], disabled: true }),
    );
    assert!(exempted(&passing, None, 0.0).is_empty(), "a disabled control's words that pass are nothing to name");
}
