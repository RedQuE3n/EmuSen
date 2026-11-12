//! ES-DE 3.4.1's help strip (EmuSen_Qt.md §4.12.4), from es-core HelpComponent.cpp (applyTheme, updateGrid) and
//! ComponentGrid.cpp: the row 1.25 × the rendered S at the help's font size, each entry's icon box and label from the
//! strip's start, carried as fractions, and the grid's width the sum of every entry's parts with the spacing after the
//! last one, none of it rounded. The scene places the strip and the painter draws each part at the nearest pixel.

use emusen_esde::fonts::REFERENCE;
use emusen_esde::scene::Help;
use emusen_esde_text::{layout, Align, Spec};

/// The row's height over the rendered reference glyph's rows (HelpComponent.cpp, mLetterHeight).
pub const ROW_OVER_REFERENCE: f32 = 1.25;

/// The entries' starts and their labels' starts, and the grid's width, its trailing spacing in.
#[derive(Clone, Debug, PartialEq)]
pub struct Places {
    pub starts: Vec<f64>,
    pub labels: Vec<f64>,
    pub width: f64,
}

/// The row's height: the rows of the reference glyph rendered at the help's font size, before entryRelativeScale, × 1.25.
/// None without FreeType, which the painter reports (`crate::hinted::ready`).
pub fn row_height(h: &Help) -> Option<f64> {
    let reference = REFERENCE.chars().next()?;
    let rows = crate::hinted::rows(h.font.file.as_deref().unwrap_or(""), h.px as f32, reference).ok()??;
    Some(f64::from(rows as f32 * ROW_OVER_REFERENCE))
}

/// The icon's box: the row's height, divided by entryRelativeScale above 1.
pub fn icon_box(h: &Help) -> Option<f64> {
    let row = row_height(h)? as f32;
    Some(f64::from(if h.scale < 1.0 { row } else { row / h.scale as f32 }))
}

/// The labels' size: the help's, times entryRelativeScale below 1.
pub fn label_px(h: &Help) -> f64 {
    if h.scale < 1.0 { f64::from(h.px as f32 * h.scale as f32) } else { h.px }
}

/// The places of a help element's entries; with no icon box (no FreeType, or a face without the reference glyph) the
/// icons take no room.
pub fn places(h: &Help) -> Places {
    places_with(h, icon_box(h).unwrap_or(0.0))
}

/// The places of a help element's entries with a given icon box.
pub fn places_with(h: &Help, icon: f64) -> Places {
    let face = crate::text::face_of(h.font.file.as_deref().unwrap_or("")).or_else(|| crate::text::face_of(""));
    let px = label_px(h);
    let (mut starts, mut labels, mut x) = (vec![], vec![], 0.0);
    for e in &h.entries {
        let words = face.as_ref().map_or(0, |f| {
            let spec = Spec { text: &e.label, px, own_face: h.font.own, width: 0.0, height: 0.0, line_spacing: 0.0, wrap: false, ellipsis: false, align: Align::Start };
            layout(f, &spec).width
        });
        starts.push(x);
        labels.push(x + icon + h.icon_gap);
        x += icon + f64::from(words) + h.icon_gap + h.entry_spacing;
    }
    Places { starts, labels, width: x }
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

    // The labels' pens as ES-DE places the strip: pos 0.995 of 1280 at origin 1, by the scene's rule (EmuSen_Qt.md §4.12.4).
    fn pens(h: &Help, icon: f64) -> Vec<f64> {
        let p = places_with(h, icon);
        let at = emusen_esde::loader::Pair { x: 0.995, y: 0.97 };
        let (start, _) = emusen_esde::scene::help::top_left(Some(at), emusen_esde::loader::Pair { x: 1.0, y: 1.0 }, h.entry_spacing, (p.width, 17.5), 1280.0, 800.0);
        p.labels.iter().map(|l| (start + l).round()).collect()
    }

    // Copies v05 and g104 of the help sweeps: rost at 20 px, 5 px between icon and label, 8 between entries; ES-DE 3.4.1
    // drew the labels' pens at these columns (EmuSen_Qt.md §4.12.1). The rendered S is 14 rows at 20 px.
    #[test]
    fn the_labels_stand_where_es_de_drew_them_from_the_grid_s_width_and_two_roundings() {
        let file = rost().expect("CarAlt's rost in the theme corpus");
        let row = 14.0 * f64::from(ROW_OVER_REFERENCE);
        assert_eq!(pens(&copy(file.clone(), 1.0), row), [985.0, 1053.0, 1115.0, 1215.0]);
        assert_eq!(pens(&copy(file, 1.04), f64::from(row as f32 / 1.04)), [987.0, 1055.0, 1115.0, 1215.0]);
    }

    // The row's height against figures measured on ES-DE's captures, not from ROW_OVER_REFERENCE: Artflix's bars, a box of
    // 17.5 and 26.25 px at 20 and 30 px from disc to disc, and rost's interval at 20 px, 17.17-17.52 (EmuSen_Qt.md §4.12.4).
    #[test]
    fn the_row_s_height_is_what_es_de_s_captures_show() {
        let home = std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/emusen/bigpicture/corpus/whole");
        let exo = home.join("artflix-revisited-es-de/_inc/fonts/Exo2-RegularCondensed.otf").to_string_lossy().into_owned();
        let at = |file: &str, px: f64| row_height(&Help { px, ..copy(file.to_owned(), 1.0) });
        assert_eq!(at(&exo, 20.0), Some(17.5), "Artflix at 1280 x 800");
        assert_eq!(at(&exo, 30.0), Some(26.25), "Artflix at 1920 x 1200");
        let rost = at(&rost().expect("CarAlt's rost in the theme corpus"), 20.0).unwrap();
        assert!((17.17..=17.52).contains(&rost), "rost at 20 px: {rost}");
    }

    // The grid's width is every entry's parts with the spacing after the last, unrounded (HelpComponent.cpp updateGrid).
    #[test]
    fn the_grid_s_width_keeps_its_fraction_and_its_trailing_spacing() {
        let file = rost().expect("CarAlt's rost in the theme corpus");
        let mut h = copy(file.clone(), 1.0);
        h.entries.truncate(2);
        let face = crate::text::face_of(&file).expect("rost");
        let width = |text: &str| {
            let spec = Spec { text, px: 20.0, own_face: false, width: 0.0, height: 0.0, line_spacing: 0.0, wrap: false, ellipsis: false, align: Align::Start };
            f64::from(layout(&face, &spec).width)
        };
        let p = places_with(&h, 17.3);
        let want = 2.0 * (17.3 + 5.0 + 8.0) + width(&h.entries[0].label) + width(&h.entries[1].label);
        assert!((p.width - want).abs() < 1e-9 && p.width.fract() > 0.5, "{p:?} against {want}");
        assert!((p.width - (p.labels[1] + width(&h.entries[1].label)) - 8.0).abs() < 1e-9, "the trailing spacing: {p:?}");
    }
}
