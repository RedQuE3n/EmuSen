//! Settings ▸ Display ▸ Shaders and Shader levers over a running console, end to end in the program (EmuSen_Qt.md §7.7,
//! §1.8.10), on Vulkan in a compositor of the test's own: the client fixture's console core, a PlayStation started
//! empty, suspended to Home. The pad opens Settings ▸ Display ▸ Shaders, chooses a preset of the shader pack for the
//! PlayStation (one the test writes, a tint toward magenta by a lever of its own), and the resumed picture is read back
//! tinted; Shader levers moves the tint to nothing and the picture is read back as it was; both are stored, and read
//! again when Settings opens again. A GPU job.

mod common;

use common::console::{back_to_library, console_owns_the_pad, list_card_start_console, picture_points, until};
use common::{approved_console_core, sandbox, wait_for};

use emusen_client_fixture::cores::installation_with_snes;
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::{DIR_SHADERS, FILE_GRAPHICS, MODEL_GRAPHICS_CONFIG};
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_play::host::Host;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::virtual_pad::Sdl;
use emusen_qt_test::{Harness, Scene};

/// A one-pass preset of the test's own: the picture mixed toward magenta by half its `tint`, 1 at first.
const PRESET: &str = "shaders = 1\nshader0 = tint.slang\nfilter_linear0 = false\nscale_type0 = viewport\n";
const SHADER: &str = r#"#version 450
#pragma parameter tint "Tint toward magenta" 1.0 0.0 1.0 0.05
layout(std140, set = 0, binding = 0) uniform UBO
{
    mat4 MVP;
    float tint;
};

#pragma stage vertex
layout(location = 0) in vec4 Position;
layout(location = 1) in vec2 TexCoord;
layout(location = 0) out vec2 vTexCoord;
void main()
{
    gl_Position = MVP * Position;
    vTexCoord = TexCoord;
}

#pragma stage fragment
layout(location = 0) in vec2 vTexCoord;
layout(location = 0) out vec4 FragColor;
layout(set = 0, binding = 1) uniform sampler2D Source;
void main()
{
    vec3 c = texture(Source, vTexCoord).rgb;
    FragColor = vec4(mix(c, vec3(1.0, 0.0, 1.0), 0.5 * tint), 1.0);
}
"#;

/// The window's mean red, green and blue over the console's picture.
fn mean(scene: &mut Scene<'_>) -> [f64; 3] {
    let points = picture_points(scene);
    let seen: Vec<_> = scene.colours_at(&points).into_iter().flatten().collect();
    assert_eq!(seen.len(), points.len(), "every point read");
    let n = seen.len() as f64;
    [seen.iter().map(|c| f64::from(c.r)).sum::<f64>() / n, seen.iter().map(|c| f64::from(c.g)).sum::<f64>() / n, seen.iter().map(|c| f64::from(c.b)).sum::<f64>() / n]
}

/// Settings from Home's Settings place, on Display.
fn open_display(scene: &mut Scene<'_>) {
    until(scene, 5, "Home holding the focus", |s| s.property("screens", "shown") == "Home" && s.property("home", "activeFocus") == "true");
    let tree = scene.tree();
    let place = tree.find_named("place.5").expect("Home's Settings place");
    assert!(scene.focus_item(&tree, place));
    scene.pad(UiButton::Accept);
    wait_for(scene, "screens", "shown", "Settings", 2000);
    wait_for(scene, "settingsModel", "ready", "true", 5000);
    while scene.property("settingsModel", "section") != "1" {
        scene.pad(UiButton::PageDown);
        scene.wait(30);
    }
}

/// Down from the first row until the pane's sentence is `sentence`, then A.
fn choose_by_sentence(scene: &mut Scene<'_>, sentence: &str) {
    for _ in 0..60 {
        if scene.property("settingsSentence", "text") == sentence {
            scene.pad(UiButton::Accept);
            scene.wait(200);
            return;
        }
        scene.pad(UiButton::Down);
        scene.wait(60);
    }
    panic!("no row whose pane says {sentence}");
}

