// A theme's fade transitions and fadeAbovePrimary as ES-DE 3.4.1 plays them, read from Stage::frame (EmuSen_Qt.md §4.8.8).
mod common;

use common::*;
use emusen_esde::loader::{Choices, ThemeSystem};
use emusen_esde::scene::*;

const W: f64 = 320.0;
const H: f64 = 200.0;

const CAROUSEL: &str = "<carousel name=\"c\"><pos>0 0.4</pos><size>1 0.2</size><maxItemCount>3</maxItemCount>";
// Below the carousel's zIndex 50 and above it; each says which system it shows.
const BELOW: &str = "<text name=\"below\"><pos>0 0</pos><size>1 0.2</size><text>${system.name}</text><zIndex>40</zIndex></text>";
const ABOVE: &str = "<text name=\"above\"><pos>0 0.8</pos><size>1 0.2</size><text>${system.name}</text><zIndex>60</zIndex></text>";

fn profile(kinds: &str) -> String {
    format!("<transitions name=\"p\">{kinds}</transitions>")
}

fn data(t: &Synth, capabilities: &str, system: &str, gamelist: &str) -> Data {
    t.capabilities(capabilities).theme(&format!("<view name=\"system\">{system}</view><view name=\"gamelist\">{gamelist}</view>"));
    let choices = Choices { screen_width: W as i32, screen_height: H as i32, ..Choices::default() };
    let systems = ["nes", "snes", "n64"]
        .iter()
        .map(|s| {
            let sys = ThemeSystem::new(s, &format!("{s} full"), s);
            System::new(sys.clone(), t.load_with(&choices, &sys, None), vec![Game::new("Aurora Drift", "a.nes"), Game::new("Brass Lantern", "b.nes")])
        })
        .collect();
    let mut data = Data::new(systems, W, H);
    data.system_index = 1;
    data.show_clock = false;
    data
}

fn systems_stage(t: &Synth, kind: &str, primary_extra: &str) -> Stage {
    let caps = profile(&format!("<systemToSystem>{kind}</systemToSystem>"));
    Stage::new(data(t, &caps, &format!("{CAROUSEL}{primary_extra}</carousel>{BELOW}{ABOVE}"), ""), "system", 0)
}

fn at(s: &mut Stage, t: f64) -> Frame {
    s.advance(ms(t));
    s.frame()
}

fn find<'a>(f: &'a Frame, name: &str) -> Option<&'a Placed> {
    f.elements.iter().find(|p| p.name == name)
}

fn words(f: &Frame, name: &str) -> String {
    match &find(f, name).unwrap_or_else(|| panic!("{name} not drawn")).paint {
        Paint::Text(t) => t.text.clone(),
        _ => panic!("{name} is not a text"),
    }
}

fn veil(f: &Frame) -> Option<f64> {
    find(f, "veil").map(|p| p.opacity)
}

// The theme's own elements in drawing order, without the default help and status ES-DE adds.
fn order(f: &Frame) -> Vec<String> {
    f.elements.iter().map(|p| p.name.clone()).filter(|n| n != "(default)").collect()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-3
}

// A tap: pressed at 0 and let go at 10 ms, before the fade's darkest.
fn tap(s: &mut Stage) {
    s.current.press(1, 0, false);
    s.advance(ms(10.0));
    s.current.release(ms(10.0));
}

#[test]
fn a_fade_between_systems_darkens_below_the_primary_and_changes_system_half_way() {
    let t = Synth::new();
    let mut s = systems_stage(&t, "fade", "");
    tap(&mut s);
    // ES-DE 3.4.1 SystemView.cpp: a run of 400 ms (line 427) x 1.3 (line 476) = 520 ms; the black is t / 0.2 while t < 0.3
    // (451-453), whole while t < 0.7 (454), then 1 - (t - 0.6) / 0.3 (457), so 2/3 at 0.7 and none from 0.9; the
    // camera moves past t = 0.5 (459). 26 ms is t = 0.05, a black of 0.25.
    let f = at(&mut s, 26.0);
    assert!(close(veil(&f).unwrap(), 0.25), "{:?}", veil(&f));
    assert_eq!(words(&f, "below"), "snes", "the system left shows until half way");
    assert_eq!(order(&f), ["below", "veil", "c", "above"], "the black lies between the elements below and the primary");
    assert!(close(find(&f, "above").unwrap().opacity, 1.0), "above the primary nothing fades without fadeAbovePrimary");
    let f = at(&mut s, 260.0);
    assert!(close(veil(&f).unwrap(), 1.0));
    assert_eq!(words(&f, "below"), "snes");
    let f = at(&mut s, 270.0);
    assert_eq!(words(&f, "below"), "n64", "past half way the new system shows");
    assert!(close(veil(&f).unwrap(), 1.0));
    let f = at(&mut s, 364.0);
    assert!(close(veil(&f).unwrap(), 2.0 / 3.0), "{:?}", veil(&f));
    let f = at(&mut s, 416.0);
    assert!(close(veil(&f).unwrap(), 1.0 / 3.0), "{:?}", veil(&f));
    assert!(f.moving);
    let f = at(&mut s, 520.0);
    assert_eq!(veil(&f), None);
    assert!(!f.moving);
}

