// The painter's contract: Stage::frame's records, the painter's measurements, mapping.tsv and the marks (EmuSen_Qt.md §4.5.5).
mod common;

use std::collections::BTreeSet;

use common::*;
use emusen_esde::check;
use emusen_esde::loader::{catalog, Choices, Color, ThemeSystem};
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
fn mapping_lists_every_catalogue_pair_once_and_the_dimmed_help_and_sound_as_not_drawn() {
    let listed: Vec<(String, String)> = mapping::pairs().map(|(k, p, _)| (k.to_owned(), p.to_owned())).collect();
    let unique: BTreeSet<_> = listed.iter().cloned().collect();
    assert_eq!(unique.len(), listed.len(), "a pair listed twice");
    let catalogue: BTreeSet<(String, String)> =
        catalog().elements.iter().flat_map(|e| e.properties.iter().map(|p| (e.kind.clone(), p.name.clone()))).collect();
    assert_eq!(unique, catalogue);
    let dimmed: Vec<_> = mapping::pairs().filter(|(k, p, _)| *k == "helpsystem" && p.ends_with("Dimmed")).collect();
    assert_eq!(dimmed.len(), 8);
    assert!(dimmed.iter().all(|(_, _, s)| **s == Mapped::Later("help-dimmed".into())), "R-20261111-75: not drawn until checked");
    assert_eq!(mapping::status("sound", "path"), Some(&Mapped::Later("sound".into())));
    assert_eq!(mapping::status("video", "delay"), Some(&Mapped::Drawn));
    // Heard, not seen: no picture shows it, and no state stands between drawn and not drawn (R-20261111-75).
    assert_eq!(mapping::status("video", "audio"), Some(&Mapped::Later("video".into())));
    assert_eq!(mapping::status("video", "scanlines"), Some(&Mapped::Later("video".into())));
    assert_eq!(mapping::pairs().filter(|(_, _, s)| **s == Mapped::Drawn).count(), 441);
    assert!(
        mapping::is_drawn("gameselector") && !mapping::is_drawn("sound"),
        "the gameselector picks the system view's games; sound is played"
    );
}

