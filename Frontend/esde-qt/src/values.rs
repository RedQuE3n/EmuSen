//! The scene's records flattened to role values by name, plain Rust, so the list models in `lists.rs` only look them up
//! (EmuSen_Qt.md §4.8.4). Paths become `file:` URLs here, since QML builds no strings.

use emusen_esde::loader::Color;
use emusen_esde::scene::{Background, Badge, Fit, HAlign, Icon, Layout, Paint, Placed, Primary, Row, Scroll, Slot, Tint, VAlign};

/// One role's value, converted to a QVariant by the model.
#[derive(Clone, Debug, PartialEq)]
pub enum V {
    N(f64),
    I(i32),
    B(bool),
    S(String),
    /// A file, as a `file:` URL; empty for none.
    U(String),
    C(Color),
    None,
}

const CLEAR: Color = Color(0);

fn fit(f: Fit) -> i32 {
    match f {
        Fit::Fill => 0,
        Fit::Contain => 1,
        Fit::Cover => 2,
        Fit::Tile => 3,
    }
}

fn h(a: HAlign) -> i32 {
    match a {
        HAlign::Left => 0,
        HAlign::Center => 1,
        HAlign::Right => 2,
    }
}

fn v(a: VAlign) -> i32 {
    match a {
        VAlign::Top => 0,
        VAlign::Center => 1,
        VAlign::Bottom => 2,
    }
}

fn url(path: &Option<String>) -> V {
    V::U(path.clone().unwrap_or_default())
}

fn icon_source(i: &Icon) -> V {
    match i {
        Icon::Path(p) => V::U(p.clone()),
        Icon::Mark(_) => V::U(String::new()),
    }
}

fn icon_mark(i: &Icon) -> V {
    match i {
        Icon::Path(_) => V::S(String::new()),
        Icon::Mark(m) => V::S(m.name()),
    }
}

fn tint(t: &Tint, role: &str) -> Option<V> {
    Some(match role {
        "tint" => V::C(t.color),
        "tintEnd" => V::C(t.end),
        "gradientVertical" => V::B(t.vertical),
        _ => return None,
    })
}

fn background(b: &Option<Background>, role: &str) -> Option<V> {
    let none = Background { tint: Tint { color: CLEAR, end: CLEAR, vertical: false }, radius: 0.0, padding: [0.0; 4] };
    let b2 = b.as_ref().unwrap_or(&none);
    Some(match role {
        "hasBackground" => V::B(b.is_some()),
        "background" => V::C(b2.tint.color),
        "backgroundEnd" => V::C(b2.tint.end),
        "backgroundVertical" => V::B(b2.tint.vertical),
        "backgroundRadius" => V::N(b2.radius),
        "padLeft" => V::N(b2.padding[0]),
        "padTop" => V::N(b2.padding[1]),
        "padRight" => V::N(b2.padding[2]),
        "padBottom" => V::N(b2.padding[3]),
        _ => return None,
    })
}

fn scroll(s: &Option<Scroll>, role: &str) -> Option<V> {
    let d = Scroll::default();
    let s2 = s.as_ref().unwrap_or(&d);
    Some(match role {
        "scrolling" => V::B(s.is_some()),
        "scrollVertical" => V::B(s.is_some() && s2.repeat_at.is_none()),
        "scrollX" => V::N(s2.x),
        "repeatAt" => V::N(s2.repeat_at.unwrap_or(0.0)),
        "scrollY" => V::N(s2.y),
        "scrollOpacity" => V::N(if s.is_some() { s2.opacity } else { 1.0 }),
        _ => return None,
    })
}

/// The `paint` role: which delegate draws the element.
pub fn paint_kind(p: &Placed, primary: Option<&Primary>) -> &'static str {
    match &p.paint {
        Paint::Picture(_) => "picture",
        Paint::Text(_) => "text",
        Paint::Rating(_) => "rating",
        Paint::Badges(_) => "badges",
        Paint::Help(_) => "help",
        Paint::Info(_) => "info",
        Paint::Status(_) => "status",
        Paint::Primary { .. } => match primary.map(|p| &p.layout) {
            Some(Layout::Carousel(_)) => "carousel",
            Some(Layout::TextList(_)) => "textlist",
            Some(Layout::Grid(_)) => "grid",
            None => "none",
        },
    }
}

