//! The oracle of Eclipse's Settings on Sound, Controllers, Collections, Saves and states, Library, Scraping and EmuSen, each at its board's state, by pad and pointer (EmuSen_Qt.md §3.4.11).
#![allow(dead_code)]

use crate::common::wait_for;
use crate::eclipse_board::{boxes, held, read, rows, same, Row};
use crate::eclipse_pad::{focused, press};
use crate::eclipse_settings_oracle::{acted, at, centre, help_line, lines_held, pictures, settings, settle, shown, state, with_settings};
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::input::PadControl;
use emusen_kit_core::settings::RowKind;
use emusen_kit_core::tokens::Colour;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::virtual_pad::{Sdl, VirtualPad};
use emusen_qt_test::Scene;

/// A page's board: its table, where it stands, the rows it draws and the words of its About panel.
pub struct Page {
    /// The page's name in pictures.
    pub name: &'static str,
    pub table: &'static str,
    /// The board's lines: each row's words' line box and baseline, by the program's row.
    pub lines: &'static str,
    /// The section's key, and the row of it that opens the page where the page is one under it.
    pub section: &'static str,
    pub opens: Option<&'static str>,
    /// The page's key, as Settings.page gives it.
    pub key: &'static str,
    pub heading: &'static str,
    pub crumb: &'static str,
    /// The row the board focuses, and how many left-hand rows its window draws there.
    pub focus: usize,
    pub drawn: usize,
    /// The rows the board draws: program row index, board name, board short line, and whether the board cuts it (None when the value is a test-world path of variable length).
    pub board: &'static [(usize, &'static str, &'static str, Option<bool>)],
    /// Program-only rows the board does not draw (R-20261116-21): index and name.
    pub program_only: &'static [(usize, &'static str)],
    /// Rows the board draws that an earlier ruling hides, by name.
    pub hidden: &'static [&'static str],
    pub glow: &'static str,
    /// The About panel's heading, literal sentence (empty when hidden), and small lines as the board's state gives them.
    pub about: &'static str,
    pub sentence: &'static str,
    pub muted: &'static [&'static str],
    /// The help line's words in order.
    pub help: &'static [&'static str],
}

/// The program row index named by a row element (`row.N`, `rowFill.N`, `rowName.N`, etc.).
fn row_index(name: &str) -> Option<usize> {
    let (head, tail) = name.rsplit_once('.')?;
    head.starts_with("row").then(|| tail.parse::<usize>().ok()).flatten()
}

/// The lines table's rows of the left rows `drawn` holds, and every non-row element.
fn lines_of(lines: &str, drawn: impl Fn(usize) -> bool) -> String {
    lines
        .lines()
        .filter(|l| l.starts_with('#') || l.starts_with("element\t") || l.split('\t').next().and_then(row_index).is_none_or(&drawn))
        .map(|l| format!("{l}\n"))
        .collect()
}

/// The model's row ids, in order.
fn ids(scene: &mut Scene<'_>) -> Vec<String> {
    settings(scene).shown_rows().iter().map(|r| r.id.clone()).collect()
}

/// The pad to the page and its board's focused row, from the first section's first row.
fn reach(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, page: &Page) {
    for _ in 0..6 {
        if at(scene).1 == page.section {
            break;
        }
        press(scene, sdl, pad, PadControl::RightShoulder);
        settle(scene);
    }
    assert_eq!(at(scene).1, page.section, "RB reaches {}", page.section);
    if let Some(row) = page.opens {
        let index = ids(scene).iter().position(|i| i == row).unwrap_or_else(|| panic!("{} has the row {row}", page.section));
        for _ in 0..index {
            press(scene, sdl, pad, PadControl::Down);
        }
        settle(scene);
        assert_eq!(at(scene).0, index as i32, "Down to {row}");
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
    }
    assert_eq!(at(scene), (0, page.key.to_owned()), "{} is shown on its first row", page.key);
    for _ in 0..page.focus {
        press(scene, sdl, pad, PadControl::Down);
    }
    settle(scene);
    assert_eq!(at(scene).0, page.focus as i32, "Down to the board's row");
}

/// Each drawn row's short line ends 24 px before its value at the latest, cut there and only there (G:30, R-20261123-31); a board row cut as the board cuts it.
fn lines_cut(scene: &mut Scene<'_>, page: &Page, left_count: usize, k: f64, at_size: &str, differences: &mut Vec<String>) {
    let tree = scene.tree();
    let unit: f64 = read(&tree, "settingsCanvas", "unit").parse().expect("the canvas's unit");
    let got = boxes(&tree, unit);
    for i in 0..left_count {
        if !shown(scene, &format!("row.{i}")) || !shown(scene, &format!("rowAbout.{i}")) {
            continue;
        }
        let (line, value) = (got[&format!("rowAbout.{i}")], got[&format!("rowValue.{i}")]);
        let cut = read(&tree, &format!("rowAbout.{i}"), "truncated") == "true";
        if line.right() > value.x - 24.0 * k + 1e-6 || (cut && line.right() < value.x - 24.0 * k - 1e-6) {
            differences.push(format!("{at_size}: row {i}'s short line ends at {} with the value at {}, cut {cut}", line.right(), value.x));
        }
        if let Some((_, _, _, Some(board_cut))) = page.board.iter().find(|b| b.0 == i) {
            if cut != *board_cut {
                differences.push(format!("{at_size}: row {i}'s short line cut {cut} where the board's is cut {board_cut}"));
            }
        }
    }
}

