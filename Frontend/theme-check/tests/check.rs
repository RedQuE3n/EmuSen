// EmuSen_Qt.md §3.7: the static step's fixtures each named in the output with the finding it was written for, the clean one ok, and the
// loaded step's plumbing through a stand-in for theme-audit.

use std::path::{Path, PathBuf};

use emusen_kit_core::tokens::{ColourToken, Shadow};
use emusen_theme_check::{run, run_with};
use emusen_theme_check::tokens::{TokenType, property_of, registry_tokens};

fn themes() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/themes")
}

fn plain(path: &str) -> Result<String, String> {
    Ok(path.to_owned())
}

fn check(name: &str) -> (String, i32) {
    let folder = themes().join(name);
    run(&["theme-check".into(), "--static".into(), folder.display().to_string()], plain).unwrap()
}

#[test]
fn qt_3_7_a_clean_package_is_ok() {
    assert_eq!(check("clean"), ("ok\n".to_owned(), 0));
}

#[test]
fn qt_3_7_a_literal_in_a_part_is_found_on_its_line() {
    let (out, status) = check("literal");
    assert_eq!(status, 1);
    assert!(out.contains("literal/parts/Shelf.qml:8 words \"CONTINUE PLAYING\"\n"), "{out}");
    assert!(out.ends_with("\n1 findings\n"), "{out}");
}

#[test]
fn qt_3_5_an_import_outside_the_sandbox_list_is_found() {
    let (out, status) = check("multimedia");
    assert_eq!(status, 1);
    assert!(out.contains("multimedia/screens/Home.qml:4 import QtMultimedia\n"), "{out}");
    assert!(out.ends_with("\n1 findings\n"), "{out}");
}

#[test]
fn qt_3_4_an_accent_under_the_kit_floors_is_found_in_the_variant_that_sets_it() {
    let (out, status) = check("low-contrast");
    assert_eq!(status, 1);
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines.iter().all(|l| l.contains("low-contrast/theme.json:11 contrast ") || l.ends_with(" findings")), "{out}");
    assert!(out.contains("accent on surface is 2.13:1, under the 3:1 floor (dark)"), "{out}");
    assert!(out.contains("overAccent on accent is 2.39:1, under the 4.5:1 floor (dark)"), "{out}");
    assert!(!out.contains("(light)"), "the light accent clears: {out}");
    assert!(out.ends_with("\n4 findings\n"), "{out}");
}

