//! A theme's video element on the GPU tier (THEMES.md, video; EmuSen_Qt.md §4.5.6): the game's still in the element's box
//! until the delay has passed, no player made before it, then the clip's frames in the box, advancing; heard at the
//! player's Volume with `audio` on and silent with it off; played its iterations and then its still again. The clip, the
//! still and the theme are the test's own.

mod common;

use common::{bmp, settings, until, Scratch};
use emusen_qt_test::clips::{self, BLUE, RED};
use emusen_qt_test::{Harness, Scene};

const W: i32 = 640;
const H: i32 = 400;
// The video's box: pos 0.5 0.1, size 0.4 0.5, so its middle is here.
const MIDDLE: (f64, f64) = (448.0, 140.0);

/// A number of the element's clip player, read through the item tree; none while no player is made.
fn playback(scene: &mut Scene<'_>, property: &str) -> Option<f64> {
    let id = scene.tree().nodes.iter().find(|n| n.name == "themeVideoPlayback").map(|n| n.id.clone())?;
    Some(scene.item_number(&id, property)).filter(|v| !v.is_nan())
}

fn colour(scene: &mut Scene<'_>) -> (u8, u8, u8) {
    let c = scene.colours_at(&[MIDDLE])[0].expect("the box drawn");
    (c.r, c.g, c.b)
}

fn green(c: (u8, u8, u8)) -> bool {
    c.1 > 120 && c.0 < 80 && c.2 < 80
}

fn red(c: (u8, u8, u8)) -> bool {
    c.0 > 180 && c.1 < 80 && c.2 < 80
}

fn blue(c: (u8, u8, u8)) -> bool {
    c.2 > 180 && c.0 < 80 && c.1 < 80
}

/// The gamelist under a theme in a folder of its own whose video element has `more` beside its box, still type and default clip.
fn open(scene: &mut Scene<'_>, root: &Scratch, name: &str, more: &str) -> std::time::Instant {
    let theme = root.path(name);
    let video = format!(
        "<view name=\"gamelist\"><textlist name=\"games\"><pos>0.02 0.1</pos><size>0.4 0.8</size></textlist>\
         <video name=\"clip\"><pos>0.5 0.1</pos><size>0.4 0.5</size><imageType>screenshot</imageType><default>{}</default>\
         <fadeInTime>0</fadeInTime>{more}</video></view>",
        root.path("clip.y4m").display()
    );
    if scene.property("esdeModel", "themed") == "true" {
        assert!(scene.set_property("view", "shelves", "[]"));
        until(scene, "the view emptied", |s| s.property("esdeModel", "themed") == "false");
    }
    common::write_theme(&theme, "", &video);
    let with_volume = settings(&theme, "\"DisplayClock\":false,\"StartupView\":\"gamelist\"").replacen('{', "{\"volume\":0.35,", 1);
    assert!(scene.set_property("view", "settings", &with_volume));
    let shelf = "[{\"name\":\"nes\",\"full_name\":\"Nintendo\",\"theme\":\"nes\",\"kind\":\"regular\",\"games\":[{\"file\":\"nes-0000.rom\",\"name\":\"Game\",\"esde_system\":\"nes\",\"favourite\":false,\"last_played\":0}]}]";
    assert!(scene.set_property("view", "shelves", shelf));
    until(scene, "the gamelist", |s| s.property("esdeModel", "themed") == "true" && s.property("esdeModel", "view") == "gamelist");
    std::time::Instant::now()
}

#[test]
fn a_theme_s_video_shows_its_still_until_the_delay_then_plays_its_frames_in_the_box() {
    assert!(emusen_esde_qt::marks::mark_named("star").is_some(), "the module is linked into this test");
    let _kwin = common::start();
    let root = Scratch::new("theme-video");
    // One second of red, then blue: a box that turns from red to blue has had its frames advance.
    clips::write_frames(&root.path("clip.y4m"), 64, 48, 2 * clips::RATE, |f, _, _| if f < clips::RATE { RED } else { BLUE });
    let shot = root.path("shot.bmp");
    bmp(&shot, 64, 48, |_, _| [0, 160, 0]);
    let mut harness = Harness::new();
    let mut scene = harness.load(common::VIEW, W, H);
    assert!(scene.set_property("view", "media", &format!("{{\"nes\":{{\"screenshot\":{{\"nes-0000\":\"{}\"}}}}}}", shot.display())));

    // The theme's 2 s delay: the still, and no player, until it has passed.
    let shown = open(&mut scene, &root, "theme-a", "<delay>2</delay>");
    until(&mut scene, "the still drawn", |s| green(colour(s)));
    let mut seen = vec![];
    while shown.elapsed().as_millis() < 1700 {
        seen.push((shown.elapsed().as_millis(), colour(&mut scene), playback(&mut scene, "position").is_some()));
        scene.wait(100);
    }
    assert!(seen.len() > 5 && seen.iter().all(|(_, c, player)| green(*c) && !player), "the still alone before the delay: {seen:?}");
    until(&mut scene, "the clip's first frames", |s| red(colour(s)));
    until(&mut scene, "the clip's later frames", |s| blue(colour(s)));
    assert_eq!(playback(&mut scene, "heard"), Some(1.0), "audio on by default");
    assert!(playback(&mut scene, "level").is_some_and(|v| (v - 0.35).abs() < 1e-6), "at the player's Volume, 35 %");

    // audio off: silent. One iteration with onIterationsDone image: the still again once the clip has ended.
    open(&mut scene, &root, "theme-b", "<delay>0.5</delay><audio>false</audio><iterationCount>1</iterationCount><onIterationsDone>image</onIterationsDone>");
    until(&mut scene, "the clip playing", |s| red(colour(s)));
    assert_eq!(playback(&mut scene, "heard"), Some(0.0), "the theme's audio off");
    until(&mut scene, "the clip ended and its player gone", |s| playback(s, "position").is_none());
    until(&mut scene, "the still once the iteration is done", |s| green(colour(s)));
    scene.assert_quiet();
}
