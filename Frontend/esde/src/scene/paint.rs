//! Each secondary element's paint from its resolved properties and the data, translated from the C# ImageElements,
//! TextElements and IndicatorElements (EmuSen_BigPicture.md §13.2, §29, §39; EmuSen_Qt.md §4.5.5).

use crate::loader::{Color, Element, Pair, ThemePath};
use crate::marks::MarkId;

use super::data::{Counts, Data, Game, System};
use super::frame::*;
use super::help;
use super::motion::TextScroll;

/// What the frame is drawn from besides the element.
pub(super) struct Ctx<'a> {
    pub view: &'a str,
    pub data: &'a Data,
    pub system: &'a System,
    pub game: Option<&'a Game>,
    pub w: f64,
    pub h: f64,
}

/// How a measurement the painter reports sizes the box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Sizing {
    Fixed,
    /// A picture at its own size, or contained in a largest box.
    Natural {
        max: Option<(f64, f64)>,
    },
    /// A picture with one axis given, the other by its own aspect.
    Aspect {
        w: f64,
        h: f64,
    },
    /// Text or a row of indicators at its own extent on the axes the theme left at 0.
    Content {
        auto_w: bool,
        auto_h: bool,
    },
}

/// An element's paint before the painter's measurement is applied: its box (pixels before origin), its sizing and scroll rule.
pub(super) struct Draft {
    pub paint: Paint,
    pub size: (f64, f64),
    pub sizing: Sizing,
    /// A self-scrolling text's rule, and whether it scrolls up rather than sideways.
    pub scroll: Option<(TextScroll, bool)>,
}

pub(super) fn existing(path: Option<&ThemePath>) -> Option<String> {
    path.filter(|p| p.exists).map(|p| p.absolute.clone())
}

/// An element's face: the theme's `fontPath`, else ES-DE's default face where the host found one.
pub(super) fn face_of(e: &Element, data: &Data) -> Face {
    match existing(e.path("fontPath")) {
        Some(file) => Face { file: Some(file), own: false },
        None => Face { file: data.default_font.clone(), own: true },
    }
}

/// A help element's face: the theme's `fontPath`, else ES-DE's help face, which rounds every advance as a theme's face
/// does (EmuSen_Qt.md §4.12.1), else its default face.
pub(super) fn help_face(e: &Element, data: &Data) -> Face {
    match existing(e.path("fontPath")) {
        Some(file) => Face { file: Some(file), own: false },
        None => es_de_help_face(data),
    }
}

/// ES-DE's own help face where the host found it, else its default face.
pub(super) fn es_de_help_face(data: &Data) -> Face {
    match &data.help_font {
        Some(file) => Face { file: Some(file.clone()), own: false },
        None => Face { file: data.default_font.clone(), own: true },
    }
}

pub(super) fn color(e: &Element, property: &str, default: u32) -> Color {
    e.color(property).unwrap_or(Color(default))
}

/// A colour and its gradient end, the end the colour itself when unset.
pub(super) fn tint(e: &Element, start: &str, end: &str, gradient: &str) -> Tint {
    let color = color(e, start, 0xFFFF_FFFF);
    Tint { color, end: e.color(end).unwrap_or(color), vertical: e.string(gradient) == Some("vertical") }
}

pub(super) fn h_align(value: Option<&str>) -> HAlign {
    match value {
        Some("center") => HAlign::Center,
        Some("right") => HAlign::Right,
        _ => HAlign::Left,
    }
}

pub(super) fn v_align(value: Option<&str>, default: VAlign) -> VAlign {
    match value {
        Some("top") => VAlign::Top,
        Some("bottom") => VAlign::Bottom,
        Some("center") => VAlign::Center,
        _ => default,
    }
}

/// The first of a binding's media types the game has.
pub(super) fn media(e: &Element, kind: &str, game: Option<&Game>) -> Option<String> {
    let game = game?;
    let binding = e.bindings.iter().find(|b| b.kind == kind)?;
    binding.names.iter().find_map(|t| game.media.get(t).cloned())
}

