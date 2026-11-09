//! The page frame's places: a hero across the top under the OnArt gradient, the cover over its foot, the title, facts,
//! actions and section strip beside the cover, and the content below, as tiles or rows (EmuSen_Interface.md §2.5, §4.3;
//! EmuSen_Kit.md §7.44). It names no game: a host hands it the screen's size and the cover's shape.

use crate::audit::Rect;
use crate::tokens::{Form, Spacing, spacing};
use crate::tool::content_foot;

/// The page's measures in design units, as Mistress's game page recording places them at 1280 × 800.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PageGeometry {
    /// The big screen's safe area, row height and help bar, the tokens' own.
    pub spacing: Spacing,
    pub hero: f64,
    pub cover_top: f64,
    pub cover_height: f64,
    /// Where the title, facts, actions and strip begin, from the safe area's left.
    pub column: f64,
    pub title_top: f64,
    pub title: f64,
    pub facts_top: f64,
    pub facts: f64,
    pub actions_top: f64,
    pub actions: f64,
    pub strip_top: f64,
    pub strip: f64,
    pub content_top: f64,
    /// A tile's picture, the gap between tiles, and where its two lines stand under the picture's top.
    pub tile_width: f64,
    pub tile_height: f64,
    pub tile_gap: f64,
    pub label_offset: f64,
    pub line_offset: f64,
    /// Words beside a picture in place of tiles or rows (a game's Overview): the picture at the content's right, the
    /// gap between it and the words.
    pub picture_width: f64,
    pub picture_height: f64,
    pub picture_gap: f64,
}

impl PageGeometry {
    pub const RECORDED: PageGeometry = PageGeometry {
        spacing: spacing(Form::BigScreen),
        hero: 360.0,
        cover_top: 180.0,
        cover_height: 268.0,
        column: 272.0,
        title_top: 226.0,
        title: 53.0,
        facts_top: 286.0,
        facts: 27.0,
        actions_top: 328.0,
        actions: 44.0,
        strip_top: 396.0,
        strip: 44.0,
        content_top: 464.0,
        tile_width: 200.0,
        tile_height: 150.0,
        tile_gap: 24.0,
        label_offset: 162.0,
        line_offset: 190.0,
        picture_width: 388.0,
        picture_height: 222.0,
        picture_gap: 24.0,
    };
}

/// Every place on the page, in design units from the screen's top left.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct PageLayout {
    pub hero: Rect,
    pub cover: Rect,
    pub title: Rect,
    pub facts: Rect,
    pub actions: Rect,
    pub strip: Rect,
    pub content: Rect,
    /// How many whole rows the content shows when it is rows.
    pub rows: usize,
}

/// The places on a screen `width` × `height` design units, for a cover `aspect` wide per unit of height (3:4 is 0.75).
/// The cover keeps its height and takes its width from its shape, never past the column beside it.
pub fn page_layout(width: f64, height: f64, aspect: f64, g: &PageGeometry) -> PageLayout {
    let s = &g.spacing;
    let left = s.safe_area.left;
    let right = (width - s.safe_area.right).max(left);
    let column = left + g.column;
    let beside = (right - column).max(0.0);
    let cover_width = (g.cover_height * aspect.max(0.0)).round().min(g.column - s.pane_gap);
    let content = (content_foot(height, s) - g.content_top).max(0.0);
    PageLayout {
        hero: Rect::new(0.0, 0.0, width.max(0.0), g.hero),
        cover: Rect::new(left, g.cover_top, cover_width, g.cover_height),
        title: Rect::new(column, g.title_top, beside, g.title),
        facts: Rect::new(column, g.facts_top, beside, g.facts),
        actions: Rect::new(column, g.actions_top, beside, g.actions),
        strip: Rect::new(column, g.strip_top, beside, g.strip),
        content: Rect::new(left, g.content_top, right - left, content),
        rows: (content / s.row_height).floor() as usize,
    }
}
