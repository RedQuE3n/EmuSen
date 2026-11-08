//! The interface's rules as checks over an abstract item tree, with no toolkit behind it (EmuSen_InterfaceSpec.md §4, EmuSen_Qt.md §8.1, EmuSen_Kit.md §3).

mod fit;
mod reach;
mod rules;
mod tree;

use std::fmt;

pub use fit::{CAPS_SLACK, DESKTOP_CAPS, SMALL_TEXT_EXCEPTIONS, SMALLEST_TEXT, TWO_ROWS, fit, smallest_capitals};
pub use reach::{Edge, pad_reach, pointer_reach, reached};
pub use rules::{CONTRAST_SPARE, LARGE_BOLD_TEXT, LARGE_TEXT, contrast, named, no_chrome, palette_reach};
pub use tree::{Axes, Edges, Item, Kind, Legibility, Rect, Text};

use crate::input::{ElementId, WalkForm};
use crate::tokens::Colour;

/// An edge of a scrolling area.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

/// One line of an audit: what is wrong and where.
#[derive(Clone, Debug, PartialEq)]
pub enum Fault {
    PastItsLimit { item: ElementId, limit: Option<ElementId> },
    WordsPastOwner { words: ElementId, owner: ElementId },
    Overlap { a: ElementId, b: ElementId },
    ScrollsInTooLittleRoom { item: ElementId, height: f64 },
    CutAtEdge { item: ElementId, area: ElementId, side: Side },
    Ellipsis { item: ElementId },
    RunsPastItsBox { item: ElementId },
    CutAtFoot { item: ElementId },
    TooSmall { item: ElementId, capitals: f64 },
    TitleCut { item: ElementId },
    FooterCut { item: ElementId },
    Unreachable { item: ElementId },
    Covered { item: ElementId, by: ElementId },
    ClickDiffers { item: ElementId },
    WheelDoesNotMove { item: ElementId },
    Chrome { item: ElementId },
    Unnamed { item: ElementId },
    OutsidePalette { item: ElementId, colour: Colour },
    LowContrast { item: ElementId, ratio: f64, needed: f64 },
}

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fault::PastItsLimit { item, limit: Some(l) } => write!(f, "past its scroller or clip {}: {}", l.0, item.0),
            Fault::PastItsLimit { item, limit: None } => write!(f, "past its panel: {}", item.0),
            Fault::WordsPastOwner { words, owner } => write!(f, "words {} past their owner {}", words.0, owner.0),
            Fault::Overlap { a, b } => write!(f, "overlap: {} and {}", a.0, b.0),
            Fault::ScrollsInTooLittleRoom { item, height } => write!(f, "scrolls in too little room: {} at {height}", item.0),
            Fault::CutAtEdge { item, area, side } => write!(f, "cut at the {side:?} edge of {} with no fade: {}", area.0, item.0),
            Fault::Ellipsis { item } => write!(f, "cut with an ellipsis: {}", item.0),
            Fault::RunsPastItsBox { item } => write!(f, "runs past its box: {}", item.0),
            Fault::CutAtFoot { item } => write!(f, "cut at the foot: {}", item.0),
            Fault::TooSmall { item, capitals } => write!(f, "too small to read: {} with capitals of {capitals:.2}", item.0),
            Fault::TitleCut { item } => write!(f, "title cut: {}", item.0),
            Fault::FooterCut { item } => write!(f, "footer cut: {}", item.0),
            Fault::Unreachable { item } => write!(f, "the pad does not reach: {}", item.0),
            Fault::Covered { item, by } => write!(f, "a click at the centre of {} hits {}", item.0, by.0),
            Fault::ClickDiffers { item } => write!(f, "a click on {} does not call what A calls", item.0),
            Fault::WheelDoesNotMove { item } => write!(f, "the wheel does not move the scroller {}", item.0),
            Fault::Chrome { item } => write!(f, "desktop chrome on the big screen: {}", item.0),
            Fault::Unnamed { item } => write!(f, "no name a reader hears: {}", item.0),
            Fault::LowContrast { item, ratio, needed } => write!(f, "low contrast: {} at {ratio:.2}:1, needs {needed}:1", item.0),
            Fault::OutsidePalette { item, colour } => write!(f, "a colour outside the palette on {}: {}", item.0, colour.to_hex()),
        }
    }
}

/// Everything a recorded state gives the rules besides its tree.
#[derive(Clone, Copy, Debug)]
pub struct State<'a> {
    /// The screen's frame.
    pub panel: Rect,
    pub form: WalkForm,
    /// The element that has the focus, if any.
    pub focus: Option<ElementId>,
    /// The recorded focus graph.
    pub edges: &'a [Edge],
    /// The colours the state's resources declare.
    pub declared: &'a [Colour],
    /// A pixel, or a pixel and a half at a scale above one.
    pub slack: f64,
    /// The capitals' floor, `smallest_capitals()` on a big screen and 0 for a desktop window.
    pub smallest: f64,
}

/// Every rule over one state, in the order the sections name them.
pub fn check(root: &Item, state: &State<'_>) -> Vec<Fault> {
    let mut faults = fit(root, state.panel, state.slack, state.smallest);
    faults.extend(pad_reach(root, state.focus, state.edges));
    faults.extend(pointer_reach(root, &reached(state.focus, state.edges)));
    faults.extend(no_chrome(root, state.form));
    faults.extend(named(root));
    faults.extend(palette_reach(root, state.declared));
    faults.extend(contrast(root));
    faults
}
