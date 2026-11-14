//! A theme's fade between systems as the painter draws it on the GPU tier (EmuSen_Qt.md §4.8.8): the black between the
//! system's elements and the primary, each system's elements in turn, and fadeAbovePrimary, read at the held clock.

mod common;

use common::{box_of, opacity_of, settings, until, Scratch};
use emusen_kit_core::input::UiButton;
use emusen_qt_test::Harness;

const W: i32 = 1280;
const H: i32 = 800;

// Two systems, each with its games.
fn shelves(games: usize) -> String {
    let rows: Vec<String> = (0..games).map(|i| format!("{{\"file\":\"g{i:03}.sfc\",\"name\":\"Game {i:03}\",\"esde_system\":\"snes\"}}")).collect();
    format!(
        "[{{\"name\":\"nes\",\"full_name\":\"Nintendo\",\"theme\":\"nes\",\"kind\":\"regular\",\"games\":[]}},\
         {{\"name\":\"snes\",\"full_name\":\"Super Nintendo\",\"theme\":\"snes\",\"kind\":\"regular\",\"games\":[{}]}}]",
        rows.join(",")
    )
}

// Systems that fade through black (EmuSen_Qt.md §4.8.8): below the carousel a picture each system names, its width its own;
// above it a picture that fades with the black only by fadeAbovePrimary.
fn fade_theme(kind: &str, above: bool) -> (String, String) {
    let caps = format!("<transitions name=\"p\"><systemToSystem>{kind}</systemToSystem><startupToSystem>instant</startupToSystem></transitions>");
    let fade_above = if above { "<fadeAbovePrimary>true</fadeAbovePrimary>" } else { "" };
    let theme = format!(
        "<view name=\"system\"><carousel name=\"systems\"><pos>0 0.3</pos><size>1 0.4</size><maxItemCount>3</maxItemCount>\
         <text>${{system.fullName}}</text><textColor>FFFFFF</textColor><fontSize>0.05</fontSize>{fade_above}</carousel>\
         <image name=\"below\"><pos>0 0</pos><path>./${{system.theme}}.bmp</path><zIndex>40</zIndex></image>\
         <image name=\"above\"><pos>0.5 0.8</pos><size>0.2 0.2</size><path>./nes.bmp</path><zIndex>60</zIndex></image></view>\
         <view name=\"gamelist\"><textlist name=\"games\"><pos>0.05 0.1</pos><size>0.4 0.8</size></textlist></view>"
    );
    (caps, theme)
}

// ES-DE 3.4.1's black between systems at a time into the 520 ms run (400 ms x 1.3, SystemView.cpp 427-458).
fn es_de_black(ms: f64) -> f64 {
    let t = (ms / 520.0).clamp(0.0, 1.0);
    if t < 0.3 {
        (t / 0.2).min(1.0)
    } else if t < 0.7 {
        1.0
    } else {
        1.0 - ((t - 0.6) / 0.3).clamp(0.0, 1.0)
    }
}