/// The page held to its board at every gate size, its words, its rows by pad and pointer, X, and B.
pub fn oracle(page: &Page, backend: &str, extra: &dyn Fn(&mut Scene<'_>, &Sdl, &VirtualPad, &mut Vec<String>)) {
    with_settings(|scene, sdl, pad| run(scene, sdl, pad, page, backend, extra));
}

fn run(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, page: &Page, backend: &str, extra: &dyn Fn(&mut Scene<'_>, &Sdl, &VirtualPad, &mut Vec<String>)) {
    assert!(scene.set_property("folders", "catching", "true"));
    reach(scene, sdl, pad, page);
    let count = settings(scene).shown_rows().len();
    let left_count = settings(scene).view().chip_start().unwrap_or(count);
    assert_eq!(page.board.len() + page.program_only.len(), left_count, "{}: every left row is accounted for by board or program_only", page.name);
    let mut differences = Vec::new();
    for name in page.hidden {
        if settings(scene).shown_rows().iter().any(|r| r.label == *name) {
            differences.push(format!("{name} is listed, which an earlier ruling hides until it is built"));
        }
    }
    let (board_height, table) = rows(page.table);
    let initial: Vec<Row> = table.iter().filter(|r| row_index(&r.name).is_none_or(|i| i < page.drawn)).cloned().collect();
    let k = DESIGN_HEIGHT / board_height;
    for (width, height) in SIZES {
        scene.resize(width, height);
        scene.wait(300);
        let tree = scene.tree();
        let at_size = format!("{width}x{height}");
        let unit: f64 = read(&tree, "settingsCanvas", "unit").parse().expect("the canvas's unit");
        let screen: u64 = tree.nodes[tree.find_named("settingsScreen").expect("Eclipse's Settings") as usize].id.parse().unwrap();
        let tokens = crate::eclipse_board::tokens(screen);
        held(&tree, &initial, k, unit, f64::from(width) / unit, &tokens, &at_size, &|_, _, _| false, &mut differences);
        lines_held(&tree, &lines_of(page.lines, |i| i < page.drawn), k, &at_size, &mut differences);
        if Colour::parse(&read(&tree, "backdropField.0", "paint")) != Colour::parse(page.glow) {
            differences.push(format!("{at_size}: the page's glow is {} where the board gives {}", read(&tree, "backdropField.0", "paint"), page.glow));
        }
        // Every board row's name, short line and About panel sentence held against the board's literal words, not the model's own.
        let mut words = vec![
            ("settingsTitle".to_owned(), "Settings".to_owned()),
            ("settingsHeading".to_owned(), page.heading.to_owned()),
            ("settingsPlace".to_owned(), format!("{} of {count}", page.focus + 1)),
            ("asideHeading".to_owned(), page.about.to_owned()),
        ];
        if !page.sentence.is_empty() {
            words.push(("asideSentence".to_owned(), page.sentence.to_owned()));
        } else if shown(scene, "asideSentence") {
            differences.push(format!("{at_size}: asideSentence is drawn where the board has none"));
        }
        if !page.crumb.is_empty() {
            words.push(("settingsCrumb".to_owned(), page.crumb.to_owned()));
        } else if shown(scene, "settingsCrumb") {
            differences.push(format!("{at_size}: a trail is drawn over a section's heading"));
        }
        for (i, line) in page.muted.iter().enumerate() {
            words.push((format!("asideMuted.{i}"), line.to_string()));
        }
        for (i, name, about, _) in page.board {
            words.push((format!("rowName.{i}"), name.to_string()));
            words.push((format!("rowAbout.{i}"), about.to_string()));
        }
        for (i, name) in page.program_only {
            words.push((format!("rowName.{i}"), name.to_string()));
            if read(&tree, &format!("rowAbout.{i}"), "text").is_empty() {
                differences.push(format!("{at_size}: program-only row {i} ({name}) has no short line"));
            }
        }
        for (name, want) in words {
            let have = read(&tree, &name, "text");
            if have != want {
                differences.push(format!("{at_size}: {name} says {have:?} where {want:?}"));
            }
        }
        for i in 0..left_count {
            if shown(scene, &format!("row.{i}")) != (i < page.drawn) {
                differences.push(format!("{at_size}: row {i} drawn {}", i >= page.drawn));
            }
        }
        lines_cut(scene, page, left_count, k, &at_size, &mut differences);
    }
    scene.resize(1920, 1200);
    scene.wait(300);
    let defaultable = settings(scene).view().defaultable(page.focus);
    let line: Vec<String> = page.help.iter().map(|w| w.to_string()).collect();
    if help_line(scene) != (line, defaultable.to_string()) {
        differences.push(format!("the help line is {:?} where the board's is {:?} (defaultable {defaultable})", help_line(scene), page.help));
    }
    pictures(scene, backend, page.name);
    extra(scene, sdl, pad, &mut differences);

    // The pad: Up to the first row, Down through every left row to the last, each focused, drawn and counted, then past the last.
    for _ in 0..left_count {
        press(scene, sdl, pad, PadControl::Up);
    }
    for i in 0..left_count {
        if i > 0 {
            press(scene, sdl, pad, PadControl::Down);
        }
        settle(scene);
        if at(scene).0 != i as i32 || focused(scene) != format!("row.{i}") || !shown(scene, &format!("row.{i}")) {
            differences.push(format!("Down to row {i}: the model at {:?}, the focus on {:?}, drawn {}", at(scene), focused(scene), shown(scene, &format!("row.{i}"))));
        }
        if read(&scene.tree(), "settingsPlace", "text") != format!("{} of {count}", i + 1) {
            differences.push(format!("row {i}: the place reads {:?}", read(&scene.tree(), "settingsPlace", "text")));
        }
        let want_about = if left_count < count { page.about.to_owned() } else { settings(scene).shown_rows()[i].label.clone() };
        if read(&scene.tree(), "asideHeading", "text") != want_about {
            differences.push(format!("row {i}: the About panel names {:?} where {want_about:?}", read(&scene.tree(), "asideHeading", "text")));
        }
    }
    press(scene, sdl, pad, PadControl::Down);
    settle(scene);
    if at(scene).0 != left_count as i32 - 1 {
        differences.push("Down past the last row moved the focus".to_owned());
    }
    // Board rows scrolled into view past the initial window held by their table rows at every gate size.
    if table.iter().any(|r| row_index(&r.name).is_some_and(|i| i >= page.drawn)) {
        for (width, height) in SIZES {
            scene.resize(width, height);
            scene.wait(300);
            let tree = scene.tree();
            let at_size = format!("{width}x{height}, the last row focused");
            let unit: f64 = read(&tree, "settingsCanvas", "unit").parse().expect("the canvas's unit");
            let screen: u64 = tree.nodes[tree.find_named("settingsScreen").expect("Eclipse's Settings") as usize].id.parse().unwrap();
            let tokens = crate::eclipse_board::tokens(screen);
            let in_window: Vec<bool> = (0..left_count).map(|i| shown(scene, &format!("row.{i}"))).collect();
            let scrolled: Vec<Row> = table.iter().filter(|r| row_index(&r.name).is_some_and(|i| i >= page.drawn && in_window.get(i) == Some(&true))).cloned().collect();
            held(&tree, &scrolled, k, unit, f64::from(width) / unit, &tokens, &at_size, &|_, _, _| false, &mut differences);
        }
        scene.resize(1920, 1200);
        scene.wait(300);
    }
    lines_cut(scene, page, left_count, k, "the last row focused", &mut differences);
    let tree = scene.tree();
    let in_window: Vec<bool> = (0..left_count).map(|i| shown(scene, &format!("row.{i}"))).collect();
    lines_held(&tree, &lines_of(page.lines, |i| in_window.get(i) == Some(&true)).lines().filter(|l| !l.starts_with("aside")).map(|l| format!("{l}\n")).collect::<String>(), k, "the last row focused", &mut differences);

    // Each row acted on by the pad from the last: Right then Left, A twice, or A and B; then X.
    let mut opened = Vec::new();
    for i in (0..left_count).rev() {
        if i + 1 < left_count {
            press(scene, sdl, pad, PadControl::Up);
            settle(scene);
        }
        if at(scene).0 != i as i32 {
            differences.push(format!("Up to row {i}: the model at {:?}", at(scene)));
            break;
        }
        let row = settings(scene).shown_rows()[i].clone();
        if matches!(row.kind, RowKind::Opens | RowKind::Asks) {
            if row.held {
                opened.push(format!("{}: held", row.label));
            } else {
                press(scene, sdl, pad, PadControl::South);
                settle(scene);
                if scene.property("folders", "waiting") == "true" {
                    opened.push(format!("{}: folder picker ({})", row.label, scene.property("folders", "asked")));
                    assert!(scene.set_property("folders", "cancel", "true"));
                    wait_for(scene, "folders", "waiting", "false", 1000);
                    assert!(scene.set_property("folders", "cancel", "false"));
                    settle(scene);
                } else if scene.property("screens", "shown") != "Settings" {
                    opened.push(format!("{}: screen {}", row.label, scene.property("screens", "shown")));
                    press(scene, sdl, pad, PadControl::East);
                    wait_for(scene, "screens", "shown", "Settings", 2000);
                    settle(scene);
                } else {
                    opened.push(format!("{}: {}", row.label, at(scene).1));
                    press(scene, sdl, pad, PadControl::East);
                    settle(scene);
                }
            }
        }
        // Swap A and B in menus swaps the very buttons that turn it (and X with Y): turned on by South, off again by East, X then North.
        if row.id == "swap" {
            let before = state(scene, i);
            let mut went = vec![before.clone()];
            for control in [PadControl::South, PadControl::East, PadControl::West, PadControl::South, PadControl::North] {
                press(scene, sdl, pad, control);
                settle(scene);
                went.push(state(scene, i));
            }
            if before.1 != Some(false) || went[1].1 != Some(true) || went[2] != before || went[3] != before || went[4].1 != Some(true) || went[5] != before || at(scene) != (i as i32, page.key.to_owned()) {
                differences.push(format!("the pad on {}: {went:?}, left {:?}", row.label, at(scene)));
            }
            continue;
        }
        acted(scene, page.key, i, "the pad", &mut differences, |scene, by| press(scene, sdl, pad, if by > 0 { PadControl::Right } else if by < 0 { PadControl::Left } else { PadControl::South }), &mut |scene| press(scene, sdl, pad, PadControl::East));
        let defaultable = settings(scene).view().defaultable(i);
        press(scene, sdl, pad, PadControl::West);
        settle(scene);
        let default = state(scene, i);
        if defaultable {
            let change = if default.1.is_some() { PadControl::South } else { PadControl::Right };
            press(scene, sdl, pad, change);
            settle(scene);
            let changed = state(scene, i);
            press(scene, sdl, pad, PadControl::West);
            settle(scene);
            if changed == default || state(scene, i) != default {
                differences.push(format!("X on row {i}: {default:?} → {changed:?} → {:?}", state(scene, i)));
            }
        } else if at(scene) != (i as i32, page.key.to_owned()) {
            differences.push(format!("X on row {i}, non-defaultable, left {:?}", at(scene)));
        }
    }
    println!("{}: the doors open {opened:?}", page.name);
    assert!(differences.is_empty(), "{} differences by the pad:\n{}", differences.len(), differences.join("\n"));

    // The pointer: each row focused where it is drawn, the wheel bringing later rows in; chevrons step, a click turns a switch or opens a door.
    for i in 0..left_count {
        for _ in 0..left_count {
            if shown(scene, &format!("row.{i}")) {
                break;
            }
            let (x, y) = centre(scene, "settingsRows");
            let towards = if at(scene).0 < i as i32 { -1 } else { 1 };
            scene.wheel(x, y, towards);
            settle(scene);
        }
        // Onto the row from the heading, as a row the wheel brought under a pointer at rest takes nothing (EmuSen_Kit.md §7.108).
        let (x, y) = centre(scene, &format!("rowName.{i}"));
        let (hx, hy) = centre(scene, "settingsHeading");
        scene.hover(hx, hy);
        scene.hover(x + 1.0, y);
        scene.hover(x, y);
        settle(scene);
        if at(scene).0 != i as i32 {
            differences.push(format!("the pointer on row {i} focused {:?}, the row drawn {}, the window from row {}", at(scene), shown(scene, &format!("row.{i}")), scene.property("settingsWalk", "first")));
            continue;
        }
        acted(
            scene,
            page.key,
            i,
            "the pointer",
            &mut differences,
            |scene, by| {
                let name = match by {
                    1 => format!("rowMore.{i}"),
                    -1 => format!("rowLess.{i}"),
                    _ => format!("rowName.{i}"),
                };
                let (x, y) = centre(scene, &name);
                scene.click(x, y);
            },
            &mut |scene| press(scene, sdl, pad, PadControl::East),
        );
    }
    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));

    // B: a page under a section back to the row that opened it, then a section's B leaves Settings for Home.
    if let Some(row) = page.opens {
        press(scene, sdl, pad, PadControl::East);
        settle(scene);
        let index = ids(scene).iter().position(|i| i == row).expect("the row that opened the page") as i32;
        assert_eq!(at(scene), (index, page.section.to_owned()), "B closes the page onto its row");
        assert!(!shown(scene, "settingsCrumb"), "no trail over a section's heading");
    }
    press(scene, sdl, pad, PadControl::East);
    wait_for(scene, "screens", "shown", "Home", 2000);
    scene.assert_quiet();
}

