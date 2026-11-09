// The orbits held to the reference theme's canvas, consoles.html of 2026-11-08 (EmuSen_Kit.md §7.30).

use emusen_kit_core::orbits::{OrbitGeometry, capacity, ellipse, orbit_layout};

const G: OrbitGeometry = OrbitGeometry::REFERENCE;

/// The canvas's eleven consoles by maker, and its three places' centres (x 338, 542, 757 at 1280).
const ELEVEN: [&str; 11] = ["N", "N", "N", "N", "N", "Sega", "Sega", "Sega", "Sega", "Sony", "Sony"];
const PLACES: [f64; 3] = [338.0, 542.0, 757.0];

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.01
}

/// The canvas's synthetic catalogue: twelve makers, `n` consoles shared out, or the first maker holding `first`.
fn makers(n: usize, first: Option<usize>) -> Vec<usize> {
    let counts: Vec<usize> = match first {
        Some(f) => std::iter::once(f).chain((1..12).map(|m| (n - f) / 11 + usize::from(m <= (n - f) % 11))).collect(),
        None => (0..12).map(|m| n / 12 + usize::from(m < n % 12)).collect(),
    };
    counts.iter().enumerate().flat_map(|(m, &c)| std::iter::repeat_n(m, c)).collect()
}

#[test]
fn the_three_slots_hold_two_six_and_nine_and_the_outer_ten_at_sixteen_by_nine() {
    let inner = ellipse(0, 1280.0, &G);
    assert!(near(inner.from.to_degrees(), 63.57) && near(inner.mean, 240.0), "below y 197 on the inner slot: {inner:?}");
    assert!(near(ellipse(1, 1280.0, &G).to.to_degrees(), 151.24));
    assert!(near(ellipse(2, 1280.0, &G).from.to_degrees(), 29.43));
    assert_eq!((capacity(0, 1280.0, &G), capacity(1, 1280.0, &G), capacity(2, 1280.0, &G)), (2, 6, 9));
    assert_eq!(capacity(2, 1422.2, &G), 10);
}

#[test]
fn the_canvas_eleven_by_size_innermost_first_all_centred() {
    let l = orbit_layout(1280.0, &ELEVEN, &PLACES, 3 + 1, None, &G);
    let sizes: Vec<(usize, i32, bool)> = l.orbits.iter().map(|o| (o.size, o.slot, o.turning)).collect();
    assert_eq!(sizes, vec![(2, 0, false), (4, 1, false), (5, 2, false)]);
    let at = |i: usize| (l.worlds[i].x, l.worlds[i].y);
    for (i, x, y) in [(9, 560.07, 209.34), (10, 719.93, 209.34), (5, 407.94, 314.35), (7, 721.02, 363.67), (0, 328.54, 449.63), (1, 478.72, 502.10), (2, 640.0, 520.0)] {
        assert!(near(at(i).0, x) && near(at(i).1, y), "world {i} at {:?}, the canvas {x}, {y}", at(i));
    }
    assert!(l.worlds.iter().all(|w| w.shown));
    let labels: Vec<(f64, f64)> = l.orbits.iter().map(|o| ((o.label.0 * 100.0).round() / 100.0, (o.label.1 * 100.0).round() / 100.0)).collect();
    assert_eq!(labels, vec![(725.35, 152.0), (971.26, 152.0), (1125.76, 219.84)]);
    assert!(l.orbits.iter().all(|o| o.before.is_none() && o.after.is_none()));
}

#[test]
fn the_walk_goes_along_an_orbit_and_to_the_nearest_in_x_of_the_next() {
    let l = orbit_layout(1280.0, &ELEVEN, &PLACES, 3 + 1, None, &G);
    let snes = l.walk[3 + 1];
    assert_eq!((snes.left, snes.right, snes.down), (Some(3), Some(5), None));
    assert_eq!(snes.up, Some(3 + 5), "Master System (408) is nearer 479 than Genesis (559)");
    assert_eq!(l.walk[3 + 9].up, Some(1), "the inner orbit's PlayStation (560) goes up to Favourites (542)");
    assert_eq!(l.walk[3 + 10].up, Some(2));
    assert_eq!((l.walk[0].left, l.walk[0].right, l.walk[0].up, l.walk[0].down), (None, Some(1), None, Some(3 + 9)));
    assert_eq!(l.walk[2].down, Some(3 + 10));
}

