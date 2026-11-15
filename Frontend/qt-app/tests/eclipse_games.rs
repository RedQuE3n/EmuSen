//! Eclipse's games view (EmuSen_Qt.md §3.4.10) over a library of 22 games, a tall, a wide and a square cover and the rest
//! with none: every built row of `eclipse/games.tsv` held to the screen at each of the gate's sizes, the tiles by the
//! board's rule, and the focus moved by the pad; the theme's fit, contrast and walk are theme-check's loaded step (EmuSen_Qt.md §3.7.2). With `ECLIPSE_GAMES_PICTURES` set, the
//! view is saved there at 1280x800 and 1920x1200.

mod common;
#[path = "eclipse/table.rs"]
mod table;

use common::{draw_covers, sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
use emusen_platform_client::Platform;
use emusen_qt::MAIN_URL;
use emusen_qt_models::themes;
use emusen_qt_test::harness::SIZES;
use emusen_qt_test::Scene;

/// The covers drawn, each a shape of its own: tall, wide and square, at sizes that are constants of this test.
const COVERS: [(&str, (u32, u32)); 3] = [("Aurora Drift", (300, 420)), ("Amber Relay", (420, 300)), ("Basalt Kings", (360, 360))];
/// The games with no cover, enough for five rows of five.
const PLAIN: [&str; 19] = [
    "Cinder Mile", "Cobalt Harbor", "Dune Signal", "Ember Circuit", "Fathom Nine", "Granite Pass", "Harbor Lights", "Indigo Run", "Juniper Vale",
    "Kestrel Run", "Lumen Garden", "Marble Arcade", "Nettle and Thorn", "Opal Tower", "Pine Barrens", "Quartz Field", "Rook and Lantern",
    "Sable Ridge", "Tidal Bloom",
];

fn set_setting(platform: &Platform, key: &str, value: serde_json::Value) {
    let mut settings = platform.model_load(MODEL_APP_SETTINGS, None, "appsettings.json").unwrap().unwrap();
    settings[key] = value;
    platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();
}

/// An item's words and whether it is shown, read from the scene's tree: a view's delegates are not found by name otherwise.
fn shown(scene: &mut Scene<'_>, name: &str) -> (bool, String) {
    let tree = scene.tree();
    let node = tree.find_named(name).map(|i| &tree.nodes[i as usize]).unwrap_or_else(|| panic!("{name} is in the scene"));
    (node.visible, node.text.clone().unwrap_or_default())
}

/// Waits until an item's words are `words`.
fn until_words(scene: &mut Scene<'_>, name: &str, words: &str) {
    for _ in 0..40 {
        if scene.tree().find_named(name).is_some() && shown(scene, name).1 == words {
            return;
        }
        scene.wait(50);
    }
    assert_eq!(shown(scene, name).1, words, "{name}'s words");
}

/// The table's rows for a state, held once its words are shown, at the size the scene has.
fn hold_state(scene: &mut Scene<'_>, state: &str) {
    let rows = table::built();
    let (held, item, words) = table::state_rows(&rows, state);
    wait_for(scene, &item, "text", words, 2000);
    let report = table::differences(scene, &held, 1280.0, 800.0);
    assert!(report.is_empty(), "{state}: {}", report.join("\n"));
}

fn focused_name(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).unwrap_or_default()
}

