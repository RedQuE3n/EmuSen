//! Eclipse Scraper (#66), Scrape Status (#67), and Library Manager (#68) on the software backend.

mod common;
mod eclipse_board;
#[path = "eclipse_board/tools_t4.rs"]
mod tools_t4;

#[test]
fn eclipse_tools_t4_hold_boards_66_to_68() {
    tools_t4::tools_t4("software");
}
