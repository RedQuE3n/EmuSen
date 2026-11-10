//! ES-DE's gamelist rules with no view behind them (EmuSen_Qt.md §4.7): Mistress's `GamelistOptionsTests`,
//! `ThemedCollectionsTests` and `ThemedFoldersTests` cases that are rules, re-expressed over plain data.

use emusen_qt_models::esde_shelves::{build, Base, EsdeGame};
use emusen_qt_models::gamelists::*;
use emusen_qt_models::list_card::esde_facts;
use std::collections::HashMap;
use std::sync::Arc;

fn g(name: &str) -> EsdeGame {
    EsdeGame { name: name.into(), file: format!("/roms/snes/{name}.sfc"), esde_system: "snes".into(), esde_full_name: "Super Nintendo".into(), ..EsdeGame::default() }
}

fn names(games: &[EsdeGame]) -> Vec<&str> {
    games.iter().map(|g| g.name.as_str()).collect()
}

fn sorted(mut games: Vec<EsdeGame>, order: GameSort, favourites_first: bool) -> Vec<EsdeGame> {
    sort(&mut games, order, favourites_first);
    games
}

#[test]
fn every_documented_sort_key_orders_with_missing_values_last_and_the_name_ascending_after() {
    let rated = |n: &str, r: Option<f64>| EsdeGame { rating: r, ..g(n) };
    let games = vec![rated("Delta", Some(0.6)), rated("alpha", Some(0.8)), rated("Charlie", None), rated("Bravo", Some(0.6)), rated("Echo", Some(0.2))];
    assert_eq!(names(&sorted(games.clone(), GameSort::default(), false)), ["alpha", "Bravo", "Charlie", "Delta", "Echo"]);
    assert_eq!(names(&sorted(games.clone(), GameSort::new(SortKey::Name, true), false)), ["Echo", "Delta", "Charlie", "Bravo", "alpha"]);
    assert_eq!(names(&sorted(games.clone(), GameSort::new(SortKey::Rating, false), false)), ["Echo", "Bravo", "Delta", "alpha", "Charlie"]);
    assert_eq!(names(&sorted(games, GameSort::new(SortKey::Rating, true), false)), ["alpha", "Bravo", "Delta", "Echo", "Charlie"]);

    // "1-4" is four players; a text without a number goes last with the games that have none.
    let players = |n: &str, p: Option<&str>| EsdeGame { players: p.map(str::to_owned), ..g(n) };
    let party = vec![players("One", Some("1")), players("Four", Some("1-4")), players("Two", Some("2")), players("None", None), players("Many", Some("lots"))];
    assert_eq!(names(&sorted(party, GameSort::new(SortKey::Players, true), false)), ["Four", "Two", "One", "Many", "None"]);

    // Favourites on top when asked, each part in the order.
    let fav = |n: &str| EsdeGame { favourite: true, ..g(n) };
    let favs = vec![g("Bravo"), g("Alpha"), fav("Zulu"), fav("Mike")];
    assert_eq!(names(&sorted(favs.clone(), GameSort::default(), true)), ["Mike", "Zulu", "Alpha", "Bravo"]);
    assert_eq!(names(&sorted(favs, GameSort::default(), false)), ["Alpha", "Bravo", "Mike", "Zulu"]);

    // System, for collections, by the source console's full name; same names keep the order they came in.
    let from = |n: &str, s: &str, full: &str| EsdeGame { esde_system: s.into(), esde_full_name: full.into(), ..g(n) };
    let mixed = vec![from("Same", "snes", "Super Nintendo"), from("Same", "nes", "Nintendo Entertainment System"), from("Other", "snes", "Super Nintendo")];
    let by_system = sorted(mixed.clone(), GameSort::new(SortKey::System, false), false);
    assert_eq!(names(&by_system), ["Same", "Other", "Same"]);
    assert_eq!(by_system.iter().map(|g| g.esde_system.as_str()).collect::<Vec<_>>(), ["nes", "snes", "snes"]);
    assert_eq!(sorted(mixed[..2].to_vec(), GameSort::default(), false).iter().map(|g| g.esde_system.as_str()).collect::<Vec<_>>(), ["snes", "nes"]);

    // The stored spelling is ES-DE's DefaultSortOrder value, and System is offered for collections only.
    assert_eq!(GameSort::new(SortKey::ReleaseDate, true).stored(), "release date, descending");
    assert_eq!(GameSort::parse("times played, descending"), GameSort::new(SortKey::TimesPlayed, true));
    assert_eq!(GameSort::parse("nonsense"), GameSort::default());
    assert!(!GameSort::all(false).iter().any(|s| s.key == SortKey::System));
    assert_eq!(GameSort::all(true).len(), 22);
}