#[test]
fn a_theme_setting_the_dimmed_help_or_a_sound_counts_them_unsupported_and_not_whole() {
    let t = Synth::new();
    t.capabilities("").theme(
        "<view name=\"system\"><helpsystem name=\"help\"><textColorDimmed>FF0000</textColorDimmed><opacityDimmed>0.5</opacityDimmed></helpsystem></view>
         <view name=\"all\"><sound name=\"systembrowse\"><path>./s.wav</path></sound></view>",
    );
    let m = check::measure(&t.root, &["nes"]);
    assert_eq!(m.properties.get("helpsystem.textColorDimmed"), Some(&"unsupported"));
    assert_eq!(m.properties.get("sound.path"), Some(&"unsupported"));
    assert_eq!(m.count("unsupported"), 3);
    assert!(m.loads() && !m.whole());
    let header: Vec<&str> = check::SUMMARY_HEADER.split('\t').collect();
    assert!(!header.contains(&"accepted"), "no state between drawn and not drawn");
    let row = check::summary_row(&m);
    let cells: Vec<&str> = row.split('\t').collect();
    assert_eq!(cells.len(), header.len());
    assert_eq!(cells[header.iter().position(|c| *c == "unsupported").unwrap()], "3");
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
    let clip = Clip { source: "/m/video".into(), stretch: false, audio: true, volume: 1.0, blur: 0.0, fade: 600.0 };
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
    // The help's spacings are ES-DE's defaults for a help element, THEMES.md's 0.00833 and 0.00416 of the width.
    let Paint::Help(help) = &placed(&f, "help").paint else { panic!("the viewer's help") };
    assert_eq!((help.entry_spacing, help.icon_gap), (f64::from(0.00833_f32 * W as f32), f64::from(0.00416_f32 * W as f32)));
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

fn help_paint(t: &Synth, help: &str, size: Option<(f64, f64)>) -> Help {
    let mut s = stage(t, "", &format!("<helpsystem name=\"h\"><entries>a,b</entries>{help}</helpsystem>"), "gamelist", 0);
    if let Some((w, h)) = size {
        (s.current.data.width, s.current.data.height) = (w, h);
    }
    let Paint::Help(h) = placed(&s.frame(), "h").paint.clone() else { panic!() };
    h
}

// ES-DE 3.4.1's help numbers from a theme, in f32 (EmuSen_Qt.md §4.12.4; es-core HelpComponent.cpp applyTheme, Font.cpp
// getFromTheme and Font::Font, Font.h getSmallFont). No capture holds the clamps; each is held here from the rule.
#[test]
fn the_help_s_size_spacings_and_scale_are_es_de_s_in_f32_with_its_clamps() {
    let t = Synth::new();
    let h = help_paint(&t, "<fontSize>0.04583333</fontSize><entrySpacing>0.015625</entrySpacing><iconTextSpacing>0.00651042</iconTextSpacing><entryRelativeScale>0.7</entryRelativeScale>", None);
    assert_eq!(h.px, f64::from(0.045_833_33_f32 * H as f32), "fontSize × height in f32");
    assert_eq!((h.entry_spacing, h.icon_gap), (f64::from(0.015_625_f32 * W as f32), f64::from(0.006_510_42_f32 * W as f32)));
    assert_eq!(h.scale, f64::from(0.7_f32));
    let h = help_paint(&t, "<fontSize>0.0001</fontSize><entrySpacing>0.06</entrySpacing><iconTextSpacing>0.05</iconTextSpacing><entryRelativeScale>0.1</entryRelativeScale>", None);
    assert_eq!((h.px, h.entry_spacing, h.icon_gap, h.scale), (3.0, f64::from(0.04_f32 * W as f32), f64::from(0.04_f32 * W as f32), f64::from(0.2_f32)), "the floors and ceilings");
    let h = help_paint(&t, "<fontSize>2</fontSize><entryRelativeScale>4</entryRelativeScale>", None);
    assert_eq!((h.px, h.scale), (1.5 * H, 3.0), "1.5 heights, a scale of 3");
    let h = help_paint(&t, "", None);
    assert_eq!((h.px, h.entry_spacing, h.icon_gap), (f64::from(0.035_f32 * W.min(H) as f32), f64::from(0.008_33_f32 * W as f32), f64::from(0.004_16_f32 * W as f32)), "the defaults");
    let h = help_paint(&t, "", Some((200.0, 320.0)));
    assert_eq!(h.px, f64::from(0.035_f32 * 200.0), "the default size is of the shorter side, here the width");
}

// The strip measured as ES-DE's grid (its width with the trailing spacing, the row's height) is placed by two roundings
// with the spacing's share of the origin, and its box is the strip seen (EmuSen_Qt.md §4.12.4; HelpComponent.cpp
// updateGrid, GuiComponent.cpp getTransform).
#[test]
fn the_help_strip_is_placed_by_es_de_s_rule_from_its_grid() {
    let t = Synth::new();
    let help = "<helpsystem name=\"h\"><entries>a,b</entries><pos>0.5 0.9375</pos><origin>0.5 1</origin><entrySpacing>0.015625</entrySpacing></helpsystem>";
    let mut s = stage(&t, "", help, "gamelist", 0);
    let id = placed(&s.frame(), "h").id;
    let es = f64::from(0.015_625_f32 * W as f32);
    s.measured(id, 101.3 + es, 33.75);
    let p = placed(&s.frame(), "h").clone();
    let x = ((0.5 * W + es * 0.5) as f32).round() + (-0.5 * (101.3 + es) as f32).round();
    let y = ((0.9375 * H) as f32).round() + (-33.75_f32).round();
    assert_eq!((p.x, p.y, p.w, p.h), (f64::from(x), f64::from(y), 101.3 + es - es, 33.75));
    let mut s = stage(&t, "", "<helpsystem name=\"h\"><entries>a,b</entries></helpsystem>", "gamelist", 0);
    let id = placed(&s.frame(), "h").id;
    s.measured(id, 80.0, 20.0);
    let p = placed(&s.frame(), "h").clone();
    assert_eq!((p.x, p.y), (f64::from((0.012 * W as f32).round()), f64::from((0.9515 * H as f32).round())), "ES-DE's default place, origin 0");
}

// Which axes of a text its words size, as ES-DE 3.4.1's GuiComponent::applyTheme and TextComponent::onTextChanged do: the
// sizes test holds a box the theme sizes to one pixel and only a worded one to its letters (R-20261111-88, -112).
#[test]
fn a_text_s_words_size_only_the_axes_its_theme_leaves_at_zero() {
    let t = Synth::new();
    t.capabilities("").theme(
        "<view name=\"system\"><text name=\"none\"><text>a</text></text><text name=\"both\"><size>0.4 0.2</size><text>a</text></text>\
         <text name=\"tall\"><size>0.4 0</size><text>a</text></text><text name=\"zero\"><size>0 0</size><text>a</text></text>\
         <datetime name=\"date\"><size>0.2 0.05</size></datetime></view>",
    );
    let theme = t.load();
    let v = theme.view("system");
    let of = |kind: &str, name: &str| paint::sized_by_words(v.find(kind, name).unwrap_or_else(|| panic!("{kind} {name}")));
    assert_eq!(of("text", "none"), (true, true));
    assert_eq!(of("text", "zero"), (true, true));
    assert_eq!(of("text", "tall"), (false, true));
    assert_eq!(of("text", "both"), (false, false));
    assert_eq!(of("datetime", "date"), (false, false));
}

fn video_of(f: &Frame) -> Video {
    match &placed(f, "v").paint {
        Paint::Video(x) => x.clone(),
        p => panic!("a video: {p:?}"),
    }
}

// A video element's stage over games that have a screenshot and no video, so `default` plays (THEMES.md, video).
fn video_stage(t: &Synth, video: &str) -> Stage {
    t.asset("shot.png").asset("clip.mkv").asset("other.mkv");
    let mut s = stage(t, "", video, "gamelist", 0);
    let shot = t.path_of("shot.png");
    for g in s.current.data.systems.iter_mut().flat_map(|sys| sys.games.iter_mut()) {
        g.media.insert("screenshot".into(), shot.clone());
    }
    s
}

#[test]
fn a_video_shows_its_still_until_its_delay_has_passed_and_wakes_the_painter_then() {
    let t = Synth::new();
    let v = "<video name=\"v\"><pos>0.5 0.5</pos><origin>0.5 0.5</origin><size>0.5 0.5</size><imageType>screenshot</imageType>\
             <default>./clip.mkv</default><delay>2</delay><fadeInTime>0.5</fadeInTime><fadeInType>transparent</fadeInType><audio>false</audio></video>";
    let mut s = video_stage(&t, v);
    let f = s.frame();
    let Paint::Video(x) = &placed(&f, "v").paint else { panic!("a video: {:?}", placed(&f, "v").paint) };
    assert!(x.source.ends_with("clip.mkv") && x.still.as_ref().is_some_and(|p| p.source.ends_with("shot.png")));
    assert!(!x.playing && x.delay == 2000.0, "the still first, for the theme's 2 s");
    assert_eq!(f.next_change_ms, Some(2000), "the painter woken when the video is due");
    assert!(!x.audio && x.fade == 500.0 && !x.black);
    assert_eq!((x.fit, x.target, x.area, x.anchor), (VideoFit::Stretch, (160.0, 100.0), (160.0, 100.0), (80.0, 50.0)));
    s.advance(ms(1999.0));
    let x = video_of(&s.frame());
    assert!(!x.playing, "not before the delay");
    s.advance(ms(2000.0));
    let f = s.frame();
    let Paint::Video(x) = &placed(&f, "v").paint else { panic!() };
    assert!(x.playing && f.next_change_ms.is_none(), "the video from the delay on");
    // A new game starts the delay again.
    s.current.step(1, ms(2500.0));
    s.advance(ms(2600.0));
    let x = video_of(&s.frame());
    assert!(!x.playing, "the still again for the next game");
}

#[test]
fn a_video_s_delay_is_ignored_without_an_image_type_or_a_still_and_path_ignores_the_game() {
    let t = Synth::new();
    // No imageType: ES-DE ignores the delay, so the video plays at once with no still.
    let s = video_stage(&t, "<video name=\"v\"><pos>0.1 0.1</pos><maxSize>0.5 0.5</maxSize><default>./clip.mkv</default><delay>3</delay></video>");
    let x = video_of(&s.frame());
    assert!(x.playing && x.delay == 0.0 && x.still.is_none() && x.fit == VideoFit::Contain && x.black && x.audio);
    assert!(close(x.pillarbox_threshold.0, 0.85) && close(x.pillarbox_threshold.1, 0.9) && x.iterations == 0 && !x.then_still);
    // A type the game lacks and no defaultImage: no still, so it starts at once too.
    let s = video_stage(&t, "<video name=\"v\"><pos>0.1 0.1</pos><size>0.5 0</size><imageType>marquee</imageType><default>./clip.mkv</default></video>");
    let x = video_of(&s.frame());
    assert!(x.playing && x.still.is_none() && x.fit == VideoFit::Width && x.target == (160.0, 0.0));
    // path: a static video, the game's media and the default ignored, the still only the defaultImage.
    let v = "<video name=\"v\"><pos>0.1 0.1</pos><cropSize>0.5 0.5</cropSize><cropPos>0 1</cropPos><path>./other.mkv</path><imageType>screenshot</imageType>\
             <default>./clip.mkv</default><iterationCount>2</iterationCount><onIterationsDone>image</onIterationsDone><videoCornerRadius>0.01</videoCornerRadius>\
             <brightness>0.5</brightness><interpolation>linear</interpolation><pillarboxes>false</pillarboxes></video>";
    let s = video_stage(&t, v);
    let x = video_of(&s.frame());
    assert!(x.source.ends_with("other.mkv") && x.still.is_none() && x.playing, "path plays its own file at once");
    assert_eq!((x.fit, x.crop_pos, x.iterations, x.then_still), (VideoFit::Cover, (0.0, 1.0), 2, false), "no still to return to under path");
    assert!(close(x.corner_radius, 3.2) && x.brightness == 0.5 && !x.nearest && !x.pillarboxes);
    // No video at all: the still alone, as a picture.
    let s = video_stage(&t, "<video name=\"v\"><pos>0.1 0.1</pos><size>0.5 0.5</size><imageType>screenshot</imageType></video>");
    assert!(matches!(placed(&s.frame(), "v").paint, Paint::Picture(_)));
}

#[test]
fn es_de_s_gradient_defaults_hold_for_the_selected_item_the_grid_s_selector_picture_and_the_help() {
    let t = Synth::new();
    // A carousel's selected end alone grades from white, which ES-DE's carousel takes before it reads imageColor, and
    // runs horizontal whatever imageGradientType says (EmuSen_Qt.md §4.5.7).
    let carousel = format!(
        "{CAROUSEL}<imageColor>FF0000</imageColor><imageColorEnd>00FF00</imageColorEnd><imageGradientType>vertical</imageGradientType><imageSelectedColorEnd>0000FF</imageSelectedColorEnd><imageBrightness>3</imageBrightness><imageCornerRadius>0.6</imageCornerRadius></carousel>"
    );
    let s = stage(&t, &carousel, "", "system", 0);
    let Layout::Carousel(c) = &s.frame().primary.unwrap().layout else { panic!() };
    let (selected, other) = (c.slots.last().unwrap(), c.slots.first().unwrap());
    assert_eq!(selected.tint, Tint { color: Color(0xFFFF_FFFF), end: Color(0x0000_FFFF), vertical: false });
    assert_eq!(other.tint, Tint { color: Color(0xFF00_00FF), end: Color(0x00FF_00FF), vertical: true });
    assert!(c.brightness == 2.0 && close(c.image_radius, 0.5 * W), "clamped as ES-DE clamps them: {} {}", c.brightness, c.image_radius);
    // The grid's selected start is its imageColor; its selector picture takes the selector's end only where the
    // background is graded; each fill needs its colour.
    t.asset("sel.png");
    let grid = |extra: &str| {
        format!("<grid name=\"g\"><pos>0 0</pos><size>1 1</size><imageColor>FF0000</imageColor><imageSelectedColorEnd>0000FF</imageSelectedColorEnd><selectorImage>./sel.png</selectorImage><selectorColor>112233</selectorColor><selectorColorEnd>445566</selectorColorEnd><selectorGradientType>vertical</selectorGradientType><backgroundColorEnd>778899</backgroundColorEnd>{extra}</grid>")
    };
    let s = stage(&t, &grid(""), "", "system", 0);
    let Layout::Grid(g) = &s.frame().primary.unwrap().layout else { panic!() };
    assert_eq!(g.selected_tint, Some(Tint { color: Color(0xFF00_00FF), end: Color(0x0000_FFFF), vertical: false }));
    assert_eq!(g.background, None, "backgroundColorEnd alone draws nothing");
    assert_eq!(g.selector, Some(Tint { color: Color(0x1122_33FF), end: Color(0x4455_66FF), vertical: true }));
    assert_eq!(g.selector_image_tint, Tint { color: Color(0x1122_33FF), end: Color(0x1122_33FF), vertical: false });
    let s = stage(&t, &grid("<backgroundColor>000000</backgroundColor>"), "", "system", 0);
    let Layout::Grid(g) = &s.frame().primary.unwrap().layout else { panic!() };
    assert_eq!(g.background, Some(Tint { color: Color(0x0000_00FF), end: Color(0x7788_99FF), vertical: false }));
    assert_eq!(g.selector_image_tint, g.selector.unwrap());
    // The help's end and gradient count only with its colour; the text list's selector grades from its default.
    let gamelist = "<helpsystem name=\"h\"><entries>a</entries><backgroundColorEnd>FF0000</backgroundColorEnd></helpsystem>\
        <helpsystem name=\"k\"><entries>a</entries><backgroundColor>00FF00</backgroundColor><backgroundColorEnd>FF0000</backgroundColorEnd><backgroundGradientType>vertical</backgroundGradientType></helpsystem>\
        <textlist name=\"l\"><selectorColorEnd>FF0000</selectorColorEnd><selectorGradientType>vertical</selectorGradientType></textlist>";
    let f = stage(&t, "", gamelist, "gamelist", 0).frame();
    let Paint::Help(h) = &placed(&f, "h").paint else { panic!() };
    assert!(h.background.is_none());
    let Paint::Help(k) = &placed(&f, "k").paint else { panic!() };
    assert_eq!(k.background.unwrap().tint, Tint { color: Color(0x00FF_00FF), end: Color(0xFF00_00FF), vertical: true });
    let Layout::TextList(l) = &f.primary.unwrap().layout else { panic!() };
    assert_eq!(l.selector.unwrap().tint, Tint { color: Color(0x3333_33FF), end: Color(0xFF00_00FF), vertical: true });
}

#[test]
fn an_image_s_flips_brightness_mipmap_and_game_override_reach_its_picture() {
    let t = Synth::new();
    t.asset("still.png");
    t.asset("over/snes/Ember Circuit.png");
    let gamelist = "<image name=\"i\"><path>./still.png</path><flipHorizontal>true</flipHorizontal><brightness>-3</brightness><mipmap>true</mipmap><gameOverridePath>./over</gameOverridePath></image>\
        <image name=\"j\"><path>./still.png</path><flipVertical>true</flipVertical><gameOverridePath>./over</gameOverridePath></image>";
    let f = stage(&t, "", gamelist, "gamelist", 4).frame();
    let Paint::Picture(i) = &placed(&f, "i").paint else { panic!() };
    assert_eq!((i.flip, i.brightness, i.mipmap), ((true, false), -2.0, true));
    assert!(i.source.ends_with("over/snes/Ember Circuit.png"), "the game's own override: {}", i.source);
    let f = stage(&t, "", gamelist, "gamelist", 0).frame();
    let Paint::Picture(j) = &placed(&f, "j").paint else { panic!() };
    assert_eq!((j.flip, j.mipmap), ((false, true), false));
    assert!(j.source.ends_with("still.png"), "no override for this game: its path");
}

#[test]
fn a_clear_selector_with_no_end_set_draws_no_bar() {
    let t = Synth::new();
    let f = stage(&t, "", "<textlist name=\"l\"><selectorColor>00000000</selectorColor></textlist>", "gamelist", 0).frame();
    let Layout::TextList(l) = &f.primary.unwrap().layout else { panic!() };
    assert_eq!(l.selector, None, "the end is the colour itself, as ES-DE's TextListComponent::applyTheme sets it");
}

// A carousel's or grid's text items: the selected item's colours, the corner radius, one line each, and the selected
// name scrolling once the painter measures it wider than its item (ES-DE 3.4.1 CarouselComponent, GridComponent).
#[test]
fn a_carousel_s_and_a_grid_s_selected_name_takes_its_colours_and_scrolls_once_measured() {
    let text = "<textColor>FFFFFF</textColor><textSelectedColor>FF0000</textSelectedColor><textBackgroundColor>000000</textBackgroundColor>\
                <textSelectedBackgroundColor>00FF00</textSelectedBackgroundColor><textBackgroundCornerRadius>0.9</textBackgroundCornerRadius>\
                <textHorizontalScrolling>true</textHorizontalScrolling><textHorizontalScrollDelay>1</textHorizontalScrollDelay><fontSize>0.1</fontSize>";
    let carousel = format!("{CAROUSEL}<imageType>none</imageType><itemSize>0.25 0.25</itemSize>{text}</carousel>");
    let grid = format!("<grid name=\"c\"><pos>0 0</pos><size>1 1</size><imageType>none</imageType><itemSize>0.25 0.25</itemSize>{text}</grid>");
    for theme in [carousel, grid] {
        let t = Synth::new();
        let mut s = stage(&t, "", &theme, "gamelist", 2);
        let f = s.frame();
        let p = f.primary.clone().unwrap();
        let (text, room) = match &p.layout {
            Layout::Carousel(c) => {
                let chosen: Vec<_> = c.slots.iter().filter(|x| x.chosen).collect();
                assert_eq!(chosen.len(), 1, "one slot is the selected item's");
                assert_eq!(chosen[0].row, 2);
                (c.text.clone(), chosen[0].w)
            }
            Layout::Grid(g) => (g.text.clone(), g.item.0),
            _ => panic!(),
        };
        assert_eq!((text.color, text.selected_color), (Color(0xFFFF_FFFF), Color(0xFF00_00FF)));
        assert_eq!((text.background, text.selected_background), (Color(0x0000_00FF), Color(0x00FF_00FF)));
        assert!(close(text.radius, 0.5 * W), "the radius is clamped to half the screen: {}", text.radius);
        assert!(text.one_line && text.marquee.is_none() && !f.moving, "no scroll before the painter measures");
        s.measured(p.id, room - 1.0, 20.0);
        assert!(text_of(&s.frame().primary.unwrap().layout).marquee.is_none(), "a name that fits does not scroll");
        assert!(s.measured(p.id, room * 3.0, 20.0));
        let f = s.frame();
        assert_eq!(text_of(&f.primary.clone().unwrap().layout).marquee.map(|m| m.x), Some(0.0));
        assert_eq!(f.next_change_ms, Some(1000), "the scroll waits its delay");
        s.advance(ms(1500.0));
        let f = s.frame();
        let m = text_of(&f.primary.unwrap().layout).marquee.unwrap();
        assert!(m.x > 0.0 && f.moving && m.repeat_at.is_some_and(|r| r > room * 3.0));
    }
}

fn text_of(l: &Layout) -> ItemText {
    match l {
        Layout::Carousel(c) => c.text.clone(),
        Layout::Grid(g) => g.text.clone(),
        _ => panic!(),
    }
}

#[test]
fn a_carousel_s_names_stay_wrapped_and_still_without_horizontal_scrolling() {
    let t = Synth::new();
    let theme = format!("{CAROUSEL}<imageType>none</imageType><fontSize>0.1</fontSize></carousel>");
    let mut s = stage(&t, "", &theme, "gamelist", 2);
    let p = s.frame().primary.unwrap();
    assert!(!text_of(&p.layout).one_line);
    s.measured(p.id, 5000.0, 20.0);
    s.advance(ms(5000.0));
    assert!(text_of(&s.frame().primary.unwrap().layout).marquee.is_none(), "off by default in a carousel");
}

// ES-DE 3.4.1 GridComponent::render clips to whole rows unless fractionalRows is set.
#[test]
fn a_grid_draws_whole_rows_only_unless_fractional_rows_is_set() {
    let grid = |fractional: bool| {
        let t = Synth::new();
        let theme = format!(
            "<grid name=\"c\"><pos>0 0</pos><size>1 1</size><itemSize>0.25 0.3</itemSize><itemSpacing>0 0</itemSpacing><itemScale>1</itemScale><fractionalRows>{fractional}</fractionalRows></grid>"
        );
        let s = stage(&t, "", &theme, "gamelist", 0);
        let Layout::Grid(g) = s.frame().primary.unwrap().layout else { panic!() };
        g.clip_height
    };
    // Rows 60 px high in a box of 200: three whole rows, 180 px, or the whole box.
    assert!(close(grid(false), 180.0), "{}", grid(false));
    assert!(close(grid(true), 200.0), "{}", grid(true));
}
