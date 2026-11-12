//! What failed, in the words the player is told (EmuSen_Qt.md §7.9, items 1 and 5): a game or a console that does not
//! open in Mistress's form, a game's page's change that is not made, and a firmware check that fails, which is said and
//! never read as "no firmware needed".

use emusen_qt_models::game_page::{change_failed, Change};
use emusen_qt_models::launch::firmware_unchecked;
use emusen_qt_models::session::{failing, Opening};

#[test]
fn a_game_or_a_console_that_does_not_open_is_said_in_mistress_s_form() {
    assert_eq!(failing(&Opening::Game("/roms/snes/Aurora Drift (Synthetic).sfc".into())), "Failed to load Aurora Drift (Synthetic)");
    assert_eq!(failing(&Opening::Console("PlayStation".into())), "Failed to start PlayStation");
}

#[test]
fn a_change_on_a_game_s_page_that_is_not_made_says_what_and_why() {
    assert_eq!(change_failed(&Change::Hide, "games.db is read-only"), "The game could not be hidden: games.db is read-only");
    assert_eq!(change_failed(&Change::DeleteState("/s/1.state".into()), "permission denied"), "The state could not be deleted: permission denied");
    assert_eq!(change_failed(&Change::Favourite, "x"), "The favourite could not be changed: x");
    assert_eq!(change_failed(&Change::Standard(None), "x"), "The game's television standard could not be stored: x");
}

#[test]
fn a_failed_firmware_check_is_said() {
    assert_eq!(firmware_unchecked("no route"), "The firmware this game needs could not be checked, so it opens without asking: no route");
}