/// The axes of a text, a date or the clock that take their extent from the words, as ES-DE 3.4.1 sizes them: no `size`
/// sizes both, an axis given as 0 sizes that one, and two given axes size neither (es-core/src/GuiComponent.cpp
/// `GuiComponent::applyTheme` sets the extent rule; es-core/src/components/TextComponent.cpp `TextComponent::onTextChanged`
/// applies it; EmuSen_Qt.md §4.8.5).
pub fn sized_by_words(e: &Element) -> (bool, bool) {
    e.pair("size").map_or((true, true), |s| (s.x <= 0.0, s.y <= 0.0))
}

fn px(pair: Option<Pair>, w: f64, h: f64) -> Option<(f64, f64)> {
    pair.map(|p| (p.x.max(0.0) as f64 * w, p.y.max(0.0) as f64 * h))
}

// size stretches (one axis 0 takes the picture's aspect), maxSize fits inside, cropSize covers; with none, the picture's own size.
fn sized(size: Option<Pair>, max: Option<Pair>, crop: Option<Pair>, w: f64, h: f64) -> (Fit, (f64, f64), Sizing) {
    if let Some((sw, sh)) = px(size, w, h) {
        if sw > 0.0 && sh > 0.0 {
            return (Fit::Fill, (sw, sh), Sizing::Fixed);
        }
        return (Fit::Fill, (sw, sh), Sizing::Aspect { w: sw, h: sh });
    }
    if let Some(m) = px(max, w, h) {
        return (Fit::Contain, m, Sizing::Natural { max: Some(m) });
    }
    if let Some(c) = px(crop, w, h) {
        return (Fit::Cover, c, Sizing::Fixed);
    }
    (Fit::Contain, (0.0, 0.0), Sizing::Natural { max: None })
}

fn picture(e: &Element, source: String, fit: Fit, crop: &str, corner: &str, ctx: &Ctx) -> Picture {
    let crop_pos = e.pair(crop).map(|p| (p.x as f64, p.y as f64)).unwrap_or((0.5, 0.5));
    Picture {
        source,
        fit,
        crop_pos,
        tile_size: (0.0, 0.0),
        tile_align: (HAlign::Left, VAlign::Bottom),
        tint: tint(e, "color", "colorEnd", "gradientType"),
        saturation: e.float("saturation").unwrap_or(1.0) as f64,
        corner_radius: e.float(corner).unwrap_or(0.0) as f64 * ctx.w,
        nearest: false,
        brightness: 0.0,
        frame: None,
    }
}

fn image(e: &Element, ctx: &Ctx) -> Option<Draft> {
    let typed = !e.list("imageType").is_empty();
    let source = if typed { media(e, "media", ctx.game) } else { existing(e.path("path")) }.or_else(|| existing(e.path("default")))?;
    let (mut fit, size, sizing) = sized(e.pair("size"), e.pair("maxSize"), e.pair("cropSize"), ctx.w, ctx.h);
    let mut p = picture(e, source, fit, "cropPos", "cornerRadius", ctx);
    // THEMES.md: tile "can only be used when path has been set", so a game's media is never tiled.
    if e.bool("tile") == Some(true) && !typed {
        fit = Fit::Tile;
        p.fit = fit;
        p.tile_size = px(e.pair("tileSize"), ctx.w, ctx.h).unwrap_or((0.0, 0.0));
        p.tile_align = (
            if e.string("tileHorizontalAlignment") == Some("right") { HAlign::Right } else { HAlign::Left },
            if e.string("tileVerticalAlignment") == Some("top") { VAlign::Top } else { VAlign::Bottom },
        );
    }
    p.nearest = e.string("interpolation") == Some("nearest");
    Some(Draft { paint: Paint::Picture(p), size, sizing, scroll: None })
}

