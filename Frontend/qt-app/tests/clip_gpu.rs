//! The game's page's clip on Vulkan (EmuSen_Qt.md §7.8, §1.8.9): the program as a player runs it, its window adopting
//! Serenity's device in a compositor of the test's own on the RX 6800 (EMUSEN_GPU_DEVICE names another); the hero shows
//! its still, then the clip VideoOutput draws, and B stops it. The clip is one this test writes itself.

mod common;

use common::{draw_covers, sandbox, wait_for, write_clip};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_play::host::Host;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::Harness;

const COBALT: &str = "Cobalt Harbor (Synthetic)";

#[test]
fn qt_7_8_the_clip_plays_on_vulkan_and_stops_when_the_page_loses_the_focus() {
    let kwin = Compositor::start(1920, 1200);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    let mut harness = Harness::new();
    let platform = Platform::load(library()).unwrap();
    let gpu = std::env::var("EMUSEN_GPU_DEVICE").ok();
    let presets = std::env::temp_dir().join("qt-app-clip-gpu-presets");
    let host = Host::new(&platform, "wayland", gpu.as_deref(), false, &presets).expect("a Vulkan device");
    let adoption = host.adoption();
    let world = sandbox("qt-app-clip-gpu", &[("SNES", &format!("{COBALT}.sfc"))]);
    let media = world.dir("Media");
    let shot = media.join("snes/screenshots").join(format!("{COBALT}.png"));
    draw_covers(&mut harness, &world.root, &[(shot, (512, 448), 0, "snes screenshots".into())], |window| {
        unsafe { adoption.adopt(window.cast()) }.expect("Qt adopts the device")
    });
    write_clip(&media.join("snes/videos").join(format!("{COBALT}.mkv")));
    let mut settings = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    settings["EsdeMediaDirectory"] = serde_json::json!(media.to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();

    let mut scene = harness.load_prepared(MAIN_URL, 1280, 800, |window| unsafe { adoption.adopt(window.cast()) }.expect("Qt adopts the device"));
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "feed", &host.feed.address().to_string()));
    assert!(scene.set_property("program", "coverStores", media.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 20000);
    scene.wait(300);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Consoles", 2000);
    let tree = scene.tree();
    let all = tree.find_named("tile.0").expect("All Games");
    assert!(scene.focus_item(&tree, all));
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);
    scene.wait(300);
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "screens", "shown", "Page", 2000);
    wait_for(&mut scene, "pageModel", "ready", "true", 10000);

    let hero = |scene: &mut emusen_qt_test::Scene<'_>| scene.colours_at(&[(640.0, 30.0)])[0].expect("the hero drawn");
    let still = hero(&mut scene);
    assert!(still.g > 150 && still.r < 150, "the still first, the screenshot's green: {still:?}");
    wait_for(&mut scene, "pageClip", "showing", "true", 5000);
    scene.wait(1200);
    let clip = hero(&mut scene);
    assert!(clip.r > 150 && clip.b > 150 && clip.g < 110, "VideoOutput draws the clip's magenta over the still: {clip:?}");
    assert_eq!(scene.property("videoFrames", "frames"), "0", "on Vulkan the kit's sink draws nothing; VideoOutput does");

    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);
    wait_for(&mut scene, "videoPlayer", "playbackState", "StoppedState", 2000);
    scene.assert_quiet();
}
