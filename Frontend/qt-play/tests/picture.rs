//! The game picture follows the player's display settings, in a real window (EmuSen_Qt.md §1.8.10): the client
//! fixture's console core, a PlayStation started empty, its 8 by 4 picture drawn at its screen's 4:3. With nothing set
//! it is drawn plain, each of its pixels a flat block; the CRT chosen while the game is paused is drawn from the resume
//! on, and no block is flat; a crop of the television's overscan, which the CRT makes itself, keeps the even 4:3; a
//! custom crop of a plain picture is clipped by the window, and square pixels draw the frame at its own 2:1. Qt adopts
//! Serenity's device in a compositor of the test's own, on the RX 6800 (EMUSEN_GPU_DEVICE names another).

mod common;

use common::{approve, assert_qt_grab_only, device_and_feed, play_window, until, vulkan_qt};
use emusen_client_fixture::{console_core, library, World};
use emusen_kit_core::input::PadControl;
use emusen_platform_client::galaxia::consts::{FILE_GRAPHICS, MODEL_GRAPHICS_CONFIG};
use emusen_platform_client::Platform;
use emusen_qt_endymion::reading::sdl_button;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::virtual_pad::{Sdl, VirtualPad};
use emusen_qt_test::{Harness, Scene};
use serde_json::{json, Value};
use std::sync::atomic::Ordering;

/// The console core's picture, in pixels, and the console its key starts.
const FRAME: (f64, f64) = (8.0, 4.0);
const CONSOLE: &str = "PlayStation";

/// The graphics model saved where the program reads it.
fn save_graphics(platform: &Platform, document: &Value) {
    let file = platform.file(FILE_GRAPHICS).unwrap();
    let name = file.file_name().unwrap().to_str().unwrap();
    platform.model_save(MODEL_GRAPHICS_CONFIG, None, name, document).unwrap();
}

/// A node's place in the window: (x, y, width, height).
fn place(scene: &mut Scene<'_>, name: &str) -> (f64, f64, f64, f64) {
    let tree = scene.tree();
    tree.nodes.iter().find(|n| n.name == name).map(|n| (n.x, n.y, n.w, n.h)).unwrap_or_else(|| panic!("the {name} item"))
}

/// The chord, pressed and let go, and the session paused or resumed by it.
fn chord(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, paused: bool) {
    let guide = sdl_button(PadControl::Guide).unwrap();
    sdl.press(pad, guide, true);
    until(scene, "the chord answered", |s| s.property("session", "paused") == paused.to_string());
    sdl.press(pad, guide, false);
    scene.wait(150);
}

/// For each of the frame's pixels whose middle is shown, whether three points down that middle read one colour: a
/// block drawn flat.
fn flat_blocks(scene: &mut Scene<'_>) -> Vec<bool> {
    let (x, y, w, h) = place(scene, "picture");
    let (sx, _, sw, _) = place(scene, "shown");
    let (bw, bh) = (w / FRAME.0, h / FRAME.1);
    let mut points = Vec::new();
    for row in 0..FRAME.1 as usize {
        for column in 0..FRAME.0 as usize {
            let (cx, top) = (x + (column as f64 + 0.5) * bw, y + row as f64 * bh);
            if cx > sx + 1.0 && cx < sx + sw - 1.0 {
                points.extend([0.3, 0.5, 0.7].map(|f| (cx, top + f * bh)));
            }
        }
    }
    let colours = scene.colours_at(&points);
    colours.chunks(3).map(|c| c.iter().all(|p| p.is_some() && *p == c[0])).collect()
}

/// The mean of red, green and blue over the shown rectangle's middle, as a grab reads it.
fn brightness(scene: &mut Scene<'_>) -> f64 {
    let (x, y, w, h) = place(scene, "shown");
    let points: Vec<(f64, f64)> = (1..8).flat_map(|i| (1..4).map(move |j| (x + w * f64::from(i) / 8.0, y + h * f64::from(j) / 4.0))).collect();
    let seen: Vec<_> = scene.colours_at(&points).into_iter().flatten().collect();
    seen.iter().map(|c| f64::from(c.r) + f64::from(c.g) + f64::from(c.b)).sum::<f64>() / (3.0 * seen.len().max(1) as f64)
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1.0
}