// A video element: its still, fitted by the image sizes, which default to the video's own unless any is set, and the
// video where one is found: `path` (which ignores the game's media), else the game's video, else `default`.
fn video(e: &Element, ctx: &Ctx) -> Option<Draft> {
    let fixed = e.explicit.contains_key("path");
    let game_video = || ctx.game.and_then(|g| g.media.get("video").cloned());
    let source = if fixed { existing(e.path("path")) } else { game_video().or_else(|| existing(e.path("default"))) };
    let still_source = if fixed { None } else { media(e, "videoFallbackImage", ctx.game) }.or_else(|| existing(e.path("defaultImage")));
    let own = ["imageSize", "imageMaxSize", "imageCropSize"].iter().any(|p| e.explicit.contains_key(*p));
    let pick = |name: &str| {
        if own {
            match e.explicit.get(name) {
                Some(crate::loader::Value::Pair(p)) => Some(*p),
                _ => None,
            }
        } else {
            e.pair(name)
        }
    };
    let (fit, size, sizing) = sized(pick("imageSize"), pick("imageMaxSize"), pick("imageCropSize"), ctx.w, ctx.h);
    let mut still = still_source.map(|s| picture(e, s, fit, "imageCropPos", "imageCornerRadius", ctx));
    let brightness = e.float("brightness").unwrap_or(0.0).clamp(-2.0, 2.0) as f64;
    let nearest = interpolation_nearest(e);
    if let Some(p) = &mut still {
        p.brightness = brightness;
        p.nearest = nearest;
    }
    let Some(source) = source else { return still.map(|p| Draft { paint: Paint::Picture(p), size, sizing, scroll: None }) };
    let (video_fit, target) = video_target(e, ctx);
    // ES-DE ignores the delay when no imageType is set, and starts at once when no still was found unless the type is none.
    let types = e.list("imageType");
    let waits = !types.is_empty() && (still.is_some() || types.iter().any(|t| t == "none"));
    let delay = if waits { e.float("delay").unwrap_or(1.5).clamp(0.0, 15.0) as f64 * 1000.0 } else { 0.0 };
    let threshold = e.pair("pillarboxThreshold").map_or((0.85, 0.90), |p| (p.x.clamp(0.2, 1.0) as f64, p.y.clamp(0.2, 1.0) as f64));
    let iterations = e.uint("iterationCount").unwrap_or(0).min(10);
    let pos = e.pair("pos").unwrap_or_default();
    let origin = e.pair("origin").unwrap_or_default();
    let v = Video {
        source,
        playing: delay == 0.0,
        delay,
        audio: e.bool("audio").unwrap_or(true),
        volume: ctx.data.viewer.volume,
        fade: e.float("fadeInTime").unwrap_or(1.0).clamp(0.0, 8.0) as f64 * 1000.0,
        black: e.string("fadeInType") != Some("transparent"),
        pillarboxes: e.bool("pillarboxes").unwrap_or(true),
        pillarbox_threshold: threshold,
        iterations,
        then_still: iterations > 0 && e.string("onIterationsDone") == Some("image") && !fixed,
        fit: video_fit,
        target,
        area: if (pos.x != 0.0 || pos.y != 0.0) && target != (0.0, 0.0) { target } else { (0.0, 0.0) },
        anchor: (pos.x as f64 * ctx.w, pos.y as f64 * ctx.h),
        origin: (origin.x as f64, origin.y as f64),
        crop_pos: e.pair("cropPos").map_or((0.5, 0.5), |p| (p.x.clamp(0.0, 1.0) as f64, p.y.clamp(0.0, 1.0) as f64)),
        tint: tint(e, "color", "colorEnd", "gradientType"),
        saturation: e.float("saturation").unwrap_or(1.0) as f64,
        brightness,
        corner_radius: e.float("videoCornerRadius").unwrap_or(0.0).clamp(0.0, 0.5) as f64 * ctx.w,
        nearest,
        still,
    };
    // With no still the box is the video's target, so the element still has a place before the video's size is known.
    let (size, sizing) = if v.still.is_some() { (size, sizing) } else { (target, Sizing::Fixed) };
    Some(Draft { paint: Paint::Video(v), size, sizing, scroll: None })
}