#[test]
fn qt_3_4_token_names_types_and_text_scales_are_the_kits() {
    let root = std::env::temp_dir().join(format!("theme-check-tokens-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let json = r##"{
  "schema": 1, "api": "1.0", "name": "T", "author": "A", "licence": "L",
  "variants": ["dark"], "screens": [], "fontSizes": [1.5],
  "tokens": { "dark": {
    "radius": 8,
    "accent": "blue",
    "tileRadius": "8",
    "quick": 1.5,
    "breathes": 1,
    "surfaceRaised": "#8026262B",
    "homepage": null
  } },
  "previews": ["shot.png"],
  "extra": 1
}
"##;
    std::fs::write(root.join("theme.json"), json).unwrap();
    let (out, status) = run(&["theme-check".into(), "--static".into(), root.display().to_string()], plain).unwrap();
    let tail: Vec<String> = out.lines().map(|l| l.rsplit_once("theme.json:").map_or(l.to_owned(), |(_, t)| t.to_owned())).collect();
    assert_eq!(
        tail,
        vec![
            "3 schema fontSizes 1.5 is outside the text scales 1 to 1.3",
            "5 token radius is not a token of the kit",
            "6 token accent is a colour, #RRGGBB or #AARRGGBB",
            "7 token tileRadius is a number",
            "8 token quick is a whole number",
            "9 token breathes is true or false",
            "11 schema homepage is a value: a colour, a number or true/false",
            "13 path shot.png is missing",
            "14 note extra is not a key of schema 1, ignored",
            "8 findings",
        ],
        "{out}"
    );
    assert_eq!(status, 1);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn qt_3_7_screens_listed_and_present_agree_and_imports_stay_inside() {
    let root = std::env::temp_dir().join(format!("theme-check-screens-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("screens")).unwrap();
    let json = "{\"schema\": 1, \"api\": \"1.0\", \"name\": \"T\", \"author\": \"A\", \"licence\": \"L\", \"variants\": [\"dark\"], \"screens\": [\"home\", \"games\"]}";
    std::fs::write(root.join("theme.json"), json).unwrap();
    std::fs::write(root.join("screens/Home.qml"), "import QtQuick\nimport \"../../elsewhere\"\nimport \"../parts\"\nItem {}\n").unwrap();
    std::fs::write(root.join("screens/Splash.qml"), "import QtQuick\nItem {}\n").unwrap();
    let (out, _) = run(&["theme-check".into(), "--static".into(), root.display().to_string()], plain).unwrap();
    for want in [
        "theme.json:1 screen games is listed and screens/Games.qml is missing",
        "theme.json:1 screen screens/Splash.qml is not listed in screens, so it never loads",
        "screens/Home.qml:2 path \"../../elsewhere\" leaves the package",
        "screens/Home.qml:3 path \"../parts\" is not a folder of the package",
    ] {
        assert!(out.contains(want), "{want}\n{out}");
    }
    assert!(out.ends_with("\n4 findings\n"), "{out}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn qt_3_7_a_missing_manifest_and_a_wrong_folder() {
    let root = std::env::temp_dir().join(format!("theme-check-empty-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let (out, status) = run(&["theme-check".into(), "--static".into(), root.display().to_string()], plain).unwrap();
    assert!(out.ends_with("theme.json:1 schema theme.json is missing\n1 findings\n") && status == 1, "{out}");
    assert!(run(&["theme-check".into()], plain).unwrap_err().contains("a folder is needed"));
    assert!(run(&["theme-check".into(), "--loud".into()], plain).unwrap_err().contains("not an option"));
    assert!(run(&["theme-check".into(), "--static".into(), "/no/such/folder".into()], plain).unwrap_err().contains("no such folder"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn rule_q_every_colour_token_names_a_read_only_colour_of_the_registry_and_back() {
    let registry = registry_tokens();
    assert!(registry.len() > 60, "the registry's Tokens were read: {}", registry.len());
    assert!(!registry.iter().any(|(n, _)| n == "form" || n == "variant"), "the switches are not tokens");
    let colours: Vec<&str> = registry.iter().filter(|(_, t)| *t == TokenType::Colour).map(|(n, _)| n.as_str()).collect();
    let mut named: Vec<&str> = ColourToken::ALL.iter().map(|t| property_of(*t)).collect();
    named.push(Shadow::COLOUR_PROPERTY);
    assert_eq!(colours, named, "the palette's colour tokens and the raised shadow's, in the registry's order");
}

/// A stand-in for theme-audit: a script that answers for the package named last, and leaves a mark that it ran.
fn stand_in(name: &str, body: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("theme-check-loaded-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let script = dir.join("theme-audit");
    std::fs::write(&script, format!("#!/bin/bash\nfolder=\"${{@: -1}}\"\ntouch \"{}/ran\"\n{body}\n", dir.display())).unwrap();
    std::process::Command::new("chmod").arg("+x").arg(&script).status().unwrap();
    script
}

#[test]
fn qt_3_7_the_loaded_step_s_lines_are_findings_of_the_folder_as_named() {
    let program = stand_in("lines", "echo \"$folder/screens/Games.qml:1 fit 1280x720 dark: past its panel: 4 (4 QQuickText heading)\"\nexit 1");
    // The shell resolves the folder as named to the package on disk; the lines name it as the caller did.
    let real = |_: &str| -> Result<String, String> { Ok(format!("{}/", themes().join("clean").display())) };
    let (out, status) = run_with(&["theme-check".into(), "MyTheme/".into()], real, Some(&program)).unwrap();
    assert_eq!(out, "MyTheme/screens/Games.qml:1 fit 1280x720 dark: past its panel: 4 (4 QQuickText heading)\n1 findings\n");
    assert_eq!(status, 1);
    let _ = std::fs::remove_dir_all(program.parent().unwrap());
}

#[test]
fn qt_3_7_a_package_with_static_findings_is_not_loaded() {
    let program = stand_in("static", "exit 0");
    let (out, status) = run_with(&["theme-check".into(), themes().join("literal").display().to_string()], plain, Some(&program)).unwrap();
    assert!(out.contains("literal/theme.json:1 note the screens were not loaded: the findings above come first\n"), "{out}");
    assert!(out.ends_with("\n1 findings\n") && status == 1, "the note is not counted: {out}");
    assert!(!program.parent().unwrap().join("ran").exists(), "theme-audit was not run");
    let _ = std::fs::remove_dir_all(program.parent().unwrap());
}

#[test]
fn qt_3_7_the_loaded_step_without_its_program_or_with_a_failing_one_cannot_run() {
    let folder = themes().join("clean").display().to_string();
    let words = run_with(&["theme-check".into(), folder.clone()], plain, None).unwrap_err();
    assert!(words.contains("EMUSEN_THEME_AUDIT") && words.contains("--static"), "{words}");
    let program = stand_in("fails", "echo 'the platform library: missing' >&2\nexit 2");
    let words = run_with(&["theme-check".into(), folder], plain, Some(&program)).unwrap_err();
    assert!(words.contains("theme-audit did not run") && words.contains("the platform library: missing"), "{words}");
    let _ = std::fs::remove_dir_all(program.parent().unwrap());
}

#[test]
fn qt_3_7_each_state_fixture_keeps_the_static_rules_so_its_one_fault_is_its_state_s() {
    for name in ["state-nogame", "state-nocover", "state-disabled", "state-pressed", "state-empty", "state-many", "state-splash", "state-fade", "state-declined"] {
        assert_eq!(check(name), ("ok\n".to_owned(), 0), "{name}");
    }
}
