// The picker's rows held to the courts canvas (EmuSen_Interface.md §24.1, EmuSen_Kit.md §7.19).

use emusen_kit_core::layout::{PlanetGeometry, capacity, planet_layout};

const G: PlanetGeometry = PlanetGeometry::COURTS;

#[test]
fn the_courts_canvas_eleven_bodies_under_three_places() {
    let l = planet_layout(1280.0, 3, 11, &G);
    assert_eq!(l.tiles.len(), 14);
    let places: Vec<(f64, f64)> = l.tiles[..3].iter().map(|t| (t.x, t.width)).collect();
    assert_eq!(places, vec![(64.0, 372.0), (454.0, 372.0), (844.0, 372.0)]);
    assert!(l.tiles[..3].iter().all(|t| t.y == 100.0 && t.height == 72.0));
    assert_eq!((l.tiles[3].x, l.tiles[3].y, l.tiles[3].width, l.tiles[3].height), (67.0, 186.0, 176.0, 190.0));
    assert_eq!((l.tiles[8].x, l.tiles[8].row), (1037.0, 1), "six in the first row");
    assert_eq!((l.tiles[9].x, l.tiles[9].y, l.tiles[9].row), (164.0, 386.0, 2), "five centred in the second");
    assert_eq!(l.orbits, vec![238.0, 438.0]);
}

#[test]
fn the_walk_goes_to_the_nearest_tile_of_the_next_row_and_stops_at_the_ends() {
    let l = planet_layout(1280.0, 3, 11, &G);
    assert_eq!((l.tiles[0].left, l.tiles[0].right, l.tiles[0].up, l.tiles[0].down), (None, Some(1), None, Some(3)));
    assert_eq!(l.tiles[1].down, Some(5), "a tie goes to the first");
    assert_eq!(l.tiles[3].up, Some(0));
    assert_eq!(l.tiles[8].right, None, "a row's last tile goes no further right");
    assert_eq!(l.tiles[9].left, None);
    assert_eq!(l.tiles[4].down, Some(9), "252 and 446 are as near 349: the first");
    assert_eq!(l.tiles[13].up, Some(7));
    assert_eq!(l.tiles[13].down, None);
}

#[test]
fn rows_divide_evenly_and_capacity_follows_the_width() {
    assert_eq!(capacity(1280.0, &G), 6);
    assert_eq!(capacity(1422.0, &G), 6);
    assert_eq!(capacity(1600.0, &G), 7);
    let l = planet_layout(1280.0, 0, 7, &G);
    assert_eq!(l.tiles.iter().filter(|t| t.row == 1).count(), 4, "seven over two rows is four and three");
    assert_eq!(l.tiles[0].up, None, "no leading row");
    assert!(planet_layout(1280.0, 3, 0, &G).orbits.is_empty());
}