/// Display's row at `row` from the first, and A.
fn open_door(scene: &mut Scene<'_>, row: i32) {
    for _ in 0..12 {
        scene.pad(UiButton::Up);
    }
    wait_for(scene, "settingsModel", "focused", "0", 1000);
    for _ in 0..row {
        scene.pad(UiButton::Down);
    }
    wait_for(scene, "settingsModel", "focused", &row.to_string(), 1000);
    scene.pad(UiButton::Accept);
    scene.wait(200);
}

/// Display's rows: Shader levers third, Shaders fourth (settings_screen.json).
const LEVERS_ROW: i32 = 2;
const SHADERS_ROW: i32 = 3;

fn graphics(platform: &Platform) -> serde_json::Value {
    let file = platform.file(FILE_GRAPHICS).unwrap();
    platform.model_load(MODEL_GRAPHICS_CONFIG, None, file.file_name().unwrap().to_str().unwrap()).unwrap().unwrap_or_default()
}

/// Settings left for Home, and the Home card's Continue: the console resumed and its picture drawn again.
fn resume(scene: &mut Scene<'_>) {
    while scene.property("screens", "shown") == "Settings" {
        scene.pad(UiButton::Back);
        scene.wait(100);
    }
    until(scene, 5, "Home holding the focus", |s| s.property("screens", "shown") == "Home" && s.property("home", "activeFocus") == "true");
    scene.pad(UiButton::Menu);
    until(scene, 5, "the Home card in place", |s| s.property("screens", "shown") == "HomeCard" && s.property("homeCardCanvas", "slide") == "0");
    until(scene, 5, "the card holding the focus", |s| common::console::focus_within(s, "homeCardCanvas"));
    scene.pad(UiButton::Accept);
    until(scene, 5, "Continue resuming the console", |s| s.property("screens", "shown") == "Game" && s.property("session", "paused") == "false");
    until(scene, 5, "the cards gone from over the console", |s| ["homeCardCanvas", "listCardCanvas", "cardCanvas"].iter().all(|c| s.property(c, "visible") == "false"));
    scene.wait(500);
}