/// The ring's value and share where a meter is the focused row's preview: the row's words inside, filled to Settings.meter's share of 270°.
pub fn ring(scene: &mut Scene<'_>, row: usize, caption: &str, differences: &mut Vec<String>) {
    let s = settings(scene);
    let (value, meter) = (s.shown_rows()[row].value.clone(), s.view().pane(row).meter);
    let tree = scene.tree();
    if !shown(scene, "asideMeter") || shown(scene, "asideFaces") {
        differences.push(format!("row {row}: the ring drawn {}, the faces {}", shown(scene, "asideMeter"), shown(scene, "asideFaces")));
    }
    if read(&tree, "asideMeterValue", "text") != value {
        differences.push(format!("row {row}: the ring says {:?} where the row is {value:?}", read(&tree, "asideMeterValue", "text")));
    }
    if read(&tree, "asideMeterCaption", "text") != caption || shown(scene, "asideMeterCaption") == caption.is_empty() {
        differences.push(format!("row {row}: the ring's caption {:?}, drawn {}, where {caption:?}", read(&tree, "asideMeterCaption", "text"), shown(scene, "asideMeterCaption")));
    }
    // The fill begins one less its share of the turn round from the gap's end, as Qt's gradient turns the other way; its stops in order at any share.
    let begin: f64 = read(&tree, "asideRingFill", "begin").parse().unwrap_or(f64::NAN);
    let want = (1.0 - 0.75 * meter).clamp(0.2502, 0.9999);
    if !(0.25 <= begin - 0.0001 && begin <= 1.0) {
        differences.push(format!("row {row}: the ring's stops out of order: the fill begins at {begin}"));
    }
    if !same(begin, want) {
        differences.push(format!("row {row}: the ring's fill begins at {begin} of the turn where the share {meter} puts it at {want}"));
    }
    // The value and the caption centred in the ring (G:167).
    let unit: f64 = read(&tree, "settingsCanvas", "unit").parse().expect("the canvas's unit");
    let got = boxes(&tree, unit);
    let ring = got["asideRing"];
    for name in ["asideMeterValue", "asideMeterCaption"] {
        let b = got[name];
        if caption.is_empty() && name == "asideMeterCaption" {
            continue;
        }
        if (b.x + b.w / 2.0 - (ring.x + ring.w / 2.0)).abs() > 1e-6 * ring.w {
            differences.push(format!("row {row}: {name} centred at {} where the ring's middle is {}", b.x + b.w / 2.0, ring.x + ring.w / 2.0));
        }
    }
}

