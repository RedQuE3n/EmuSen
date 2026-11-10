//! The reading on Endymion's simulated pads, through the platform library built from this commit (EmuSen_Kit.md §7.15):
//! every control's press, the left stick and the triggers, pads merged, the family of the pad last used, pads plugged in
//! and pulled out while reading, and the thread that hands each tick on.

use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use emusen_client_fixture::{library, World};
use emusen_kit_core::glyph::PadFamily;
use emusen_kit_core::input::UiButton;
use emusen_kit_qt::router::PadEvent;
use emusen_platform_client::pads::{consts, Pads, SimPad, SimSet};
use emusen_platform_client::Platform;
use emusen_qt_endymion::reader::Reader;
use emusen_qt_endymion::reading::{Reading, Tick};

fn down(button: UiButton) -> PadEvent {
    PadEvent { button, down: true }
}

fn up(button: UiButton) -> PadEvent {
    PadEvent { button, down: false }
}

/// What a tick says to the interface: its presses, the family and the count; the pads' events are the seats'.
fn seen(tick: Tick) -> (Vec<PadEvent>, Option<PadFamily>, Option<usize>) {
    (tick.events, tick.family, tick.connected)
}

fn pad(p: &Arc<Platform>, kind: i32, name: &str) -> SimPad {
    let pad = SimPad::new(p, None).unwrap();
    pad.set(consts::SIM_PAD_KIND, kind).unwrap();
    pad.set_text(consts::SIM_PAD_NAME, name).unwrap();
    pad
}

#[test]
fn every_control_gives_its_press_and_the_stick_and_triggers_past_their_thresholds() {
    let _world = World::new("reading-controls");
    let p = Platform::load(library()).unwrap();
    let set = SimSet::new(&p).unwrap();
    let one = pad(&p, 0, "Simulated pad");
    set.connect(&one).unwrap();
    let pads = Pads::simulated(&p, &set).unwrap();
    let mut reading = Reading::default();
    let first = reading.start(&pads).unwrap();
    assert_eq!(seen(first), seen(Tick { events: vec![], family: Some(PadFamily::Generic), connected: Some(1), ..Tick::default() }));
    assert!(reading.poll(&pads).unwrap().is_empty(), "nothing changed");

    // SDL's buttons by number, each with the press kit-core's table gives it.
    let buttons = [
        (0, UiButton::Accept),
        (1, UiButton::Back),
        (2, UiButton::Screensaver),
        (3, UiButton::Search),
        (4, UiButton::Options),
        (5, UiButton::Guide),
        (6, UiButton::Menu),
        (7, UiButton::Random),
        (8, UiButton::Random),
        (9, UiButton::PageUp),
        (10, UiButton::PageDown),
        (11, UiButton::Up),
        (12, UiButton::Down),
        (13, UiButton::Left),
        (14, UiButton::Right),
    ];
    for (sdl, button) in buttons {
        one.press(sdl, true).unwrap();
        assert_eq!(reading.poll(&pads).unwrap().events, [down(button)], "SDL button {sdl}");
        one.press(sdl, false).unwrap();
        assert_eq!(reading.poll(&pads).unwrap().events, [up(button)], "SDL button {sdl} let go");
    }

    // The left stick past 0.55 of its travel, and a trigger past half of its.
    let axes = [
        (1, -0.8, UiButton::Up),
        (1, 0.8, UiButton::Down),
        (0, -0.8, UiButton::Left),
        (0, 0.8, UiButton::Right),
        (4, 0.7, UiButton::First),
        (5, 0.7, UiButton::Last),
    ];
    for (axis, value, button) in axes {
        one.set_axis(axis, value * 0.6).unwrap();
        assert!(reading.poll(&pads).unwrap().events.is_empty(), "axis {axis} short of its threshold");
        one.set_axis(axis, value).unwrap();
        assert_eq!(reading.poll(&pads).unwrap().events, [down(button)], "axis {axis} at {value}");
        one.set_axis(axis, 0.0).unwrap();
        assert_eq!(reading.poll(&pads).unwrap().events, [up(button)], "axis {axis} let go");
    }

    // Two at once come in the kit's order.
    one.press(1, true).unwrap();
    one.press(11, true).unwrap();
    assert_eq!(reading.poll(&pads).unwrap().events, [down(UiButton::Up), down(UiButton::Back)]);
}

