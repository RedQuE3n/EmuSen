//! Vulkan GPU verification and capture for Option 3 Docked Eclipse Resume Card (R-20261126-83).

use crate::common::wait_for;
use crate::eclipse_board::read;
use crate::eclipse_resume_oracle::{
    assert_resume_layout, focused_name, open_eclipse_program_prepared, prepare_core_slot0, refresh_library, wait_tree_prop,
    write_synthetic_png, StageKind,
};
use emusen_client_fixture::cores::{installation_with_snes, snes_image};
use emusen_client_fixture::{library, World};
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt_play::host::Host;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::Harness;
use std::path::Path;

fn capture_if_requested(scene: &mut emusen_qt_test::Scene<'_>, file: &str) {
    let Some(dir) = std::env::var_os("EMUSEN_RESUME_CAPTURE_DIR").or_else(|| std::env::var_os("EMUSEN_ECLIPSE_PICTURES")) else {
        return;
    };
    std::fs::create_dir_all(&dir).unwrap();
    scene.wait(350);
    let path = Path::new(&dir).join(file);
    assert!(scene.grab(&path.display().to_string()), "grabbed {}", path.display());
}

fn assert_playing_advances(scene: &mut emusen_qt_test::Scene<'_>) {
    wait_for(scene, "session", "resumeAsked", "false", 2000);
    wait_for(scene, "session", "paused", "false", 2000);
    wait_for(scene, "screens", "shown", "Game", 2000);
    let mut waited = 0;
    while waited < 3000 && scene.property("session", "firstPictureMs").parse::<f64>().unwrap_or(0.0) <= 0.0 {
        scene.wait(50);
        waited += 50;
    }
    let first_ms: f64 = scene.property("session", "firstPictureMs").parse().unwrap_or(0.0);
    assert!(first_ms > 0.0, "expected firstPictureMs > 0 after resume answer, got {first_ms}");
    let before = scene.frames();
    scene.wait(500);
    assert!(scene.frames() - before > 10, "expected advancing GPU frames after play started, got {}", scene.frames() - before);
    assert!(scene.set_property("game", "game", ""));
    wait_for(scene, "session", "running", "false", 2000);
    wait_for(scene, "screens", "shown", "Home", 2000);
}