/// A trail of three levels (R-20261123-39): from Saves and states to a console's page and a page under it, the line over the heading every level above it joined as the boards join them; then back to the board's state.
pub fn three_levels(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, page: &Page, backend: &str, differences: &mut Vec<String>) {
    press(scene, sdl, pad, PadControl::East);
    settle(scene);
    for _ in 0..6 {
        if at(scene).1 == "consoles" {
            break;
        }
        press(scene, sdl, pad, PadControl::LeftShoulder);
        settle(scene);
    }
    press(scene, sdl, pad, PadControl::South);
    settle(scene);
    let rows = settings(scene).shown_rows().to_vec();
    let Some(door) = rows.iter().position(|r| r.kind == RowKind::Opens) else {
        differences.push(format!("the console's page {:?} has no page under it", at(scene).1));
        return;
    };
    for _ in 0..door {
        press(scene, sdl, pad, PadControl::Down);
    }
    press(scene, sdl, pad, PadControl::South);
    settle(scene);
    let trail = settings(scene).view().trail();
    let tree = scene.tree();
    let want = trail[..trail.len().saturating_sub(1)].join("  ›  ");
    if trail.len() != 3 || read(&tree, "settingsCrumb", "text") != want || !shown(scene, "settingsCrumb") || read(&tree, "settingsHeading", "text") != *trail.last().unwrap() {
        differences.push(format!("a trail of {trail:?}: the line over the heading {:?}, drawn {}, the heading {:?}", read(&tree, "settingsCrumb", "text"), shown(scene, "settingsCrumb"), read(&tree, "settingsHeading", "text")));
    }
    let unit: f64 = read(&tree, "settingsCanvas", "unit").parse().expect("the canvas's unit");
    let k = DESIGN_HEIGHT / 1200.0;
    // The trail's and the heading's line boxes, each Line before its words in the tree.
    let line = |name: &str| tree.nodes.iter().find(|n| n.name == name && n.visible).map_or((f64::NAN, f64::NAN), |n| (n.y / unit, (n.y + n.h) / unit));
    let (heading, trail) = (line("settingsHeading").0, line("settingsCrumb").1);
    if !same(heading, trail + 16.0 * k) {
        differences.push(format!("the heading at {heading} under a trail ending at {trail}"));
    }
    pictures(scene, backend, "trail-of-three");
    press(scene, sdl, pad, PadControl::East);
    settle(scene);
    press(scene, sdl, pad, PadControl::East);
    settle(scene);
    for _ in 0..door {
        press(scene, sdl, pad, PadControl::Up);
    }
    reach(scene, sdl, pad, page);
}

