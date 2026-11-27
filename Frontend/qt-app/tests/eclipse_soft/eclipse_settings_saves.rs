//! Eclipse's Settings on EmuSen's Saves and states page on the software backend, held to EclipseSettingsSaves.dc.html, and a trail of three levels (EmuSen_Qt.md §3.4.11).

use crate::eclipse_settings_pages;

#[test]
fn qt_3_4_11_eclipse_s_settings_draws_saves_and_states_as_its_board_and_a_trail_of_three() {
    eclipse_settings_pages::saves("software");
}
