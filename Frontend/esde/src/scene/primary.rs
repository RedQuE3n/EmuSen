//! The primary elements' layouts at a time, translated from the C# PrimaryElements and GridElements and LunaP's
//! ImageCarousel and TextRowList (EmuSen_BigPicture.md §13.2, §16, §36.3, §39.2; EmuSen_Qt.md §4.5.5).

use crate::loader::{Color, Element, SystemKind};

use super::data::Data;
use super::frame::*;
use super::grid::{pixels, GridGeometry};
use super::paint::{color, existing, face_of, h_align, media, tint, v_align};
use super::view::GridFocus;
use super::words;

fn systems_view(view: &str) -> bool {
    view == "system"
}

/// The rows a primary element lists: each system with its own theme's artwork, or the selected system's games.
pub fn rows(e: &Element, view: &str, data: &Data) -> Vec<Row> {
    let n = if systems_view(view) { data.systems.len() } else { data.system().games.len() };
    (0..n).filter_map(|i| row(e, view, data, i)).collect()
}

/// One row of a primary element's list. A collection's name takes its own letter case in the system view, a custom
/// collection listed as a folder in the gamelist view; a game in a collection then gains its system's suffix, cased on its
/// own (ES-DE 3.4.1 `SystemView::populate`'s letterCaseFunc, `GamelistBase::populateList`).
pub fn row(e: &Element, view: &str, data: &Data, i: usize) -> Option<Row> {
    if systems_view(view) {
        let s = data.systems.get(i)?;
        let case = match s.system.kind {
            SystemKind::CustomCollection => e.string("letterCaseCustomCollections"),
            SystemKind::AutoCollection => e.string("letterCaseAutoCollections"),
            SystemKind::Regular => e.string("letterCase"),
        };
        let own = s.theme.system_view.primary();
        let pick = own.filter(|o| o.kind() == e.kind()).or(if e.kind() == "grid" { Some(e) } else { own });
        let image = match e.kind() {
            "textlist" => None,
            _ => pick.and_then(|o| existing(o.path("staticImage")).or_else(|| existing(o.path("defaultImage")))),
        };
        let text = match e.kind() {
            "textlist" => s.system.full_name.clone(),
            _ => pick.and_then(|o| o.string("text")).map(str::to_owned).unwrap_or_else(|| s.system.full_name.clone()),
        };
        return Some(Row { text: words::cased(&text, case), image, marker: Marker::None, folder: false });
    }
    let system = data.system();
    let g = system.games.get(i)?;
    let case = if system.system.kind == SystemKind::CustomCollection && g.folder {
        e.string("letterCaseCustomCollections")
    } else {
        e.string("letterCase")
    };
    let named = |name: &str| {
        let mut text = words::cased(name, case);
        if system.is_collection() && !g.folder && e.bool("systemNameSuffix") != Some(false) {
            let source = g.source.as_ref().unwrap_or(&system.system);
            text.push_str(&words::system_suffix(&source.name, e.string("letterCaseSystemNameSuffix")));
        }
        text
    };
    let fallback = existing(e.path("defaultImage"));
    // A folder's picture falls back to defaultFolderImage, then defaultImage (ES-DE 3.4.1 `GamelistBase::populateList`).
    let image = || media(e, "media", Some(g)).or(if g.folder { existing(e.path("defaultFolderImage")).or_else(|| fallback.clone()) } else { fallback.clone() });
    Some(match e.kind() {
        "textlist" => {
            let ascii = e.string("collectionIndicators") == Some("ascii");
            let marker = if g.in_collection && !ascii {
                Marker::Tick
            } else if e.string("indicators") == Some("none") || g.is_collection {
                Marker::None
            } else if g.folder && g.folder_link.is_some() {
                Marker::FolderLink
            } else if g.folder {
                Marker::Folder
            } else if g.favorite && system.stars {
                Marker::Star
            } else {
                Marker::None
            };
            let name = if g.in_collection && ascii { format!("! {}", g.name) } else { g.name.clone() };
            Row { text: named(&name), image: None, marker, folder: g.folder }
        }
        _ => Row { text: named(&g.name), image: image(), marker: Marker::None, folder: g.folder },
    })
}

