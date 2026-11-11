//! The view's load at a library's scale (EmuSen_Qt.md §4.9): the GUI thread's part hands the four texts to the load's
//! thread without turning them into Rust text, and the shelves' text that does not read is a problem, never an empty view.

use cxx_qt_lib::QString;
use emusen_esde_qt::model::LoadTexts;

#[test]
fn the_gui_thread_s_part_of_a_load_stays_under_two_milliseconds_at_nine_thousand_games() {
    // The texts' sizes at 9,007 games (§4.9): the shelves 6.6 MB, the media index 2.2 MB.
    let row = r#"{"file":"/roms/snes/Game 0000.sfc","name":"Game 0000","esde_system":"snes","description":"A game invented for EmuSen's tests, long enough to wrap.","developer":"Studio A","publisher":"Publisher P","genre":"Puzzle","players":"1","release_date":"19900101T000000","favourite":false},"#;
    let shelves = QString::from(format!("[{{\"name\":\"snes\",\"games\":[{}]}}]", row.repeat(9000).trim_end_matches(',')).as_str());
    let media = QString::from("m".repeat(2_200_000).as_str());
    let (settings, presence) = (QString::from("{\"theme\":\"t\"}"), QString::from("{}"));
    assert!(shelves.as_slice().len() > 2_000_000, "a library's scale");
    let mut best = std::time::Duration::MAX;
    for _ in 0..3 {
        let started = std::time::Instant::now();
        let texts = LoadTexts::of([&settings, &shelves, &media, &presence]);
        best = best.min(started.elapsed());
        assert_eq!(texts.0[1].as_slice().as_ptr(), shelves.as_slice().as_ptr(), "the shelves' text shared, not copied");
        let read = std::thread::spawn(move || texts.read()).join().unwrap();
        assert_eq!(read[1], shelves.to_string(), "read whole on the load's thread");
        assert_eq!((read[0].as_str(), read[3].as_str()), ("{\"theme\":\"t\"}", "{}"), "each text in its own place");
    }
    eprintln!("the GUI thread's part of a load: {best:?}");
    assert!(best < std::time::Duration::from_millis(2), "{best:?}");
}
