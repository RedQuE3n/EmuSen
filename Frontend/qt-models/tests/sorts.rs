//! The List card's Most played, Year and Rating (EmuSen_Qt.md §7.9, item 4): a refresh reads games.db's plays and the
//! scraper's release date and rating once, the grid's rows carry them, and each sort orders by its datum, a game without
//! it last and a tie by name, as Mistress's `GamesScreen.Sorted` does.

use emusen_crystal_client::shapes::SystemRow;
use emusen_platform_client::library::GameFile;
use emusen_qt_models::esde_shelves::{rating, release_date, year_of, Metadata, Plays};
use emusen_qt_models::games::{from_json, to_json};
use emusen_qt_models::library::{games_of, refresh, Engine, LibrarySource};
use emusen_qt_models::listing::{shown, Listing, SORTS};

struct Source;

const GAMES: [&str; 5] = ["/r/Aurora.nes", "/r/brass.nes", "/r/Cobalt.nes", "/r/Dune.nes", "/r/Ember.nes"];

impl LibrarySource for Source {
    fn systems(&self) -> Result<Vec<SystemRow>, String> {
        Ok(vec![SystemRow { id: "nes".into(), name: "NES".into(), engines: vec!["Venus".into()], ..SystemRow::default() }])
    }
    fn games(&self) -> Result<Vec<GameFile>, String> {
        Ok(GAMES.iter().map(|p| GameFile { path: (*p).into(), system: Some("nes".into()), ..GameFile::default() }).collect())
    }
    fn favourites(&self, _: &[GameFile]) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }
    fn recent(&self, _: &[GameFile]) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }
    fn engines(&self, _: &[SystemRow]) -> Result<Vec<(String, Engine)>, String> {
        Ok(Vec::new())
    }
    fn plays(&self, _: &[GameFile]) -> Result<Vec<(String, Plays)>, String> {
        Ok(vec![("/r/Cobalt.nes".into(), Plays { count: 9, seconds: 60 }), ("/r/Ember.nes".into(), Plays { count: 3, seconds: 1 }), ("/r/brass.nes".into(), Plays { count: 3, seconds: 5 })])
    }
    fn metadata(&self, games: &[GameFile]) -> Result<Vec<(String, Metadata)>, String> {
        let of = |date: &str, rating: &str| -> Metadata {
            [("releasedate", date), ("rating", rating)].iter().filter(|(_, v)| !v.is_empty()).map(|(k, v)| (k.to_string(), v.to_string())).collect()
        };
        let table = [("/r/Aurora.nes", "1994-03-01", "0.6"), ("/r/brass.nes", "1987-07-10", ""), ("/r/Dune.nes", "1994-01-01", "0.9"), ("/r/Ember.nes", "", "0.6")];
        Ok(games.iter().filter_map(|g| table.iter().find(|t| t.0 == g.path).map(|t| (g.path.clone(), of(t.1, t.2)))).collect())
    }
}

fn sorted(name: &str) -> Vec<String> {
    let read = refresh(&Source).unwrap();
    // The rows as GameModel reads them: the data crosses the plain text between the two models.
    let rows = from_json(&to_json(&games_of("all", &read.systems, &read.catalog)));
    let sort = SORTS.iter().position(|s| *s == name).unwrap();
    shown(&rows, "all", &Listing { sort, ..Listing::default() }).into_iter().map(|g| g.title).collect()
}

#[test]
fn most_played_year_and_rating_sort_by_their_data_without_it_last_and_ties_by_name() {
    assert_eq!(sorted("Name"), ["Aurora", "brass", "Cobalt", "Dune", "Ember"]);
    assert_eq!(sorted("Most played"), ["Cobalt", "brass", "Ember", "Aurora", "Dune"], "most first; a tie by name; never played last");
    assert_eq!(sorted("Year"), ["brass", "Aurora", "Dune", "Cobalt", "Ember"], "oldest first by year alone, a tie by name; no date last");
    assert_eq!(sorted("Rating"), ["Dune", "Aurora", "Ember", "brass", "Cobalt"], "highest first; a tie by name; no rating last");
}

#[test]
fn the_scraper_s_date_and_rating_have_one_reader() {
    let m: Metadata = [("releasedate".to_string(), "1991-11-21".to_string()), ("rating".to_string(), "1.4".to_string())].into_iter().collect();
    assert_eq!(release_date(&m).as_deref(), Some("1991-11-21"));
    assert_eq!(rating(&m), Some(1.0), "clamped to 0 to 1");
    assert_eq!((year_of("1991-11-21"), year_of("19x1"), year_of("")), (Some("1991".into()), None, None));
    assert_eq!((release_date(&Metadata::new()), rating(&Metadata::new())), (None, None));
}
