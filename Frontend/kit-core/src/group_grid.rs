//! Groups as a column and the chosen group's items as a grid, with each item's four neighbours; names no console (EmuSen_Kit.md §7.120).

/// One item's place in its group's grid and its neighbours there, by the item's index in the model; none past the grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridItem {
    pub column: usize,
    pub row: usize,
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub up: Option<usize>,
    /// The item below, or the last item when the row below is shorter; none from the last row.
    pub down: Option<usize>,
}

/// The layout: an entry for each item in the model's order, and for each group its first item and its rows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GroupGrid {
    pub items: Vec<GridItem>,
    pub firsts: Vec<Option<usize>>,
    pub rows: Vec<usize>,
}

/// The grid of every group at `columns` columns (one at least), from each item's group and place in the group.
pub fn group_grid(places: &[(usize, usize)], columns: usize) -> GroupGrid {
    let columns = columns.max(1);
    let groups = places.iter().map(|(g, _)| g + 1).max().unwrap_or(0);
    let mut at: Vec<Vec<Option<usize>>> = vec![Vec::new(); groups];
    for (item, &(g, i)) in places.iter().enumerate() {
        if at[g].len() <= i {
            at[g].resize(i + 1, None);
        }
        at[g][i] = Some(item);
    }
    let find = |g: usize, i: usize| at[g].get(i).copied().flatten();
    let items = places
        .iter()
        .map(|&(g, i)| {
            let size = at[g].len();
            let (column, row) = (i % columns, i / columns);
            let below = (i + columns).min(size.saturating_sub(1));
            GridItem {
                column,
                row,
                left: if column > 0 { find(g, i - 1) } else { None },
                right: if column + 1 < columns { find(g, i + 1) } else { None },
                up: i.checked_sub(columns).and_then(|u| find(g, u)),
                down: if below / columns > row { find(g, below) } else { None },
            }
        })
        .collect();
    GroupGrid { items, firsts: (0..groups).map(|g| find(g, 0)).collect(), rows: at.iter().map(|a| a.len().div_ceil(columns)).collect() }
}