#[test]
fn pads_are_merged_the_last_used_gives_the_family_and_pads_come_and_go_while_reading() {
    let _world = World::new("reading-pads");
    let p = Platform::load(library()).unwrap();
    let set = SimSet::new(&p).unwrap();
    let sony = pad(&p, 6, "DualSense Wireless Controller");
    let nintendo = pad(&p, 1, "Nintendo Switch Pro Controller");
    set.connect(&sony).unwrap();
    set.connect(&nintendo).unwrap();
    let pads = Pads::simulated(&p, &set).unwrap();
    let mut reading = Reading::default();
    let first = reading.start(&pads).unwrap();
    assert_eq!((first.family, first.connected), (Some(PadFamily::PlayStation), Some(2)), "the first pad's family before any is used");

    // A button held on one pad stays held while the other presses and lets go of it.
    sony.press(12, true).unwrap();
    assert_eq!(reading.poll(&pads).unwrap().events, [down(UiButton::Down)]);
    nintendo.press(12, true).unwrap();
    let tick = reading.poll(&pads).unwrap();
    assert_eq!((tick.events.len(), tick.family), (0, Some(PadFamily::Nintendo)), "held already; the pad last used is the Nintendo's");
    nintendo.press(12, false).unwrap();
    assert!(reading.poll(&pads).unwrap().is_empty(), "the PlayStation pad still holds Down");
    sony.press(12, false).unwrap();
    assert_eq!(reading.poll(&pads).unwrap().events, [up(UiButton::Down)]);
    sony.press(0, true).unwrap();
    assert_eq!(reading.poll(&pads).unwrap().family, Some(PadFamily::PlayStation), "used again");
    sony.press(0, false).unwrap();
    reading.poll(&pads).unwrap();

    // The pad last used pulled out while it holds a button: the button goes up and the family falls to the first pad read.
    nintendo.press(0, true).unwrap();
    assert_eq!(
        seen(reading.poll(&pads).unwrap()),
        seen(Tick { events: vec![down(UiButton::Accept)], family: Some(PadFamily::Nintendo), connected: None, ..Tick::default() })
    );
    set.disconnect(&nintendo).unwrap();
    assert_eq!(
        seen(reading.poll(&pads).unwrap()),
        seen(Tick { events: vec![up(UiButton::Accept)], family: Some(PadFamily::PlayStation), connected: Some(1), ..Tick::default() })
    );

    // One plugged in while reading is read; with none left the family is the generic face.
    let xbox = pad(&p, 3, "Xbox Wireless Controller");
    set.connect(&xbox).unwrap();
    assert_eq!(reading.poll(&pads).unwrap().connected, Some(2));
    xbox.press(3, true).unwrap();
    assert_eq!(seen(reading.poll(&pads).unwrap()), seen(Tick { events: vec![down(UiButton::Search)], family: Some(PadFamily::Xbox), connected: None, ..Tick::default() }));
    set.disconnect(&xbox).unwrap();
    set.disconnect(&sony).unwrap();
    assert_eq!(
        seen(reading.poll(&pads).unwrap()),
        seen(Tick { events: vec![up(UiButton::Search)], family: Some(PadFamily::Generic), connected: Some(0), ..Tick::default() })
    );
}

#[test]
fn the_thread_hands_on_each_tick_and_stops_when_the_sink_refuses_or_the_reader_drops() {
    let _world = World::new("reading-thread");
    let p = Platform::load(library()).unwrap();
    let set = Arc::new(SimSet::new(&p).unwrap());
    let one = pad(&p, 5, "Wireless Controller");
    set.connect(&one).unwrap();
    let (sent, got) = mpsc::channel();
    let opener = (Arc::clone(&p), Arc::clone(&set));
    let reader =
        Reader::spawn(move || Pads::simulated(&opener.0, &opener.1).map_err(|e| e.to_string()), None, None, move |d| sent.send(d).is_ok());
    let wait = Duration::from_secs(2);
    let first = got.recv_timeout(wait).unwrap().unwrap();
    assert_eq!((first.family, first.connected), (Some(PadFamily::PlayStation), Some(1)));
    one.press(6, true).unwrap();
    assert_eq!(got.recv_timeout(wait).unwrap().unwrap().events, [down(UiButton::Menu)]);
    one.press(6, false).unwrap();
    assert_eq!(got.recv_timeout(wait).unwrap().unwrap().events, [up(UiButton::Menu)]);
    drop(reader);
    assert!(got.recv_timeout(Duration::from_millis(100)).is_err(), "nothing after the reader dropped");

    let (sent, got) = mpsc::channel();
    let refused = Reader::spawn(|| Err("no pads here".to_string()), None, None, move |d| sent.send(d).is_ok() && false);
    assert_eq!(got.recv_timeout(wait).unwrap(), Err("no pads here".to_string()), "a failed opening is said once");
    drop(refused);
}
