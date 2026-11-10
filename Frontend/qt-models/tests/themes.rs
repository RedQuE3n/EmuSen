//! The Themes tab's facts (EmuSen_Qt.md §3.8) over a scratch themes store holding one theme of each kind: the rows, the id
//! resolved, the check, and when Use is offered.

use emusen_qt_models::themes::{self, Checked, Kind, Resolved, REFERENCE};
use serde_json::json;
use std::path::{Path, PathBuf};

const REFERENCE_MANIFEST: &str = include_str!("../../themes/emusen/theme.json");
const REFERENCE_FOLDER: &str = "qrc:/emusen/themes/emusen/";
const CLEAN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../theme-check/tests/themes/clean");
const LITERAL: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../theme-check/tests/themes/literal");

/// A scratch store: `dawn`, a copy of theme-check's clean package; `book`, a small ES-DE theme; and folders that are no theme.
fn store(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("emusen-themes-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    copy(Path::new(CLEAN), &root.join("dawn"));
    let book = root.join("book");
    std::fs::create_dir_all(book.join("snes")).unwrap();
    std::fs::write(book.join("capabilities.xml"), "<themeCapabilities><themeName>Picture Book</themeName><variant name=\"one\"><selectable>true</selectable></variant><variant name=\"two\"><selectable>true</selectable></variant></themeCapabilities>").unwrap();
    std::fs::write(book.join("theme.xml"), "<theme><view name=\"system, gamelist\"><text name=\"t\"><text>x</text></text></view></theme>").unwrap();
    std::fs::create_dir_all(root.join("notes")).unwrap();
    copy(Path::new(CLEAN), &root.join(".hidden"));
    copy(Path::new(CLEAN), &root.join("emusen"));
    root
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap().flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

#[test]
fn the_reference_comes_first_then_each_theme_of_the_store_in_name_order() {
    let root = store("rows");
    let listing = themes::listing(&json!({}), Some(&root), REFERENCE_MANIFEST, REFERENCE_FOLDER);
    let ids: Vec<(&str, Kind)> = listing.rows.iter().map(|r| (r.id.as_str(), r.kind)).collect();
    assert_eq!(ids, [(REFERENCE, Kind::EmuSen), ("book", Kind::Esde), ("dawn", Kind::EmuSen)], "no folder that is not a theme, hidden, or named as the reference");
    let reference = &listing.rows[0];
    assert_eq!((reference.name.as_str(), reference.api.as_str(), reference.variants.as_str()), ("EmuSen", "1.0", "dark and light"));
    assert_eq!(reference.preview, "qrc:/emusen/themes/emusen/assets/previews/home.png");
    assert!(reference.current && reference.checked == Checked::Ok, "the reference is in use with nothing stored, and built clean");
    let book = &listing.rows[1];
    assert_eq!((book.name.as_str(), book.api.as_str(), book.variants.as_str()), ("Picture Book", "ES-DE", "2 variants"));
    assert!(listing.words.is_empty());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn use_is_offered_only_for_an_emusen_package_checked_clean_and_not_in_use() {
    let root = store("usable");
    let mut rows = themes::listing(&json!({}), Some(&root), REFERENCE_MANIFEST, REFERENCE_FOLDER).rows;
    assert!(!rows[0].usable(), "the reference is in use");
    assert!(!rows[2].usable(), "a package not yet checked");
    rows[2].checked = Checked::Findings(vec!["dawn/screens/Home.qml:3 literal".into()]);
    assert!(!rows[2].usable(), "a package with findings has no Use");
    assert_eq!(rows[2].value(), "1 finding");
    rows[2].checked = Checked::Ok;
    assert!(rows[2].usable());
    rows[1].checked = Checked::Ok;
    assert!(!rows[1].usable(), "an ES-DE theme is checked here, not chosen");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_stored_id_resolves_through_the_store_and_a_missing_one_falls_back_with_words() {
    let root = store("resolve");
    assert_eq!(themes::resolve("", Some(&root)), Resolved::Reference);
    assert_eq!(themes::resolve(REFERENCE, Some(&root)), Resolved::Reference, "emusen is the reference in the resources, never a folder");
    assert_eq!(themes::resolve("dawn", Some(&root)), Resolved::Theme { folder: root.join("dawn"), kind: Kind::EmuSen });
    assert_eq!(themes::resolve("dawn", Some(&root)).host_folder(), root.join("dawn").to_string_lossy());
    for id in ["gone", "../dawn", ".hidden", "notes"] {
        let resolved = themes::resolve(id, Some(&root));
        assert_eq!(resolved, Resolved::Missing, "{id}");
        assert_eq!(resolved.host_folder(), "");
        assert!(resolved.words(id).contains("EmuSen's own theme is shown"), "{id}");
    }
    let book = themes::resolve("book", Some(&root));
    assert_eq!((book.host_folder().as_str(), book.words("book").is_empty()), ("", false), "an ES-DE theme is not drawn by the host");
    let listing = themes::listing(&json!({ "EmuSenTheme": "gone" }), Some(&root), REFERENCE_MANIFEST, REFERENCE_FOLDER);
    assert_eq!(listing.chosen, "gone");
    assert!(listing.rows[0].current, "the reference stands in");
    assert!(listing.words.contains("“gone”"), "{}", listing.words);
    let listing = themes::listing(&json!({ "EmuSenTheme": "dawn" }), Some(&root), REFERENCE_MANIFEST, REFERENCE_FOLDER);
    assert_eq!(listing.rows.iter().filter(|r| r.current).map(|r| r.id.as_str()).collect::<Vec<_>>(), ["dawn"]);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_package_is_checked_by_theme_check_and_an_es_de_theme_by_its_measure() {
    let root = store("check");
    // theme-audit stood in for by a script that finds nothing, as theme-check's own tests do; the static step is real.
    let audit = root.join("audit.sh");
    std::fs::write(&audit, "#!/bin/bash\nexit 0\n").unwrap();
    std::process::Command::new("chmod").arg("+x").arg(&audit).status().unwrap();
    assert_eq!(themes::check_package(&root.join("dawn"), Some(&audit)), Checked::Ok);
    match themes::check_package(Path::new(LITERAL), Some(&audit)) {
        Checked::Findings(found) => assert!(!found.is_empty() && found.iter().all(|f| f.starts_with("literal/")), "{found:?}"),
        other => panic!("a literal in a part is a finding: {other:?}"),
    }
    match themes::check_package(&root.join("dawn"), None) {
        Checked::Findings(found) => assert!(found[0].contains("theme-audit"), "{found:?}"),
        other => panic!("without the loaded step there is no clean check: {other:?}"),
    }
    let broken = root.join("broken");
    std::fs::create_dir_all(&broken).unwrap();
    std::fs::write(broken.join("capabilities.xml"), "<themeCapabilities></themeCapabilities>").unwrap();
    assert!(matches!(themes::check_esde(&broken), Checked::Findings(_)), "an ES-DE theme with no theme.xml is themed for no system");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn the_id_stored_is_the_setting_s_or_the_reference_s() {
    assert_eq!(themes::chosen_of(&json!(null)), REFERENCE);
    assert_eq!(themes::chosen_of(&json!({ "EmuSenTheme": "  " })), REFERENCE);
    assert_eq!(themes::chosen_of(&json!({ "EmuSenTheme": "dawn" })), "dawn");
    assert!(themes::valid_id("dawn") && !themes::valid_id("a/b") && !themes::valid_id("..") && !themes::valid_id(""));
}

#[test]
fn settings_opens_from_home_and_the_home_card_and_holds_the_themes_tab_under_it() {
    use emusen_qt_models::screens::{step, step_in, Ask, Shown, POWER_PLACE, SETTINGS_PLACE, THEMES};
    let act = |key: &str| Ask::Act(key.to_owned());
    assert_eq!(step(Shown::Home, &Ask::Place(SETTINGS_PLACE)), Some((Shown::Settings, None)), "Home's Settings place");
    assert_eq!(step(Shown::HomeCard, &act("settings")), Some((Shown::Settings, None)), "the Home card's Settings row");
    assert_eq!(step(Shown::Home, &Ask::Place(POWER_PLACE)).map(|s| s.0), Some(Shown::HomeCard), "Power unchanged");
    assert_eq!(step(Shown::Settings, &act(THEMES)), Some((Shown::Themes, None)), "Library ▸ Themes");
    assert_eq!(step(Shown::Settings, &Ask::Back), Some((Shown::Home, None)));
    assert_eq!(step(Shown::Settings, &act("home")), Some((Shown::Home, None)));
    assert_eq!(step(Shown::Settings, &act("cheats")), None, "a door of a later piece goes nowhere yet");
    assert_eq!(step(Shown::Themes, &Ask::Back), Some((Shown::Settings, None)), "B goes back to Settings");
    assert_eq!(step(Shown::Themes, &act("home")), Some((Shown::Home, None)));
    assert_eq!(step(Shown::Themes, &act("check")), None, "Check is the tab's own");
    assert_eq!(step(Shown::Themes, &Ask::Play), None);
    // Over an ES-DE theme the card's row opens Settings too, and leaving it returns to the theme's view.
    assert_eq!(step_in(true, Shown::HomeCard, &act("settings")), Some((Shown::Settings, None)));
    assert_eq!(step_in(true, Shown::Settings, &Ask::Back), Some((Shown::Esde, None)));
    assert_eq!(step_in(true, Shown::Themes, &Ask::Back), Some((Shown::Settings, None)));
}
