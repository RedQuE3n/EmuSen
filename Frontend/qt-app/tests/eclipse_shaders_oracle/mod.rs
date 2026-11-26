//! Shared oracle for Eclipse Screen filter / Shaders (#20) on software and real RX 6800 Vulkan card.
#![allow(dead_code)]

use crate::common::wait_for;
use crate::eclipse_board::{glyphs_in, held, kit_tokens, picture, read, rows, same, words_under};
use crate::eclipse_pad::{focused, press};
use crate::eclipse_settings_oracle::{at, centre, settings, settle, shown};
use emusen_client_fixture::{library, World};
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::input::PadControl;
use emusen_platform_client::galaxia::consts::{DIR_SHADERS, FILE_GRAPHICS, MODEL_APP_SETTINGS, MODEL_GRAPHICS_CONFIG};
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_endymion::reading::sdl_button;
use emusen_qt_models::themes;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::virtual_pad::{Sdl, VirtualPad};
use emusen_qt_test::Scene;
use serde_json::json;

const BOARD: &str = include_str!("../eclipse/shaders.tsv");
const WORLD_JSON: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/eclipse/settings_s6_world.json");
const UTC_941_PM: &str = "1794001260";

const ROYALE_SLANG: &str = concat!(
    "#version 450\n",
    "#pragma parameter scanline_strength \"Scanline strength\" 0.6 0.0 1.0 0.05\n",
    "#pragma parameter glow \"Glow\" 0.25 0.0 1.0 0.05\n",
    "#pragma parameter curvature \"Curve of the glass\" 0.1 0.0 1.0 0.05\n",
    "#pragma parameter sharpen \"Sharpen\" 0.2 0.0 1.0 0.05\n",
    "#pragma stage vertex\nvoid main() {}\n",
    "#pragma stage fragment\nvoid main() {}\n",
);

const TINT_SLANG: &str = r#"#version 450
#pragma parameter tint "Tint toward magenta" 1.0 0.0 1.0 0.05
#pragma parameter mix_strength "Mix strength" 0.8 0.0 1.0 0.05
#pragma parameter warmth "Warmth" 0.2 0.0 1.0 0.05
layout(std140, set = 0, binding = 0) uniform UBO
{
    mat4 MVP;
    float tint;
    float mix_strength;
    float warmth;
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
    FragColor = vec4(mix(c, vec3(1.0, 0.0, 1.0), 0.7 * tint), 1.0);
}
"#;

fn until(scene: &mut Scene<'_>, what: &str, mut pred: impl FnMut(&mut Scene<'_>) -> bool) {
    for _ in 0..120 {
        if pred(scene) {
            return;
        }
        scene.wait(50);
    }
    panic!("timed out waiting for {what}");
}

fn press_handoff(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, control: PadControl, what: &str, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    let code = sdl_button(control).expect("pad button");
    sdl.press(pad, code, true);
    until(scene, what, &mut done);
    sdl.press(pad, code, false);
    scene.wait(80);
}

