//! EmuSen_Kit.md §7.120: groups as a column and the chosen group as a grid; the walk stays inside a group's grid.

use emusen_kit_core::group_grid::{group_grid, GridItem};

/// Each item's group and place, in a model's order: two groups interleaved, five and two items.
fn places() -> Vec<(usize, usize)> {
    vec![(0, 0), (1, 0), (0, 1), (0, 2), (1, 1), (0, 3), (0, 4)]
}

#[test]
fn kit_7_120_each_item_stands_in_its_group_s_grid_by_its_place() {
    let grid = group_grid(&places(), 4);
    let at: Vec<(usize, usize)> = grid.items.iter().map(|i| (i.column, i.row)).collect();
    assert_eq!(at, [(0, 0), (0, 0), (1, 0), (2, 0), (1, 0), (3, 0), (0, 1)]);
    assert_eq!(grid.firsts, [Some(0), Some(1)], "each group's first item, by its index in the model");
    assert_eq!(grid.rows, [2, 1]);
}

#[test]
fn kit_7_120_the_walk_stays_inside_a_group_and_down_lands_on_a_short_row_s_last() {
    let grid = group_grid(&places(), 4);
    // The first group's first item: nothing left or up, its neighbour right, the fifth item (row 1) below.
    assert_eq!(grid.items[0], GridItem { column: 0, row: 0, left: None, right: Some(2), up: None, down: Some(6) });
    // The fourth item of the first group, at the row's end: nothing right; down from column 3 lands on the short row's last.
    assert_eq!(grid.items[5], GridItem { column: 3, row: 0, left: Some(3), right: None, up: None, down: Some(6) });
    assert_eq!(grid.items[6].up, Some(0));
    assert_eq!(grid.items[6].down, None, "the last row has nothing below");
    // The second group never reaches into the first.
    assert_eq!((grid.items[1].right, grid.items[4].left, grid.items[4].right), (Some(4), Some(1), None));
}

#[test]
fn kit_7_120_one_column_and_no_items_hold() {
    let grid = group_grid(&places(), 0);
    assert!(grid.items.iter().all(|i| i.column == 0 && i.left.is_none() && i.right.is_none()), "no columns is one");
    assert_eq!(grid.rows, [5, 2]);
    let none = group_grid(&[], 4);
    assert!(none.items.is_empty() && none.firsts.is_empty() && none.rows.is_empty());
}

/// One group of `n` items in the model's order.
fn one_group(n: usize) -> Vec<(usize, usize)> {
    (0..n).map(|i| (0, i)).collect()
}

/// The 120-console library's shape (theme-audit's crowd): 40 under one maker, 10 under none, 70 under five more, the groups
/// interleaved in the model's order as a catalogue lists them.
fn crowd() -> Vec<(usize, usize)> {
    let mut sizes = [0usize; 7];
    (0..120)
        .map(|n| {
            let g = if n % 3 == 0 && sizes[0] < 40 { 0 } else if n % 12 == 1 && sizes[1] < 10 { 1 } else { 2 + n % 5 };
            sizes[g] += 1;
            (g, sizes[g] - 1)
        })
        .collect()
}

/// Where a player expects each press to land from place `i` of a group of `n` at `c` columns, by the grid's arithmetic:
/// along the row, the row above, the row below, or the last place when the row below is short of this column.
fn expected(i: usize, n: usize, c: usize) -> [Option<usize>; 4] {
    let left = (i % c > 0).then(|| i - 1);
    let right = (i % c + 1 < c && i + 1 < n).then(|| i + 1);
    let up = i.checked_sub(c);
    let down = if i + c < n { Some(i + c) } else { ((n - 1) / c > i / c).then(|| n - 1) };
    [left, right, up, down]
}

/// Every place of every group: each press lands where `expected` says, a press toward a place that lies that way never
/// does nothing, and every place is reached from the group's first by presses alone.
fn walk_holds(places: &[(usize, usize)], c: usize) {
    let grid = group_grid(places, c);
    let groups = places.iter().map(|p| p.0 + 1).max().unwrap_or(0);
    for g in 0..groups {
        let members: Vec<usize> = (0..places.len()).filter(|&k| places[k].0 == g).collect();
        let n = members.len();
        let at = |place: usize| members.iter().copied().find(|&k| places[k].1 == place);
        for &k in &members {
            let i = places[k].1;
            let item = grid.items[k];
            let got = [item.left, item.right, item.up, item.down];
            let want = expected(i, n, c).map(|p| p.and_then(at));
            assert_eq!(got, want, "group {g} of {n} at {c} columns, place {i}: left, right, up, down");
            // A place lies that way: left or right in its row, any row above or below.
            let lies = [i % c > 0, i % c + 1 < c && i + 1 < n, i >= c, (n - 1) / c > i / c];
            for (way, (lies, got)) in ["left", "right", "up", "down"].iter().zip(lies.iter().zip(got)) {
                assert!(!lies || got.is_some(), "group {g} of {n} at {c} columns, place {i}: {way} does nothing where a place lies");
            }
        }
        let mut reached = vec![grid.firsts[g].expect("each group has a first")];
        let mut next = 0;
        while next < reached.len() {
            let item = grid.items[reached[next]];
            for to in [item.left, item.right, item.up, item.down].into_iter().flatten() {
                assert_eq!(places[to].0, g, "a press never leaves the group");
                if !reached.contains(&to) {
                    reached.push(to);
                }
            }
            next += 1;
        }
        assert_eq!(reached.len(), n, "group {g} of {n} at {c} columns: every place reached from the first");
        assert_eq!(grid.rows[g], n.div_ceil(c), "group {g}'s rows");
    }
}

#[test]
fn kit_7_120_every_press_lands_where_a_player_expects_on_1_a_row_11_30_40_and_120() {
    for c in 1..=6 {
        for n in [1, c, 11, 30, 40] {
            walk_holds(&one_group(n), c);
        }
        let crowd = crowd();
        assert_eq!((crowd.len(), crowd.iter().filter(|p| p.0 == 0).count()), (120, 40), "the crowd's 120, 40 of one maker");
        walk_holds(&crowd, c);
    }
}
