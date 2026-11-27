//! Eclipse Search (#8), Collections (#11), and Tools hub (#9) held on the software backend.

use crate::eclipse_sys2_oracle;

#[test]
fn eclipse_sys2_holds_boards_8_9_and_11() {
    eclipse_sys2_oracle::sys2("software");
}
