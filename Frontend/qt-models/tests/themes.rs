//! The Themes tab's facts (EmuSen_Qt.md §3.8) over a scratch themes store holding one theme of each kind: the rows, the id
//! resolved, the check, when Use is offered, and the ES-DE theme in use with its choices as they are stored.

use emusen_qt_models::esde_shelves::{settings_of, theme_key};
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
fn use_is_offered_only_for_a_theme_checked_clean_and_not_in_use() {
    let root = store("usable");
    let mut rows = themes::listing(&json!({}), Some(&root), REFERENCE_MANIFEST, REFERENCE_FOLDER).rows;
    assert!(!rows[0].usable(), "the reference is in use");
    assert!(!rows[2].usable(), "a package not yet checked");
    rows[2].checked = Checked::Findings(vec!["dawn/screens/Home.qml:3 literal".into()]);
    assert!(!rows[2].usable(), "a package with findings has no Use");
    assert_eq!(rows[2].value(), "1 finding");
    rows[2].checked = Checked::Ok;
    assert!(rows[2].usable());
    assert!(!rows[1].usable(), "an ES-DE theme not yet checked");
    rows[1].checked = Checked::Ok;
    assert!(rows[1].usable(), "an ES-DE theme checked clean is chosen like a package");
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
    let ran = root.join("audit-ran");
    std::fs::write(&audit, format!("#!/bin/bash\ntouch \"{}\"\nexit 0\n", ran.display())).unwrap();
    assert!(std::process::Command::new("chmod").arg("+x").arg(&audit).status().unwrap().success(), "the stand-in made runnable");
    assert_eq!(themes::check_package(&root.join("dawn"), Some(&audit)), Checked::Ok);
    assert!(ran.exists(), "ok only once the loaded step ran");
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
    assert_eq!(step(Shown::Settings, &act("shell")), None, "a door of a later piece goes nowhere yet");
    assert_eq!(step(Shown::Themes, &Ask::Back), Some((Shown::Settings, None)), "B goes back to Settings");
    assert_eq!(step(Shown::Themes, &act("home")), Some((Shown::Home, None)));
    assert_eq!(step(Shown::Themes, &act("check")), None, "Check is the tab's own");
    assert_eq!(step(Shown::Themes, &Ask::Play), None);
    // Over an ES-DE theme the card's row opens Settings too, and leaving it returns to the theme's view.
    assert_eq!(step_in(true, Shown::HomeCard, &act("settings")), Some((Shown::Settings, None)));
    assert_eq!(step_in(true, Shown::Settings, &Ask::Back), Some((Shown::Esde, None)));
    assert_eq!(step_in(true, Shown::Themes, &Ask::Back), Some((Shown::Settings, None)));
}

/// `book`'s capabilities in full: two variants and a hidden one, two colour schemes, three font sizes, two languages, two
/// aspect ratios, a profile of its own and the slide suppressed.
const BOOK_CAPABILITIES: &str = "<themeCapabilities><themeName>Picture Book</themeName>\
    <variant name=\"one\"><label>One</label><selectable>true</selectable></variant><variant name=\"two\"><selectable>true</selectable></variant>\
    <variant name=\"hidden\"></variant>\
    <colorScheme name=\"dark\"><label>Dark</label></colorScheme><colorScheme name=\"light\"><label>Light</label></colorScheme>\
    <fontSize>large</fontSize><fontSize>medium</fontSize><fontSize>small</fontSize>\
    <language>en_US</language><language>de_DE</language>\
    <aspectRatio>16:9</aspectRatio><aspectRatio>4:3</aspectRatio>\
    <transitions name=\"calm\"><label>Calm</label><selectable>true</selectable><systemToSystem>fade</systemToSystem></transitions>\
    <suppressTransitionProfiles><entry>builtin-slide</entry></suppressTransitionProfiles></themeCapabilities>";

