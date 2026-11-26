// EmuSen_Qt.md §3.3, §3.4, §5.2: theme.json's schema 1, what it reads and what it refuses.

use emusen_theme_manifest::{API, DesktopScreen, Kind, SCHEMA, Screen, TokenValue, Variant, inside_package, line_of, parse};

const GOOD: &str = r##"{
    "schema": 1,
    "api": "1.0",
    "name": "Dawnline",
    "author": "A. Tester",
    "licence": "CC-BY-4.0",
    "variants": ["dark", "light"],
    "fontSizes": [1, 1.15, 1.3],
    "screens": ["home", "games", "splash"],
    "tokens": {
        "dark": { "accent": "#4C9AE8", "tileRadius": 8 },
        "light": { "accent": "#B4442A" }
    },
    "cues": { "scroll": "assets/scroll.ogg" },
    "previews": ["assets/home.png"],
    "homepage": "unread"
}
"##;

fn kinds(text: &str) -> Vec<(usize, Kind)> {
    parse(text).unwrap().1.into_iter().map(|f| (f.line, f.kind)).collect()
}

#[test]
fn qt_3_4_a_whole_manifest_reads_with_its_lines_and_an_unknown_key_is_a_note() {
    let (m, findings) = parse(GOOD).unwrap();
    assert_eq!((m.schema, m.api, m.name.as_str()), (1, (1, 0), "Dawnline"), "a package written for 1.0 reads on a newer minor");
    assert_eq!(m.variants, vec![Variant::Dark, Variant::Light]);
    assert_eq!(m.screens, vec![Screen::Home, Screen::Games, Screen::Splash]);
    assert_eq!(m.font_sizes, vec![1.0, 1.15, 1.3]);
    assert_eq!(m.tokens.len(), 3);
    assert_eq!(
        (m.tokens[0].variant, m.tokens[0].name.as_str(), &m.tokens[0].value, m.tokens[0].line),
        (Variant::Dark, "accent", &TokenValue::Text("#4C9AE8".into()), 11)
    );
    assert_eq!((m.tokens[2].variant, m.tokens[2].line), (Variant::Light, 12));
    assert_eq!((m.cues[0].0.as_str(), m.cues[0].1.path.as_str(), m.cues[0].1.line), ("scroll", "assets/scroll.ogg", 14));
    assert_eq!(m.previews[0].path, "assets/home.png");
    assert_eq!(findings.len(), 1);
    assert_eq!((findings[0].line, findings[0].kind, findings[0].refuses()), (16, Kind::Note, false));
    assert_eq!(Screen::Splash.file(), "screens/Splash.qml");
}

#[test]
fn qt_3_3_a_newer_api_or_schema_is_refused_and_an_older_minor_is_not_possible_below_1_0() {
    assert_eq!(kinds(&GOOD.replace("\"1.0\"", "\"2.0\"")), vec![(3, Kind::Api), (16, Kind::Note)]);
    // A minor past the build's own, read from the build rather than typed, so the next minor's bump keeps the case.
    assert_eq!(kinds(&GOOD.replace("\"1.0\"", &format!("\"{}.{}\"", API.0, API.1 + 1))), vec![(3, Kind::Api), (16, Kind::Note)]);
    assert_eq!(kinds(&GOOD.replace("\"1.0\"", "\"one\"")), vec![(3, Kind::Schema), (16, Kind::Note)]);
    assert_eq!(kinds(&GOOD.replace("\"schema\": 1", &format!("\"schema\": {}", SCHEMA + 1))), vec![(2, Kind::Api), (16, Kind::Note)]);
    assert_eq!(kinds(&GOOD.replace("\"schema\": 1", "\"schema\": 0")), vec![(2, Kind::Schema), (16, Kind::Note)]);
    let words = parse(&GOOD.replace("\"1.0\"", "\"2.0\"")).unwrap().1[0].words.clone();
    assert!(words.contains("2.0") && words.contains(&format!("{}.{}", API.0, API.1)), "the words name both: {words}");
}

