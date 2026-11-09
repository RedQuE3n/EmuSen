// The wheels and the dial held to Orrery's Wheelwork canvas: its slots, dots, dial and camera over its states, and a walk that reaches every node (EmuSen_Kit.md §7.32).

use emusen_kit_core::wheels::{Steps, WheelGeometry, Wheels, turn, wheels};

const G: WheelGeometry = WheelGeometry::WHEELWORK;

/// The canvas's catalogues: each maker's count in table order (its default, and the synthetic n30, n120 and maker40).
fn catalogue(counts: &[usize]) -> Vec<usize> {
    counts.iter().enumerate().flat_map(|(g, n)| std::iter::repeat_n(g, *n)).collect()
}

/// The canvas's synthetic catalogue of `n` among twelve makers, the first holding `first` when given.
fn synthetic(n: usize, first: Option<usize>) -> Vec<usize> {
    let counts: Vec<usize> = match first {
        Some(f) => std::iter::once(f).chain((1..12).map(|m| (n - f) / 11 + usize::from(m <= (n - f) % 11))).collect(),
        None => (0..12).map(|m| n / 12 + usize::from(m < n % 12)).collect(),
    };
    catalogue(&counts)
}

/// Every node reached from `from` by the four arrows and the shoulders.
fn reached(w: &Wheels, from: usize) -> usize {
    let mut seen = vec![false; w.steps.len()];
    let mut queue = vec![from];
    seen[from] = true;
    while let Some(at) = queue.pop() {
        let Steps { left, right, up, down, previous, next } = w.steps[at];
        for to in [left, right, up, down, previous, next].into_iter().flatten() {
            if !seen[to] {
                seen[to] = true;
                queue.push(to);
            }
        }
    }
    seen.iter().filter(|s| **s).count()
}

/// The slots of the shown items about the top: a distance d takes slot (d + 8) mod 8.
fn slots(w: &Wheels, current: usize) -> Vec<(usize, usize)> {
    let t = turn(w, Some(current), &G);
    (0..w.group_of.len()).filter(|i| t.shown[*i]).map(|i| (i, t.turns[i].rem_euclid(8) as usize)).collect()
}

#[test]
fn the_default_eleven_fill_the_slots_about_the_second() {
    let keys = catalogue(&[5, 4, 2]);
    let w = wheels(1280.0, &keys, 3, &G);
    assert_eq!((w.firsts.clone(), w.sizes.clone()), (vec![0, 5, 9], vec![5, 4, 2]));
    assert_eq!(w.ranks(), vec![2, 1, 0], "the fewest innermost");
    assert_eq!(wheels(1280.0, &catalogue(&[2, 3, 2]), 3, &G).ranks(), vec![0, 2, 1], "a tie in the groups' order");
    assert_eq!(slots(&w, 1), vec![(0, 7), (1, 0), (2, 1), (3, 2), (4, 3)], "11: slots 0, 1, 2, 3 and 7");
    let t = turn(&w, Some(1), &G);
    assert_eq!((t.wheel, t.dimmed, t.hidden, t.lit, t.camera), (Some(0), false, 0, Some(14), 0.0));
    assert_eq!(w.dial_xs, vec![132.0, 268.0, 404.0, 540.0, 676.0, 812.0], "six stops from x 64 at 136");
    assert!(!w.scrolls);
    assert_eq!(w.steps[1], Steps { left: Some(0), right: Some(2), up: None, down: Some(14), previous: None, next: Some(15) });
    assert_eq!(w.steps[0].left, None, "the walk stops at the first maker's first");
    assert_eq!((w.steps[4].right, w.steps[5].left, w.steps[10].right), (Some(5), Some(4), None), "and runs on between makers");
    assert_eq!(w.steps[14], Steps { left: Some(13), right: Some(15), up: Some(0), down: None, previous: None, next: Some(15) });
    assert_eq!(w.steps[11], Steps { left: None, right: Some(12), up: None, down: None, previous: None, next: Some(14) }, "a place");
    assert_eq!(w.steps[16].right, None, "the dial ends at its last maker");
    // A maker's stop turns its wheel to its first; a place dims the first wheel.
    let s = turn(&w, Some(15), &G);
    assert_eq!((s.wheel, s.turns[5], s.turns[8], s.lit), (Some(1), 0, 3, Some(15)));
    let p = turn(&w, Some(12), &G);
    assert_eq!((p.wheel, p.dimmed, p.lit), (Some(0), true, Some(12)));
    assert_eq!(reached(&w, 1), 17);
}