/// The selected item's tint when imageSelectedColor or imageSelectedColorEnd is set: the end the selected colour unless
/// given, the colour `start` when only the end is, horizontal unless imageSelectedGradientType says vertical, whatever
/// imageGradientType says (ES-DE 3.4.1 `CarouselComponent::applyTheme` and `render`, `GridComponent::applyTheme` and
/// `render`; EmuSen_Qt.md §4.5.7).
fn selected_tint(e: &Element, start: Color) -> Option<Tint> {
    // The catalogue's sameAs fills both from imageColor, so what the theme set is read from `explicit`.
    let set = |p: &str| e.explicit.contains_key(p).then(|| e.color(p)).flatten();
    let (color, end) = (set("imageSelectedColor"), set("imageSelectedColorEnd"));
    if color.is_none() && end.is_none() {
        return None;
    }
    let color = color.unwrap_or(start);
    Some(Tint { color, end: end.unwrap_or(color), vertical: e.string("imageSelectedGradientType") == Some("vertical") })
}

/// fontSize's default for an item's text, a fraction of the screen's height.
pub(super) fn item_font(e: &Element) -> f64 {
    e.float("fontSize").unwrap_or(if e.kind() == "carousel" { 0.085 } else { 0.045 }) as f64
}

// ES-DE 3.4.1 `CarouselComponent::applyTheme` and `GridComponent::applyTheme`: the selected colours fall back to the
// item's own, and the corner radius is clamped to half the screen's width.
fn item_text(e: &Element, data: &Data) -> ItemText {
    let relative = e.float("textRelativeScale").unwrap_or(1.0) as f64;
    let (text, background) = (color(e, "textColor", 0x0000_00FF), color(e, "textBackgroundColor", 0));
    ItemText {
        font: face_of(e, data),
        px: item_font(e) * relative * data.height,
        line_spacing: e.float("lineSpacing").unwrap_or(1.5) as f64,
        color: text,
        background,
        selected_color: e.color("textSelectedColor").unwrap_or(text),
        selected_background: e.color("textSelectedBackgroundColor").unwrap_or(background),
        radius: e.float("textBackgroundCornerRadius").unwrap_or(0.0).clamp(0.0, 0.5) as f64 * data.width,
        one_line: e.bool("textHorizontalScrolling").unwrap_or(false),
        marquee: None,
    }
}

fn fit_of(e: &Element) -> Fit {
    match e.string("imageFit") {
        Some("fill") => Fit::Fill,
        Some("cover") => Fit::Cover,
        _ => Fit::Contain,
    }
}

