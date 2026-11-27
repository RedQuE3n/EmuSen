//! Shared oracle for Eclipse Wave 6 sys1: Dialogs/Notices (#14) and Power (#15) (R-20261113-5, R-20261115-2).
#![allow(dead_code)]

use crate::common::{draw_covers, sandbox, wait_for};
use crate::eclipse_board::{boxes, held, object, painted, read, rows, same, tokens, Row};
use cxx_qt_lib::QString;
use emusen_client_fixture::library;
use emusen_kit_core::canvas::DESIGN_HEIGHT;
use emusen_kit_core::input::UiButton;
use emusen_kit_core::tokens::Colour;
use emusen_platform_client::galaxia::consts::{FILE_GAMES, MODEL_APP_SETTINGS, SAVE_PICTURE, SAVE_STATE};
use emusen_platform_client::library::Library;
use emusen_platform_client::{Lent, Platform};
use emusen_qt::MAIN_URL;
use emusen_qt_models::{home_card, words};
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::{Harness, Scene};
use serde_json::json;
use std::path::Path;

const ECLIPSE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../themes/eclipse");
const DAY: i64 = 86_400;
const TABLE_DIALOG: &str = include_str!("../../../qt-app/tests/eclipse/dialog.tsv");
const TABLE_POWER: &str = include_str!("../../../qt-app/tests/eclipse/power.tsv");

fn midnight() -> i64 {
    words::now().div_euclid(DAY) * DAY
}

fn records(platform: &std::sync::Arc<Platform>) -> Library {
    let sqlite = Lent::open("libsqlite3.so.0").unwrap();
    Library::open(platform, &platform.file(FILE_GAMES).unwrap(), &sqlite).unwrap()
}

fn theme_object(scene: &mut Scene<'_>, name: &str) -> u64 {
    let tree = scene.tree();
    tree.nodes[tree.find_named(name).unwrap_or_else(|| panic!("{name} is placed")) as usize].id.parse().unwrap()
}

fn words_under(tree: &emusen_qt_test::tree::Tree, name: &str) -> Vec<String> {
    let Some(top) = tree.nodes.iter().rposition(|n| n.name == name && n.visible) else { return Vec::new() };
    let top = tree.nodes[top].id.clone();
    let under = |n: &emusen_qt_test::tree::Node| {
        let mut at = n.parent.clone();
        while let Some(p) = tree.nodes.iter().find(|p| p.id == at) {
            if p.id == top {
                return true;
            }
            at = p.parent.clone();
        }
        false
    };
    tree.nodes.iter().filter(|n| n.visible && n.ty.contains("Text") && under(n)).filter_map(|n| n.text.clone()).filter(|t| !t.is_empty()).collect()
}

fn glyphs(tree: &emusen_qt_test::tree::Tree, names: &[&str]) -> Vec<String> {
    names
        .iter()
        .map(|g| {
            let g = tree.nodes.iter().rposition(|n| n.name == *g && n.visible);
            let child = g.and_then(|g| tree.nodes.iter().position(|n| n.parent == tree.nodes[g].id));
            child
                .map(|c| {
                    let id: u64 = tree.nodes[c].id.parse().unwrap();
                    let p = |q: &str| emusen_qt_test::ffi::ffi::harness_property(object(id), &QString::from(tree.nodes[c].name.as_str()), &QString::from(q)).to_string();
                    if p("named") == "true" { p("word") } else { p("letter") }
                })
                .unwrap_or_default()
        })
        .collect()
}

