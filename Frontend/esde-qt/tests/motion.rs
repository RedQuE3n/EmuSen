//! The scene's motion as the painter draws it on the GPU tier (EmuSen_Qt.md §4.8.5): the slide with the help and the
//! clock held still, `scrollFadeIn`, and a held list fading its metadata out from the first repeat and back after release.

mod common;

use common::{box_of, opacity_of, settings, until, Scratch};
use emusen_kit_core::input::UiButton;
use emusen_qt_test::Harness;

const W: i32 = 1280;
const H: i32 = 800;

const CAPABILITIES: &str =
    "<transitions name=\"slide\"><systemToGamelist>slide</systemToGamelist><gamelistToSystem>slide</gamelistToSystem></transitions>";

// A carousel of systems; a list, its developer, a cover that fades in on each new game; the help and the clock in both views.
const THEME: &str = "<view name=\"system, gamelist\"><helpsystem name=\"help\"><pos>0.02 0.94</pos><textColor>FFFFFF</textColor><iconColor>FFFFFF</iconColor></helpsystem>\
    <clock name=\"clock\"><pos>0.8 0.02</pos><size>0.18 0.05</size><fontSize>0.04</fontSize><color>FFFFFF</color></clock></view>\
    <view name=\"system\"><carousel name=\"systems\"><pos>0 0.3</pos><size>1 0.4</size><maxItemCount>3</maxItemCount>\
    <text>${system.fullName}</text><textColor>FFFFFF</textColor><fontSize>0.05</fontSize></carousel></view>\
    <view name=\"gamelist\"><textlist name=\"games\"><pos>0.05 0.1</pos><size>0.4 0.8</size><fontSize>0.04</fontSize>\
    <primaryColor>FFFFFF</primaryColor><selectedColor>FFFF00</selectedColor><selectorColor>404080</selectorColor></textlist>\
    <text name=\"dev\"><pos>0.55 0.1</pos><size>0.4 0.08</size><metadata>developer</metadata><fontSize>0.05</fontSize><color>FFFFFF</color></text>\
    <image name=\"art\"><pos>0.55 0.3</pos><size>0.3 0.5</size><imageType>cover</imageType><scrollFadeIn>true</scrollFadeIn></image></view>";

fn shelves(games: usize) -> String {
    let rows: Vec<String> = (0..games)
        .map(|i| format!("{{\"file\":\"g{i:03}.sfc\",\"name\":\"Game {i:03}\",\"esde_system\":\"snes\",\"developer\":\"Studio {i}\"}}"))
        .collect();
    format!(
        "[{{\"name\":\"nes\",\"full_name\":\"Nintendo\",\"theme\":\"nes\",\"kind\":\"regular\",\"games\":[]}},\
         {{\"name\":\"snes\",\"full_name\":\"Super Nintendo\",\"theme\":\"snes\",\"kind\":\"regular\",\"games\":[{}]}}]",
        rows.join(",")
    )
}

#[test]
fn the_slide_holds_the_overlays_still_and_a_held_list_fades_its_metadata() {
    assert!(emusen_esde_qt::marks::mark_named("star").is_some(), "the module is linked into this test");
    let _kwin = common::start();
    let dir = Scratch::new("motion");
    let theme = dir.path("theme");
    common::write_theme(&theme, CAPABILITIES, THEME);
    let cover = dir.path("cover.bmp");
    common::halves(&cover, 60, 80, [255, 140, 0], [0, 128, 128]);
    let media: Vec<String> = (0..60).map(|i| format!("\"g{i:03}\":\"{}\"", cover.display())).collect();
    let media = format!("{{\"snes\":{{\"cover\":{{{}}}}}}}", media.join(","));

    let mut harness = Harness::new();
    let mut scene = harness.load(common::VIEW, W, H);
    assert!(scene.set_property("view", "media", &media));
    assert!(scene.set_property("view", "shelves", &shelves(60)));
    assert!(scene.set_property("view", "settings", &settings(&theme, "\"DisplayClock\":true")));
    until(&mut scene, "the theme loaded", |s| s.property("esdeModel", "themed") == "true");
    assert_eq!(scene.property("esdeModel", "problem"), "");
    scene.pad(UiButton::Right);
    until(&mut scene, "the step settled", |s| s.property("esdeModel", "moving") == "false");
    let help = box_of(&mut scene, "element-help").expect("the help drawn");
    let clock = box_of(&mut scene, "element-clock").expect("the clock drawn");

    // The slide: the gamelist comes up over 402 ms while the help and the clock stay where they are.
    scene.pad(UiButton::Accept);
    scene.wait(120);
    assert_eq!(scene.property("esdeModel", "moving"), "true", "the slide is under way");
    let list = box_of(&mut scene, "element-games").expect("the coming list drawn during the slide");
    assert!(list.1 > 0.1 * f64::from(H) + 20.0, "the list is still below its place: {list:?}");
    assert_eq!(box_of(&mut scene, "element-help"), Some(help), "the help held still through the slide");
    assert_eq!(box_of(&mut scene, "element-clock"), Some(clock), "the clock held still through the slide");
    until(&mut scene, "the slide settled", |s| s.property("esdeModel", "view") == "gamelist" && s.property("esdeModel", "moving") == "false");
    let rested = box_of(&mut scene, "element-games").unwrap();
    assert!((rested.1 - 0.1 * f64::from(H)).abs() < 1.0, "the list rests at its place: {rested:?}");
    assert_eq!(box_of(&mut scene, "element-help"), Some(help));

    // scrollFadeIn: a new game's cover comes in from half over 326 ms; the view's opening did not fade it.
    let opened = opacity_of(&mut scene, "element-art");
    assert!((opened - 1.0).abs() < 1e-6, "the cover at full when the view opens: {opened}; {:?}", box_of(&mut scene, "element-art"));
    scene.pad(UiButton::Down);
    scene.wait(60);
    let fading = opacity_of(&mut scene, "element-art");
    assert!(fading > 0.45 && fading < 0.95, "the cover fades in from 0.5 after a step: {fading}");
    until(&mut scene, "the fade ended", |s| s.property("esdeModel", "moving") == "false");
    assert!((opacity_of(&mut scene, "element-art") - 1.0).abs() < 1e-6, "the cover back at full after 326 ms");

    // A held list: the metadata at full before the first repeat (500 ms), out by 149 ms after it, back after release.
    let frames = scene.frames();
    scene.pad_down(UiButton::Down);
    scene.wait(300);
    assert!((opacity_of(&mut scene, "element-dev") - 1.0).abs() < 1e-6, "the metadata at full before the first repeat");
    scene.wait(500);
    assert!(opacity_of(&mut scene, "element-dev") < 1e-6, "the metadata out by 149 ms after the first repeat");
    assert!(scene.frames() > frames + 20, "frames swapped while the list is held");
    assert_eq!(scene.property("esdeModel", "moving"), "true");
    scene.pad_up(UiButton::Down);
    until(&mut scene, "the list rested", |s| s.property("esdeModel", "moving") == "false");
    assert!((opacity_of(&mut scene, "element-dev") - 1.0).abs() < 1e-6, "the metadata back after the release");
    assert_eq!(scene.property("esdeModel", "view"), "gamelist");
    scene.assert_still("the list after the hold");
    scene.assert_quiet();
}