#[test]
fn a_sort_name_orders_the_name_sort_and_indexes_the_quick_selector_and_an_uncounted_game_joins_no_collection() {
    let punisher = EsdeGame { sort_name: Some("Punisher, The".into()), ..g("The Punisher") };
    let list = sorted(vec![punisher, g("Alpha"), g("Quest")], GameSort::default(), false);
    assert_eq!(names(&list), ["Alpha", "The Punisher", "Quest"]);
    assert_eq!(letters(&list, false, false), [("A".to_owned(), 0), ("P".to_owned(), 1), ("Q".to_owned(), 2)]);

    let counted = EsdeGame { file: "/s/a.sfc".into(), favourite: true, ..g("Counted") };
    let setup = EsdeGame { file: "/s/b.sfc".into(), favourite: true, not_counted: true, ..g("Setup") };
    let shelf = Shelf::regular("snes", "snes", "Super Nintendo", vec![counted, setup]);
    let settings = Settings { auto: vec![ALL_GAMES.into(), FAVORITES.into()], ..Settings::default() };
    let custom = [Collection { id: 1, name: "Mine".into(), members: vec!["/s/a.sfc".into(), "/s/b.sfc".into()] }];
    let shelves = with_collections(vec![shelf], &custom, &settings, &|_| true, &[]);
    for s in &shelves[1..] {
        assert_eq!(names(&s.games), ["Counted"], "{}", s.name);
    }
    assert_eq!(shelves[0].games.len(), 2);
}

#[test]
fn filter_values_come_from_the_list_and_a_field_with_nothing_to_tell_apart_offers_none() {
    let gr = |n: &str, genre: Option<&str>, rating: Option<f64>, fav: bool| EsdeGame { genre: genre.map(str::to_owned), rating, favourite: fav, ..g(n) };
    let games = vec![gr("A", Some("Racing"), Some(0.8), false), gr("B", Some("puzzle"), Some(0.8), false), gr("C", Some("Racing"), None, false), gr("D", None, None, true)];
    assert_eq!(values(&games, FilterField::Genre), ["puzzle", "Racing", UNKNOWN]);
    assert_eq!(values(&games, FilterField::Rating), ["4 stars", UNKNOWN]);
    assert_eq!(values(&games, FilterField::Favorite), ["No", "Yes"]);
    assert!(values(&games, FilterField::Publisher).is_empty());
    assert!(values(&games, FilterField::Completed).is_empty());
    assert!(values(&games[..1], FilterField::Genre).is_empty());
    assert_eq!(stars(0.5), "2.5 stars");
    assert_eq!(stars(0.2), "1 star");

    // Any chosen value within a field, every field, the name anywhere in any case; a folder always passes.
    let kept = |f: &GameFilter| games.iter().filter(|x| passes(x, f)).map(|x| x.name.as_str()).collect::<Vec<_>>();
    let filter = GameFilter::default().with(FilterField::Genre, &["Racing", UNKNOWN]);
    assert_eq!(kept(&filter), ["A", "C", "D"]);
    let filter = filter.with(FilterField::Favorite, &["No"]);
    assert_eq!(kept(&filter), ["A", "C"]);
    assert_eq!(kept(&GameFilter { name: "c".into(), ..filter.clone() }), ["C"]);
    assert!(passes(&EsdeGame { folder: true, ..g("X") }, &GameFilter { name: "zzz".into(), ..filter.clone() }));
    assert!(!GameFilter::default().is_active());
    assert!(!filter.with(FilterField::Genre, &[]).with(FilterField::Favorite, &[]).is_active());
}

