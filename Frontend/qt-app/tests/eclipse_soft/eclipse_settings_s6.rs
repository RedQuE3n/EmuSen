//! Eclipse's Settings on Players, Profiles, One profile, Hotkeys and Sound Output on the software backend, held to boards #52-#56 (EmuSen_Qt.md §3.4.11).

use crate::eclipse_settings_s6_pages;

#[test]
fn qt_3_4_11_eclipse_s_settings_draws_controllers_inner_pages_and_output_as_their_boards() {
    eclipse_settings_s6_pages::s6("software");
}