/// An element's role by name: the box, then its paint's fields, then its primary layout's.
pub fn element(p: &Placed, primary: Option<&Primary>, role: &str) -> V {
    match role {
        "elementId" => return V::S(p.id.to_string()),
        "paint" => return V::S(paint_kind(p, primary).to_owned()),
        "name" => return V::S(p.name.clone()),
        "boxX" => return V::N(p.x),
        "boxY" => return V::N(p.y),
        "boxWidth" => return V::N(p.w),
        "boxHeight" => return V::N(p.h),
        "turn" => return V::N(p.rotation),
        "originX" => return V::N(p.rotation_origin.0),
        "originY" => return V::N(p.rotation_origin.1),
        "alpha" => return V::N(p.opacity),
        _ => {}
    }
    let found = match &p.paint {
        Paint::Picture(q) => picture(q, role),
        Paint::Text(t) => text(t, role),
        Paint::Rating(r) => match role {
            "value" => Some(V::N(r.value)),
            "filledSource" => Some(icon_source(&r.filled)),
            "filledMark" => Some(icon_mark(&r.filled)),
            "unfilledSource" => Some(icon_source(&r.unfilled)),
            "unfilledMark" => Some(icon_mark(&r.unfilled)),
            "tint" => Some(V::C(r.tint)),
            "overlay" => Some(V::B(r.overlay)),
            _ => None,
        },
        Paint::Badges(_) => None,
        Paint::Help(x) => match role {
            "fontSource" => Some(url(&x.font)),
            "px" => Some(V::N(x.px)),
            "color" => Some(V::C(x.label_color)),
            "iconColor" => Some(V::C(x.icon_color)),
            "entrySpacing" => Some(V::N(x.entry_spacing)),
            "iconGap" => Some(V::N(x.icon_gap)),
            "entryScale" => Some(V::N(x.scale)),
            "filled" => Some(V::B(x.filled_glyphs)),
            _ => background(&x.background, role),
        },
        Paint::Info(x) => match role {
            "fontSource" => Some(url(&x.font)),
            "px" => Some(V::N(x.px)),
            "color" => Some(V::C(x.color)),
            "background" => Some(V::C(x.background)),
            "hAlign" => Some(V::I(h(x.h_align))),
            "vAlign" => Some(V::I(v(x.v_align))),
            _ => None,
        },
        Paint::Status(x) => match role {
            "iconHeight" => Some(V::N(x.height)),
            "fontSource" => Some(url(&x.font)),
            "textScale" => Some(V::N(x.text_scale)),
            "color" => Some(V::C(x.color)),
            "entrySpacing" => Some(V::N(x.entry_spacing)),
            _ => background(&x.background, role),
        },
        Paint::Primary { leaving } => match (role, primary) {
            ("leaving", _) => Some(V::B(*leaving)),
            (_, Some(pr)) => layout(pr, role),
            _ => None,
        },
    };
    found.unwrap_or(V::None)
}

fn picture(q: &emusen_esde::scene::Picture, role: &str) -> Option<V> {
    Some(match role {
        "source" => V::U(q.source.clone()),
        "fit" => V::I(fit(q.fit)),
        "cropX" => V::N(q.crop_pos.0),
        "cropY" => V::N(q.crop_pos.1),
        "tileWidth" => V::N(q.tile_size.0),
        "tileHeight" => V::N(q.tile_size.1),
        "tileRight" => V::B(q.tile_align.0 == HAlign::Right),
        "tileBottom" => V::B(q.tile_align.1 == VAlign::Bottom),
        "saturation" => V::N(q.saturation),
        "radius" => V::N(q.corner_radius),
        "nearest" => V::B(q.nearest),
        "frame" => V::I(q.frame.map_or(-1, |f| f as i32)),
        _ => return tint(&q.tint, role),
    })
}

fn text(t: &emusen_esde::scene::Text, role: &str) -> Option<V> {
    Some(match role {
        "text" => V::S(t.text.clone()),
        "fontSource" => url(&t.font),
        "px" => V::N(t.px),
        "color" => V::C(t.color),
        "hAlign" => V::I(h(t.h_align)),
        "vAlign" => V::I(v(t.v_align)),
        "lineSpacing" => V::N(t.line_spacing),
        "wrap" => V::B(t.wrap),
        "ellipsis" => V::B(t.ellipsis),
        _ => return background(&t.background, role).or_else(|| scroll(&t.scroll, role)),
    })
}

