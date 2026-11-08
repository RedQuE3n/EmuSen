// EmuSen_Qt.md §3.3, §3.4, §5.2: theme.json's schema 1, what it reads and what it refuses.

use emusen_theme_manifest::{API, Kind, Screen, TokenValue, Variant, inside_package, line_of, parse};

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
    assert_eq!((m.schema, m.api, m.name.as_str()), (1, API, "Dawnline"));
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
    assert_eq!(kinds(&GOOD.replace("\"1.0\"", "\"1.4\"")), vec![(3, Kind::Api), (16, Kind::Note)]);
    assert_eq!(kinds(&GOOD.replace("\"1.0\"", "\"one\"")), vec![(3, Kind::Schema), (16, Kind::Note)]);
    assert_eq!(kinds(&GOOD.replace("\"schema\": 1", "\"schema\": 2")), vec![(2, Kind::Api), (16, Kind::Note)]);
    let words = parse(&GOOD.replace("\"1.0\"", "\"2.0\"")).unwrap().1[0].words.clone();
    assert!(words.contains("2.0") && words.contains("1.0"), "the words name both: {words}");
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
        .replace("[\"home\", \"games\", \"splash\"]", "[\"home\", \"settings\", \"home\"]")
        .replace("\"light\": { \"accent\": \"#B4442A\" }", "\"dusk\": {}");
    let found = parse(&odd).unwrap().1;
    let words: Vec<String> = found.iter().filter(|f| f.refuses()).map(|f| format!("{}:{}", f.line, f.words)).collect();
    assert_eq!(
        words,
        vec![
            "9:screens names home twice",
            "9:settings is not a screen: home, consoles, games or splash",
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
