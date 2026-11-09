//! The settings frame's measures in design units on the big screen, as screen 6 of EmuSen_Interface.md §4 and LunaP's
//! `SettingsLayout` set them (EmuSen_Kit.md §7.40): the strip under the safe area's top, the rows from the left, the pane
//! at the right, the rows taking what the pane and the gap leave.

use crate::tokens::{Form, spacing};

const BIG: crate::tokens::Spacing = spacing(Form::BigScreen);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SettingsGeometry {
    /// The rows' and the strip's left edge, and the pane's distance from the right.
    pub side: f64,
    pub strip_top: f64,
    pub strip_height: f64,
    /// Between two names of the strip.
    pub strip_gap: f64,
    /// A name's distance from its tab's edges.
    pub tab_padding: f64,
    /// The top of the rows and of the pane.
    pub content_top: f64,
    pub pane_width: f64,
    pub pane_height: f64,
    /// Between the rows and the pane.
    pub pane_gap: f64,
    /// The pane's words and slot from its edges.
    pub pane_inset: f64,
    /// Below the pane's slot, before its first line.
    pub slot_gap: f64,
    /// Between two lines of the pane.
    pub line_gap: f64,
}

pub const SETTINGS: SettingsGeometry = SettingsGeometry {
    side: BIG.safe_area.left,
    strip_top: BIG.safe_area.top,
    strip_height: BIG.control_height,
    strip_gap: BIG.base * 2.0,
    tab_padding: BIG.base * 4.0,
    content_top: BIG.safe_area.top + BIG.control_height + BIG.pane_gutter,
    pane_width: 416.0,
    pane_height: 578.0,
    pane_gap: BIG.pane_gap,
    pane_inset: BIG.pane_gutter,
    slot_gap: BIG.base * 3.0,
    line_gap: BIG.base * 2.0,
};

impl SettingsGeometry {
    /// The pane's left edge on a canvas this many units wide.
    pub fn pane_left(&self, units_wide: f64) -> f64 {
        units_wide - self.side - self.pane_width
    }

    /// The rows' width: from the side to the gap before the pane.
    pub fn rows_width(&self, units_wide: f64) -> f64 {
        (self.pane_left(units_wide) - self.pane_gap - self.side).max(0.0)
    }

    /// How many whole rows stand beside the pane.
    pub fn rows_shown(&self, row_height: f64) -> usize {
        if row_height > 0.0 { (self.pane_height / row_height).floor() as usize } else { 0 }
    }

    /// The width of the pane's slot and lines.
    pub fn slot_width(&self) -> f64 {
        self.pane_width - 2.0 * self.pane_inset
    }
}
