//! The oracle of Eclipse's Settings on Sound, Controllers, Collections and Saves and states, each at its board's state, by pad and pointer (EmuSen_Qt.md §3.4.11).
#![allow(dead_code)]

use crate::eclipse_board::{boxes, held, read, rows, same};
use crate::eclipse_pad::{focused, press};
use crate::eclipse_settings_oracle::{acted, at, centre, help_line, pictures, settings, settle, shown, state, with_settings};
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
    /// The section's key, and the row of it that opens the page where the page is one under it.
    pub section: &'static str,
    pub opens: Option<&'static str>,
    /// The page's key, as Settings.page gives it.
    pub key: &'static str,
    pub heading: &'static str,
    pub crumb: &'static str,
    /// The row the board focuses, and how many rows its window draws there.
    pub focus: usize,
    pub drawn: usize,
    /// The rows the board draws: the program's row, its name, its short line and whether the board cuts it.
    pub board: &'static [(usize, &'static str, &'static str, bool)],
    /// Rows the board draws that an earlier ruling hides, by name.
    pub hidden: &'static [&'static str],
    /// Doors whose B is not held to return to their row, by name, and why.
    pub unheld: &'static [(&'static str, &'static str)],
    pub glow: &'static str,
    /// The About panel's heading, and its small lines as the board's state gives them.
    pub about: &'static str,
    pub muted: &'static [&'static str],
    /// The help line's words in order.
    pub help: &'static [&'static str],
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
fn lines_cut(scene: &mut Scene<'_>, page: &Page, k: f64, at_size: &str, differences: &mut Vec<String>) {
    let tree = scene.tree();
    let unit: f64 = read(&tree, "settingsCanvas", "unit").parse().expect("the canvas's unit");
    let got = boxes(&tree, unit);
    for i in 0..settings(scene).shown_rows().len() {
        if !shown(scene, &format!("row.{i}")) || !shown(scene, &format!("rowAbout.{i}")) {
            continue;
        }
        let (line, value) = (got[&format!("rowAbout.{i}")], got[&format!("rowValue.{i}")]);
        let cut = read(&tree, &format!("rowAbout.{i}"), "truncated") == "true";
        if line.right() > value.x - 24.0 * k + 1e-6 || (cut && line.right() < value.x - 24.0 * k - 1e-6) {
            differences.push(format!("{at_size}: row {i}'s short line ends at {} with the value at {}, cut {cut}", line.right(), value.x));
        }
        if let Some((_, _, _, board_cut)) = page.board.iter().find(|b| b.0 == i) {
            if cut != *board_cut {
                differences.push(format!("{at_size}: row {i}'s short line cut {cut} where the board's is cut {board_cut}"));
            }
        }
    }
}

/// The pad back to row `i` of the page from wherever its focus stands.
fn refocus(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, i: usize) {
    let at_now = at(scene).0.max(0) as usize;
    for _ in i..at_now {
        press(scene, sdl, pad, PadControl::Up);
    }
    for _ in at_now..i {
        press(scene, sdl, pad, PadControl::Down);
    }
    settle(scene);
}

