// The painter's contract: Stage::frame's records, the painter's measurements, mapping.tsv and the marks (EmuSen_Qt.md §4.5.5).
mod common;

use std::collections::BTreeSet;

use common::*;
use emusen_esde::check;
use emusen_esde::loader::{catalog, Choices, ThemeSystem};
use emusen_esde::marks::MarkId;
use emusen_esde::scene::mapping::{self, Status as Mapped};
use emusen_esde::scene::*;

const W: f64 = 320.0;
const H: f64 = 200.0;

fn games() -> Vec<Game> {
    ["Aurora Drift", "Brass Lantern", "Cobalt Harbor", "Dune Relay", "Ember Circuit", "Fable of Tiles"]
        .iter()
        .enumerate()
        .map(|(i, t)| Game { favorite: i % 2 == 0, rating: Some(0.6), ..Game::new(t, &format!("{t}.nes")) })
        .collect()
}

// A synthetic theme for three systems, each system's carousel artwork its own, kept with the data.
fn stage(t: &Synth, system: &str, gamelist: &str, view: &str, game: usize) -> Stage {
    stage_with(t, "", system, gamelist, view, game)
}

fn stage_with(t: &Synth, capabilities: &str, system: &str, gamelist: &str, view: &str, game: usize) -> Stage {
    t.capabilities(capabilities).theme(&format!("<view name=\"system\">{system}</view><view name=\"gamelist\">{gamelist}</view>"));
    let choices = Choices { screen_width: W as i32, screen_height: H as i32, ..Choices::default() };
    let systems = ["nes", "snes", "n64"]
        .iter()
        .map(|s| {
            let sys = ThemeSystem::new(s, &format!("{s} full"), s);
            System::new(sys.clone(), t.load_with(&choices, &sys, None), games())
        })
        .collect();
    let mut data = Data::new(systems, W, H);
    data.game_index = game;
    data.system_index = 1;
    data.show_clock = false;
    Stage::new(data, view, 0)
}

fn placed<'a>(f: &'a Frame, name: &str) -> &'a Placed {
    f.elements
        .iter()
        .find(|p| p.name == name)
        .unwrap_or_else(|| panic!("{name} not drawn: {:?}", f.elements.iter().map(|p| &p.name).collect::<Vec<_>>()))
}

// Theme values are f32, so pixels agree to a ten-thousandth.
fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-4
}

const CAROUSEL: &str = "<carousel name=\"c\"><pos>0 0</pos><size>1 1</size><maxItemCount>3</maxItemCount><itemScale>1.5</itemScale><unfocusedItemOpacity>0.3</unfocusedItemOpacity>";

#[test]
fn mapping_lists_every_catalogue_pair_once_and_the_dimmed_help_as_accepted() {
    let listed: Vec<(String, String)> = mapping::pairs().map(|(k, p, _)| (k.to_owned(), p.to_owned())).collect();
    let unique: BTreeSet<_> = listed.iter().cloned().collect();
    assert_eq!(unique.len(), listed.len(), "a pair listed twice");
    let catalogue: BTreeSet<(String, String)> =
        catalog().elements.iter().flat_map(|e| e.properties.iter().map(|p| (e.kind.clone(), p.name.clone()))).collect();
    assert_eq!(unique, catalogue);
    let dimmed: Vec<_> = mapping::pairs().filter(|(k, p, _)| *k == "helpsystem" && p.ends_with("Dimmed")).collect();
    assert_eq!(dimmed.len(), 8);
    assert!(dimmed.iter().all(|(_, _, s)| **s == Mapped::Accepted));
    assert!(mapping::is_mapped("helpsystem", "textColorDimmed"));
    assert_eq!(mapping::status("video", "delay"), Some(&Mapped::Later("video".into())));
    assert!(!mapping::is_mapped("carousel", "itemLinearScale"));
    assert_eq!(mapping::pairs().filter(|(_, _, s)| **s == Mapped::Drawn).count(), 347);
    assert!(!mapping::is_drawn("gameselector") && !mapping::is_drawn("sound"), "the gameselector waits for its family; sound is played");
}

