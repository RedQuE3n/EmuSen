//! What the Cheats tool's tests share: the focus read, a wait, the tool opened from Home by the pad and left, the List
//! card's row chosen, and the folder the base reads the player's database from.
#![allow(dead_code)]

use super::wait_for;
use emusen_client_fixture::library;
use emusen_crystal_client::consts::CHEAT_DB_STATUS;
use emusen_crystal_client::service_flags::NO_GAME;
use emusen_crystal_client::{Api, Config, Runtime};
use emusen_kit_core::input::UiButton;
use emusen_qt_test::Scene;
use serde_json::json;
use std::path::PathBuf;

pub use emusen_client_fixture::cheats::{GAME, SNES};

pub fn focused_words(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    let Some(focus) = scene.focus(&tree) else { return String::new() };
    tree.nodes.iter().skip(focus as usize).find_map(|n| n.text.clone()).unwrap_or_default()
}

pub fn focused(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).unwrap_or_default()
}

/// `done` within `ms`, else the test fails saying `what`.
pub fn until(scene: &mut Scene<'_>, what: &str, ms: i32, mut done: impl FnMut(&mut Scene<'_>) -> bool) {
    let mut waited = 0;
    while !done(scene) && waited < ms {
        scene.wait(25);
        waited += 25;
    }
    assert!(done(scene), "{what} within {ms} ms: rows {:?}, summary {:?}", scene.property("jobsModel", "rows"), scene.property("jobsModel", "summary"));
}

/// The Home card's Tools page's Cheats row, from Home, by the pad; the tool's read ended.
pub fn open_cheats(scene: &mut Scene<'_>) {
    scene.pad(UiButton::Menu);
    wait_for(scene, "screens", "shown", "HomeCard", 2000);
    scene.wait(300);
    for row in ["Tools", "Cheats"] {
        for _ in 0..12 {
            if focused_words(scene).starts_with(row) {
                break;
            }
            scene.pad(UiButton::Down);
            scene.wait(50);
        }
        assert!(focused_words(scene).starts_with(row), "no row reads {row}: {}", focused_words(scene));
        scene.pad(UiButton::Accept);
        scene.wait(300);
    }
    wait_for(scene, "screens", "shown", "Tool", 2000);
    wait_for(scene, "jobsModel", "title", "Cheats", 2000);
    wait_for(scene, "jobsModel", "busy", "false", 10000);
}

/// B on the tool screen: Home again, holding the pad.
pub fn back_home(scene: &mut Scene<'_>) {
    scene.pad(UiButton::Back);
    wait_for(scene, "screens", "shown", "Home", 2000);
    until(scene, "Home holds the pad", 2000, |s| focused(s).starts_with("place."));
}

/// The folder the base reads the player's database from, as its status says it.
pub fn database_folder() -> PathBuf {
    let rt = Runtime::new(&Api::load(library()).unwrap(), &Config::default()).unwrap();
    let status = rt.session_service(0, NO_GAME).unwrap().cheat_op(0, CHEAT_DB_STATUS, &json!({})).unwrap();
    PathBuf::from(status["directory"].as_str().expect("the database's folder"))
}

/// Select's List card, its row of `label` chosen by the pad.
pub fn list_choose(scene: &mut Scene<'_>, label: &str) {
    scene.pad(UiButton::Options);
    wait_for(scene, "jobsModel", "cardOpen", "true", 1000);
    scene.wait(300);
    let rows: serde_json::Value = serde_json::from_str(&scene.property("jobsModel", "cardRows")).unwrap();
    let at = rows.as_array().unwrap().iter().position(|r| r["label"] == label).unwrap_or_else(|| panic!("no row {label} on the List card: {rows}"));
    for _ in 0..at {
        scene.pad(UiButton::Down);
        scene.wait(80);
    }
    scene.wait(100);
    scene.pad(UiButton::Accept);
    wait_for(scene, "jobsModel", "cardOpen", "false", 1000);
}
