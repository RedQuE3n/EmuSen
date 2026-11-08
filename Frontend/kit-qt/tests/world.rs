// EmuSen_Kit.md §7.5: a body's rows are kit-core's shapes, back to front, as a Shapes delegate reads them.

use emusen_kit_core::tokens::Colour;
use emusen_kit_core::world::{self, Paint, Shape, Stroke, WorldRing, WorldShape};
use emusen_kit_qt::world::{body, pad_stops, row, rows, CLEAR, STOPS};

#[test]
fn kit_7_5_every_shape_of_a_body_is_one_row_in_order() {
    let body = body(WorldShape::Planet, 0.3, 0.5, 0.9, WorldRing::Big, 2);
    let shapes = world::geometry(&body, 38.0, true);
    let rows = rows(&body, 38.0, true);
    assert_eq!(rows.len(), shapes.len());
    assert_eq!(shapes.len(), 12, "ring back, disc, three craters, two bands, cap, ring front, highlight, two moons");
    for (row, shape) in rows.iter().zip(&shapes) {
        assert_eq!(row.opacity, shape.opacity);
        let already_closed = shape.points.first() == shape.points.last();
        assert_eq!(row.points.len(), shape.points.len() + usize::from(shape.closed && !already_closed));
        if shape.closed {
            assert_eq!(row.points.first(), row.points.last(), "a closed outline repeats its first point");
        }
    }
}

#[test]
fn kit_7_5_the_disc_is_a_radial_gradient_padded_to_four_stops() {
    let disc = rows(&world::WorldBody::PLAIN, 200.0, false).into_iter().find(|r| r.gradient.is_some()).expect("the disc");
    let gradient = disc.gradient.unwrap();
    assert_eq!(disc.fill, CLEAR, "a gradient row paints no solid fill");
    assert_eq!(gradient.centre, [6.0 + 0.36 * 88.0, 6.0 + 0.32 * 88.0]);
    assert_eq!(gradient.radius, 0.74 * 88.0);
    assert_eq!(gradient.stops[0].0, 0.0);
    assert_eq!(gradient.stops[1].0, 0.48);
    assert_eq!(gradient.stops[2].0, 1.0);
    assert_eq!(gradient.stops[3], gradient.stops[2], "the fourth stop repeats the third");
    assert_eq!(disc.stroke_width, 0.0);
}

#[test]
fn kit_7_5_solid_stroked_and_bare_shapes() {
    let ink = Colour::hex("#4C9AE8");
    let filled = row(&Shape::filled(world::circle(10.0, 10.0, 5.0), Paint::Solid(ink)));
    assert_eq!(filled.fill, ink);
    assert!(filled.gradient.is_none());
    assert_eq!((filled.stroke, filled.stroke_width), (CLEAR, 0.0));
    let stroked = row(&Shape::stroked(vec![[0.0, 0.0], [10.0, 10.0]], false, Stroke { colour: ink, width: 3.0 }));
    assert_eq!(stroked.fill, CLEAR);
    assert_eq!((stroked.stroke, stroked.stroke_width), (ink, 3.0));
    assert_eq!(stroked.points.len(), 2, "an open outline is left open");
    let padded = pad_stops(&[]);
    assert_eq!(padded.len(), STOPS);
    assert_eq!(padded[0], (1.0, CLEAR));
}

#[test]
fn kit_7_5_shards_and_a_gem_fill_their_tall_box() {
    for shape in [WorldShape::Shards, WorldShape::Gem] {
        let rows = rows(&body(shape, 0.0, 0.0, 0.0, WorldRing::None, 0), 220.0, false);
        assert_eq!(rows.len(), 8, "six facets and two sparks");
        let (w, h) = world::unit_box(shape);
        for row in &rows {
            for p in &row.points {
                assert!(p[0] >= 0.0 && p[0] <= w && p[1] >= 0.0 && p[1] <= h, "{shape:?} {p:?}");
            }
        }
    }
}

#[test]
fn a_palette_is_three_colours_or_none_and_paints_the_disc() {
    use emusen_kit_qt::world::{palette, palette_words};
    let nes = palette("#FFF8FB #F3CFE3 #A9789F").expect("three colours");
    assert_eq!(nes[0], Colour::rgb(0xFF, 0xF8, 0xFB));
    assert_eq!(palette_words(Some(nes)), "#FFF8FB #F3CFE3 #A9789F");
    assert_eq!((palette(""), palette("#FFF8FB #F3CFE3"), palette_words(None)), (None, None, String::new()));
    let mut planet = body(WorldShape::Planet, 0.0, 0.0, 0.0, WorldRing::None, 0);
    let from_hue = rows(&planet, 330.0, false);
    planet.palette = Some(nes);
    let painted = rows(&planet, 330.0, false);
    assert_ne!(from_hue, painted, "the palette, not the hue, paints the disc");
}
