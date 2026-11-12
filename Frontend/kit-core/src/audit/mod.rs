//! The interface's rules as checks over an abstract item tree, with no toolkit behind it (EmuSen_InterfaceSpec.md §4, EmuSen_Qt.md §8.1, EmuSen_Kit.md §3).

mod accepted;
mod exempt;
mod fit;
mod reach;
mod rules;
mod tree;

use std::fmt;

pub use accepted::{ACCEPTED, Accepted, Class, accepted};
pub use exempt::{Exempted, exempted};
pub use fit::{CAPS_SLACK, DESKTOP_CAPS, MOONRISE_SOURCE, SMALL_TEXT_EXCEPTIONS, SMALLEST_TEXT, TWO_ROWS, fit, small_text_exception, smallest_capitals};
pub use reach::{Edge, pad_reach, pointer_reach, reached};
pub use rules::{CONTRAST_SPARE, LARGE_BOLD_TEXT, LARGE_TEXT, STATE_CHANGE, changed_share, contrast, named, no_chrome, palette_reach};
pub use tree::{Axes, Edges, Item, Kind, Legibility, Rect, Stacking, Text};

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

impl Fault {
    /// The finding's name, the same for every fault of its kind; most lines begin with it.
    pub fn kind(&self) -> &'static str {
        match self {
            Fault::PastItsLimit { limit: Some(_), .. } => "past its scroller or clip",
            Fault::PastItsLimit { limit: None, .. } => "past its panel",
            Fault::WordsPastOwner { .. } => "words past their owner",
            Fault::Overlap { .. } => "overlap",
            Fault::ScrollsInTooLittleRoom { .. } => "scrolls in too little room",
            Fault::CutAtEdge { .. } => "cut at an edge with no fade",
            Fault::Ellipsis { .. } => "cut with an ellipsis",
            Fault::RunsPastItsBox { .. } => "runs past its box",
            Fault::CutAtFoot { .. } => "cut at the foot",
            Fault::TooSmall { .. } => "too small to read",
            Fault::TitleCut { .. } => "title cut",
            Fault::FooterCut { .. } => "footer cut",
            Fault::Unreachable { .. } => "the pad does not reach",
            Fault::Covered { .. } => "a click at the centre hits another",
            Fault::ClickDiffers { .. } => "a click does not call what A calls",
            Fault::WheelDoesNotMove { .. } => "the wheel does not move the scroller",
            Fault::Chrome { .. } => "desktop chrome on the big screen",
            Fault::Unnamed { .. } => "no name a reader hears",
            Fault::OutsidePalette { .. } => "a colour outside the palette",
            Fault::LowContrast { .. } => "low contrast",
        }
    }

    /// The items the finding names: the one it is about first, then the other of a pair (what it overlaps, the limit it
    /// passes, the area that cuts it, the owner its words pass, what a click hits instead).
    pub fn items(&self) -> Vec<ElementId> {
        match self {
            Fault::PastItsLimit { item, limit } => [Some(*item), *limit].into_iter().flatten().collect(),
            Fault::WordsPastOwner { words, owner } => vec![*words, *owner],
            Fault::Overlap { a, b } => vec![*a, *b],
            Fault::CutAtEdge { item, area, .. } => vec![*item, *area],
            Fault::Covered { item, by } => vec![*item, *by],
            Fault::ScrollsInTooLittleRoom { item, .. }
            | Fault::Ellipsis { item }
            | Fault::RunsPastItsBox { item }
            | Fault::CutAtFoot { item }
            | Fault::TooSmall { item, .. }
            | Fault::TitleCut { item }
            | Fault::FooterCut { item }
            | Fault::Unreachable { item }
            | Fault::ClickDiffers { item }
            | Fault::WheelDoesNotMove { item }
            | Fault::Chrome { item }
            | Fault::Unnamed { item }
            | Fault::OutsidePalette { item, .. }
            | Fault::LowContrast { item, .. } => vec![*item],
        }
    }
}

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = self.kind();
        match self {
            Fault::PastItsLimit { item, limit: Some(l) } => write!(f, "{kind} {}: {}", l.0, item.0),
            Fault::WordsPastOwner { words, owner } => write!(f, "words {} past their owner {}", words.0, owner.0),
            Fault::Overlap { a, b } => write!(f, "{kind}: {} and {}", a.0, b.0),
            Fault::ScrollsInTooLittleRoom { item, height } => write!(f, "{kind}: {} at {height}", item.0),
            Fault::CutAtEdge { item, area, side } => write!(f, "cut at the {side:?} edge of {} with no fade: {}", area.0, item.0),
            Fault::TooSmall { item, capitals } => write!(f, "{kind}: {} with capitals of {capitals:.2}", item.0),
            Fault::Covered { item, by } => write!(f, "a click at the centre of {} hits {}", item.0, by.0),
            Fault::ClickDiffers { item } => write!(f, "a click on {} does not call what A calls", item.0),
            Fault::WheelDoesNotMove { item } => write!(f, "{kind} {}", item.0),
            Fault::Chrome { item } => write!(f, "{kind}: {}", item.0),
            Fault::LowContrast { item, ratio, needed } => write!(f, "{kind}: {} at {ratio:.2}:1, needs {needed}:1", item.0),
            Fault::OutsidePalette { item, colour } => write!(f, "{kind} on {}: {}", item.0, colour.to_hex()),
            Fault::PastItsLimit { item, limit: None }
            | Fault::Ellipsis { item }
            | Fault::RunsPastItsBox { item }
            | Fault::CutAtFoot { item }
            | Fault::TitleCut { item }
            | Fault::FooterCut { item }
            | Fault::Unreachable { item }
            | Fault::Unnamed { item } => write!(f, "{kind}: {}", item.0),
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