#[test]
fn a_theme_setting_only_accepted_pairs_counts_none_unsupported() {
    let t = Synth::new();
    t.capabilities("").theme(
        "<view name=\"system\"><helpsystem name=\"help\"><textColorDimmed>FF0000</textColorDimmed><opacityDimmed>0.5</opacityDimmed></helpsystem></view>
         <view name=\"gamelist\"><carousel name=\"c\"><itemLinearScale>0.1 0.1</itemLinearScale></carousel></view>",
    );
    let m = check::measure(&t.root, &["nes"]);
    assert_eq!(m.properties.get("helpsystem.textColorDimmed"), Some(&"set"));
    assert_eq!(m.properties.get("carousel.itemLinearScale"), Some(&"unsupported"), "a later pair is unsupported, not unknown");
    assert!(m.loads());
}

#[test]
fn every_mark_is_polylines_in_its_unit_box() {
    let mut names = BTreeSet::new();
    for id in MarkId::ALL {
        let mark = id.mark();
        assert!(mark.aspect > 0.0 && !mark.parts.is_empty(), "{id:?}");
        for part in &mark.parts {
            assert!(part.fill || part.stroke > 0.0, "{id:?} draws nothing");
            for ring in &part.rings {
                for p in ring {
                    assert!((-0.01..=1.01).contains(&p[0]) && (-0.01..=1.01).contains(&p[1]), "{id:?} leaves its box at {p:?}");
                }
            }
        }
        assert!(names.insert(id.name()), "{} named twice", id.name());
    }
    assert_eq!(MarkId::Battery(3).name(), "battery3");
    assert_eq!(MarkId::BadgeKidGame.name(), "badge-kid-game");
}

#[test]
fn a_carousel_s_selected_slot_grows_from_its_edge_and_the_others_fade() {
    let t = Synth::new();
    let s = stage(&t, &format!("{CAROUSEL}</carousel>"), "", "system", 0);
    let f = s.frame();
    let p = f.primary.as_ref().expect("a primary");
    assert_eq!((p.rows, p.selected), (Rows { view_is_system: true, system: 1 }, 1));
    let Layout::Carousel(c) = &p.layout else { panic!("{:?}", p.layout) };
    let selected = c.slots.last().unwrap();
    assert_eq!(selected.row, 1, "the selection is drawn last");
    // itemSize 0.25 0.155 of 320 × 200, three items across.
    let near = |a: f64, b: f64| (a - b).abs() < 1e-4;
    assert!(near(selected.x, 120.0) && near(selected.y, 84.5) && near(selected.w, 80.0) && near(selected.h, 31.0), "{selected:?}");
    assert!(
        near(selected.scale, 1.5)
            && near(selected.opacity, 1.0)
            && near(selected.scale_origin.0, 160.0)
            && near(selected.scale_origin.1, 100.0)
    );
    let next = c.slots.iter().rev().find(|s| s.row == 2).unwrap();
    assert!(near(next.x - selected.x, 320.0 / 3.0) && near(next.opacity, 0.3) && near(next.scale, 1.0), "{next:?}");
    assert!(matches!(placed(&f, "c").paint, Paint::Primary { leaving: false }));
    assert!(!f.moving && f.next_change_ms.is_none(), "a carousel at rest asks for no frame");
}

#[test]
fn a_carousel_mid_step_has_its_slots_between_items_and_moves() {
    let t = Synth::new();
    let mut s = stage(&t, &format!("{CAROUSEL}</carousel>"), "", "system", 0);
    s.current.step(1, 0);
    s.advance(ms(100.0));
    let f = s.frame();
    let Layout::Carousel(c) = &f.primary.as_ref().unwrap().layout else { panic!() };
    let p = s.current.position();
    let selected = c.slots.iter().rev().find(|s| s.row == 2).unwrap();
    assert!((selected.x - (120.0 - (p - 2.0) * 320.0 / 3.0)).abs() < 1e-4, "{selected:?} at {p}");
    assert!(f.moving);
}