#[test]
fn the_picture_is_drawn_through_the_console_s_filter_crop_and_shape_as_the_settings_say() {
    let _kwin = Compositor::start(1280, 800);
    vulkan_qt();
    let world = World::new("play-picture");
    let core = world.player_library(&world.dir("Home/Cores"), console_core());
    approve(&core, None, &world);
    let platform = Platform::load(library()).unwrap();
    let sdl = Sdl::open();
    let pad = sdl.attach("EmuSen Test Xbox Pad", 0x045e, 0x028e);

    let (device, feed, adoption) = device_and_feed(&world.dir("presets"));
    let mut harness = Harness::new();
    let mut scene = play_window(&mut harness, &adoption);
    assert!(scene.set_property("play", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("play", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("play", "feed", &feed.address().to_string()));
    assert!(scene.set_property("play", "consoleKey", CONSOLE));
    until(&mut scene, "the console running", |s| {
        let problem = s.property("session", "problem");
        assert!(problem.is_empty() || problem.starts_with("The game plays without sound"), "the session: {problem}");
        s.property("session", "running") == "true"
    });
    until(&mut scene, "its first picture", |s| s.property("session", "firstPictureMs").parse::<f64>().is_ok_and(|ms| ms > 0.0));

    // Nothing set: plain, whole, at the screen's 4:3, the picture item the shown rectangle itself.
    chord(&mut scene, &sdl, &pad, true);
    // The window's own height, which its decoration may make less than asked for.
    let (_, _, ww, wh) = place(&mut scene, "bars");
    assert!(ww > wh * 1.5, "a window wider than 3:2: {ww} by {wh}");
    let (sx, sy, sw, sh) = place(&mut scene, "shown");
    assert!(near(sw, wh * 4.0 / 3.0) && near(sh, wh), "4:3 in {ww} by {wh}: {sw} by {sh}");
    assert_eq!(place(&mut scene, "picture"), (sx, sy, sw, sh), "nothing cropped");
    let plain = flat_blocks(&mut scene);
    assert!(plain.iter().all(|f| *f), "drawn plain, each pixel a flat block: {plain:?}");

    // The CRT, Balanced, and the television's overscan, chosen while the game is paused, drawn from the resume.
    save_graphics(
        &platform,
        &json!({"Consoles": {CONSOLE: {"ScreenFilter": "CRT", "Overscan": "tv"}}, "ShaderParameters": {CONSOLE: {"CRT": {"quality": "1"}}}}),
    );
    let shown = feed.counts().shown.load(Ordering::Relaxed);
    chord(&mut scene, &sdl, &pad, false);
    until(&mut scene, "pictures through the CRT", |_| feed.counts().shown.load(Ordering::Relaxed) > shown + 30);
    chord(&mut scene, &sdl, &pad, true);
    assert_eq!(feed.problem(), None, "the CRT's chain built and drew");
    let (cx, cy, cw, ch) = place(&mut scene, "shown");
    assert!(near(cw, sw) && near(ch, sh) && near(cx, sx) && near(cy, sy), "an even crop the CRT makes keeps 4:3: {cw} by {ch}");
    assert_eq!(place(&mut scene, "picture"), (cx, cy, cw, ch), "the CRT crops itself; the window clips nothing");
    let crt = flat_blocks(&mut scene);
    assert!(crt.iter().all(|f| !*f), "through the CRT no pixel is a flat block: {crt:?}");

    // A lever set on the running chain with no chain built again: the CRT's contrast down to its least, and the picture
    // drawn darker from the next frames; a parameter the chain does not declare is refused.
    let handoff = feed.handoff();
    assert!(!handoff.set_value("no_such_parameter", 1.0), "an undeclared parameter is refused");
    assert!(handoff.declared().iter().any(|d| d == "contrast"), "the CRT declares its contrast: {:?}", handoff.declared());
    let before = brightness(&mut scene);
    assert!(handoff.set_value("contrast", 0.5));
    let shown = feed.counts().shown.load(Ordering::Relaxed);
    chord(&mut scene, &sdl, &pad, false);
    until(&mut scene, "pictures at the lower contrast", |_| feed.counts().shown.load(Ordering::Relaxed) > shown + 30);
    chord(&mut scene, &sdl, &pad, true);
    let after = brightness(&mut scene);
    assert!(after < before * 0.9, "the lever darkened the picture at once: {before} then {after}");
    assert_eq!(feed.problem(), None, "the values set on the chain drawn");

    // Plain again, a custom crop of 10% at the left and 20% at the right, and square pixels: 8 by 4 is 2:1, 70% of it 1.4:1.
    save_graphics(
        &platform,
        &json!({"Consoles": {CONSOLE: {"ScreenFilter": "effect.None.nearest", "Overscan": "custom", "CropLeft": "10", "CropRight": "20", "PictureShape": "square"}}}),
    );
    let shown = feed.counts().shown.load(Ordering::Relaxed);
    chord(&mut scene, &sdl, &pad, false);
    until(&mut scene, "pictures plain again", |_| feed.counts().shown.load(Ordering::Relaxed) > shown + 30);
    chord(&mut scene, &sdl, &pad, true);
    let (qx, qy, qw, qh) = place(&mut scene, "shown");
    assert!(near(qw, wh * 1.4) && near(qh, wh), "square pixels, 70% kept: {qw} by {qh}");
    let (px, py, pw, ph) = place(&mut scene, "picture");
    assert!(near(pw, qw / 0.7) && near(ph, qh), "the whole picture drawn past the clip: {pw} by {ph}");
    assert!(near(px, qx - 0.1 * pw) && near(py, qy), "its left 10% outside the clip: {px} against {qx}");
    assert_eq!(scene.property("shown", "clip"), "true");
    let back = flat_blocks(&mut scene);
    assert!(back.iter().all(|f| *f), "plain again: {back:?}");

    assert_eq!(feed.problem(), None, "the render thread met no failure");
    assert_qt_grab_only(&device.findings().unwrap());
    sdl.detach(pad);
    drop(harness);
    drop(feed);
}
