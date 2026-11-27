//! Software verification of Option 3 Docked Eclipse Resume Card across real sessions, fallbacks, input, and desktop scaling.

use crate::common::wait_for;
use crate::eclipse_board::read;
use crate::eclipse_resume_oracle::{
    assert_resume_layout, assert_resume_navigation, attach_resume_session, click_named, focused_name,
    open_eclipse_program_with_cores, prepare_core_slot0, prepare_testcore_slot0, refresh_library, wait_tree_prop,
    write_synthetic_png, StageKind,
};
use emusen_client_fixture::cores::{installation_with_snes, snes_image};
use emusen_client_fixture::{image, library, World};
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt_test::Harness;

#[test]
fn eclipse_resume_card_holds_board_geometry_fallbacks_navigation_and_desktop() {
    let world = World::new("qt-app-eclipse-resume");
    let snes_roms = world.dir("Roms/snes");
    let test_roms = world.dir("Roms/test");
    std::fs::create_dir_all(&snes_roms).unwrap();
    std::fs::create_dir_all(&test_roms).unwrap();
    let rom_256 = snes_roms.join("Chrono Trigger (USA).sfc");
    let rom_long = snes_roms.join("The Chronological Adventures of Aurora Drift Across the Northern Constellation DX (USA).sfc");
    let rom_480 = snes_roms.join("Pixel Check 480 (USA).sfc");
    let rom_space = snes_roms.join("Space Trigger (USA).sfc");
    let rom_return = snes_roms.join("Return Trigger (USA).sfc");
    let rom_pad = snes_roms.join("Pad Trigger (USA).sfc");
    let rom_unscraped = test_roms.join("Unscraped Prototype (USA).tst");
    for (p, title) in [
        (&rom_256, "CHRONO TRIGGER"),
        (&rom_long, "AURORA DRIFT DX"),
        (&rom_480, "PIXEL CHECK 480"),
        (&rom_space, "SPACE TRIGGER"),
        (&rom_return, "RETURN TRIGGER"),
        (&rom_pad, "PAD TRIGGER"),
    ] {
        std::fs::write(p, snes_image(title)).unwrap();
    }
    std::fs::write(&rom_unscraped, image(&[9, 8, 7, 6])).unwrap();

    let cores = installation_with_snes(&world.dir("Cores"));
    let media = world.dir("EsdeMedia");
    let mut harness = Harness::new();
    let (rt_256, sid_256, esde_sys) = prepare_core_slot0(&world, &mut harness, &cores, &rom_256, Some((256, 224, 0)));
    let (rt_long, sid_long, _) = prepare_core_slot0(&world, &mut harness, &cores, &rom_long, None);
    let (rt_480, sid_480, _) = prepare_core_slot0(&world, &mut harness, &cores, &rom_480, Some((640, 480, 2)));
    let (rt_space, sid_space, _) = prepare_core_slot0(&world, &mut harness, &cores, &rom_space, None);
    let (rt_return, sid_return, _) = prepare_core_slot0(&world, &mut harness, &cores, &rom_return, None);
    let (rt_pad, sid_pad, _) = prepare_core_slot0(&world, &mut harness, &cores, &rom_pad, None);
    let (rt_unscraped, sid_unscraped, _) = prepare_testcore_slot0(&world, &mut harness, &rom_unscraped, None);

    let cover_256 = media.join(&esde_sys).join("covers/Chrono Trigger (USA).png");
    let cover_long = media
        .join(&esde_sys)
        .join("covers/The Chronological Adventures of Aurora Drift Across the Northern Constellation DX (USA).png");
    write_synthetic_png(&mut harness, &world.root, &cover_256, 300, 420, 4);
    std::fs::create_dir_all(cover_long.parent().unwrap()).unwrap();
    std::fs::copy(&cover_256, &cover_long).unwrap();

    let platform = Platform::load(library()).unwrap();
    let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    settings["RomDirectory"] = serde_json::json!(world.root.join("Roms").to_str().unwrap());
    settings["EsdeMediaDirectory"] = serde_json::json!(media.to_str().unwrap());
    settings["OpenEmuFallback"] = serde_json::json!(false);
    settings["ResumeOnLaunch"] = serde_json::json!("Ask");
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();

    let mut scene = open_eclipse_program_with_cores(&mut harness, 1920, 1200, true, None, Some(&cores));
    wait_for(&mut scene, "screens", "shown", "Home", 3000);
    assert!(!focused_name(&mut scene).is_empty(), "Home should start focused");

    // Tier 0 at 1920x1200 with crisp 256x224 pixel art and native 8:7 metadata.
    attach_resume_session(&mut scene, rt_256, sid_256, &rom_256);
    wait_tree_prop(&mut scene, "eclipseResumeStageImage", "status", "Ready", 3000);
    assert_resume_layout(&mut scene, true, StageKind::Picture);
    let tree = scene.tree();
    assert_eq!(read(&tree, "eclipseResumeTitle", "text"), "Chrono Trigger (USA)");
    assert_eq!(read(&tree, "eclipseResumeStageMetaText", "text"), "8:7 · 256×224");
    assert_eq!(read(&tree, "eclipseResumeStageImage", "smooth"), "false");
    assert!(read(&tree, "eclipseResumeSavedLine", "text").starts_with("Saved "));
    assert!(read(&tree, "eclipseResumeStageCaption", "text").ends_with(" · Auto-resume slot"));
    assert_eq!(read(&tree, "eclipseResumePlanet", "visible"), "true");
    assert_resume_navigation(&mut scene, true);

    // Handheld 800p layout check with readability pixel floors.
    scene.resize(1280, 800);
    scene.wait(250);
    assert_resume_layout(&mut scene, true, StageKind::Picture);

    // Desktop form factor bounds and strict actionsSlot containment at 1280x800 and 960x640.
    assert!(scene.set_property("program", "bigScreen", "false"));
    scene.wait(250);
    assert_resume_layout(&mut scene, false, StageKind::Picture);
    scene.resize(960, 640);
    scene.wait(250);
    assert_resume_layout(&mut scene, false, StageKind::Picture);

    // Back dismisses card, clears resume facts, returns to Home, and restores focus.
    assert!(scene.set_property("program", "bigScreen", "true"));
    scene.resize(1920, 1200);
    scene.wait(200);
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "session", "resumeAsked", "false", 2000);
    wait_for(&mut scene, "session", "running", "false", 2000);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    wait_for(&mut scene, "resumeCanvas", "sheetOpen", "false", 2000);
    scene.wait(100);
    assert!(!focused_name(&mut scene).starts_with("eclipseResume"), "focus must leave Resume Card after Back");
    assert_eq!(scene.property("session", "resumePicture"), "");
    assert_eq!(scene.property("session", "resumeWhen"), "");
    assert_eq!(scene.property("session", "resumeConsole"), "");
    assert_eq!(scene.property("session", "resumeCore"), "");
    assert_eq!(scene.property("session", "resumePlanet"), "-2");

    // Library refresh preserves real LibraryModel::coverOf fallback on long 3-line title, closed with Escape.
    refresh_library(&mut scene);
    attach_resume_session(&mut scene, rt_long, sid_long, &rom_long);
    wait_tree_prop(&mut scene, "eclipseResumeCoverImage", "status", "Ready", 3000);
    assert_resume_layout(&mut scene, true, StageKind::Cover);
    let long_tree = scene.tree();
    assert_eq!(read(&long_tree, "eclipseResumeTitle", "maximumLineCount"), "3");
    assert_eq!(read(&long_tree, "eclipseResumeTitle", "truncated"), "true");
    assert_eq!(read(&long_tree, "eclipseResumeStageMetaText", "text"), "STATE READY");
    assert!(scene.property("session", "resumePlanet").parse::<i32>().unwrap_or(-2) >= 0, "native core shows planet");
    scene.key(0x01000000, 0);
    wait_for(&mut scene, "session", "resumeAsked", "false", 2000);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    wait_for(&mut scene, "resumeCanvas", "sheetOpen", "false", 2000);

    // Tier 2 procedural plate and crescent fallback on unscraped ROM, closed via HelpBar Back click.
    attach_resume_session(&mut scene, rt_unscraped, sid_unscraped, &rom_unscraped);
    assert_resume_layout(&mut scene, true, StageKind::Tier2);
    assert_eq!(read(&scene.tree(), "eclipseResumeStageMetaText", "text"), "STATE READY");
    assert_eq!(read(&scene.tree(), "eclipseResumePlanet", "visible"), "true");
    click_named(&mut scene, "help-Back");
    wait_for(&mut scene, "session", "resumeAsked", "false", 2000);
    wait_for(&mut scene, "session", "running", "false", 2000);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    wait_for(&mut scene, "resumeCanvas", "sheetOpen", "false", 2000);

    // 480p smooth 4:3 stage + left-click on Start again activates choice cleanly.
    attach_resume_session(&mut scene, rt_480, sid_480, &rom_480);
    wait_tree_prop(&mut scene, "eclipseResumeStageImage", "status", "Ready", 3000);
    assert_resume_layout(&mut scene, true, StageKind::Picture);
    let tree_480 = scene.tree();
    assert_eq!(read(&tree_480, "eclipseResumeStageMetaText", "text"), "4:3 · 640×480");
    assert_eq!(read(&tree_480, "eclipseResumeStageImage", "smooth"), "true");
    click_named(&mut scene, "eclipseResumeAnswer.1");
    wait_for(&mut scene, "session", "resumeAsked", "false", 2000);
    wait_for(&mut scene, "screens", "shown", "Game", 2000);
    assert!(scene.set_property("game", "game", ""));
    wait_for(&mut scene, "session", "running", "false", 2000);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);

    // Keyboard Space and Return each pick once cleanly.
    attach_resume_session(&mut scene, rt_space, sid_space, &rom_space);
    scene.key(0x20, 0);
    wait_for(&mut scene, "session", "resumeAsked", "false", 2000);
    wait_for(&mut scene, "screens", "shown", "Game", 2000);
    assert!(scene.set_property("game", "game", ""));
    wait_for(&mut scene, "session", "running", "false", 2000);

    attach_resume_session(&mut scene, rt_return, sid_return, &rom_return);
    scene.key(0x01000004, 0);
    wait_for(&mut scene, "session", "resumeAsked", "false", 2000);
    wait_for(&mut scene, "screens", "shown", "Game", 2000);
    assert!(scene.set_property("game", "game", ""));
    wait_for(&mut scene, "session", "running", "false", 2000);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);

    // Mid-enterFade pad reopen dispatches cleanly before fade settles.
    {
        let ptr = scene.object("session");
        let session = unsafe { core::pin::Pin::new_unchecked(&mut *(ptr as *mut emusen_qt_models::session_model::qobject::SessionModel)) };
        session.attach_resume_test_session(rt_pad, sid_pad, &rom_pad.to_string_lossy());
    }
    wait_for(&mut scene, "session", "resumeAsked", "true", 2000);
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "session", "resumeAsked", "false", 2000);
    wait_for(&mut scene, "screens", "shown", "Game", 2000);
    assert!(scene.set_property("game", "game", ""));
    wait_for(&mut scene, "session", "running", "false", 2000);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);

    scene.assert_quiet();
}
