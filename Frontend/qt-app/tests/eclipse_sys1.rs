//! Program-drawn Eclipse Dialogs/Notices (#14) and Power (#15) held on the software backend (R-20261113-5).

mod common;
mod eclipse_board;
mod eclipse_sys1_oracle;

#[test]
fn eclipse_sys1_holds_boards_14_and_15() {
    eclipse_sys1_oracle::sys1("software");
}