#[test]
fn the_quick_selector_offers_only_the_letters_present_and_a_star_for_favourites_on_top() {
    let list = sorted(vec![EsdeGame { favourite: true, ..g("Kestrel") }, g("aurora"), g("Brass"), g("Bolt"), g("9 Lives")], GameSort::default(), true);
    let l = |pairs: &[(&str, usize)]| pairs.iter().map(|(a, b)| ((*a).to_owned(), *b)).collect::<Vec<_>>();
    assert_eq!(letters(&list, true, false), l(&[(STAR, 0), ("9", 1), ("A", 2), ("B", 3)]));
    assert_eq!(letters(&sorted(list, GameSort::default(), false), false, false), l(&[("9", 0), ("A", 1), ("B", 2), ("K", 4)]));
    assert_eq!(letters(&[EsdeGame { favourite: true, ..g("Kestrel") }], true, false), l(&[("K", 0)]), "only favourites: no star");
}

#[test]
fn the_random_entry_never_picks_the_game_already_selected_when_there_is_another() {
    let picks: std::collections::BTreeSet<usize> = (0..400u64).map(|d| random_index(5, 2, d * 2654435761)).collect();
    assert!(!picks.contains(&2));
    assert_eq!(picks.into_iter().collect::<Vec<_>>(), [0, 1, 3, 4]);
    assert_eq!(random_index(1, 0, 99), 0);
}

#[test]
fn collection_names_lose_es_de_s_forbidden_characters_and_a_taken_name_is_numbered() {
    assert_eq!(clean("Beat: em/ Up*"), "Beat em Up");
    assert_eq!(clean("a*\",./:;<>\\|b"), "ab");
    assert_eq!(unique("Platform", &["platform"]), "Platform (1)");
    assert_eq!(unique("Platform", &["Platform", "Platform (1)"]), "Platform (2)");
    assert_eq!(unique("New", &["Old"]), "New");
}

fn two_consoles() -> (Shelf, Shelf, [Collection; 2]) {
    let nes = Shelf::regular("nes", "nes", "Nintendo Entertainment System", vec![EsdeGame { file: "/n/fable.nes".into(), esde_system: "nes".into(), ..g("Fable") }]);
    let snes = Shelf::regular("snes", "snes", "Super Nintendo", vec![EsdeGame { file: "/s/a.sfc".into(), favourite: true, ..g("Aurora") }, EsdeGame { file: "/s/b.sfc".into(), ..g("Brass") }]);
    let custom = [
        Collection { id: 1, name: "Platform".into(), members: vec!["/s/a.sfc".into(), "/n/fable.nes".into(), "/gone.sfc".into()] },
        Collection { id: 2, name: "Beat".into(), members: vec!["/s/b.sfc".into()] },
    ];
    (nes, snes, custom)
}

#[test]
fn collection_shelves_follow_es_de_s_measured_order_and_group_what_the_theme_does_not_style() {
    let (nes, snes, custom) = two_consoles();
    let mut settings = Settings { auto: vec![LAST_PLAYED.into(), ALL_GAMES.into(), FAVORITES.into()], ..Settings::default() };
    let order = |s: &Settings, themed: &dyn Fn(&str) -> bool| with_collections(vec![nes.clone(), snes.clone()], &custom, s, themed, &[]).into_iter().map(|s| s.name).collect::<Vec<_>>();
    let shelves = with_collections(vec![nes.clone(), snes.clone()], &custom, &settings, &|_| false, &[]);
    assert_eq!(shelves.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["nes", "snes", "collections", "all", "favorites", "recent"]);
    let grouped = shelves[2].folders.as_ref().unwrap();
    assert_eq!(grouped.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["Beat", "Platform"]);
    let mut platform = names(&grouped[1].games);
    platform.sort();
    assert_eq!(platform, ["Aurora", "Fable"]);
    assert_eq!(grouped[1].games.iter().find(|x| x.name == "Fable").unwrap().esde_system, "nes");
    assert!(grouped.iter().all(|f| !f.stars));
    assert_eq!((shelves[3].games.len(), names(&shelves[4].games), shelves[4].stars, shelves[3].stars), (3, vec!["Aurora"], false, true));

    assert_eq!(order(&settings, &|n| n == "Platform"), ["nes", "snes", "collections", "Platform", "all", "favorites", "recent"], "a theme folder of its name: a system of its own");
    settings.group_custom = GROUP_ALWAYS.into();
    assert_eq!(order(&settings, &|n| n == "Platform"), ["nes", "snes", "collections", "all", "favorites", "recent"]);
    settings.group_custom = GROUP_NEVER.into();
    assert_eq!(order(&settings, &|_| false), ["nes", "snes", "Beat", "Platform", "all", "favorites", "recent"]);
    assert_eq!(with_collections(vec![nes.clone(), snes.clone()], &custom, &settings, &|_| false, &[])[2].kind, Kind::Custom);
    settings.hidden_custom = vec![1, 2];
    settings.auto = Vec::new();
    assert_eq!(order(&settings, &|_| false), ["nes", "snes"], "hidden collections and no automatic ones: the consoles alone");
}