#[test]
fn every_choice_the_capabilities_declare_is_listed_at_what_is_stored_or_where_the_loader_falls_back() {
    let root = store("choices");
    std::fs::write(root.join("book/capabilities.xml"), BOOK_CAPABILITIES).unwrap();
    let book = root.join("book");
    let chosen = |stored: serde_json::Value| {
        let mut settings = json!({});
        themes::choose_esde(&mut settings, &book);
        settings["BigPicture"] = json!({ theme_key(book.to_str().unwrap()): stored });
        let mut rows = themes::listing(&json!({}), Some(&root), REFERENCE_MANIFEST, REFERENCE_FOLDER).rows;
        let in_use = themes::mark(&mut rows, &settings, Some(&root)).expect("book is in use");
        (rows, in_use)
    };
    let (rows, in_use) = chosen(json!({}));
    assert_eq!(rows.iter().filter(|r| r.current).map(|r| r.id.as_str()).collect::<Vec<_>>(), ["book"], "no EmuSen theme is in use beside it");
    assert_eq!(in_use.root, book);
    let listed: Vec<(&str, &str, Vec<&str>)> =
        in_use.choices.iter().map(|c| (c.key, c.value(), c.options.iter().map(|o| o.0.as_str()).collect())).collect();
    assert_eq!(
        listed,
        [
            ("Variant", "One", vec!["one", "two"]),
            ("ColorScheme", "Dark", vec!["dark", "light"]),
            ("FontSize", "medium", vec!["medium", "large", "small"]),
            ("Language", "en_US", vec!["en_US", "de_DE"]),
            ("AspectRatio", "Automatic", vec!["automatic", "16:9", "4:3"]),
            ("Transitions", "Automatic", vec!["automatic", "calm", "builtin-instant", "builtin-fade"]),
        ],
        "a hidden variant and a suppressed profile are not offered; nothing stored is the loader's fallback"
    );
    let (_, in_use) = chosen(json!({ "Variant": "two", "colorScheme": "light", "FontSize": "small", "AspectRatio": "4:3", "Transitions": "gone" }));
    let values: Vec<&str> = in_use.choices.iter().map(|c| c.value()).collect();
    assert_eq!(values, ["two", "Light", "small", "en_US", "4:3", "Automatic"], "stored values, either key form; one not offered falls back");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_choice_steps_round_and_is_stored_under_the_theme_s_folder_as_the_view_reads_it() {
    let root = store("store");
    std::fs::write(root.join("book/capabilities.xml"), BOOK_CAPABILITIES).unwrap();
    let book = root.join("book");
    let mut settings = json!({ "EmuSenTheme": "dawn", "BigPicture": "not a table" });
    themes::choose_esde(&mut settings, &book);
    assert!(settings_of(&settings).themed, "the ES-DE theme is the library's look");
    let mut rows = themes::listing(&json!({}), Some(&root), REFERENCE_MANIFEST, REFERENCE_FOLDER).rows;
    let mut choices = themes::mark(&mut rows, &settings, Some(&root)).unwrap().choices;
    assert_eq!(choices[0].step(-1), "two", "Left from the first goes round to the last");
    assert_eq!(choices[0].step(1), "one");
    assert_eq!(choices[5].step(2), "builtin-instant");
    themes::store_choice(&mut settings, &book, choices[0].key, "two");
    themes::store_choice(&mut settings, &book, "ColorScheme", "light");
    let view: serde_json::Value = serde_json::from_str(&settings_of(&settings).text).unwrap();
    assert_eq!(view["theme"], book.to_str().unwrap());
    assert_eq!(view["choices"], json!({ "Variant": "two", "ColorScheme": "light" }), "the view's settings carry the choices stored");
    // Back to an EmuSen package: its id stored, the ES-DE theme's folder and choices kept for a return.
    themes::choose_own(&mut settings, "dawn");
    assert!(!settings_of(&settings).themed);
    assert_eq!(settings["LibraryStyle"], "Mistress");
    themes::mark(&mut rows, &settings, Some(&root));
    assert_eq!(rows.iter().filter(|r| r.current).map(|r| r.id.as_str()).collect::<Vec<_>>(), ["dawn"]);
    themes::choose_esde(&mut settings, &book);
    assert_eq!(themes::mark(&mut rows, &settings, Some(&root)).unwrap().choices[0].value(), "two", "the choices kept");
    std::fs::remove_dir_all(root).unwrap();
}
