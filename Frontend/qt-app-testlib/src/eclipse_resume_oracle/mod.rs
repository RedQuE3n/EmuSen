//! Shared software and GPU verification oracle for the Option 3 Docked Eclipse Resume Card (R-20261126-83).
#![allow(dead_code)]

use crate::common::wait_for;
use crate::eclipse_board::{boxes, font_of, painted, read, Boxed};
use core::pin::Pin;
use cxx_qt_lib::{QString, QVariant};
use emusen_client_fixture::{library, test_core, World};
use emusen_crystal_client::{Api, Config, Open, Runtime};
use emusen_kit_core::input::UiButton;
use emusen_kit_core::tokens::Colour;
use emusen_platform_client::galaxia::consts::{DIR_CORES, SAVE_PICTURE};
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_models::library_model::qobject::LibraryModel;
use emusen_qt_models::session_model::qobject::SessionModel;
use emusen_qt_test::tree::Tree;
use emusen_qt_test::{Harness, Scene};
use std::collections::BTreeMap;
use std::path::Path;

const ECLIPSE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../themes/eclipse");

/// Stage visual state expected on the Resume Card.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StageKind {
    Picture,
    Cover,
    Tier2,
}

/// Boots MAIN_URL directly into Eclipse with splash off for Resume Card inspection.
pub fn open_eclipse_program<'a>(harness: &'a mut Harness, width: i32, height: i32, big_screen: bool, only_pads: Option<&str>) -> Scene<'a> {
    open_eclipse_program_with_cores(harness, width, height, big_screen, only_pads, None)
}

/// Boots MAIN_URL directly into Eclipse with splash off and an optional installed core manifest directory.
pub fn open_eclipse_program_with_cores<'a>(
    harness: &'a mut Harness,
    width: i32,
    height: i32,
    big_screen: bool,
    only_pads: Option<&str>,
    cores: Option<&Path>,
) -> Scene<'a> {
    open_eclipse_program_prepared(harness, width, height, big_screen, only_pads, cores, |_| {})
}

/// Boots MAIN_URL directly into Eclipse with a window preparation hook and optional cores directory.
pub fn open_eclipse_program_prepared<'a>(
    harness: &'a mut Harness,
    width: i32,
    height: i32,
    big_screen: bool,
    only_pads: Option<&str>,
    cores: Option<&Path>,
    prepare: impl FnOnce(*mut cxx_qt::QObject),
) -> Scene<'a> {
    let start = emusen_qt::start::read(library());
    let mut initial = emusen_qt::start::initial(&start);
    initial.insert(QString::from("splash"), QVariant::from(&false));
    initial.insert(QString::from("bigScreen"), QVariant::from(&big_screen));
    initial.insert(QString::from("theme"), QVariant::from(&QString::from(&std::fs::canonicalize(ECLIPSE).unwrap().display().to_string())));
    if let Some(pad_id) = only_pads {
        initial.insert(QString::from("onlyPads"), QVariant::from(&QString::from(pad_id)));
    }
    if cores.is_some() {
        initial.remove(&QString::from("platformLibrary"));
    }
    let mut scene = harness.load_initial(MAIN_URL, width, height, &initial, prepare);
    if let Some(cores_dir) = cores {
        assert!(scene.set_property("library", "cores", &cores_dir.to_string_lossy()));
        assert!(scene.set_property("program", "platformLibrary", &library().to_string_lossy()));
    }
    wait_for(&mut scene, "library", "loading", "false", 15000);
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);
    wait_for(&mut scene, "resumeQuestion", "eclipse", "true", 2000);
    assert!(scene.set_property("theme", "battery", "82"));
    scene.wait(150);
    scene
}