/// gen_settings.py:172-178: four rows, Volume focused, the ring in the About panel.
pub const SOUND: Page = Page {
    name: "sound",
    table: include_str!("../eclipse/settings_sound.tsv"),
    lines: include_str!("../eclipse/settings_sound_lines.tsv"),
    section: "sound",
    opens: None,
    key: "sound",
    heading: "Sound",
    crumb: "",
    focus: 0,
    drawn: 4,
    board: &[
        (0, "Volume", "The game's volume.", Some(false)),
        (1, "Navigation sounds", "The sounds for moving, choosing and going back, on a stream of their own beside the game's.", Some(true)),
        (2, "Sound volume", "How loud the navigation sounds are, beside the game's own volume.", Some(false)),
        (3, "Output", "Where the game's sound and the interface's sounds play.", Some(false)),
    ],
    program_only: &[],
    hidden: &[],
    glow: "#7A3FA8",
    about: "Volume",
    sentence: "The game's volume. On the handheld this row defers to the device's quick menu.",
    // R-20261123-12's words for the board's line.
    muted: &["The ring fills as the volume rises; the next sound plays at the level you chose."],
    help: &["Change", "Choose or open", "Back to the default", "Back", "Section", "Home"],
};

/// The Sound page's oracle on `backend`.
pub fn sound(backend: &str) {
    oracle(&SOUND, backend, &|scene, sdl, pad, differences| {
        ring(scene, 0, "the game's volume", differences);
        // The ring follows a step, and Sound volume's ring has no caption (R-20261123-38); Output has none.
        press(scene, sdl, pad, PadControl::Right);
        settle(scene);
        ring(scene, 0, "the game's volume", differences);
        press(scene, sdl, pad, PadControl::Left);
        press(scene, sdl, pad, PadControl::Down);
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        ring(scene, 2, "", differences);
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        if shown(scene, "asideMeter") {
            differences.push("Output draws the ring".to_owned());
        }
        for _ in 0..3 {
            press(scene, sdl, pad, PadControl::Up);
        }
        settle(scene);
    });
}

/// gen_settings.py:180-195: the board's nine rows among the program's fifteen, Controller type focused, the four faces in the About panel.
pub const CONTROLLERS: Page = Page {
    name: "controllers",
    table: include_str!("../eclipse/settings_controllers.tsv"),
    lines: include_str!("../eclipse/settings_controllers_lines.tsv"),
    section: "controllers",
    opens: None,
    key: "controllers",
    heading: "Controllers",
    crumb: "",
    focus: 8,
    drawn: 9,
    board: &[
        (3, "Rumble", "The motors in your controller, where the console's games drive them.", Some(false)),
        (4, "Swap A and B in menus", "A and B trade what they do in the library, the menus and every sheet, and so do X and Y.", Some(true)),
        (5, "Guide chords", "Guide held with Up saves, with Down loads, with R1 takes a screenshot.", Some(false)),
        (8, "Controller type", "Which buttons the help bar draws.", Some(false)),
        (9, "Only the first controller", "Only the first controller connected steers the menus.", Some(false)),
        (10, "Controller notices", "A notice at the top of the screen when a controller is connected or disconnected.", Some(true)),
        (11, "Left stick as the D-pad", "In games whose console has no stick, the left stick also presses the D-pad.", Some(true)),
        (12, "Stick deadzone", "How far a stick must travel before it counts as a direction.", Some(false)),
    ],
    program_only: &[
        (0, "Players"),
        (1, "Controller bindings"),
        (2, "Profiles"),
        (6, "Keyboard keys"),
        (7, "Hotkeys"),
        (13, "Mirror player 1 to player 2"),
        (14, "Reset every binding"),
    ],
    // R-20261112-13: the On-screen keyboard row waits for the Steam keyboard call.
    hidden: &["On-screen keyboard"],
    glow: "#2A6FD6",
    about: "Controller type",
    sentence: "Which buttons the help bar draws. Automatic follows the controller last pressed; only the pictures change.",
    muted: &["Automatic · Xbox · PlayStation · Nintendo · Generic"],
    help: &["Change", "Choose or open", "Back to the default", "Back", "Section", "Home"],
};

/// The Controllers page's oracle on `backend`.
pub fn controllers(backend: &str) {
    oracle(&CONTROLLERS, backend, &|scene, sdl, pad, differences| {
        // The four faces as the help bar draws them now, A filled, the others ringed in the muted ink, every letter bold (G:181-183).
        let tree = scene.tree();
        if !shown(scene, "asideFaces") || shown(scene, "asideMeter") {
            differences.push(format!("Controller type: the faces drawn {}, the ring {}", shown(scene, "asideFaces"), shown(scene, "asideMeter")));
        }
        let faces: Vec<(String, String, String)> = (0..4).map(|i| { let n = format!("asideFace.{i}"); (read(&tree, &n, "letter"), read(&tree, &n, "filled"), read(&tree, &n, "letterWeight")) }).collect();
        let want: Vec<(String, String, String)> = [("A", "true"), ("B", "false"), ("X", "false"), ("Y", "false")].iter().map(|(l, f)| (l.to_string(), f.to_string(), "700".to_owned())).collect();
        if faces != want {
            differences.push(format!("the faces are {faces:?} where the board draws {want:?}"));
        }
        // Stick deadzone's ring, the row's value with no caption (R-20261123-38).
        for _ in 0..4 {
            press(scene, sdl, pad, PadControl::Down);
        }
        settle(scene);
        ring(scene, 12, "", differences);
        for _ in 0..4 {
            press(scene, sdl, pad, PadControl::Up);
        }
        settle(scene);
    });
}