fn hold_screen(
    scene: &mut Scene<'_>,
    canvas: &str,
    screen: &str,
    table: &str,
    centered: &[&str],
    words_want: &[(&str, &str)],
    help_name: &str,
    help: &[&str],
    help_glyphs: &[&str],
    before_size: &mut dyn FnMut(&mut Scene<'_>),
    differences: &mut Vec<String>,
) {
    let tokens = tokens(theme_object(scene, screen));
    let (board_height, all_rows) = rows(table);
    let k = DESIGN_HEIGHT / board_height;
    for (width, height) in SIZES {
        scene.resize(width, height);
        before_size(scene);
        scene.wait(300);
        let tree = scene.tree();
        let unit: f64 = read(&tree, canvas, "unit").parse().expect("canvas unit");
        let canvas_width = width as f64 / unit;
        let at = format!("{canvas} {width}x{height}");
        let (present, missing): (Vec<Row>, Vec<Row>) = all_rows.iter().cloned().partition(|r| r.status != "built" || tree.find_named(&r.name).is_some());
        differences.extend(missing.iter().map(|r| format!("{at}: {} is not drawn", r.name)));
        held(&tree, &present, k, unit, canvas_width, &tokens, &at, &|_, _, _| false, differences);
        let got = boxes(&tree, unit);
        for name in centered {
            if let Some(b) = got.get(*name) {
                let want_x = (canvas_width - b.w) / 2.0;
                if !same(b.x, want_x) {
                    differences.push(format!("{at}: {name} x {} not centered at {want_x}", b.x));
                }
            }
        }
        for (name, want) in words_want {
            let have = read(&tree, name, "text");
            if have != *want {
                differences.push(format!("{at}: {name} says {have:?} where the board says {want:?}"));
            }
        }
        let line = words_under(&tree, help_name);
        if line != help {
            differences.push(format!("{at}: {help_name} says {line:?} where the board says {help:?}"));
        }
        let gnames: Vec<String> = help.iter().map(|w| format!("ownGlyph-{w}")).collect();
        let letters = glyphs(&tree, &gnames.iter().map(String::as_str).collect::<Vec<_>>());
        if letters != help_glyphs {
            differences.push(format!("{at}: {help_name} glyphs read {letters:?} where the board draws {help_glyphs:?}"));
        }
    }
    scene.resize(1280, 800);
    scene.wait(250);
}

fn picture(scene: &mut Scene<'_>, name: &str, prepare: impl FnOnce(&mut Scene<'_>)) {
    let Some(folder) = std::env::var_os("EMUSEN_ECLIPSE_PICTURES") else { return };
    std::fs::create_dir_all(&folder).unwrap();
    scene.resize(1920, 1200);
    prepare(scene);
    scene.wait(450);
    let path = Path::new(&folder).join(name);
    assert!(scene.grab(&path.display().to_string()), "{} is grabbed", path.display());
    scene.resize(1280, 800);
    scene.wait(250);
}

fn raise_board_notice(scene: &mut Scene<'_>, folder: &str) {
    let head = "Tower Set could not start";
    let body = "Its second disc is missing: Tower Set (Disc 2).bin is named in the sheet and is not in the folder. Nothing was changed.";
    let presenter = scene.object("noticePresenter");
    assert!(!presenter.is_null(), "noticePresenter");
    let presenter = unsafe {
        std::pin::Pin::new_unchecked(&mut *(presenter as *mut emusen_qt_models::notice_presenter::qobject::NoticePresenter))
    };
    presenter.present(&QString::from(head), &QString::from(body), true, &QString::from("Open its folder"), &QString::from(folder));
    let notices = scene.object("notices");
    assert!(!notices.is_null(), "notices");
    let notices = unsafe {
        std::pin::Pin::new_unchecked(&mut *(notices as *mut emusen_kit_qt::notices::qobject::NoticeModel))
    };
    notices.raise(&QString::from(format!("{head}\n{body}").as_str()), true, false);
}

fn raise_plain_notice(scene: &mut Scene<'_>, words: &str, attention: bool) {
    let notices = scene.object("notices");
    assert!(!notices.is_null(), "notices");
    let notices = unsafe {
        std::pin::Pin::new_unchecked(&mut *(notices as *mut emusen_kit_qt::notices::qobject::NoticeModel))
    };
    notices.raise(&QString::from(words), attention, false);
}

fn expire_notice(scene: &mut Scene<'_>) {
    let notices = scene.object("notices");
    assert!(!notices.is_null(), "notices");
    let notices = unsafe {
        std::pin::Pin::new_unchecked(&mut *(notices as *mut emusen_kit_qt::notices::qobject::NoticeModel))
    };
    notices.expire();
    wait_for(scene, "notices", "shown", "false", 2000);
}