fn save_slot0(platform: &Platform, world: &World, harness: &mut Harness, rt: Runtime, rom_path: &Path, slot_png: Option<(u32, u32, usize)>) -> (Runtime, u32, String) {
    let sid = rt.open(&Open { path: Some(rom_path.to_string_lossy().into_owned()), ..Open::default() }).unwrap();
    let info = rt.session_info(sid).unwrap();
    let systems = rt.systems().unwrap_or_default();
    let esde_sys = systems
        .iter()
        .find(|s| s.id.eq_ignore_ascii_case(&info.console))
        .map(|s| s.esde_system.clone())
        .filter(|s| !s.is_empty())
        .unwrap_or(info.console);
    rt.resume_save(sid).unwrap();
    let slots = rt.state_slots(sid).unwrap();
    let slot0 = slots.iter().find(|s| s.slot == 0 && s.exists).expect("slot 0 exists");
    let pic = platform.save_path(SAVE_PICTURE, &slot0.path, None, 0).unwrap();
    if let Some((w, h, art_idx)) = slot_png {
        write_synthetic_png(harness, &world.root, &pic, w, h, art_idx);
    } else {
        let _ = std::fs::remove_file(&pic);
    }
    (rt, sid, esde_sys)
}

/// Installs/approves testcore if needed, opens `rom_path`, saves slot 0, and writes or removes its slot 0 PNG.
pub fn prepare_testcore_slot0(world: &World, harness: &mut Harness, rom_path: &Path, slot_png: Option<(u32, u32, usize)>) -> (Runtime, u32, String) {
    let platform = Platform::load(library()).unwrap();
    let player = platform.directory(DIR_CORES).unwrap().expect("player cores dir");
    std::fs::create_dir_all(&player).unwrap();
    let core_dest = player.join(test_core().file_name().unwrap());
    if !core_dest.exists() {
        world.player_core(&player);
    }
    let api = Api::load(library()).unwrap();
    let config = Config { development: false, cores: Some(Vec::new()), player_cores: Some(vec![player.to_str().unwrap().into()]) };
    let rt = Runtime::new(&api, &config).unwrap();
    let sha = rt.cores().unwrap().into_iter().find(|c| c.directory.as_deref() == Some("player")).and_then(|c| c.sha256).unwrap();
    rt.core_approve(&sha).unwrap();
    save_slot0(&platform, world, harness, rt, rom_path, slot_png)
}

/// Opens `rom_path` with an installed cores directory (such as `installation_with_snes`), saves slot 0, and writes or clears slot 0 PNG.
pub fn prepare_core_slot0(world: &World, harness: &mut Harness, cores: &Path, rom_path: &Path, slot_png: Option<(u32, u32, usize)>) -> (Runtime, u32, String) {
    let platform = Platform::load(library()).unwrap();
    let config = emusen_qt_models::clients::runtime_config(&platform, &cores.to_string_lossy()).unwrap();
    let api = Api::load(library()).unwrap();
    let rt = Runtime::new(&api, &config).unwrap();
    save_slot0(&platform, world, harness, rt, rom_path, slot_png)
}

/// Triggers `LibraryModel::refresh()` and waits until catalog scanning finishes.
pub fn refresh_library(scene: &mut Scene<'_>) {
    let ptr = scene.object("library");
    assert!(!ptr.is_null(), "library object must exist in Main.qml");
    let library_model = unsafe { Pin::new_unchecked(&mut *(ptr as *mut LibraryModel)) };
    library_model.refresh();
    wait_for(scene, "library", "loading", "false", 5000);
    scene.wait(150);
}

/// Attaches a real opened session onto SessionModel and runs `left_off(sid)` to populate Resume Card facts.
pub fn attach_resume_session(scene: &mut Scene<'_>, runtime: Runtime, sid: u32, game: &Path) {
    let ptr = scene.object("session");
    assert!(!ptr.is_null(), "session object must exist in Main.qml");
    let session = unsafe { Pin::new_unchecked(&mut *(ptr as *mut SessionModel)) };
    let game_str = game.to_string_lossy();
    session.attach_resume_test_session(runtime, sid, &game_str);
    wait_for(scene, "session", "resumeAsked", "true", 2000);
    let mut waited = 0;
    while waited < 2000 && !scene.tree().nodes.iter().any(|n| n.name == "eclipseResumeDock" && n.visible) {
        scene.wait(25);
        waited += 25;
    }
    scene.wait(150);
}

