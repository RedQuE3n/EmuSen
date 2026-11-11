// The shelves' one schema (EmuSen_Qt.md §4.9): what is written as an entry reads back as the same row, a member as a
// row with no name, and a key or a type either side does not name is an error.

use emusen_esde::shelves::{CountsText, Entry, GameRow, ShelfText};
use std::borrow::Cow;

fn row() -> GameRow<'static> {
    GameRow {
        alt_emulator: true,
        controller: Some("snes_mouse".into()),
        description: Some("A game invented for the tests.".into()),
        developer: Some("Studio A".into()),
        esde_system: "snes".into(),
        favourite: true,
        file: "/roms/snes/Aurora Drift.sfc".into(),
        last_played: 1_700_000_000,
        name: "Aurora Drift".into(),
        play_count: -1,
        play_time: 3_600,
        rating: Some(0.8),
        ..GameRow::default()
    }
}

fn shelf<G>(games: Vec<G>) -> ShelfText<'static, G> {
    ShelfText {
        collection_id: Some(4),
        counts: Some(CountsText { favorites: 1, filtered: true, games: 2, in_folder: false, shown: 1 }),
        favourites_on_top: false,
        folder: "".into(),
        full_name: "Super Nintendo".into(),
        games,
        key: "snes".into(),
        kind: "regular".into(),
        letters: Cow::Owned(vec![("A".into(), 0)]),
        name: "snes".into(),
        sort: "name, ascending".into(),
        stars: true,
        system: "snes".into(),
        theme: "snes".into(),
    }
}

#[test]
fn qt_4_9_a_shelf_written_reads_back_as_written_a_member_as_a_row_with_no_name() {
    let member = Entry::Member { esde_system: "snes".into(), file: "/roms/snes/Aurora Drift.sfc".into(), sort_name: None };
    let text = serde_json::to_string(&[shelf(vec![Entry::Row(row()), member])]).unwrap();
    let read: Vec<ShelfText> = serde_json::from_str(&text).unwrap();
    assert_eq!(read[0].games[0], row(), "a whole row, negative plays and all");
    assert!(read[0].games[0].is_whole() && !read[0].games[1].is_whole());
    assert_eq!(read[0].games[1].file, "/roms/snes/Aurora Drift.sfc");
    assert_eq!(read, vec![shelf(vec![row(), GameRow { esde_system: "snes".into(), file: "/roms/snes/Aurora Drift.sfc".into(), ..GameRow::default() }])]);
}

#[test]
fn qt_4_9_a_key_or_a_type_the_schema_does_not_name_is_an_error() {
    let text = serde_json::to_string(&[shelf(vec![Entry::Row(row())])]).unwrap();
    for (from, to) in [("\"developer\"", "\"maker\""), ("\"play_count\":-1", "\"play_count\":\"-1\""), ("\"stars\"", "\"starred\""), ("\"shown\"", "\"seen\"")] {
        let bad = text.replacen(from, to, 1);
        assert_ne!(bad, text, "{from} is in the text");
        assert!(serde_json::from_str::<Vec<ShelfText>>(&bad).is_err(), "{to} read without an error");
    }
}