#[test]
fn qt_3_4_missing_keys_wrong_shapes_and_unknown_names_are_schema_findings() {
    let bare = "{\n  \"schema\": 1\n}\n";
    let found = parse(bare).unwrap().1;
    let words: Vec<&str> = found.iter().map(|f| f.words.as_str()).collect();
    assert_eq!(
        words,
        vec!["api is missing", "name is missing", "author is missing", "licence is missing", "variants is missing", "screens is missing"]
    );
    let odd = GOOD
        .replace("[\"home\", \"games\", \"splash\"]", "[\"home\", \"pause\", \"home\"]")
        .replace("\"light\": { \"accent\": \"#B4442A\" }", "\"dusk\": {}");
    let found = parse(&odd).unwrap().1;
    let words: Vec<String> = found.iter().filter(|f| f.refuses()).map(|f| format!("{}:{}", f.line, f.words)).collect();
    assert_eq!(
        words,
        vec![
            "9:screens names home twice",
            "9:pause is not a screen: home, consoles, games, splash, settings or game",
            "12:dusk is not a variant: dark or light"
        ]
    );
    assert!(parse("[1]").is_err());
    assert_eq!(parse("{\n  \"schema\": 1,\n  oops\n}").unwrap_err().line, 3);
}

#[test]
fn qt_3_5_a_path_that_leaves_the_package_is_refused() {
    for good in ["assets/a.ogg", "a.png", "assets/sub/b.png"] {
        assert!(inside_package(good), "{good}");
    }
    for bad in ["../a.ogg", "/etc/passwd", "http://x/a.png", "qrc:/a", "assets/../../a", "a\\b", "", "assets//a"] {
        assert!(!inside_package(bad), "{bad}");
    }
    let found = parse(&GOOD.replace("assets/home.png", "../home.png")).unwrap().1;
    assert_eq!((found[0].line, found[0].kind), (15, Kind::Path));
}

#[test]
fn qt_3_4_a_key_is_found_after_its_parent_and_never_as_a_value() {
    let text = "{\n \"name\": \"accent\",\n \"tokens\": {\n  \"dark\": {\n   \"accent\": \"#000\"\n  }\n }\n}";
    assert_eq!(line_of(text, &["tokens", "dark", "accent"]), 5);
    assert_eq!(line_of(text, &["nothing"]), 1);
}

const GOOD_V2: &str = r##"{
    "schema": 2,
    "api": "1.22",
    "name": "Eclipse",
    "author": "The EmuSen project",
    "licence": "GPL-3.0-or-later",
    "variants": ["dark", "light"],
    "faces": {
        "bigPicture": ["home", "consoles", "games", "splash", "settings", "game"],
        "desktop": ["library", "table", "playing", "settings"]
    },
    "tokens": {
        "dark": {
            "surface": "#07080F",
            "rowHeight": 56.0,
            "desktop": {
                "rowHeight": 38.0,
                "fontRow": 13.5
            }
        },
        "light": {
            "accent": "#B4442A",
            "desktop": {
                "tileRadius": 12.0
            }
        }
    },
    "extra": true
}
"##;

#[test]
fn schema_2_reads_faces_desktop_screens_and_desktop_tokens() {
    let (m, findings) = parse(GOOD_V2).unwrap();
    assert_eq!((m.schema, m.api, m.name.as_str()), (2, (1, 22), "Eclipse"));
    assert_eq!(m.screens, Screen::ALL.to_vec());
    assert_eq!(m.desktop_screens, DesktopScreen::ALL.to_vec());
    assert_eq!(DesktopScreen::Library.file(), "screens/desktop/Library.qml");
    assert_eq!(DesktopScreen::Table.file(), "screens/desktop/Table.qml");
    assert_eq!(DesktopScreen::Playing.file(), "screens/desktop/Playing.qml");
    assert_eq!(DesktopScreen::Settings.file(), "screens/desktop/Settings.qml");
    assert_eq!(DesktopScreen::listed(), "library, table, playing or settings");
    assert_eq!(m.tokens.len(), 3);
    assert_eq!(m.desktop_tokens.len(), 3);
    assert_eq!((m.desktop_tokens[0].variant, m.desktop_tokens[0].name.as_str(), &m.desktop_tokens[0].value, m.desktop_tokens[0].line), (Variant::Dark, "fontRow", &TokenValue::Number(13.5), 18));
    assert_eq!((m.desktop_tokens[1].variant, m.desktop_tokens[1].name.as_str(), &m.desktop_tokens[1].value, m.desktop_tokens[1].line), (Variant::Dark, "rowHeight", &TokenValue::Number(38.0), 17));
    assert_eq!((m.desktop_tokens[2].variant, m.desktop_tokens[2].name.as_str(), m.desktop_tokens[2].line), (Variant::Light, "tileRadius", 24));
    assert_eq!(findings.len(), 1);
    assert_eq!((findings[0].line, findings[0].kind, findings[0].refuses()), (28, Kind::Note, false));
    assert!(findings[0].words.contains("schema 2"), "{}", findings[0].words);
    assert_eq!(kinds(&GOOD_V2.replace("\"schema\": 2", "\"schema\": 3")), vec![(2, Kind::Api), (28, Kind::Note)]);
}

