//! Program-drawn Eclipse Pause Card held to boards #6 and #35-#43 on the software backend (R-20261113-5).

use crate::eclipse_pause_oracle;

#[test]
fn eclipse_pause_holds_boards_6_and_35_to_43() {
    eclipse_pause_oracle::pause("software");
}
