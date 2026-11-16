//! Eclipse's Choose a console in the running program on the software backend (EmuSen_Qt.md §3.4.9): the oracle of
//! eclipse_consoles_oracle, its walk, its table and its pictures.

mod common;
mod eclipse_board;
mod eclipse_consoles_oracle;

#[test]
fn qt_3_4_9_eclipse_s_consoles_walks_opens_steps_makers_and_draws_each_console_in_its_colour_in_the_program() {
    eclipse_consoles_oracle::oracle("software");
}
