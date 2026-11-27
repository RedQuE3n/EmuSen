//! A theme's Settings and game page read and acted on in the program (EmuSen_Qt.md §3.2, the proof paragraph).

use crate::common;

use common::{first_game_page, sandbox, saved_state, wait_for};
use cxx_qt::casting::Upcast;
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_kit_core::settings::RowKind;
use emusen_platform_client::galaxia::consts::SAVE_STATE;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_models::game_page_model::qobject::GamePageModel;
use emusen_qt_models::library_model::qobject::QAbstractListModel;
use emusen_qt_models::settings_model::qobject::SettingsModel;
use emusen_qt_test::Scene;

const PAGES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../theme-host/tests/themes/pages");

/// A model of the window's by its objectName, as its own type.
fn model<T: Upcast<QAbstractListModel>>(scene: &mut Scene<'_>, name: &str) -> &'static T {
    let object = scene.object(name);
    assert!(!object.is_null(), "{name} is in the window");
    // Safety: the window's model of that name is a T, its list model's QObject at its address, alive while the window is.
    unsafe { &*T::from_base_ptr(object as *const QAbstractListModel) }
}

/// The theme's screen asked to call `Actions` with `command`, and what Actions answered.
fn ask(scene: &mut Scene<'_>, screen: &str, command: &str) -> String {
    assert!(scene.set_property(screen, "command", ""));
    assert!(scene.set_property(screen, "command", command));
    scene.wait(60);
    scene.property(screen, "took")
}

/// The words of an item the theme drew, from the item tree, as a Repeater's delegate has no QObject parent.
fn drawn(scene: &mut Scene<'_>, name: &str) -> String {
    let tree = scene.tree();
    tree.nodes.iter().find(|n| n.name == name).and_then(|n| n.text.clone()).unwrap_or_default()
}

/// Settings' rows as the theme drew them, each `|`-joined in `Settings.rows`' roles' order.
fn drawn_rows(scene: &mut Scene<'_>, count: usize) -> Vec<Vec<String>> {
    let tree = scene.tree();
    let text = |name: String| tree.nodes.iter().find(|n| n.name == name).and_then(|n| n.text.clone()).unwrap_or_default();
    (0..count).map(|i| text(format!("setRow{i}")).split('|').map(str::to_owned).collect()).collect()
}

/// Every row the theme drew is the program's row, its facets the view's, and the pane and the trail the model's.
fn settings_read_whole(scene: &mut Scene<'_>, at: &str) {
    scene.wait(60);
    let settings = model::<SettingsModel>(scene, "settingsModel");
    let rows = settings.shown_rows().to_vec();
    let facets = settings.view().facets();
    let focused = *settings.focused();
    assert_eq!(scene.property("setFocused", "text"), format!("{focused} of {}", rows.len()), "{at}: the focus and the count");
    assert_eq!(scene.property("setPage", "text"), settings.view().page_key(), "{at}: the page's key");
    assert_eq!(scene.property("setTrail", "text"), settings.trail().iter().map(|t| t.to_string()).collect::<Vec<_>>().join("|"), "{at}");
    let joined = |list: &cxx_qt_lib::QStringList| list.iter().map(|t| t.to_string()).collect::<Vec<_>>().join("/");
    let tones = settings.view().pane(focused.max(0) as usize).tones.iter().map(|t| (*t as i32).to_string()).collect::<Vec<_>>().join("/");
    let scales = (0..settings.samples().iter().count()).map(|i| settings.sample_scale(i as i32).to_string()).collect::<Vec<_>>().join("/");
    let pane = [settings.sentence().to_string(), joined(settings.pane_lines()), joined(settings.pane_muted()), tones, settings.preview().to_string(),
        settings.caption().to_string(), settings.meter().to_string(), joined(settings.samples()), scales];
    assert_eq!(scene.property("setPane", "text"), pane.join("|"), "{at}: the pane is the model's, each sample's scale with it");
    for (i, (row, drawn)) in rows.iter().zip(drawn_rows(scene, rows.len())).enumerate() {
        assert_eq!(drawn.len(), 18, "{at} row {i} drawn with every role: {drawn:?} {}", scene.property("theme", "warnings"));
        let kind = row.kind.index();
        let facet = &facets[i];
        assert_eq!(drawn[..2], [row.id.clone(), row.label.clone()], "{at} row {i}: key and label");
        assert_eq!(drawn[2], facet.about, "{at} row {i}: about");
        assert_eq!(drawn[3], row.value, "{at} row {i}: value");
        assert_eq!(drawn[4], kind.to_string(), "{at} row {i}: kind");
        assert_eq!(drawn[5..10], [facet.switch.is_some(), facet.switch == Some(true), row.held, row.marked, row.code].map(|b| b.to_string()), "{at} row {i}");
        if let Some(slider) = settings.view().slider(i) {
            let fields: Vec<f64> = drawn[10..14].iter().map(|t| t.parse().unwrap()).collect();
            assert_eq!(fields, slider, "{at} row {i}: the slider's minimum, maximum, step and amount");
        }
        if let Some(tile) = &facet.tile {
            assert_eq!(drawn[14..], [tile.console.clone(), tile.system.clone(), tile.games.to_string(), tile.engine.clone()], "{at} row {i}: the tile");
        }
    }
    assert!(!rows.is_empty(), "{at}: rows to read");
}

/// The first row of the shown rows that `test` picks, by its index.
fn row_where(scene: &mut Scene<'_>, test: impl Fn(usize, &emusen_qt_models::settings::Row) -> bool) -> Option<usize> {
    let settings = model::<SettingsModel>(scene, "settingsModel");
    settings.shown_rows().iter().enumerate().position(|(i, r)| test(i, r))
}

/// The page's question as `Page` holds it is the model's.
fn question_read(scene: &mut Scene<'_>, page: &GamePageModel) {
    let joined = |list: &cxx_qt_lib::QStringList| list.iter().map(|t| t.to_string()).collect::<Vec<_>>().join("/");
    let want = [page.asking().to_string(), page.question_title().to_string(), joined(page.question_sentences()), joined(page.question_choices()),
        page.question_back().to_string()];
    assert_eq!(scene.property("gameQuestion", "text"), want.join("|"), "the question as the model asks it");
}

/// The page's words, art and focus as `Page` holds them are the model's.
fn page_read(scene: &mut Scene<'_>, page: &GamePageModel, at: &str) {
    let want = [page.title(), page.facts(), page.cover(), page.hero(), page.clip(), page.words(), page.picture()].map(|t| t.to_string());
    assert_eq!(scene.property("gameWords", "text"), want.join("|"), "{at}: the page's words and art");
    let (action, content) = page.focused();
    let help = emusen_theme_qt::help::kind_of(page.help_screen()).map_or(-1, |k| k.repr);
    assert_eq!(scene.property("gameFocus", "text"), format!("{action} {content} {help}"), "{at}: the focus and its help");
    assert!(scene.property("gameSections", "text").ends_with(&format!("|{}", page.cover_art())), "{at}: whether cover art is open");
}

#[test]
fn qt_3_2_a_theme_draws_settings_and_the_game_page_through_read_only_views() {
    let world = sandbox("qt-app-theme-pages", &[("NES", "One.nes"), ("NES", "Two.nes")]);
    // Two states of the first game, so Resume steps between them.
    let platform = Platform::load(library()).unwrap();
    let one = world.root.join("Roms/NES/One.nes");
    saved_state(&platform, SAVE_STATE, &one, 1, 3600);
    saved_state(&platform, SAVE_STATE, &one, 2, 2 * 3600);
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    assert!(scene.set_property("program", "theme", PAGES));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "settingsModel", "ready", "true", 5000);
    assert_eq!(scene.property("theme", "failed"), "false", "{}", scene.property("theme", "warnings"));
    scene.wait(300);

    // Home's Settings place opens the theme's Settings in place of the program's.
    for _ in 0..5 {
        scene.pad(UiButton::Right);
    }
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Settings", 2000);
    wait_for(&mut scene, "theme", "settings", "true", 2000);
    assert_eq!(scene.property("settingsSlot", "active"), "false", "the program's Settings is not made under the theme's");
    assert_eq!(scene.property("pagesSettings", "visible"), "true");

    // Every section's rows, read through Settings.rows; the sections are the model's, the help the section's.
    let sections = model::<SettingsModel>(&mut scene, "settingsModel").sections().iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert_eq!(scene.property("setSections", "text"), sections.join("|"));
    for (i, name) in sections.iter().enumerate() {
        assert_eq!(ask(&mut scene, "pagesSettings", &format!("settingsSection {i}")), "true");
        wait_for(&mut scene, "setSection", "text", &i.to_string(), 1000);
        settings_read_whole(&mut scene, name);
        assert_eq!(drawn(&mut scene, "setHelpEntry0"), "Change", "{name}: Settings' own help, A Change");
    }

    // A switch turned and a choice stepped on Display, each through settingsStep, as the program's own Left and Right.
    let display = sections.iter().position(|s| s == "Display").unwrap();
    ask(&mut scene, "pagesSettings", &format!("settingsSection {display}"));
    wait_for(&mut scene, "setPage", "text", "display", 1000);
    // The page's calls from Settings are declined, and none reaches the page's model; nor does a section past the last.
    let page_section = scene.property("pageModel", "section");
    assert_eq!(ask(&mut scene, "pagesSettings", "pageSection 1"), "false", "a page's call from Settings");
    assert_eq!(ask(&mut scene, "pagesSettings", "pageAct 0"), "false");
    assert_eq!(scene.property("pageModel", "section"), page_section, "the page's model untouched");
    assert_eq!(ask(&mut scene, "pagesSettings", &format!("settingsSection {}", sections.len())), "false", "no section past the last");
    assert_eq!(ask(&mut scene, "pagesSettings", "settingsFocus -1"), "false", "no row before the first");
    assert_eq!(scene.property("setPage", "text"), "display");
    let switch = (0..40).find(|i| drawn(&mut scene, &format!("setRow{i}")).split('|').nth(5) == Some("true")).expect("a switch on Display");
    let before = drawn(&mut scene, &format!("setRow{switch}"));
    assert_eq!(ask(&mut scene, "pagesSettings", &format!("settingsFocus {switch}")), "true");
    wait_for(&mut scene, "settingsModel", "focused", &switch.to_string(), 1000);
    assert_eq!(ask(&mut scene, "pagesSettings", &format!("settingsStep {switch} 1")), "true");
    scene.wait(200);
    let after = drawn(&mut scene, &format!("setRow{switch}"));
    assert_ne!(before.split('|').nth(6), after.split('|').nth(6), "the switch turned: {before} -> {after}");
    settings_read_whole(&mut scene, "Display, a switch turned");
    let choice = row_where(&mut scene, |_, r| r.id == "textsize").expect("Text size, a choice of three on Display");
    let value = drawn(&mut scene, &format!("setRow{choice}"));
    ask(&mut scene, "pagesSettings", &format!("settingsStep {choice} 1"));
    scene.wait(200);
    assert_ne!(drawn(&mut scene, &format!("setRow{choice}")), value, "the choice stepped");
    assert_eq!(ask(&mut scene, "pagesSettings", &format!("settingsStep {choice} -1")), "true");
    scene.wait(200);
    assert_eq!(drawn(&mut scene, &format!("setRow{choice}")), value, "Left steps it back: of three choices, only a step by -1 returns");
    assert_eq!(ask(&mut scene, "pagesSettings", &format!("settingsFocus {choice}")), "true");
    wait_for(&mut scene, "settingsModel", "focused", &choice.to_string(), 1000);
    settings_read_whole(&mut scene, "Text size focused");
    let samples = model::<SettingsModel>(&mut scene, "settingsModel").samples().iter().count();
    assert!(samples > 1, "Text size's pane shows a sample at each size: {}", scene.property("setPane", "text"));

    // A page of choices: Shaders, a console, the CRT chosen among its shaders and the mark moved to it.
    let shaders = row_where(&mut scene, |_, r| r.id == "shaders").expect("Display's Shaders");
    assert_eq!(ask(&mut scene, "pagesSettings", &format!("settingsChoose {shaders}")), "true");
    wait_for(&mut scene, "setPage", "text", "shaderConsoles", 1000);
    settings_read_whole(&mut scene, "the shaders' consoles");
    let first = row_where(&mut scene, |_, r| r.kind == RowKind::Opens && !r.held).expect("a console's shaders");
    ask(&mut scene, "pagesSettings", &format!("settingsChoose {first}"));
    scene.wait(300);
    assert!(scene.property("setPage", "text").starts_with("shaders:"), "{}", scene.property("setPage", "text"));
    settings_read_whole(&mut scene, "a console's shaders");
    assert_eq!(drawn(&mut scene, "setHelpEntry0"), "Choose", "a page of choices' help");
    let crt = row_where(&mut scene, |_, r| r.kind == RowKind::Choice && r.label == "CRT").expect("the CRT among the shaders");
    assert_eq!(ask(&mut scene, "pagesSettings", &format!("settingsChoose {crt}")), "true");
    scene.wait(300);
    assert_eq!(drawn_rows(&mut scene, crt + 1)[crt][8], "true", "the mark moved to the CRT");
    ask(&mut scene, "pagesSettings", "settingsBack");
    wait_for(&mut scene, "setPage", "text", "shaderConsoles", 1000);
    ask(&mut scene, "pagesSettings", "settingsBack");
    wait_for(&mut scene, "setPage", "text", "display", 1000);

    // A door opened and left: Shader levers opens its consoles, the console's CRT its levers, each read whole; a slider moved.
    let door = row_where(&mut scene, |_, r| r.id == "levers").expect("Display's Shader levers");
    assert_eq!(ask(&mut scene, "pagesSettings", &format!("settingsChoose {door}")), "true");
    wait_for(&mut scene, "setPage", "text", "leverConsoles", 1000);
    settings_read_whole(&mut scene, "the levers' consoles");
    assert_eq!(drawn(&mut scene, "setHelpEntry0"), "Change", "a page's help");
    ask(&mut scene, "pagesSettings", &format!("settingsChoose {first}"));
    scene.wait(300);
    let page = scene.property("setPage", "text");
    assert!(page.starts_with("levers:"), "a console's levers: {page}");
    settings_read_whole(&mut scene, "a console's levers");
    if let Some(slider) = row_where(&mut scene, |_, r| r.kind == RowKind::Slider) {
        let row = drawn(&mut scene, &format!("setRow{slider}"));
        let parts: Vec<f64> = row.split('|').skip(10).take(4).map(|t| t.parse().unwrap()).collect();
        let (minimum, maximum, amount) = (parts[0], parts[1], parts[3]);
        let to = if amount < maximum { maximum } else { minimum };
        assert_eq!(ask(&mut scene, "pagesSettings", &format!("settingsSlide {slider} {to}")), "true");
        scene.wait(200);
        let moved: f64 = drawn(&mut scene, &format!("setRow{slider}")).split('|').nth(13).unwrap().parse().unwrap();
        assert!((moved - to).abs() < 1e-6, "the slider moved to {to}: {moved}");
        ask(&mut scene, "pagesSettings", &format!("settingsSlide {slider} {amount}"));
    } else {
        panic!("no slider on {page}: {}", drawn_rows(&mut scene, 30).iter().map(|r| r.join("|")).collect::<Vec<_>>().join("\n"));
    }
    assert_eq!(ask(&mut scene, "pagesSettings", "settingsBack"), "true");
    wait_for(&mut scene, "setPage", "text", "leverConsoles", 1000);
    ask(&mut scene, "pagesSettings", "settingsBack");
    wait_for(&mut scene, "setPage", "text", "display", 1000);
    assert_eq!(scene.property("setFocused", "text").split(' ').next(), Some(door.to_string().as_str()), "B comes back to the door's row");

    // Consoles: a console's tile, then its page, where the shoulders step the console, and B leaves Settings at the last.
    let consoles = sections.iter().position(|s| s == "Consoles").unwrap();
    ask(&mut scene, "pagesSettings", &format!("settingsSection {consoles}"));
    wait_for(&mut scene, "setPage", "text", "consoles", 1000);
    let nes = (0..200).find(|i| drawn(&mut scene, &format!("setRow{i}")).split('|').nth(16) == Some("2")).expect("the console of the two games");
    let tile = drawn(&mut scene, &format!("setRow{nes}"));
    let parts: Vec<&str> = tile.split('|').collect();
    assert!(!parts[14].is_empty() && !parts[15].is_empty(), "the tile names its console and its system: {tile}");
    ask(&mut scene, "pagesSettings", &format!("settingsChoose {nes}"));
    wait_for(&mut scene, "setPage", "text", &format!("console:{}", parts[14]), 1000);
    settings_read_whole(&mut scene, "the NES's page");
    assert_eq!(drawn(&mut scene, "setHelpEntry2"), "Console", "a console's page steps the console on the shoulders");
    assert_eq!(ask(&mut scene, "pagesSettings", "settingsShoulder 1"), "true");
    scene.wait(200);
    let next = scene.property("setPage", "text");
    assert!(next.starts_with("console:") && next != format!("console:{}", parts[14]), "the shoulder stepped the console: {next}");
    settings_read_whole(&mut scene, "the console after the NES");
    assert_eq!(ask(&mut scene, "pagesSettings", "settingsBack"), "true", "B closing a page is Settings' own");
    wait_for(&mut scene, "setPage", "text", "consoles", 1000);
    assert_eq!(ask(&mut scene, "pagesSettings", "settingsBack"), "false", "B with no page left leaves Settings");
    wait_for(&mut scene, "screens", "shown", "Home", 2000);

    // The game's page: Games, All Games and X on the first game open the theme's page in place of the program's.
    scene.wait(200);
    first_game_page(&mut scene);
    wait_for(&mut scene, "theme", "game", "true", 2000);
    wait_for(&mut scene, "gameReady", "text", "true", 10000);
    assert_eq!(scene.property("gamePage", "visible"), "false", "the program's page is not drawn under the theme's");
    let page = model::<GamePageModel>(&mut scene, "pageModel");
    assert!(scene.property("gameKey", "text").ends_with("One.nes"), "the game's key: {}", scene.property("gameKey", "text"));
    page_read(&mut scene, page, "opened");
    question_read(&mut scene, page);

    // Settings' calls from the page are declined, and none reaches Settings' model.
    let settings = model::<SettingsModel>(&mut scene, "settingsModel");
    let values = |s: &SettingsModel| s.shown_rows().iter().map(|r| r.value.clone()).collect::<Vec<_>>();
    let (stored, section) = (values(settings), scene.property("settingsModel", "section"));
    for call in ["settingsStep 0 1", "settingsChoose 0", "settingsSlide 0 1", "settingsSection 0", "settingsFocus 1", "settingsShoulder 1", "settingsBack"] {
        assert_eq!(ask(&mut scene, "pagesGame", call), "false", "{call} from the page");
    }
    scene.wait(200);
    assert_eq!(values(settings), stored, "no stored setting changed");
    assert_eq!(scene.property("settingsModel", "section"), section);
    assert_eq!(scene.property("screens", "shown"), "Page");

    // Each tab, its rows or cards read whole through Page.rows; the actions through Page.actions.
    let actions = page.shown_actions();
    for (i, a) in actions.iter().enumerate() {
        let shown = drawn(&mut scene, &format!("gameAction{i}"));
        assert!(shown.starts_with(&format!("{}|{}|{}|", a.label(), a.value(), a.button())), "action {i}: {shown}");
    }
    let tabs: Vec<String> = page.sections().iter().map(|s| s.to_string()).collect();
    for (i, tab) in tabs.iter().enumerate() {
        assert_eq!(ask(&mut scene, "pagesGame", &format!("pageSection {i}")), "true");
        wait_for(&mut scene, "pageModel", "section", &i.to_string(), 1000);
        scene.wait(200);
        assert!(scene.property("gameSections", "text").starts_with(&format!("{}|{i}|{}|", tabs.join("|"), page.shows())), "{tab}");
        for (r, item) in page.shown_items().iter().enumerate() {
            let want = [item.key().to_owned(), item.label().to_owned(), item.value().to_owned(), item.line().to_owned(), item.picture().to_owned()].join("|");
            let shown = drawn(&mut scene, &format!("gameRow{r}"));
            assert!(shown.starts_with(&want), "{tab} row {r}: {shown} against {want}");
            assert!(shown.ends_with(&format!("|{}|{}|{}", item.steps(), item.opens(), item.marked())), "{tab} row {r}: {shown}");
        }
        page_read(&mut scene, page, tab);
        // The focus walked down the tab's items, each move read back, though the help may stay the same.
        for r in 0..page.shown_items().len().min(3) {
            assert_eq!(ask(&mut scene, "pagesGame", &format!("pageFocus -1 {r}")), "true");
            wait_for(&mut scene, "gameFocus", "text", &format!("-1 {r} {}", emusen_theme_qt::help::kind_of(page.help_screen()).map_or(-1, |k| k.repr)), 1000);
        }
    }
    assert_eq!(ask(&mut scene, "pagesGame", &format!("pageSection {}", tabs.len())), "false", "no tab past the last");
    assert_eq!(ask(&mut scene, "pagesGame", &format!("pageFocus {} 0", actions.len())), "false", "no action past the last");

    // The focus told to the page moves its help; a stepped row of Options steps; Favourite is the program's own.
    ask(&mut scene, "pagesGame", "pageSection 0");
    wait_for(&mut scene, "pageModel", "section", "0", 1000);
    let favourite = actions.iter().position(|a| a.label() == "Favourite").expect("Favourite among the actions");
    assert_eq!(ask(&mut scene, "pagesGame", &format!("pageFocus {favourite} 0")), "true");
    scene.wait(100);
    assert_eq!(scene.property("gameFocus", "text").split(' ').next(), Some(favourite.to_string().as_str()));
    assert_eq!(drawn(&mut scene, "gameHelpEntry0"), "Favourite", "the help follows the focus");
    let before = drawn(&mut scene, &format!("gameAction{favourite}"));
    assert_eq!(ask(&mut scene, "pagesGame", &format!("pageAct {favourite}")), "true");
    scene.wait(400);
    assert_ne!(drawn(&mut scene, &format!("gameAction{favourite}")), before, "the favourite turned");
    assert_eq!(ask(&mut scene, "pagesGame", "pageDeclared favourite"), "true");
    scene.wait(400);
    assert_eq!(drawn(&mut scene, &format!("gameAction{favourite}")), before, "and back, through the help's Y");
    let options = tabs.iter().position(|t| t == "Options").expect("an Options tab");
    ask(&mut scene, "pagesGame", &format!("pageSection {options}"));
    wait_for(&mut scene, "pageModel", "section", &options.to_string(), 1000);
    scene.wait(200);
    let stepped = page.shown_items().iter().position(|i| i.steps()).expect("a stepped row on Options");
    let value = drawn(&mut scene, &format!("gameRow{stepped}"));
    assert_eq!(ask(&mut scene, "pagesGame", &format!("pageStep {stepped} 1")), "true");
    scene.wait(400);
    assert_ne!(drawn(&mut scene, &format!("gameRow{stepped}")), value, "the row stepped");
    ask(&mut scene, "pagesGame", &format!("pageStep {stepped} -1"));
    scene.wait(400);

    // A door of Details opened and left: Edit Details, then back to Details by pageBack; a question asked and answered No.
    let details = tabs.iter().position(|t| t == "Details").expect("a Details tab");
    ask(&mut scene, "pagesGame", &format!("pageSection {details}"));
    wait_for(&mut scene, "pageModel", "section", &details.to_string(), 1000);
    scene.wait(200);
    let keys: Vec<String> = page.shown_items().iter().map(|i| format!("{}:{}", i.key(), i.opens())).collect();
    let edit = page.shown_items().iter().position(|i| i.opens()).unwrap_or_else(|| panic!("a door on Details: {keys:?}"));
    {
        assert_eq!(ask(&mut scene, "pagesGame", &format!("pageFocus -1 {edit}")), "true");
        assert_eq!(ask(&mut scene, "pagesGame", &format!("pageChoose {edit}")), "true");
        scene.wait(300);
        assert_eq!(ask(&mut scene, "pagesGame", "pageBack"), "true");
        scene.wait(300);
        assert_eq!(scene.property("screens", "shown"), "Page", "B on a deeper part stays on the page");
    }
    let asks = page.shown_items().iter().position(|i| i.key() == "hide").unwrap_or_else(|| panic!("Hide on Details: {keys:?}"));
    {
        ask(&mut scene, "pagesGame", &format!("pageChoose {asks}"));
        wait_for(&mut scene, "pageModel", "asking", "true", 1000);
        scene.wait(100);
        question_read(&mut scene, page);
        assert_eq!(ask(&mut scene, "pagesGame", "pageAnswer 0"), "true");
        wait_for(&mut scene, "pageModel", "asking", "false", 1000);
        assert_eq!(ask(&mut scene, "pagesGame", "pageAnswer 0"), "false", "no question left to answer");
    }

    // B with nothing of the page's own open leaves the page for the grid.
    assert_eq!(ask(&mut scene, "pagesGame", "pageBack"), "false", "B with nothing of the page's own open leaves it");
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);

    // The page again: Resume stepped to its other state by pageStepAction, and run: the game starts from that state.
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "screens", "shown", "Page", 2000);
    wait_for(&mut scene, "gameReady", "text", "true", 10000);
    scene.wait(200);
    let actions = page.shown_actions();
    let resume = actions.iter().position(|a| a.label() == "Resume").unwrap_or_else(|| panic!("Resume of the two states: {actions:?}"));
    let by = if actions[resume].step_right() { 1 } else { -1 };
    let before = drawn(&mut scene, &format!("gameAction{resume}"));
    assert_eq!(ask(&mut scene, "pagesGame", &format!("pageStepAction {resume} {by}")), "true");
    scene.wait(300);
    assert_ne!(drawn(&mut scene, &format!("gameAction{resume}")), before, "Resume stepped its state");
    assert_eq!(ask(&mut scene, "pagesGame", &format!("pageStepAction {resume} {by}")), "false", "past the last state Resume does not step");
    let stepped_to = page.shown_actions()[resume].value().to_owned();
    let slot = (1..=2).find(|n| stepped_to.starts_with(&format!("Slot {n} "))).unwrap_or_else(|| panic!("Resume names a slot: {stepped_to}"));
    let chosen = platform.save_path(SAVE_STATE, one.to_str().unwrap(), None, slot).unwrap();
    assert_eq!(ask(&mut scene, "pagesGame", &format!("pageAct {resume}")), "true");
    for _ in 0..40 {
        if scene.property("session", "startState") == chosen.to_str().unwrap() {
            break;
        }
        scene.wait(50);
    }
    let said = (scene.property("pageModel", "problem"), scene.property("screens", "shown"), scene.property("launch", "asking"));
    assert_eq!(scene.property("session", "startState"), chosen.to_str().unwrap(), "the game starts from the state stepped to: {said:?}");
    scene.assert_quiet();
}
