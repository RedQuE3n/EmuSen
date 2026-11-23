//! The oracle of Eclipse's Settings on its Display page in the running program, driven by an SDL pad in this process and by the pointer, and the means the other pages' oracles share (EmuSen_Qt.md §3.4.11).
#![allow(dead_code)]

use crate::common::wait_for;
use crate::eclipse_board::{held, object, read, rows, Row};
use crate::eclipse_pad::{focused, press};
use cxx_qt::casting::Upcast;
use emusen_client_fixture::{library, World};
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::input::PadControl;
use emusen_kit_core::settings::RowKind;
use emusen_kit_core::tokens::Colour;
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_models::library_model::qobject::QAbstractListModel;
use emusen_qt_models::settings_model::qobject::SettingsModel;
use emusen_qt_models::themes;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::virtual_pad::{Sdl, VirtualPad};
use emusen_qt_test::Scene;
use emusen_theme_qt::host;

const BOARD: &str = include_str!("../eclipse/settings_display.tsv");

/// The board's nine rows: name and short line; the values are the program's, which the board's samples show (EclipseSettings.dc.html's renderVals, gen_settings.py:153-161).
const BOARD_ROWS: [(&str, &str); 9] = [
    ("Picture shape", "The screen each console was made for, or each of the frame's pixels square."),
    ("CRT shader", "The television's picture for every console that had one."),
    ("Shader levers", "The CRT's own levers for one console: the mask, the bloom, the scanlines and the glass."),
    ("Shaders", "The shader each console draws with: the built-in filters, the recent ones and the slang pack's presets."),
    ("Crop overscan", "What is hidden at the picture's edges on a console that had a television."),
    ("Text size", "Every word on the big screen, and the desktop's, through the interface's type scale."),
    ("Background art on Home", "The focused game's art behind Home, dimmed and blurred."),
    ("Reduced motion", "No movement, shorter fades. System follows the desktop's own setting."),
    ("Start-up splash", "The eclipse while big picture starts. Any button skips it."),
];

/// The board's section icons in its order, each a path in a 24-unit box (gen_settings.py:7-14).
const BOARD_ICONS: [&str; 6] = [
    "M6.5 8h11a4.5 4.5 0 0 1 0 9c-1.6 0-2.4-1.5-3.5-1.5h-4c-1.1 0-1.9 1.5-3.5 1.5a4.5 4.5 0 0 1 0-9zM8 11v3M6.5 12.5h3M16 12.5h.01",
    "M4 5h16v11H4zM9 20h6M12 16v4",
    "M5 10v4h3l5 4V6L8 10zM16.5 9a4 4 0 0 1 0 6M19 6.5a8 8 0 0 1 0 11",
    "M4 8h16v9H4zM8 17v3M16 17v3M8 12h3M15 12h.01",
    "M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z",
    "M17 4.5a8.5 8.5 0 1 0 0 15 6.5 6.5 0 1 1 0-15z",
];

