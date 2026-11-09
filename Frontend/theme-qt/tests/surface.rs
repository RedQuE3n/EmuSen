//! EmuSen.Theme's surface (EmuSen_Qt.md §3.2, §5.3 rule Q): the registry qmltyperegistrar writes is the committed one, and
//! it holds exactly the API's names, written out by hand here; the kit's names it mirrors are the kit's; the QML keeps the
//! rules; the module names no client and no runtime crate.

use emusen_qt_test::lint::{frontend, qmllint, rules};
use std::collections::BTreeSet;
use std::path::PathBuf;

const COMMITTED_TYPES: &str = include_str!("../../../Testing/qmltypes/EmuSen.Theme-1.0.qmltypes");
const COMMITTED_QMLDIR: &str = include_str!("../../../Testing/qmltypes/EmuSen.Theme-1.0.qmldir");
const KIT_TYPES: &str = include_str!("../../../Testing/qmltypes/EmuSen.Kit-1.0.qmltypes");
const MANIFEST: &str = include_str!("../Cargo.toml");

/// The registry without its line numbers and header file names, which move with every edit and mean nothing to a QML author.
fn normalized(text: &str) -> String {
    text.lines()
        .map(str::trim_end)
        .filter(|line| {
            let trimmed = line.trim_start();
            !(trimmed.starts_with("lineNumber:") || trimmed.starts_with("file:"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn built(name: &str) -> String {
    let target = std::env::var("CARGO_TARGET_DIR").map(PathBuf::from).unwrap_or_else(|_| frontend().join("target"));
    let path = target.join("cxxqt").join("qml_modules").join("EmuSen").join("Theme").join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("the build's {}: {e}", path.display()))
}

/// A component's block of a registry, found by its export.
fn component<'a>(registry: &'a str, export: &str) -> &'a str {
    let at = registry.find(&format!("\"{export} 1.0\"")).unwrap_or_else(|| panic!("{export} is exported"));
    let start = registry[..at].rfind("    Component {").unwrap();
    let end = registry[at..].find("\n    Component {").map_or(registry.len(), |e| at + e);
    &registry[start..end]
}

/// The names declared in a component under `kind` (`Property`, `Method`, `Signal`), less the change signals and setters.
fn names(block: &str, kind: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = block;
    while let Some(at) = rest.find(&format!("{kind} {{")) {
        rest = &rest[at..];
        let name = rest.split("name: \"").nth(1).and_then(|r| r.split('"').next()).unwrap_or_default().to_owned();
        if !(name.ends_with("Changed") || (kind == "Method" && name.starts_with("set") && name.len() > 3)) {
            out.insert(name);
        }
        rest = &rest[kind.len()..];
    }
    out
}

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn qt_5_3_the_committed_registry_is_the_built_one() {
    assert_eq!(
        normalized(COMMITTED_TYPES),
        normalized(&built("plugin.qmltypes")),
        "EmuSen.Theme's types changed: review it against §3.2, bump the minor for an addition, then copy plugin.qmltypes to Testing/qmltypes/EmuSen.Theme-1.0.qmltypes"
    );
    assert_eq!(
        normalized(COMMITTED_QMLDIR),
        normalized(&built("qmldir")),
        "copy the reviewed qmldir to Testing/qmltypes/EmuSen.Theme-1.0.qmldir"
    );
}

#[test]
fn qt_3_2_the_registry_holds_the_api_and_nothing_more() {
    let rows = ["data", "roleNames", "rowCount"];
    let api: [(&str, &[&str], &[&str]); 12] = [
        ("Library", &["consoles", "places", "shelves", "recent", "shelf", "favourites", "collections"], &["gamesOf", "search", "firstOf"]),
        ("Session", &["hasGame", "key", "title", "consoleId", "console", "hue", "pausedSince", "played", "frame"], &[]),
        ("Device", &["controllers", "family", "clock", "battery", "charging", "network", "bigScreen", "form"], &[]),
        (
            "Actions",
            &[],
            &[
                "play",
                "openPage",
                "favourite",
                "random",
                "openConsole",
                "openPlace",
                "home",
                "back",
                "stepSection",
                "openHomeCard",
                "openListCard",
                "search",
                "act",
                "focus",
                "enabled",
            ],
        ),
        ("Cues", &[], &["scroll", "systembrowse", "select", "back", "favorite", "launch", "quicksysselect"]),
        ("Bells", &[], &["ladder", "strike"]),
        (
            "Words",
            &[
                "title",
                "gamesHeading",
                "chooseAConsole",
                "gamesPath",
                "continuePlaying",
                "recent",
                "between",
                "search",
                "emusenTag",
                "libretroTag",
                "noGames",
                "noRecent",
                "noConsoles",
                "addFolder",
                "places",
                "continueEntry",
                "recentEntry",
            ],
            &["games", "ago", "pausedFacts", "facts", "runsOn", "statusLine", "summary", "splashCaption", "pausedAgo"],
        ),
        ("Help", &[], &["entries"]),
        (
            "Browse",
            &["section", "heading", "world", "hue", "countLine", "previous", "next", "focusRow", "searching", "typing", "listOpen", "games", "searched"],
            &[],
        ),
        ("Marks", &["heart", "sparkle", "crescent", "attention", "badge", "alien"], &["path"]),
        ("Rows", &["count"], &rows),
        ("HelpEntries", &["actionLines"], &rows),
    ];
    for (export, properties, methods) in api {
        let block = component(COMMITTED_TYPES, &format!("EmuSen.Theme/{export}"));
        assert_eq!(names(block, "Property"), set(properties), "{export}'s properties");
        assert_eq!(names(block, "Method"), set(methods), "{export}'s methods");
        let singleton = ["Rows", "HelpEntries"].contains(&export);
        assert_eq!(block.contains("isSingleton: true"), !singleton, "{export} is a singleton, a list is none");
        assert!(block.contains("isCreatable: false"), "{export}: a theme makes none");
    }
    let exports = COMMITTED_TYPES.matches("exports: [").count();
    assert_eq!(exports, api.len(), "no type beyond the API's in the registry");
    let mut files: Vec<&str> =
        COMMITTED_QMLDIR.lines().filter(|l| l.ends_with(".qml")).map(|l| l.split(' ').rev().nth(2).unwrap_or(l)).collect();
    files.sort_unstable();
    assert_eq!(
        files,
        [
            "Body",
            "Carousel",
            "CoverTile",
            "FigureLayout",
            "Frame",
            "HelpBar",
            "Moonrise",
            "NormalizedCanvas",
            "PadGlyph",
            "PlanetLayout",
            "RoundPicture",
            "Scene",
            "Sky",
            "ThemeScreen",
            "Tokens",
            "Walkable"
        ],
        "the kit's types by name, the screen and the tokens; never the whole kit"
    );
    assert!(COMMITTED_QMLDIR.contains("singleton Tokens 1.0"), "Tokens is a singleton");
    assert!(!COMMITTED_QMLDIR.lines().any(|l| l.starts_with("import ")), "the qmldir does not hand a theme the kit");
}

#[test]
fn qt_3_2_the_tokens_and_the_screen_s_enums_are_the_kit_s() {
    let tokens = std::fs::read_to_string(frontend().join("theme-qt/qml/Tokens.qml")).unwrap();
    let mirrored: Vec<&str> = tokens
        .lines()
        .filter_map(|l| l.trim().strip_prefix("readonly property "))
        .filter_map(|l| l.split(' ').nth(1))
        .map(|n| n.trim_end_matches(':'))
        .collect();
    let kit: Vec<String> = names(component(KIT_TYPES, "EmuSen.Kit/Tokens"), "Property").into_iter().collect();
    let mut sorted = mirrored.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, kit, "every token of the kit, each read-only, and no other");
    for line in tokens.lines().filter(|l| l.trim().starts_with("readonly property ")) {
        let name = line.trim().split(' ').nth(3).unwrap().trim_end_matches(':');
        assert!(line.trim_end().ends_with(&format!("Kit.Tokens.{name}")), "{name} reads the kit's own token");
    }
    let values = |registry: &str, name: &str| -> String {
        let at = registry.find(&format!("name: \"{name}\"")).unwrap_or_else(|| panic!("the kit's {name}"));
        registry[at..].split("values: [").nth(1).unwrap().split(']').next().unwrap().split_whitespace().collect::<String>().replace('"', "")
    };
    let screen = std::fs::read_to_string(frontend().join("theme-qt/qml/ThemeScreen.qml")).unwrap();
    let declared = |text: &str, name: &str| -> String {
        text.split(&format!("enum {name} {{")).nth(1).unwrap().split('}').next().unwrap().split_whitespace().collect::<String>()
    };
    assert_eq!(declared(&screen, "Walk"), values(KIT_TYPES, "Role").trim_end_matches(",NoRole"), "the walk's roles in the kit's order");
    assert_eq!(declared(&tokens, "Variant"), values(KIT_TYPES, "Variant"));
    assert_eq!(declared(&tokens, "Form"), values(KIT_TYPES, "Form"));
    let glyph = std::fs::read_to_string(frontend().join("theme-qt/qml/PadGlyph.qml")).unwrap();
    assert_eq!(declared(&glyph, "Button"), values(KIT_TYPES, "PadButton"), "the pad's buttons in the kit's order");
}

#[test]
fn qt_8_4_qml_rules_and_qmllint_pass_the_module() {
    let module = frontend().join("theme-qt").join("qml");
    let scenes = frontend().join("theme-qt").join("tests").join("qml");
    let lines = rules(&[(&module, false), (&scenes, true)]);
    assert!(lines.is_empty(), "{}", lines.join("\n"));
    let complaints = qmllint(&[&module, &scenes]);
    assert!(complaints.is_empty(), "{}", complaints.join("\n"));
}

/// The crate's own dependencies, from its manifest's `[dependencies]` table.
fn dependencies(manifest: &str) -> Vec<String> {
    manifest
        .split("\n[")
        .find(|t| t.starts_with("dependencies]"))
        .map(|t| {
            t.lines()
                .skip(1)
                .filter_map(|l| l.split_once('=').map(|(n, _)| n.trim().split('.').next().unwrap_or_default().to_owned()))
                .filter(|n| !n.starts_with('#'))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn qt_3_2_the_module_names_no_client_and_no_runtime() {
    assert_eq!(
        dependencies(MANIFEST),
        ["cxx", "cxx-qt", "cxx-qt-lib", "emusen-kit-core", "emusen-kit-qt", "emusen-qt-models", "serde_json"],
        "the module's dependencies are its design's: the models it curates, the kit, Qt"
    );
    for file in std::fs::read_dir(frontend().join("theme-qt/src")).unwrap() {
        let text = std::fs::read_to_string(file.unwrap().path()).unwrap();
        for name in ["emusen_crystal_client", "emusen_platform_client", "emusen_crystal", "emusen_platform"] {
            assert!(!text.contains(&format!("{name}::")), "the module reaches the back end only through the models it curates");
        }
    }
}