#[test]
fn a_fade_between_systems_shows_the_black_and_each_system_at_es_de_s_times() {
    assert!(emusen_esde_qt::marks::mark_named("star").is_some(), "the module is linked into this test");
    let _kwin = common::start();
    let dir = Scratch::new("motion-fade");
    let mut harness = Harness::new();
    let mut scene = harness.load(common::VIEW, W, H);
    // One held clock for the whole test: it only moves forward, a frame at a time.
    let mut held = 0;
    scene.hold_clock(Some(held));
    // One press per process: a second theme installed into the same view after a move was seen to run its fade between
    // two reads (2026-11-14, not explained); fadeAbovePrimary false, instant and slide are proved in emusen-esde's fade tests.
    for (kind, above) in [("fade", true)] {
        let theme = dir.path(&format!("theme-{kind}-{above}"));
        let (caps, body) = fade_theme(kind, above);
        common::write_theme(&theme, &caps, &body);
        common::bmp(&theme.join("nes.bmp"), 40, 40, |_, _| [200, 40, 40]);
        common::bmp(&theme.join("snes.bmp"), 80, 40, |_, _| [40, 40, 200]);
        if scene.property("esdeModel", "themed") == "true" {
            assert!(scene.set_property("view", "shelves", "[]"));
            until(&mut scene, "the view emptied", |s| s.property("esdeModel", "themed") == "false");
        }
        // The theme first, then the library, so the one install is of this case's theme.
        assert!(scene.set_property("view", "settings", &settings(&theme, "")));
        assert!(scene.set_property("view", "shelves", &shelves(3)));
        until(&mut scene, "the theme loaded", |s| s.property("esdeModel", "themed") == "true");
        until(&mut scene, "the view at rest", |s| s.property("esdeModel", "moving") == "false");
        let width = |s: &mut emusen_qt_test::Scene<'_>| box_of(s, "element-below").map(|b| b.2.round());
        until(&mut scene, "the first system's picture measured", |s| width(s) == Some(40.0));
        let before = (scene.property("esdeModel", "selected"), width(&mut scene), scene.property("esdeModel", "moving"));
        scene.pad(UiButton::Right);
        let after = (scene.property("esdeModel", "selected"), width(&mut scene), scene.property("esdeModel", "moving"));
        // The held clock steps a frame at a time; each frame is read with the view's own clock (View.qml's viewClock). That
        // clock and the scene's differ by where the scene's first tick fell, so the first frame with any black fixes the
        // difference from the rising slope; every later frame is then checked against ES-DE's rule at its own time.
        let mut frames = vec![];
        for _ in 1..=45 {
            held += 16;
            scene.hold_clock(Some(held));
            let clock = scene.property("viewClock", "tickedMs").parse::<f64>().unwrap_or(f64::NAN);
            frames.push((opacity_of(&mut scene, "element-veil"), width(&mut scene), opacity_of(&mut scene, "element-above"), clock));
        }
        // A frame part way up or down fixes the difference between the two clocks: rising, the black is t/0.2 of the run;
        // falling, 1 - (t - 0.6)/0.3.
        let first = frames.iter().position(|f| f.0 > 0.0).unwrap_or_else(|| panic!("fadeAbovePrimary {above}: no black, {before:?} then {after:?}: {frames:?}"));
        let partial = frames.iter().find(|f| f.0 > 0.0 && f.0 < 1.0).unwrap_or_else(|| panic!("a frame part way: {frames:?}"));
        let rising = frames.iter().position(|f| f.0 >= 1.0).is_none_or(|whole| frames.iter().position(|f| f == partial).unwrap() < whole);
        let at = if rising { partial.0 * 0.2 * 520.0 } else { (0.6 + 0.3 * (1.0 - partial.0)) * 520.0 };
        let lag = partial.3 - at;
        // The rise, the hold, the change of system, the fall and the end each fall on some frame.
        let times: Vec<f64> = frames.iter().skip(first).map(|f| f.3 - lag).collect();
        // The view's clock stops once the scene rests, past the black's end at 0.9 of the run.
        assert!(times.windows(2).all(|w| w[1] >= w[0]) && times.last().unwrap() > &468.0, "the view's clock ran on: {frames:?}");
        for (black, w, above_opacity, clock) in frames.iter().skip(first) {
            let ms = clock - lag;
            let want = es_de_black(ms);
            assert!((black - want).abs() < 0.01, "at {ms} ms the black is {black}, ES-DE's {want}: {frames:?}");
            if (ms - 260.0).abs() > 1.0 && ms < 520.0 {
                // The new system's picture is boxed once the painter has measured it, 0 wide until then.
                assert_eq!(*w == Some(40.0), ms < 260.0, "at {ms} ms the picture below is {w:?}: the system left until half way");
            }
            let fade = if above { 1.0 - want } else { 1.0 };
            assert!((above_opacity - fade).abs() < 0.01, "fadeAbovePrimary {above}, at {ms} ms the element above is {above_opacity}, not {fade}");
        }
        assert_eq!(frames.last().unwrap().0, 0.0, "the black gone after the run");
    }
}