#[test]
fn last_played_is_the_library_s_recent_list_in_its_order_less_the_games_no_collection_holds() {
    let file = |n: &str, f: &str| EsdeGame { file: f.into(), ..g(n) };
    let games = vec![file("Older", "/s/o.sfc"), file("Newer", "/s/n.sfc"), EsdeGame { not_counted: true, ..file("Setup", "/s/u.sfc") }, EsdeGame { play_count: 9, ..file("Counted", "/s/c.sfc") }];
    let settings = Settings { auto: vec![LAST_PLAYED.into()], ..Settings::default() };
    let recent: Vec<String> = ["/s/n.sfc", "/s/u.sfc", "/gone.sfc", "/s/o.sfc"].map(str::to_owned).to_vec();
    let shelves = with_collections(vec![Shelf::regular("snes", "snes", "Super Nintendo", games)], &[], &settings, &|_| false, &recent);
    let last = shelves.last().unwrap();
    assert_eq!((last.name.as_str(), last.theme.as_str()), (LAST_PLAYED, "auto-lastplayed"));
    assert_eq!(names(&last.games), ["Newer", "Older"]);
    assert!(last.in_given_order && !last.favourites_first);
    assert_eq!(last.default_sort, Some(GameSort::new(SortKey::LastPlayed, true)));
    // Listed as given until the player picks a sort.
    let mut lists = Gamelists::default();
    assert_eq!(names(&lists.listing(last, &HashMap::new(), &settings).entries), ["Newer", "Older"]);
    lists.sorts.insert(LAST_PLAYED.into(), GameSort::new(SortKey::Name, true));
    assert_eq!(names(&lists.listing(last, &HashMap::new(), &settings).entries), ["Older", "Newer"], "in the order chosen once one is");
}

#[test]
fn a_grouped_collection_s_folder_names_its_games_in_a_random_order_and_shows_the_first() {
    let from = |n: &str, s: &str| EsdeGame { esde_system: s.into(), file: format!("/{s}/{n}"), ..g(n) };
    let platform = Shelf { name: "Platform".into(), ..Shelf::regular("", "Platform", "Platform", vec![from("Hollow Comet", "snes"), from("Ember Circuit", "snes"), from("Fable", "nes")]) };
    let folder = collection_folder(&platform, 1);
    assert!(folder.folder && folder.is_collection && folder.hide_metadata);
    assert_eq!((folder.name.as_str(), folder.file.as_str()), ("Platform", "collection:Platform"));
    let description = folder.description.clone().unwrap();
    assert!(description.starts_with("This collection contains 3 games: '"), "{description}");
    assert!(description.contains("'Fable [NES]'") && description.contains("' and '"), "{description}");
    let first = description.split(": '").nth(1).unwrap().split(" [").next().unwrap();
    assert_eq!(folder.face.as_deref(), Some(platform.games.iter().find(|x| x.name == first).unwrap().file.as_str()), "the first named is the one shown");
    let one = Shelf { name: "One".into(), ..Shelf::regular("", "One", "One", vec![from("Solo", "snes")]) };
    assert_eq!(collection_folder(&one, 1).description.as_deref(), Some("This collection contains 1 game: 'Solo [SNES]'"));
    let none = Shelf { name: "None".into(), ..Shelf::regular("", "None", "None", Vec::new()) };
    assert_eq!(collection_folder(&none, 1).description.as_deref(), Some("This collection contains 0 games"));
    assert!((0..20).any(|seed| collection_folder(&platform, seed).face != folder.face), "some seed puts another game first");
}