#[test]
fn qt_7_7_shaders_and_levers_change_the_running_console_s_picture_and_are_kept() {
    unsafe { std::env::set_var("SDL_AUDIODRIVER", "dummy") };
    let kwin = Compositor::start(3840, 2160);
    kwin.watch(600);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    let mut harness = Harness::new();
    let platform = Platform::load(library()).unwrap();
    let gpu = std::env::var("EMUSEN_GPU_DEVICE").ok();
    let world = sandbox("qt-app-shaders", &[("nes", "Brass Lantern.nes")]);
    let host = Host::new(&platform, "wayland", gpu.as_deref(), false, &world.dir("presets")).expect("a Vulkan device");
    let adoption = host.adoption();
    let cores = installation_with_snes(&world.dir("lib"));
    approved_console_core(&cores);
    let pack = platform.directory(DIR_SHADERS).unwrap().expect("the shader pack's folder");
    std::fs::create_dir_all(pack.join("test")).unwrap();
    std::fs::write(pack.join("test/magenta_tint.slangp"), PRESET).unwrap();
    std::fs::write(pack.join("test/tint.slang"), SHADER).unwrap();

    let sdl = Sdl::open();
    let pad = sdl.attach("EmuSen Test Xbox Pad", 0x045e, 0x028e);
    let mut scene = harness.load_prepared(MAIN_URL, 1280, 800, |window| unsafe { adoption.adopt(window.cast()) }.expect("Qt adopts the device"));
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("clock", "now", common::HELD_NOW));
    assert!(scene.set_property("program", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("program", "feed", &host.feed.address().to_string()));
    assert!(scene.set_property("library", "cores", cores.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 20000);
    until(&mut scene, 10, "the console listed", |s| s.property("library", "startable").contains("\"PlayStation\""));
    until(&mut scene, 20, "the virtual pad read", |s| s.property("pads", "connected") == "1");

    // The PlayStation started from the List card, plain: its picture as the core draws it.
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    list_card_start_console(&mut scene);
    scene.pad(UiButton::Accept);
    console_owns_the_pad(&mut scene, &sdl, &pad, "PlayStation");
    scene.wait(500);
    let plain = mean(&mut scene);
    eprintln!("plain: {plain:?}");

    // Settings ▸ Display ▸ Shaders ▸ PlayStation, the pack's magenta tint chosen with the console suspended.
    back_to_library(&mut scene, &sdl, &pad, "PlayStation (no disc)", "Home");
    open_display(&mut scene);
    open_door(&mut scene, SHADERS_ROW);
    choose_by_sentence(&mut scene, "PlayStation · None");
    choose_by_sentence(&mut scene, "test/magenta_tint.slangp");
    until(&mut scene, 5, "the choice stored", |_| graphics(&platform)["Consoles"]["PlayStation"]["ScreenFilter"] == "slang:test/magenta_tint.slangp");
    resume(&mut scene);
    // Half way to magenta: each channel half its own, red and blue half of white over it.
    let expected = [0.5 * plain[0] + 127.5, 0.5 * plain[1], 0.5 * plain[2] + 127.5];
    assert!((0..3).any(|i| (expected[i] - plain[i]).abs() > 20.0), "a picture the tint changes: {plain:?}");
    until(&mut scene, 10, "the picture drawn through the tint", |s| {
        let tinted = mean(s);
        (0..3).all(|i| (tinted[i] - expected[i]).abs() < 12.0)
    });
    assert_eq!(host.feed.problem(), None, "the preset's chain built and drew");
    let tinted = mean(&mut scene);
    eprintln!("tinted: {tinted:?}");

    // Shader levers ▸ PlayStation: the tint's lever, a slider, moved to nothing with Left.
    back_to_library(&mut scene, &sdl, &pad, "PlayStation (no disc)", "Home");
    open_display(&mut scene);
    open_door(&mut scene, LEVERS_ROW);
    choose_by_sentence(&mut scene, "PlayStation · magenta tint");
    wait_for(&mut scene, "settingsModel", "focused", "0", 1000);
    assert_eq!(scene.property("settingsSentence", "text"), "1 is the shader's own value.");
    for _ in 0..20 {
        scene.pad(UiButton::Left);
        scene.wait(20);
    }
    until(&mut scene, 5, "the lever stored at nothing", |_| graphics(&platform)["ShaderParameters"]["PlayStation"]["slang:test/magenta_tint.slangp"]["tint"] == "0");
    resume(&mut scene);
    until(&mut scene, 10, "the picture drawn without the tint", |s| {
        let back = mean(s);
        (0..3).all(|i| (back[i] - plain[i]).abs() < 6.0)
    });

    // Both kept: Settings opened again reads them back, the choice on the consoles' list and the lever on its page.
    back_to_library(&mut scene, &sdl, &pad, "PlayStation (no disc)", "Home");
    open_display(&mut scene);
    open_door(&mut scene, SHADERS_ROW);
    choose_by_sentence(&mut scene, "PlayStation · magenta tint");
    assert!(scene.tree().nodes.iter().any(|n| n.visible && n.text.as_deref() == Some("Default")), "the tint marked as the one chosen");
    while scene.property("screens", "shown") == "Settings" {
        scene.pad(UiButton::Back);
        scene.wait(100);
    }
    open_display(&mut scene);
    open_door(&mut scene, LEVERS_ROW);
    choose_by_sentence(&mut scene, "PlayStation · magenta tint");
    wait_for(&mut scene, "settingsModel", "focused", "0", 1000);
    assert_eq!(scene.property("settingsSentence", "text"), "1 is the shader's own value.");
    let tree = scene.tree();
    assert!(tree.nodes.iter().any(|n| n.visible && n.text.as_deref() == Some("0")), "the lever's value read back as 0");

    assert_eq!(host.feed.problem(), None, "the render thread met no failure");
    sdl.detach(pad);
    drop(scene);
    drop(harness);
}