/// A carousel at a position in items (fractional while it moves, not wrapped) in its box of `bw` × `bh` pixels, `cursor` its selected row.
/// `falling`: the carousel moves toward lower positions, which decides the item ES-DE stacks as its centre while it moves.
#[allow(clippy::too_many_arguments)]
pub fn carousel(e: &Element, data: &Data, count: usize, cursor: usize, position: f64, falling: bool, bw: f64, bh: f64) -> Carousel {
    let (sw, sh) = (data.width, data.height);
    let kind = e.string("type").unwrap_or("horizontal");
    let wheel = kind.ends_with("Wheel");
    let across = !kind.starts_with("vertical");
    let max_items = e.float("maxItemCount").unwrap_or(3.0).max(0.01) as f64;
    let item_scale = e.float("itemScale").unwrap_or(1.2) as f64;
    let unfocused = (e.float("unfocusedItemOpacity").unwrap_or(0.5) as f64).clamp(0.0, 1.0);
    let unfocused_sat = (e.float("unfocusedItemSaturation").unwrap_or(1.0) as f64).clamp(0.0, 1.0);
    let unfocused_dim = (e.float("unfocusedItemDimming").unwrap_or(1.0) as f64).clamp(0.0, 1.0);
    let saturation = (e.float("imageSaturation").unwrap_or(1.0) as f64).clamp(0.0, 1.0);
    let base_tint = tint(e, "imageColor", "imageColorEnd", "imageGradientType");
    // ES-DE's carousel takes the selected colour's default before it reads imageColor, so white.
    let selected_tint = selected_tint(e, Tint::WHITE.color).unwrap_or(base_tint);
    let item_v = v_align(e.string("itemVerticalAlignment"), VAlign::Center);
    let item_h = if kind == "horizontalWheel" { HAlign::Center } else { h_align(e.string("itemHorizontalAlignment").or(Some("center"))) };
    let item = pixels(e.pair("itemSize"), sw, sh).unwrap_or((0.0, 0.0));
    let offset = (e.float("horizontalOffset").unwrap_or(0.0) as f64 * bw, e.float("verticalOffset").unwrap_or(0.0) as f64 * bh);
    let margins = e.pair("selectedItemMargins").filter(|_| !wheel).map(|m| {
        let axis = if kind == "vertical" { sh } else { sw };
        (m.x as f64 * axis, m.y as f64 * axis)
    });
    let reflects = !wheel && across && e.bool("reflections") == Some(true);
    // The row types' own placement, ES-DE 3.4.1's `CarouselComponent::applyTheme` and `render`: the selected item's offset
    // (both axes a share of the screen's width across, its height down), each item turned about its own centre, and a
    // per-item diagonal shift a share of the screen's height across, its width down.
    let row_type = !wheel;
    let chosen_offset = e.pair("selectedItemOffset").filter(|_| row_type).map_or((0.0, 0.0), |p| {
        let axis = if across { sw } else { sh };
        (p.x.clamp(-1.0, 1.0) as f64 * axis, p.y.clamp(-1.0, 1.0) as f64 * axis)
    });
    let axis_rotation = if row_type { e.float("itemAxisRotation").unwrap_or(0.0) as f64 } else { 0.0 };
    let diagonal = if row_type { e.float("itemDiagonalOffset").unwrap_or(0.0).clamp(-0.5, 0.5) as f64 * if across { sh } else { sw } } else { 0.0 };
    let stacking = if row_type { e.string("itemStacking").unwrap_or("centered") } else { "centered" };
    let (length, cross) = if across { (bw, bh) } else { (bh, bw) };
    let spacing = length / max_items;
    let w = if item.0 > 0.0 {
        item.0
    } else if across {
        spacing
    } else {
        cross
    };
    let h = if item.1 > 0.0 {
        item.1
    } else if across {
        cross
    } else {
        spacing
    };
    let hub = {
        let (hw, hh) = (if item.0 > 0.0 { item.0 } else { bw }, if item.1 > 0.0 { item.1 } else { bh });
        let x = if across {
            (bw - hw) / 2.0
        } else {
            match h_align(e.string("wheelHorizontalAlignment").or(Some("center"))) {
                HAlign::Left => 0.0,
                HAlign::Right => bw - hw,
                HAlign::Center => (bw - hw) / 2.0,
            }
        };
        let y = if !across {
            (bh - hh) / 2.0
        } else {
            match v_align(e.string("wheelVerticalAlignment"), VAlign::Center) {
                VAlign::Top => 0.0,
                VAlign::Bottom => bh - hh,
                VAlign::Center => (bh - hh) / 2.0,
            }
        };
        (x + offset.0, y + offset.1, hw, hh)
    };
    // The selected item's margins as the row moves: full beyond one item from the selection, in proportion within it (§39.2).
    let shift = |d: f64| match margins {
        Some((before, _)) if d < 0.0 => -before * (-d).min(1.0),
        Some((_, after)) => after * d.min(1.0),
        None => 0.0,
    };
    let item_rect = |d: f64| -> (f64, f64, f64, f64) {
        if wheel {
            return hub;
        }
        let centre = length / 2.0 + d * spacing + shift(d);
        if across {
            let unit = if reflects { 2.0 * h } else { h };
            let y = match item_v {
                VAlign::Top => 0.0,
                VAlign::Bottom => cross - unit,
                VAlign::Center => (cross - unit) / 2.0,
            };
            (centre - w / 2.0 + offset.0, y + offset.1, w, h)
        } else {
            let x = match item_h {
                HAlign::Left => 0.0,
                HAlign::Right => cross - w,
                HAlign::Center => (cross - w) / 2.0,
            };
            (x + offset.0, centre - h / 2.0 + offset.1, w, h)
        }
    };
    // The point the selected item grows from: a horizontal wheel's item centre, or a row item's edge its cross-axis alignment names.
    let growth = |b: (f64, f64, f64, f64)| -> (f64, f64) {
        let (cx, cy) = (b.0 + b.2 / 2.0, b.1 + b.3 / 2.0);
        if wheel && across {
            return (cx, cy);
        }
        if across {
            return (
                cx,
                match item_v {
                    VAlign::Top => b.1,
                    VAlign::Bottom => b.1 + b.3,
                    VAlign::Center => cy,
                },
            );
        }
        (
            match item_h {
                HAlign::Left => b.0,
                HAlign::Right => b.0 + b.2,
                HAlign::Center => cx,
            },
            cy,
        )
    };
    let rotation_step = e.float("itemRotation").unwrap_or(7.5) as f64;
    let origin = e.pair("itemRotationOrigin").map(|p| (p.x as f64, p.y as f64)).unwrap_or((-3.0, 0.5));
    let upright = e.bool("itemAxisHorizontal") == Some(true);
    let wheel_centre = {
        let to = if !across {
            ((origin.0 - 0.5) * hub.2, (origin.1 - 0.5) * hub.3)
        } else {
            ((origin.1 - 0.5) * hub.3, (0.5 - origin.0) * hub.2)
        };
        (hub.0 + hub.2 / 2.0 + to.0, hub.1 + hub.3 / 2.0 + to.1)
    };
    // A vertical wheel aligns its items to the selected item's grown box, so a left or right alignment moves every item by half the growth.
    let align_shift = if wheel && !across {
        let half = (item_scale - 1.0) * hub.2 / 2.0;
        match item_h {
            HAlign::Left => -half,
            HAlign::Right => half,
            HAlign::Center => 0.0,
        }
    } else {
        0.0
    };
    let mut slots = vec![];
    if count > 0 {
        let base = (position + 1e-9).floor();
        let raw = position - base;
        let fraction = if raw < 1e-6 { 0.0 } else { raw };
        let nearest = if fraction < 0.5 { 0 } else { 1 };
        let reach = (max_items.max(1.0) / 2.0).ceil() as i64 + 1;
        let (before, after) = if wheel {
            (e.uint("itemsBeforeCenter").unwrap_or(8) as i64, e.uint("itemsAfterCenter").unwrap_or(8) as i64)
        } else {
            (reach, reach)
        };
        let wraps = count > 1;
        let last = after + i64::from(fraction > 0.0);
        for k in -before..=last {
            let index = base as i64 + k;
            let row = if wraps {
                index.rem_euclid(count as i64)
            } else if index < 0 || index >= count as i64 {
                continue;
            } else {
                index
            } as usize;
            let d = k as f64 - fraction;
            let focus = (1.0 - d.abs()).max(0.0);
            let selected = k == nearest;
            let dim = if selected { 1.0 } else { unfocused_dim };
            let b = item_rect(d);
            let g = growth(b);
            let (mut dx, mut dy, mut rotation) = (align_shift, 0.0, 0.0);
            if wheel {
                let angle = d * rotation_step;
                if upright {
                    let arm = if !across { (-origin.0 * hub.2, 0.0) } else { (0.0, origin.0 * hub.2) };
                    let (s, c) = angle.to_radians().sin_cos();
                    dx += arm.0 * c - arm.1 * s - arm.0;
                    dy += arm.0 * s + arm.1 * c - arm.1;
                } else {
                    rotation = angle;
                }
            }
            let mut turn_about = (wheel_centre.0 + align_shift, wheel_centre.1);
            if row_type {
                dx += chosen_offset.0 * focus;
                dy += chosen_offset.1 * focus;
                if axis_rotation != 0.0 {
                    rotation = axis_rotation;
                    turn_about = (b.0 + dx + b.2 / 2.0, b.1 + dy + b.3 / 2.0);
                }
                // The shift is the item's own position, so it turns with the item: it moves the box, not the centre.
                if across {
                    dy -= diagonal * d;
                } else {
                    dx -= diagonal * d;
                }
            }
            let opacity = unfocused + (1.0 - unfocused) * focus;
            slots.push((
                (k as f64 - fraction).abs(),
                k,
                Slot {
                    row,
                    chosen: row == cursor && d.abs() < 1.0,
                    x: b.0 + dx,
                    y: b.1 + dy,
                    w: b.2,
                    h: b.3,
                    scale: 1.0 + (item_scale - 1.0) * focus,
                    scale_origin: (g.0 + dx, g.1 + dy),
                    rotation,
                    rotation_origin: turn_about,
                    opacity,
                    dim,
                    saturation: saturation * if selected { 1.0 } else { unfocused_sat },
                    // The painter multiplies by dim once, so the tint stays the theme's.
                    tint: if selected { selected_tint } else { base_tint },
                    reflection: reflects.then(|| {
                        (
                            (e.float("reflectionsOpacity").unwrap_or(0.5) as f64).clamp(0.0, 1.0) * opacity,
                            e.float("reflectionsFalloff").unwrap_or(1.0) as f64,
                        )
                    }),
                },
            ));
        }
        // itemStacking: the row in order, or reversed, the centre item last when raised; the centre is the item at the
        // position's floor while it falls and its ceiling otherwise (`render`, belowCenter and mPositiveDirection).
        let centre = if fraction > 0.0 && !falling { 1 } else { 0 };
        match stacking {
            "ascending" | "ascendingRaised" => slots.sort_by_key(|s| s.1),
            "descending" | "descendingRaised" => slots.sort_by_key(|s| std::cmp::Reverse(s.1)),
            _ => slots.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)),
        }
        if stacking.ends_with("Raised") {
            if let Some(at) = slots.iter().position(|s| s.1 == centre) {
                let raised = slots.remove(at);
                slots.push(raised);
            }
        }
    }
    let fill = tint(e, "color", "colorEnd", "gradientType");
    let background = e.color("color").filter(|_| fill.color.a() > 0 || fill.end.a() > 0).map(|_| fill);
    let image_align = (if across && !wheel { HAlign::Center } else { item_h }, if !across && !wheel { VAlign::Center } else { item_v });
    let text = item_text(e, data);
    Carousel {
        slots: slots.into_iter().map(|(_, _, s)| s).collect(),
        background,
        fit: fit_of(e),
        image_align,
        crop_pos: e.pair("imageCropPos").map_or((0.5, 0.5), |p| (p.x.clamp(0.0, 1.0) as f64, p.y.clamp(0.0, 1.0) as f64)),
        nearest: e.string("imageInterpolation") == Some("nearest"),
        image_radius: e.float("imageCornerRadius").unwrap_or(0.0).clamp(0.0, 0.5) as f64 * sw,
        brightness: e.float("imageBrightness").unwrap_or(0.0).clamp(-2.0, 2.0) as f64,
        text,
    }
}

