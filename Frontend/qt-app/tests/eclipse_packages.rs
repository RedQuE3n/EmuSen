//! Eclipse Packages (#22 Library, #23 Layers, #69 Package Page) held to their boards on software oracle.

mod common;
mod eclipse_board;
mod eclipse_packages_oracle;

#[test]
fn eclipse_packages_hold_boards_22_23_and_69() {
    eclipse_packages_oracle::packages("software");
}