fn pads_button(scene: &mut Scene<'_>, button: UiButton) {
    let pads = scene.object("pads");
    assert!(!pads.is_null(), "pads");
    let raw = emusen_kit_qt::kinds::pad_button(button);
    let mut pads = unsafe {
        std::pin::Pin::new_unchecked(&mut *(pads as *mut emusen_qt_endymion::pad_reader::qobject::PadReader))
    };
    pads.as_mut().button(raw, true);
    pads.as_mut().button(raw, false);
    scene.wait(120);
}

fn folders_caught(scene: &mut Scene<'_>) -> Vec<String> {
    let folders = scene.object("folders");
    assert!(!folders.is_null(), "folders");
    let folders = unsafe { &*(folders as *const emusen_qt_models::folders::ffi::Folders) };
    folders.caught().iter().map(|s| s.to_string()).collect()
}

fn set_state_time(path: &Path, secs: i64) {
    let file = std::fs::OpenOptions::new().write(true).open(path).unwrap();
    let time = std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs.max(0) as u64);
    file.set_modified(time).unwrap();
}

/// Runs both Wave 6 sys1 boards (#14 Dialog/Notice and #15 Power) and interactive states on the active backend.
pub fn sys1(backend: &str) {
    std::env::set_var("LC_ALL", "en_US.UTF-8");
    std::env::set_var("TZ", "UTC");
    let world = sandbox(&format!("qt-app-eclipse-sys1-{backend}"), &[("SNES", "Aurora Drift.sfc")]);
    let cores = world.dir("Cores");
    crate::common::approved_console_core(&cores);
    let platform = Platform::load(library()).unwrap();
    let rom = world.root.join("Roms/SNES/Aurora Drift.sfc");
    let rom_str = rom.to_string_lossy().into_owned();
    let rom_dir = rom.parent().unwrap().to_string_lossy().into_owned();
    let games = records(&platform);
    games.change(&json!({ "op": "started", "path": rom_str, "at": words::utc(words::now() - 3600) })).unwrap();
    drop(games);

    // Create Slot 1 state saved yesterday at 8:40 PM UTC with a real thumbnail PNG for board #14.
    let slot1 = platform.save_path(SAVE_STATE, &rom_str, None, 1).unwrap();
    std::fs::create_dir_all(slot1.parent().unwrap()).unwrap();
    std::fs::write(&slot1, b"state").unwrap();
    let yesterday_840 = midnight() - DAY + 20 * 3600 + 40 * 60;
    set_state_time(&slot1, yesterday_840);
    let slot1_pic = platform.save_path(SAVE_PICTURE, &slot1.to_string_lossy(), None, 0).unwrap();

    let mut harness = Harness::new();
    let media = world.dir("EsdeMedia");
    draw_covers(
        &mut harness,
        &world.root,
        &[
            (media.join("snes/covers/Aurora Drift.png"), (560, 315), 0, String::new()),
            (slot1_pic, (440, 276), 1, String::new()),
        ],
        |_| {},
    );
    let mut settings = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    settings["EsdeMediaDirectory"] = json!(media.to_str().unwrap());
    settings["OpenEmuFallback"] = json!(false);
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();

    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("library", "cores", cores.to_str().unwrap()));
    assert!(scene.set_property("program", "coverStores", media.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("program", "theme", &std::fs::canonicalize(ECLIPSE).unwrap().display().to_string()));
    assert!(scene.set_property("power", "dryRun", "true"));
    assert!(scene.set_property("folders", "catching", "true"));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "host.theme", "ready", "true", 5000);
    assert_eq!(scene.property("theme", "failed"), "false", "{}", scene.property("theme", "warnings"));
    let family = emusen_kit_qt::kinds::family_of(emusen_kit_core::glyph::PadFamily::Xbox).repr;
    assert!(scene.set_property("theme", "family", &family.to_string()));
    scene.wait(350);

    let mut differences = Vec::new();

    // --- BOARD #15: Power dialog with suspended game ---
    let power_facts = json!({
        "suspended": "Aurora Drift",
        "suspendedConsole": "Super Nintendo",
        "suspendedPicture": "",
        "paused": "Super Nintendo · Paused just now",
        "totalGames": 1,
        "inHand": "Aurora Drift",
        "inHandConsole": "Super Nintendo",
        "inHandCover": "",
        "inHandFavourite": false,
        "search": true,
        "startable": [],
        "gameMode": false,
        "tools": {},
        "unread": ""
    });
    assert!(scene.set_property("homeCardModel", "facts", &power_facts.to_string()));
    assert!(scene.set_property("homeCardModel", "start", home_card::POWER));
    scene.pad(UiButton::Menu);
    wait_for(&mut scene, "screens", "shown", "HomeCard", 3000);
    wait_for(&mut scene, "homeCardModel", "power", "true", 2000);
    scene.wait(350);

    hold_screen(
        &mut scene,
        "homeCardCanvas",
        "homeScreen",
        TABLE_POWER,
        &["powerDisc", "powerSection", "powerHelp"],
        &[
            ("powerTitle", "Power"),
            ("powerSubtitle", "Aurora Drift is suspended. It is saved before anything here happens."),
            ("powerLabel.0", "Sleep"),
            ("powerHint.0", "Wakes where you are"),
            ("powerLabel.1", "Restart"),
            ("powerLabel.2", "Shut down"),
            ("powerLabel.3", "Back to the desktop"),
            ("powerHint.3", "EmuSen keeps running"),
            ("powerLabel.4", "Quit EmuSen"),
        ],
        "powerHelp",
        &["Choose", "Back"],
        &["A", "B"],
        &mut |_| {},
        &mut differences,
    );
    picture(&mut scene, &format!("power-1920x1200-{backend}.png"), |_| {});

    // Power interaction: navigate to Restart, Shut down, Back to the desktop, and suspended Quit dialog (#14).
    scene.pad(UiButton::Down);
    scene.wait(120);
    assert_eq!(painted(&scene.tree(), "powerRowBg.1"), Colour::parse("#FFF6E7D8"), "Restart row focused");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Home", 3000);
    assert_eq!(scene.property("power", "recordedLog"), "reboot", "Restart recorded reboot verb");

    // Trigger Shut down with simulated runner error -> raises #14 failure Notice banner.
    assert!(scene.set_property("power", "failWith", "polkit refused"));
    scene.pad(UiButton::Menu);
    wait_for(&mut scene, "homeCardModel", "power", "true", 2000);
    scene.wait(250);
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Down);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Home", 3000);
    wait_for(&mut scene, "notices", "shown", "true", 2000);
    assert_eq!(scene.property("noticePresenter", "heading"), "Shut down could not run");
    assert_eq!(scene.property("noticePresenter", "body"), "polkit refused");
    assert_eq!(scene.property("noticePresenter", "icon"), "notice.warning");
    assert!(scene.set_property("power", "failWith", ""));
    scene.wait(2200);

    // Back to the desktop records desktop action.
    scene.pad(UiButton::Menu);
    wait_for(&mut scene, "homeCardModel", "power", "true", 2000);
    scene.wait(250);
    for _ in 0..3 {
        scene.pad(UiButton::Down);
    }
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Home", 3000);
    assert_eq!(scene.property("power", "recordedLog"), "reboot,poweroff,desktop", "Back to the desktop recorded");

    // Verify save-before-power ordering through live PowerModel -> SessionModel signal loop (Sleep, Desktop, Quit).
    let power_ptr = scene.object("power");
    assert!(!power_ptr.is_null(), "power");
    let power_obj = unsafe { std::pin::Pin::new_unchecked(&mut *(power_ptr as *mut emusen_qt_models::power_model::qobject::PowerModel)) };
    power_obj.clear_recorded();
    let power_obj = unsafe { std::pin::Pin::new_unchecked(&mut *(power_ptr as *mut emusen_qt_models::power_model::qobject::PowerModel)) };
    power_obj.trigger(&QString::from(home_card::POWER_SLEEP), true);
    scene.wait(50);
    assert_eq!(scene.property("power", "recordedLog"), "save:keep,suspend", "Sleep saves before suspend");

    let power_obj = unsafe { std::pin::Pin::new_unchecked(&mut *(power_ptr as *mut emusen_qt_models::power_model::qobject::PowerModel)) };
    power_obj.clear_recorded();
    let power_obj = unsafe { std::pin::Pin::new_unchecked(&mut *(power_ptr as *mut emusen_qt_models::power_model::qobject::PowerModel)) };
    power_obj.trigger(&QString::from(home_card::POWER_DESKTOP), true);
    scene.wait(50);
    assert_eq!(scene.property("power", "recordedLog"), "save:keep,desktop", "Back to the desktop saves before minimizing");

    assert!(scene.set_property("power", "failSaveWith", "disk full"));
    let power_obj = unsafe { std::pin::Pin::new_unchecked(&mut *(power_ptr as *mut emusen_qt_models::power_model::qobject::PowerModel)) };
    power_obj.clear_recorded();
    let power_obj = unsafe { std::pin::Pin::new_unchecked(&mut *(power_ptr as *mut emusen_qt_models::power_model::qobject::PowerModel)) };
    power_obj.trigger(&QString::from(home_card::POWER_DESKTOP), true);
    wait_for(&mut scene, "notices", "shown", "true", 2000);
    assert_eq!(scene.property("power", "recordedLog"), "save:keep", "failed save aborts Back to the desktop");
    assert_eq!(scene.property("noticePresenter", "heading"), "Back to the desktop cancelled");
    expire_notice(&mut scene);

    let power_obj = unsafe { std::pin::Pin::new_unchecked(&mut *(power_ptr as *mut emusen_qt_models::power_model::qobject::PowerModel)) };
    power_obj.clear_recorded();
    let power_obj = unsafe { std::pin::Pin::new_unchecked(&mut *(power_ptr as *mut emusen_qt_models::power_model::qobject::PowerModel)) };
    power_obj.finish_quit(&QString::from(""));
    wait_for(&mut scene, "notices", "shown", "true", 2000);
    assert_eq!(scene.property("power", "recordedLog"), "", "failed save aborts Quit EmuSen");
    assert_eq!(scene.property("noticePresenter", "heading"), "Quit EmuSen cancelled");
    assert!(scene.set_property("power", "failSaveWith", ""));
    expire_notice(&mut scene);

    let power_obj = unsafe { std::pin::Pin::new_unchecked(&mut *(power_ptr as *mut emusen_qt_models::power_model::qobject::PowerModel)) };
    power_obj.finish_quit(&QString::from(""));
    scene.wait(50);
    assert_eq!(scene.property("power", "recordedLog"), "quit", "successful save completes Quit EmuSen");

    // Suspended Quit EmuSen opens circular #14 confirmation dialog; B returns to Power.
    scene.pad(UiButton::Menu);
    wait_for(&mut scene, "homeCardModel", "power", "true", 2000);
    scene.wait(250);
    for _ in 0..4 {
        scene.pad(UiButton::Down);
    }
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "homeCardModel", "dialog", "true", 2000);
    scene.wait(200);
    assert_eq!(read(&scene.tree(), "questionTitle", "text"), "Quit EmuSen?");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "homeCardModel", "power", "true", 2000);
    // Idle Power omits subtitle.
    let mut idle_facts = power_facts.clone();
    idle_facts["suspended"] = json!("");
    assert!(scene.set_property("homeCardModel", "facts", &idle_facts.to_string()));
    scene.wait(150);
    assert_eq!(scene.property("homeCardModel", "subtitle"), "");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 3000);
    assert!(scene.set_property("homeCardModel", "start", ""));
    scene.wait(250);

    // Real SessionModel.failedFolder -> Main -> NoticeModel.raised -> NoticePresenter.sync across two identical launch failures, plus X on Eclipse Home.
    let missing_rom = world.root.join("Roms/SNES/Missing Disc.sfc").to_string_lossy().into_owned();
    let session_ptr = scene.object("session");
    assert!(!session_ptr.is_null(), "session");
    for attempt in 1..=2 {
        let session_obj = unsafe { std::pin::Pin::new_unchecked(&mut *(session_ptr as *mut emusen_qt_models::session_model::qobject::SessionModel)) };
        session_obj.open(&QString::from(missing_rom.as_str()));
        wait_for(&mut scene, "notices", "shown", "true", 4000);
        assert!(scene.property("noticePresenter", "heading").contains("Missing Disc"), "attempt {attempt}: heading");
        assert_eq!(scene.property("noticePresenter", "hasAction"), "true", "attempt {attempt}: repeat failure keeps Open its folder");
        assert_eq!(scene.property("noticePresenter", "actionLabel"), "Open its folder");
        assert_eq!(scene.property("noticePresenter", "actionPath"), rom_dir);
        let before = folders_caught(&mut scene).len();
        pads_button(&mut scene, UiButton::Screensaver);
        let after = folders_caught(&mut scene);
        assert_eq!(after.len(), before + 1, "attempt {attempt}: X on Eclipse Home triggers Open its folder");
        assert!(after.last().unwrap().contains("Roms/SNES"), "attempt {attempt}: opened {}", after.last().unwrap());
        assert_eq!(scene.property("screens", "shown"), "Home", "attempt {attempt}: X did not leak to Home");
        expire_notice(&mut scene);
        assert_eq!(scene.property("noticePresenter", "hasAction"), "false", "attempt {attempt}: hiding clears action");
    }

    // Unrelated attention notice cannot steal a pending folder action scoped to a different failure.
    let presenter_ptr = scene.object("noticePresenter");
    let presenter_obj = unsafe { std::pin::Pin::new_unchecked(&mut *(presenter_ptr as *mut emusen_qt_models::notice_presenter::qobject::NoticePresenter)) };
    presenter_obj.set_pending_folder(&QString::from("Failed to load Other: missing"), &QString::from(rom_dir.as_str()));
    raise_plain_notice(&mut scene, "Settings could not save", true);
    scene.wait(120);
    assert_eq!(scene.property("noticePresenter", "hasAction"), "false", "unrelated notice did not steal pending folder");
    raise_plain_notice(&mut scene, "Failed to load Other: missing", true);
    scene.wait(120);
    assert_eq!(scene.property("noticePresenter", "hasAction"), "false", "stale pending folder was cleared on mismatch");
    expire_notice(&mut scene);

    // Lightweight Eclipse QuestionHost checks for LaunchQuestion, ResumeQuestion, and Cores.
    assert_eq!(scene.property("launchQuestion", "eclipse"), "true");
    assert_eq!(scene.property("resumeQuestion", "eclipse"), "true");
    let screens_ptr = scene.object("screens");
    let screens_obj = unsafe { std::pin::Pin::new_unchecked(&mut *(screens_ptr as *mut emusen_qt_models::screens::qobject::Screens)) };
    screens_obj.place(emusen_qt_models::screens::CORES_PLACE as i32);
    wait_for(&mut scene, "screens", "shown", "Cores", 3000);
    scene.wait(250);
    assert_eq!(scene.property("coresQuestion", "eclipse"), "true");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "screens", "shown", "Home", 3000);
    scene.wait(250);

    // --- BOARD #14: Delete Slot 1? dialog + Failure Notice banner with X Open its folder ---
    let home_id = theme_object(&mut scene, "homeScreen");
    assert!(emusen_theme_qt::host::actions(object(home_id)).expect("Actions").open_page(&QString::from(rom_str.as_str())));
    wait_for(&mut scene, "screens", "shown", "Page", 3000);
    wait_for(&mut scene, "pageModel", "ready", "true", 5000);
    scene.wait(350);
    // Navigate to States tab (pageSection 1); X while failure notice is active opens folder WITHOUT leaking to Delete.
    scene.pad(UiButton::PageDown);
    wait_for(&mut scene, "pageModel", "section", "1", 2000);
    scene.wait(250);
    raise_board_notice(&mut scene, &rom_dir);
    scene.wait(150);
    let before = folders_caught(&mut scene).len();
    pads_button(&mut scene, UiButton::Screensaver);
    let after = folders_caught(&mut scene);
    assert_eq!(after.len(), before + 1, "X on GamePage with failure notice opens its folder");
    assert!(after.last().unwrap().contains("Roms/SNES"), "opened {}", after.last().unwrap());
    assert_eq!(scene.property("pageModel", "asking"), "false", "X did not leak to GamePage Delete");
    assert_eq!(scene.property("pageQuestion", "opened"), "false", "Delete dialog stayed closed");
    assert!(slot1.exists(), "Slot 1 untouched when X handled by failure notice");
    expire_notice(&mut scene);

    // With no failure notice active, X opens Delete Slot 1? dialog (#14).
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "pageModel", "asking", "true", 3000);
    wait_for(&mut scene, "pageQuestion", "opened", "true", 3000);
    scene.wait(350);
    assert_eq!(scene.property("theme", "pageDrawn"), "false", "question stays program-owned");

    hold_screen(
        &mut scene,
        "eclipseNoticeCanvas",
        "gameScreen",
        TABLE_DIALOG,
        &["notice", "dialogDisc", "dialogSection", "dialogHelp"],
        &[
            ("noticeHeading", "Tower Set could not start"),
            ("noticeBody", "Its second disc is missing: Tower Set (Disc 2).bin is named in the sheet and is not in the folder. Nothing was changed."),
            ("noticeActionLabel", "Open its folder"),
            ("questionTitle", "Delete Slot 1?"),
            ("questionSentence.0", "Saved yesterday at 8:40 PM. The state and its picture go. Your in-game saves stay."),
            ("answerLabel.0", "Keep it"),
            ("answerLabel.1", "Delete"),
        ],
        "dialogHelp",
        &["Choose", "Keep it"],
        &["A", "B"],
        &mut |s| raise_board_notice(s, &rom_dir),
        &mut differences,
    );
    picture(&mut scene, &format!("dialog-1920x1200-{backend}.png"), |s| raise_board_notice(s, &rom_dir));

    // Pressing X through PadReader while failure notice with action is shown opens its folder via Folders.
    raise_board_notice(&mut scene, &rom_dir);
    scene.wait(150);
    let before = folders_caught(&mut scene).len();
    pads_button(&mut scene, UiButton::Screensaver);
    let after = folders_caught(&mut scene);
    assert_eq!(after.len(), before + 1, "X over dialog opened folder");
    assert!(after.last().unwrap().contains("Roms/SNES"), "X opened folder: {}", after.last().unwrap());

    // Non-attention success notice draws checkmark icon and hides body + action pill.
    raise_plain_notice(&mut scene, "State saved to slot 1", false);
    scene.wait(150);
    assert_eq!(scene.property("noticePresenter", "heading"), "State saved to slot 1");
    assert_eq!(scene.property("noticePresenter", "body"), "");
    assert_eq!(scene.property("noticePresenter", "icon"), "notice.ok");
    assert_eq!(scene.property("noticePresenter", "hasAction"), "false");

    // Horizontal dialog navigation: Right moves focus to Delete, Left returns to Keep it, B keeps state.
    scene.pad(UiButton::Right);
    scene.wait(150);
    assert_eq!(painted(&scene.tree(), "answerBg.1"), Colour::parse("#FFF6E7D8"), "Right focuses Delete pill");
    scene.pad(UiButton::Left);
    scene.wait(150);
    assert_eq!(painted(&scene.tree(), "answerBg.0"), Colour::parse("#FFF6E7D8"), "Left focuses Keep it pill");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "pageModel", "asking", "false", 3000);
    assert!(slot1.exists(), "B keeps Slot 1 intact");

    // Opening again, moving Right to Delete, and pressing A deletes Slot 1.
    wait_for(&mut scene, "theme", "pageDrawn", "true", 3000);
    scene.wait(250);
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "pageQuestion", "opened", "true", 3000);
    scene.pad(UiButton::Right);
    scene.wait(120);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "pageModel", "asking", "false", 3000);
    for _ in 0..40 {
        if !slot1.exists() {
            break;
        }
        scene.wait(50);
    }
    assert!(!slot1.exists(), "Delete removes Slot 1");

    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
    scene.assert_quiet();
}