/// A text list with its selection, in a box `bw` × `bh` pixels; the selection kept at the middle row once the list is longer than its window.
pub fn textlist(e: &Element, data: &Data, count: usize, selected: usize, bw: f64, bh: f64) -> TextList {
    let (sw, sh) = (data.width, data.height);
    let font = e.float("fontSize").unwrap_or(0.045) as f64;
    let px = font * sh;
    let line_spacing = e.float("lineSpacing").unwrap_or(1.5) as f64;
    let row_h = (px * line_spacing * 100.0).round() / 100.0;
    let visible = if row_h <= 0.0 { 1 } else { (((bh + 0.01) / row_h).floor() as usize).max(1) };
    let first_row = if count <= visible { 0 } else { selected.saturating_sub((visible - 1) / 2).min(count - visible) };
    let selector_h = e.float("selectorHeight").map(|f| f as f64).unwrap_or(font * 1.5) * sh;
    let primary = color(e, "primaryColor", 0x0000_FFFF);
    let selected_color = e.color("selectedColor").unwrap_or(primary);
    let selector_color = color(e, "selectorColor", 0x3333_33FF);
    let vertical = e.string("selectorGradientType") == Some("vertical");
    let selector = Tint { color: selector_color, end: e.color("selectorColorEnd").unwrap_or(selector_color), vertical };
    let y = (selected as f64 - first_row as f64) * row_h + e.float("selectorVerticalOffset").unwrap_or(0.0) as f64 * sh;
    let back = color(e, "selectedBackgroundColor", 0);
    let back_secondary = e.color("selectedSecondaryBackgroundColor").unwrap_or(back);
    let m = e.pair("selectedBackgroundMargins").unwrap_or_default();
    // The bar's width and shift are shares of the screen's width (ES-DE 3.4.1 `TextListComponent::applyTheme`); a picture
    // takes the bar's place, tinted by the bar's colours across, its width at 0 following its aspect.
    let selector_image = existing(e.path("selectorImagePath"));
    let shift = e.float("selectorHorizontalOffset").unwrap_or(0.0) as f64 * sw;
    let selector_w = e.float("selectorWidth").map_or(bw, |f| f as f64 * sw);
    let image_tint = Tint { vertical: false, ..selector };
    TextList {
        first_row,
        row_h,
        band_h: selector_h,
        content_y: first_row as f64 * row_h,
        selector: ((selector_image.is_some() || selector.color.a() > 0 || selector.end.a() > 0) && selected < count)
            .then_some(Bar { x: shift, y, w: selector_w, h: selector_h, tint: if selector_image.is_some() { image_tint } else { selector } }),
        selector_image,
        selector_tile: e.bool("selectorImageTile") == Some(true),
        selected_secondary_background: back_secondary,
        selected_background_shift: shift,
        selected_background: (back.a() > 0 || back_secondary.a() > 0).then(|| {
            (
                back,
                e.float("selectedBackgroundCornerRadius").unwrap_or(0.0) as f64 * sw,
                (m.x.max(0.0) as f64 * sw, m.y.max(0.0) as f64 * sw),
            )
        }),
        primary_color: primary,
        secondary_color: color(e, "secondaryColor", 0x00FF_00FF),
        selected_color,
        selected_secondary_color: e.color("selectedSecondaryColor").unwrap_or(selected_color),
        font: face_of(e, data),
        px,
        line_spacing,
        margin: e.float("horizontalMargin").unwrap_or(0.0) as f64 * sw,
        h_align: h_align(e.string("horizontalAlignment")),
        marker_w: px * 1.5,
        marquee: None,
    }
}