/// gen_settings.py:332-341: seven rows under Library, Default sort order focused, its orders listed from the program's choices (R-20261123-18).
pub const COLLECTIONS: Page = Page {
    name: "collections",
    table: include_str!("../eclipse/settings_collections.tsv"),
    lines: include_str!("../eclipse/settings_collections_lines.tsv"),
    section: "library",
    opens: Some("collections"),
    key: "group:Collections",
    heading: "Collections",
    crumb: "Library",
    focus: 4,
    drawn: 7,
    board: &[
        (0, "Group your collections", "If unthemed puts a collection the theme has no folder for inside Collections.", Some(true)),
        (1, "Favourites on top", "Favourite games sorted above the rest.", Some(false)),
        (2, "Favourites on top in your collections", "The same, in your own collections.", Some(false)),
        (3, "Stars in your collections", "A star beside a favourite in your own collections.", Some(false)),
        (4, "Default sort order", "A list's own choice in its options lasts until EmuSen closes.", Some(true)),
        (5, "Folders on top", "A console's folders above its games, as ES-DE sorts them.", Some(false)),
        (6, "Random entry button", "Either thumbstick pressed in jumps to a random game, or console.", Some(true)),
    ],
    program_only: &[],
    hidden: &[],
    glow: "#1FA890",
    about: "Default sort order",
    sentence: "A list's own choice in its options lasts until EmuSen closes.",
    muted: &["Name · Rating · Release date · Developer · Publisher · Genre · Players · Last played · Times played · Play time, each ascending or descending."],
    help: &["Change", "Choose or open", "Back to the default", "Back", "Home"],
};

/// The Collections page's oracle on `backend`.
pub fn collections(backend: &str) {
    oracle(&COLLECTIONS, backend, &|_, _, _, _| {});
}

/// gen_settings.py:343-347: two rows under EmuSen, Continue where you left off focused.
pub const SAVES: Page = Page {
    name: "saves",
    table: include_str!("../eclipse/settings_saves.tsv"),
    lines: include_str!("../eclipse/settings_saves_lines.tsv"),
    section: "emusen",
    opens: Some("saves"),
    key: "group:Saves and states",
    heading: "Saves and states",
    crumb: "EmuSen",
    focus: 0,
    drawn: 2,
    board: &[
        (0, "Continue where you left off", "Every game is saved when it is closed; this is what happens the next time it starts.", Some(true)),
        (1, "Play time counted up to", "A launch longer than this adds nothing to the game's play time.", Some(false)),
    ],
    program_only: &[],
    hidden: &[],
    glow: "#B0487A",
    about: "Continue where you left off",
    sentence: "Every game is saved when it is closed; this is what happens the next time it starts.",
    // R-20261123-37: the third choice is the board's word.
    muted: &["Ask each time · Always resume · Always restart"],
    help: &["Change", "Choose or open", "Back to the default", "Back", "Home"],
};

/// The Saves and states page's oracle, and a trail of three levels, on `backend`.
pub fn saves(backend: &str) {
    oracle(&SAVES, backend, &|scene, sdl, pad, differences| three_levels(scene, sdl, pad, &SAVES, backend, differences));
}

/// Board #30 (EclipseSettingsLibrary.dc.html): Library section, ROM folder focused with the count card in the About panel, 12 rows in the program (R-20261116-21, -24).
pub const LIBRARY: Page = Page {
    name: "library",
    table: include_str!("../eclipse/settings_library.tsv"),
    lines: include_str!("../eclipse/settings_library_lines.tsv"),
    section: "library",
    opens: None,
    key: "library",
    heading: "Library",
    crumb: "",
    focus: 0,
    drawn: 9,
    board: &[
        (0, "ROM folder", "The folder the library lists, read in place and never moved.", None),
        (1, "Artwork folder", "Box art, named like the game's file; blank keeps it beside the games.", Some(true)),
        (2, "Themes", "The big picture's look: EmuSen's own, or an ES-DE theme's.", Some(false)),
        (3, "Scraping", "Where art and facts come from, and what is fetched.", Some(false)),
        (5, "Collections", "The automatic collections shown, your own, and how lists are sorted.", Some(false)),
        (6, "Hidden games", "Games hidden from the library. Their files are never touched.", Some(false)),
        (8, "Cheats", "The cheats of the game last played, turned on or off.", Some(false)),
        (9, "Compressed discs", "The library's discs converted to CHD beside them, checked and listed.", Some(false)),
        (10, "ES-DE media folder", "An ES-DE downloaded_media folder whose art the theme shows; only ever read.", Some(true)),
    ],
    program_only: &[
        (4, "Account and run"),
        (7, "Media viewer"),
        (11, "ES-DE theme folder"),
    ],
    hidden: &[],
    glow: "#1FA890",
    about: "ROM folder",
    sentence: "The folder the library lists, read in place and never moved.",
    muted: &["A opens the folder picker. EmuSen reads this folder. It writes there only if you ask it to convert discs beside the originals."],
    help: &["Change", "Choose or open", "Back to the default", "Back", "Section", "Home"],
};

/// The Library page's oracle on `backend`.
pub fn library(backend: &str) {
    oracle(&LIBRARY, backend, &|scene, sdl, pad, differences| {
        let tree = scene.tree();
        let rom_dir = settings(scene).shown_rows()[0].value.clone();
        if !shown(scene, "asideFolder") {
            differences.push("ROM folder: the count card is not drawn".to_owned());
        }
        for (name, want) in [("asideFolderCount", "1"), ("asideFolderUnder", "game under"), ("asideFolderPath", rom_dir.as_str())] {
            let have = read(&tree, name, "text");
            if have != want {
                differences.push(format!("ROM folder: {name} says {have:?} where {want:?}"));
            }
        }
        // Monospace path in the aside; Sora on the row's value (owner ruling 3c).
        if !read(&tree, "asideFolderPath", "font").contains("JetBrains Mono") {
            differences.push(format!("ROM folder: asideFolderPath font is {:?}, not JetBrains Mono", read(&tree, "asideFolderPath", "font")));
        }
        if !read(&tree, "rowWords.0", "font").contains("Sora") {
            differences.push(format!("ROM folder: rowWords.0 font is {:?}, not Sora", read(&tree, "rowWords.0", "font")));
        }
        // Moving to Artwork folder hides the count card; back to ROM folder restores it.
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
        if shown(scene, "asideFolder") {
            differences.push("Artwork folder draws the ROM folder count card".to_owned());
        }
        press(scene, sdl, pad, PadControl::Up);
        settle(scene);
        if !shown(scene, "asideFolder") {
            differences.push("ROM folder lost its count card after returning".to_owned());
        }
    });
}

