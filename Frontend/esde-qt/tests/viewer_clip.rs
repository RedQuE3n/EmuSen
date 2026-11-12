//! The media viewer's video as the player sees and hears it, offscreen on the software backend (EmuSen_Qt.md §4.14): with
//! Keep videos running on, the clip plays on behind a picture and is where it would be on return; off, it starts again;
//! Play audio for media viewer videos is the clip's own muted state; and it plays at the player's Volume. The clip and the picture are the test's own.

mod common;

use common::{Scratch, bmp, settings, until, write_theme};
use emusen_kit_core::input::UiButton;
use emusen_qt_test::{Harness, Scene};

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/View.qml");

/// A number of the viewer's clip player, read through the item tree; none while no clip plays.
fn playback(scene: &mut Scene<'_>, property: &str) -> Option<f64> {
    let id = scene.tree().nodes.iter().find(|n| n.name == "videoPlayback").map(|n| n.id.clone())?;
    Some(scene.item_number(&id, property)).filter(|v| !v.is_nan())
}

/// The clip's position in milliseconds once it plays; none while no clip plays.
fn position(scene: &mut Scene<'_>) -> Option<i64> {
    playback(scene, "position").filter(|p| *p > 0.0).map(|p| p as i64)
}

/// The viewer opened on the clip, played past `ms`, then a picture shown for `away` ms and the clip shown again: the
/// position before leaving, while the picture shows (none if no clip plays), and once back.
fn there_and_back(scene: &mut Scene<'_>, ms: i64, away: i32) -> (i64, Option<i64>, i64) {
    scene.pad(UiButton::Screensaver);
    until(scene, "the clip played", |s| position(s).is_some_and(|p| p > ms));
    let before = position(scene).unwrap();
    scene.pad(UiButton::Right);
    scene.wait(away);
    let behind = position(scene);
    scene.pad(UiButton::Left);
    until(scene, "the clip shown again", |s| position(s).is_some());
    let back = position(scene).unwrap();
    (before, behind, back)
}

#[test]
fn the_viewer_s_video_plays_on_behind_a_picture_when_kept_running_and_is_heard_at_the_volume_when_its_audio_is_on() {
    assert!(emusen_esde_qt::marks::mark_named("star").is_some(), "the module is linked into this test");
    let root = Scratch::new("viewer-clip");
    write_theme(&root.path("theme"), "", "<view name=\"gamelist\"><textlist name=\"games\"><pos>0.05 0.1</pos><size>0.4 0.8</size></textlist></view>");
    let clip = root.path("clip.y4m");
    emusen_qt_test::clips::write(&clip, 20);
    let shot = root.path("shot.bmp");
    bmp(&shot, 64, 56, |_, _| [0, 160, 0]);
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, 640, 400);
    let media = format!("{{\"nes\":{{\"video\":{{\"nes-0000\":\"{}\"}},\"screenshot\":{{\"nes-0000\":\"{}\"}}}}}}", clip.display(), shot.display());
    assert!(scene.set_property("view", "media", &media));
    let shelf = "[{\"name\":\"nes\",\"full_name\":\"Nintendo\",\"theme\":\"nes\",\"kind\":\"regular\",\"games\":[{\"file\":\"nes-0000.rom\",\"name\":\"Game\",\"esde_system\":\"nes\",\"favourite\":false,\"last_played\":0}]}]";
    let open = |scene: &mut Scene<'_>, keep: bool, audio: bool| {
        let interface = format!("\"DisplayClock\":false,\"StartupView\":\"gamelist\",\"MediaViewerKeepVideoRunning\":{keep},\"MediaViewerVideoAudio\":{audio}");
        let with_volume = settings(&root.path("theme"), &interface).replacen('{', "{\"volume\":0.35,", 1);
        assert!(scene.set_property("view", "settings", &with_volume));
        assert!(scene.set_property("view", "shelves", shelf));
        until(scene, "the theme loaded", |s| s.property("esdeModel", "themed") == "true");
        until(scene, "the gamelist", |s| s.property("esdeModel", "view") == "gamelist");
    };

    // On (ES-DE's default): the clip plays on, unseen, behind the picture, and is further on when shown again; heard.
    open(&mut scene, true, true);
    let (before, behind, back) = there_and_back(&mut scene, 1500, 1500);
    assert!(behind.is_some_and(|p| p > before), "the clip plays on behind the picture: {before} then {behind:?}");
    assert!(back >= before + 1200, "the clip resumes where it would be, not from the start: {before} then {back}");
    assert_eq!(playback(&mut scene, "heard"), Some(1.0), "the viewer's clip heard with its audio on");
    assert!(playback(&mut scene, "level").is_some_and(|v| (v - 0.35).abs() < 1e-6), "at the player's Volume, 35 %");
    assert_eq!(playback(&mut scene, "fade"), Some(600.0), "faded in over ES-DE's 600 ms (R-20261111-101)");
    scene.pad(UiButton::Back);
    until(&mut scene, "the viewer closed", |s| position(s).is_none());

    // Keep videos running off, its audio on: the clip goes with the picture shown and starts again on return, still heard.
    open(&mut scene, false, true);
    let (before, behind, back) = there_and_back(&mut scene, 1500, 1500);
    assert_eq!(behind, None, "no clip plays behind the picture: {before}");
    assert!(back < before, "the clip starts again: {before} then {back}");
    assert_eq!(playback(&mut scene, "heard"), Some(1.0), "Keep videos running off leaves the sound alone");
    scene.pad(UiButton::Back);
    until(&mut scene, "the viewer closed", |s| position(s).is_none());

    // Its audio off, kept running: silent, and still playing on behind a picture.
    open(&mut scene, true, false);
    let (before, behind, _) = there_and_back(&mut scene, 1500, 1500);
    assert!(behind.is_some_and(|p| p > before), "audio off leaves the clip running behind the picture: {before} then {behind:?}");
    assert_eq!(playback(&mut scene, "heard"), Some(0.0), "the viewer's clip silent with its audio off");
    scene.pad(UiButton::Back);
    scene.assert_quiet();
}