/// Clicks the center of the deepest visible named element in the scene tree.
pub fn click_named(scene: &mut Scene<'_>, name: &str) {
    let tree = scene.tree();
    let n = tree.nodes.iter().rfind(|n| n.name == name && n.visible).unwrap_or_else(|| panic!("expected visible node {name} to click"));
    scene.click(n.x + n.w / 2.0, n.y + n.h / 2.0);
}

/// Hovers the center of the deepest visible named element in the scene tree.
pub fn hover_named(scene: &mut Scene<'_>, name: &str) {
    let tree = scene.tree();
    let n = tree.nodes.iter().rfind(|n| n.name == name && n.visible).unwrap_or_else(|| panic!("expected visible node {name} to hover"));
    scene.hover(n.x + n.w / 2.0, n.y + n.h / 2.0);
}

/// Returns the objectName of the active focus item in the current tree.
pub fn focused_name(scene: &mut Scene<'_>) -> String {
    let t = scene.tree();
    scene.focus(&t).map(|i| t.nodes[i as usize].name.clone()).unwrap_or_default()
}

/// Generates a real multi-band RGBA PNG fixture of arbitrary pixel size via Qt Quick.
pub fn write_synthetic_png(harness: &mut Harness, scratch: &Path, path: &Path, width: u32, height: u32, art_index: usize) {
    emusen_qt_test::covers::draw_art(harness, scratch, &[(path.to_path_buf(), (width, height), art_index)], |_| {});
}

/// Waits until a named QML item in the live tree reaches expected property value or visibility.
pub fn wait_tree_prop(scene: &mut Scene<'_>, name: &str, prop: &str, expected: &str, timeout_ms: i32) {
    let mut waited = 0;
    while waited < timeout_ms {
        let tree = scene.tree();
        if read(&tree, name, prop) == expected {
            return;
        }
        scene.wait(25);
        waited += 25;
    }
    let tree = scene.tree();
    let got = read(&tree, name, prop);
    panic!("{name}.{prop} stayed {got:?}, expected {expected:?} after {timeout_ms}ms");
}

fn node_box(map: &BTreeMap<String, Boxed>, name: &str) -> Boxed {
    *map.get(name).unwrap_or_else(|| panic!("expected visible element {name}"))
}

fn approx(got: f64, want: f64, label: &str) {
    assert!((got - want).abs() <= 1.5, "{label}: got {got:.2}, expected {want:.2}");
}

fn text_screen_px(tree: &Tree, name: &str, screen_per_board: f64) -> f64 {
    let raw = read(tree, name, "font");
    let (px, _, _, _, _) = font_of(&raw).unwrap_or_else(|| panic!("missing font on {name} (got {raw:?})"));
    px * screen_per_board
}