// THEMES.md, video interpolation: nearest unless the rotation is off the right angles.
fn interpolation_nearest(e: &Element) -> bool {
    match e.string("interpolation") {
        Some("nearest") => true,
        Some("linear") => false,
        _ => e.float("rotation").unwrap_or(0.0).rem_euclid(90.0) == 0.0,
    }
}

// The video's own sizing from size, maxSize and cropSize, in that precedence (THEMES.md, video size).
fn video_target(e: &Element, ctx: &Ctx) -> (VideoFit, (f64, f64)) {
    if let Some((w, h)) = px(e.pair("size"), ctx.w, ctx.h) {
        return match (w > 0.0, h > 0.0) {
            (true, true) => (VideoFit::Stretch, (w, h)),
            (true, false) => (VideoFit::Width, (w, 0.0)),
            (false, true) => (VideoFit::Height, (0.0, h)),
            (false, false) => (VideoFit::Stretch, (0.01 * ctx.w, 0.01 * ctx.h)),
        };
    }
    if let Some(m) = px(e.pair("maxSize"), ctx.w, ctx.h) {
        return (VideoFit::Contain, m);
    }
    if let Some(c) = px(e.pair("cropSize"), ctx.w, ctx.h) {
        return (VideoFit::Cover, c);
    }
    (VideoFit::Own, (0.0, 0.0))
}

// An animated GIF; a Lottie file is not drawn (EmuSen_Qt.md §4.5.5, the Lottie family).
fn animation(e: &Element, ctx: &Ctx) -> Option<Draft> {
    let path = existing(e.path("path")).filter(|p| p.to_ascii_lowercase().ends_with(".gif"))?;
    let (fit, size, sizing) = sized(e.pair("size"), e.pair("maxSize"), None, ctx.w, ctx.h);
    let mut p = picture(e, path, fit, "cropPos", "cornerRadius", ctx);
    p.nearest = e.string("interpolation") != Some("linear");
    Some(Draft { paint: Paint::Picture(p), size, sizing, scroll: None })
}

fn background(e: &Element, ctx: &Ctx) -> Option<Background> {
    let c = e.color("backgroundColor").filter(|c| c.a() > 0)?;
    let m = e.pair("backgroundMargins").unwrap_or_default();
    Some(Background {
        tint: Tint { color: c, end: c, vertical: false },
        radius: e.float("backgroundCornerRadius").unwrap_or(0.0) as f64 * ctx.w,
        padding: [m.x.max(0.0) as f64 * ctx.w, 0.0, m.y.max(0.0) as f64 * ctx.w, 0.0],
    })
}

// backgroundHorizontalPadding is left and right by the width, backgroundVerticalPadding top and bottom by the height.
fn padding(e: &Element, ctx: &Ctx) -> [f64; 4] {
    let (h, v) = (e.pair("backgroundHorizontalPadding").unwrap_or_default(), e.pair("backgroundVerticalPadding").unwrap_or_default());
    [h.x as f64 * ctx.w, v.x as f64 * ctx.h, h.y as f64 * ctx.w, v.y as f64 * ctx.h]
}

fn padded(e: &Element, ctx: &Ctx, tint: Tint) -> Option<Background> {
    if tint.color.a() == 0 && tint.end.a() == 0 {
        return None;
    }
    Some(Background { tint, radius: e.float("backgroundCornerRadius").unwrap_or(0.0) as f64 * ctx.w, padding: padding(e, ctx) })
}

fn element_size(e: &Element, ctx: &Ctx) -> (f64, f64) {
    px(e.pair("size"), ctx.w, ctx.h).unwrap_or((0.0, 0.0))
}

