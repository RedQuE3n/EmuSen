//! Eclipse Cores (#10) and its three core subpages (#72-#74) held to their boards on the software backend.

mod common;
mod eclipse_board;
mod eclipse_cores_oracle;

#[test]
fn eclipse_cores_hold_boards_10_and_72_to_74() {
    eclipse_cores_oracle::cores("software");
}
