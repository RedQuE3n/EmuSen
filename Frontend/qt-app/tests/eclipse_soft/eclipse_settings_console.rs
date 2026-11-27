//! Eclipse's Settings on a console's page on the software backend, held to EclipseSettingsConsole.dc.html (EmuSen_Qt.md §3.4.11).

use crate::eclipse_settings_consoles_pages;

#[test]
fn qt_3_4_11_eclipse_s_settings_draws_one_console_as_its_board_and_every_row_is_reached_and_acted_on() {
    eclipse_settings_consoles_pages::console("software");
}