/// Board #31 (EclipseSettingsScraping.dc.html): Scraping under Library, 7 left rows and 11 media-kind chips in the About panel (R-20261116-22).
pub const SCRAPING: Page = Page {
    name: "scraping",
    table: include_str!("../eclipse/settings_scraping.tsv"),
    lines: include_str!("../eclipse/settings_scraping_lines.tsv"),
    section: "library",
    opens: Some("scraping"),
    key: "group:Scraping",
    heading: "Scraping",
    crumb: "Library",
    focus: 0,
    drawn: 7,
    board: &[
        (0, "Use ScreenScraper", "Art and facts from ScreenScraper, for the games the Scraper is asked to fetch.", Some(true)),
        (1, "Region", "Whose box art and names come first.", Some(false)),
        (2, "Other regions after", "Else world, USA, Europe, Japan, then any.", Some(false)),
        (3, "Language", "The language of descriptions and names.", Some(false)),
        (4, "ScreenScraper's names", "Show the names ScreenScraper gives rather than the files'.", Some(false)),
        (5, "Refresh what is kept", "Fetch again what a game already has.", Some(false)),
        (6, "OpenEmu's sources", "OpenVGDB and libretro's thumbnails when ScreenScraper has nothing.", Some(false)),
    ],
    program_only: &[],
    hidden: &[],
    glow: "#1FA890",
    about: "What is fetched",
    sentence: "",
    muted: &["9 of 11 kinds. Each is its own switch; move right from the rows to reach them."],
    help: &["Change", "Choose or open", "Back to the default", "Back", "Home"],
};

/// The Scraping page's oracle on `backend`, including Right into the 11 chips, flow navigation, A toggle, X default, Left/B return, and pointer clicks.
pub fn scraping(backend: &str) {
    oracle(&SCRAPING, backend, &|scene, sdl, pad, differences| {
        let chip_labels = ["Covers", "Screenshots", "Marquees", "Mix images", "Title screens", "Back covers", "Physical media", "Fan art", "3D boxes", "Manuals", "Videos"];
        let tree = scene.tree();
        for (i, want) in chip_labels.iter().enumerate() {
            let have = read(&tree, &format!("chipWords.{i}"), "text");
            if have != *want {
                differences.push(format!("chip {i} says {have:?} where {want:?}"));
            }
            let on = i < 9;
            if shown(scene, &format!("chipCheck.{i}")) != on || shown(scene, &format!("chipDot.{i}")) != !on {
                differences.push(format!("chip {i} ({want}): check {}, dot {} where on={on}", shown(scene, &format!("chipCheck.{i}")), shown(scene, &format!("chipDot.{i}"))));
            }
        }
        // Right from a switch row enters chip 0 with standard Eclipse FocusRing (owner decisions 2a, 2b).
        press(scene, sdl, pad, PadControl::Right);
        settle(scene);
        if at(scene) != (7, "group:Scraping".to_owned()) || focused(scene) != "chip.0" {
            differences.push(format!("Right from Use ScreenScraper left {:?} focused {:?}", at(scene), focused(scene)));
        }
        if read(&scene.tree(), "settingsPlace", "text") != "8 of 18" {
            differences.push(format!("chip 0 place reads {:?}", read(&scene.tree(), "settingsPlace", "text")));
        }
        if !shown(scene, "chipRing") {
            differences.push("focused chip 0 draws no FocusRing".to_owned());
        } else {
            let tree = scene.tree();
            let unit: f64 = read(&tree, "settingsCanvas", "unit").parse().expect("the canvas's unit");
            let screen: u64 = tree.nodes[tree.find_named("settingsScreen").expect("Eclipse's Settings") as usize].id.parse().unwrap();
            let tokens = crate::eclipse_board::tokens(screen);
            let ring_row = Row {
                name: "chipRing".to_owned(),
                part: "FocusRing".to_owned(),
                x: "-".to_owned(),
                y: "-".to_owned(),
                w: "-".to_owned(),
                h: "-".to_owned(),
                radius: "-".to_owned(),
                colour: "-".to_owned(),
                size: "-".to_owned(),
                status: "built".to_owned(),
                kind: "ring 3 3 40 8 33 #66F0B4C4 #FF07080F #FFF6E7D8".to_owned(),
            };
            held(&tree, &[ring_row], DESIGN_HEIGHT / 1200.0, unit, 1920.0 / unit, &tokens, "chip 0 focused", &|_, _, _| false, differences);
        }
        // Walk the wrapped flow: Right -> chip 1, Down -> chip 3, Left -> chip 2, Up -> chip 0; toggle chip 0 with A and reset with X.
        for (ctl, want_row, want_focus) in [(PadControl::Right, 8, "chip.1"), (PadControl::Down, 10, "chip.3"), (PadControl::Left, 9, "chip.2"), (PadControl::Up, 7, "chip.0")] {
            press(scene, sdl, pad, ctl);
            settle(scene);
            if at(scene).0 != want_row || focused(scene) != want_focus {
                differences.push(format!("chip walk {ctl:?}: got {:?} / {:?}, want {want_row} / {want_focus}", at(scene), focused(scene)));
            }
        }
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        if read(&scene.tree(), "asideMuted.0", "text") != "8 of 11 kinds. Each is its own switch; move right from the rows to reach them." {
            differences.push(format!("after turning Covers off, summary reads {:?}", read(&scene.tree(), "asideMuted.0", "text")));
        }
        press(scene, sdl, pad, PadControl::West);
        settle(scene);
        if read(&scene.tree(), "asideMuted.0", "text") != "9 of 11 kinds. Each is its own switch; move right from the rows to reach them." {
            differences.push(format!("after X on Covers, summary reads {:?}", read(&scene.tree(), "asideMuted.0", "text")));
        }
        // Walk to chip 10 (Videos) and toggle on/off; Left from leftmost chip returns to row 0.
        for ctl in [PadControl::Down, PadControl::Down, PadControl::Down, PadControl::Down, PadControl::Right, PadControl::Down] {
            press(scene, sdl, pad, ctl);
            settle(scene);
        }
        if at(scene).0 != 17 || focused(scene) != "chip.10" || read(&scene.tree(), "settingsPlace", "text") != "18 of 18" {
            differences.push(format!("walk to chip 10 left {:?} / {:?} / {:?}", at(scene), focused(scene), read(&scene.tree(), "settingsPlace", "text")));
        }
        press(scene, sdl, pad, PadControl::South);
        settle(scene);
        if read(&scene.tree(), "asideMuted.0", "text") != "10 of 11 kinds. Each is its own switch; move right from the rows to reach them." {
            differences.push(format!("after turning Videos on, summary reads {:?}", read(&scene.tree(), "asideMuted.0", "text")));
        }
        press(scene, sdl, pad, PadControl::West);
        settle(scene);
        press(scene, sdl, pad, PadControl::Left);
        settle(scene);
        if at(scene) != (0, "group:Scraping".to_owned()) || focused(scene) != "row.0" {
            differences.push(format!("Left from leftmost chip left {:?} / {:?}", at(scene), focused(scene)));
        }
        // From another switch row (row 4), Right enters chips and B returns to that remembered row without leaving Scraping.
        for _ in 0..4 {
            press(scene, sdl, pad, PadControl::Down);
        }
        settle(scene);
        press(scene, sdl, pad, PadControl::Right);
        settle(scene);
        if at(scene).0 != 7 || focused(scene) != "chip.0" {
            differences.push(format!("Right from row 4 left {:?} / {:?}", at(scene), focused(scene)));
        }
        press(scene, sdl, pad, PadControl::East);
        settle(scene);
        if at(scene) != (4, "group:Scraping".to_owned()) || focused(scene) != "row.4" {
            differences.push(format!("B from chips left {:?} / {:?}, want row 4", at(scene), focused(scene)));
        }
        // Pointer click on chip 10 toggles Videos on and back off; return to row 0 for the main walk.
        let (cx, cy) = centre(scene, "chip.10");
        scene.click(cx, cy);
        settle(scene);
        if read(&scene.tree(), "asideMuted.0", "text") != "10 of 11 kinds. Each is its own switch; move right from the rows to reach them." {
            differences.push(format!("pointer click on Videos left {:?}", read(&scene.tree(), "asideMuted.0", "text")));
        }
        scene.click(cx, cy);
        settle(scene);
        let (hx, hy) = centre(scene, "settingsHeading");
        scene.hover(hx, hy);
        press(scene, sdl, pad, PadControl::East);
        settle(scene);
        for _ in 0..4 {
            press(scene, sdl, pad, PadControl::Up);
        }
        settle(scene);
    });
}