/// The page held to its board at every gate size, its words, its rows by pad and pointer, X, and B.
pub fn oracle(page: &Page, backend: &str, extra: &dyn Fn(&mut Scene<'_>, &Sdl, &VirtualPad, &mut Vec<String>)) {
    with_settings(|scene, sdl, pad| run(scene, sdl, pad, page, backend, extra));
}

fn run(scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, page: &Page, backend: &str, extra: &dyn Fn(&mut Scene<'_>, &Sdl, &VirtualPad, &mut Vec<String>)) {
    reach(scene, sdl, pad, page);
    let count = settings(scene).shown_rows().len();
    let mut differences = Vec::new();
    for name in page.hidden {
        if settings(scene).shown_rows().iter().any(|r| r.label == *name) {
            differences.push(format!("{name} is listed, which an earlier ruling hides until it is built"));
        }
    }
    let (board_height, table) = rows(page.table);
    let k = DESIGN_HEIGHT / board_height;
    for (width, height) in SIZES {
        scene.resize(width, height);
        scene.wait(300);
        let tree = scene.tree();
        let at_size = format!("{width}x{height}");
        let unit: f64 = read(&tree, "settingsCanvas", "unit").parse().expect("the canvas's unit");
        let screen: u64 = tree.nodes[tree.find_named("settingsScreen").expect("Eclipse's Settings") as usize].id.parse().unwrap();
        let tokens = crate::eclipse_board::tokens(screen);
        held(&tree, &table, k, unit, f64::from(width) / unit, &tokens, &at_size, &|_, _, _| false, &mut differences);
        if Colour::parse(&read(&tree, "backdropField.0", "paint")) != Colour::parse(page.glow) {
            differences.push(format!("{at_size}: the page's glow is {} where the board gives {}", read(&tree, "backdropField.0", "paint"), page.glow));
        }
        let s = settings(scene);
        let model_rows = s.shown_rows().to_vec();
        let facets = s.view().facets();
        let sentence = s.view().pane(page.focus).sentence;
        let mut words = vec![
            ("settingsTitle".to_owned(), "Settings".to_owned()),
            ("settingsHeading".to_owned(), page.heading.to_owned()),
            ("settingsPlace".to_owned(), format!("{} of {count}", page.focus + 1)),
            ("asideHeading".to_owned(), page.about.to_owned()),
            ("asideSentence".to_owned(), sentence),
        ];
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
        for (i, row) in model_rows.iter().enumerate() {
            words.push((format!("rowName.{i}"), row.label.clone()));
            words.push((format!("rowAbout.{i}"), facets[i].about.clone()));
            words.push((format!("rowWords.{i}"), row.value.clone()));
        }
        for (name, want) in words {
            let have = read(&tree, &name, "text");
            if have != want {
                differences.push(format!("{at_size}: {name} says {have:?} where {want:?}"));
            }
        }
        for i in 0..count {
            if shown(scene, &format!("row.{i}")) != (i < page.drawn) {
                differences.push(format!("{at_size}: row {i} drawn {}", i >= page.drawn));
            }
        }
        lines_cut(scene, page, k, &at_size, &mut differences);
    }
    scene.resize(1920, 1200);
    scene.wait(300);
    let defaultable = settings(scene).view().defaultable(page.focus);
    let mut line: Vec<String> = page.help.iter().map(|w| w.to_string()).collect();
    if !defaultable {
        line.retain(|w| w != "Back to the default");
    }
    if help_line(scene).0 != line {
        differences.push(format!("the help line is {:?} where the board's is {:?}", help_line(scene).0, page.help));
    }
    extra(scene, sdl, pad, &mut differences);
    pictures(scene, backend, page.name);

    // The pad: Up to the first row, Down through every row to the last, each focused, drawn and counted, then past the last.
    for _ in 0..count {
        press(scene, sdl, pad, PadControl::Up);
    }
    for i in 0..count {
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
        if read(&scene.tree(), "asideHeading", "text") != settings(scene).shown_rows()[i].label {
            differences.push(format!("row {i}: the About panel names {:?}", read(&scene.tree(), "asideHeading", "text")));
        }
    }
    press(scene, sdl, pad, PadControl::Down);
    settle(scene);
    if at(scene).0 != count as i32 - 1 {
        differences.push("Down past the last row moved the focus".to_owned());
    }
    lines_cut(scene, page, k, "the last row focused", &mut differences);

    // Each row acted on by the pad from the last: Right then Left, A twice, or A and B; then X.
    let mut opened = Vec::new();
    for i in (0..count).rev() {
        if i + 1 < count {
            press(scene, sdl, pad, PadControl::Up);
            settle(scene);
        }
        if at(scene).0 != i as i32 {
            differences.push(format!("Up to row {i}: the model at {:?}", at(scene)));
            break;
        }
        let row = settings(scene).shown_rows()[i].clone();
        if matches!(row.kind, RowKind::Opens | RowKind::Asks) {
            press(scene, sdl, pad, PadControl::South);
            settle(scene);
            opened.push(format!("{}: {}", row.label, at(scene).1));
            press(scene, sdl, pad, PadControl::East);
            settle(scene);
            if let Some((_, why)) = page.unheld.iter().find(|u| u.0 == row.label) {
                // Not held: where B from this door's page lands; the page is left and the focus walked back to the row.
                println!("{}: B from {}'s page lands on {:?} ({why})", page.name, row.label, at(scene));
                if at(scene).1 != page.key || opened.last().is_some_and(|o| o.ends_with(page.key)) {
                    differences.push(format!("{}: A opened {:?}, B left {:?}", row.label, opened.last(), at(scene)));
                }
                refocus(scene, sdl, pad, i);
                press(scene, sdl, pad, PadControl::West);
                settle(scene);
                if at(scene) != (i as i32, page.key.to_owned()) {
                    differences.push(format!("X on {}, a door, left {:?}", row.label, at(scene)));
                }
                continue;
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
        // An action with no value does its work in place: A opens nothing and leaves the focus on its row.
        if row.kind == RowKind::Action && row.value.is_empty() {
            press(scene, sdl, pad, PadControl::South);
            settle(scene);
            if at(scene) != (i as i32, page.key.to_owned()) || scene.property("screens", "shown") != "Settings" {
                differences.push(format!("A on {}, an action, left {:?} on {}", row.label, at(scene), scene.property("screens", "shown")));
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
            differences.push(format!("X on row {i}, a door, left {:?}", at(scene)));
        }
    }
    println!("{}: the doors open {opened:?}", page.name);
    assert!(differences.is_empty(), "{} differences by the pad:\n{}", differences.len(), differences.join("\n"));

    // The pointer: each row focused where it is drawn, the wheel bringing later rows in; chevrons step, a click turns a switch or opens a door.
    for i in 0..count {
        for _ in 0..count {
            if shown(scene, &format!("row.{i}")) {
                break;
            }
            let (x, y) = centre(scene, "settingsRows");
            let towards = if at(scene).0 < i as i32 { -1 } else { 1 };
            scene.wheel(x, y, towards);
            settle(scene);
        }
        let (x, y) = centre(scene, &format!("rowName.{i}"));
        scene.hover(x + 1.0, y);
        scene.hover(x, y);
        settle(scene);
        if at(scene).0 != i as i32 {
            differences.push(format!("the pointer on row {i} focused {:?}, the row drawn {}, the window from row {}", at(scene), shown(scene, &format!("row.{i}")), scene.property("settingsWalk", "first")));
            continue;
        }
        if let Some((_, why)) = page.unheld.iter().find(|u| u.0 == settings(scene).shown_rows()[i].label) {
            scene.click(x, y);
            settle(scene);
            if at(scene).1 == page.key {
                differences.push(format!("a click on row {i}, a door, opened nothing"));
            }
            press(scene, sdl, pad, PadControl::East);
            settle(scene);
            println!("{}: B from row {i}'s page after a click lands on {:?} ({why})", page.name, at(scene));
            refocus(scene, sdl, pad, i);
            continue;
        }
        if settings(scene).shown_rows()[i].kind == RowKind::Action && settings(scene).shown_rows()[i].value.is_empty() {
            let (x, y) = centre(scene, &format!("rowName.{i}"));
            scene.click(x, y);
            settle(scene);
            if at(scene) != (i as i32, page.key.to_owned()) {
                differences.push(format!("a click on row {i}, an action, left {:?}", at(scene)));
            }
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
    crate::common::wait_for(scene, "screens", "shown", "Home", 2000);
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
    let end: f64 = read(&tree, "asideRingFill", "end").parse().unwrap_or(f64::NAN);
    if !same(end, 0.75 * meter) {
        differences.push(format!("row {row}: the ring filled to {end} of the turn where the share {meter} is {}", 0.75 * meter));
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
    let got = boxes(&tree, unit);
    let k = DESIGN_HEIGHT / 1200.0;
    if !same(got["settingsHeading"].y, got["settingsCrumb"].bottom() + 16.0 * k) {
        differences.push(format!("the heading at {} under a trail ending at {}", got["settingsHeading"].y, got["settingsCrumb"].bottom()));
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
    section: "sound",
    opens: None,
    key: "sound",
    heading: "Sound",
    crumb: "",
    focus: 0,
    drawn: 4,
    board: &[
        (0, "Volume", "The game's volume.", false),
        (1, "Navigation sounds", "The sounds for moving, choosing and going back, on a stream of their own beside the game's.", true),
        (2, "Sound volume", "How loud the navigation sounds are, beside the game's own volume.", false),
        (3, "Output", "Where the game's sound and the interface's sounds play.", false),
    ],
    hidden: &[],
    unheld: &[],
    glow: "#7A3FA8",
    about: "Volume",
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
    section: "controllers",
    opens: None,
    key: "controllers",
    heading: "Controllers",
    crumb: "",
    focus: 8,
    drawn: 9,
    board: &[
        (3, "Rumble", "The motors in your controller, where the console's games drive them.", false),
        (4, "Swap A and B in menus", "A and B trade what they do in the library, the menus and every sheet, and so do X and Y.", true),
        (5, "Guide chords", "Guide held with Up saves, with Down loads, with R1 takes a screenshot.", false),
        (8, "Controller type", "Which buttons the help bar draws.", false),
        (9, "Only the first controller", "Only the first controller connected steers the menus.", false),
        (10, "Controller notices", "A notice at the top of the screen when a controller is connected or disconnected.", true),
        (11, "Left stick as the D-pad", "In games whose console has no stick, the left stick also presses the D-pad.", true),
        (12, "Stick deadzone", "How far a stick must travel before it counts as a direction.", false),
    ],
    // R-20261112-13: the On-screen keyboard row waits for the Steam keyboard call.
    hidden: &["On-screen keyboard"],
    // Not held until its cause is found: B from a bindings page (Controller bindings, Keyboard keys) lands on the first row, where Hotkeys' lands on its own.
    unheld: &[("Controller bindings", "B from a bindings page lands on the first row, a defect not traced to its line"), ("Keyboard keys", "B from a bindings page lands on the first row, a defect not traced to its line")],
    glow: "#2A6FD6",
    about: "Controller type",
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
    section: "library",
    opens: Some("collections"),
    key: "group:Collections",
    heading: "Collections",
    crumb: "Library",
    focus: 4,
    drawn: 7,
    board: &[
        (0, "Group your collections", "If unthemed puts a collection the theme has no folder for inside Collections.", true),
        (1, "Favourites on top", "Favourite games sorted above the rest.", false),
        (2, "Favourites on top in your collections", "The same, in your own collections.", false),
        (3, "Stars in your collections", "A star beside a favourite in your own collections.", false),
        (4, "Default sort order", "A list's own choice in its options lasts until EmuSen closes.", true),
        (5, "Folders on top", "A console's folders above its games, as ES-DE sorts them.", false),
        (6, "Random entry button", "Either thumbstick pressed in jumps to a random game, or console.", true),
    ],
    hidden: &[],
    unheld: &[],
    glow: "#1FA890",
    about: "Default sort order",
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
    section: "emusen",
    opens: Some("saves"),
    key: "group:Saves and states",
    heading: "Saves and states",
    crumb: "EmuSen",
    focus: 0,
    drawn: 2,
    board: &[
        (0, "Continue where you left off", "Every game is saved when it is closed; this is what happens the next time it starts.", true),
        (1, "Play time counted up to", "A launch longer than this adds nothing to the game's play time.", false),
    ],
    hidden: &[],
    unheld: &[],
    glow: "#B0487A",
    about: "Continue where you left off",
    // R-20261123-37: the third choice is the board's word.
    muted: &["Ask each time · Always resume · Always restart"],
    help: &["Change", "Choose or open", "Back to the default", "Back", "Home"],
};

/// The Saves and states page's oracle, and a trail of three levels, on `backend`.
pub fn saves(backend: &str) {
    oracle(&SAVES, backend, &|scene, sdl, pad, differences| three_levels(scene, sdl, pad, &SAVES, backend, differences));
}