#[test]
fn a_wheel_turns_each_item_about_its_centre_and_a_row_reflects() {
    let t = Synth::new();
    let wheel = format!("{CAROUSEL}<type>verticalWheel</type><itemRotation>10</itemRotation><itemsBeforeCenter>2</itemsBeforeCenter><itemsAfterCenter>1</itemsAfterCenter></carousel>");
    let s = stage(&t, &wheel, "", "system", 0);
    let Layout::Carousel(c) = &s.frame().primary.unwrap().layout else { panic!() };
    assert_eq!(c.slots.len(), 4, "two before, the selection and one after, wrapped over three systems");
    let before = c.slots.iter().find(|s| close(s.rotation, -10.0)).expect("one item before turns back");
    assert!(c.slots.iter().all(|s| s.x == before.x && s.y == before.y && s.rotation_origin == before.rotation_origin));
    assert!(c.slots.iter().all(|s| s.reflection.is_none()));
    let row = stage(
        &t,
        &format!("{CAROUSEL}<reflections>true</reflections><reflectionsOpacity>0.4</reflectionsOpacity></carousel>"),
        "",
        "system",
        0,
    );
    let Layout::Carousel(c) = &row.frame().primary.unwrap().layout else { panic!() };
    let selected = c.slots.last().unwrap();
    assert!(selected.reflection.is_some_and(|(o, f)| (o - 0.4).abs() < 1e-6 && f == 1.0));
    assert!((selected.y - (200.0 - 62.0) / 2.0).abs() < 1e-4, "the row makes room for the reflection: {selected:?}");
}

const LIST: &str = "<textlist name=\"l\"><pos>0 0</pos><size>0.5 0.5</size><fontSize>0.1</fontSize><lineSpacing>1</lineSpacing><textHorizontalScrollDelay>1</textHorizontalScrollDelay></textlist>";

#[test]
fn a_text_list_keeps_the_selection_mid_window_and_scrolls_its_name_once_measured() {
    let t = Synth::new();
    let mut s = stage(&t, "", LIST, "gamelist", 3);
    let f = s.frame();
    let p = f.primary.clone().unwrap();
    let Layout::TextList(l) = &p.layout else { panic!() };
    // 100 px high, rows of 20: five shown, the fourth of six kept in the middle, clamped at the end.
    assert_eq!((l.first_row, l.row_h, l.content_y), (1, 20.0, 20.0));
    assert_eq!(l.selector.unwrap().y, 40.0);
    assert!(l.marquee.is_none() && !f.moving, "no scroll before the painter measures");
    let rows = s.rows(p.rows).unwrap();
    assert_eq!((rows.len(), rows[0].marker, rows[1].marker), (6, Marker::Star, Marker::None));
    assert!(s.measured(p.id, 400.0, 20.0));
    assert!(!s.measured(p.id, 400.0, 20.0), "the same measurement changes nothing");
    let f = s.frame();
    let Layout::TextList(l) = &f.primary.unwrap().layout else { panic!() };
    assert_eq!(l.marquee.map(|m| m.x), Some(0.0));
    assert!(!f.moving && f.next_change_ms == Some(1000), "the marquee waits its delay: {:?}", f.next_change_ms);
    s.advance(ms(1500.0));
    let f = s.frame();
    let Layout::TextList(l) = &f.primary.unwrap().layout else { panic!() };
    assert!(l.marquee.unwrap().x > 0.0 && f.moving);
    s.current.step(1, ms(1500.0));
    let Layout::TextList(l) = &s.frame().primary.unwrap().layout else { panic!() };
    assert!(l.marquee.is_none(), "a new name is measured afresh");
}

