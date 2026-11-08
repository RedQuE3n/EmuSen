//! The painter without Qt (EmuSen_Qt.md §4.8): the library's texts read into the scene, the theme cache, the clock, and
//! the pad's and the pointer's grammar on a synthetic theme written for the test.

use std::collections::BTreeMap;
use std::path::PathBuf;

use emusen_esde::scene::{DateTime, Layout};
use emusen_esde_qt::painter::{systems, Action, Painter, ThemeCache};
use emusen_kit_core::input::UiButton as Button;
use emusen_esde_qt::shelves::{self, from_unix, Settings};

const W: f64 = 1280.0;
const H: f64 = 800.0;

struct Theme(PathBuf);

impl Theme {
    // A carousel of systems and a text list of games, nothing copied from a real theme.
    fn new(tag: &str) -> Theme {
        let root = std::env::temp_dir().join(format!("esde-qt-painter-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("capabilities.xml"), "<themeCapabilities></themeCapabilities>").unwrap();
        std::fs::write(
            root.join("theme.xml"),
            "<theme><view name=\"system\"><carousel name=\"systems\"><pos>0 0.3</pos><size>1 0.4</size><maxItemCount>3</maxItemCount></carousel></view>\
             <view name=\"gamelist\"><textlist name=\"games\"><pos>0.1 0.1</pos><size>0.8 0.8</size></textlist></view></theme>",
        )
        .unwrap();
        Theme(root)
    }
}

impl Drop for Theme {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn shelf(name: &str, full: &str, games: &[&str]) -> String {
    let rows: Vec<String> = games
        .iter()
        .map(|g| format!("{{\"file\":\"{g}.sfc\",\"name\":\"{g}\",\"esde_system\":\"{name}\",\"favourite\":false,\"last_played\":0}}"))
        .collect();
    format!("{{\"name\":\"{name}\",\"full_name\":\"{full}\",\"theme\":\"{name}\",\"kind\":\"regular\",\"games\":[{}]}}", rows.join(","))
}

fn library() -> String {
    format!(
        "[{},{},{}]",
        shelf("nes", "Nintendo", &["Aurora Drift", "Brass Lantern"]),
        shelf("snes", "Super Nintendo", &["Cobalt Harbor", "Dune Relay", "Ember Circuit", "Fable of Tiles"]),
        shelf("n64", "Nintendo 64", &["Granite Choir"])
    )
}

fn painter(theme: &Theme) -> Painter {
    let text = library();
    let settings = Settings { theme: theme.0.to_string_lossy().into_owned(), ..Settings::default() };
    let cache = ThemeCache::default();
    let found = systems(&cache, &settings, shelves::shelves(&text, "{}"), &BTreeMap::new(), W as i32, H as i32);
    let mut p = Painter::new(settings);
    p.install(found, W, H, DateTime::new(2026, 11, 8, 12, 0, 0));
    p
}

fn selected(p: &Painter) -> usize {
    p.frame.primary.as_ref().expect("a primary element").selected
}

#[test]
fn the_settings_read_mistress_s_keys() {
    let s = shelves::settings(
        "{\"theme\":\"/t\",\"choices\":{\"Variant\":\"v\",\"ColorScheme\":\"dark\",\"FontSize\":\"medium\"},\"interface\":{\"DisplayClock\":true,\"QuickSystemSelect\":\"leftrighttriggers\"}}",
    );
    assert_eq!((s.theme.as_str(), s.choices.variant.as_deref(), s.choices.color_scheme.as_deref()), ("/t", Some("v"), Some("dark")));
    assert_eq!(s.choices.font_size.as_deref(), Some("medium"));
    assert!(s.show_clock && s.show_help, "the clock as set, the help as defaulted");
    assert_eq!(s.quick_select, "leftrighttriggers");
    assert_eq!(shelves::settings("not json"), Settings::default());
}

#[test]
fn a_collection_s_game_takes_its_console_s_row_and_media_by_folder_then_stem() {
    let text = "[{\"name\":\"snes\",\"full_name\":\"Super Nintendo\",\"theme\":\"snes\",\"kind\":\"regular\",\"games\":[\
        {\"file\":\"Tower/Aurora Drift.sfc\",\"name\":\"Aurora Drift\",\"esde_system\":\"snes\",\"folder_path\":\"Tower\",\"favourite\":true,\"last_played\":1700000000,\"release_date\":\"1990-01-01\"},\
        {\"file\":\"Cobalt.sfc\",\"name\":\"Cobalt\",\"esde_system\":\"snes\"}]},\
        {\"name\":\"favorites\",\"full_name\":\"Favorites\",\"theme\":\"favorites\",\"kind\":\"auto\",\"games\":[{\"file\":\"Tower/Aurora Drift.sfc\",\"esde_system\":\"snes\"}]}]";
    let media = "{\"snes\":{\"covers\":{\"Tower/Aurora Drift\":\"/m/tower.png\",\"Aurora Drift\":\"/m/stem.png\",\"Cobalt\":\"/m/cobalt.png\"}}}";
    let found = shelves::shelves(text, media);
    assert_eq!(found.len(), 2);
    let aurora = &found[0].games[0];
    assert_eq!(aurora.media.get("covers").map(String::as_str), Some("/m/tower.png"), "the folder and stem first");
    assert_eq!(found[0].games[1].media.get("covers").map(String::as_str), Some("/m/cobalt.png"), "then the stem");
    assert_eq!(aurora.last_played, Some(DateTime::new(2023, 11, 14, 22, 13, 20)));
    assert_eq!(aurora.release_date, Some(DateTime::new(1990, 1, 1, 0, 0, 0)));
    let member = &found[1].games[0];
    assert_eq!((member.name.as_str(), member.favorite), ("Aurora Drift", true), "the collection's entry is its console's row");
    assert_eq!(member.source.as_ref().map(|s| s.name.as_str()), Some("snes"));
    assert_eq!(from_unix(0), DateTime::new(1970, 1, 1, 0, 0, 0));
}

#[test]
fn the_cache_loads_a_system_s_theme_once_per_choices() {
    let theme = Theme::new("cache");
    let settings = Settings { theme: theme.0.to_string_lossy().into_owned(), ..Settings::default() };
    let cache = ThemeCache::default();
    let text = format!("[{}]", shelf("snes", "Super Nintendo", &["Cobalt Harbor"]));
    let first = systems(&cache, &settings, shelves::shelves(&text, "{}"), &BTreeMap::new(), 1280, 800);
    std::fs::write(theme.0.join("theme.xml"), "<theme><view name=\"system\"></view></theme>").unwrap();
    let again = systems(&cache, &settings, shelves::shelves(&text, "{}"), &BTreeMap::new(), 1280, 800);
    assert_eq!(first[0].theme.files_read, again[0].theme.files_read);
    assert!(again[0].theme.system_view.elements.iter().any(|e| e.name == "systems"), "the second refresh read the cache, not the changed file");
    cache.forget();
    let fresh = systems(&cache, &settings, shelves::shelves(&text, "{}"), &BTreeMap::new(), 1280, 800);
    assert!(!fresh[0].theme.system_view.elements.iter().any(|e| e.name == "systems"), "forget reads the folder again");
}

#[test]
fn a_carousel_step_glides_and_settles_on_the_frame_of_the_new_state() {
    let theme = Theme::new("glide");
    let mut p = painter(&theme);
    assert_eq!(p.view(), "system");
    assert!(!p.frame.moving && p.frame.next_change_ms.is_none(), "the system view at rest asks for nothing");
    assert_eq!(p.pressed(Button::Right, false), Some(Action::Cue("navigate")));
    p.released(Button::Right);
    assert_eq!(selected(&p), 1);
    p.tick(200.0);
    assert!(p.frame.moving, "mid-glide");
    p.tick(201.0);
    assert!(!p.frame.moving, "settled after ES-DE's 400 ms");
    let still = emusen_esde::scene::Stage::new(p.stage.as_ref().unwrap().current.data.clone(), "system", p.now);
    assert_eq!(p.frame.elements, still.frame().elements, "the settled glide is the static frame of the new state");
}

#[test]
fn the_router_s_repeats_are_the_scene_s_own_and_a_release_stops_the_hold() {
    let theme = Theme::new("hold");
    let mut p = painter(&theme);
    p.pressed(Button::Right, false);
    p.released(Button::Right);
    p.tick(500.0);
    p.pressed(Button::Accept, false);
    assert_eq!(p.view(), "gamelist", "the SNES's four games");
    p.tick(1000.0);
    assert_eq!(p.pressed(Button::Down, false), Some(Action::Cue("navigate")));
    assert_eq!(selected(&p), 1);
    assert_eq!(p.pressed(Button::Down, true), None, "the router's repeat is not a second step");
    assert_eq!(selected(&p), 1);
    for _ in 0..60 {
        p.tick(16.0);
    }
    let held_to = selected(&p);
    assert!(held_to >= 3, "the scene repeats a held direction on its own clock (at {held_to})");
    p.released(Button::Down);
    let at = selected(&p);
    for _ in 0..60 {
        p.tick(16.0);
    }
    assert_eq!(selected(&p), at, "nothing moves after the release");
    assert!(matches!(p.frame.primary.as_ref().unwrap().layout, Layout::TextList(_)));
}

#[test]
fn every_pointer_path_makes_the_pad_s_call() {
    let theme = Theme::new("pointer");
    let mut pad = painter(&theme);
    let mut pointer = painter(&theme);
    pad.pressed(Button::Right, false);
    pad.released(Button::Right);
    pointer.hover(1);
    assert_eq!(selected(&pad), selected(&pointer), "hover selects as the pad's step");
    assert_eq!(pad.pressed(Button::Accept, false), pointer.tap(1), "a tap is A on its row");
    assert_eq!(pad.view(), "gamelist");
    assert_eq!(pointer.view(), "gamelist");
    for p in [&mut pad, &mut pointer] {
        p.tick(1000.0);
    }
    pad.pressed(Button::Down, false);
    pad.released(Button::Down);
    pointer.wheel(-1);
    assert_eq!(selected(&pad), selected(&pointer), "a wheel notch down is one step on");
    let played = Some(Action::Played("Dune Relay.sfc".into()));
    assert_eq!(pad.pressed(Button::Accept, false), played);
    assert_eq!(pointer.tap(1), played);
    assert_eq!(pad.pressed(Button::Back, false), Some(Action::Cue("back")));
    assert_eq!(pad.view(), "system");
    assert_eq!(pad.pressed(Button::Back, false), None, "B at the system view does nothing (decided 2026-11-08)");
}

#[test]
fn an_item_gliding_under_a_still_pointer_does_not_take_the_selection() {
    let theme = Theme::new("glide");
    let mut p = painter(&theme);
    p.hover(1);
    assert_eq!(selected(&p), 1, "hover at rest selects");
    p.hover(2);
    assert_eq!(selected(&p), 1, "the next item, come under the pointer while the carousel glides, is not chosen");
    p.tick(1000.0);
    p.hover(2);
    assert_eq!(selected(&p), 2, "at rest again, the pointer's item is chosen");
}

#[test]
fn a_measurement_survives_a_reload() {
    let theme = Theme::new("measured");
    std::fs::write(
        theme.0.join("theme.xml"),
        "<theme><view name=\"system\"><carousel name=\"systems\"><pos>0 0.3</pos><size>1 0.4</size></carousel>\
         <text name=\"words\"><pos>0.5 0.1</pos><origin>0.5 0</origin><text>Hello</text></text></view></theme>",
    )
    .unwrap();
    let mut p = painter(&theme);
    let id = p.frame.elements.iter().find(|e| e.name == "words").expect("the text drawn").id;
    assert!(p.measured(id, 120.0, 30.0), "the first measurement changes the frame");
    let words = |p: &Painter| p.frame.elements.iter().find(|e| e.name == "words").map(|e| (e.x, e.w)).unwrap();
    assert_eq!(words(&p), (W / 2.0 - 60.0, 120.0));
    let settings = p.settings.clone();
    let found = systems(&ThemeCache::default(), &settings, shelves::shelves(&library(), "{}"), &BTreeMap::new(), W as i32, H as i32);
    p.install(found, W, H, DateTime::new(2026, 11, 8, 12, 0, 0));
    assert_eq!(words(&p), (W / 2.0 - 60.0, 120.0), "a reload keeps what the painter measured, which no delegate reports again");
}

#[test]
fn four_thousand_games_install_and_step_on_the_gui_thread_within_two_milliseconds() {
    let theme = Theme::new("big");
    let games: Vec<String> = (0..4000).map(|i| format!("Game {i:04}")).collect();
    let names: Vec<&str> = games.iter().map(String::as_str).collect();
    let text = format!("[{}]", shelf("snes", "Super Nintendo", &names));
    let settings = Settings { theme: theme.0.to_string_lossy().into_owned(), startup_view: "gamelist".into(), ..Settings::default() };
    let found = systems(&ThemeCache::default(), &settings, shelves::shelves(&text, "{}"), &BTreeMap::new(), W as i32, H as i32);
    let mut p = Painter::new(settings);
    let t = std::time::Instant::now();
    p.install(found, W, H, DateTime::new(2026, 11, 8, 12, 0, 0));
    let rows = p.rows(p.frame.primary.as_ref().expect("the list").rows);
    let install = t.elapsed();
    let t = std::time::Instant::now();
    p.pressed(Button::Down, false);
    p.tick(16.0);
    let step = t.elapsed();
    eprintln!("4,000 games: install and rows {install:?}, a step and a tick {step:?}");
    assert_eq!(rows.len(), 4000);
    assert!(step.as_secs_f64() < 0.002, "a step's GUI-thread work {step:?}");
    assert!(install.as_secs_f64() < 0.002 || cfg!(debug_assertions), "the install's GUI-thread work {install:?}");
}

#[test]
fn a_help_entry_s_place_is_the_kit_button_that_draws_there_under_the_swap() {
    use emusen_esde::scene::PadButton;
    use emusen_esde_qt::painter::help_buttons;
    use emusen_kit_core::input::PadControl;
    for swap in [false, true] {
        for (place, control) in [(PadButton::South, PadControl::South), (PadButton::East, PadControl::East), (PadButton::West, PadControl::West), (PadButton::North, PadControl::North), (PadButton::Start, PadControl::Start)] {
            let (button, pair) = help_buttons(place, false, swap);
            assert_eq!(Button::ALL[button as usize].control(swap), control, "{place:?} with the swap {swap}");
            assert_eq!(pair, -1);
        }
    }
    let (up, down) = help_buttons(PadButton::DpadUpDown, true, false);
    assert_eq!((Button::ALL[up as usize], Button::ALL[down as usize]), (Button::Up, Button::Down));
}