/// Board #32 (EclipseSettingsEmuSen.dc.html): EmuSen section, 3 rows (Big picture, Logging, Shell hidden per rulings), About focused with the eclipse mark.
pub const EMUSEN: Page = Page {
    name: "emusen",
    table: include_str!("../eclipse/settings_emusen.tsv"),
    lines: include_str!("../eclipse/settings_emusen_lines.tsv"),
    section: "emusen",
    opens: None,
    key: "emusen",
    heading: "EmuSen",
    crumb: "",
    focus: 2,
    drawn: 3,
    board: &[
        (0, "Saves and states", "Where states go, what happens when a game starts again, and how play time is counted.", Some(true)),
        (1, "Pause in the background", "A game paused because another window came to the front starts again when this one comes back.", Some(true)),
        (2, "About", "EmuSen's version.", Some(false)),
    ],
    program_only: &[],
    hidden: &["Big picture", "Logging", "Shell"],
    glow: "#B0487A",
    about: "About",
    sentence: "EmuSen's version.",
    muted: &[],
    help: &["Change", "Choose or open", "Back to the default", "Back", "Section", "Home"],
};

/// The EmuSen page's oracle on `backend`.
pub fn emusen(backend: &str) {
    oracle(&EMUSEN, backend, &|scene, sdl, pad, differences| {
        let tree = scene.tree();
        let version = emusen_qt_models::words::version_line().strip_prefix("EmuSen ").unwrap_or("").to_owned();
        if !shown(scene, "asideMark") {
            differences.push("About: the eclipse mark card is not drawn".to_owned());
        }
        if read(&tree, "rowWords.2", "text") != version || read(&tree, "asideLine.0", "text") != version {
            differences.push(format!("About version reads {:?} / {:?} where {version:?}", read(&tree, "rowWords.2", "text"), read(&tree, "asideLine.0", "text")));
        }
        // Monospace version in the aside; Sora on the row's value (owner ruling 3c).
        if !read(&tree, "asideLine.0", "font").contains("JetBrains Mono") {
            differences.push(format!("About: asideLine.0 font is {:?}, not JetBrains Mono", read(&tree, "asideLine.0", "font")));
        }
        if !read(&tree, "rowWords.2", "font").contains("Sora") {
            differences.push(format!("About: rowWords.2 font is {:?}, not Sora", read(&tree, "rowWords.2", "font")));
        }
        // Saves and states row carries R-20261123-19's About sentence and hides the eclipse mark.
        press(scene, sdl, pad, PadControl::Up);
        press(scene, sdl, pad, PadControl::Up);
        settle(scene);
        if shown(scene, "asideMark") {
            differences.push("Saves and states draws the eclipse mark".to_owned());
        }
        let saves_sentence = read(&scene.tree(), "asideSentence", "text");
        if saves_sentence != "States and saves are kept in EmuSen's own Saves folder, with one resume point per game." {
            differences.push(format!("Saves and states About sentence says {saves_sentence:?}"));
        }
        press(scene, sdl, pad, PadControl::Down);
        press(scene, sdl, pad, PadControl::Down);
        settle(scene);
    });
}
