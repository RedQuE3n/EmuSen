//! What the rules let pass by an exemption, item by item, so a test names each one it accepts (EmuSen_Kit.md §7.102).

use super::accepted::{Accepted, Class, accepted};
use super::fit::{CAPS_SLACK, fading_out, small_text_exception};
use super::rules::{CONTRAST_SPARE, declared_colour, ratios};
use super::tree::{Item, Kind};
use crate::input::ElementId;
use crate::tokens::Colour;
use std::fmt;

/// One item a rule did not hold to it, and the exemption that let it pass.
#[derive(Clone, Debug, PartialEq)]
pub enum Exempted {
    /// Words under their contrast over their worst samples that pass over all but `CONTRAST_SPARE` of them.
    Spare { item: ElementId, worst: f64, needed: f64 },
    /// Words under their contrast that cross a scroller's faded edge.
    Fading { item: ElementId, ratio: f64, needed: f64 },
    /// A colour outside the palette painted inside a theme's scene.
    Scene { item: ElementId, colour: Colour },
    /// Words under the capitals' floor that `SMALL_TEXT_EXCEPTIONS` names in its scene, with the class its entry gives.
    SmallText { item: ElementId, capitals: f64, class: Class },
    /// Words marked as data (a title, a name from the library) cut with an ellipsis.
    Elided { item: ElementId },
}

impl Exempted {
    /// The exemption's name, as its line begins with it.
    pub fn kind(&self) -> &'static str {
        match self {
            Exempted::Spare { .. } => "low contrast in the spare tenth",
            Exempted::Fading { .. } => "low contrast at a faded edge",
            Exempted::Scene { .. } => "a colour outside the palette in a scene",
            Exempted::SmallText { .. } => "too small to read, a named exception",
            Exempted::Elided { .. } => "data cut with an ellipsis",
        }
    }

    /// The class of finding it is, as the table of accepted classes names it.
    pub fn class(&self) -> Class {
        match self {
            Exempted::Spare { .. } => Class::SpareTenth,
            Exempted::Fading { .. } => Class::FadedEdge,
            Exempted::Scene { .. } => Class::SceneColour,
            Exempted::SmallText { class, .. } => *class,
            Exempted::Elided { .. } => Class::DataWordsElided,
        }
    }

    /// The row of `ACCEPTED` that accepts it; none, and it is a finding.
    pub fn accepted(&self) -> Option<&'static Accepted> {
        accepted(self.class())
    }

    /// The one item it let pass.
    pub fn item(&self) -> ElementId {
        match self {
            Exempted::Spare { item, .. }
            | Exempted::Fading { item, .. }
            | Exempted::Scene { item, .. }
            | Exempted::SmallText { item, .. }
            | Exempted::Elided { item } => *item,
        }
    }
}

impl fmt::Display for Exempted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = self.kind();
        match self {
            Exempted::Spare { item, worst, needed } => write!(f, "{kind}: {} at {worst:.2}:1, needs {needed}:1", item.0),
            Exempted::Fading { item, ratio, needed } => write!(f, "{kind}: {} at {ratio:.2}:1, needs {needed}:1", item.0),
            Exempted::Scene { item, colour } => write!(f, "{kind}: {} {}", item.0, colour.to_hex()),
            Exempted::SmallText { item, capitals, .. } => write!(f, "{kind}: {} with capitals of {capitals:.2}", item.0),
            Exempted::Elided { item } => write!(f, "{kind}: {}", item.0),
        }
    }
}

/// Every item the contrast, palette, floor and ellipsis rules let pass by an exemption, over one state; `declared` is the palette
/// the palette rule holds, and none where that rule is not run, so no scene colour is an exemption from it.
pub fn exempted(root: &Item, declared: Option<&[Colour]>, smallest: f64) -> Vec<Exempted> {
    let mut out = Vec::new();
    root.visit(&mut |item, ancestors| {
        if let Some((ratios, needed)) = item.legibility.as_ref().and_then(ratios) {
            let spare = ratios[((ratios.len() as f64) * CONTRAST_SPARE).floor() as usize];
            if fading_out(item, ancestors) && spare < needed {
                out.push(Exempted::Fading { item: item.id, ratio: spare, needed });
            } else if ratios[0] < needed && spare >= needed {
                out.push(Exempted::Spare { item: item.id, worst: ratios[0], needed });
            }
        }
        if let Some(declared) = declared.filter(|_| item.scene || ancestors.iter().any(|a| a.scene)) {
            for colour in item.paints.iter().filter(|c| !declared_colour(c, declared)) {
                out.push(Exempted::Scene { item: item.id, colour: *colour });
            }
        }
        if let Some(text) = item.text
            && item.kind == Kind::Words
            && item.is_drawn()
            && smallest > 0.0
            && text.cap_height < smallest - CAPS_SLACK
            && let Some(class) = small_text_exception(item, ancestors)
        {
            out.push(Exempted::SmallText { item: item.id, capitals: text.cap_height, class });
        }
        if item.kind == Kind::Words && item.is_drawn() && item.text.is_some_and(|t| t.ellipsis && t.data) {
            out.push(Exempted::Elided { item: item.id });
        }
    });
    out
}