fn with_shaders_world(tag: &str, with_host: bool, f: impl FnOnce(&mut Scene<'_>, &Sdl, &VirtualPad, &Platform)) {
    std::env::set_var("TZ", "UTC");
    std::env::set_var("LC_ALL", "en_US.UTF-8");
    let world = World::new(tag);
    let snes = world.dir("Roms/snes");
    std::fs::write(snes.join("Aurora Drift.sfc"), vec![0u8; 4096]).unwrap();
    let platform = Platform::load(library()).unwrap();

    let mut app = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    app["RomDirectory"] = json!(world.root.join("Roms").to_str().unwrap());
    app["EmuSenTheme"] = json!("eclipse");
    app["OpenEmuFallback"] = json!(false);
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &app).unwrap();

    let pack = platform.directory(DIR_SHADERS).unwrap().expect("shaders dir");
    std::fs::create_dir_all(pack.join("crt")).unwrap();
    std::fs::write(pack.join("crt/royale.slang"), ROYALE_SLANG).unwrap();
    std::fs::write(pack.join("crt/crt-royale.slangp"), "shaders = 1\nshader0 = royale.slang\n").unwrap();
    std::fs::write(pack.join("crt/tint.slang"), TINT_SLANG).unwrap();
    std::fs::write(pack.join("crt/tint-preview.slangp"), "shaders = 1\nshader0 = tint.slang\nfilter_linear0 = false\nscale_type0 = viewport\n").unwrap();

    let graphics_file = platform.file(FILE_GRAPHICS).unwrap();
    let graphics = json!({
        "Consoles": { "SNES": { "ScreenFilter": "slang:crt/crt-royale.slangp" } }
    });
    platform.model_save(MODEL_GRAPHICS_CONFIG, None, graphics_file.file_name().unwrap().to_str().unwrap(), &graphics).unwrap();

    let power = world.power_supply().join("BAT0");
    std::fs::create_dir_all(&power).unwrap();
    for (key, value) in [("type", "Battery"), ("capacity", "82"), ("status", "Discharging")] {
        std::fs::write(power.join(key), format!("{value}\n")).unwrap();
    }

    let sdl = Sdl::open();
    let pad = sdl.attach("Simulated pad", 0x045e, 0x028e);
    let mut harness = emusen_qt_test::Harness::new();
    let gpu = std::env::var("EMUSEN_GPU_DEVICE").ok();
    let host = with_host.then(|| emusen_qt_play::host::Host::new(&platform, "wayland", gpu.as_deref(), false, &world.dir("presets")).expect("a Vulkan device"));
    let mut scene = match host.as_ref() {
        Some(host) => {
            let adoption = host.adoption();
            harness.load_prepared(MAIN_URL, 1280, 800, move |window| unsafe { adoption.adopt(window.cast()) }.expect("Qt adopts the device"))
        }
        None => harness.load(MAIN_URL, 1280, 800),
    };
    if let Some(host) = host.as_ref() {
        assert!(scene.set_property("program", "feed", &host.feed.address().to_string()));
    }
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("settingsModel", "world", WORLD_JSON));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "settingsModel", "ready", "true", 5000);
    wait_for(&mut scene, "pads", "connected", "1", 5000);
    assert!(scene.set_property("clock", "now", UTC_941_PM));
    scene.run_pad_clock();

    until(&mut scene, "Home holding focus", |s| focused(s) == "place.0");
    for _ in 0..5 {
        press(&mut scene, &sdl, &pad, PadControl::Right);
    }
    press(&mut scene, &sdl, &pad, PadControl::South);
    wait_for(&mut scene, "screens", "shown", "Settings", 2000);
    assert!(scene.set_property("program", "theme", &themes::built_in_folder("eclipse")));
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);
    until(&mut scene, "Eclipse draws Settings", |s| s.property("theme", "settings") == "true");
    until(&mut scene, "row.0 focused", |s| focused(s) == "row.0");

    press(&mut scene, &sdl, &pad, PadControl::RightShoulder);
    settle(&mut scene);
    assert_eq!(at(&mut scene).1, "display");
    let shaders_row = settings(&mut scene).shown_rows().iter().position(|r| r.id == "shaders").expect("shaders row");
    while (at(&mut scene).0 as usize) < shaders_row {
        press(&mut scene, &sdl, &pad, PadControl::Down);
        settle(&mut scene);
    }
    press_handoff(&mut scene, &sdl, &pad, PadControl::South, "#20 shaders shown", |s| s.property("settingsModel", "shadersShown") == "true" && shown(s, "shadersEclipseCanvas"));
    f(&mut scene, &sdl, &pad, &platform);
    sdl.detach(pad);
    drop(scene);
    drop(harness);
    drop(host);
}

