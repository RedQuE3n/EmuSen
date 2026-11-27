//! Program-drawn Eclipse Cards frame held to boards #5, #44, #45 and #46 on the software backend (R-20261113-5).

use crate::eclipse_cards_oracle;

#[test]
fn eclipse_cards_hold_boards_5_44_45_and_46() {
    eclipse_cards_oracle::cards("software");
}
