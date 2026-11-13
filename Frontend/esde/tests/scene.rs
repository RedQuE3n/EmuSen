// SceneMotionTests, GridSceneTests and SceneSemanticsTests re-expressed where they assert the scene's state rather than a control's pixels
// (EmuSen_BigPicture.md §13.9, §14.6, §14.7, §16).
mod common;

use common::*;
use emusen_esde::loader::{Choices, SystemKind, ThemeSystem};
use emusen_esde::scene::motion::RepeatRule;
use emusen_esde::scene::{ms, words, Data, DateTime, Easing, Game, Motion, SceneView, Stage, System};

// SyntheticLibrary's invented systems and games.
fn library_systems() -> Vec<(ThemeSystem, &'static str)> {
    vec![
        (ThemeSystem::new("nes", "Nintendo Entertainment System", "nes"), ".nes"),
        (ThemeSystem::new("snes", "Super Nintendo", "snes"), ".sfc"),
        (ThemeSystem::new("n64", "Nintendo 64", "n64"), ".z64"),
        (ThemeSystem::new("gb", "Game Boy", "gb"), ".gb"),
        (ThemeSystem::new("gbc", "Game Boy Color", "gbc"), ".gbc"),
    ]
}

const TITLES: [&str; 12] = [
    "Aurora Drift",
    "Brass Lantern",
    "Cobalt Harbor",
    "Dune Relay",
    "Ember Circuit",
    "Fable of Tiles",
    "Granite Choir",
    "Hollow Comet",
    "Ivory Signal",
    "Juniper Vault",
    "Kestrel Run",
    "Lumen Garden",
];

fn games(extension: &str) -> Vec<Game> {
    let genres = ["Platform", "Puzzle", "Racing", "Role playing", "Shooter", "Sports"];
    TITLES
        .iter()
        .enumerate()
        .map(|(i, t)| Game {
            developer: Some(format!("Studio {}", (b'A' + (i % 5) as u8) as char)),
            publisher: Some(format!("Publisher {}", (b'P' + (i % 3) as u8) as char)),
            genre: Some(genres[i % genres.len()].into()),
            players: Some((1 + i % 4).to_string()),
            release_date: Some(DateTime::new(1990 + i as i32, 1 + (i % 12) as u32, 1 + i as u32, 0, 0, 0)),
            last_played: if i % 3 == 0 { None } else { Some(DateTime::new(2026, 9, 24 - i as u32, 10, 0, 0)) },
            play_count: i as u32,
            favorite: i % 2 == 0,
            completed: i % 3 == 1,
            ..Game::new(t, &format!("{t} (Synthetic){extension}"))
        })
        .collect()
}

const W: f64 = 320.0;
// SceneSemanticsTests' sixth system, the All Games collection, added only when it is selected.
const ALL: usize = 5;
const H: f64 = 200.0;

fn round() -> Motion {
    Motion {
        carousel_step: ms(200.0),
        carousel_easing: Easing::QuadraticOut,
        carousel_repeat: RepeatRule::new(ms(500.0), ms(200.0)),
        carousel_fast_repeat: RepeatRule::fast(ms(500.0), ms(200.0), ms(1000.0), ms(100.0), 1),
        list_repeat: RepeatRule::fast(ms(400.0), ms(100.0), ms(1000.0), ms(50.0), 1),
        metadata_fade_out: ms(100.0),
        metadata_fade_in: ms(200.0),
        scroll_fade_in: ms(300.0),
        scroll_fade_in_from: 0.5,
        marquee_speed_per_em: 2.0,
        marquee_gap_seconds: 1.0,
        horizontal_container_speed_per_em: 2.0,
        horizontal_container_gap_seconds: 1.0,
        vertical_container_speed_per_em: 1.5,
        vertical_container_fade_in: ms(400.0),
        view_slide: ms(400.0),
        view_slide_easing: Easing::CubicInOut,
        ..Motion::esde()
    }
}

#[allow(clippy::too_many_arguments)]
fn data_sized(system: &str, gamelist: &str, motion: Motion, capabilities: &str, system_index: usize, game: usize, w: f64, h: f64) -> Data {
    let t = Synth::new();
    t.asset("art.png").capabilities(capabilities).theme(&format!("<view name=\"system\">{system}</view><view name=\"gamelist\">{gamelist}</view>"));
    let choices = Choices { screen_width: w as i32, screen_height: h as i32, ..Choices::default() };
    let mut systems: Vec<System> = library_systems()
        .into_iter()
        .map(|(s, ext)| {
            let mut list = games(ext);
            list[3].name = "A name far too long to fit in the narrow list it is shown in".into();
            System::new(s.clone(), t.load_with(&choices, &s, None), list)
        })
        .collect();
    if system_index == ALL {
        let all = ThemeSystem::of_kind("all", "All Games", "auto-allgames", SystemKind::AutoCollection);
        systems.push(System::new(all.clone(), t.load_with(&choices, &all, None), games(".nes")));
    }
    let mut data = Data::new(systems, w, h);
    data.system_index = system_index;
    data.game_index = game;
    data.motion = motion;
    data
}

