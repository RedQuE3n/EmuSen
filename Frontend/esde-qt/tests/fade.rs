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

// The view's own clock since the press: held near the time, then moved by what the view still lacks, as mapping.rs reads it.
fn read_at(scene: &mut emusen_qt_test::Scene<'_>, ms: i64) {
    let near = ms - 32;
    scene.hold_clock(Some(near));
    let since = |s: &mut emusen_qt_test::Scene<'_>| s.property("viewClock", "tickedMs").parse::<f64>().unwrap_or(f64::NAN);
    let lacking = (ms as f64 - since(scene)).round() as i64;
    scene.hold_clock(Some(near + lacking));
    let t = since(scene);
    assert!((t - ms as f64).abs() < 0.5, "the view read at {t} ms after the press, not {ms}");
}

#[test]
fn a_fade_between_systems_shows_the_black_and_each_system_at_es_de_s_times() {
    let _kwin = common::start();
    let dir = Scratch::new("motion-fade");
    let mut harness = Harness::new();
    let mut scene = harness.load(common::VIEW, W, H);
    for (kind, above) in [("fade", false), ("fade", true), ("instant", true), ("slide", true)] {
        let theme = dir.path(&format!("theme-{kind}-{above}"));
        let (caps, body) = fade_theme(kind, above);
        common::write_theme(&theme, &caps, &body);
        common::bmp(&theme.join("nes.bmp"), 40, 40, |_, _| [200, 40, 40]);
        common::bmp(&theme.join("snes.bmp"), 80, 40, |_, _| [40, 40, 200]);
        if scene.property("esdeModel", "themed") == "true" {
            assert!(scene.set_property("view", "shelves", "[]"));
            until(&mut scene, "the view emptied", |s| s.property("esdeModel", "themed") == "false");
        }
        scene.hold_clock(None);
        assert!(scene.set_property("view", "shelves", &shelves(3)));
        assert!(scene.set_property("view", "settings", &settings(&theme, "")));
        until(&mut scene, "the theme loaded", |s| s.property("esdeModel", "themed") == "true");
        until(&mut scene, "the view at rest", |s| s.property("esdeModel", "moving") == "false");
        let width = |s: &mut emusen_qt_test::Scene<'_>| box_of(s, "element-below").map(|b| b.2.round());
        until(&mut scene, "the first system's picture measured", |s| width(s) == Some(40.0));
        scene.hold_clock(Some(0));
        scene.pad(UiButton::Right);
        if kind != "fade" {
            read_at(&mut scene, 26);
            assert_eq!(opacity_of(&mut scene, "element-veil"), 0.0, "{kind}: no black");
            assert_eq!(width(&mut scene), Some(80.0), "{kind}: the new system at once");
            assert!((opacity_of(&mut scene, "element-above") - 1.0).abs() < 1e-6, "{kind}");
            continue;
        }
        // 520 ms: 400 x 1.3. At 0.05 the black is 0.25; whole from 0.2 to 0.7; 1/3 at 0.8; none from 0.9.
        read_at(&mut scene, 26);
        assert!((opacity_of(&mut scene, "element-veil") - 0.25).abs() < 0.01, "{}", opacity_of(&mut scene, "element-veil"));
        assert_eq!(width(&mut scene), Some(40.0), "the system left shows until half way");
        let expected = if above { 0.75 } else { 1.0 };
        assert!((opacity_of(&mut scene, "element-above") - expected).abs() < 0.01, "fadeAbovePrimary {above}: {}", opacity_of(&mut scene, "element-above"));
        read_at(&mut scene, 270);
        assert!((opacity_of(&mut scene, "element-veil") - 1.0).abs() < 0.01);
        assert_eq!(width(&mut scene), Some(80.0), "past half way the new system shows");
        read_at(&mut scene, 416);
        assert!((opacity_of(&mut scene, "element-veil") - 1.0 / 3.0).abs() < 0.01, "{}", opacity_of(&mut scene, "element-veil"));
        let expected = if above { 2.0 / 3.0 } else { 1.0 };
        assert!((opacity_of(&mut scene, "element-above") - expected).abs() < 0.01, "fadeAbovePrimary {above}");
        read_at(&mut scene, 600);
        assert_eq!(opacity_of(&mut scene, "element-veil"), 0.0, "the black gone");
    }
}