fn text(e: &Element, said: String, scroll: Option<TextScroll>, ctx: &Ctx) -> Draft {
    let size = element_size(e, ctx);
    let clock = e.kind() == "clock";
    let mut t = Text {
        text: said,
        font: face_of(e, ctx.data),
        px: e.float("fontSize").unwrap_or(if clock { 0.035 } else { 0.045 }) as f64 * ctx.h,
        color: color(e, "color", if clock { 0xFFFF_FFFF } else { 0x0000_00FF }),
        h_align: h_align(e.string("horizontalAlignment")),
        v_align: v_align(e.string("verticalAlignment"), VAlign::Center),
        line_spacing: if clock { 1.0 } else { e.float("lineSpacing").unwrap_or(1.5) as f64 },
        wrap: size.0 > 0.0 && !clock,
        // An auto-sized text grows to its words, so it is never cut short (a 0 box would elide them all).
        ellipsis: !clock && size.0 > 0.0,
        every_line: false,
        snap_rows: false,
        background: if clock {
            padded(e, ctx, tint(e, "backgroundColor", "backgroundColorEnd", "backgroundGradientType"))
        } else {
            background(e, ctx)
        },
        scroll: None,
    };
    if clock && e.color("backgroundColor").is_none() {
        t.background = None;
    }
    let vertical = e.bool("container") == Some(true) && e.string("containerType") != Some("horizontal");
    if e.bool("container") == Some(true) {
        t.ellipsis = false;
        if vertical {
            t.v_align = VAlign::Top;
            t.every_line = true;
            t.snap_rows = size.1 > 0.0 && e.bool("containerVerticalSnap") != Some(false);
        } else {
            t.wrap = false;
        }
    }
    let (auto_w, auto_h) = sized_by_words(e);
    Draft { paint: Paint::Text(t), size, sizing: Sizing::Content { auto_w, auto_h }, scroll: scroll.map(|s| (s, vertical)) }
}

fn rating(e: &Element, ctx: &Ctx) -> Option<Draft> {
    let game = ctx.game?;
    let value = game.rating.unwrap_or(0.0) as f64;
    if value <= 0.0 && e.bool("hideIfZero") == Some(true) {
        return None;
    }
    let s = e.pair("size").unwrap_or(Pair::new(0.0, 0.06));
    let (mut w, mut h) = (s.x.max(0.0) as f64 * ctx.w, s.y.max(0.0) as f64 * ctx.h);
    if w <= 0.0 {
        w = 5.0 * h;
    } else if h <= 0.0 {
        h = w / 5.0;
    }
    let icon = |p: &str| existing(e.path(p)).map(Icon::Path).unwrap_or(Icon::Mark(MarkId::Star));
    let paint = Paint::Rating(Rating {
        value: value.clamp(0.0, 1.0),
        filled: icon("filledPath"),
        unfilled: icon("unfilledPath"),
        tint: color(e, "color", 0xFFFF_FFFF),
        overlay: e.bool("overlay").unwrap_or(true),
    });
    Some(Draft { paint, size: (w, h), sizing: Sizing::Fixed, scroll: None })
}

/// THEMES.md's badge slots in its documented order.
pub const BADGE_SLOTS: [&str; 9] =
    ["collection", "folder", "favorite", "completed", "kidgame", "broken", "controller", "altemulator", "manual"];

fn has_badge(g: &Game, slot: &str) -> bool {
    match slot {
        "collection" => g.in_collection,
        "folder" => g.folder,
        "favorite" => g.favorite,
        "completed" => g.completed,
        "kidgame" => g.kid_game,
        "broken" => g.broken,
        "altemulator" => g.alt_emulator,
        "controller" => g.controller.is_some(),
        "manual" => g.manual,
        _ => false,
    }
}

fn badge_mark(slot: &str) -> MarkId {
    match slot {
        "collection" => MarkId::BadgeCollection,
        "folder" => MarkId::BadgeFolder,
        "completed" => MarkId::BadgeCompleted,
        "kidgame" => MarkId::BadgeKidGame,
        "broken" => MarkId::BadgeBroken,
        "controller" => MarkId::BadgeController,
        "altemulator" => MarkId::BadgeAltEmulator,
        "manual" => MarkId::BadgeManual,
        _ => MarkId::BadgeFavorite,
    }
}