#[test]
fn fade_above_primary_fades_the_elements_above_with_the_black() {
    let t = Synth::new();
    let mut s = systems_stage(&t, "fade", "<fadeAbovePrimary>true</fadeAbovePrimary>");
    tap(&mut s);
    let f = at(&mut s, 26.0);
    assert!(close(find(&f, "above").unwrap().opacity, 0.75), "{}", find(&f, "above").unwrap().opacity);
    assert!(close(find(&f, "below").unwrap().opacity, 1.0), "below the primary the black does the dimming");
    let f = at(&mut s, 260.0);
    assert!(close(find(&f, "above").unwrap().opacity, 0.0));
    let f = at(&mut s, 416.0);
    assert!(close(find(&f, "above").unwrap().opacity, 2.0 / 3.0));
    let f = at(&mut s, 600.0);
    assert!(close(find(&f, "above").unwrap().opacity, 1.0));
}

#[test]
fn a_held_direction_keeps_the_faded_elements_off_from_the_darkest() {
    for above in [false, true] {
        let t = Synth::new();
        let mut s = systems_stage(&t, "fade", if above { "<fadeAbovePrimary>true</fadeAbovePrimary>" } else { "" });
        s.current.press(1, 0, false);
        let f = at(&mut s, 300.0);
        assert!(find(&f, "below").is_some(), "before 0.7 of the run the elements are drawn");
        // 0.7 x 520 = 364 ms: ES-DE's mMaxFade, with the direction held (SystemView.cpp 462).
        let f = at(&mut s, 400.0);
        assert!(find(&f, "below").is_none() && veil(&f).is_none(), "{:?}", order(&f));
        assert_eq!(find(&f, "above").is_none(), above, "fadeAbovePrimary {above}: {:?}", order(&f));
        s.current.release(ms(450.0));
        let f = at(&mut s, 450.0);
        assert!(find(&f, "below").is_some() && find(&f, "above").is_some(), "let go, they are drawn again: {:?}", order(&f));
    }
}

#[test]
fn a_press_after_a_long_hold_starts_with_the_elements_off_until_the_black_is_whole() {
    let t = Synth::new();
    let mut s = systems_stage(&t, "fade", "");
    s.current.press(1, 0, false);
    s.advance(ms(400.0));
    s.current.release(ms(400.0));
    let f = at(&mut s, 1000.0);
    assert!(find(&f, "below").is_some());
    // mMaxFade stayed set; the next press navigates, so the elements are off until the new fade's black is whole, 0.2 x 520
    // = 104 ms (SystemView.cpp 453 and 471-472).
    s.current.press(1, ms(1000.0), false);
    let f = at(&mut s, 1050.0);
    assert!(find(&f, "below").is_none(), "{:?}", order(&f));
    let f = at(&mut s, 1105.0);
    assert!(find(&f, "below").is_some(), "{:?}", order(&f));
}

#[test]
fn a_grid_fades_between_systems_over_its_shorter_run() {
    let t = Synth::new();
    let caps = profile("<systemToSystem>fade</systemToSystem>");
    let grid = "<grid name=\"c\"><pos>0 0.4</pos><size>1 0.4</size><itemSize>0.2 0.2</itemSize></grid>";
    let mut s = Stage::new(data(&t, &caps, &format!("{grid}{BELOW}"), ""), "system", 0);
    tap(&mut s);
    // 390 ms: a grid's 300 ms (SystemView.cpp 432) x 1.3 (476); 0.05 of it is 19.5 ms, a black of 0.25 (453).
    let f = at(&mut s, 19.5);
    assert!(close(veil(&f).unwrap(), 0.25), "{:?}", veil(&f));
    let f = at(&mut s, 390.0);
    assert_eq!(veil(&f), None);
}

