//! Program-drawn Eclipse Pause Card held to boards #6 and #35-#38 on the software backend (R-20261113-5).

mod common;
mod eclipse_board;
mod eclipse_pause_oracle;

#[test]
fn eclipse_pause_holds_boards_6_and_35_to_38() {
    eclipse_pause_oracle::pause("software");
}