fn data(system: &str, gamelist: &str, motion: Motion, capabilities: &str, system_index: usize, game: usize) -> Data {
    data_sized(system, gamelist, motion, capabilities, system_index, game, W, H)
}

fn carousel(extra: &str) -> String {
    format!("<carousel name=\"c\"><pos>0 0</pos><size>1 1</size><maxItemCount>3</maxItemCount><itemScale>1.5</itemScale><unfocusedItemOpacity>0.3</unfocusedItemOpacity>{extra}</carousel>")
}

fn list(extra: &str) -> String {
    format!("<textlist name=\"l\"><pos>0 0</pos><size>0.4 1</size><fontSize>0.08</fontSize><primaryColor>FFFFFF</primaryColor><selectedColor>FFFF00</selectedColor>{extra}</textlist>")
}

fn opacity_of(view: &SceneView, name: &str) -> f64 {
    view.frame().into_iter().find(|e| e.name == name).unwrap().opacity
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

// The values §14.7 records from ES-DE 3.4.1's recordings, so a change to one is a change to the record.
#[test]
fn the_es_de_motion_is_what_was_measured() {
    let m = Motion::esde();
    assert_eq!(m.carousel_step, ms(400.0));
    assert_eq!(m.carousel_easing, Easing::QuadraticOut);
    assert_eq!(m.carousel_repeat, RepeatRule::new(ms(500.0), ms(200.0)));
    assert_eq!(m.carousel_fast_repeat, RepeatRule::fast(ms(500.0), ms(180.0), ms(1580.0), ms(80.0), 1));
    assert_eq!(m.list_repeat, RepeatRule::fast(ms(500.0), ms(114.0), ms(1703.0), ms(15.9), 4));
    assert_eq!((m.metadata_fade_out, m.metadata_fade_in), (ms(149.0), ms(150.0)));
    assert_eq!((m.scroll_fade_in, m.scroll_fade_in_from), (ms(326.0), 0.5));
    assert!(close(m.marquee_speed_per_em, 131.5 / 30.0));
    assert_eq!(m.marquee_gap_seconds, 1.0);
    assert!(close(m.vertical_container_speed_per_em, 37.03 / 30.0));
    assert_eq!(m.vertical_container_fade_in, ms(298.0));
    assert_eq!(m.view_slide, ms(402.0));
    assert_eq!(m.view_slide_easing, Easing::CubicOut);
}

#[test]
fn a_carousel_step_eases_to_the_next_item_and_settles() {
    let mut view = SceneView::new(data(&carousel(""), "", round(), "", 1, 2), "system", ms(1000.0));
    view.step(1, ms(1000.0));
    assert_eq!(view.data.system_index, 2);
    assert!(close(view.position(), 1.0));
    view.advance(ms(1100.0));
    assert!(close(view.position(), 1.0 + Easing::QuadraticOut.ease(0.5)));
    let c = view.frame().into_iter().find(|e| e.kind == "carousel").unwrap();
    assert!(close(c.offset, 1.75));
    assert!(view.is_moving());
    view.advance(ms(1200.0));
    assert!(!view.is_moving());
}

#[test]
fn a_step_past_the_last_item_keeps_moving_the_same_way_and_item_transitions_instant_jumps() {
    let mut view = SceneView::new(data(&carousel(""), "", round(), "", 4, 2), "system", 0);
    view.step(1, 0);
    assert_eq!(view.data.system_index, 0);
    view.advance(ms(50.0));
    assert!(view.position() > 4.0 && view.position() < 5.0);
    let mut instant = SceneView::new(data(&carousel("<itemTransitions>instant</itemTransitions>"), "", round(), "", 1, 2), "system", 0);
    instant.step(-1, 0);
    instant.advance(ms(1.0));
    assert_eq!(instant.position(), 0.0);
    assert!(!instant.is_moving());
}

#[test]
fn a_held_direction_repeats_after_its_delay_and_faster_after_a_while() {
    let mut list_view = SceneView::new(data("", &list(""), round(), "", 1, 0), "gamelist", 0);
    list_view.press(1, 0, false);
    assert_eq!(list_view.data.game_index, 1);
    list_view.advance(ms(399.0));
    assert_eq!(list_view.data.game_index, 1);
    list_view.advance(ms(400.0));
    assert_eq!(list_view.data.game_index, 2);
    list_view.advance(ms(900.0));
    assert_eq!(list_view.data.game_index, 7);
    list_view.advance(ms(1100.0));
    assert_eq!(list_view.data.game_index, 10);
    list_view.release(ms(1100.0));
    list_view.advance(ms(5000.0));
    assert_eq!(list_view.data.game_index, 10);
}

#[test]
fn a_list_wraps_on_a_tap_stops_at_its_end_when_held_and_jumps_at_the_fast_tier() {
    let jump = Motion { list_repeat: RepeatRule::fast(ms(400.0), ms(100.0), ms(1050.0), ms(50.0), 4), ..round() };
    let elements =
        list("") + "<text name=\"dev\"><pos>0.5 0</pos><size>0.5 0.2</size><metadata>developer</metadata><color>FFFFFF</color></text>";
    let mut tap = SceneView::new(data("", &elements, jump, "", 1, 11), "gamelist", 0);
    tap.step(1, 0);
    assert_eq!(tap.data.game_index, 0);
    let mut held = SceneView::new(data("", &elements, jump, "", 1, 0), "gamelist", 0);
    held.press(1, 0, false);
    held.advance(ms(1000.0));
    assert_eq!(held.data.game_index, 8);
    held.advance(ms(1050.0));
    assert_eq!(held.data.game_index, 11);
    assert!(close(opacity_of(&held, "dev"), 0.0));
    held.advance(ms(1200.0));
    assert_eq!(held.data.game_index, 11);
    assert!(opacity_of(&held, "dev") > 0.0);
}

#[test]
fn fast_scrolling_takes_the_carousels_faster_tier_only_when_the_theme_asks() {
    let mut slow = SceneView::new(data(&carousel("<fastScrolling>false</fastScrolling>"), "", round(), "", 0, 2), "system", 0);
    let mut fast = SceneView::new(data(&carousel("<fastScrolling>true</fastScrolling>"), "", round(), "", 0, 2), "system", 0);
    for v in [&mut slow, &mut fast] {
        v.press(1, 0, false);
        v.advance(ms(1500.0));
    }
    assert_eq!(slow.target(), 7.0);
    assert_eq!(fast.target(), 10.0);
}

#[test]
fn the_selected_name_scrolls_after_its_delay_and_starts_again_on_each_selection() {
    let mut view =
        SceneView::new(data("", &list("<textHorizontalScrollDelay>1</textHorizontalScrollDelay>"), round(), "", 1, 3), "gamelist", 0);
    let state = |v: &SceneView| v.frame().into_iter().find(|e| e.kind == "textlist").unwrap();
    let width = 2000.0;
    view.advance(ms(900.0));
    let rule = state(&view).scroll.unwrap();
    assert_eq!(rule.loop_offset(state(&view).since_selected, width), 0.0);
    assert!((rule.gap - 1.0 * 1.5 * 2.0 * 0.08 * H).abs() < 1e-4);
    view.advance(ms(1500.0));
    assert_eq!(state(&view).since_selected, ms(1500.0));
    assert!(close(rule.loop_offset(state(&view).since_selected, width), 0.5 * 2.0 * 0.08 * H));
    view.step(1, ms(1600.0));
    view.step(-1, ms(1700.0));
    view.advance(ms(2000.0));
    assert_eq!(state(&view).since_selected, ms(300.0));
    assert_eq!(rule.loop_offset(state(&view).since_selected, width), 0.0);
    let mut off =
        SceneView::new(data("", &list("<textHorizontalScrolling>false</textHorizontalScrolling>"), round(), "", 1, 3), "gamelist", 0);
    off.advance(ms(10000.0));
    assert!(state(&off).scroll.is_none());
}

#[test]
fn a_description_container_scrolls_by_the_themes_delays_and_the_measured_rate() {
    let text = "<text name=\"d\"><pos>0.5 0</pos><size>0.5 0.3</size><metadata>description</metadata><fontSize>0.06</fontSize><lineSpacing>1.5</lineSpacing>\
        <containerStartDelay>2</containerStartDelay><containerResetDelay>3</containerResetDelay><containerScrollSpeed>2</containerScrollSpeed><color>FFFFFF</color></text>";
    let mut games_with_text = data("", &(list("") + text), round(), "", 1, 1);
    for system in &mut games_with_text.systems {
        for g in &mut system.games {
            g.description = Some(format!("{} is a game invented for EmuSen's tests.", g.name));
        }
    }
    let mut view = SceneView::new(games_with_text, "gamelist", 0);
    let d = |v: &SceneView| v.frame().into_iter().find(|e| e.name == "d").unwrap();
    let rule = d(&view).scroll.unwrap();
    assert_eq!(rule.delay, ms(2000.0));
    assert_eq!(rule.end_pause, ms(3000.0));
    assert_eq!(rule.fade_in, ms(400.0));
    assert!(close(rule.speed, 1.5 * 0.06 * H * 2.0));
    assert!(rule.whole_pixels);
    view.advance(ms(2500.0));
    assert_eq!(d(&view).since_selected, ms(2500.0));
}

#[test]
fn a_scroll_fade_in_image_fades_in_on_each_new_game_but_not_when_the_view_opens() {
    let elements = list("")
        + "<image name=\"art\"><pos>0.5 0</pos><size>0.4 0.5</size><imageType>cover</imageType><scrollFadeIn>true</scrollFadeIn><opacity>0.8</opacity></image>\
           <image name=\"plain\"><pos>0.5 0.5</pos><size>0.4 0.5</size><imageType>cover</imageType></image>";
    let mut view = SceneView::new(data("", &elements, round(), "", 1, 2), "gamelist", 0);
    assert!(close(opacity_of(&view, "art"), 0.8));
    view.step(1, ms(1000.0));
    assert!(close(opacity_of(&view, "art"), 0.4));
    view.advance(ms(1150.0));
    assert!(close(opacity_of(&view, "art"), 0.6));
    assert!(close(opacity_of(&view, "plain"), 1.0));
    view.advance(ms(1300.0));
    assert!(close(opacity_of(&view, "art"), 0.8));
}

#[test]
fn metadata_fades_out_while_the_list_scrolls_fast_and_back_in_when_it_stops() {
    let elements = list("")
        + "<text name=\"dev\"><pos>0.5 0</pos><size>0.5 0.2</size><metadata>developer</metadata><color>FFFFFF</color></text>\
           <text name=\"sys\"><pos>0.5 0.5</pos><size>0.5 0.2</size><metadata>systemName</metadata><color>FFFFFF</color></text>";
    let mut view = SceneView::new(data("", &elements, round(), "", 1, 0), "gamelist", 0);
    view.press(1, 0, false);
    view.advance(ms(399.0));
    assert!(close(opacity_of(&view, "dev"), 1.0));
    view.advance(ms(450.0));
    assert!(close(opacity_of(&view, "dev"), 0.5));
    assert!(close(opacity_of(&view, "sys"), 1.0));
    view.advance(ms(1050.0));
    assert!(close(opacity_of(&view, "dev"), 0.0));
    view.release(ms(1200.0));
    assert!(close(opacity_of(&view, "dev"), 0.0));
    view.advance(ms(1300.0));
    assert!(close(opacity_of(&view, "dev"), 0.5));
    view.advance(ms(1400.0));
    assert!(close(opacity_of(&view, "dev"), 1.0));
}

#[test]
fn a_slide_moves_both_views_across_and_settles_on_the_gamelist_alone() {
    let capabilities =
        "<transitions name=\"slide\"><systemToGamelist>slide</systemToGamelist><gamelistToSystem>slide</gamelistToSystem></transitions>";
    let help = "<helpsystem name=\"h\"><pos>0 0.9</pos><textColor>FFFFFF</textColor></helpsystem>";
    let mut stage = Stage::new(data(&(carousel("") + help), &(list("") + help), round(), capabilities, 1, 2), "system", 0);
    stage.switch(ms(1000.0));
    assert_eq!(stage.layers().len(), 2);
    stage.advance(ms(1200.0));
    let layers = stage.layers();
    let (leaving, coming) = (&layers[0], &layers[1]);
    assert!(close(leaving.y, -H * Easing::CubicInOut.ease(0.5)));
    assert!(close(coming.y, H * (1.0 - Easing::CubicInOut.ease(0.5))));
    let bar = coming.elements.iter().find(|e| e.kind == "helpsystem").unwrap();
    let still = stage.current.frame().into_iter().find(|e| e.kind == "helpsystem").unwrap();
    assert!(close(bar.rect.y - still.rect.y, -coming.y));
    assert!(leaving.elements.iter().all(|e| e.kind != "helpsystem"));
    stage.advance(ms(1400.0));
    assert_eq!(stage.layers().len(), 1);
    assert_eq!(stage.current.view, "gamelist");

    let mut instant = Stage::new(
        data(
            &carousel(""),
            &list(""),
            round(),
            "<transitions name=\"instant\"><systemToGamelist>instant</systemToGamelist></transitions>",
            1,
            2,
        ),
        "system",
        0,
    );
    instant.switch(0);
    assert_eq!(instant.layers().len(), 1);
    assert_eq!(instant.current.view, "gamelist");
}

// ES-DE 3.4.1's `SystemView::renderElements` and `GamelistView::render`: during a slide between the two views an element
// that is stationary betweenViews or always rests where it is, and one with renderDuringTransitions false is not drawn.
#[test]
fn a_stationary_element_rests_and_a_transit_hidden_one_waits_while_the_views_pan() {
    let capabilities =
        "<transitions name=\"slide\"><systemToGamelist>slide</systemToGamelist><gamelistToSystem>slide</gamelistToSystem></transitions>";
    let image = |name: &str, extra: &str| format!("<image name=\"{name}\"><pos>0.5 0.5</pos><size>0.2 0.2</size><path>./art.png</path>{extra}</image>");
    let system = carousel("")
        + &image("moves", "")
        + &image("between", "<stationary>betweenViews</stationary>")
        + &image("always", "<stationary>always</stationary>")
        + &image("within", "<stationary>withinView</stationary>")
        + &image("waits", "<renderDuringTransitions>false</renderDuringTransitions>");
    let gamelist = list("") + &image("comes", "<stationary>always</stationary>") + &image("later", "<renderDuringTransitions>false</renderDuringTransitions>");
    let mut stage = Stage::new(data(&system, &gamelist, round(), capabilities, 1, 2), "system", 0);
    let y = |stage: &Stage, name: &str| stage.frame().elements.iter().find(|e| e.name == name).map(|e| e.y);
    let rest = y(&stage, "moves").unwrap();
    assert_eq!(y(&stage, "waits"), Some(rest));
    stage.switch(ms(1000.0));
    stage.advance(ms(1200.0));
    let moved = y(&stage, "moves").unwrap();
    assert!(moved < rest - 1.0, "a plain element pans with its view: {moved} against {rest}");
    assert!(close(y(&stage, "between").unwrap(), rest));
    assert!(close(y(&stage, "always").unwrap(), rest));
    assert!(close(y(&stage, "within").unwrap(), moved), "withinView is for a move within one view");
    assert!(close(y(&stage, "comes").unwrap(), rest));
    assert_eq!(y(&stage, "waits"), None);
    assert_eq!(y(&stage, "later"), None);
    // The layers say the same.
    let layers = stage.layers();
    let at = |name: &str| layers.iter().find_map(|l| l.elements.iter().find(|e| e.name == name).map(|e| e.rect.y + l.y));
    assert!(close(at("between").unwrap(), rest));
    assert!(close(at("comes").unwrap(), rest));
    assert!(at("waits").is_none() && at("later").is_none());
    stage.advance(ms(1500.0));
    assert!(close(y(&stage, "later").unwrap(), rest));
}

// GridSceneTests: four columns of 0.2 x 0.3, spacing 0.02 x 0.03, scale 1.2, two whole rows, and a metadata text above.
fn grid(extra: &str) -> String {
    format!(
        "<grid name=\"g\"><pos>0 0.1</pos><size>1 0.8</size><itemSize>0.2 0.3</itemSize><itemSpacing>0.02 0.03</itemSpacing><itemScale>1.2</itemScale>\
         <imageType>cover</imageType><imageFit>fill</imageFit><unfocusedItemOpacity>0.4</unfocusedItemOpacity>{extra}</grid>\
         <text name=\"n\"><pos>0 0</pos><size>1 0.08</size><metadata>name</metadata><fontSize>0.05</fontSize><color>FFFFFF</color></text>"
    )
}

fn grid_view(elements: &str, game: usize) -> SceneView {
    SceneView::new(data_sized("", elements, Motion::esde(), "", 1, game, 640.0, 400.0), "gamelist", 0)
}

fn grid_state(view: &SceneView) -> emusen_esde::scene::GridFocus {
    view.frame().into_iter().find(|e| e.kind == "grid").unwrap().grid.unwrap()
}

#[test]
fn the_grid_layout_is_es_de_s_measured_one() {
    let view = grid_view(&grid(""), 0);
    let g = view.grid().unwrap();
    assert_eq!(g.columns(), 4);
    assert_eq!(g.whole_rows(), 2);
    let (x, y, _, _) = g.cell(0, 0.0);
    assert!((x - 12.8).abs() < 1e-3 && (y - 12.0).abs() < 1e-3);
    assert_eq!(g.rows(), 3);
}

#[test]
fn a_grid_step_eases_the_focus_and_settles() {
    let mut view = grid_view(&grid(""), 0);
    view.press(1, 0, false);
    view.release(ms(10.0));
    view.advance(ms(125.0));
    let s = grid_state(&view);
    assert_eq!((s.from, view.index()), (Some(0), 1));
    assert!((s.progress - Easing::QuadraticOut.ease(0.5)).abs() < 1e-3);
    view.advance(ms(260.0));
    assert!(!view.is_moving());
}

#[test]
fn a_row_past_the_last_shown_slides_into_view_and_settles() {
    let mut view = grid_view(&grid(""), 0);
    view.press(1, 0, true);
    view.release(ms(10.0));
    view.advance(ms(300.0));
    assert_eq!((view.index(), grid_state(&view).scroll_row), (4, 0.0));
    view.press(1, ms(300.0), true);
    view.release(ms(310.0));
    view.advance(ms(425.0));
    assert_eq!(view.index(), 8);
    assert!((grid_state(&view).scroll_row - Easing::QuadraticOut.ease(0.5)).abs() < 1e-3);
    view.advance(ms(600.0));
    assert_eq!(grid_state(&view).scroll_row, 1.0);
}

#[test]
fn the_grid_ends_are_es_de_s() {
    let mut view = grid_view(&grid(""), 0);
    view.step(-1, 0);
    assert_eq!(view.index(), 11);
    view.step(1, ms(10.0));
    assert_eq!(view.index(), 0);
    view.step(3, ms(20.0));
    view.step(1, ms(30.0));
    assert_eq!(view.index(), 4);
    view.press(-1, ms(40.0), true);
    view.release(ms(41.0));
    assert_eq!(view.index(), 0);
    view.press(-1, ms(50.0), true);
    view.release(ms(51.0));
    assert_eq!(view.index(), 0);
    let mut shorter_data = data_sized("", &grid(""), Motion::esde(), "", 1, 0, 640.0, 400.0);
    for s in &mut shorter_data.systems {
        s.games.truncate(10);
    }
    let mut shorter = SceneView::new(shorter_data, "gamelist", 0);
    shorter.step(7, 0);
    shorter.press(1, ms(10.0), true);
    shorter.release(ms(11.0));
    assert_eq!(shorter.index(), 9);
    shorter.press(1, ms(20.0), true);
    shorter.release(ms(21.0));
    assert_eq!(shorter.index(), 9);
    let mut held = grid_view(&grid(""), 9);
    held.press(1, 0, false);
    held.advance(ms(2500.0));
    assert_eq!(held.index(), 11);
}

#[test]
fn a_grid_step_while_moving_starts_from_the_items_current_focus() {
    let mut view = grid_view(&grid(""), 0);
    view.step(1, 0);
    view.step(1, ms(100.0));
    view.advance(ms(100.0));
    let s = grid_state(&view);
    assert_eq!((s.from, view.index()), (Some(1), 2));
    assert!((s.from_level - Easing::QuadraticOut.ease(0.4)).abs() < 1e-3);
    assert!(s.to_level.abs() < 1e-3);
    view.step(-1, ms(150.0));
    view.advance(ms(150.0));
    let left = Easing::QuadraticOut.ease(0.2);
    let s = grid_state(&view);
    assert_eq!((s.from, view.index()), (Some(2), 1));
    assert!((s.to_level - Easing::QuadraticOut.ease(0.4) * (1.0 - left)).abs() < 1e-3);
}

#[test]
fn grid_item_sizes_are_in_es_de_s_units() {
    let square = grid_view(&grid("").replace("<itemSize>0.2 0.3</itemSize>", "<itemSize>0.2 -1</itemSize>"), 0);
    let item = square.grid().unwrap().item;
    assert!((item.0 - 128.0).abs() < 1e-3 && (item.1 - 128.0).abs() < 1e-3);
    let tall = grid_view(&grid("").replace("<itemSize>0.2 0.3</itemSize>", "<itemSize>-1 0.3</itemSize>"), 0);
    let item = tall.grid().unwrap().item;
    assert!((item.0 - 120.0).abs() < 1e-3 && (item.1 - 120.0).abs() < 1e-3);
}

#[test]
fn a_held_grid_direction_repeats_at_500_then_200_ms_with_no_faster_tier() {
    for vertical in [false, true] {
        let mut many = data_sized("", &grid(""), Motion::esde(), "", 1, 0, 640.0, 400.0);
        for s in &mut many.systems {
            s.games = (0..200).map(|i| Game::new(&format!("G{i}"), &format!("/g/{i}.sfc"))).collect();
        }
        let mut view = SceneView::new(many, "gamelist", 0);
        view.press(1, 0, vertical);
        let mut t = 0;
        while t <= 4000 {
            view.advance(ms(t as f64));
            t += 10;
        }
        let steps = view.take_steps();
        let first: Vec<f64> = steps.iter().take(4).map(|s| s.at as f64 / 1000.0).collect();
        assert_eq!(first, [0.0, 500.0, 700.0, 900.0]);
        assert_eq!(steps.len(), 1 + 1 + (4000 - 500) / 200);
        assert_eq!(steps.len(), if vertical { view.index() / 4 } else { view.index() });
    }
}

#[test]
fn a_held_grid_fades_the_metadata_out_from_the_first_repeat() {
    let mut view = grid_view(&grid(""), 0);
    view.press(1, 0, false);
    view.advance(ms(400.0));
    assert!((opacity_of(&view, "n") - 1.0).abs() < 1e-3);
    view.advance(ms(500.0 + 149.0));
    assert!(opacity_of(&view, "n").abs() < 1e-2);
    view.release(ms(700.0));
    view.advance(ms(700.0 + 150.0));
    assert!((opacity_of(&view, "n") - 1.0).abs() < 1e-2);
}

#[test]
fn instant_grid_transitions_do_not_animate() {
    let mut view = grid_view(&grid("<itemTransitions>instant</itemTransitions><rowTransitions>instant</rowTransitions>"), 4);
    view.press(1, 0, true);
    view.release(ms(1.0));
    view.advance(ms(1.0));
    let s = grid_state(&view);
    assert_eq!((s.progress, s.scroll_row), (1.0, 1.0));
}

// SceneSemanticsTests: a datetime by strftime, and the system name suffix for collections only.
#[test]
fn a_datetime_formats_by_strftime() {
    for (format, expected) in [
        ("%Y-%m-%d", "1994-05-05"),
        ("%d/%m/%y", "05/05/94"),
        ("%b %e, %Y", "May 5, 1994"),
        ("Released %Y", "Released 1994"),
        ("%%%Y", "%1994"),
    ] {
        let element = format!("<datetime name=\"x\"><metadata>releasedate</metadata><format>{format}</format></datetime>");
        let view = SceneView::new(data("", &element, Motion::esde(), "", 1, 4), "gamelist", 0);
        assert_eq!(view.frame().into_iter().find(|e| e.name == "x").unwrap().text.as_deref(), Some(expected), "{format}");
    }
}

#[test]
fn the_system_name_suffix_is_for_collections_only() {
    let element = "<text name=\"x\"><metadata>name</metadata><systemNameSuffix>true</systemNameSuffix></text>";
    let said = |system: usize| {
        let view = SceneView::new(data("", element, Motion::esde(), "", system, 4), "gamelist", 0);
        view.frame().into_iter().find(|e| e.name == "x").unwrap().text
    };
    assert_eq!(said(1).as_deref(), Some("Ember Circuit"));
    assert_eq!(said(5).as_deref(), Some("Ember Circuit [ALL]"));
    assert_eq!(words::cased("ember circuit", Some("capitalize")), "Ember Circuit");
}

// ES-DE 3.4.1 SystemView::populate and GamelistBase::populateList: collections take their own letter case, and a game's
// name is cased before its system suffix, which is cased on its own.
#[test]
fn collections_take_their_own_letter_case_and_the_suffix_is_cased_apart() {
    let t = Synth::new();
    let element = "<carousel name=\"c\"><pos>0 0</pos><size>1 1</size><letterCase>lowercase</letterCase>\
                   <letterCaseAutoCollections>uppercase</letterCaseAutoCollections><letterCaseCustomCollections>capitalize</letterCaseCustomCollections>\
                   <letterCaseSystemNameSuffix>capitalize</letterCaseSystemNameSuffix></carousel>";
    t.capabilities("").theme(&format!("<view name=\"system\">{element}</view><view name=\"gamelist\">{element}</view>"));
    let choices = Choices { screen_width: W as i32, screen_height: H as i32, ..Choices::default() };
    let kinds = [
        ThemeSystem::new("snes", "Super Nintendo", "snes"),
        ThemeSystem::of_kind("all", "All Games", "auto-allgames", SystemKind::AutoCollection),
        ThemeSystem::of_kind("mine", "my FAVOURITE ones", "custom-collections", SystemKind::CustomCollection),
    ];
    let systems: Vec<System> = kinds
        .iter()
        .map(|s| {
            let mut list = games(".sfc");
            list[0].source = Some(ThemeSystem::new("snes", "Super Nintendo", "snes"));
            list[1].folder = true;
            System::new(s.clone(), t.load_with(&choices, s, None), list)
        })
        .collect();
    let rows = |view: &str, system: usize| {
        let mut data = Data::new(systems.clone(), W, H);
        data.system_index = system;
        let s = Stage::new(data, view, 0);
        let p = s.frame().primary.unwrap();
        s.rows(p.rows).unwrap().into_iter().map(|r| r.text).collect::<Vec<_>>()
    };
    assert_eq!(rows("system", 0), ["super nintendo", "ALL GAMES", "My Favourite Ones"]);
    assert_eq!(rows("gamelist", 0)[0], "aurora drift", "a regular system's games take no suffix");
    assert_eq!(rows("gamelist", 1)[0], "aurora drift [Snes]", "the suffix keeps its own case");
    assert_eq!(rows("gamelist", 1)[1], "brass lantern", "a folder takes no suffix");
    assert_eq!(rows("gamelist", 2)[1], "Brass Lantern", "a folder in a custom collection takes its case");
}

#[test]
fn capitalize_lowers_the_rest_and_raises_each_word_as_unicode_bounds_it() {
    assert_eq!(words::cased("SUPER mario BROS. 3", Some("capitalize")), "Super Mario Bros. 3");
    assert_eq!(words::cased("don't stop-me now", Some("capitalize")), "Don't Stop-Me Now");
    assert_eq!(words::cased("3d world (usa)", Some("capitalize")), "3d World (Usa)");
    assert_eq!(words::system_suffix("snes", Some("lowercase")), " [snes]");
    assert_eq!(words::system_suffix("snes", None), " [SNES]");
}

// ES-DE 3.4.1's `GIFAnimComponent`: the first frame stands one pacing, then the sequence one frame a pacing; the
// alternating forms turn without showing an end twice; after the iterations nothing is drawn.
#[test]
fn a_gif_steps_by_its_first_delay_over_the_speed_in_its_direction_for_its_iterations() {
    use emusen_esde::scene::motion::gif_frame;
    let at = |t: f64, speed: f64, dir: &str, it: u32| gif_frame(ms(t), 4, 100, speed, dir, it).map(|(f, _)| f);
    let run = |speed: f64, dir: &str, it: u32| (0..16).map(|i| at(i as f64 * 100.0 / speed + 1.0, speed, dir, it)).collect::<Vec<_>>();
    let s = |v: &[u32]| v.iter().map(|f| Some(*f)).collect::<Vec<_>>();
    assert_eq!(run(1.0, "normal", 0), s(&[0, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2]));
    assert_eq!(run(1.0, "reverse", 0), s(&[0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1, 0, 3, 2, 1]));
    assert_eq!(run(1.0, "alternate", 0), s(&[0, 0, 1, 2, 3, 2, 1, 0, 1, 2, 3, 2, 1, 0, 1, 2]));
    assert_eq!(run(1.0, "alternateReverse", 0), s(&[0, 3, 2, 1, 0, 1, 2, 3, 2, 1, 0, 1, 2, 3, 2, 1]));
    assert_eq!(run(2.0, "normal", 0), run(1.0, "normal", 0), "speed divides the pacing");
    assert_eq!(&run(1.0, "normal", 2)[..9], &s(&[0, 0, 1, 2, 3, 0, 1, 2, 3])[..]);
    assert_eq!(run(1.0, "normal", 2)[9], None, "two plays, then nothing");
    assert_eq!(gif_frame(ms(150.0), 4, 100, 1.0, "normal", 0), Some((0, Some(ms(50.0)))));
    assert_eq!(gif_frame(ms(150.0), 4, 100, 3.0, "normal", 0).map(|f| f.0), Some(3), "33 ms a frame, whole milliseconds: four pacings, the fourth frame");
    assert_eq!(gif_frame(ms(5000.0), 1, 100, 1.0, "normal", 1), Some((0, None)), "one frame stands");
}
