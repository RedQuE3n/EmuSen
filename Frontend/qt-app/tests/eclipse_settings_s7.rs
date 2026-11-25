//! Eclipse's Settings on Shader levers, Shader parameters, Themes and Scraping (account and run) on the software backend, held to boards #59-#62 (EmuSen_Qt.md §3.4.11).

mod common;
mod eclipse_board;
mod eclipse_pad;
mod eclipse_settings_oracle;
mod eclipse_settings_s7_pages;

#[test]
fn qt_3_4_11_eclipse_s_settings_draws_levers_shader_params_themes_and_scrape_run_as_their_boards() {
    eclipse_settings_s7_pages::s7("software");
}
