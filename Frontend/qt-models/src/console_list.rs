//! Choose a console's listing (EmuSen_Qt.md §3.4.9): which consoles a core runs are shown, by its chips, and in what order,
//! by its Y; read from the library's rows, nothing kept here.

use crate::library::LibraryRow;
use std::cmp::Ordering;

/// Which consoles are shown, in `words::CONSOLE_SHOWS`' order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Show {
    /// Every console a core runs, those with no games among them.
    All,
    /// Those holding a game.
    WithGames,
    /// Those holding a favourite game.
    Favourites,
}

/// The order the consoles stand in, in `words::CONSOLE_SORTS`' order. The makers keep their order and each maker's consoles
/// are ordered among themselves, so the makers' column and LT RT stay as they were.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    /// The shelves' order.
    Maker,
    /// The oldest first.
    Year,
    /// By name, letters before case.
    Name,
    /// The most games first.
    Games,
}

pub const SHOWS: [Show; 3] = [Show::All, Show::WithGames, Show::Favourites];
pub const ORDERS: [Order; 4] = [Order::Maker, Order::Year, Order::Name, Order::Games];

impl Show {
    pub fn holds(self, row: &LibraryRow) -> bool {
        match self {
            Show::All => true,
            Show::WithGames => row.count > 0,
            Show::Favourites => row.favourites > 0,
        }
    }
}

impl Order {
    fn cmp(self, a: &LibraryRow, b: &LibraryRow, short: &[String]) -> Ordering {
        match self {
            Order::Maker => Ordering::Equal,
            Order::Year => a.year.cmp(&b.year),
            // By the name the plates show.
            Order::Name => plate_name(a, short).to_lowercase().cmp(&plate_name(b, short).to_lowercase()),
            Order::Games => b.count.cmp(&a.count),
        }
    }
}

/// The name a console's plate shows: its short name where its key is among `short`, the consoles whose full name the theme
/// measured as wider than its plate's two lines, else its shown name (EmuSen_Qt.md §3.4.9).
pub fn plate_name<'a>(row: &'a LibraryRow, short: &[String]) -> &'a str {
    if short.iter().any(|k| *k == row.key) { &row.name } else { row.shown_name() }
}

/// The consoles `show` holds, each maker's together in the order its first console stands, ordered within by `order`, by
/// name the name `plate_name` gives with `short`; an empty maker is a group of its own, as `view::maker_groups` has it.
pub fn listed<'a>(consoles: &'a [LibraryRow], show: Show, order: Order, short: &[String]) -> Vec<&'a LibraryRow> {
    let shown: Vec<&LibraryRow> = consoles.iter().filter(|r| show.holds(r)).collect();
    let mut makers: Vec<&str> = Vec::new();
    let mut group = |r: &'a LibraryRow| match makers.iter().position(|m| !r.maker.is_empty() && *m == r.maker) {
        Some(g) => g,
        None => {
            makers.push(&r.maker);
            makers.len() - 1
        }
    };
    let mut placed: Vec<(usize, usize, &LibraryRow)> = shown.iter().enumerate().map(|(i, r)| (group(r), i, *r)).collect();
    placed.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| order.cmp(a.2, b.2, short)).then(a.1.cmp(&b.1)));
    placed.into_iter().map(|(_, _, r)| r).collect()
}

/// How many consoles each of `SHOWS` holds.
pub fn counts(consoles: &[LibraryRow]) -> [usize; 3] {
    SHOWS.map(|show| consoles.iter().filter(|r| show.holds(r)).count())
}
