//! The program hosting an EmuSen theme (EmuSen_Qt.md §3.5): the theme's screens over the reference's where it provides them,
//! the reference wherever it does not or once it fails, its splash over Moonrise, and one engine however often it is switched.

use crate::common;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_qt::MAIN_URL;
use emusen_qt_models::themes;

const THEMES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../theme-host/tests/themes");

fn choose(scene: &mut emusen_qt_test::Scene<'_>, theme: &str) {
    let folder = if theme.is_empty() { String::new() } else { format!("{THEMES}/{theme}") };
    assert!(scene.set_property("program", "theme", &folder));
    scene.wait(100);
}

#[test]
fn qt_3_5_the_theme_s_screens_replace_the_reference_s_and_a_failed_theme_shows_the_reference() {
    let _world = sandbox("qt-app-theme", &[("NES", "One.nes")]);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));

    // At the start a theme's splash is drawn over Moonrise, faded with it, and gone once Moonrise is done.
    choose(&mut scene, "splash");
    assert_eq!(scene.property("theme", "splash"), "true", "{}", scene.property("theme", "warnings"));
    assert_eq!(scene.property("splashLayer", "visible"), "true");
    assert_eq!(scene.property("stubSplash", "width"), "1280", "the splash fills the window");
    assert!(scene.set_property("program", "splash", "false"));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "theme", "splash", "false", 2000);
    assert_eq!(scene.property("splashLayer", "visible"), "false");

    // The theme's Home stands in for the reference's; Choose a console, which it omits, stays the reference's.
    assert_eq!(scene.property("theme", "home"), "true");
    assert_eq!(scene.property("splashHome", "visible"), "true");
    assert_eq!(scene.property("reference", "home"), "false", "the reference's Home gives way to a theme's");
    assert_eq!((scene.property("theme", "consoles").as_str(), scene.property("reference", "consoles").as_str()), ("false", "true"), "a screen the theme omits is the reference's");

    // A theme that fails to load is reported with the engine's words, and the reference is shown.
    choose(&mut scene, "models");
    assert_eq!(scene.property("theme", "failed"), "true");
    assert!(scene.property("theme", "warnings").contains("imports EmuSen.Models"), "{}", scene.property("theme", "warnings"));
    assert_eq!(scene.property("theme", "home"), "false");
    assert_eq!(scene.property("home", "visible"), "true", "the reference's Home is shown for a failed theme");

    // A theme's token override is set on its own engine, where its screen draws the theme's accent, and on the program's, which
    // Settings, the cards and the page read (decided by the tester, 2026-11-13; EmuSen_Qt.md §3.5.1).
    choose(&mut scene, "accent");
    assert_eq!(scene.property("accentProbe", "color"), "#b4442a", "{}", scene.property("theme", "warnings"));
    let program = scene.kit_tokens().map(|t| (t.accent().red(), t.accent().green(), t.accent().blue()));
    assert_eq!(program, Some((0xB4, 0x44, 0x2A)), "the program's accent is the theme's");
    // Its own shadow colour too (R-20261111-68), on both engines.
    assert_eq!(scene.property("shadowProbe", "color"), "#2a1b4c", "{}", scene.property("theme", "warnings"));
    let program = scene.kit_tokens().map(|t| (t.shadow_color().red(), t.shadow_color().green(), t.shadow_color().blue()));
    assert_eq!(program, Some((0x2A, 0x1B, 0x4C)), "the program's shadow is the theme's");
    // A number and a switch too (EmuSen_Qt.md §3.5.1): the theme's screen and the program's tokens read the theme's.
    assert_eq!(scene.property("tokenProbe", "tileRadius"), "21", "{}", scene.property("theme", "warnings"));
    assert_eq!(scene.property("tokenProbe", "breathes"), "false");
    let program = scene.kit_tokens().map(|t| (*t.tile_radius(), *t.breathes()));
    assert_eq!(program, Some((21.0, false)), "the program's radius and breathing are the theme's");

    // Switching at run time makes a new engine and lets the last one go; no theme, no engine.
    choose(&mut scene, "stub");
    assert_eq!(scene.property("stubHome", "visible"), "true");
    // A theme with no overrides gives the program the kit's tokens back.
    let kit = scene.kit_tokens().map(|t| (t.accent().red(), t.accent().green(), t.accent().blue(), *t.tile_radius(), *t.form()));
    let (r, g, b, radius, form) = kit.expect("the program's tokens");
    let form = emusen_kit_qt::kinds::token_form(form);
    assert_eq!((r, g, b, radius), (0x4C, 0x9A, 0xE8, emusen_kit_core::tokens::spacing(form).tile_radius), "the program's are the kit's");
    choose(&mut scene, "splash");
    choose(&mut scene, "stub");
    assert_eq!(scene.property("host.theme", "liveEngines"), "1", "switching twice leaks no engine");
    assert_eq!(scene.property("theme", "home"), "true");
    choose(&mut scene, "");
    assert_eq!(scene.property("chosenTheme", "status"), "Null", "no theme, no host and no engine");
    assert_eq!(scene.property("host.theme", "liveEngines"), "");
    assert_eq!(scene.property("home", "visible"), "true");
}

