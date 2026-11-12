//! The media viewer on the GPU tier (EmuSen_Qt.md §4.13, §4.14): a PDF manual the test writes, two pages of flat colour,
//! one upright and one across, opened with up, paged, zoomed and drawn through Qt PDF; and the video blur over the whole
//! screen beneath the strip, the bars beside a narrow video with it, as ES-DE 3.4.1 blurs it (R-20261111-103).

mod common;

use std::path::{Path, PathBuf};

use cxx_qt_lib::QImage;
use emusen_kit_core::input::UiButton;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::{Harness, Scene};

const SCENE: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/View.qml");
const W: i32 = 1280;
const H: i32 = 800;

/// A PDF of whole pages, each (width, height) in points filled with one colour (r, g, b in 0–1), its cross-reference
/// table counted from the bytes written.
fn pdf(pages: &[((u32, u32), (f64, f64, f64))]) -> Vec<u8> {
    let n = pages.len();
    let kids: Vec<String> = (0..n).map(|i| format!("{} 0 R", 3 + 2 * i)).collect();
    let mut objects = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        format!("<< /Type /Pages /Kids [{}] /Count {n} >>", kids.join(" ")),
    ];
    for (i, ((w, h), (r, g, b))) in pages.iter().enumerate() {
        let content = format!("{r} {g} {b} rg 0 0 {w} {h} re f");
        objects.push(format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w} {h}] /Contents {} 0 R >>", 4 + 2 * i));
        objects.push(format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len()));
    }
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = vec![];
    for (i, o) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend(format!("{} 0 obj\n{o}\nendobj\n", i + 1).bytes());
    }
    let xref = out.len();
    out.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).bytes());
    for o in offsets {
        out.extend(format!("{o:010} 00000 n \n").bytes());
    }
    out.extend(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objects.len() + 1).bytes());
    out
}

// A text list of the system's games; nothing from a real theme.
fn theme(root: &Path) {
    std::fs::create_dir_all(root).unwrap();
    std::fs::write(root.join("capabilities.xml"), "<themeCapabilities></themeCapabilities>").unwrap();
    std::fs::write(
        root.join("theme.xml"),
        "<theme><view name=\"system\"><carousel name=\"systems\"><pos>0 0.3</pos><size>1 0.4</size><text>${system.fullName}</text></carousel></view>\
         <view name=\"gamelist\"><textlist name=\"games\"><pos>0.05 0.1</pos><size>0.4 0.8</size><fontSize>0.04</fontSize></textlist></view></theme>",
    )
    .unwrap();
}

// The words an element draws, one glyph per item, joined with the spaces left out.
fn words(scene: &mut Scene<'_>, name: &str) -> String {
    let tree = scene.tree();
    let Some(top) = tree.nodes.iter().find(|n| n.name == name).map(|n| n.id.clone()) else { return String::new() };
    let under = |start: &str| {
        let mut id = start.to_owned();
        loop {
            if id == top {
                return true;
            }
            match tree.nodes.iter().find(|n| n.id == id) {
                Some(n) if !n.parent.is_empty() => id = n.parent.clone(),
                _ => return false,
            }
        }
    };
    let found: Vec<String> = tree.nodes.iter().filter(|n| n.visible && n.text.as_deref().is_some_and(|t| !t.is_empty()) && under(&n.id)).filter_map(|n| n.text.clone()).collect();
    found.join("").replace(' ', "")
}

fn box_of(scene: &mut Scene<'_>, name: &str) -> (f64, f64, f64, f64) {
    scene.tree().nodes.iter().find(|n| n.name == name).map(|n| (n.x, n.y, n.w, n.h)).unwrap_or_else(|| panic!("{name} drawn"))
}

fn until(scene: &mut Scene<'_>, what: &str, ok: impl Fn(&mut Scene<'_>) -> bool) {
    let mut waited = 0;
    while !ok(scene) {
        assert!(waited < 5000, "{what} within 5 s");
        scene.wait(20);
        waited += 20;
    }
}

// The grab's pixel at (x, y), read back through Qt's image.
fn pixel(scene: &mut Scene<'_>, scratch: &Path, x: i32, y: i32) -> [u8; 3] {
    let path = scratch.join("grab.png");
    assert!(scene.grab(&path.to_string_lossy()));
    let image = QImage::from_data(&std::fs::read(&path).unwrap(), None).expect("the grab reads");
    let c = image.pixel_color(x, y);
    [c.red() as u8, c.green() as u8, c.blue() as u8]
}

