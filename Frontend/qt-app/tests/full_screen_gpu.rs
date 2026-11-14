//! The start's window full screen in a compositor at the handheld panel's 1920 by 1200, graphics.json naming a window of
//! 1000 by 640: test inputs (`common::full_screen`). A GPU job.

mod common;

#[test]
fn the_start_shows_the_window_full_screen_and_the_hotkey_turns_it() {
    common::full_screen::run(1920, 1200, (1000, 640));
}
