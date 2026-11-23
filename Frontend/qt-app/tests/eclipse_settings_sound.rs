//! Eclipse's Settings on its Sound page on the software backend, held to EclipseSettingsSound.dc.html (EmuSen_Qt.md §3.4.11).

mod common;
mod eclipse_board;
mod eclipse_pad;
mod eclipse_settings_oracle;
mod eclipse_settings_pages;

#[test]
fn qt_3_4_11_eclipse_s_settings_draws_sound_as_its_board_and_every_row_is_reached_and_acted_on() {
    eclipse_settings_pages::sound("software");
}