/// The window's SettingsModel, as its own type.
pub fn settings(scene: &mut Scene<'_>) -> &'static SettingsModel {
    let object = scene.object("settingsModel");
    assert!(!object.is_null(), "the window's settings model");
    // Safety: the window's model of that name is a SettingsModel, its list model's QObject at its address, alive while the window is.
    unsafe { &*<SettingsModel as Upcast<QAbstractListModel>>::from_base_ptr(object as *const QAbstractListModel) }
}

/// The focused row and the page as the model holds them.
pub fn at(scene: &mut Scene<'_>) -> (i32, String) {
    let s = settings(scene);
    (*s.focused(), s.view().page_key())
}

/// The row's values as the model holds them: its value and, for a switch, whether it is on.
pub fn state(scene: &mut Scene<'_>, row: usize) -> (String, Option<bool>) {
    let s = settings(scene);
    (s.shown_rows()[row].value.clone(), s.view().facets()[row].switch)
}

/// The centre of a drawn element by name, in the window's pixels.
pub fn centre(scene: &mut Scene<'_>, name: &str) -> (f64, f64) {
    let tree = scene.tree();
    let n = tree.nodes.iter().rev().find(|n| n.name == name && n.visible).unwrap_or_else(|| panic!("{name} is drawn"));
    (n.x + n.w / 2.0, n.y + n.h / 2.0)
}

/// Whether a row is drawn: inside the rows' window.
pub fn shown(scene: &mut Scene<'_>, name: &str) -> bool {
    scene.tree().nodes.iter().any(|n| n.name == name && n.visible)
}

/// The help line's words as drawn, left to right, and whether X's entry is drawn taken.
pub fn help_line(scene: &mut Scene<'_>) -> (Vec<String>, String) {
    let tree = scene.tree();
    let mut entries: Vec<(f64, String)> = tree.nodes.iter().filter(|n| n.visible && n.name.starts_with("help-")).map(|n| (n.x, n.name["help-".len()..].to_owned())).collect();
    entries.sort_by(|a, b| a.0.total_cmp(&b.0));
    entries.dedup_by(|a, b| a.1 == b.1);
    (entries.into_iter().map(|e| e.1).collect(), read(&tree, "help-Back to the default", "enabled"))
}

/// Session.frame as the program sets it: the game's own picture item while a game is suspended (`ThemeHost.qml`'s showSession), else none.
fn paused_frame(scene: &mut Scene<'_>, paused: bool) {
    let tree = scene.tree();
    let id = |name: &str| -> u64 { tree.nodes[tree.find_named(name).unwrap_or_else(|| panic!("{name} in the tree")) as usize].id.parse().unwrap() };
    let frame = if paused { id("picture") as *mut _ } else { std::ptr::null_mut() };
    host::session(object(id("settingsScreen"))).expect("Session").show_frame(frame);
    settle(scene);
}

/// Waits a moment for the model to answer a press or a click.
pub fn settle(scene: &mut Scene<'_>) {
    scene.wait(120);
}

/// What a row's own action does to it, undone again, by `act` (a press or a click); the differences pushed.
pub fn acted(scene: &mut Scene<'_>, page: &str, i: usize, how: &str, differences: &mut Vec<String>, mut act: impl FnMut(&mut Scene<'_>, i32), back: &mut dyn FnMut(&mut Scene<'_>)) {
    let kind = settings(scene).shown_rows()[i].kind;
    let before = state(scene, i);
    match kind {
        RowKind::Opens | RowKind::Asks => {
            act(scene, 0);
            settle(scene);
            if at(scene).1 == page {
                differences.push(format!("{how}: row {i}'s door opened nothing"));
            }
            back(scene);
            settle(scene);
            if at(scene) != (i as i32, page.to_owned()) {
                differences.push(format!("{how}: B from row {i}'s page left {:?}", at(scene)));
            }
        }
        _ if before.1.is_some() => {
            act(scene, 0);
            settle(scene);
            let turned = state(scene, i);
            act(scene, 0);
            settle(scene);
            if turned.1 == before.1 || state(scene, i) != before {
                differences.push(format!("{how}: row {i}'s switch went {before:?} → {turned:?} → {:?}", state(scene, i)));
            }
        }
        RowKind::Steps | RowKind::Slider => {
            act(scene, 1);
            settle(scene);
            let stepped = state(scene, i);
            act(scene, -1);
            settle(scene);
            if stepped == before || state(scene, i) != before {
                differences.push(format!("{how}: row {i} stepped {before:?} → {stepped:?} → {:?}", state(scene, i)));
            }
        }
        _ => differences.push(format!("{how}: row {i} is a {kind:?}, which {page} has none of")),
    }
}

/// Eclipse's Settings on Display in the program: the board's table at every gate size, the words, every row by pad and pointer, the sections, B (EmuSen_Qt.md §3.4.11).
pub fn oracle(backend: &str) {
    with_settings(|scene, sdl, pad| display(scene, sdl, pad, backend));
}

/// The program in a made-up world with one NES game and the board's battery, Eclipse chosen on Settings by the pad, `f` run there on its first section's first row.
pub fn with_settings(f: impl FnOnce(&mut Scene<'_>, &Sdl, &VirtualPad)) {
    std::env::set_var("LC_ALL", "en_US.UTF-8");
    let world = World::new("qt-app-eclipse-settings");
    let roms = world.dir("Roms/NES");
    let mut image = b"NES\x1a\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
    image.resize(16 + 16384, 0);
    std::fs::write(roms.join("One.nes"), image).unwrap();
    let platform = Platform::load(library()).unwrap();
    let mut stored = platform.model_new(MODEL_APP_SETTINGS).unwrap();
    stored["RomDirectory"] = serde_json::json!(world.root.join("Roms").to_str().unwrap());
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &stored).unwrap();
    // The machine's battery at the board's 82%, discharging, in the world's made-up power_supply tree (G:98).
    let power = world.power_supply().join("BAT0");
    std::fs::create_dir_all(&power).unwrap();
    for (key, value) in [("type", "Battery"), ("capacity", "82"), ("status", "Discharging")] {
        std::fs::write(power.join(key), format!("{value}\n")).unwrap();
    }
    let sdl = Sdl::open();
    let pad = sdl.attach("EmuSen Test Xbox Pad", 0x045e, 0x028e);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "onlyPads", &pad.id.to_string()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "settingsModel", "ready", "true", 5000);
    wait_for(&mut scene, "pads", "connected", "1", 5000);
    scene.run_pad_clock();

    // Home's Settings place by the pad, then Eclipse chosen over it, which places its own Settings.
    for _ in 0..40 {
        if focused(&mut scene) == "place.0" {
            break;
        }
        scene.wait(50);
    }
    for _ in 0..5 {
        press(&mut scene, &sdl, &pad, PadControl::Right);
    }
    press(&mut scene, &sdl, &pad, PadControl::South);
    wait_for(&mut scene, "screens", "shown", "Settings", 2000);
    assert!(scene.set_property("program", "theme", &themes::built_in_folder("eclipse")));
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);
    for _ in 0..40 {
        if scene.property("theme", "settings") == "true" {
            break;
        }
        scene.wait(50);
    }
    assert_eq!(scene.property("theme", "settings"), "true", "Eclipse draws Settings: {} {}", scene.property("theme", "warnings"), scene.property("host.theme", "warnings"));
    assert_eq!(scene.property("settingsSlot", "active"), "false", "the program's Settings is not made under Eclipse's");
    for _ in 0..40 {
        if focused(&mut scene).starts_with("row.") {
            break;
        }
        scene.wait(50);
    }
    assert_eq!(focused(&mut scene), "row.0", "Settings opens on its first section's first row");
    assert_eq!(at(&mut scene), (0, "controllers".to_owned()));
    f(&mut scene, &sdl, &pad);
}