#[test]
fn a_picture_with_max_size_is_contained_once_its_own_size_is_known() {
    let t = Synth::new();
    t.asset("art.png");
    let image = "<image name=\"i\"><pos>0.5 0.5</pos><origin>0.5 0.5</origin><maxSize>0.5 0.5</maxSize><path>./art.png</path></image>";
    let mut s = stage(&t, image, "", "system", 0);
    let i = placed(&s.frame(), "i").clone();
    assert_eq!((i.x, i.y, i.w, i.h), (80.0, 50.0, 160.0, 100.0));
    let Paint::Picture(p) = &i.paint else { panic!() };
    assert!(p.source.ends_with("art.png") && p.fit == Fit::Contain);
    assert!(s.measured(i.id, 400.0, 100.0));
    let i = placed(&s.frame(), "i").clone();
    assert_eq!((i.x, i.y, i.w, i.h), (80.0, 80.0, 160.0, 40.0));
}

#[test]
fn a_text_sized_by_its_words_is_placed_by_its_origin_once_measured() {
    let t = Synth::new();
    let text = "<text name=\"t\"><pos>1 0</pos><origin>1 0</origin><text>Hello</text><letterCase>uppercase</letterCase><fontSize>0.1</fontSize><backgroundColor>000000</backgroundColor><backgroundMargins>0.01 0.02</backgroundMargins></text>";
    let mut s = stage(&t, text, "", "system", 0);
    let p = placed(&s.frame(), "t").clone();
    let Paint::Text(x) = &p.paint else { panic!() };
    assert!(x.text == "HELLO" && close(x.px, 20.0) && !x.wrap);
    let pad = x.background.unwrap().padding;
    assert!(close(pad[0], 3.2) && close(pad[2], 6.4) && pad[1] == 0.0, "{pad:?}");
    s.measured(p.id, 50.0, 30.0);
    let p = placed(&s.frame(), "t").clone();
    assert_eq!((p.x, p.w, p.h), (270.0, 50.0, 30.0));
}

#[test]
fn the_help_names_the_project_s_actions_on_the_buttons_and_icons_of_the_pad() {
    let t = Synth::new();
    t.asset("b.svg");
    let help = "<helpsystem name=\"h\"><entries>a,b,up/down</entries><customButtonIcon button=\"button_b_PS\">./b.svg</customButtonIcon></helpsystem>";
    let mut s = stage(&t, "", help, "gamelist", 0);
    let f = s.frame();
    let Paint::Help(h) = &placed(&f, "h").paint else { panic!() };
    let shown: Vec<_> = h.entries.iter().map(|e| (e.label.as_str(), e.button, e.pair)).collect();
    assert_eq!(shown, [("LAUNCH", PadButton::South, false), ("BACK", PadButton::East, false), ("CHOOSE", PadButton::DpadUpDown, true)]);
    assert!(h.entries.iter().all(|e| e.icon.is_none()) && h.filled_glyphs);
    s.current.data.controller_type = ControllerType::PlayStation;
    s.current.data.swap = true;
    let f = s.frame();
    let Paint::Help(h) = &placed(&f, "h").paint else { panic!() };
    assert_eq!(h.entries[0].button, PadButton::East);
    assert!(h.entries[0].icon.as_deref().is_some_and(|p| p.ends_with("b.svg")), "the swapped Launch takes the theme's B icon");
}