#[test]
fn one_console_sits_at_the_middle_apex_and_none_lays_out_nothing() {
    let l = orbit_layout(1280.0, &["N"], &PLACES, 3, None, &G);
    assert_eq!(l.orbits[0].slot, 1);
    assert!(near(l.worlds[0].x, 640.0) && near(l.worlds[0].y, 370.0));
    assert_eq!(l.walk[3].up, Some(1), "640 is nearest Favourites");
    let seven = orbit_layout(1280.0, &["N"; 7], &PLACES, 3, None, &G);
    assert_eq!(seven.orbits[0].slot, 2, "seven cannot show whole on the middle: the outer");
    let none = orbit_layout(1280.0, &[] as &[&str], &PLACES, 0, None, &G);
    assert!(none.orbits.is_empty() && none.worlds.is_empty());
    assert_eq!(none.walk[0].down, None);
}

#[test]
fn a_hundred_and_twenty_turn_and_the_forty_maker_turns_under_its_focus() {
    let n120 = makers(120, None);
    let l = orbit_layout(1280.0, &n120, &PLACES, 3 + 1, None, &G);
    assert_eq!(l.orbits.iter().filter(|o| o.shown).count(), 3);
    assert_eq!(l.worlds.iter().filter(|w| w.shown).count(), 17, "2, 6 and 9 of ten each");
    assert!(l.orbits[..3].iter().all(|o| o.turning) && l.orbits[1].offset == 0.0 && l.orbits[2].offset == 0.0);
    assert!(l.worlds[1].shown && l.worlds[2].shown && !l.worlds[0].shown, "the second focused shows itself and the third, a whole window");
    assert!(near(l.worlds[1].x, 560.07) && near(l.worlds[2].x, 719.93));
    let after = l.orbits[2].after.expect("the outer's ellipses past its right end");
    assert!(near(after.0, 1215.30) && near(after.1, 193.87), "{after:?}");
    let inner = &l.orbits[0];
    assert!(inner.before.is_some_and(|p| near(p.0, 458.29) && near(p.1, 157.35)) && inner.after.is_some_and(|p| near(p.0, 821.71)), "{inner:?}");

    let m40 = makers(120, Some(40));
    let mid = orbit_layout(1280.0, &m40, &PLACES, 3 + 20, None, &G);
    let outer = &mid.orbits[mid.worlds[20].orbit];
    assert_eq!((outer.size, outer.slot), (40, 2), "the forty sit outermost");
    assert!(near(mid.worlds[20].x, 640.0) && near(mid.worlds[20].y, 520.0), "the twenty-first at the apex");
    assert!(outer.before.is_some_and(|p| near(p.0, 64.70)) && outer.after.is_some());
    assert_eq!(mid.worlds.iter().filter(|w| w.shown).count(), 17);
    let last = orbit_layout(1280.0, &m40, &PLACES, 3 + 39, None, &G);
    let outer = &last.orbits[last.worlds[39].orbit];
    assert!(outer.before.is_some() && outer.after.is_none(), "at the last of the forty, worlds wait before it alone");
    assert!(last.worlds[39].shown && !last.worlds[30].shown, "the last nine shown");
}

