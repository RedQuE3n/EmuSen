// EmuSen_Kit.md §7.102: every item a rule's exemption lets pass is reported by itself, and only where the strict rule fails it.

use emusen_kit_core::audit::{Axes, Edges, Exempted, Item, Kind, Legibility, Rect, Text, exempted};
use emusen_kit_core::input::ElementId;
use emusen_kit_core::tokens::{Colour, ColourToken, Variant, colour};

const PANEL: Rect = Rect::new(0.0, 0.0, 1280.0, 800.0);

fn words(id: u32, x: f64, ink: Colour, behind: Vec<Colour>) -> Item {
    Item::new(id, Kind::Words, Rect::new(x, 300.0, 200.0, 30.0)).legible(Legibility { ink, size: 16.0, bold: false, behind })
}

#[test]
fn kit_7_102_words_passing_by_the_spare_tenth_are_named() {
    let (black, white) = (Colour::hex("#000000"), Colour::hex("#FFFFFF"));
    let mut sky = vec![black; 19];
    sky.push(white);
    let state = Item::new(1, Kind::Panel, PANEL).child(words(2, 10.0, white, sky)).child(words(3, 300.0, white, vec![black; 20]));
    let found = exempted(&state, &[], 0.0);
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
    assert!(matches!(exempted(&strip(1180.0), &[], 0.0)[..], [Exempted::Fading { item: ElementId(3), .. }]), "crossing the faded edge");
    assert!(exempted(&strip(600.0), &[], 0.0).is_empty(), "inside the strip the rule holds it, and nothing is exempted");
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
    assert_eq!(exempted(&state, &declared, 0.0), vec![Exempted::Scene { item: ElementId(91), colour: stray }]);
}

#[test]
fn kit_7_102_a_named_small_text_under_the_floor_is_named() {
    let small = |name: &str| {
        Item::new(1, Kind::Panel, PANEL).child(
            Item::new(2, Kind::Words, Rect::new(10.0, 10.0, 200.0, 20.0))
                .named(name)
                .words(Text { cap_height: 9.0, width: 100.0, height: 18.0, ellipsis: false, data: false, own: None }),
        )
    };
    assert!(matches!(exempted(&small("moonriseCaption"), &[], 11.64)[..], [Exempted::SmallText { item: ElementId(2), .. }]));
    assert!(exempted(&small("other"), &[], 11.64).is_empty(), "another item is the floor's finding, not an exemption");
    assert!(exempted(&small("moonriseCaption"), &[], 0.0).is_empty(), "no floor, nothing exempted");
}

#[test]
fn kit_7_102_an_exemption_reads_as_one_line() {
    let line = Exempted::Scene { item: ElementId(91), colour: Colour::hex("#007ACC") }.to_string();
    assert_eq!(line, "a colour outside the palette in a scene: 91 #007ACC");
}