#[test]
fn instant_and_slide_between_systems_draw_as_before() {
    for kind in ["instant", "slide"] {
        let t = Synth::new();
        let mut s = systems_stage(&t, kind, "<fadeAbovePrimary>true</fadeAbovePrimary>");
        tap(&mut s);
        let f = at(&mut s, 26.0);
        assert_eq!(veil(&f), None, "{kind}");
        assert_eq!(words(&f, "below"), "n64", "{kind}: the new system's elements at once");
        assert!(close(find(&f, "above").unwrap().opacity, 1.0), "{kind}");
    }
}

#[test]
fn the_system_view_draws_an_element_at_the_primary_s_z_index_beneath_it() {
    let t = Synth::new();
    let tie = "<text name=\"tie\"><pos>0 0</pos><size>1 0.1</size><text>tie</text><zIndex>50</zIndex></text>";
    let s = Stage::new(data(&t, "", &format!("{CAROUSEL}</carousel>{tie}{ABOVE}"), ""), "system", 0);
    assert_eq!(order(&s.frame()), ["tie", "c", "above"], "SystemView::render draws zIndex <= the primary's before it");
}

// ES-DE keeps the help, clock and status out of the view's children (SystemView.cpp 758-772) and draws the clock and status
// from the window after the views (Window.cpp 715-735), so no fade between systems dims them; their types have no zIndex,
// and one a theme writes is dropped, so the scene ranks them over the black whatever the theme says.
#[test]
fn the_status_stays_over_a_fade_between_systems_even_with_a_low_z_index_written() {
    let t = Synth::new();
    let status = "<systemstatus name=\"status\"><zIndex>10</zIndex></systemstatus>";
    let caps = profile("<systemToSystem>fade</systemToSystem>");
    let mut s = Stage::new(data(&t, &caps, &format!("{CAROUSEL}</carousel>{BELOW}{status}"), ""), "system", 0);
    tap(&mut s);
    let f = at(&mut s, 26.0);
    let order = order(&f);
    let place = |n: &str| order.iter().position(|o| o == n).unwrap_or_else(|| panic!("{n} not drawn: {order:?}"));
    assert!(place("veil") < place("c") && place("c") < place("status"), "the status over the primary and the black: {order:?}");
}

const LIST: &str = "<textlist name=\"l\"><pos>0 0</pos><size>0.5 0.5</size><fontSize>0.1</fontSize></textlist>";

fn views_stage(t: &Synth, kinds: &str, view: &str) -> Stage {
    let system = format!("{CAROUSEL}</carousel>{BELOW}<systemstatus name=\"status\"/>");
    let gamelist = format!("{LIST}<text name=\"title\"><pos>0.5 0</pos><size>0.5 0.1</size><text>title</text></text><systemstatus name=\"status\"/>");
    Stage::opening(data(t, &profile(kinds), &system, &gamelist), view, 0)
}

#[test]
fn a_fade_between_views_goes_to_black_holds_and_comes_back_with_the_new_view() {
    let t = Synth::new();
    let mut s = views_stage(&t, "<systemToGamelist>fade</systemToGamelist><gamelistToSystem>fade</gamelistToSystem>", "system");
    s.switch(0);
    // ViewController.cpp 971-972: FADE_DURATION 120 ms to black, FADE_WAIT 200 ms held, 120 ms back (the second
    // animation reversed, 979); 60 ms is half way up, 380 ms half way down.
    let f = at(&mut s, 60.0);
    assert!(close(veil(&f).unwrap(), 0.5));
    assert!(find(&f, "below").is_some() && find(&f, "title").is_none(), "the view left shows until the black covers it");
    assert!(f.leaving_primary.is_some() && f.primary.is_none());
    assert_eq!(order(&f).last().map(String::as_str), Some("status"), "the status is drawn over the black: {:?}", order(&f));
    let f = at(&mut s, 200.0);
    assert!(close(veil(&f).unwrap(), 1.0));
    assert!(find(&f, "title").is_some() && find(&f, "below").is_none());
    let f = at(&mut s, 380.0);
    assert!(close(veil(&f).unwrap(), 0.5), "{:?}", veil(&f));
    assert!(f.moving);
    let f = at(&mut s, 440.0);
    assert_eq!(veil(&f), None);
    assert!(!f.moving && f.leaving_primary.is_none());
    // And back to the system view the same way.
    s.switch(ms(1000.0));
    let f = at(&mut s, 1060.0);
    assert!(close(veil(&f).unwrap(), 0.5) && find(&f, "title").is_some());
    let f = at(&mut s, 1200.0);
    assert!(find(&f, "below").is_some());
}

