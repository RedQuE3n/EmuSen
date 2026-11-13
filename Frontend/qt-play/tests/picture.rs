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

/// The mean blue over the middle of each of the frame's inner pixels, 27 points a pixel so that a mask's stripes and
/// the scanlines average out, as one grab reads it. The core's red rises with every frame, so two frames are compared on
/// its blue, which is one value for every pixel of every frame: 0xFF while its B is held, 0 otherwise (the test core's
/// `pressed_blue`).
fn blue(scene: &mut Scene<'_>) -> f64 {
    let (x, y, w, h) = place(scene, "shown");
    let (bw, bh) = (w / FRAME.0, h / FRAME.1);
    let mut points = Vec::new();
    for column in 1..7 {
        for row in 1..3 {
            for fx in 0..9 {
                for fy in [0.3, 0.5, 0.7] {
                    points.push((x + (f64::from(column) + 0.3 + 0.05 * f64::from(fx)) * bw, y + (f64::from(row) + fy) * bh));
                }
            }
        }
    }
    let seen: Vec<_> = scene.colours_at(&points).into_iter().flatten().collect();
    assert_eq!(seen.len(), points.len(), "the grab read every point of the picture's middle");
    seen.iter().map(|c| f64::from(c.b)).sum::<f64>() / seen.len() as f64
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
    // Read on the core's blue (see `blue`), of 255. Its B is the pad's South by the default bindings. A game given the
    // pad back hears nothing until every button is let go, and it is given back as its loop turns after the resume: so
    // South goes down once pictures are drawn again, the blue is waited for, and South comes up while paused, where the
    // interface does not take a button that went down under the game.
    let none = blue(&mut scene);
    let south = sdl_button(PadControl::South).unwrap();
    let pictures = |more: u64, scene: &mut Scene<'_>, what: &str| {
        let shown = feed.counts().shown.load(Ordering::Relaxed);
        until(scene, what, |_| feed.counts().shown.load(Ordering::Relaxed) > shown + more);
    };
    let held_blue = |scene: &mut Scene<'_>, what: &str| {
        chord(scene, &sdl, &pad, false);
        pictures(5, scene, "pictures after the resume");
        sdl.press(&pad, south, true);
        until(scene, "the held button's blue drawn", |s| blue(s) > none + 50.0);
        pictures(30, scene, what);
        chord(scene, &sdl, &pad, true);
        let level = blue(scene);
        sdl.press(&pad, south, false);
        level
    };
    let before = held_blue(&mut scene, "pictures with B held");
    assert!(before > none + 150.0, "the held button's full blue is drawn through the CRT: {none} with none held, {before} held");
    assert!(handoff.set_value("contrast", 0.5));
    let after = held_blue(&mut scene, "pictures at the lower contrast");
    // The CRT's light is (contrast × v + black)^gamma: at half contrast and gamma 2.4 a fifth of the light, about half the level drawn.
    let part = (after - none) / (before - none);
    assert!(part > 0.3 && part < 0.7, "the lever darkened the picture at once, by what half contrast comes to: {none} with none held, {before} then {after}");
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