#[test]
fn schema_1_and_schema_2_reject_mixed_or_invalid_faces_and_desktop_tokens() {
    let v1_with_faces = "{\n  \"schema\": 1,\n  \"api\": \"1.22\",\n  \"name\": \"T\",\n  \"author\": \"A\",\n  \"licence\": \"L\",\n  \"variants\": [\"dark\"],\n  \"screens\": [\"home\"],\n  \"faces\": { \"desktop\": [\"library\"] },\n  \"tokens\": { \"dark\": { \"desktop\": { \"rowHeight\": 38.0 } } }\n}\n";
    let f1: Vec<_> = parse(v1_with_faces).unwrap().1.into_iter().filter(|f| f.refuses()).map(|f| (f.line, f.kind, f.words)).collect();
    assert_eq!(f1, vec![(9, Kind::Schema, "faces requires schema 2".to_owned()), (10, Kind::Schema, "desktop token overrides require schema 2".to_owned())]);

    let v2_bare = "{\n  \"schema\": 2\n}\n";
    let words: Vec<String> = parse(v2_bare).unwrap().1.into_iter().map(|f| f.words).collect();
    assert_eq!(words, vec!["api is missing", "name is missing", "author is missing", "licence is missing", "variants is missing", "faces is missing"]);

    let v2_bad = "{\n  \"schema\": 2,\n  \"api\": \"1.22\",\n  \"name\": \"T\",\n  \"author\": \"A\",\n  \"licence\": \"L\",\n  \"variants\": [\"dark\"],\n  \"screens\": [\"home\"],\n  \"faces\": {\n    \"bigPicture\": [\"library\"],\n    \"desktop\": [\"home\", \"library\", \"library\"],\n    \"mobile\": []\n  },\n  \"tokens\": { \"dark\": { \"desktop\": { \"bad\": null } } }\n}\n";
    let f2: Vec<_> = parse(v2_bad).unwrap().1.into_iter().filter(|f| f.refuses()).map(|f| (f.line, f.kind, f.words)).collect();
    assert!(f2.iter().any(|(l, k, w)| *l == 8 && *k == Kind::Schema && w.contains("screens")), "{f2:?}");
    assert!(f2.iter().any(|(l, k, w)| *l == 10 && *k == Kind::Schema && w.contains("is not a screen")), "{f2:?}");
    assert!(f2.iter().any(|(l, k, w)| *l == 11 && *k == Kind::Schema && w.contains("is not a desktop screen")), "{f2:?}");
    assert!(f2.iter().any(|(l, k, w)| *l == 11 && *k == Kind::Schema && w.contains("twice")), "{f2:?}");
    assert!(f2.iter().any(|(l, k, w)| *l == 12 && *k == Kind::Schema && w.contains("is not a face")), "{f2:?}");
    assert!(f2.iter().any(|(l, k, w)| *l == 14 && *k == Kind::Schema && w.contains("bad is a value")), "{f2:?}");

    let v2_desktop_non_obj = GOOD_V2.replace("\"desktop\": {\n                \"rowHeight\": 38.0,\n                \"fontRow\": 13.5\n            }", "\"desktop\": 42");
    assert!(parse(&v2_desktop_non_obj).unwrap().1.iter().any(|f| f.refuses() && f.kind == Kind::Schema && f.words.contains("tokens.dark.desktop")));
}