fn start() -> Compositor {
    let kwin = Compositor::start(3840, 2160);
    kwin.watch(120);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    std::env::set_var("QSG_RHI_DEBUG_LAYER", "1");
    std::env::set_var("VK_LAYER_SYNCVAL_SHADER_ACCESSES_HEURISTIC", "1");
    std::env::set_var("QT_VK_PHYSICAL_DEVICE_INDEX", std::env::var("EMUSEN_GPU_INDEX").unwrap_or_else(|_| "0".into()));
    kwin
}

// One application a process (EmuSen_Kit.md §7.9): both cases run in one test on one scene, never two tests at once.
#[test]
fn the_media_viewer_pages_a_pdf_manual_and_blurs_the_bars_beside_a_narrow_video() {
    assert!(emusen_esde_qt::marks::mark_named("star").is_some(), "the module is linked into this test");
    let _kwin = start();
    let mut harness = Harness::new();
    let mut scene = harness.load(SCENE, W, H);
    a_game_s_pdf_manual_opens_with_up_and_pages_zooms_and_draws_through_qt_pdf(&mut scene);
    the_viewer_s_blur_takes_the_bars_beside_a_narrow_video_and_leaves_them_sharp_when_off(&mut scene);
}

fn a_game_s_pdf_manual_opens_with_up_and_pages_zooms_and_draws_through_qt_pdf(scene: &mut Scene<'_>) {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("esde-qt-manual-{}", std::process::id()));
    theme(&root.join("theme"));
    let manual = root.join("manual.pdf");
    std::fs::write(&manual, pdf(&[((600, 800), (1.0, 0.0, 0.0)), ((800, 600), (0.0, 0.0, 1.0))])).unwrap();
    let shot = root.join("shot.bmp");
    common::bmp(&shot, 64, 56, |_, _| [0, 160, 0]);
    let media = format!("{{\"nes\":{{\"manual\":{{\"nes-0000\":\"{}\"}},\"screenshot\":{{\"nes-0000\":\"{}\"}}}}}}", manual.display(), shot.display());
    assert!(scene.set_property("view", "media", &media));
    let settings = format!("{{\"theme\":\"{}\",\"interface\":{{\"DisplayClock\":false,\"StartupView\":\"gamelist\"}}}}", root.join("theme").display());
    assert!(scene.set_property("view", "settings", &settings));
    let shelf = "[{\"name\":\"nes\",\"full_name\":\"Nintendo\",\"theme\":\"nes\",\"kind\":\"regular\",\"games\":[{\"file\":\"nes-0000.rom\",\"name\":\"Game\",\"esde_system\":\"nes\",\"favourite\":false,\"last_played\":0}]}]";
    assert!(scene.set_property("view", "shelves", shelf));
    until(scene, "the theme loaded", |s| s.property("esdeModel", "themed") == "true");
    until(scene, "the gamelist", |s| s.property("esdeModel", "view") == "gamelist");

    // X opens the viewer on a game with a screenshot and a manual; up opens the manual at its first page.
    scene.pad(UiButton::Screensaver);
    scene.wait(200);
    assert_eq!(words(scene, "element-help"), "BROWSEPDFMANUALFIRSTLAST", "a game with a manual offers it");
    scene.pad(UiButton::Up);
    until(scene, "the pages counted", |s| words(s, "element-left") == "PAGE1OF2");
    assert_eq!(words(scene, "element-help"), "BROWSEGAMEMEDIAFIRSTLASTZOOM", "the manual's help");
    let strip = emusen_esde::scene::Viewer::strip(f64::from(H));
    until(scene, "the page placed", |s| box_of(s, "element-item").2 > 0.0);
    let (x, y, w, h) = box_of(scene, "element-item");
    let fit = (f64::from(H) - strip) / 800.0;
    assert!((w - 600.0 * fit).abs() < 0.5 && (h - 800.0 * fit).abs() < 0.5 && (y - strip).abs() < 0.5, "fitted below the strip: {x} {y} {w} {h}");
    let scratch = root.join("grabs");
    std::fs::create_dir_all(&scratch).unwrap();
    until(scene, "the page drawn", |s| pixel(s, &scratch, W / 2, H / 2)[0] > 200);
    assert!(pixel(scene, &scratch, W / 2, H / 2)[2] < 50, "the first page is red");

    // Right turns to the second page, across: the box takes its shape, and it is drawn blue.
    scene.pad(UiButton::Right);
    until(scene, "the second page", |s| words(s, "element-left") == "PAGE2OF2");
    until(scene, "the second page placed", |s| {
        let b = box_of(s, "element-item");
        b.2 > b.3
    });
    until(scene, "the second page drawn", |s| pixel(s, &scratch, W / 2, H / 2)[2] > 200);

    // The right shoulder zooms: the page half the screen's height taller, centred, and the help the pan's.
    let before = box_of(scene, "element-item");
    scene.pad(UiButton::PageDown);
    until(scene, "the zoom", |s| box_of(s, "element-item").3 > before.3 + 1.0);
    let after = box_of(scene, "element-item");
    assert!((after.3 - before.3 - 0.5 * f64::from(H)).abs() < 0.5, "a zoom step: {before:?} to {after:?}");
    assert_eq!(words(scene, "element-help"), "PANRESETZOOM");
    scene.pad(UiButton::Back);
    scene.wait(200);
    assert!(scene.tree().nodes.iter().all(|n| !(n.visible && n.name == "element-strip")), "any other button closes the viewer");
    scene.assert_quiet();
}