#[test]
fn eclipse_resume_card_renders_and_captures_on_vulkan() {
    let kwin = Compositor::start(1280, 800);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");

    let world = World::new("qt-app-eclipse-resume-gpu");
    let roms = world.dir("Roms/snes");
    std::fs::create_dir_all(&roms).unwrap();
    let rom_main = roms.join("Chrono Trigger (USA).sfc");
    let rom_long = roms.join("The Chronological Adventures of Aurora Drift Across the Northern Constellation DX (USA).sfc");
    std::fs::write(&rom_main, snes_image("CHRONO TRIGGER")).unwrap();
    std::fs::write(&rom_long, snes_image("AURORA DRIFT DX")).unwrap();

    let platform = Platform::load(library()).unwrap();
    let gpu = std::env::var("EMUSEN_GPU_DEVICE").ok();
    let host = Host::new(&platform, "wayland", gpu.as_deref(), false, &world.dir("presets")).expect("a Vulkan device");
    let adoption = host.adoption();

    let cores = installation_with_snes(&world.dir("Cores"));
    let media = world.dir("EsdeMedia");
    let mut harness = Harness::new();
    let (_, _, esde_sys) = prepare_core_slot0(&world, &mut harness, &cores, &rom_main, Some((256, 224, 0)));
    let _ = prepare_core_slot0(&world, &mut harness, &cores, &rom_long, None);
    let cover_long = media
        .join(&esde_sys)
        .join("covers/The Chronological Adventures of Aurora Drift Across the Northern Constellation DX (USA).png");
    write_synthetic_png(&mut harness, &world.root, &cover_long, 300, 420, 4);

    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = serde_json::json!(world.root.join("Roms").to_str().unwrap());
    settings["EsdeMediaDirectory"] = serde_json::json!(media.to_str().unwrap());
    settings["OpenEmuFallback"] = serde_json::json!(false);
    settings["ResumeOnLaunch"] = serde_json::json!("Ask");
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();

    let mut scene = open_eclipse_program_prepared(&mut harness, 1920, 1200, true, None, Some(&cores), |window| {
        unsafe { adoption.adopt(window.cast()) }.expect("Qt adopts the device");
    });
    assert!(scene.set_property("program", "feed", &host.feed.address().to_string()));
    wait_for(&mut scene, "screens", "shown", "Home", 3000);
    scene.wait(150);

    let main_str = rom_main.to_string_lossy().into_owned();
    let long_str = rom_long.to_string_lossy().into_owned();

    // Primary board capture at 1920x1200, 800p handheld capture, and Desktop form-factor capture.
    assert!(scene.set_property("game", "game", &main_str));
    wait_for(&mut scene, "session", "resumeAsked", "true", 5000);
    assert_eq!(scene.property("session", "firstPictureMs"), "0", "game held unplayed behind card");
    wait_tree_prop(&mut scene, "eclipseResumeStageImage", "status", "Ready", 3000);
    assert_resume_layout(&mut scene, true, StageKind::Picture);
    assert_eq!(read(&scene.tree(), "eclipseResumeStageMetaText", "text"), "8:7 · 256×224");
    capture_if_requested(&mut scene, "resume-1920x1200.png");

    scene.resize(1280, 800);
    scene.wait(300);
    assert_resume_layout(&mut scene, true, StageKind::Picture);
    capture_if_requested(&mut scene, "resume-1280x800.png");

    assert!(scene.set_property("program", "bigScreen", "false"));
    scene.wait(300);
    assert_resume_layout(&mut scene, false, StageKind::Picture);
    capture_if_requested(&mut scene, "resume-desktop-1280x800.png");

    // Back dismisses card, returns to Home, and restores focus cleanly.
    assert!(scene.set_property("program", "bigScreen", "true"));
    scene.resize(1920, 1200);
    scene.wait(200);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "session", "resumeAsked", "false", 2000);
    wait_for(&mut scene, "session", "running", "false", 2000);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    wait_for(&mut scene, "resumeCanvas", "sheetOpen", "false", 2000);
    scene.wait(100);
    assert!(!focused_name(&mut scene).starts_with("eclipseResume") && !focused_name(&mut scene).is_empty(), "Home focus restored after Back");

    // Edge cases capture: real LibraryModel::coverOf fallback across refresh on long 3-line title + Continue playback.
    refresh_library(&mut scene);
    assert!(scene.set_property("game", "game", &long_str));
    wait_for(&mut scene, "session", "resumeAsked", "true", 5000);
    wait_tree_prop(&mut scene, "eclipseResumeCoverImage", "status", "Ready", 3000);
    assert_resume_layout(&mut scene, true, StageKind::Cover);
    assert_eq!(read(&scene.tree(), "eclipseResumeStageMetaText", "text"), "STATE READY");
    capture_if_requested(&mut scene, "resume-edge-cases-1920x1200.png");
    scene.pad(UiButton::Accept);
    assert_playing_advances(&mut scene);

    // Start again plays cleanly with advancing GPU frames.
    assert!(scene.set_property("game", "game", &main_str));
    wait_for(&mut scene, "session", "resumeAsked", "true", 5000);
    scene.pad(UiButton::Down);
    scene.wait(80);
    assert_eq!(read(&scene.tree(), "eclipseResumeCounter", "text"), "2 of 2");
    scene.pad(UiButton::Accept);
    assert_playing_advances(&mut scene);

    scene.assert_quiet();
    assert!(scene.set_property("program", "visible", "false"));
    scene.wait(50);
    drop(scene);
    drop(harness);
    drop(host);
}