#[test]
fn the_shown_groups_move_only_when_the_focus_leaves_them() {
    let n120 = makers(120, None);
    let first = orbit_layout(1280.0, &n120, &PLACES, 3, None, &G);
    assert_eq!(first.base, 0);
    let fourth = orbit_layout(1280.0, &n120, &PLACES, 3 + 30, Some(first.base), &G);
    assert_eq!(fourth.base, 1, "the fourth maker scrolls in as the outermost");
    let second = orbit_layout(1280.0, &n120, &PLACES, 3 + 10, Some(fourth.base), &G);
    assert_eq!(second.base, 1, "the second maker is still shown: nothing moves");
    let place = orbit_layout(1280.0, &n120, &PLACES, 0, Some(second.base), &G);
    assert_eq!(place.base, 1, "a place keeps the orbits");
    assert_eq!(orbit_layout(1280.0, &n120, &PLACES, 3 + 119, None, &G).base, 9, "the first focus centres when it can");
    let hidden = &second.orbits[0];
    assert_eq!(hidden.slot, -1);
    assert!(!hidden.shown && second.worlds[0].shown == false);
    assert_eq!(second.walk[3 + 10].up, Some(3), "up to a hidden group's first world");
}

#[test]
fn a_jump_to_an_orbit_lands_nearest_the_focus_across() {
    let l = orbit_layout(1280.0, &ELEVEN, &PLACES, 3 + 1, None, &G);
    let reach: Vec<usize> = l.orbits.iter().map(|o| o.reach).collect();
    assert_eq!(reach, vec![3 + 9, 3 + 5, 3 + 1], "from the Super Nintendo at 479: PlayStation, Master System, itself");
    let from_place = orbit_layout(1280.0, &ELEVEN, &PLACES, 2, None, &G);
    assert_eq!(from_place.orbits[0].reach, 3 + 10, "from Recently played at 757: PlayStation 2");
}

#[test]
fn an_interleaved_table_groups_by_key_and_keeps_each_group_in_the_table_s_order() {
    let mixed = ["Sony", "N", "Sega", "N", "Sony", "Sega", "N", "Sega", "N", "Sega", "N"];
    let l = orbit_layout(1280.0, &mixed, &PLACES, 3 + 1, None, &G);
    let firsts: Vec<(usize, usize)> = l.orbits.iter().map(|o| (o.first, o.size)).collect();
    assert_eq!(firsts, vec![(0, 2), (2, 4), (1, 5)], "Sony, Sega, N by size, each from its first row");
    let along = |list: &[usize]| list.windows(2).all(|p| l.worlds[p[0]].x < l.worlds[p[1]].x);
    assert!(along(&[1, 3, 6, 8, 10]) && along(&[2, 5, 7, 9]) && along(&[0, 4]), "each orbit left to right in the table's order");
    assert_eq!(l.walk[3 + 3].left, Some(3 + 1));
    assert_eq!(l.walk[3 + 3].right, Some(3 + 6));
}

#[test]
fn a_focused_world_on_the_inner_orbit_keeps_its_ring_below_the_places_at_the_five_sizes() {
    // 1280 x 800 and 1920 x 1200 are 1280 units wide, 1920 x 1080, 1280 x 720 and 3840 x 2160 are 1422.2; the places end at 140.
    for width in [1280.0, 1280.0, 1422.2, 1422.2, 1422.2] {
        let mut inner = 0;
        let text = |list: Vec<usize>| -> Vec<String> { list.iter().map(usize::to_string).collect() };
        let eleven: Vec<String> = ELEVEN.iter().map(|m| (*m).to_owned()).collect();
        for groups in [text(makers(120, None)), text(makers(30, None)), text(makers(120, Some(40))), eleven] {
            for focus in 0..groups.len() {
                let l = orbit_layout(width, &groups, &PLACES, 3 + focus, None, &G);
                let w = l.worlds[focus];
                if l.orbits[w.orbit].slot != 0 {
                    continue;
                }
                inner += 1;
                assert!(w.shown && w.y - 51.0 >= 140.0, "focused {focus} at {width}: its ring's top at {}", w.y - 51.0);
                for shown in l.worlds.iter().filter(|o| o.shown && l.orbits[o.orbit].slot == 0) {
                    assert!(shown.y - 36.0 >= 142.0, "a resting world's top at {}", shown.y - 36.0);
                }
            }
        }
        assert!(inner >= 20, "every catalogue focused on its inner orbit: {inner}");
    }
}