/// A console with games at its top and in two levels of folders, one folder with a link.
fn foldered() -> (Shelf, HashMap<String, String>) {
    let at = |n: &str, folder: &str, fav: bool| EsdeGame {
        file: if folder.is_empty() { format!("/roms/snes/{n}.sfc") } else { format!("/roms/snes/{folder}/{n}.sfc") },
        folder_path: folder.into(),
        favourite: fav,
        ..g(n)
    };
    let games = vec![at("Top", "", false), at("Fav", "", true), at("Spire", "Tower Set", false), at("Gate", "Tower Set", false), at("Deep", "RPG/Square", false), at("Uncounted", "RPG", false)];
    let games = games.into_iter().map(|x| if x.name == "Uncounted" { EsdeGame { not_counted: true, ..x } } else { x }).collect();
    let links = HashMap::from([("/roms/snes/Tower Set".to_owned(), "/roms/snes/Tower Set/Spire.sfc".to_owned())]);
    (Shelf::regular("snes", "snes", "Super Nintendo", games), links)
}

#[test]
fn a_console_s_folders_are_entries_on_top_entered_and_left_and_a_link_rides_on_its_folder() {
    let (shelf, links) = foldered();
    let settings = Settings::default();
    let mut lists = Gamelists::default();
    let top = lists.listing(&shelf, &links, &settings);
    assert_eq!(names(&top.entries), ["RPG", "Tower Set", "Fav", "Top"], "folders on top by name, then the favourite, then by name");
    let tower = &top.entries[1];
    assert_eq!((tower.folder, tower.folder_path.as_str(), tower.file.as_str()), (true, "Tower Set", "/roms/snes/Tower Set"));
    assert_eq!(tower.folder_link.as_deref(), Some("/roms/snes/Tower Set/Spire.sfc"), "the folder's link");
    assert_eq!(top.letters[0], (FOLDER_ENTRY.to_owned(), 0));
    assert_eq!(top.counts, Counts { games: 5, favorites: 1, shown: 5, filtered: false, in_folder: false }, "every game of every folder, less the uncounted");

    assert!(lists.enter(&shelf, &top.entries[0]));
    let rpg = lists.listing(&shelf, &links, &settings);
    assert_eq!((names(&rpg.entries), rpg.folder.as_str(), rpg.counts.in_folder), (vec!["Square", "Uncounted"], "RPG", true));
    assert!(lists.enter(&shelf, &rpg.entries[0]));
    assert_eq!(names(&lists.listing(&shelf, &links, &settings).entries), ["Deep"]);
    assert!(!lists.enter(&shelf, &g("Deep")), "a game is not entered");
    assert!(lists.leave(&shelf));
    assert_eq!(lists.listing(&shelf, &links, &settings).folder, "RPG");
    assert!(lists.leave(&shelf));
    assert!(!lists.leave(&shelf), "at the top B is the view's");

    // A filter that empties the folder the list is in walks it up; the filter holds over every folder.
    assert!(lists.enter(&shelf, &top.entries[1]));
    lists.search = "deep".into();
    let searched = lists.listing(&shelf, &links, &settings);
    assert_eq!((searched.folder.as_str(), names(&searched.entries)), ("", vec!["RPG"]));
    assert_eq!(searched.counts, Counts { games: 5, favorites: 1, shown: 1, filtered: true, in_folder: false });
    lists.search.clear();

    // Flattened, or folders not on top.
    let flat = Shelf { flatten: true, ..shelf.clone() };
    assert_eq!(names(&Gamelists::default().listing(&flat, &links, &settings).entries), ["Fav", "Deep", "Gate", "Spire", "Top", "Uncounted"]);
    let mixed = Settings { folders_on_top: false, ..Settings::default() };
    assert_eq!(names(&Gamelists::default().listing(&shelf, &links, &mixed).entries), ["Fav", "RPG", "Top", "Tower Set"]);
}