#[test]
fn qt_3_4_10_eclipses_games_view_is_its_board_by_the_table_at_every_size() {
    let files: Vec<String> = COVERS.iter().map(|(t, _)| *t).chain(PLAIN).map(|t| format!("{t}.sfc")).collect();
    let games: Vec<(&str, &str)> = files.iter().map(|f| ("SNES", f.as_str())).collect();
    let world = sandbox("qt-app-eclipse-games", &games);
    let platform = Platform::load(library()).unwrap();
    let media = world.dir("Media");
    let mut harness = emusen_qt_test::Harness::new();
    let pictures: Vec<_> = COVERS.iter().enumerate().map(|(i, (t, size))| (media.join("snes/covers").join(format!("{t}.png")), *size, i, t.to_string())).collect();
    draw_covers(&mut harness, &world.root, &pictures, |_| {});
    set_setting(&platform, "EsdeMediaDirectory", serde_json::json!(media.to_str().unwrap()));

    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "coverStores", media.to_str().unwrap()));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "screens", "shown", "Home", 3000);
    for _ in 0..40 {
        if focused_name(&mut scene) == "place.0" {
            break;
        }
        scene.wait(50);
    }
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Consoles", 2000);
    let tree = scene.tree();
    let all = tree.find_named("tile.0").expect("All games on Choose a console");
    assert!(scene.focus_item(&tree, all));
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Grid", 2000);
    // Eclipse chosen on the games view, reached through the reference's Choose a console: Eclipse's Home has no places
    // until the top navigation's piece (R-20261115-7).
    assert!(scene.set_property("program", "theme", &themes::built_in_folder("eclipse")));
    wait_for(&mut scene, "host.theme", "ready", "true", 3000);
    let warnings = scene.property("host.theme", "warnings");
    assert!(warnings.is_empty() && scene.property("host.theme", "failed") == "false", "Eclipse's screens load clean: {warnings}");
    wait_for(&mut scene, "gamesGrid", "count", &files.len().to_string(), 3000);
    scene.wait(300);

    // The covers are drawn, each filling its tile; a game with none is the plain tile.
    for (i, title) in ["Amber Relay", "Aurora Drift", "Basalt Kings"].iter().enumerate() {
        assert_eq!(shown(&mut scene, &format!("tileTitle.{i}")).1, *title, "the name sort's order");
        assert!(!shown(&mut scene, &format!("gamesTileGround.{i}")).0, "cover {i} drawn, so no plain tile");
    }
    assert!(!shown(&mut scene, "gamesTileCover.3").0, "no cover");
    assert!(shown(&mut scene, "gamesTileGround.3").0, "the plain tile");

    // The backdrop is the focused game's art at the board's first field's blur and opacity (EclipseLibrary:19, provisional);
    // a game with none has none.
    assert_eq!(scene.property("gamesBackdrop", "sourceOpacity"), "0.35");
    let blur: f64 = scene.property("gamesBackdrop", "sourceBlur").parse().unwrap();
    let board_h = table::built().iter().find(|r| r.item == "gamesBackdrop").expect("the backdrop's row").board_h;
    assert!((blur - 120.0 * emusen_kit_core::canvas::DESIGN_HEIGHT / board_h).abs() < 1e-9, "the board's 120 px blur: {blur}");
    assert!(scene.property("gamesBackdrop", "source").ends_with("Amber%20Relay.png") || scene.property("gamesBackdrop", "source").ends_with("Amber Relay.png"), "{}", scene.property("gamesBackdrop", "source"));
    // The focused game's words are its row's.
    assert_eq!(focused_name(&mut scene), "gamesTile.0");
    assert_eq!(scene.property("gamesTitle", "text"), "Amber Relay");
    assert!(!scene.property("gamesConsole", "text").is_empty(), "the console's name");
    assert!(shown(&mut scene, "gamesRing.0").0, "the focused tile's ring");

    let pictures = std::env::var_os("ECLIPSE_GAMES_PICTURES").map(std::path::PathBuf::from);
    let rows = table::built();
    let held: Vec<&table::Row> = rows.iter().filter(|r| !r.empty_board() && r.state().is_none()).collect();
    assert!(held.len() >= 6, "the games' rows: {}", held.len());
    let mut report = Vec::new();
    for (w, h) in SIZES {
        scene.resize(w, h);
        scene.wait(200);
        report.extend(table::differences(&mut scene, &held, f64::from(w), f64::from(h)).into_iter().map(|d| format!("{w}x{h}: {d}")));
        if f64::from(w) / f64::from(h) == 1.6 {
            let tree = scene.tree();
            let y = |name: &str| tree.find_named(name).map(|i| tree.nodes[i as usize].y);
            assert_eq!(y("gamesTile.4"), y("gamesTile.0"), "{w}x{h}: five columns at the board's shape");
            assert_ne!(y("gamesTile.5"), y("gamesTile.0"), "{w}x{h}: five columns at the board's shape");
        }
        if let Some(dir) = pictures.as_ref().filter(|_| (w, h) == (1280, 800) || (w, h) == (1920, 1200)) {
            std::fs::create_dir_all(dir).unwrap();
            assert!(scene.grab(&dir.join(format!("games-{w}x{h}.png")).display().to_string()));
        }
    }
    scene.resize(1280, 800);
    scene.wait(200);
    assert!(report.is_empty(), "{}", report.join("\n"));


    // The pad moves the focus and the words follow; the rows shown move a row at a time once the focus leaves them.
    let tree = scene.tree();
    let first = tree.find_named("gamesTile.0").unwrap();
    assert!(scene.focus_item(&tree, first));
    scene.pad(UiButton::Right);
    wait_for(&mut scene, "gamesTitle", "text", "Aurora Drift", 1000);
    let top = scene.property("gamesGrid", "contentY");
    for _ in 0..3 {
        scene.pad(UiButton::Down);
    }
    assert_eq!(scene.property("gamesGrid", "contentY"), top, "row 4 of 4 shown: nothing moves");
    scene.pad(UiButton::Down);
    scene.wait(100);
    assert_eq!(focused_name(&mut scene), "gamesTile.21");
    assert_ne!(scene.property("gamesGrid", "contentY"), top, "the fifth row brought in");

    let tree = scene.tree();
    assert!(scene.focus_item(&tree, tree.find_named("gamesTile.3").unwrap()));
    wait_for(&mut scene, "gamesTitle", "text", "Cinder Mile", 1000);
    assert_eq!(scene.property("gamesBackdrop", "source"), "", "a game with no art has no backdrop");

    // The triggers step letters (R-20261115-12): RT to the first game under the next letter, LT back; the bar says Letter.
    let tree = scene.tree();
    assert!(scene.focus_item(&tree, tree.find_named("gamesTile.0").unwrap()));
    until_words(&mut scene, "gamesRailLetterWords", "A");
    assert_eq!(shown(&mut scene, "gamesRailLast").1, "T", "the shown games' last letter");
    scene.pad(UiButton::Last);
    wait_for(&mut scene, "gamesTitle", "text", "Basalt Kings", 1000);
    until_words(&mut scene, "gamesRailLetterWords", "B");
    scene.pad(UiButton::First);
    wait_for(&mut scene, "gamesTitle", "text", "Amber Relay", 1000);
    assert!(scene.tree().nodes.iter().any(|n| n.visible && n.text.as_deref() == Some("Letter")), "the help bar's Letter entry");

    // The chips are the Filter page's choices with their counts (R-20261115-10); Up from the first row reaches the one in
    // force. Favourites, with none, leaves no game, and the list says so (R-20261115-13); Every game again brings them back.
    let words_of = |scene: &mut Scene<'_>, i: usize| shown(scene, &format!("gamesFilterWords.{i}")).1;
    assert_eq!(words_of(&mut scene, 0), format!("Every game · {}", files.len()));
    assert_eq!(words_of(&mut scene, 1), "Favourites · 0");
    assert_eq!(words_of(&mut scene, 2), format!("Not played · {}", files.len()));
    scene.pad(UiButton::Up);
    assert_eq!(focused_name(&mut scene), "gamesFilter.0");
    scene.pad(UiButton::Right);
    assert_eq!(focused_name(&mut scene), "gamesFilter.1");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "gamesGrid", "count", "0", 2000);
    hold_state(&mut scene, "filter");
    assert_eq!(scene.property("emptyLine", "visible"), "false", "a library with games: no folder to add");
    assert_eq!(scene.property("emptyDisc", "visible"), "false");
    scene.pad(UiButton::Left);
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "gamesGrid", "count", &files.len().to_string(), 2000);

    // The sort pill shows the sort in force in the program's words, and A steps it (R-20261115-11).
    assert_eq!(scene.property("gamesSortWords", "text"), "Name");
    let tree = scene.tree();
    assert!(scene.focus_item(&tree, tree.find_named("gamesSort").unwrap()));
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "gamesSortWords", "text", emusen_qt_models::listing::Sort::LastPlayed.words(), 2000);

    // Favourites and Recent with nothing in them, opened from Choose a console's places, each say their own words
    // (R-20261115-13): no game here is a favourite or was played.
    for (place, key, state) in [(1, emusen_qt_models::library::FAVOURITES, "favourites"), (2, emusen_qt_models::library::RECENT, "recent")] {
        scene.pad(UiButton::Back);
        wait_for(&mut scene, "screens", "shown", "Consoles", 2000);
        let tree = scene.tree();
        let tile = tree.find_named(&format!("tile.{place}")).expect("the place on Choose a console");
        assert!(scene.focus_item(&tree, tile));
        scene.pad(UiButton::Accept);
        wait_for(&mut scene, "screens", "shown", "Grid", 2000);
        assert_eq!(scene.property("screens", "key"), key);
        wait_for(&mut scene, "gamesGrid", "count", "0", 2000);
        hold_state(&mut scene, state);
    }
}

