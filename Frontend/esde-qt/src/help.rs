//! ES-DE's help bar places (EmuSen_Qt.md §4.12.1): each entry's icon box and label from the bar's start, carried as
//! fractions, the labels at their laid-out widths, the bar ending at its last label and its width rounded to a whole
//! pixel. The painter rounds the bar's start to the nearest pixel and draws each part at the nearest pixel from there.

use emusen_esde::fonts::REFERENCE;
use emusen_esde::scene::Help;
use emusen_esde_text::{layout, Align, Spec};

/// The icon's box over the height of the reference glyph hinted at the help's font size, measured on ES-DE 3.4.1's help
/// copies of 2026-11-10.
pub const BOX_OVER_REFERENCE: f64 = 1.25;

/// The entries' starts and their labels' starts, and the bar's width, whole.
#[derive(Clone, Debug, PartialEq)]
pub struct Places {
    pub starts: Vec<f64>,
    pub labels: Vec<f64>,
    pub width: f64,
}

/// The icon's box of a help element: from its face's reference glyph hinted at the font size; a scale above 1 divides it.
/// None without FreeType, which the painter reports (`crate::hinted::ready`).
pub fn icon_box(h: &Help) -> Option<f64> {
    let reference = REFERENCE.chars().next()?;
    let height = crate::hinted::height(h.font.file.as_deref().unwrap_or(""), h.px, reference).ok()??;
    Some(height * BOX_OVER_REFERENCE / h.scale.max(1.0))
}

/// The places of a help element's entries; with no icon box (no FreeType, or a face without the reference glyph) the
/// icons take no room.
pub fn places(h: &Help) -> Places {
    places_with(h, icon_box(h).unwrap_or(0.0))
}

/// The places of a help element's entries with a given icon box.
pub fn places_with(h: &Help, icon: f64) -> Places {
    let face = crate::text::face_of(h.font.file.as_deref().unwrap_or("")).or_else(|| crate::text::face_of(""));
    let px = h.px * h.scale.min(1.0);
    let (mut starts, mut labels, mut x, mut width) = (vec![], vec![], 0.0, 0.0);
    for e in &h.entries {
        let words = face.as_ref().map_or(0, |f| {
            let spec = Spec { text: &e.label, px, own_face: h.font.own, width: 0.0, height: 0.0, line_spacing: 0.0, wrap: false, ellipsis: false, align: Align::Start };
            layout(f, &spec).width
        });
        starts.push(x);
        labels.push(x + icon + h.icon_gap);
        width = x + icon + h.icon_gap + f64::from(words);
        x = width + h.entry_spacing;
    }
    Places { starts, labels, width: width.round() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use emusen_esde::loader::Color;
    use emusen_esde::scene::{Face, HelpEntry, PadButton};

    fn rost() -> Option<String> {
        let p = std::path::PathBuf::from(std::env::var("HOME").ok()?).join(".cache/emusen/bigpicture/corpus/whole/caralt-es-de/assets/fonts/rost.otf");
        Some(p.to_string_lossy().into_owned()).filter(|_| p.is_file())
    }

    fn copy(file: String, scale: f64) -> Help {
        let entry = |label: &str| HelpEntry { label: label.into(), button: PadButton::South, pair: false, icon: None };
        Help {
            entries: ["Select", "Back", "View Media", "Favorites"].map(entry).to_vec(),
            font: Face { file: Some(file), own: false },
            px: 20.0,
            label_color: Color(0),
            icon_color: Color(0),
            background: None,
            entry_spacing: 8.0,
            icon_gap: 5.0,
            scale,
            filled_glyphs: true,
        }
    }

    // The labels' pens as ES-DE places the bar: its whole width laid back from 0.995 of 1280, the start rounded.
    fn pens(h: &Help, icon: f64) -> Vec<f64> {
        let p = places_with(h, icon);
        let start = (0.995 * 1280.0 - p.width).round();
        p.labels.iter().map(|l| (start + l).round()).collect()
    }

    // Copies v05 and g104 of the help sweeps: rost at 20 px, 5 px between icon and label, 8 between entries; ES-DE 3.4.1
    // drew the labels' pens at these columns (EmuSen_Qt.md §4.12.1). The hinted S is 14 px at 20 px.
    #[test]
    fn the_labels_stand_where_es_de_drew_them_from_a_whole_bar_and_a_rounded_start() {
        let file = rost().expect("CarAlt's rost in the theme corpus");
        assert_eq!(pens(&copy(file.clone(), 1.0), 14.0 * BOX_OVER_REFERENCE), [985.0, 1053.0, 1115.0, 1215.0]);
        assert_eq!(pens(&copy(file, 1.04), 14.0 * BOX_OVER_REFERENCE / 1.04), [987.0, 1055.0, 1115.0, 1215.0]);
    }
}