pub fn oracle_software() {
    with_shaders_world("qt-app-eclipse-shaders", false, |scene, sdl, pad, platform| {
        let mut differences = Vec::new();
        assert_eq!(scene.property("theme", "holdsFocus"), "false");
        let dump = scene.property("settingsModel", "shadersBoard");
        assert!(!dump.contains("ms of each"), "unmeasured timing badge omitted: {dump}");
        assert!(!dump.contains("ml"), "no fabricated timing units: {dump}");
        assert!(!dump.contains("This game") && !dump.contains("Newer pack") && !dump.contains("Get it"));

        let (board_height, table) = rows(BOARD);
        let k = DESIGN_HEIGHT / board_height;
        let tokens = kit_tokens(scene);
        for (width, height) in SIZES {
            scene.resize(width, height);
            scene.wait(300);
            let tree = scene.tree();
            let unit: f64 = read(&tree, "shadersEclipseCanvas", "unit").parse().expect("canvas unit");
            let at_size = format!("{width}x{height}");
            held(&tree, &table, k, unit, f64::from(width) / unit, &tokens, &at_size, &|_, _, _| false, &mut differences);
            let help_words = words_under(&tree, "help");
            if help_words != ["Use shader", "Hold to see off", "Favourite", "Scrub split", "Step split", "Back"] {
                differences.push(format!("{at_size}: help words {help_words:?}"));
            }
            let glyphs = glyphs_in(&tree, "help");
            if glyphs != ["A", "X", "Y", "RS", "LB", "B"] {
                differences.push(format!("{at_size}: help glyphs {glyphs:?}"));
            }
            for (name, want) in [
                ("crumbSettings", "Tools"),
                ("crumbScreenFilter", "Shaders"),
                ("battery", "82%"),
                ("clock", "9:41 PM"),
                ("previewBeforeWords", "No filter"),
                ("previewAfterWords", "crt-royale"),
                ("previewScrubWords", "RS or LB / RB moves split"),
                ("presetHeading", "crt-royale"),
                ("allSettingsWords", "All 4 parameters ›"),
                ("shadersCounter", "1 of 2"),
            ] {
                let got = read(&tree, name, "text");
                if got != want {
                    differences.push(format!("{at_size}: {name} text {got:?} != {want:?}"));
                }
            }
            let div_x: f64 = read(&tree, "previewDivider", "x").parse().unwrap_or(0.0);
            let box_w: f64 = read(&tree, "previewBox", "width").parse().unwrap_or(0.0);
            if !same(div_x, box_w / 2.0 - 2.0 * k) {
                differences.push(format!("{at_size}: divider not centered at 0.5 (div_x={div_x}, box_w={box_w})"));
            }
        }
        scene.resize(1920, 1200);
        scene.wait(250);
        assert!(differences.is_empty(), "{}", differences.join("\n"));

        // Y toggles favourite and updates Favourites category count 0 -> 1.
        assert_eq!(read(&scene.tree(), "shaderCatCount.0", "text"), "0");
        press(scene, sdl, pad, PadControl::North);
        until(scene, "favourite added", |s| read(&s.tree(), "shaderCatCount.0", "text") == "1" && shown(s, "shaderRowStar.0"));

        // LB/RB moves the vertical divider line left/right cleanly.
        let mid_div: f64 = read(&scene.tree(), "previewDivider", "x").parse().unwrap();
        press(scene, sdl, pad, PadControl::LeftShoulder);
        until(scene, "LB moves divider left", |s| read(&s.tree(), "previewDivider", "x").parse::<f64>().unwrap() < mid_div - 10.0);
        press(scene, sdl, pad, PadControl::RightShoulder);
        press(scene, sdl, pad, PadControl::RightShoulder);
        until(scene, "RB moves divider right", |s| read(&s.tree(), "previewDivider", "x").parse::<f64>().unwrap() > mid_div + 10.0);

        // Right stick horizontal scrub smoothly slides the divider without leaving focus.
        let stick_start: f64 = read(&scene.tree(), "previewDivider", "x").parse().unwrap();
        sdl.axis(pad, 2, -20000);
        until(scene, "RS left scrubs divider left", |s| read(&s.tree(), "previewDivider", "x").parse::<f64>().unwrap() < stick_start - 40.0);
        sdl.axis(pad, 2, 0);

        // Hold X compares without filter, release restores active shader.
        let west = sdl_button(PadControl::West).unwrap();
        sdl.press(pad, west, true);
        until(scene, "X held hides filter", |s| read(&s.tree(), "previewAfterWords", "text") == "No filter");
        sdl.press(pad, west, false);
        until(scene, "X release restores filter", |s| read(&s.tree(), "previewAfterWords", "text") == "crt-royale");

        // Navigate across left/middle/sliders/All 4 settings and switch scope cleanly.
        press(scene, sdl, pad, PadControl::Left);
        until(scene, "left reaches category", |s| read(&s.tree(), "shaderCatRing.1", "visible") == "true");
        press(scene, sdl, pad, PadControl::Up);
        press(scene, sdl, pad, PadControl::Up);
        until(scene, "up reaches source tab", |s| read(&s.tree(), "sourceRing.1", "visible") == "true");
        press(scene, sdl, pad, PadControl::Down);
        press(scene, sdl, pad, PadControl::Right);
        until(scene, "right returns to preset list", |s| read(&s.tree(), "shaderRowRing.0", "visible") == "true");
        press(scene, sdl, pad, PadControl::Up);
        press(scene, sdl, pad, PadControl::Right);
        until(scene, "right reaches scope switch", |s| read(&s.tree(), "scopeRing.1", "visible") == "true");
        press(scene, sdl, pad, PadControl::Left);
        until(scene, "left switches to Every console", |s| read(&s.tree(), "scopeRing.0", "visible") == "true");
        press(scene, sdl, pad, PadControl::Right);
        press(scene, sdl, pad, PadControl::Down);
        until(scene, "down reaches first slider", |s| read(&s.tree(), "shaderSliderRing.0", "visible") == "true");
        let before_slider = read(&scene.tree(), "shaderSliderVal.0", "text");
        press(scene, sdl, pad, PadControl::Right);
        until(scene, "right steps slider live", |s| read(&s.tree(), "shaderSliderVal.0", "text") != before_slider);
        for _ in 0..3 {
            press(scene, sdl, pad, PadControl::Down);
        }
        until(scene, "down reaches All 4 settings link", |s| read(&s.tree(), "allSettingsRing", "visible") == "true");

        // Enter #60 via A on All 4 settings and return cleanly via B -> #20 -> B -> display.
        press_handoff(scene, sdl, pad, PadControl::South, "#60 opens", |s| s.property("settingsModel", "shadersShown") == "false" && at(s).1 == "shaderParams:SNES");
        press_handoff(scene, sdl, pad, PadControl::East, "B returns to #20", |s| s.property("settingsModel", "shadersShown") == "true");

        // Remove the pack via #14 confirm dialog: Cancel leaves pack, Confirm clears DIR_SHADERS.
        let (rx, ry) = centre(scene, "removePackButton");
        scene.click(rx, ry);
        until(scene, "remove confirm opens", |s| s.property("settingsModel", "shadersBoard").contains("\"confirmOpen\":true"));
        press(scene, sdl, pad, PadControl::East);
        until(scene, "cancel closes confirm", |s| s.property("settingsModel", "shadersBoard").contains("\"confirmOpen\":false") && !shown(s, "dialogDisc"));
        until(scene, "navScope regains focus after cancel", |s| s.property("shadersNavScope", "activeFocus") == "true");
        let pack = platform.directory(DIR_SHADERS).unwrap().unwrap();
        assert!(pack.join("crt/crt-royale.slangp").exists());
        scene.click(rx, ry);
        until(scene, "remove confirm reopens", |s| s.property("settingsModel", "shadersBoard").contains("\"confirmOpen\":true"));
        press(scene, sdl, pad, PadControl::Right);
        press(scene, sdl, pad, PadControl::South);
        until(scene, "pack cleared locally", |s| !pack.join("crt/crt-royale.slangp").exists() && !shown(s, "dialogDisc"));
        until(scene, "navScope regains focus", |s| s.property("shadersNavScope", "activeFocus") == "true");

        press_handoff(scene, sdl, pad, PadControl::East, "B returns to Display", |s| s.property("settingsModel", "shadersShown") == "false" && at(s).1 == "display");
    });
}