/// Every file under `dir`, by its path from `root`.
fn files(root: &std::path::Path, dir: &std::path::Path, out: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            files(root, &path, out);
        } else {
            out.push(path.strip_prefix(root).unwrap().display().to_string());
        }
    }
}

#[test]
fn qt_3_4_2_the_program_s_reference_theme_is_the_package_theme_audit_holds_to_its_canvas() {
    // theme-audit's test holds Frontend/themes/emusen's Choose a console to Testing/mockups/Consoles.mockup.spec over the
    // canvas's eleven consoles (EmuSen_Kit.md §7.104); the program loads each built-in from the resources, the reference
    // first. Each file of a built-in's package is in the resources under its own path, from that file, and nothing else is.
    let qrc = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/themes.qrc")).unwrap();
    assert_eq!(emusen_qt::reference_theme(), themes::built_in_folder(themes::REFERENCE));
    for id in themes::BUILT_IN {
        let folder = themes::built_in_folder(id);
        let prefix = folder.trim_start_matches("qrc:").trim_end_matches('/');
        let block = qrc.split("<qresource prefix=\"").find(|b| b.starts_with(&format!("{prefix}\">"))).expect("the built-in's block");
        let block = &block[..block.find("</qresource>").unwrap()];
        let mut listed: Vec<(String, String)> = block
            .split("<file alias=\"")
            .skip(1)
            .map(|f| {
                let (alias, rest) = f.split_once("\">").unwrap();
                (alias.to_owned(), rest[..rest.find("</file>").unwrap()].to_owned())
            })
            .collect();
        let package = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../themes/{id}"));
        let mut on_disk = Vec::new();
        files(&package, &package, &mut on_disk);
        let mut wanted: Vec<(String, String)> = on_disk.into_iter().map(|f| (f.clone(), format!("../themes/{id}/{f}"))).collect();
        listed.sort();
        wanted.sort();
        assert_eq!(listed, wanted, "the resources' {id} is Frontend/themes/{id}, file for file");
    }
    assert!(qrc.contains("<file alias=\"screens/Consoles.qml\">../themes/emusen/screens/Consoles.qml</file>"), "Choose a console is the reference's");
}

#[test]
fn qt_3_4_5_the_program_carries_the_built_in_themes_but_the_retired_ones() {
    // Constellation, Horizon, Meridian and Orrery were retired 2026-11-23 (EmuSen_Qt.md §3.4.5).
    let qrc = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/themes.qrc")).unwrap();
    let carried: Vec<&str> =
        qrc.split("<qresource prefix=\"/emusen/themes/").skip(1).map(|b| &b[..b.find('"').unwrap()]).collect();
    assert_eq!(carried, ["emusen", "eclipse"]);
    for retired in ["constellation", "horizon", "meridian", "orrery"] {
        assert!(!qrc.contains(&format!("themes/{retired}/")), "no file of {retired}'s in the resources");
        assert!(!std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../themes").join(retired).exists(), "{retired}'s folder is gone");
    }
}
