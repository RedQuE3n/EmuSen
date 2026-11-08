//! Serenity's display clock through the library (EmuSen_Qt.md §1.8.3): swaps at 60 Hz give their period, a game within a
//! hundredth of it locks to the display, one at 50 Hz does not, swaps seen together count one period each, and a look
//! before any swap is no sample.

use emusen_client_fixture::library;
use emusen_platform_client::pace::DisplayClock;
use emusen_platform_client::Platform;

#[test]
fn the_display_clock_reads_the_period_and_locks_a_game_within_a_hundredth_of_it() {
    let platform = Platform::load(library()).unwrap();
    let mut clock = DisplayClock::new(&platform);
    assert!(!clock.locks(16_666_667).unwrap(), "no swap seen yet");
    // A look before any swap, as a frame thread makes at its first frame: count 0 at time 0, no sample.
    clock.saw(0, 0).unwrap();
    clock.saw(1, 1_000_000_000 + 16_666_667).unwrap();
    clock.saw(2, 1_000_000_000 + 2 * 16_666_667).unwrap();
    assert!((clock.period_ns() - 16_666_667.0).abs() < 1.0, "the first two swaps give the period: {}", clock.period_ns());
    for n in 3..=10u64 {
        clock.saw(n, 1_000_000_000 + n as i64 * 16_666_667).unwrap();
    }
    assert!((clock.period_ns() - 16_666_667.0).abs() < 1.0, "{}", clock.period_ns());
    assert!(clock.locks(16_639_276).unwrap(), "the NES's 60.0988 Hz on a 60 Hz display");
    assert!(!clock.locks(20_000_000).unwrap(), "a 50 Hz game runs on its own clock");
    clock.saw(12, 1_000_000_000 + 12 * 16_666_667).unwrap();
    assert!((clock.period_ns() - 16_666_667.0).abs() < 1.0, "two swaps between looks are one period each");
}