fn layout(pr: &Primary, role: &str) -> Option<V> {
    if role == "selected" {
        return Some(V::I(pr.selected as i32));
    }
    match &pr.layout {
        Layout::Carousel(c) => Some(match role {
            "fit" => V::I(fit(c.fit)),
            "alignX" => V::N(match c.image_align.0 {
                HAlign::Left => 0.0,
                HAlign::Center => 0.5,
                HAlign::Right => 1.0,
            }),
            "alignY" => V::N(match c.image_align.1 {
                VAlign::Top => 0.0,
                VAlign::Center => 0.5,
                VAlign::Bottom => 1.0,
            }),
            "hasBackground" => V::B(c.background.is_some()),
            "background" => V::C(c.background.map_or(CLEAR, |t| t.color)),
            "backgroundEnd" => V::C(c.background.map_or(CLEAR, |t| t.end)),
            "backgroundVertical" => V::B(c.background.is_some_and(|t| t.vertical)),
            "fontSource" => url(&c.text.font),
            "px" => V::N(c.text.px),
            "lineSpacing" => V::N(c.text.line_spacing),
            "color" => V::C(c.text.color),
            "itemBackground" => V::C(c.text.background),
            _ => return None,
        }),
        Layout::TextList(t) => {
            let bar = t.selector.unwrap_or(emusen_esde::scene::Bar { x: 0.0, y: 0.0, w: 0.0, h: 0.0, color: CLEAR });
            let sb = t.selected_background.unwrap_or((CLEAR, 0.0, (0.0, 0.0)));
            Some(match role {
                "firstRow" => V::I(t.first_row as i32),
                "rowHeight" => V::N(t.row_h),
                "bandHeight" => V::N(t.band_h),
                "contentY" => V::N(t.content_y),
                "hasSelector" => V::B(t.selector.is_some()),
                "selectorX" => V::N(bar.x),
                "selectorY" => V::N(bar.y),
                "selectorW" => V::N(bar.w),
                "selectorH" => V::N(bar.h),
                "selectorColor" => V::C(bar.color),
                "hasSelectedBackground" => V::B(t.selected_background.is_some()),
                "selectedBackground" => V::C(sb.0),
                "selectedBackgroundRadius" => V::N(sb.1),
                "selectedBackgroundLeft" => V::N(sb.2 .0),
                "selectedBackgroundRight" => V::N(sb.2 .1),
                "primaryColor" => V::C(t.primary_color),
                "secondaryColor" => V::C(t.secondary_color),
                "selectedColor" => V::C(t.selected_color),
                "selectedSecondaryColor" => V::C(t.selected_secondary_color),
                "fontSource" => url(&t.font),
                "px" => V::N(t.px),
                "lineSpacing" => V::N(t.line_spacing),
                "margin" => V::N(t.margin),
                "hAlign" => V::I(h(t.h_align)),
                "markerWidth" => V::N(t.marker_w),
                _ => return scroll(&t.marquee, role),
            })
        }
        Layout::Grid(g) => Some(match role {
            "columns" => V::I(g.columns as i32),
            "itemWidth" => V::N(g.item.0),
            "itemHeight" => V::N(g.item.1),
            "pitchX" => V::N(g.pitch.0),
            "pitchY" => V::N(g.pitch.1),
            "marginX" => V::N(g.margin.0),
            "marginY" => V::N(g.margin.1),
            "contentY" => V::N(g.content_y),
            "itemScale" => V::N(g.item_scale),
            "focusFrom" => V::I(g.focus_from.map_or(-1, |f| f as i32)),
            "fromLevel" => V::N(g.from_level),
            "toLevel" => V::N(g.to_level),
            "progress" => V::N(g.progress),
            "unfocusedOpacity" => V::N(g.unfocused_opacity),
            "unfocusedSaturation" => V::N(g.unfocused_saturation),
            "unfocusedDim" => V::N(g.unfocused_dim),
            "fit" => V::I(fit(g.fit)),
            "cropX" => V::N(g.crop_pos.0),
            "cropY" => V::N(g.crop_pos.1),
            "nearest" => V::B(g.nearest),
            "imageScale" => V::N(g.image_scale),
            "radius" => V::N(g.image_radius),
            "hasSelectedTint" => V::B(g.selected_tint.is_some()),
            "selectedTint" => V::C(g.selected_tint.unwrap_or(g.tint.color)),
            "saturation" => V::N(g.saturation),
            "hasBackground" => V::B(g.background.is_some()),
            "background" => V::C(g.background.unwrap_or(CLEAR)),
            "backgroundSource" => url(&g.background_image),
            "backgroundScale" => V::N(g.background_scale),
            "backgroundRadius" => V::N(g.background_radius),
            "hasSelector" => V::B(g.selector.is_some()),
            "selectorColor" => V::C(g.selector.unwrap_or(CLEAR)),
            "selectorSource" => url(&g.selector_image),
            "selectorScale" => V::N(g.selector_scale),
            "selectorRadius" => V::N(g.selector_radius),
            "selectorOnTop" => V::B(g.selector_layer == "top"),
            "selectorInMiddle" => V::B(g.selector_layer == "middle"),
            "fontSource" => url(&g.text.font),
            "px" => V::N(g.text.px),
            "lineSpacing" => V::N(g.text.line_spacing),
            "color" => V::C(g.text.color),
            "itemBackground" => V::C(g.text.background),
            "selectedColor" => V::C(g.text_selected_color.unwrap_or(g.text.color)),
            "selectedItemBackground" => V::C(g.text_selected_background.unwrap_or(g.text.background)),
            "textRadius" => V::N(g.text_radius),
            _ => return tint(&g.tint, role),
        }),
    }
}