#[test]
fn badges_lay_out_the_game_s_slots_and_status_shows_the_device() {
    let t = Synth::new();
    let gamelist = "<badges name=\"b\"><pos>0 0</pos><size>0.4 0.1</size><slots>favorite controller</slots><itemsPerLine>2</itemsPerLine><lines>1</lines><itemMargin>0 0</itemMargin></badges>
        <systemstatus name=\"s\"/><rating name=\"r\"><pos>0 0.5</pos><size>0 0.1</size></rating>";
    let mut s = stage(&t, "", gamelist, "gamelist", 0);
    s.current.data.systems[1].games[0].controller = Some("gamepad_nintendo_snes".into());
    let f = s.frame();
    let Paint::Badges(b) = &placed(&f, "b").paint else { panic!() };
    assert_eq!(b.slots.len(), 2);
    let cell = b.slots[1].cell;
    assert!(close(cell.0, 64.0) && cell.1 == 0.0 && close(cell.2, 64.0) && close(cell.3, 20.0), "{cell:?}");
    assert_eq!(b.slots[0].icon.icon, Icon::Mark(MarkId::BadgeFavorite));
    assert_eq!(b.slots[1].controller.as_ref().map(|c| c.icon.clone()), Some(Icon::Mark(MarkId::ControllerSnes)));
    let Paint::Status(st) = &placed(&f, "s").paint else { panic!() };
    let icons: Vec<_> = st.items.iter().map(|i| (i.icon.clone(), i.text.clone())).collect();
    assert_eq!(icons, [(Some(Icon::Mark(MarkId::Wifi)), None), (Some(Icon::Mark(MarkId::Battery(4))), None), (None, Some("80%".into()))]);
    let r = placed(&f, "r");
    assert!(close(r.w, 100.0) && close(r.h, 20.0));
    let Paint::Rating(rating) = &r.paint else { panic!() };
    assert!(close(rating.value, 0.6) && rating.filled == Icon::Mark(MarkId::Star));
}

#[test]
fn the_overlays_hold_still_while_the_views_pan_and_both_primaries_are_drawn() {
    let t = Synth::new();
    let slide =
        "<transitions name=\"slide\"><systemToGamelist>slide</systemToGamelist><gamelistToSystem>slide</gamelistToSystem></transitions>";
    let mut s = stage_with(
        &t,
        slide,
        &format!("{CAROUSEL}</carousel><systemstatus name=\"s\"/>"),
        &format!("{LIST}<systemstatus name=\"s\"/>"),
        "system",
        0,
    );
    s.switch(0);
    s.advance(ms(100.0));
    let f = s.frame();
    assert!(f.moving && f.primary.is_some() && f.leaving_primary.is_some());
    let statuses: Vec<_> = f.elements.iter().filter(|p| p.kind == "systemstatus").collect();
    assert_eq!(statuses.len(), 1, "the leaving view's overlays are not drawn");
    assert!(f.elements.iter().any(|p| matches!(p.paint, Paint::Primary { leaving: true })));
    let moving_y = statuses[0].y;
    s.advance(ms(1000.0));
    let f = s.frame();
    assert!(!f.moving && f.leaving_primary.is_none());
    assert_eq!(f.elements.iter().find(|p| p.kind == "systemstatus").unwrap().y, moving_y, "the status held still through the pan");
}

#[test]
fn a_held_list_shows_the_quick_scroll_letters_once_the_metadata_fades() {
    let t = Synth::new();
    let mut s = stage(&t, "", LIST, "gamelist", 0);
    s.current.data.scroll_overlay = true;
    s.current.data.systems[1].favorites_on_top = true;
    assert!(s.frame().overlay.is_none());
    s.current.press(1, 0, false);
    s.advance(ms(900.0));
    let f = s.frame();
    let o = f.overlay.expect("letters while held");
    assert_eq!(o.px, 30.0);
    assert!(o.letters.chars().count() == 2);
    s.current.release(ms(900.0));
    assert!(s.frame().overlay.is_none());
}

#[test]
fn a_shown_clock_wakes_the_painter_at_the_next_minute() {
    let t = Synth::new();
    let mut s = stage(&t, "<clock name=\"k\"><pos>0 0</pos></clock>", "", "system", 0);
    s.current.data.show_clock = true;
    s.current.data.now = DateTime::new(2026, 11, 8, 12, 0, 45);
    let f = s.frame();
    assert!(!f.moving);
    assert_eq!(f.next_change_ms, Some(15_000));
    let Paint::Text(k) = &placed(&f, "k").paint else { panic!() };
    assert_eq!(k.text, "12:00");
}
