//! Eclipse Cheats (#21) on the software backend.

mod common;
mod eclipse_board;
mod eclipse_cheats_oracle;

#[test]
fn qt_eclipse_cheats_board_filters_editor_browse_and_entries() {
    eclipse_cheats_oracle::cheats_oracle("software");
}
