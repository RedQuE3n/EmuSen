//! Eclipse's Controller Bindings (#12) and Keyboard Keys (#75) on the software backend, held to their boards.

mod common;
mod eclipse_board;
#[path = "eclipse_pad/mod.rs"]
mod eclipse_pad;
mod eclipse_pad_oracle;
mod eclipse_settings_oracle;

#[test]
fn qt_3_4_11_eclipse_draws_controller_bindings_and_keyboard_keys_as_their_boards() {
    eclipse_pad_oracle::oracle("software");
}
