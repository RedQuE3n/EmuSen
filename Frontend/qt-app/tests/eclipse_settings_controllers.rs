//! Eclipse's Settings on its Controllers page on the software backend, held to EclipseSettingsControllers.dc.html (EmuSen_Qt.md §3.4.11).

mod common;
mod eclipse_board;
mod eclipse_pad;
mod eclipse_settings_oracle;
mod eclipse_settings_pages;

#[test]
fn qt_3_4_11_eclipse_s_settings_draws_controllers_as_its_board_and_every_row_is_reached_and_acted_on() {
    eclipse_settings_pages::controllers("software");
}