/// Display at the board's state, held to its table, its words, its rows by pad and pointer, its sections and B.
fn display(mut scene: &mut Scene<'_>, sdl: &Sdl, pad: &VirtualPad, backend: &str) {

    // RB to Display, Down to CRT shader: the board's state.
    press(&mut scene, &sdl, &pad, PadControl::RightShoulder);
    settle(&mut scene);
    assert_eq!(at(&mut scene), (0, "display".to_owned()), "RB steps to Display");
    press(&mut scene, &sdl, &pad, PadControl::Down);
    settle(&mut scene);
    assert_eq!(at(&mut scene), (1, "display".to_owned()), "Down focuses CRT shader");
    assert_eq!(focused(&mut scene), "row.1");
    let count = settings(&mut scene).shown_rows().len();
    let mut differences = Vec::new();
    // The preview (R-20261116-25): no picture and no caption while no game is suspended; the board's state has one paused.
    if shown(&mut scene, "asidePicture") || shown(&mut scene, "asideCaption") {
        differences.push("the preview is drawn with no game suspended".to_owned());
    }
    paused_frame(&mut scene, true);
    if !shown(&mut scene, "asidePicture") || !shown(&mut scene, "asidePictureFrame") || !shown(&mut scene, "asideCaption") {
        differences.push("the paused game's frame is not drawn in the preview".to_owned());
    }

    // The board's table at every gate size.
    let (board_height, table) = rows(BOARD);
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
        // The page's glow in Display's colour (G:163).
        if Colour::parse(&read(&tree, "backdropField.0", "paint")) != Colour::parse("#3B4AA8") {
            differences.push(format!("{at_size}: the page's glow is {} where the board gives #3B4AA8", read(&tree, "backdropField.0", "paint")));
        }
        // The words: the board's, where the program's are the board's; the rows' the model's.
        let s = settings(&mut scene);
        let model_rows = s.shown_rows().to_vec();
        let facets = s.view().facets();
        let sections: Vec<String> = s.sections().iter().map(|t| t.to_string()).collect();
        let mut words = vec![
            ("settingsTitle".to_owned(), "Settings".to_owned()),
            ("settingsHeading".to_owned(), "Display".to_owned()),
            ("clock".to_owned(), read(&tree, "clock", "text")),
            ("battery".to_owned(), "82%".to_owned()),
            ("settingsPlace".to_owned(), format!("2 of {count}")),
            ("asideHeading".to_owned(), "CRT shader".to_owned()),
            ("asideSentence".to_owned(), s.sentence().to_string()),
            ("asideCaption".to_owned(), s.caption().to_string()),
            ("asideMuted.0".to_owned(), "For every console that had a television. A console can choose its own in Consoles.".to_owned()),
        ];
        if sections != ["Controllers", "Display", "Sound", "Consoles", "Library", "EmuSen"] {
            differences.push(format!("{at_size}: the sections are {sections:?} where the board names Controllers, Display, Sound, Consoles, Library, EmuSen"));
        }
        for (i, name) in sections.iter().enumerate() {
            words.push((format!("sectionWords.{i}"), name.clone()));
            words.push((format!("sectionIcon.{i}"), BOARD_ICONS[i].to_owned()));
        }
        for (i, (name, about)) in BOARD_ROWS.iter().enumerate() {
            words.push((format!("rowName.{i}"), name.to_string()));
            words.push((format!("rowAbout.{i}"), about.to_string()));
            words.push((format!("rowWords.{i}"), model_rows[i].value.clone()));
        }
        for (name, want) in words {
            let have = read(&tree, &name, if name.starts_with("sectionIcon.") { "path" } else { "text" });
            if have != want {
                differences.push(format!("{at_size}: {name} says {have:?} where {want:?}"));
            }
        }
        for (i, row) in model_rows.iter().enumerate() {
            if row.label != read(&tree, &format!("rowName.{i}"), "text") || facets[i].about != read(&tree, &format!("rowAbout.{i}"), "text") {
                differences.push(format!("{at_size}: row {i} is drawn {:?} where the model has {:?}", read(&tree, &format!("rowName.{i}"), "text"), row.label));
            }
        }
        // The board's rows are drawn, the program's six past them wait below the window; nothing drawn is cut but the board's short lines.
        for i in 0..count {
            if shown(&mut scene, &format!("row.{i}")) != (i < BOARD_ROWS.len()) {
                differences.push(format!("{at_size}: row {i} drawn {}", !(i < BOARD_ROWS.len())));
            }
        }
    }
    scene.resize(1920, 1200);
    scene.wait(300);
    // The help line: the board's four, then Section and Home (G:24, R-20261123-20); X taken on CRT shader, declined on a door; ◀ ▶ one key.
    let board_line = ["Change", "Choose or open", "Back to the default", "Back", "Section", "Home"].map(str::to_owned).to_vec();
    if help_line(&mut scene) != (board_line.clone(), "true".to_owned()) {
        differences.push(format!("the help line on CRT shader is {:?}", help_line(&mut scene)));
    }
    let tree = scene.tree();
    let unit: f64 = read(&tree, "settingsCanvas", "unit").parse().expect("the canvas's unit");
    let key = crate::eclipse_board::boxes(&tree, unit)[&"ownGlyph-Change".to_owned()];
    if shown(&mut scene, "ownGlyph2-Change") || !crate::eclipse_board::same(key.h, 34.0 * k) {
        differences.push(format!("◀ ▶ is not one key 34 px high: a second glyph {}, {} high", shown(&mut scene, "ownGlyph2-Change"), key.h / k));
    }
    press(&mut scene, &sdl, &pad, PadControl::Down);
    settle(&mut scene);
    if help_line(&mut scene) != (board_line.clone(), "false".to_owned()) {
        differences.push(format!("the help line on Shader levers, a door, is {:?}", help_line(&mut scene)));
    }
    press(&mut scene, &sdl, &pad, PadControl::Up);
    settle(&mut scene);
    pictures(&mut scene, backend, "display-crt");
    // On CRT shader still, the game no longer suspended: the preview and its caption go.
    paused_frame(&mut scene, false);
    if shown(&mut scene, "asidePicture") || shown(&mut scene, "asideCaption") {
        differences.push("the preview stays with no game suspended".to_owned());
    }


    // The pad: Up to the first row, Down through every row to the last, each focused and in the window, then back up.
    press(&mut scene, &sdl, &pad, PadControl::Up);
    for i in 0..count {
        if i > 0 {
            press(&mut scene, &sdl, &pad, PadControl::Down);
        }
        settle(&mut scene);
        if at(&mut scene).0 != i as i32 || focused(&mut scene) != format!("row.{i}") || !shown(&mut scene, &format!("row.{i}")) {
            differences.push(format!("Down to row {i}: the model at {:?}, the focus on {:?}, drawn {}", at(&mut scene), focused(&mut scene), shown(&mut scene, &format!("row.{i}"))));
        }
        if read(&scene.tree(), "settingsPlace", "text") != format!("{} of {count}", i + 1) {
            differences.push(format!("row {i}: the place reads {:?}", read(&scene.tree(), "settingsPlace", "text")));
        }
    }
    press(&mut scene, &sdl, &pad, PadControl::Down);
    settle(&mut scene);
    if at(&mut scene).0 != count as i32 - 1 {
        differences.push("Down past the last row moved the focus".to_owned());
    }
    pictures(&mut scene, backend, "display-last");
    // The program's rows past the board's nine, now in the window, each held by the table's rows of the board's row of its form.
    for (width, height) in SIZES {
        scene.resize(width, height);
        scene.wait(300);
        let tree = scene.tree();
        let at_size = format!("{width}x{height}, the last row focused");
        let unit: f64 = read(&tree, "settingsCanvas", "unit").parse().expect("the canvas's unit");
        let screen: u64 = tree.nodes[tree.find_named("settingsScreen").expect("Eclipse's Settings") as usize].id.parse().unwrap();
        let tokens = crate::eclipse_board::tokens(screen);
        let s = settings(&mut scene);
        let (model_rows, facets) = (s.shown_rows().to_vec(), s.view().facets());
        let mut later = Vec::new();
        for i in BOARD_ROWS.len()..count {
            later.extend(like(&table, i, model_rows[i].kind, facets[i].switch, !facets[i].about.is_empty(), i + 1 == count));
        }
        later.extend(table.iter().filter(|r| r.name == "rowRing").map(|r| renamed(r, 0, 0)));
        held(&tree, &later, k, unit, f64::from(width) / unit, &tokens, &at_size, &|_, _, _| false, &mut differences);
        // Each later row's short line, its hint (R-20261123-31), ends 24 px before the value at the latest, cut with an ellipsis there (G:30).
        let got = crate::eclipse_board::boxes(&tree, unit);
        for i in BOARD_ROWS.len()..count {
            let (line, value) = (got[&format!("rowAbout.{i}")], got[&format!("rowValue.{i}")]);
            let cut = read(&tree, &format!("rowAbout.{i}"), "truncated") == "true";
            if line.right() > value.x - 24.0 * k + 1e-6 || (cut && line.right() < value.x - 24.0 * k - 1e-6) {
                differences.push(format!("{at_size}: row {i}'s short line ends at {} with the value at {}, cut {cut}", line.right(), value.x));
            }
        }
    }
    scene.resize(1920, 1200);
    scene.wait(300);
    // Each row acted on by the pad: Right then Left, A twice, or A and B.
    for i in (0..count).rev() {
        if i + 1 < count {
            press(&mut scene, &sdl, &pad, PadControl::Up);
            settle(&mut scene);
        }
        if at(&mut scene).0 != i as i32 {
            let walk = (scene.property("settingsWalk", "current"), scene.property("settingsWalk", "count"));
            differences.push(format!("Up to row {i}: the model at {:?}, the focus on {:?}, the walk at {walk:?}", at(&mut scene), focused(&mut scene)));
            break;
        }
        let (sdl_ref, pad_ref): (&Sdl, &VirtualPad) = (&sdl, &pad);
        acted(
            &mut scene,
            "display",
            i,
            "the pad",
            &mut differences,
            |scene, by| press(scene, sdl_ref, pad_ref, if by > 0 { PadControl::Right } else if by < 0 { PadControl::Left } else { PadControl::South }),
            &mut |scene| press(scene, sdl_ref, pad_ref, PadControl::East),
        );
        // X: a value back to its default after a change; a door's row left as it is.
        let defaultable = settings(&mut scene).view().defaultable(i);
        press(&mut scene, &sdl, &pad, PadControl::West);
        settle(&mut scene);
        let default = state(&mut scene, i);
        if defaultable {
            let change = if default.1.is_some() { PadControl::South } else { PadControl::Right };
            press(&mut scene, &sdl, &pad, change);
            settle(&mut scene);
            let changed = state(&mut scene, i);
            press(&mut scene, &sdl, &pad, PadControl::West);
            settle(&mut scene);
            if changed == default || state(&mut scene, i) != default {
                differences.push(format!("X on row {i}: {default:?} → {changed:?} → {:?}", state(&mut scene, i)));
            }
        } else if at(&mut scene) != (i as i32, "display".to_owned()) {
            differences.push(format!("X on row {i}, a door, left {:?}", at(&mut scene)));
        }
    }

    assert!(differences.is_empty(), "{} differences by the pad:\n{}", differences.len(), differences.join("\n"));
    // The pointer: each row focused where it is drawn, the wheel bringing later rows in; chevrons step, a click turns a switch or opens a door.
    for i in 0..count {
        for _ in 0..count {
            if shown(&mut scene, &format!("row.{i}")) {
                break;
            }
            let (x, y) = centre(&mut scene, "settingsRows");
            scene.wheel(x, y, -1);
            settle(&mut scene);
        }
        // The pointer moved onto the row focuses it; a click is A.
        let (x, y) = centre(&mut scene, &format!("rowName.{i}"));
        scene.hover(x + 1.0, y);
        scene.hover(x, y);
        settle(&mut scene);
        if at(&mut scene).0 != i as i32 {
            differences.push(format!("the pointer on row {i} focused {:?}", at(&mut scene)));
            continue;
        }
        let (sdl_ref, pad_ref): (&Sdl, &VirtualPad) = (&sdl, &pad);
        acted(
            &mut scene,
            "display",
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
            &mut |scene| press(scene, sdl_ref, pad_ref, PadControl::East),
        );
    }

    // The sections: RB and LB, and a click on Sound and on Display.
    press(&mut scene, &sdl, &pad, PadControl::RightShoulder);
    settle(&mut scene);
    let sound = at(&mut scene).1;
    press(&mut scene, &sdl, &pad, PadControl::LeftShoulder);
    settle(&mut scene);
    let display = at(&mut scene).1;
    let (x, y) = centre(&mut scene, "section.2");
    scene.click(x, y);
    settle(&mut scene);
    let clicked = at(&mut scene).1;
    let (x, y) = centre(&mut scene, "section.1");
    scene.click(x, y);
    settle(&mut scene);
    if (sound.as_str(), display.as_str(), clicked.as_str(), at(&mut scene).1.as_str()) != ("sound", "display", "sound", "display") {
        differences.push(format!("the sections went {sound}, {display}, {clicked}, {:?}", at(&mut scene)));
    }
    let tree = scene.tree();
    if read(&tree, "settingsHeading", "text") != "Display" || Colour::parse(&read(&tree, "section.1", "color")) != Colour::parse("#F6E7D8") {
        differences.push("Display's heading and its filled link after the clicks".to_owned());
    }

    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
    // B on a section's page leaves Settings for the screen it came from.
    press(&mut scene, &sdl, &pad, PadControl::East);
    wait_for(&mut scene, "screens", "shown", "Home", 2000);
    scene.assert_quiet();
}