fn the_viewer_s_blur_takes_the_bars_beside_a_narrow_video_and_leaves_them_sharp_when_off(scene: &mut Scene<'_>) {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("esde-qt-blur-{}", std::process::id()));
    theme(&root.join("theme"));
    let clip = root.join("clip.y4m");
    emusen_qt_test::clips::write(&clip, 20);
    let media = format!("{{\"nes\":{{\"video\":{{\"nes-0000\":\"{}\"}}}}}}", clip.display());
    assert!(scene.set_property("view", "media", &media));
    let shelf = "[{\"name\":\"nes\",\"full_name\":\"Nintendo\",\"theme\":\"nes\",\"kind\":\"regular\",\"games\":[{\"file\":\"nes-0000.rom\",\"name\":\"Game\",\"esde_system\":\"nes\",\"favourite\":false,\"last_played\":0}]}]";
    let scratch = root.join("grabs");
    std::fs::create_dir_all(&scratch).unwrap();
    let mid = H / 2 + 20;
    // The 64 x 48 clip fitted below the strip is narrower than the screen: black bars stand at its left and right.
    let edge_and_bar = |scene: &mut Scene<'_>, blur: bool| {
        let settings = format!("{{\"theme\":\"{}\",\"interface\":{{\"DisplayClock\":false,\"StartupView\":\"gamelist\",\"MediaViewerVideoBlur\":{blur}}}}}", root.join("theme").display());
        assert!(scene.set_property("view", "settings", &settings));
        assert!(scene.set_property("view", "shelves", shelf));
        until(scene, "the theme loaded", |s| s.property("esdeModel", "themed") == "true");
        until(scene, "the gamelist", |s| s.property("esdeModel", "view") == "gamelist");
        scene.pad(UiButton::Screensaver);
        until(scene, "the clip shown and faded in", |s| pixel(s, &scratch, W / 2, mid)[0] > 200);
        scene.wait(700);
        let path = scratch.join("blur.png");
        assert!(scene.grab(&path.to_string_lossy()));
        let image = QImage::from_data(&std::fs::read(&path).unwrap(), None).expect("the grab reads");
        let red = |x: i32| image.pixel_color(x, mid).red();
        let edge = (0..W / 2).find(|x| red(*x) > 128).expect("the clip's left edge");
        scene.pad(UiButton::Back);
        scene.wait(200);
        (edge, red(edge - 3), red(edge - 40))
    };
    let (edge, near, far) = edge_and_bar(scene, false);
    assert!(edge > 100, "a narrow clip leaves a bar at its left: edge at {edge}");
    assert!(near < 4 && far < 4, "off, the bar is sharp black beside the clip: {near} and {far} at 3 and 40 px");
    let (edge_on, near, far) = edge_and_bar(scene, true);
    assert!((edge_on - edge).abs() <= 2, "the clip is placed alike: {edge} and {edge_on}");
    assert!(near > 12, "on, the blur reaches the bar 3 px beside the clip, as ES-DE's over the whole screen: red {near}");
    assert!(far < 4, "and the bar further out is black: {far}");
    scene.assert_quiet();
}
