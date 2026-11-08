// EmuSen_Kit.md §7.5: the sky's state is kit-core's stars and glow on the host's clock, into one buffer.

use std::time::Duration;

use emusen_kit_core::world::{self, Timeline, MOONRISE_SKY, SCREENS_SKY};
use emusen_kit_qt::stars::SkyState;

#[test]
fn kit_7_5_the_stars_are_moonrises_from_the_seed_and_fill_the_screen() {
    let timeline = Timeline::MOONRISE;
    let mut sky = SkyState::new(timeline.seed, timeline.star_count, true);
    assert_eq!(sky.stars().len(), 220);
    let mut random = world::SeededRandom::new(timeline.seed);
    assert_eq!(sky.stars(), world::place_stars(&mut random, timeline.star_count).as_slice(), "the same draws as the splash");
    sky.tick(0.0, [1280.0, 800.0], 1.0);
    assert_eq!(sky.points.len(), 220);
    for p in &sky.points {
        assert!(p.x >= 0.0 && p.x <= 1280.0 && p.y >= 0.0 && p.y <= 800.0 * world::HEIGHT_SHARE);
        assert!(p.opacity >= 0.1 && p.opacity <= 1.0, "0.55 to 1 of the twinkle");
    }
    sky.tick(0.0, [1280.0, 800.0], 0.0);
    assert!(sky.points.is_empty(), "no opacity, no stars drawn");
}

#[test]
fn kit_7_5_the_two_skies() {
    assert_eq!(SkyState::new(1, 1, true).stops(), MOONRISE_SKY);
    assert_eq!(SkyState::new(1, 1, false).stops(), SCREENS_SKY);
}

#[test]
fn kit_7_5_the_glow_comes_in_over_pan_and_goes_with_no_hue() {
    let mut sky = SkyState::new(7, 10, false);
    let size = [1280.0, 800.0];
    assert!(sky.glow_at(Duration::ZERO, size).is_none());
    sky.set_glow_hue(Some(200.0), Duration::ZERO);
    let early = sky.glow_at(Duration::from_millis(10), size).expect("coming in");
    let settled = sky.glow_at(Duration::from_millis(400), size).expect("in");
    assert!(early.glow.alpha < settled.glow.alpha);
    assert!((settled.glow.alpha - world::GLOW_STRENGTH).abs() < 1e-9);
    assert_eq!(settled.glow.hue, 200.0);
    assert_eq!(settled.bounds[2], 2.0 * world::GLOW_RADII.0, "the radii at a height of 800");
    assert_eq!(settled.stops[0].0, 0.0);
    assert_eq!(settled.stops[3].1.a, 0, "clear at the edge");
    sky.set_glow_hue(None, Duration::from_millis(400));
    assert!(sky.glow_at(Duration::from_millis(800), size).is_none(), "gone after Pan");
}

#[test]
fn kit_7_5_a_screens_stars_hold_still_and_the_glow_settles_after_pan() {
    let size = [1280.0, 800.0];
    let mut screens = SkyState::new(7, 40, false);
    screens.tick(0.0, size, 1.0);
    let first = screens.points.clone();
    screens.tick(5.3, size, 1.0);
    assert_eq!(screens.points, first, "a screen's stars at their twinkle for time 0, whatever the clock");
    let mut splash = SkyState::new(7, 40, true);
    splash.tick(0.0, size, 1.0);
    let first = splash.points.clone();
    splash.tick(5.3, size, 1.0);
    assert_ne!(splash.points, first, "the splash's stars twinkle");
    assert!(screens.glow_settled(Duration::ZERO), "no glow, nothing to ease");
    screens.set_glow_hue(Some(200.0), Duration::from_secs(1));
    assert!(!screens.glow_settled(Duration::from_millis(1100)), "easing in");
    assert!(screens.glow_settled(Duration::from_millis(1400)), "at rest after Pan");
    screens.set_glow_centre([0.3, 0.6], Duration::from_millis(1400));
    assert!(!screens.glow_settled(Duration::from_millis(1500)), "a move eases too");
}