/// A row of a primary list by role name.
pub fn row(r: &Row, role: &str) -> V {
    match role {
        "text" => V::S(r.text.clone()),
        "image" => url(&r.image),
        "marker" => V::S(match r.marker {
            emusen_esde::scene::Marker::None => String::new(),
            emusen_esde::scene::Marker::Star => "star".into(),
            emusen_esde::scene::Marker::Folder => "folder".into(),
            emusen_esde::scene::Marker::Tick => "tick".into(),
        }),
        "folder" => V::B(r.folder),
        _ => V::None,
    }
}

/// A carousel slot by role name, its row's text and picture beside it.
pub fn slot(s: &Slot, r: Option<&Row>, role: &str) -> V {
    match role {
        "row" => V::I(s.row as i32),
        "boxX" => V::N(s.x),
        "boxY" => V::N(s.y),
        "boxWidth" => V::N(s.w),
        "boxHeight" => V::N(s.h),
        "grow" => V::N(s.scale),
        "growX" => V::N(s.scale_origin.0),
        "growY" => V::N(s.scale_origin.1),
        "turn" => V::N(s.rotation),
        "turnX" => V::N(s.rotation_origin.0),
        "turnY" => V::N(s.rotation_origin.1),
        "alpha" => V::N(s.opacity),
        "dim" => V::N(s.dim),
        "saturation" => V::N(s.saturation),
        "tint" => V::C(s.tint),
        "reflected" => V::B(s.reflection.is_some()),
        "reflectionOpacity" => V::N(s.reflection.map_or(0.0, |r| r.0)),
        "reflectionFalloff" => V::N(s.reflection.map_or(0.0, |r| r.1)),
        _ => r.map_or(V::None, |r| row(r, role)),
    }
}

/// How many part rows an element's paint has: badges, help entries, info and status items.
pub fn part_count(p: &Paint) -> usize {
    match p {
        Paint::Badges(b) => b.slots.len(),
        Paint::Help(x) => x.entries.len(),
        Paint::Info(x) => x.items.len(),
        Paint::Status(x) => x.items.len(),
        _ => 0,
    }
}

fn overlay(o: &Option<emusen_esde::scene::Overlay>, prefix: &str, role: &str) -> Option<V> {
    let rest = role.strip_prefix(prefix)?;
    let Some(o) = o else {
        return Some(match rest {
            "" => V::B(false),
            "Source" => V::U(String::new()),
            "Mark" => V::S(String::new()),
            "Tint" => V::C(CLEAR),
            _ => V::N(0.0),
        });
    };
    Some(match rest {
        "" => V::B(true),
        "Source" => icon_source(&o.icon),
        "Mark" => icon_mark(&o.icon),
        "X" => V::N(o.x),
        "Y" => V::N(o.y),
        "W" => V::N(o.w),
        "H" => V::N(o.h),
        "Tint" => V::C(o.tint),
        _ => return None,
    })
}

fn badge(b: &Badge, role: &str) -> V {
    let found = match role {
        "cellX" => Some(V::N(b.cell.0)),
        "cellY" => Some(V::N(b.cell.1)),
        "cellW" => Some(V::N(b.cell.2)),
        "cellH" => Some(V::N(b.cell.3)),
        _ => overlay(&Some(b.icon.clone()), "icon", role).or_else(|| overlay(&b.controller, "controller", role)).or_else(|| overlay(&b.link, "link", role)),
    };
    found.unwrap_or(V::None)
}

