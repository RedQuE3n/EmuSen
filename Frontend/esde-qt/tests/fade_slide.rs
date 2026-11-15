//! A `slide` systemToSystem profile on the real view: the next system's elements at once and no black, with
//! fadeAbovePrimary set, which only a fade reads (EmuSen_Qt.md §4.8.8).

mod common;

use common::fade::{assert_clock_held, es_de, run, ARRIVING_WIDTH, STEP_MS};

#[test]
fn a_slide_profile_shows_the_next_system_at_once_with_no_black() {
    // As long as a fade would run, and two frames past it.
    let reads = run("slide", true, (es_de::RUN_MS as i64) / STEP_MS + 2);
    assert_clock_held(&reads);
    for r in &reads {
        assert_eq!(r.black, 0.0, "at {} ms no black: {reads:#?}", r.ms);
        assert_eq!(r.below_width, Some(ARRIVING_WIDTH), "at {} ms the new system's picture", r.ms);
        assert_eq!(r.above, 1.0, "at {} ms the picture above is whole", r.ms);
    }
}