/// The project's drawings for its own consoles' controllers, a generic pad for any other pad, the unknown shape for the rest (§29).
pub fn controller_mark(controller: &str) -> MarkId {
    match controller {
        "gamepad_nintendo_nes" => MarkId::ControllerNes,
        "gamepad_nintendo_snes" => MarkId::ControllerSnes,
        "gamepad_nintendo_64" => MarkId::ControllerN64,
        c if c.starts_with("gamepad_") => MarkId::ControllerGamepad,
        _ => MarkId::ControllerUnknown,
    }
}

/// THEMES.md's "all": the slots a theme names first, in its order, then the rest in the documented order.
pub fn badge_slots(named: &[String]) -> Vec<String> {
    if !named.iter().any(|s| s == "all") {
        return named.to_vec();
    }
    let mut out: Vec<String> = vec![];
    for s in named
        .iter()
        .map(String::as_str)
        .filter(|s| *s != "all")
        .chain(BADGE_SLOTS.iter().copied().filter(|s| !named.iter().any(|n| n == s)))
    {
        if !out.iter().any(|o| o == s) {
            out.push(s.to_owned());
        }
    }
    out
}

fn square_in(cell: (f64, f64, f64, f64)) -> (f64, f64, f64, f64) {
    let side = cell.2.min(cell.3);
    (cell.0 + (cell.2 - side) / 2.0, cell.1 + (cell.3 - side) / 2.0, side, side)
}

fn overlay_box(cell: (f64, f64, f64, f64), pos: Pair, size: f64) -> (f64, f64, f64, f64) {
    let side = square_in(cell).2 * size;
    let (cx, cy) = (cell.0 + pos.x as f64 * cell.2, cell.1 + pos.y as f64 * cell.3);
    (cx - side / 2.0, cy - side / 2.0, side, side)
}

fn badges(e: &Element, ctx: &Ctx) -> Option<Draft> {
    let game = ctx.game?;
    let s = e.pair("size").unwrap_or(Pair::new(0.15, 0.2));
    let (bw, bh) = (s.x.max(0.0) as f64 * ctx.w, s.y.max(0.0) as f64 * ctx.h);
    let m = e.pair("itemMargin").unwrap_or(Pair::new(0.01, 0.01));
    let mx = if m.x < 0.0 { m.y as f64 * ctx.h } else { m.x as f64 * ctx.w };
    let my = if m.y < 0.0 { mx } else { m.y as f64 * ctx.h };
    let shown: Vec<String> = badge_slots(e.list("slots")).into_iter().filter(|s| has_badge(game, s)).collect();
    let rows = e.string("direction") != Some("column");
    let (lines, per) = (e.uint("lines").unwrap_or(3).max(1) as usize, e.uint("itemsPerLine").unwrap_or(4).max(1) as usize);
    let count = shown.len().min(lines * per);
    let (columns, row_count) = if rows { (per, lines) } else { (lines, per) };
    let cw = (bw - (columns as f64 - 1.0) * mx) / columns as f64;
    let ch = (bh - (row_count as f64 - 1.0) * my) / row_count as f64;
    let used_columns = if rows { per.min(count) } else { count.div_ceil(per) };
    let used_w = used_columns as f64 * cw + (used_columns as f64 - 1.0).max(0.0) * mx;
    let x0 = match h_align(e.string("horizontalAlignment")) {
        HAlign::Center => (bw - used_w) / 2.0,
        HAlign::Right => bw - used_w,
        HAlign::Left => 0.0,
    };
    let icons = e.keyed("customBadgeIcon");
    let controllers = e.keyed("customControllerIcon");
    let link_icon = existing(e.path("customFolderLinkIcon"));
    let found = |map: Option<&std::collections::BTreeMap<String, ThemePath>>, key: &str| map.and_then(|m| existing(m.get(key)));
    let (tint, controller_tint, link_tint) = (
        color(e, "badgeIconColor", 0xFFFF_FFFF),
        color(e, "controllerIconColor", 0xFFFF_FFFF),
        color(e, "folderLinkIconColor", 0xFFFF_FFFF),
    );
    let controller_pos = e.pair("controllerPos").unwrap_or(Pair::new(0.5, 0.5));
    let link_pos = e.pair("folderLinkPos").unwrap_or(Pair::new(0.5, 0.5));
    let at = |b: (f64, f64, f64, f64), icon: Icon, tint: Color| Overlay { icon, x: b.0, y: b.1, w: b.2, h: b.3, tint };
    let slots = shown
        .iter()
        .take(count)
        .enumerate()
        .map(|(i, slot)| {
            let (line, within) = (i / per, i % per);
            let (col, row) = if rows { (within, line) } else { (line, within) };
            let cell = (x0 + col as f64 * (cw + mx), row as f64 * (ch + my), cw, ch);
            let icon = match found(icons, slot) {
                Some(p) => at(cell, Icon::Path(p), tint),
                None => at(square_in(cell), Icon::Mark(badge_mark(slot)), tint),
            };
            let controller = game.controller.as_deref().filter(|_| slot == "controller").map(|c| {
                let icon = found(controllers, c).map(Icon::Path).unwrap_or(Icon::Mark(controller_mark(c)));
                at(overlay_box(cell, controller_pos, e.float("controllerSize").unwrap_or(0.5) as f64), icon, controller_tint)
            });
            let link = (slot == "folder" && game.folder_link.is_some()).then(|| {
                let icon = link_icon.clone().map(Icon::Path).unwrap_or(Icon::Mark(MarkId::FolderLink));
                at(overlay_box(cell, link_pos, e.float("folderLinkSize").unwrap_or(0.5) as f64), icon, link_tint)
            });
            Badge { cell, icon, controller, link }
        })
        .collect();
    Some(Draft { paint: Paint::Badges(Badges { slots }), size: (bw, bh), sizing: Sizing::Fixed, scroll: None })
}

