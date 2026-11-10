//! The tool screen's places: a header row of facts, a section strip with search, the list in whole rows, the apply bar
//! under it, and the pane beside both, spanning the list and the bar (EmuSen_Interface.md §15.2, §15.9; EmuSen_Kit.md §7.43).
//! It names no tool: a host hands it the screen's size and lays its own content into the places it gives back.

use crate::audit::Rect;
use crate::tokens::{Form, Spacing, spacing};

/// The screen's measures in design units, as Mistress's Cores recording places them at 1280 × 800.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToolGeometry {
    /// The big screen's safe area, row height and help bar, the tokens' own.
    pub spacing: Spacing,
    pub header: f64,
    /// From the safe area's top to the strip's.
    pub strip_offset: f64,
    pub strip: f64,
    pub search: f64,
    /// From the safe area's top to the list's.
    pub body_offset: f64,
    /// The list's share of the width, the pane taking the rest less the gap (`AdaptiveLayout.ContentShare`).
    pub content_share: f64,
    pub gap: f64,
    /// The apply bar under the list: its inset from the list's foot, its words and its controls.
    pub bar: f64,
    pub bar_inset: f64,
    pub pane_padding: f64,
}

impl ToolGeometry {
    pub const RECORDED: ToolGeometry = ToolGeometry {
        spacing: spacing(Form::BigScreen),
        header: 48.0,
        strip_offset: 60.0,
        strip: 44.0,
        search: 240.0,
        body_offset: 120.0,
        content_share: 0.6,
        gap: 16.0,
        bar: 110.0,
        bar_inset: 18.0,
        pane_padding: 24.0,
    };
}

/// Every place on the tool screen, in design units from the screen's top left.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct ToolLayout {
    pub header: Rect,
    pub strip: Rect,
    pub search: Rect,
    pub list: Rect,
    pub bar: Rect,
    pub pane: Rect,
    /// How many whole rows the list shows.
    pub rows: usize,
}

/// The foot of a screen's content: the safe area's foot less a help bar, where Mistress's recordings end it.
pub fn content_foot(height: f64, s: &Spacing) -> f64 {
    height - s.safe_area.bottom - s.help_bar
}

/// The places on a screen `width` × `height` design units. The list holds as many whole rows as leave room for the bar
/// under it; the bar starts at the list's foot; the pane stands beside both, from the list's top to the bar's foot.
pub fn tool_layout(width: f64, height: f64, g: &ToolGeometry) -> ToolLayout {
    let s = &g.spacing;
    let (left, top) = (s.safe_area.left, s.safe_area.top);
    let across = (width - left - s.safe_area.right).max(0.0);
    let body = top + g.body_offset;
    let rows = ((content_foot(height, s) - body - g.bar) / s.row_height).floor().max(0.0) as usize;
    let content = (across * g.content_share).round();
    let search = g.search.min(across);
    let list = Rect::new(left, body, content, rows as f64 * s.row_height);
    let bar = Rect::new(left, list.bottom(), content, g.bar);
    let pane_x = left + content + g.gap;
    ToolLayout {
        header: Rect::new(left, top, across, g.header),
        strip: Rect::new(left, top + g.strip_offset, across - search, g.strip),
        search: Rect::new(left + across - search, top + g.strip_offset, search, g.strip),
        list,
        bar,
        pane: Rect::new(pane_x, body, (left + across - pane_x).max(0.0), bar.bottom() - body),
        rows,
    }
}
