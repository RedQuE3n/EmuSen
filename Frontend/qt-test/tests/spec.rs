// EmuSen_Kit.md §7.11: the spec writer records a Qt state in the `spec 1` format, its facts decided as kit-core's audit decides them.

use emusen_kit_core::input::WalkForm;
use emusen_kit_core::tokens::Variant;
use emusen_qt_test::harness::{edges, palette, Measures};
use emusen_qt_test::spec::{self, Header};
use emusen_qt_test::Harness;

/// The lines of the tree whose facts carry this pair.
fn lines_with<'a>(text: &'a str, fact: &str) -> Vec<&'a str> {
    text.lines().filter(|l| l.starts_with("  ") && l.split(' ').any(|t| t == fact)).collect()
}

/// A line's tokens, split on spaces outside quotes.
fn tokens(line: &str) -> Vec<String> {
    let (mut out, mut current, mut quoted, mut escaped) = (Vec::new(), String::new(), false, false);
    for c in line.chars() {
        match c {
            _ if escaped => escaped = false,
            '\\' if quoted => escaped = true,
            '"' => quoted = !quoted,
            ' ' if !quoted => {
                out.push(std::mem::take(&mut current));
                continue;
            }
            _ => {}
        }
        current.push(c);
    }
    out.push(current);
    out
}

fn id_of(line: &str) -> &str {
    line.trim_start().split(' ').next().unwrap_or_default()
}

#[test]
fn qt_kit_7_11_a_state_is_recorded_in_the_spec_format() {
    assert_eq!(spec::num(-0.001), "0");
    assert_eq!(spec::num(12.5), "12.5");
    assert_eq!(spec::num(3.0), "3");
    assert_eq!(spec::num(0.126), "0.13");
    assert_eq!(spec::quote("Display"), "Display");
    assert_eq!(spec::quote("4:3 CRT"), "\"4:3 CRT\"");
    assert_eq!(spec::quote("a=b"), "\"a=b\"");
    assert_eq!(spec::quote(""), "\"\"");
    assert_eq!(spec::quote("\"x\"\n"), "\"\\\"x\\\"\\n\"");
    assert_eq!(spec::quote("·"), "\"·\"");

    let mut harness = Harness::new();
    let mut scene = harness.load("qrc:/qt/qml/EmuSen/Scenes/qml/MenuScene.qml", 1280, 800);
    let tree = scene.tree();
    let rows = tree.find_type("MenuRow");
    scene.focus_item(&tree, rows[0]);
    let tree = scene.tree();
    let focus = scene.focus(&tree);
    let walk = scene.walk(&tree);
    let declared = palette(Variant::Dark);
    let header = Header {
        state: "MenuScene-1280x800",
        case: "MenuScene",
        kind: "menu",
        width: 1280,
        height: 800,
        measures: Measures::of(WalkForm::BigScreen, 800.0),
        palette: &declared,
    };
    let text = spec::write(&tree, &tree.root, &header, focus, &walk);
    let out = std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cache/emusen/tmp/qt-test");
    std::fs::create_dir_all(&out).expect("the output folder");
    std::fs::write(out.join("MenuScene-1280x800.spec"), &text).expect("the recording written");

    let head: Vec<&str> = text.lines().take(13).collect();
    assert_eq!(head[0], spec::BANNER);
    let fixed = [
        "spec 1",
        "state MenuScene-1280x800",
        "size 1280 800",
        "kind menu",
        "root RootItem",
        "unit 1",
        "slack 1",
        "floor 16",
        "frame 0,0,1280,800",
        "case MenuScene",
    ];
    assert_eq!(&head[1..11], &fixed);
    assert!(head[11].starts_with("palette #"), "{}", head[11]);
    let mut unique = declared.clone();
    unique.sort_by_key(|c| (c.r, c.g, c.b, c.a));
    unique.dedup();
    assert_eq!(head[11].split(' ').count(), 1 + unique.len(), "the variant's colours, each once: {}", head[11]);
    assert_eq!(head[12], "tree");

    // The ids run in order from 0 and each line is indented a level under its parent or less.
    let tree_lines: Vec<&str> = text.lines().skip(13).take_while(|l| l.starts_with("  ")).collect();
    let mut depth = 0;
    for (i, line) in tree_lines.iter().enumerate() {
        assert_eq!(id_of(line), i.to_string(), "line {i}: {line}");
        let level = (line.len() - line.trim_start().len()) / 2;
        assert!(level >= 1 && level <= depth + 1, "line {i} jumps a level: {line}");
        depth = level;
    }

    // Each line's facts in the format's order, by key, as the C# writer writes them back.
    for line in &tree_lines {
        let keys: Vec<String> =
            tokens(line.trim_start()).into_iter().skip(2).map(|t| t.split('=').next().unwrap_or_default().to_owned()).collect();
        assert!(keys.windows(2).all(|k| k[0] < k[1]), "facts out of order: {line}");
    }

    // The panel by its name, its rows operable and named, words owned by their row and measured in design pixels.
    let panel = lines_with(&text, "name=panel");
    assert_eq!(panel.len(), 1, "the panel: {panel:?}");
    let rows_written: Vec<&str> = text.lines().filter(|l| l.trim_start().split(' ').nth(1) == Some("MenuRow")).collect();
    assert!(rows_written.len() >= 5, "{} rows recorded", rows_written.len());
    for row in &rows_written {
        assert!(row.contains(" op") || row.contains(" unseen"), "a row is operable: {row}");
        assert!(row.contains(" heard="), "a row has a name a reader hears: {row}");
    }
    let display = tree_lines.iter().find(|l| l.contains(" text=Display")).expect("the first row's words");
    assert!(
        display.contains(" drawn") && display.contains(" caps=") && display.contains(" face=") && display.contains(" by="),
        "{display}"
    );
    let owner = display.split(' ').find_map(|t| t.strip_prefix("owner=")).expect("the row owns its words");
    assert_eq!(owner, id_of(rows_written[0]), "the first row owns its words: {display}");

    // The focus on the first row, and the walk's every press: Down from the first row reaches the second.
    let focus_line = text.lines().find(|l| l.starts_with("focus ")).expect("the focus line");
    assert_eq!(focus_line, format!("focus {}", id_of(rows_written[0])));
    let down = format!("edge {} down {}", id_of(rows_written[0]), id_of(rows_written[1]));
    assert!(
        text.lines().any(|l| l == down),
        "{down} in:\n{}",
        text.lines().filter(|l| l.starts_with("edge")).collect::<Vec<_>>().join("\n")
    );
    let stays = format!("edge {} up same", id_of(rows_written[0]));
    assert!(text.lines().any(|l| l == stays), "Up from the first row keeps the focus where it is: {stays}");
    let recorded = text.lines().filter(|l| l.starts_with("edge ")).count();
    assert_eq!(recorded, walk.len(), "every press of the walk is an edge line");
    assert!(edges(&walk).len() >= 8, "the moves among them");
    assert!(text.ends_with('\n'));
}