#[test]
fn a_fade_begun_over_another_starts_from_its_black() {
    let t = Synth::new();
    let mut s = views_stage(&t, "<systemToGamelist>fade</systemToGamelist><gamelistToSystem>fade</gamelistToSystem>", "system");
    s.switch(0);
    s.advance(ms(60.0));
    s.switch(ms(60.0));
    let f = at(&mut s, 60.0);
    assert!(close(veil(&f).unwrap(), 0.5), "{:?}", veil(&f));
    assert!(find(&f, "below").is_some(), "the system view is still the one on the screen");
    let f = at(&mut s, 120.0);
    assert!(close(veil(&f).unwrap(), 1.0));
    let f = at(&mut s, 200.0);
    assert!(find(&f, "below").is_some() && find(&f, "title").is_none(), "{:?}", order(&f));
}

#[test]
fn instant_and_slide_between_views_draw_as_before() {
    let t = Synth::new();
    let mut s = views_stage(&t, "<systemToGamelist>instant</systemToGamelist>", "system");
    s.switch(0);
    let f = at(&mut s, 60.0);
    assert!(veil(&f).is_none() && find(&f, "title").is_some() && !f.moving);
    let t = Synth::new();
    let mut s = views_stage(&t, "<systemToGamelist>slide</systemToGamelist>", "system");
    s.switch(0);
    let f = at(&mut s, 60.0);
    assert!(veil(&f).is_none() && f.leaving_primary.is_some() && f.primary.is_some(), "a slide draws both views");
}

#[test]
fn the_program_opens_on_black_and_fades_its_first_view_in() {
    for (kind, view, shown) in [("startupToSystem", "system", "below"), ("startupToGamelist", "gamelist", "title")] {
        let t = Synth::new();
        let mut s = views_stage(&t, &format!("<{kind}>fade</{kind}>"), view);
        let f = at(&mut s, 60.0);
        assert_eq!(order(&f), ["veil"], "{kind}: nothing is in front of the camera before the black covers the screen");
        let f = at(&mut s, 380.0);
        assert!(close(veil(&f).unwrap(), 0.5) && find(&f, shown).is_some(), "{kind}");
        let f = at(&mut s, 440.0);
        assert!(veil(&f).is_none() && !f.moving, "{kind}");
        let t = Synth::new();
        let mut s = views_stage(&t, &format!("<{kind}>instant</{kind}>"), view);
        assert!(veil(&at(&mut s, 0.0)).is_none(), "{kind} instant");
    }
}

#[test]
fn another_system_s_gamelist_fades_in_when_the_profile_says() {
    let t = Synth::new();
    let caps = "<gamelistToGamelist>fade</gamelistToGamelist><startupToGamelist>instant</startupToGamelist>";
    let mut s = views_stage(&t, caps, "gamelist");
    let mut next = s.current.data.clone();
    next.system_index = 2;
    s.open_gamelist(next, 0);
    let f = at(&mut s, 60.0);
    assert!(close(veil(&f).unwrap(), 0.5));
    assert_eq!(f.leaving_primary.as_ref().map(|p| p.rows.system), Some(1), "the gamelist left shows first");
    let f = at(&mut s, 200.0);
    assert_eq!(f.primary.as_ref().map(|p| p.rows.system), Some(2));
    let t = Synth::new();
    let mut s = views_stage(&t, "<gamelistToGamelist>instant</gamelistToGamelist>", "gamelist");
    let mut next = s.current.data.clone();
    next.system_index = 2;
    s.open_gamelist(next, 0);
    let f = at(&mut s, 0.0);
    assert!(veil(&f).is_none() && f.primary.as_ref().map(|p| p.rows.system) == Some(2));
}
