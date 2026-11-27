//! Eclipse's Settings on Memory cards, Saves on the cards and One save on the software backend, held to boards #58, #63 and #64 (EmuSen_Qt.md §3.4.11).

use crate::eclipse_settings_cards_pages;

#[test]
fn qt_3_4_11_eclipse_s_settings_draws_memory_cards_and_saves_as_their_boards_and_every_row_works() {
    eclipse_settings_cards_pages::cards("software");
}