fn help_element(e: &Element, ctx: &Ctx) -> Option<Draft> {
    let entries = help::entries(ctx.view, e, ctx.data);
    if entries.is_empty() {
        return None;
    }
    let bg = color(e, "backgroundColor", 0);
    let paint = Paint::Help(Help {
        entries,
        font: help_face(e, ctx.data),
        px: help::font_px(e, ctx.w, ctx.h),
        label_color: color(e, "textColor", 0x7777_77FF),
        icon_color: color(e, "iconColor", 0x7777_77FF),
        background: padded(e, ctx, Tint { color: bg, end: bg, vertical: false }),
        entry_spacing: help::spacing_px(e, "entrySpacing", ctx.w),
        icon_gap: help::spacing_px(e, "iconTextSpacing", ctx.w),
        scale: help::scale(e),
        filled_glyphs: true,
    });
    Some(Draft { paint, size: (0.0, 0.0), sizing: Sizing::Content { auto_w: true, auto_h: true }, scroll: None })
}

// The list's counts as ES-DE 3.4.1 was measured to show them: games and favourites, or kept over all while filtered, and a folder once entered (§39).
fn info(e: &Element, ctx: &Ctx) -> Option<Draft> {
    let c = ctx.system.counts.unwrap_or_else(|| Counts::of(&ctx.system.games));
    let mut items = vec![];
    if c.filtered {
        items.push(InfoItem { mark: MarkId::InfoFunnel, text: Some(format!("{} / {}", c.shown, c.games)) });
    } else {
        items.push(InfoItem { mark: MarkId::InfoGamepad, text: Some(c.games.to_string()) });
        items.push(InfoItem { mark: MarkId::InfoStar, text: Some(c.favorites.to_string()) });
    }
    if c.in_folder {
        items.push(InfoItem { mark: MarkId::InfoFolder, text: None });
    }
    let h_al = h_align(e.string("horizontalAlignment"));
    if h_al == HAlign::Right {
        items.sort_by_key(|i| i.mark != MarkId::InfoFolder);
    }
    let size = element_size(e, ctx);
    let paint = Paint::Info(Info {
        items,
        font: face_of(e, ctx.data),
        px: e.float("fontSize").unwrap_or(0.045) as f64 * ctx.h,
        color: color(e, "color", 0x0000_00FF),
        background: color(e, "backgroundColor", 0),
        h_align: h_al,
        v_align: v_align(e.string("verticalAlignment"), VAlign::Center),
    });
    Some(Draft { paint, size, sizing: Sizing::Content { auto_w: size.0 <= 0.0, auto_h: size.1 <= 0.0 }, scroll: None })
}

