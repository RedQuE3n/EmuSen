//! Eclipse's games view in the running program on the software backend (EmuSen_Qt.md §3.4.10): the oracle of eclipse_games_oracle, its table, its walk and its pictures; and the empty lists' table's reader and the listing's actions.

mod common;
mod eclipse_board;
mod eclipse_games_oracle;
mod eclipse_pad;
#[path = "eclipse/table.rs"]
mod table;

#[test]
fn qt_3_4_10_eclipses_games_view_is_its_board_and_every_game_is_reached_in_the_program() {
    eclipse_games_oracle::oracle("software");
}

#[test]
fn qt_3_4_10_the_table_refuses_a_built_row_with_no_check() {
    let read = std::panic::catch_unwind(|| table::rows_of("a part\tEclipseLibrary:1\t1200\tthe program's\tbuilt\tpart\t\t").len());
    let words = read.expect_err("a built row with no check is refused").downcast::<String>().map(|s| *s).unwrap_or_default();
    assert!(words.contains("built, with no item or no check"), "{words}");
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