/// Asserts Option 3 Docked Stage geometry, typography floors, copy rules, pill containment, and visible stage node.
pub fn assert_resume_layout(scene: &mut Scene<'_>, big_screen: bool, stage_kind: StageKind) {
    let tree = scene.tree();
    let map = boxes(&tree, 1.0);
    let dock = node_box(&map, "eclipseResumeDock");
    let stage = node_box(&map, "eclipseResumeStageWell");
    let border = node_box(&map, "eclipseResumeStageBorder");
    let bar = node_box(&map, "eclipseResumeBottomBar");
    let title = node_box(&map, "eclipseResumeTitle");
    let saved = node_box(&map, "eclipseResumeSavedPill");
    let prompt_title = node_box(&map, "eclipseResumePromptTitle");
    let prompt_body = node_box(&map, "eclipseResumePromptBody");
    let actions_slot = node_box(&map, "eclipseResumeActionsSlot");
    let choices = node_box(&map, "eclipseResumeChoices");
    let caption_block = node_box(&map, "eclipseResumeCaptionBlock");
    let caption = node_box(&map, "eclipseResumeCaption");
    let pill0 = node_box(&map, "eclipseResumeAnswer.0");
    let pill1 = node_box(&map, "eclipseResumeAnswer.1");
    let foot = node_box(&map, "eclipseResumeStageFoot");
    let card = node_box(&map, "eclipseResumeCard");

    let spb: f64 = read(&tree, "eclipseResumeCard", "screenPerBoard").parse().expect("screenPerBoard");
    assert!(title.y + title.h <= saved.y + 0.5, "title overlaps saved pill: {title:?} vs {saved:?}");
    assert!(saved.y + saved.h <= prompt_title.y + 0.5, "saved pill overlaps prompt title: {saved:?} vs {prompt_title:?}");
    assert!(prompt_title.y + prompt_title.h <= prompt_body.y + 0.5, "prompt title overlaps body: {prompt_title:?} vs {prompt_body:?}");
    assert!(prompt_body.y + prompt_body.h <= actions_slot.y + 0.5, "prompt body overlaps actions slot: {prompt_body:?} vs {actions_slot:?}");
    assert!(prompt_body.y + prompt_body.h <= pill0.y + 0.5, "prompt body overlaps pill0: {prompt_body:?} vs {pill0:?}");
    assert!(pill0.y >= actions_slot.y - 0.5, "pill0 above actions slot: {pill0:?} vs {actions_slot:?}");
    assert!(pill0.y + pill0.h <= pill1.y + 0.5, "pill0 overlaps pill1: {pill0:?} vs {pill1:?}");
    assert!(pill1.y + pill1.h <= actions_slot.y + actions_slot.h + 0.5, "pill1 overflows actions slot bottom: {pill1:?} vs {actions_slot:?}");
    assert!(actions_slot.y + actions_slot.h <= caption_block.y + 0.5, "actions slot overlaps caption block: {actions_slot:?} vs {caption_block:?}");
    assert!(pill1.y + pill1.h <= caption_block.y + 0.5, "pill1 overlaps caption block: {pill1:?} vs {caption_block:?}");
    assert!(caption.y + caption.h <= dock.y + dock.h + 0.5, "caption overflows dock bottom: {caption:?} vs {dock:?}");
    assert!(choices.h <= actions_slot.h + 0.5, "choices list ({0:.2}) exceeded actionsSlot height ({1:.2})", choices.h, actions_slot.h);
    assert!(stage.y + stage.h < foot.y + 0.5, "stage well overlaps stage foot: {stage:?} vs {foot:?}");
    assert!(foot.y + foot.h <= bar.y + 0.5, "stage foot overlaps bottom bar: {foot:?} vs {bar:?}");

    approx(border.x, stage.x, "stage border x");
    approx(border.y, stage.y, "stage border y");
    approx(border.w, stage.w, "stage border w");
    approx(border.h, stage.h, "stage border h");

    // Readability pixel floors across handheld 800p and desktop form factors.
    let body_px = text_screen_px(&tree, "eclipseResumePromptBody", spb);
    let cap_px = text_screen_px(&tree, "eclipseResumeCaption", spb);
    let meta_px = text_screen_px(&tree, "eclipseResumeStageMetaText", spb);
    assert!(body_px >= 13.9, "prompt body floor violated: {body_px:.2}px");
    assert!(cap_px >= 12.9, "caption floor violated: {cap_px:.2}px");
    assert!(meta_px >= 11.9, "meta pill floor violated: {meta_px:.2}px");
    assert!(pill0.h >= 47.9, "Continue pill height {0:.2}px below 48px floor", pill0.h);
    assert!(pill1.h >= 47.9, "Start again pill height {0:.2}px below 48px floor", pill1.h);

    if !big_screen {
        assert!(card.w <= 1160.5, "desktop card width {} exceeded 1160px", card.w);
        assert!(card.h <= 600.5, "desktop card height {} exceeded 600px", card.h);
        assert!(pill0.h <= 52.0, "desktop Continue pill height {} doubled above 52px", pill0.h);
        assert!(pill1.h <= 52.0, "desktop Start again pill height {} doubled above 52px", pill1.h);
        assert!(saved.h >= 27.9 && saved.h <= 34.0, "desktop saved pill height {} out of 28..34px bounds", saved.h);
        assert!(dock.x >= card.x && dock.y >= card.y && dock.x + dock.w <= card.x + card.w);
        assert!(stage.x + stage.w <= card.x + card.w + 0.5);
    } else if (card.w - 1920.0).abs() < 2.0 && (card.h - 1200.0).abs() < 2.0 {
        approx(dock.x, 52.0, "1920 dock.x");
        approx(dock.y, 48.0, "1920 dock.y");
        approx(dock.w, 576.0, "1920 dock.w");
        approx(dock.h, 1036.0, "1920 dock.h");
        approx(stage.x, 680.0, "1920 stage.x");
        approx(stage.y, 58.0, "1920 stage.y");
        approx(stage.w, 1184.0, "1920 stage.w");
        approx(stage.h, 888.0, "1920 stage.h");
        approx(bar.h, 76.0, "1920 bottomBar.h");
    }

    // Verify eyebrow copy strictly uses CORE and never ENGINE.
    let eyebrow_core = read(&tree, "eclipseResumeEyebrowCore", "text");
    assert!(eyebrow_core.ends_with(" CORE") && !eyebrow_core.contains("ENGINE") && !eyebrow_core.contains("Engine"), "bad core eyebrow: {eyebrow_core:?}");
    assert_eq!(read(&tree, "eclipseResumeHeaderTitle", "text"), "Launch");
    assert_eq!(read(&tree, "eclipseResumePromptTitle", "text"), "Continue where you left off?");
    assert_eq!(read(&tree, "eclipseResumeAnswerLabel.0", "text"), "Continue");
    assert_eq!(read(&tree, "eclipseResumeAnswerLabel.1", "text"), "Start again");
    assert_eq!(read(&tree, "eclipseResumeAnswerSub.1", "text"), "From the beginning");

    match stage_kind {
        StageKind::Picture => {
            assert_eq!(painted(&tree, "eclipseResumeStageWell"), Some(Colour::rgba(0, 0, 0, 255)), "stage well must be pure black");
            assert_eq!(read(&tree, "eclipseResumeStageImage", "visible"), "true");
            assert_eq!(read(&tree, "eclipseResumeStageImage", "status"), "Ready");
            assert_ne!(read(&tree, "eclipseResumeCoverPlate", "visible"), "true");
            assert_ne!(read(&tree, "eclipseResumeTier2Plate", "visible"), "true");
        }
        StageKind::Cover => {
            assert_eq!(read(&tree, "eclipseResumeCoverPlate", "visible"), "true");
            assert_eq!(read(&tree, "eclipseResumeCoverImage", "visible"), "true");
            assert_eq!(read(&tree, "eclipseResumeCoverImage", "status"), "Ready");
            assert_eq!(read(&tree, "eclipseResumeFallbackPill", "visible"), "true");
            assert_ne!(read(&tree, "eclipseResumeStageImage", "visible"), "true");
            assert_ne!(read(&tree, "eclipseResumeTier2Plate", "visible"), "true");
        }
        StageKind::Tier2 => {
            assert_eq!(read(&tree, "eclipseResumeTier2Plate", "visible"), "true");
            assert_eq!(read(&tree, "eclipseResumeTier2Crescent", "visible"), "true");
            assert_eq!(read(&tree, "eclipseResumeTier2Pill", "visible"), "true");
            assert_ne!(read(&tree, "eclipseResumeStageImage", "visible"), "true");
            assert_ne!(read(&tree, "eclipseResumeCoverPlate", "visible"), "true");
        }
    }
}

