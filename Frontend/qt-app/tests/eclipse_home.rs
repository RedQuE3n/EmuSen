//! Eclipse's Home held to its board (EmuSen_Qt.md §3.4.8): the screen placed through the theme host on a synthetic library
//! of the board's six games, the first suspended, and every row of `eclipse_home.tsv` held to the built screen at each of the
//! gate's sizes: its box by the board's own rule, its colour, its type size and the part that draws it; every row the program
//! has nothing for is not drawn; every word is the program's.

// The modules the scene imports register through the program's crate, which a test must link.
extern crate emusen_qt;

mod eclipse_home_oracle;

#[test]
fn qt_3_4_8_eclipse_s_home_is_its_board_at_every_size() {
    eclipse_home_oracle::oracle("software");
}