#[test]
fn a_console_s_text_carries_the_games_its_level_does_not_list_for_es_de_s_gameselectors() {
    let (shelf, links) = foldered();
    let mut lists = Gamelists::default();
    let top = lists.listing(&shelf, &links, &Settings::default());
    let text = listing_json(&shelf, &top);
    let unlisted: Vec<&str> = text["unlisted"].as_array().unwrap().iter().map(|g| g["name"].as_str().unwrap()).collect();
    assert_eq!(unlisted, ["Spire", "Gate", "Deep", "Uncounted"], "every game the level does not list, in the shelf's order, none twice");
    assert!(text["unlisted"].as_array().unwrap().iter().all(|g| g["folder_path"].as_str().is_some_and(|p| !p.is_empty())));
    assert!(lists.enter(&shelf, &top.entries[1]));
    let tower = lists.listing(&shelf, &links, &Settings::default());
    let unlisted: Vec<&str> = listing_json(&shelf, &tower)["unlisted"].as_array().unwrap().iter().map(|g| g["name"].as_str().unwrap()).collect();
    assert_eq!(unlisted, ["Top", "Fav", "Deep", "Uncounted"], "inside a folder, the games outside it");
}

#[test]
fn a_grouped_collection_is_entered_and_left_and_keeps_its_own_sort() {
    let (nes, snes, custom) = two_consoles();
    let settings = Settings { auto: Vec::new(), ..Settings::default() };
    let shelves = with_collections(vec![nes, snes], &custom, &settings, &|_| false, &[]);
    let grouped = &shelves[2];
    let mut lists = Gamelists::default();
    let top = lists.listing(grouped, &HashMap::new(), &settings);
    assert_eq!(names(&top.entries), ["Beat", "Platform"]);
    assert_eq!(top.counts.games, 3, "every game the collections hold");
    assert!(lists.enter(grouped, &top.entries[1]));
    let inside = lists.listing(grouped, &HashMap::new(), &settings);
    assert_eq!((inside.key.as_str(), names(&inside.entries)), ("collections/Platform", vec!["Aurora", "Fable"]));
    lists.step_sort(grouped, &settings, 1);
    assert_eq!(lists.listing(grouped, &HashMap::new(), &settings).sort, GameSort::new(SortKey::Name, true));
    assert_eq!(names(&lists.listing(grouped, &HashMap::new(), &settings).entries), ["Fable", "Aurora"]);
    assert!(lists.leave(grouped));
    assert_eq!(lists.listing(grouped, &HashMap::new(), &settings).sort, GameSort::default(), "the top keeps its own");
    assert_eq!(GameSort::all(grouped.is_collection()).len(), 22, "a collection offers System");
}

#[test]
fn the_list_card_s_sort_and_filter_change_the_text_s_order_and_its_facts() {
    let (nes, snes, _) = two_consoles();
    let base = Arc::new(Base { regular: vec![nes, snes], settings: Settings { auto: Vec::new(), ..Settings::default() }, ..Base::default() });
    let mut lists = Gamelists::default();
    let order = |text: &str| -> Vec<String> {
        let shelves: serde_json::Value = serde_json::from_str(text).unwrap();
        shelves[1]["games"].as_array().unwrap().iter().map(|x| x["name"].as_str().unwrap().to_owned()).collect()
    };
    let (esde, state) = build(base.clone(), &lists);
    assert_eq!(order(&esde.shelves), ["Aurora", "Brass"]);
    let facts = esde_facts(&state.summaries["snes"], 1);
    assert_eq!((facts["sort"].as_str(), facts["letter"].as_str(), facts["count"].as_str(), facts["letters"].as_str()), (Some("Name, ascending"), Some("B"), Some("2 games"), Some("★ · B")));
    assert_eq!(facts["sorts"].as_array().unwrap().len(), 20, "a console offers every order but System");

    let snes = state.shelf("snes").unwrap().clone();
    assert_eq!(lists.step_sort(&snes, &base.settings, 1), GameSort::new(SortKey::Name, true), "Sort steps to the next order");
    let (esde, state) = build(base.clone(), &lists);
    assert_eq!(order(&esde.shelves), ["Aurora", "Brass"], "favourites stay first");
    lists.favourites_only(&snes, true);
    let (esde, state2) = build(base.clone(), &lists);
    assert_eq!(order(&esde.shelves), ["Aurora"], "Favourites narrows the list");
    let facts = esde_facts(&state2.summaries["snes"], 0);
    assert_eq!((facts["filter"].as_str(), facts["count"].as_str()), (Some("Favourites"), Some("1 of 2 games")));
    lists.favourites_only(&snes, false);
    assert_eq!(order(&build(base, &lists).0.shelves), ["Aurora", "Brass"]);
    assert_eq!(state.summaries["snes"].sort, "Name, descending");
}