/// A grid's layout and styles at its focus.
pub fn grid(e: &Element, g: &GridGeometry, focus: GridFocus, data: &Data) -> Grid {
    let sw = data.width;
    let image_tint = tint(e, "imageColor", "imageColorEnd", "imageGradientType");
    // Each fill only with its colour set; the selector picture takes its end only where the background is graded (ES-DE
    // 3.4.1 `GridComponent::applyTheme`; EmuSen_Qt.md §4.5.7).
    let background = e.color("backgroundColor").map(|_| tint(e, "backgroundColor", "backgroundColorEnd", "backgroundGradientType"));
    let selector = e.color("selectorColor").map(|_| tint(e, "selectorColor", "selectorColorEnd", "selectorGradientType"));
    let graded = background.is_some_and(|b| b.color != b.end);
    let selector_image_tint = selector.map_or(Tint::WHITE, |t| if graded { t } else { Tint { end: t.color, vertical: false, ..t } });
    Grid {
        columns: g.columns(),
        item: g.item,
        pitch: g.pitch(),
        margin: g.margin(),
        content_y: focus.scroll_row * g.pitch().1,
        item_scale: g.scale,
        focus_from: focus.from,
        from_level: focus.from_level,
        to_level: focus.to_level,
        progress: focus.progress,
        unfocused_opacity: e.float("unfocusedItemOpacity").unwrap_or(1.0) as f64,
        unfocused_saturation: e.float("unfocusedItemSaturation").unwrap_or(1.0) as f64,
        unfocused_dim: e.float("unfocusedItemDimming").unwrap_or(1.0) as f64,
        fit: fit_of(e),
        crop_pos: e.pair("imageCropPos").map(|p| (p.x as f64, p.y as f64)).unwrap_or((0.5, 0.5)),
        nearest: e.string("imageInterpolation") == Some("nearest"),
        image_scale: e.float("imageRelativeScale").unwrap_or(1.0) as f64,
        image_radius: e.float("imageCornerRadius").unwrap_or(0.0) as f64 * sw,
        tint: image_tint,
        selected_tint: selected_tint(e, image_tint.color),
        saturation: e.float("imageSaturation").unwrap_or(1.0) as f64,
        brightness: e.float("imageBrightness").unwrap_or(0.0).clamp(-2.0, 2.0) as f64,
        background,
        background_image: existing(e.path("backgroundImage")),
        background_scale: e.float("backgroundRelativeScale").unwrap_or(1.0) as f64,
        background_radius: e.float("backgroundCornerRadius").unwrap_or(0.0) as f64 * sw,
        selector,
        selector_image_tint,
        selector_image: existing(e.path("selectorImage")),
        selector_scale: e.float("selectorRelativeScale").unwrap_or(1.0) as f64,
        selector_radius: e.float("selectorCornerRadius").unwrap_or(0.0) as f64 * sw,
        selector_layer: e.string("selectorLayer").unwrap_or("top").to_owned(),
        text: item_text(e, data),
        clip_height: g.clip_height(),
    }
}