/// A table row with every element of the board's row `from` named for row `to`, the row before it for the row before `to`.
pub fn renamed(r: &Row, from: usize, to: usize) -> Row {
    let spec = |s: &str| {
        s.split(':')
            .map(|t| match t.rsplit_once('.').and_then(|(n, i)| Some((n, i.parse::<usize>().ok()?))) {
                Some((n, i)) if n.starts_with("row") && i == from => format!("{n}.{to}"),
                Some((n, i)) if n.starts_with("row") && i + 1 == from => format!("{n}.{}", to - 1),
                _ => t.to_owned(),
            })
            .collect::<Vec<_>>()
            .join(":")
    };
    Row {
        name: spec(&r.name),
        part: r.part.clone(),
        x: spec(&r.x),
        y: spec(&r.y),
        w: spec(&r.w),
        h: spec(&r.h),
        radius: r.radius.clone(),
        colour: r.colour.clone(),
        size: r.size.clone(),
        status: r.status.clone(),
        kind: r.kind.clone(),
    }
}

/// Row `i`'s table rows by the board's frame, the rule of its form's board row (EmuSen_Qt.md §3.4.11, the proof).
pub fn like(table: &[Row], i: usize, kind: RowKind, switch: Option<bool>, about: bool, focused: bool) -> Vec<Row> {
    let from = match (switch, kind) {
        (Some(_), _) => 6,
        (None, RowKind::Steps | RowKind::Slider) => 4,
        _ => 3,
    };
    let suffix = format!(".{from}");
    let mut out: Vec<Row> = table.iter().filter(|r| r.name.ends_with(&suffix) && r.name.starts_with("row")).map(|r| renamed(r, from, i)).collect();
    // A short line's width is its words', cut at the value as the oracle's own check holds; the board row's width is its words'.
    out.iter_mut().filter(|r| r.name.starts_with("rowAbout.")).for_each(|r| r.w = "-".to_owned());
    if !about {
        out.retain(|r| !r.name.starts_with("rowAbout."));
        out.iter_mut().filter(|r| r.name.starts_with("rowName.")).for_each(|r| r.y = format!("mid:row.{i}"));
    }
    if focused {
        let lit = table.iter().find(|r| r.name == "rowFill.1").expect("the focused row's fill");
        out.iter_mut().filter(|r| r.name.starts_with("rowFill.")).for_each(|r| (r.colour, r.kind) = (lit.colour.clone(), lit.kind.clone()));
    }
    if switch == Some(false) {
        for r in out.iter_mut() {
            if r.name.starts_with("rowSwitch.") {
                (r.colour, r.kind) = ("#00000000".to_owned(), "line 2 #FF5A6076".to_owned());
            } else if r.name.starts_with("rowKnob.") {
                (r.x, r.colour) = (format!("in:rowSwitch.{i}:6"), "Tokens.muted".to_owned());
            }
        }
    }
    out
}

/// With `EMUSEN_ECLIPSE_PICTURES` naming a folder, the screen grabbed at the board's size into it.
pub fn pictures(scene: &mut Scene<'_>, backend: &str, what: &str) {
    let Some(folder) = std::env::var_os("EMUSEN_ECLIPSE_PICTURES") else { return };
    std::fs::create_dir_all(&folder).unwrap();
    scene.resize(1920, 1200);
    scene.wait(500);
    let path = std::path::Path::new(&folder).join(format!("settings-{what}-1920x1200-{backend}.png"));
    assert!(scene.grab(&path.display().to_string()), "{} is grabbed", path.display());
}
