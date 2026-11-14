//! The start's window full screen in a compositor at a television's 1280 by 720, graphics.json naming a fixed window of
//! 900 by 500: a second set of test inputs, in a process of its own, since a compositor is one per process
//! (`common::full_screen`). A GPU job.

mod common;

#[test]
fn the_start_shows_the_window_full_screen_at_a_second_size() {
    common::full_screen::run(1280, 720, (900, 500), false);
}
