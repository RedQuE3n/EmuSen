//! Eclipse's Settings on its Display page on the software backend: eclipse_settings_oracle's table, words, walk and actions (EmuSen_Qt.md §3.4.11).

use crate::eclipse_settings_oracle;

#[test]
fn qt_3_4_11_eclipse_s_settings_draws_display_as_its_board_and_every_row_is_reached_and_acted_on() {
    eclipse_settings_oracle::oracle("software");
}
