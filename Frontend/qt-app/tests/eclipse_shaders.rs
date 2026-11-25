//! Eclipse Screen filter / Shaders (#20) software oracle test (R-20261125-10, -14, -15).

mod common;
mod eclipse_board;
#[path = "eclipse_pad/mod.rs"]
mod eclipse_pad;
mod eclipse_settings_oracle;
mod eclipse_shaders_oracle;

#[test]
fn qt_3_4_11_eclipse_shaders_holds_its_board_and_interactive_states() {
    eclipse_shaders_oracle::oracle_software();
}