fn battery_key(percent: u8, charging: bool) -> &'static str {
    match (charging, percent) {
        (true, _) => "battery_charging",
        (_, 90..) => "battery_full",
        (_, 60..) => "battery_high",
        (_, 30..) => "battery_medium",
        _ => "battery_low",
    }
}

fn status(e: &Element, ctx: &Ctx) -> Option<Draft> {
    let named = e.list("entries");
    let all = named.is_empty() || named.iter().any(|n| n == "all");
    let wants = |n: &str| all || named.iter().any(|x| x == n);
    let (s, on) = (ctx.data.status, ctx.data.status_shown);
    let icons = e.keyed("customIcon");
    let icon =
        |key: &str, mark: MarkId| icons.and_then(|m| existing(m.get(&format!("icon_{key}")))).map(Icon::Path).unwrap_or(Icon::Mark(mark));
    let mut items = vec![];
    for (key, mark, state, kept) in [
        ("bluetooth", MarkId::Bluetooth, s.bluetooth, on.bluetooth),
        ("wifi", MarkId::Wifi, s.wifi, on.wifi),
        ("cellular", MarkId::Cellular, s.cellular, on.cellular),
    ] {
        if wants(key) && kept && state == Some(true) {
            items.push(StatusItem { icon: Some(icon(key, mark)), text: None });
        }
    }
    if let Some(level) = s.battery_percent.filter(|_| wants("battery")) {
        if on.battery {
            let bars = (level.min(100) as f64 / 25.0).ceil() as u8;
            items.push(StatusItem { icon: Some(icon(battery_key(level, s.charging), MarkId::Battery(bars))), text: None });
        }
        if on.percentage {
            items.push(StatusItem { icon: None, text: Some(format!("{level}%")) });
        }
    }
    // With nothing to show the element still stands at its place, an empty row with no background (THEMES.md, systemstatus).
    let bg = color(e, "backgroundColor", 0);
    let paint = Paint::Status(Status {
        items,
        height: e.float("height").unwrap_or(0.035) as f64 * ctx.h,
        font: face_of(e, ctx.data),
        text_scale: e.float("textRelativeScale").unwrap_or(0.9) as f64,
        color: color(e, "color", 0xFFFF_FFFF),
        background: padded(e, ctx, Tint { color: bg, end: bg, vertical: false }),
        entry_spacing: e.float("entrySpacing").unwrap_or(0.005) as f64 * ctx.w,
    });
    Some(Draft { paint, size: (0.0, 0.0), sizing: Sizing::Content { auto_w: true, auto_h: true }, scroll: None })
}

/// A secondary element's draft; None when it draws nothing (C#'s "nothing to show").
pub(super) fn draft(e: &Element, said: Option<String>, scroll: Option<TextScroll>, ctx: &Ctx) -> Option<Draft> {
    match e.kind() {
        "image" => image(e, ctx),
        "video" => video(e, ctx),
        "animation" => animation(e, ctx),
        "text" | "datetime" | "clock" => said.map(|s| text(e, s, scroll, ctx)),
        "rating" => rating(e, ctx),
        "badges" => badges(e, ctx),
        "helpsystem" => help_element(e, ctx),
        "gamelistinfo" => info(e, ctx),
        "systemstatus" => status(e, ctx),
        _ => None,
    }
}
