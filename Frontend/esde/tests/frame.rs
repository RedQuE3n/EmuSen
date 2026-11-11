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
    assert_eq!(mapping::pairs().filter(|(_, _, s)| **s == Mapped::Drawn).count(), 361);
    assert!(
        mapping::is_drawn("gameselector") && !mapping::is_drawn("sound"),
        "the gameselector picks the system view's games; sound is played"
    );
}

#[test]
fn a_theme_setting_only_accepted_pairs_counts_none_unsupported() {
    let t = Synth::new();
    t.capabilities("").theme(
        "<view name=\"system\"><helpsystem name=\"help\"><textColorDimmed>FF0000</textColorDimmed><opacityDimmed>0.5</opacityDimmed></helpsystem></view>
         <view name=\"gamelist\"><carousel name=\"c\"><itemLinearScale>0.1 0.1</itemLinearScale></carousel></view>",
    );
    let m = check::measure(&t.root, &["nes"]);
    assert_eq!(m.properties.get("helpsystem.textColorDimmed"), Some(&"accepted"), "accepted is counted apart from drawn");
    assert_eq!(m.properties.get("carousel.itemLinearScale"), Some(&"unsupported"), "a later pair is unsupported, not unknown");
    assert!(m.loads());
}

#[test]
fn accepted_pairs_have_a_column_of_their_own_and_are_never_whole() {
    let t = Synth::new();
    t.capabilities("").theme(
        "<view name=\"system\"><helpsystem name=\"help\"><textColorDimmed>FF0000</textColorDimmed><opacityDimmed>0.5</opacityDimmed></helpsystem></view>",
    );
    let m = check::measure(&t.root, &["nes"]);
    assert_eq!(m.count("accepted"), 2);
    assert!(m.loads() && m.undrawn.is_empty() && m.count("unsupported") == 0 && m.count("unknown") == 0);
    assert!(!m.whole(), "a theme whose only pairs are accepted is not drawn whole");
    let header: Vec<&str> = check::SUMMARY_HEADER.split('\t').collect();
    let row = check::summary_row(&m);
    let cells: Vec<&str> = row.split('\t').collect();
    assert_eq!(cells.len(), header.len());
    assert_eq!(cells[header.iter().position(|c| *c == "accepted").unwrap()], "2");
    assert_eq!(cells[header.iter().position(|c| *c == "whole").unwrap()], "no");
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
fn a_pictures_own_size_holds_across_a_resize() {
    let t = Synth::new();
    t.asset("art.png");
    let images = "<image name=\"i\"><pos>0 0</pos><maxSize>0.5 0.5</maxSize><path>./art.png</path></image>\
                  <image name=\"j\"><pos>0 0.5</pos><size>0 0.25</size><path>./art.png</path></image>";
    let mut s = stage(&t, images, "", "system", 0);
    for name in ["i", "j"] {
        let id = placed(&s.frame(), name).id;
        s.measured(id, 400.0, 100.0);
    }
    let boxes = |s: &mut Stage| ["i", "j"].map(|n| placed(&s.frame(), n).clone()).map(|p| (p.w / s.current.data.width, p.h / s.current.data.height));
    let before = boxes(&mut s);
    assert_eq!(before[0], (0.5, 0.2));
    let mut data = s.current.data.clone();
    (data.width, data.height) = (W * 1.5, H * 1.5);
    s.restart(data, "system", 0);
    assert_eq!(boxes(&mut s), before, "no delegate need report again: the picture's aspect is its file's");
    // Decoded again at the new box's width, the picture's height rounds to a whole pixel: the same aspect, so nothing moves.
    let id = placed(&s.frame(), "i").id;
    assert!(!s.measured(id, 240.0, 61.0), "a rounding of the same aspect is no new measurement");
    assert!(s.measured(id, 240.0, 80.0), "another aspect is");
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
fn the_help_names_es_de_s_actions_on_the_buttons_and_icons_of_the_pad() {
    let t = Synth::new();
    t.asset("b.svg");
    let help = "<helpsystem name=\"h\"><entries>a,b,up/down</entries><customButtonIcon button=\"button_b_PS\">./b.svg</customButtonIcon></helpsystem>";
    let mut s = stage(&t, "", help, "gamelist", 0);
    let f = s.frame();
    let Paint::Help(h) = &placed(&f, "h").paint else { panic!() };
    let shown: Vec<_> = h.entries.iter().map(|e| (e.label.as_str(), e.button, e.pair)).collect();
    // ES-DE 3.4.1 lists no up/down entry in a gamelist, and A reads Select on a game as on a folder.
    assert_eq!(shown, [("SELECT", PadButton::South, false), ("BACK", PadButton::East, false)]);
    assert!(h.entries.iter().all(|e| e.icon.is_none()) && h.filled_glyphs);
    s.current.data.controller_type = ControllerType::PlayStation;
    s.current.data.swap = true;
    let f = s.frame();
    let Paint::Help(h) = &placed(&f, "h").paint else { panic!() };
    assert_eq!(h.entries[0].button, PadButton::East);
    assert!(h.entries[0].icon.as_deref().is_some_and(|p| p.ends_with("b.svg")), "the swapped Select takes the theme's B icon");
}

#[test]
fn the_help_takes_es_de_s_help_face_where_found_and_its_default_face_else() {
    let t = Synth::new();
    let gamelist = "<helpsystem name=\"h\"><entries>a,b</entries></helpsystem><text name=\"t\"><text>HELLO</text></text>";
    let mut s = stage(&t, "", gamelist, "gamelist", 0);
    s.current.data.default_font = Some("/f/Akrobat-Regular.ttf".to_owned());
    let f = s.frame();
    let Paint::Help(h) = &placed(&f, "h").paint else { panic!() };
    assert_eq!(h.font, Face { file: Some("/f/Akrobat-Regular.ttf".to_owned()), own: true }, "no help face: the default face");
    s.current.data.help_font = Some("/f/Akrobat-SemiBold.ttf".to_owned());
    let f = s.frame();
    let Paint::Help(h) = &placed(&f, "h").paint else { panic!() };
    assert_eq!(h.font, Face { file: Some("/f/Akrobat-SemiBold.ttf".to_owned()), own: false }, "its advances rounded as a theme face's");
    let Paint::Text(x) = &placed(&f, "t").paint else { panic!() };
    assert_eq!(x.font.file.as_deref(), Some("/f/Akrobat-Regular.ttf"), "other text keeps the default face");
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

#[test]
fn the_help_follows_es_de_s_order_and_the_player_s_switches() {
    let t = Synth::new();
    let help = "<helpsystem name=\"h\"/>";
    let labels = |s: &Stage| {
        let f = s.frame();
        let Paint::Help(h) = &placed(&f, "h").paint else { panic!() };
        h.entries.iter().map(|e| e.label.clone()).collect::<Vec<_>>()
    };
    let mut s = stage(&t, help, "", "system", 0);
    assert_eq!(labels(&s), ["MENU", "SELECT", "CHOOSE"]);
    s.current.data.help.screensaver = true;
    s.current.data.help.random_systems = true;
    assert_eq!(labels(&s), ["MENU", "SELECT", "SCREENSAVER", "CHOOSE", "RANDOM"]);
    let mut s = stage(&t, "", help, "gamelist", 0);
    s.current.data.help.random_games = true;
    assert_eq!(labels(&s), ["OPTIONS", "MENU", "SELECT", "BACK", "VIEW MEDIA", "FAVORITES", "SYSTEM", "RANDOM"]);
    s.current.data.help.quick_select = "disabled".into();
    assert_eq!(labels(&s), ["OPTIONS", "MENU", "SELECT", "BACK", "VIEW MEDIA", "FAVORITES", "RANDOM"]);
}

#[test]
fn only_a_picture_named_by_path_tiles() {
    let t = Synth::new();
    t.asset("a.png");
    let gamelist = "<image name=\"path\"><pos>0 0</pos><size>0.5 0.5</size><path>./a.png</path><tile>true</tile></image>\
        <image name=\"typed\"><pos>0.5 0</pos><size>0.5 0.5</size><imageType>screenshot</imageType><default>./a.png</default><tile>true</tile></image>";
    let s = stage(&t, "", gamelist, "gamelist", 0);
    let f = s.frame();
    let fit = |name: &str| match &placed(&f, name).paint {
        Paint::Picture(p) => p.fit,
        _ => panic!("{name} is a picture"),
    };
    assert_eq!(fit("path"), Fit::Tile);
    assert_eq!(fit("typed"), Fit::Fill, "THEMES.md: tile can only be used when path has been set");
}

// The system view's pictures and words of a game as a gameselector picks them (THEMES.md, gameselector; EmuSen_Qt.md §4.8.6).
fn selected(t: &Synth, selector: &str, seed: u64, prepare: impl Fn(&mut Data)) -> Stage {
    let covers: String = (0..8)
        .map(|i| format!("<image name=\"g{i}\"><pos>0 0</pos><cropSize>0.1 0.1</cropSize><tile>true</tile><imageType>screenshot</imageType><gameselectorEntry>{i}</gameselectorEntry></image>"))
        .collect();
    let mut s = stage(t, &format!("{CAROUSEL}</carousel>{selector}{covers}<text name=\"title\"><metadata>name</metadata><gameselectorEntry>1</gameselectorEntry></text>"), "", "system", 0);
    let mut data = s.current.data.clone();
    for system in &mut data.systems {
        for g in &mut system.games {
            g.media.insert("screenshot".into(), format!("/shots/{}.png", g.name));
        }
    }
    data.selection_seed = seed;
    prepare(&mut data);
    s.restart(data, "system", 0);
    s
}

fn shown(f: &Frame) -> Vec<String> {
    (0..8)
        .filter_map(|i| f.elements.iter().find(|p| p.name == format!("g{i}")))
        .map(|p| match &p.paint {
            Paint::Picture(pic) => pic.source.trim_start_matches("/shots/").trim_end_matches(".png").to_owned(),
            other => panic!("{other:?}"),
        })
        .collect()
}

#[test]
fn a_gameselector_gives_the_system_view_its_games_by_entry() {
    let t = Synth::new();
    // A shuffle of the six games, none twice; entries past them are not rendered; the same seed draws the same.
    let random = "<gameselector name=\"pick\"><gameCount>8</gameCount></gameselector>";
    let f = selected(&t, random, 7, |_| {}).frame();
    let names = shown(&f);
    assert_eq!(names.len(), 6, "entries 6 and 7 have no game: {names:?}");
    assert_eq!(names.iter().collect::<BTreeSet<_>>().len(), 6, "{names:?}");
    assert_eq!(shown(&selected(&t, random, 7, |_| {}).frame()), names);
    assert_ne!(shown(&selected(&t, random, 8, |_| {}).frame()), names, "another seed, another shuffle");
    let Paint::Picture(p) = &placed(&f, "g0").paint else { panic!() };
    assert_eq!(p.fit, Fit::Cover, "a game's media is cropped, never tiled");
    let Paint::Text(title) = &placed(&f, "title").paint else { panic!() };
    assert_eq!(title.text, names[1]);
    // Each move of the carousel draws again.
    let mut s = selected(&t, random, 7, |_| {});
    s.current.step(1, 0);
    s.advance(ms(2000.0));
    assert_ne!(shown(&s.frame()), names);
    // allowDuplicates fills every entry.
    let duplicates = "<gameselector name=\"pick\"><gameCount>8</gameCount><allowDuplicates>true</allowDuplicates></gameselector>";
    assert_eq!(shown(&selected(&t, duplicates, 7, |_| {}).frame()).len(), 8);
    // The last played, newest first; the most played; an entry past gameCount clamps to its last.
    let last = "<gameselector name=\"pick\"><selection>lastplayed</selection><gameCount>2</gameCount></gameselector>";
    let f = selected(&t, last, 0, |d| {
        d.systems[1].games[3].last_played = Some(DateTime::new(2026, 1, 2, 0, 0, 0));
        d.systems[1].games[1].last_played = Some(DateTime::new(2026, 1, 1, 0, 0, 0));
    })
    .frame();
    assert_eq!(
        shown(&f),
        [
            "Dune Relay",
            "Brass Lantern",
            "Brass Lantern",
            "Brass Lantern",
            "Brass Lantern",
            "Brass Lantern",
            "Brass Lantern",
            "Brass Lantern"
        ]
    );
    let most = "<gameselector name=\"pick\"><selection>mostplayed</selection><gameCount>3</gameCount></gameselector>";
    let f = selected(&t, most, 0, |d| d.systems[1].games[4].play_count = 5).frame();
    assert_eq!(shown(&f), ["Ember Circuit"], "one played game: the other entries are not rendered");
    // A named selector, else the one whose name sorts first.
    let two = "<gameselector name=\"b\"><selection>mostplayed</selection></gameselector><gameselector name=\"a\"><selection>lastplayed</selection></gameselector>";
    let f = selected(&t, two, 0, |d| {
        d.systems[1].games[4].play_count = 5;
        d.systems[1].games[2].last_played = Some(DateTime::new(2026, 1, 1, 0, 0, 0));
    })
    .frame();
    assert_eq!(shown(&f)[0], "Cobalt Harbor");
}

// ES-DE's media viewer (USERGUIDE.md, "Game media viewer"; EmuSen_Qt.md §4.10).
#[test]
fn the_media_viewer_shows_a_game_s_media_in_es_de_s_order_and_stops_at_the_ends() {
    let t = Synth::new();
    let mut s = stage(&t, "", "<helpsystem name=\"h\"/>", "gamelist", 0);
    let mut data = s.current.data.clone();
    let game = &mut data.systems[1].games[0];
    for kind in ["custom", "screenshot", "video", "cover", "marquee", "fanart"] {
        game.media.insert(kind.to_owned(), format!("/m/{kind}"));
    }
    data.systems[1].games[1].folder = true;
    s.restart(data.clone(), "gamelist", 0);
    assert!(s.open_viewer());
    let v = s.viewer.clone().unwrap();
    let kinds: Vec<&str> = v.entries.iter().map(|e| e.kind.as_str()).collect();
    assert_eq!(kinds, ["video", "cover", "screenshot", "fanart", "custom"], "ES-DE's order; a marquee is not shown");
    let item = |s: &Stage| placed(&s.frame(), "item").paint.clone();
    let clip = Clip { source: "/m/video".into(), stretch: false, audio: true, blur: 0.0 };
    assert_eq!(placed(&s.frame(), "video").paint, Paint::Clip(clip), "the video first, played");
    let viewer = s.viewer.as_mut().unwrap();
    assert!(!viewer.browse(-1), "left at the first stays");
    assert!(viewer.jump(true));
    assert!(!viewer.browse(1), "right at the last stays");
    assert!(viewer.jump(false) && viewer.browse(1));
    let Paint::Picture(p) = item(&s) else { panic!("a picture") };
    assert_eq!((p.source.as_str(), p.fit), ("/m/cover", Fit::Contain));
    let f = s.frame();
    let (strip, it) = (placed(&f, "strip"), placed(&f, "item"));
    assert_eq!((strip.h, it.y, it.h), (Viewer::strip(H), Viewer::strip(H), H - Viewer::strip(H)), "the item fills the screen below the strip");
    assert!(f.elements.last().is_some_and(|e| e.name == "help"), "the help is drawn last, over the strip");
    let Paint::Help(h) = &placed(&f, "help").paint else { panic!() };
    assert_eq!(h.entries.iter().map(|e| e.label.as_str()).collect::<Vec<_>>(), ["BROWSE", "FIRST", "LAST"]);
    assert!(h.font.own, "ES-DE's own face");
    s.switch(0);
    assert!(s.viewer.is_none(), "leaving the gamelist closes it");
    let no_video = &mut data.systems[1].games[0].media;
    no_video.remove("video");
    no_video.insert("titlescreen".into(), "/m/titlescreen".into());
    no_video.insert("miximage".into(), "/m/miximage".into());
    s.restart(data.clone(), "gamelist", 0);
    assert!(s.open_viewer());
    let kinds: Vec<String> = s.viewer.as_ref().unwrap().entries.iter().map(|e| e.kind.clone()).collect();
    assert_eq!(kinds, ["screenshot", "cover", "titlescreen", "fanart", "miximage", "custom"], "with no video the screenshot leads");
    s.switch(0);
    data.game_index = 1;
    s.restart(data.clone(), "gamelist", 0);
    assert!(!s.open_viewer(), "a folder opens nothing");
    data.game_index = 2;
    s.restart(data, "gamelist", 0);
    assert!(!s.open_viewer(), "a game with no media opens nothing");
}

// The manual's mode, as ES-DE 3.4.1's captures of 2026-11-09 show it (EmuSen_Qt.md §4.13).
#[test]
fn the_media_viewer_shows_a_game_s_pdf_manual_paged_zoomed_and_panned() {
    let t = Synth::new();
    let mut s = stage(&t, "", "<helpsystem name=\"h\"/>", "gamelist", 0);
    let mut data = s.current.data.clone();
    let game = &mut data.systems[1].games[0];
    game.media.insert("screenshot".into(), "/m/shot".into());
    game.media.insert("manual".into(), "/m/manual.pdf".into());
    s.restart(data, "gamelist", 0);
    assert!(s.open_viewer());
    let labels = |s: &Stage| {
        let f = s.frame();
        let Paint::Help(h) = &placed(&f, "help").paint else { panic!() };
        h.entries.iter().map(|e| e.label.clone()).collect::<Vec<_>>()
    };
    assert_eq!(labels(&s), ["BROWSE", "PDF MANUAL", "FIRST", "LAST"], "a game with a manual offers it");
    let press = |s: &mut Stage, k: Key| s.viewer.as_mut().unwrap().press(k);
    assert_eq!(press(&mut s, Key::Up), Some(true));
    assert_eq!(labels(&s), ["BROWSE", "GAME MEDIA", "FIRST", "LAST", "ZOOM"]);
    let Paint::Page(p) = placed(&s.frame(), "item").paint.clone() else { panic!("the manual's page") };
    assert_eq!((p.source.as_str(), p.page), ("/m/manual.pdf", 0));
    assert!(s.frame().elements.iter().all(|e| e.name != "left"), "no page words before the painter has counted the pages");
    assert!(s.manual_pages(3));
    let words = |s: &Stage| match &placed(&s.frame(), "left").paint {
        Paint::Text(t) => t.text.clone(),
        _ => panic!(),
    };
    assert_eq!(words(&s), "PAGE 1 OF 3");
    assert_eq!(press(&mut s, Key::Left), Some(false), "the first page stays");
    assert_eq!(press(&mut s, Key::Last), Some(true));
    assert_eq!(press(&mut s, Key::Right), Some(false), "the last page stays");
    assert_eq!(words(&s), "PAGE 3 OF 3");
    assert_eq!(press(&mut s, Key::First), Some(true));
    // The page's size in points, as the painter reads it: 600 x 800, fitted to the screen below the strip.
    let strip = Viewer::strip(H);
    let id = placed(&s.frame(), "item").id;
    assert!(s.measured(id, 600.0, 800.0));
    let page = |s: &Stage| {
        let f = s.frame();
        let p = placed(&f, "item");
        (p.x, p.y, p.w, p.h)
    };
    let fit = (H - strip) / 800.0;
    let (x, y, w, h) = page(&s);
    assert!(close(w, 600.0 * fit) && close(h, H - strip) && close(y, strip) && close(x, (W - w) / 2.0), "{:?}", page(&s));
    assert_eq!(press(&mut s, Key::ZoomIn), Some(true));
    assert_eq!(labels(&s), ["PAN", "RESET", "ZOOM"]);
    let (x1, y1, w1, h1) = page(&s);
    assert!(close(h1, (fit + 0.5 * H / 800.0) * 800.0) && close(x1 + w1 / 2.0, W / 2.0) && close(y1 + h1 / 2.0, strip + (H - strip) / 2.0), "a step is half the screen's height more, centred");
    assert_eq!(press(&mut s, Key::Right), Some(true), "zoomed, right pans");
    assert!(close(page(&s).0, x1 - W / 8.0), "an eighth of the screen's width a step");
    assert_eq!(press(&mut s, Key::Down), Some(true), "zoomed, down pans");
    assert!(close(page(&s).1, y1 - W / 8.0), "down moves the page up by the same step");
    assert_eq!(press(&mut s, Key::ZoomIn), Some(true));
    assert_eq!(press(&mut s, Key::ZoomOut), Some(true));
    assert!(close(page(&s).3, h1), "the left shoulder steps the zoom back by one step");
    assert_eq!(words(&s), "PAGE 1 OF 3", "panning turns no page");
    assert_eq!(press(&mut s, Key::Last), Some(true), "a trigger resets the view");
    assert_eq!(page(&s), (x, y, w, h));
    assert_eq!(press(&mut s, Key::ZoomIn), Some(true));
    assert_eq!(press(&mut s, Key::ZoomOut), Some(true));
    assert_eq!(press(&mut s, Key::Right), Some(true), "zoomed out entirely, right turns the page again");
    assert_eq!(words(&s), "PAGE 2 OF 3");
    assert_eq!(press(&mut s, Key::Down), Some(true), "down returns to the media");
    assert_eq!(labels(&s), ["BROWSE", "PDF MANUAL", "FIRST", "LAST"]);
    assert!(matches!(placed(&s.frame(), "item").paint, Paint::Picture(_)));
}

// The player's viewer settings (USERGUIDE.md, "Media viewer settings"), as the captures of 2026-11-09 show each (EmuSen_Qt.md §4.14).
#[test]
fn the_media_viewer_follows_the_player_s_viewer_settings() {
    let t = Synth::new();
    let mut s = stage(&t, "", "<helpsystem name=\"h\"/>", "gamelist", 0);
    let mut data = s.current.data.clone();
    for kind in ["video", "screenshot"] {
        data.systems[1].games[0].media.insert(kind.to_owned(), format!("/m/{kind}"));
    }
    let open = |s: &mut Stage, data: &Data, settings: ViewerSettings| {
        let mut d = data.clone();
        d.viewer = settings;
        s.restart(d, "gamelist", 0);
        assert!(s.open_viewer());
    };
    let strip = Viewer::strip(H);
    let base = ViewerSettings::default();
    open(&mut s, &data, ViewerSettings { help: HelpPlace::Bottom, ..base });
    let f = s.frame();
    assert!(close(placed(&f, "strip").y, H - strip) && close(placed(&f, "video").y, 0.0) && close(placed(&f, "video").h, H - strip), "the strip at the bottom");
    open(&mut s, &data, ViewerSettings { help: HelpPlace::Off, ..base });
    let f = s.frame();
    assert!(f.elements.iter().all(|e| e.name != "strip" && e.name != "help") && close(placed(&f, "video").h, H), "no help, the whole screen");
    open(&mut s, &data, ViewerSettings { show_types: true, ..base });
    let left = |s: &Stage| match &placed(&s.frame(), "left").paint {
        Paint::Text(t) => t.text.clone(),
        _ => panic!(),
    };
    assert_eq!(left(&s), "VIDEO");
    s.viewer.as_mut().unwrap().press(Key::Right);
    assert_eq!(left(&s), "SCREENSHOT");
    let video = placed(&s.frame(), "video").clone();
    assert_eq!(video.opacity, 0.0, "the video runs on, unseen, while a picture is shown");
    open(&mut s, &data, ViewerSettings { keep_video_running: false, stretch_videos: true, audio: false, blur: true, ..base });
    let f = s.frame();
    let Paint::Clip(c) = &placed(&f, "video").paint else { panic!() };
    assert!(c.stretch && !c.audio && close(c.blur, W / 512.0), "the blur's sigma is a 512th of the width");
    s.viewer.as_mut().unwrap().press(Key::Right);
    assert!(s.frame().elements.iter().all(|e| e.name != "video"), "not kept running: the video stops on a picture");
}

// ES-DE's gameselectors draw from every game of the system, those its list's level does not show among them (EmuSen_Qt.md §4.11).
#[test]
fn a_gameselector_picks_from_the_games_inside_folders_too() {
    let t = Synth::new();
    let random = "<gameselector name=\"pick\"><gameCount>8</gameCount></gameselector>";
    let inside = |d: &mut Data| {
        for name in ["Quartz Spire", "Rune Gate"] {
            let mut g = Game::new(name, &format!("{name}.nes"));
            g.media.insert("screenshot".into(), format!("/shots/{name}.png"));
            d.systems[1].unlisted.push(g);
        }
        d.systems[1].games[1].folder = true;
    };
    let names = shown(&selected(&t, random, 7, inside).frame());
    assert_eq!(names.len(), 7, "five games and the two inside, the folder never: {names:?}");
    assert!(names.contains(&"Rune Gate".to_owned()) && names.contains(&"Quartz Spire".to_owned()), "{names:?}");
    assert!(!names.contains(&"Brass Lantern".to_owned()), "a folder is not picked");
}

#[test]
fn a_vertical_container_of_fixed_height_shows_whole_rows_unless_the_theme_turns_the_snap_off() {
    let t = Synth::new();
    let text = |name: &str, extra: &str| {
        format!("<text name=\"{name}\"><text>WORDS</text><pos>0 0</pos><size>0.4 0.2</size><container>true</container>{extra}</text>")
    };
    let gamelist = text("snapped", "")
        + &text("off", "<containerVerticalSnap>false</containerVerticalSnap>")
        + &text("across", "<containerType>horizontal</containerType>")
        + "<text name=\"grows\"><text>WORDS</text><pos>0 0.5</pos><size>0.4 0</size><container>true</container></text>";
    let f = stage(&t, "", &gamelist, "gamelist", 0).frame();
    let snap = |name: &str| match &placed(&f, name).paint {
        Paint::Text(x) => x.snap_rows,
        _ => panic!("{name} is a text"),
    };
    assert!(snap("snapped"), "THEMES.md: containerVerticalSnap is true by default");
    assert!(!snap("off"));
    assert!(!snap("across"), "a horizontal container has no rows to snap");
    assert!(!snap("grows"), "a box that grows to its words has no window to cut");
}