pub fn oracle_gpu() {
    with_shaders_world("qt-app-eclipse-shaders-gpu", true, |scene, sdl, pad, _| {
        scene.resize(1920, 1200);
        scene.wait(300);
        let (tx, ty) = centre(scene, "shaderRow.1");
        scene.click(tx, ty);
        until(scene, "tint-preview active with Vulkan feed", |s| {
            read(&s.tree(), "presetHeading", "text") == "tint-preview" && s.property("settingsModel", "shaderPreviewFeed") != "0" && s.property("settingsModel", "shaderPreviewPending") == "false"
        });
        scene.wait(400);
        picture(scene, "shaders-1920x1200-card.png");
        scene.resize(1920, 1200);
        scene.wait(300);

        // Sample unfiltered left side vs magenta-tinted right side at 1920x1200 (y=328 hits pure #17306E sky).
        let left_pt = (1150.0, 328.0);
        let mid_pt = (1340.0, 328.0);
        let right_pt = (1650.0, 328.0);
        let cols = scene.colours_at(&[left_pt, right_pt]);
        let (left, right) = (cols[0].expect("left pixel"), cols[1].expect("right pixel"));
        assert!(left.b > left.r + 30, "unfiltered sky blue on left: {left:?}");
        assert!(right.r > left.r + 40 && right.r > right.g + 30, "filtered tint shifted toward magenta on right: left={left:?} right={right:?}");

        // Move split line left via LB so mid_pt flips from unfiltered to tinted.
        let mid_before = scene.colours_at(&[mid_pt])[0].expect("mid before");
        for _ in 0..3 {
            press(scene, sdl, pad, PadControl::LeftShoulder);
        }
        scene.wait(250);
        let mid_after = scene.colours_at(&[mid_pt])[0].expect("mid after");
        assert!(mid_after.r > mid_before.r + 40, "LB moved split boundary past mid_pt: before={mid_before:?} after={mid_after:?}");

        // Smooth right-stick tilt right scrubs divider back past mid_pt while leaving right_pt tinted.
        sdl.axis(pad, 2, 16000);
        until(scene, "RS right scrubs divider past mid_pt", |s| {
            let col = s.colours_at(&[mid_pt])[0].expect("mid restored");
            col.b > col.r + 30 && col.r < mid_after.r - 40
        });
        sdl.axis(pad, 2, 0);
        scene.wait(100);
        let tree = scene.tree();
        let div_scene_x = read(&tree, "previewBox", "x").parse::<f64>().unwrap() + read(&tree, "previewDivider", "x").parse::<f64>().unwrap();
        assert!(div_scene_x < right_pt.0 - 40.0, "RS stopped before right_pt (div_scene_x={div_scene_x})");

        // Hold X shows unfiltered pixels across the entire box, release restores tint.
        let west = sdl_button(PadControl::West).unwrap();
        sdl.press(pad, west, true);
        scene.wait(200);
        let held_right = scene.colours_at(&[right_pt])[0].expect("right while X held");
        assert!(held_right.b > held_right.r + 30 && held_right.r < right.r - 40, "X hold shows unfiltered: held={held_right:?} vs filtered={right:?}");
        sdl.press(pad, west, false);
        scene.wait(200);
    });
}