#[test]
fn one_console_and_none() {
    let w = wheels(1280.0, &catalogue(&[1]), 3, &G);
    assert_eq!((w.dial_xs.len(), slots(&w, 0)), (4, vec![(0, 0)]), "one world at the top and a dial of four");
    assert_eq!(reached(&w, 0), 5);
    let e = wheels(1280.0, &[] as &[usize], 3, &G);
    let t = turn(&e, Some(0), &G);
    assert_eq!((e.dial_xs.len(), t.wheel, t.lit, t.hidden), (3, None, Some(0), 0), "no wheel, the dial of three places");
    assert_eq!(reached(&e, 0), 3);
}

#[test]
fn thirty_scroll_the_dial_under_the_camera() {
    for width in [1280.0, 1422.2] {
        let w = wheels(width, &synthetic(30, None), 3, &G);
        assert_eq!((w.sizes.len(), w.dial_xs.len(), w.scrolls), (12, 15, true), "{width}: a fifteen-stop dial scrolls");
        assert_eq!(turn(&w, Some(1), &G).camera, 0.0, "{width}: the first maker's stop needs no camera");
        let last = w.stop_of(11);
        let c = turn(&w, Some(last), &G).camera;
        assert!((c - (2040.0 - (width - 128.0))).abs() < 1e-9, "{width}: the camera clamps at the dial's end, {c}");
        let mid = turn(&w, Some(w.stop_of(5)), &G).camera;
        assert!((mid - (8.0 * 136.0 + 68.0 - (width - 128.0) / 2.0)).abs() < 1e-9, "{width}: a middle stop centred");
        assert_eq!(reached(&w, 1), 45);
    }
}

#[test]
fn a_hundred_and_twenty_show_seven_of_ten_and_dots_for_the_rest() {
    let w = wheels(1280.0, &synthetic(120, None), 3, &G);
    let t = turn(&w, Some(1), &G);
    assert_eq!((t.shown.iter().filter(|s| **s).count(), t.hidden), (5, 5), "the second: one before, three after");
    let t = turn(&w, Some(5), &G);
    assert_eq!((t.shown.iter().filter(|s| **s).count(), t.hidden), (7, 3), "seven of ten");
    // Any maker in at most eleven presses of a shoulder, from the first maker's first world.
    let (mut at, mut presses) = (0, 0);
    while let Some(next) = w.steps[at].next {
        at = next;
        presses += 1;
    }
    assert_eq!((at, presses), (w.stop_of(11), 11), "the shoulder reaches the last maker's stop");
    assert_eq!(reached(&w, 1), 135);
    // A sweep Right from the first world hears every console, in maker order.
    let (mut at, mut heard) = (0, 1);
    while let Some(next) = w.steps[at].right {
        at = next;
        heard += 1;
    }
    assert_eq!(heard, 120);
}

#[test]
fn a_maker_of_forty_turns_through_seven_at_a_time() {
    let keys = synthetic(120, Some(40));
    let w = wheels(1422.2, &keys, 3, &G);
    assert_eq!(&w.sizes[..4], &[40, 8, 8, 8]);
    assert_eq!(slots(&w, 20), vec![(17, 5), (18, 6), (19, 7), (20, 0), (21, 1), (22, 2), (23, 3)], "maker40-mid: every slot but the rim's");
    let t = turn(&w, Some(20), &G);
    assert_eq!((t.hidden, t.turns[0], t.turns[39]), (33, -20, 19));
    assert_eq!(slots(&w, 39), vec![(36, 5), (37, 6), (38, 7), (39, 0)], "the last at the top, nothing after it");
}

#[test]
fn groups_come_in_order_of_first_appearance() {
    let w = wheels(1280.0, &["b", "a", "b", "c", "a"], 3, &G);
    assert_eq!((w.group_of.clone(), w.place_in.clone()), (vec![0, 1, 0, 2, 1], vec![0, 0, 1, 0, 1]));
    assert_eq!((w.steps[0].right, w.steps[2].left, w.steps[1].right), (Some(2), Some(0), Some(4)), "the wheel steps by group, not by row");
    assert_eq!((w.steps[2].right, w.steps[1].left, w.steps[4].right), (Some(1), Some(2), Some(3)), "and on in maker order");
    assert_eq!(w.steps[w.stop_of(1)].up, Some(1));
    assert_eq!(w.totals(&[1.0, 10.0, 2.0, 100.0]), vec![3.0, 10.0, 100.0], "each group's sum, an item without a weight none");
}