/// Exercises D-pad, directional, Tab/Backtab, single-step bounds, and pointer hover focus between choices 0 and 1.
pub fn assert_resume_navigation(scene: &mut Scene<'_>, with_pad_and_mouse: bool) {
    assert_eq!(focused_name(scene), "eclipseResumeAnswer.0", "Continue must be focused on open");
    assert_eq!(read(&scene.tree(), "eclipseResumeCounter", "text"), "1 of 2");

    // Non-wrapping directional upper bound: Up from index 0 keeps index 0.
    scene.pad(UiButton::Up);
    scene.wait(60);
    assert_eq!(focused_name(scene), "eclipseResumeAnswer.0");
    assert_eq!(read(&scene.tree(), "eclipseResumeCounter", "text"), "1 of 2");

    scene.pad(UiButton::Down);
    scene.wait(60);
    assert_eq!(focused_name(scene), "eclipseResumeAnswer.1");
    assert_eq!(read(&scene.tree(), "eclipseResumeCounter", "text"), "2 of 2");

    // Non-wrapping directional lower bound: Down from index 1 keeps index 1.
    scene.pad(UiButton::Down);
    scene.wait(60);
    assert_eq!(focused_name(scene), "eclipseResumeAnswer.1");
    assert_eq!(read(&scene.tree(), "eclipseResumeCounter", "text"), "2 of 2");

    scene.pad(UiButton::Up);
    scene.wait(60);
    assert_eq!(focused_name(scene), "eclipseResumeAnswer.0");
    assert_eq!(read(&scene.tree(), "eclipseResumeCounter", "text"), "1 of 2");

    scene.pad(UiButton::Right);
    scene.wait(60);
    assert_eq!(focused_name(scene), "eclipseResumeAnswer.1");
    assert_eq!(read(&scene.tree(), "eclipseResumeCounter", "text"), "2 of 2");

    scene.pad(UiButton::Left);
    scene.wait(60);
    assert_eq!(focused_name(scene), "eclipseResumeAnswer.0");
    assert_eq!(read(&scene.tree(), "eclipseResumeCounter", "text"), "1 of 2");

    if with_pad_and_mouse {
        // Tab and Backtab wrap cleanly one step at a time without double dispatch.
        scene.key(0x01000001, 0);
        scene.wait(60);
        assert_eq!(focused_name(scene), "eclipseResumeAnswer.1");
        assert_eq!(read(&scene.tree(), "eclipseResumeCounter", "text"), "2 of 2");

        scene.key(0x01000001, 0);
        scene.wait(60);
        assert_eq!(focused_name(scene), "eclipseResumeAnswer.0");
        assert_eq!(read(&scene.tree(), "eclipseResumeCounter", "text"), "1 of 2");

        scene.key(0x01000002, 0x02000000);
        scene.wait(60);
        assert_eq!(focused_name(scene), "eclipseResumeAnswer.1");
        assert_eq!(read(&scene.tree(), "eclipseResumeCounter", "text"), "2 of 2");

        scene.key(0x01000002, 0x02000000);
        scene.wait(60);
        assert_eq!(focused_name(scene), "eclipseResumeAnswer.0");
        assert_eq!(read(&scene.tree(), "eclipseResumeCounter", "text"), "1 of 2");

        // Pointer hover focuses each answer pill directly.
        hover_named(scene, "eclipseResumeAnswer.1");
        scene.wait(60);
        assert_eq!(focused_name(scene), "eclipseResumeAnswer.1");
        assert_eq!(read(&scene.tree(), "eclipseResumeCounter", "text"), "2 of 2");

        hover_named(scene, "eclipseResumeAnswer.0");
        scene.wait(60);
        assert_eq!(focused_name(scene), "eclipseResumeAnswer.0");
        assert_eq!(read(&scene.tree(), "eclipseResumeCounter", "text"), "1 of 2");
    }
}
