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
