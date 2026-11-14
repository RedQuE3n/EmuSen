//! A fade between systems on the real view with fadeAbovePrimary false, every frame of the held clock read against ES-DE
//! 3.4.1's rule (EmuSen_Qt.md §4.8.8).

mod common;

use common::fade::{assert_clock_held, es_de, run, ARRIVING_WIDTH, LEFT_WIDTH, PRECISION, STEP_MS};

#[test]
fn a_fade_without_fade_above_primary_darkens_below_the_primary_only_at_es_de_s_times() {
    // Past the run's end by two frames, so the frame the scene comes to rest on is read.
    let frames = (es_de::RUN_MS as i64) / STEP_MS + 2;
    let reads = run("fade", false, frames);
    assert_clock_held(&reads);
    for r in &reads {
        let black = es_de::black(r.ms);
        assert!((r.black - black).abs() < PRECISION, "at {} ms the black is {}, ES-DE's {black}: {reads:#?}", r.ms, r.black);
        let width = if es_de::shows_left(r.ms) { LEFT_WIDTH } else { ARRIVING_WIDTH };
        assert_eq!(r.below_width, Some(width), "at {} ms the picture below: {reads:#?}", r.ms);
        assert_eq!(r.above, 1.0, "at {} ms the picture above is drawn whole", r.ms);
        if black > 0.0 {
            assert_eq!(r.order, ["below", "veil", "systems", "above"], "at {} ms the black lies between the elements below and the primary", r.ms);
        }
        assert_eq!(r.moving, r.ms < es_de::RUN_MS, "at {} ms the scene moves only within the run", r.ms);
    }
}