/// A part row by role name; `swap` places the help's buttons as the kit draws them.
pub fn part(p: &Paint, i: usize, role: &str, swap: bool) -> V {
    match p {
        Paint::Badges(b) => b.slots.get(i).map_or(V::None, |b| badge(b, role)),
        Paint::Help(x) => x.entries.get(i).map_or(V::None, |e| {
            let (button, pair) = crate::painter::help_buttons(e.button, e.pair, swap);
            match role {
                "button" => V::I(button),
                "pair" => V::I(pair),
                "label" => V::S(e.label.clone()),
                "icon" => url(&e.icon),
                _ => V::None,
            }
        }),
        Paint::Info(x) => x.items.get(i).map_or(V::None, |it| match role {
            "mark" => V::S(it.mark.name()),
            "text" => V::S(it.text.clone().unwrap_or_default()),
            _ => V::None,
        }),
        Paint::Status(x) => x.items.get(i).map_or(V::None, |it| match role {
            "source" => it.icon.as_ref().map_or(V::U(String::new()), icon_source),
            "mark" => it.icon.as_ref().map_or(V::S(String::new()), icon_mark),
            "text" => V::S(it.text.clone().unwrap_or_default()),
            _ => V::None,
        }),
        _ => V::None,
    }
}

/// Every role name the element model answers, in role order.
pub const ELEMENT_ROLES: [&str; 107] = [
    "elementId", "paint", "name", "boxX", "boxY", "boxWidth", "boxHeight", "turn", "originX", "originY", "alpha",
    "source", "fit", "cropX", "cropY", "tileWidth", "tileHeight", "tileRight", "tileBottom", "tint", "tintEnd", "gradientVertical",
    "saturation", "radius", "nearest", "frame",
    "text", "fontSource", "px", "color", "hAlign", "vAlign", "lineSpacing", "wrap", "ellipsis",
    "hasBackground", "background", "backgroundEnd", "backgroundVertical", "backgroundRadius", "padLeft", "padTop", "padRight", "padBottom",
    "scrolling", "scrollVertical", "scrollX", "repeatAt", "scrollY", "scrollOpacity",
    "value", "filledSource", "filledMark", "unfilledSource", "unfilledMark", "overlay",
    "iconColor", "entrySpacing", "iconGap", "entryScale", "filled", "iconHeight", "textScale",
    "leaving", "selected", "alignX", "alignY", "itemBackground",
    "firstRow", "rowHeight", "bandHeight", "contentY", "hasSelector", "selectorX", "selectorY", "selectorW", "selectorH", "selectorColor",
    "hasSelectedBackground", "selectedBackground", "selectedBackgroundRadius", "selectedBackgroundLeft", "selectedBackgroundRight",
    "primaryColor", "secondaryColor", "selectedColor", "selectedSecondaryColor", "margin", "markerWidth",
    "columns", "itemWidth", "itemHeight", "pitchX", "pitchY", "marginX", "marginY", "itemScale", "focusFrom", "fromLevel", "toLevel",
    "progress", "unfocusedOpacity", "unfocusedSaturation", "unfocusedDim", "imageScale", "hasSelectedTint", "selectedTint",
];

/// The grid's roles past the element list's first 107.
pub const GRID_ROLES: [&str; 9] = [
    "backgroundSource", "backgroundScale", "selectorSource", "selectorScale", "selectorRadius", "selectorOnTop", "selectorInMiddle",
    "selectedItemBackground", "textRadius",
];

pub const ROW_ROLES: [&str; 4] = ["text", "image", "marker", "folder"];

pub const SLOT_ROLES: [&str; 22] = [
    "row", "boxX", "boxY", "boxWidth", "boxHeight", "grow", "growX", "growY", "turn", "turnX", "turnY", "alpha", "dim",
    "saturation", "tint", "reflected", "reflectionOpacity", "reflectionFalloff", "text", "image", "marker", "folder",
];

pub const PART_ROLES: [&str; 34] = [
    "button", "pair", "label", "icon", "mark", "text", "source",
    "cellX", "cellY", "cellW", "cellH",
    "iconSource", "iconMark", "iconX", "iconY", "iconW", "iconH", "iconTint",
    "controller", "controllerSource", "controllerMark", "controllerX", "controllerY", "controllerW", "controllerH", "controllerTint",
    "link", "linkSource", "linkMark", "linkX", "linkY", "linkW", "linkH", "linkTint",
];
