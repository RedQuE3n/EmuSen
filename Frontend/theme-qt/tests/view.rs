//! The library as a theme sees it, over the real clients (EmuSen_Qt.md §3.2): each list's roles are the API's, written out
//! by hand here, and each row is curated from the view with a cell for every role.

mod common;

use emusen_client_fixture::World;
use emusen_theme_qt::view::{self, Cell, Row};

fn text(c: &Cell) -> &str {
    match c {
        Cell::Text(t) => t,
        other => panic!("a text cell, not {other:?}"),
    }
}

fn whole(rows: &[Row]) -> bool {
    rows.iter().all(|r| !r.is_empty())
}

#[test]
fn qt_3_2_each_list_has_the_api_s_roles_and_no_other() {
    assert_eq!(
        view::CONSOLE_ROLES,
        [
            "key",
            "name",
            "displayName",
            "maker",
            "year",
            "count",
            "hue",
            "body",
            "engineName",
            "engineKind",
            "engineMark",
            "libretro",
            "group",
            "groupIndex",
            "groupSize",
            "groupStart"
        ]
    );
    assert_eq!(view::PLACE_ROLES, ["key", "name", "kind", "line", "attention", "count", "available"]);
    assert_eq!(view::SHELF_ROLES, ["key", "name", "line", "count"]);
    assert_eq!(
        view::GAME_ROLES,
        [
            "key",
            "title",
            "cover",
            "coverShape",
            "favourite",
            "lastPlayed",
            "played",
            "consoleId",
            "consoleHue",
            "players",
            "year",
            "genre",
            "rating",
            "description",
            "video",
            "hero",
            "lastPlayedWords",
            "facts"
        ]
    );
    assert_eq!(view::COLLECTION_ROLES, ["key", "name", "count"]);
    assert_eq!(view::HELP_ROLES, ["button", "label", "pair", "enabled"]);
}

#[test]
fn qt_3_2_the_library_s_lists_over_the_real_clients() {
    let world = World::new("theme-view");
    let read = common::read(&world);
    let v = &read.view;

    let consoles = view::consoles(v);
    assert!(whole(&consoles) && consoles.iter().all(|r| r.len() == view::CONSOLE_ROLES.len()));
    assert_eq!(consoles.len(), 1, "the NES alone has games");
    let nes = &consoles[0];
    assert_eq!((text(&nes[0]), text(&nes[1])), ("nes", read.nes_name.as_str()), "the table's id and name");
    assert!(!text(&nes[7]).is_empty(), "the body is the world the kit's Body draws");
    assert_eq!(text(&nes[9]), "emusen", "the engine's kind");
    assert_eq!(nes[5], Cell::Whole(1));
    assert_eq!(nes[12..], [Cell::Whole(0), Cell::Whole(0), Cell::Whole(1), Cell::Whole(0)], "the one console is its maker's group alone, with none before it");

    let places = view::places(v, &view::lines_of(common::HOME_LINES), &[]);
    assert!(places.iter().all(|r| r.len() == view::PLACE_ROLES.len()));
    let ids: Vec<&str> = places.iter().map(|r| text(&r[0])).collect();
    assert_eq!(ids, ["games", "scraper", "packages", "library", "cores", "settings", "power"], "Home's destinations in Home's order");
    assert!(ids.iter().all(|key| !emusen_qt_models::marks::path(key).is_empty()), "every place has its mark in Marks.path");
    let kinds: Vec<&str> = places.iter().map(|r| text(&r[2])).collect();
    assert_eq!(kinds, ["library", "tool", "tool", "tool", "tool", "settings", "power"]);
    assert_eq!((text(&places[1][3]), &places[1][4]), ("41 no art", &Cell::Flag(true)), "the scraper's live line asks for the player");
    assert_eq!(places[0][5], Cell::Whole(1), "Games counts its consoles");
    assert!(places.iter().all(|r| r[6] == Cell::Flag(true)), "every place available while the host declines none");
    let declined = view::places(v, &[], &["packages".to_owned()]);
    let enabled: Vec<&Cell> = declined.iter().map(|r| &r[6]).collect();
    assert_eq!(enabled.iter().filter(|c| ***c == Cell::Flag(false)).count(), 1);
    assert_eq!(declined[2][6], Cell::Flag(false), "a place the host would decline is not available");

    let shelves = view::shelves(v);
    let seen: Vec<(&str, &Cell)> = shelves.iter().map(|r| (text(&r[0]), &r[3])).collect();
    assert_eq!(seen, [("all", &Cell::Whole(1)), ("favourites", &Cell::Whole(0)), ("recent", &Cell::Whole(0))]);

    for id in ["all", "nes"] {
        let games = view::games_of(v, id, 0);
        assert_eq!(games.len(), 1, "{id}");
        assert!(games.iter().all(|r| r.len() == view::GAME_ROLES.len()));
        let g = &games[0];
        assert!(text(&g[0]).ends_with("Synthetic.nes"), "the id is the game's file, which Actions take");
        assert_eq!((text(&g[1]), text(&g[7])), ("Synthetic", "nes"));
        assert_eq!(g[3], Cell::Number(1.0 / read.nes_shape), "the cover's shape is the console's, width over height where the table gives height over width");
        assert_eq!((&g[6], text(&g[16])), (&Cell::Flag(false), ""), "never played, so no words for it");
    }
    assert!(view::games_of(v, "favourites", 0).is_empty() && view::games_of(v, "recent", 0).is_empty());
    assert!(view::games_of(v, "no-such-console", 0).is_empty());
    assert_eq!(view::search(v, "SYNTH", 0).len(), 1, "a search in any case");
    assert!(view::search(v, "zelda", 0).is_empty() && view::search(v, "  ", 0).is_empty());
}

#[test]
fn qt_3_2_a_console_s_group_is_its_maker_s_in_order_of_first_appearance() {
    let makers = ["Nintendo", "Sega", "Nintendo", "", "Sony", "", "Sega", "Nintendo"];
    assert_eq!(
        view::maker_groups(&makers),
        [(0, 0, 3, 0), (1, 0, 2, 3), (0, 1, 3, 0), (2, 0, 1, 5), (3, 0, 1, 6), (4, 0, 1, 7), (1, 1, 2, 3), (0, 2, 3, 0)],
        "the ordinal, the place within, the size and the consoles of the groups before; an unknown maker is a group of its own"
    );
    let mut places: Vec<i32> = view::maker_groups(&makers).iter().map(|&(_, index, _, start)| start + index).collect();
    places.sort_unstable();
    assert_eq!(places, (0..8).collect::<Vec<_>>(), "start + index places each console once in maker order");
    assert!(view::maker_groups(&[]).is_empty());
}