#[test]
#[should_panic(expected = "built, with no item or no check")]
fn qt_3_4_10_the_table_refuses_a_built_row_with_no_check() {
    table::rows_of("a part\tEclipseLibrary:1\t1200\tthe program's\tbuilt\tpart\t\t");
}

#[test]
fn qt_3_4_10_the_listings_actions_are_carried_as_the_list_cards_own() {
    // A theme's filter, sort and letter steps, and the triggers' declared actions, become the List card's actions on the grid.
    use emusen_qt::theme::{carried, Carried};
    use emusen_qt_models::list_card;
    use emusen_theme_qt::actions::Action;
    let listed = |action: Action| carried(&action, None, &[]);
    assert_eq!(listed(Action::SetFilter(list_card::NOT_PLAYED.into())), Some(Carried::Listed(list_card::NOT_PLAYED.into())));
    assert_eq!(listed(Action::StepSort(1)), Some(Carried::Listed(list_card::SORT_ON.into())));
    assert_eq!(listed(Action::StepLetter(-1)), Some(Carried::Listed(list_card::JUMP_BACK.into())));
    assert_eq!(listed(Action::StepLetter(1)), Some(Carried::Listed(list_card::JUMP_ON.into())));
    assert_eq!(listed(Action::Act(list_card::JUMP_ON.into())), Some(Carried::Listed(list_card::JUMP_ON.into())));
    assert_eq!(listed(Action::Act("list".into())), Some(Carried::Acted("list".into())), "another declared action is acted as before");
}
