//! What the real-window tests share: the window adopting Serenity's device on the RX 6800 (EMUSEN_GPU_DEVICE names
//! another) in a compositor of the test's own, a player's core approved, the scene waited on, and the picture read back.

#![allow(dead_code)]

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use emusen_client_fixture::cores::{painted_pixel, PAINTED_SIZE};
use emusen_client_fixture::{library, World};
use emusen_crystal_client::{Api, Config, Runtime};
use emusen_platform_client::serenity::Device;
use emusen_platform_client::Platform;
use emusen_qt_models::picture::PictureFeed;
use emusen_qt_play::host::{Adoption, Host};
use emusen_qt_play::PLAY_URL;
use emusen_qt_test::{Harness, Scene};
use std::sync::atomic::Ordering;

/// Waits up to ten seconds for a condition the scene answers, letting Qt run between looks.
pub fn until(scene: &mut Scene<'_>, what: &str, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    let start = Instant::now();
    while !done(scene) {
        assert!(start.elapsed() < Duration::from_secs(10), "{what}: not within ten seconds");
        scene.wait(8);
    }
}

/// Qt drawing with Vulkan on the threaded render loop, as the program does.
pub fn vulkan_qt() {
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
}

/// The player's core whose library is `core` approved by its hash, through a runtime over `cores` (the installation's
/// folder, or the runtime's default) and the player's folder, as the Cores window approves one.
pub fn approve(core: &Path, cores: Option<&Path>, world: &World) {
    let config = runtime_config(cores, world);
    let rt = Runtime::new(&Api::load(library()).unwrap(), &config).unwrap();
    let sha = emusen_client_fixture::cores::sha256(core);
    rt.core_approve(&sha).unwrap_or_else(|e| panic!("{e}; the runtime lists {:?}", rt.cores()));
}

/// A runtime's config over `cores` (the installation's folder, or the runtime's default) and the player's folder.
pub fn runtime_config(cores: Option<&Path>, world: &World) -> Config {
    let player = Some(vec![world.root.join("Home/Cores").to_str().unwrap().into()]);
    Config { cores: cores.map(|c| vec![c.to_str().unwrap().into()]), player_cores: cores.and(player), ..Config::default() }
}

/// Serenity's device for the window, validated, the feed drawing through the plain preset, and the window's adoption,
/// as the programs make them (`emusen_qt_play::host`): made before the window so that they outlive it, on a failure too.
pub fn device_and_feed(presets: &Path) -> (Arc<Device>, PictureFeed, Adoption) {
    let platform = Platform::load(library()).unwrap();
    let gpu = std::env::var("EMUSEN_GPU_DEVICE").ok();
    let host = Host::new(&platform, "wayland", gpu.as_deref(), true, presets).expect("a Vulkan device");
    let adoption = host.adoption();
    let Host { device, feed, .. } = host;
    (device, feed, adoption)
}

/// Play.qml at 1280 by 800, its window adopting the device.
pub fn play_window<'h>(harness: &'h mut Harness, adoption: &Adoption) -> Scene<'h> {
    harness.load_prepared(PLAY_URL, 1280, 800, |window| unsafe { adoption.adopt(window.cast()) }.expect("Qt adopts the device"))
}

/// The game at `game` running in the window and its first picture drawn.
pub fn play(scene: &mut Scene<'_>, feed: &PictureFeed, game: &Path) {
    assert!(scene.set_property("play", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("play", "feed", &feed.address().to_string()));
    assert!(scene.set_property("play", "game", game.to_str().unwrap()));
    until(scene, "the game running", |s| {
        let problem = s.property("session", "problem");
        assert!(problem.is_empty() || problem.starts_with("The game plays without sound"), "the session: {problem}");
        s.property("session", "running") == "true"
    });
    until(scene, "the first picture", |s| s.property("session", "firstPictureMs").parse::<f64>().is_ok_and(|ms| ms > 0.0));
}

/// A pixel the window shows otherwise than the core made it: (x, y, shown, made).
pub type Difference = (u32, u32, [u8; 4], [u8; 4]);

/// The window's picture read back where 48 of the painted picture's pixels fall, the picture stretched over its item at
/// its display shape, against the core's own picture of the frame drawn last; the pixels that differ by more than one
/// level, as (x, y, shown, made).
pub fn painted_differences(scene: &mut Scene<'_>, feed: &PictureFeed) -> (u64, Vec<Difference>) {
    let (w, h) = PAINTED_SIZE;
    let points: Vec<(u32, u32)> = (0..8u32).flat_map(|i| (0..6u32).map(move |j| (i * 41 + 3, j * 39 + 7))).collect();
    let tree = scene.tree();
    let (px, py, pw, ph) = tree.nodes.iter().find(|n| n.name == "picture").map(|n| (n.x, n.y, n.w, n.h)).expect("the picture's item");
    let window: Vec<(f64, f64)> =
        points.iter().map(|&(x, y)| (px + (x as f64 + 0.5) * pw / w as f64, py + (y as f64 + 0.5) * ph / h as f64)).collect();
    let colours = scene.colours_at(&window);
    let serial = feed.counts().shown_serial.load(Ordering::Relaxed);
    let mut differ = Vec::new();
    for (&(x, y), colour) in points.iter().zip(colours) {
        let shown = colour.map_or([0; 4], |c| [c.r, c.g, c.b, c.a]);
        let made = painted_pixel(serial, x, y);
        if shown.iter().zip(made).any(|(a, b)| a.abs_diff(b) > 1) {
            differ.push((x, y, shown, made));
        }
    }
    (serial, differ)
}

/// What the layer said over a window grab: only the two hazards Qt's `grabWindow` makes on its own swapchain under the
/// threaded loop, a render pass after the present and a barrier after the acquire, which a run without the grab never
/// shows (EmuSen_Qt.md §1.8.8).
pub fn assert_qt_grab_only(findings: &[String]) {
    for f in findings {
        let swapchain = f.contains("previously written by vkQueuePresentKHR") || f.contains("previously read by vkAcquireNextImageKHR");
        assert!(swapchain, "a finding other than Qt's grab on its swapchain: {f}");
    }
}

/// Runs `step`, which may block the GUI thread; past `seconds` the compositor at `kwin` is stopped and the test aborts
/// with `what`, since a hung thread cannot be unwound.
pub fn within<T>(seconds: u64, kwin: u32, what: &str, step: impl FnOnce() -> T) -> T {
    let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let watch = {
        let (done, what) = (done.clone(), what.to_string());
        std::thread::spawn(move || {
            let until = Instant::now() + Duration::from_secs(seconds);
            while !done.load(Ordering::Relaxed) {
                if Instant::now() > until {
                    eprintln!("{what}: not within {seconds} s");
                    let _ = std::process::Command::new("kill").arg(kwin.to_string()).status();
                    std::process::abort();
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        })
    };
    let out = step();
    done.store(true, Ordering::Relaxed);
    let _ = watch.join();
    out
}
