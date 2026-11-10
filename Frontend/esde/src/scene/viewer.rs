//! ES-DE's media viewer (USERGUIDE.md, "Game media viewer"; EmuSen_Qt.md §4.10): a game's video and pictures full
//! screen in ES-DE's order, left and right one at a time stopping at the ends, the triggers to the first and last, its
//! help prompts in a strip at the top, as ES-DE 3.4.1's captures show it. Plain records out, drawn above the view.

use crate::loader::Color;

use super::data::{Data, Game};
use super::draw::{key, Measures};
use super::frame::{Background, Face, Fit, HAlign, Help, HelpEntry, PadButton, Paint, Picture, Placed, Text, Tint, VAlign};

/// The types the viewer shows, in its order: the user guide's list, and the order its captures step through.
pub const ORDER: [&str; 8] = ["video", "cover", "backcover", "titlescreen", "screenshot", "fanart", "miximage", "custom"];

// Measured on the captures of 2026-11-09 (EmuSen_Qt.md §4.10): the strip's colour and the words' and icons', the words'
// size over the screen's height, and the strip's height over the words' size.
const STRIP: u32 = 0x2222_22FF;
const INK: u32 = 0xAAAA_AAFF;
const WORDS: f64 = 0.03;
const STRIP_OVER_WORDS: f64 = 1.35;

/// One shown item: its type and its file.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub kind: String,
    pub source: String,
}

/// The viewer of one game: its items and the one shown.
#[derive(Clone, Debug, PartialEq)]
pub struct Viewer {
    pub entries: Vec<Entry>,
    pub index: usize,
}

impl Viewer {
    /// The viewer of a game's media; none for a folder or a game with no media of the viewer's types.
    pub fn of(game: &Game) -> Option<Viewer> {
        if game.folder {
            return None;
        }
        let entries: Vec<Entry> =
            ORDER.iter().filter_map(|k| game.media.get(*k).map(|s| Entry { kind: (*k).to_owned(), source: s.clone() })).collect();
        (!entries.is_empty()).then_some(Viewer { entries, index: 0 })
    }

    /// One item on, or back; at an end it stays. Whether it moved.
    pub fn browse(&mut self, step: i32) -> bool {
        let to = (self.index as i64 + i64::from(step)).clamp(0, self.entries.len() as i64 - 1) as usize;
        std::mem::replace(&mut self.index, to) != to
    }

    /// The first item, or the last. Whether it moved.
    pub fn jump(&mut self, last: bool) -> bool {
        let to = if last { self.entries.len() - 1 } else { 0 };
        std::mem::replace(&mut self.index, to) != to
    }

    pub fn shown(&self) -> &Entry {
        &self.entries[self.index]
    }

    /// The strip's height in pixels for a screen `h` high.
    pub fn strip(h: f64) -> f64 {
        (WORDS * h * STRIP_OVER_WORDS).round()
    }

    /// The viewer's records over a screen of the data's size: the black ground, the strip, the item fitted below the
    /// strip, and the help centred in the strip once the painter has measured it.
    pub fn placed(&self, data: &Data, measures: &Measures) -> Vec<Placed> {
        let (w, h) = (data.width, data.height);
        let strip = Viewer::strip(h);
        let fill = |name: &str, y: f64, height: f64, color: u32| {
            let tint = Tint { color: Color(color), end: Color(color), vertical: false };
            let text = Text {
                text: String::new(),
                font: Face::default(),
                px: 1.0,
                color: Color(color),
                h_align: HAlign::Left,
                v_align: VAlign::Top,
                line_spacing: 0.0,
                wrap: false,
                ellipsis: false,
                background: Some(Background { tint, radius: 0.0, padding: [0.0; 4] }),
                scroll: None,
            };
            placed(name, 0.0, y, w, height, Paint::Text(text))
        };
        let shown = self.shown();
        let item = if shown.kind == "video" {
            Paint::Clip(shown.source.clone())
        } else {
            Paint::Picture(Picture {
                source: shown.source.clone(),
                fit: Fit::Contain,
                crop_pos: (0.5, 0.5),
                tile_size: (0.0, 0.0),
                tile_align: (HAlign::Left, VAlign::Top),
                tint: Tint::WHITE,
                saturation: 1.0,
                corner_radius: 0.0,
                nearest: false,
                frame: None,
            })
        };
        let entry = |label: &str, button: PadButton, pair: bool| HelpEntry { label: label.to_owned(), button, pair, icon: None };
        let help = Paint::Help(Help {
            entries: vec![
                entry("BROWSE", PadButton::DpadLeftRight, true),
                entry("FIRST", PadButton::LeftTrigger, false),
                entry("LAST", PadButton::RightTrigger, false),
            ],
            font: Face { file: data.default_font.clone(), own: true },
            px: WORDS * h,
            label_color: Color(INK),
            icon_color: Color(INK),
            background: None,
            entry_spacing: 0.00833 * w,
            icon_gap: 0.00416 * w,
            scale: 1.0,
            filled_glyphs: true,
        });
        let id = key(&["mediaviewer", "help"]);
        let (hw, hh) = measures.get(&id).map_or((0.0, 0.0), |m| (m.1, m.2));
        vec![
            fill("ground", 0.0, h, 0x0000_00FF),
            placed("item", 0.0, strip, w, h - strip, item),
            fill("strip", 0.0, strip, STRIP),
            placed("help", (w - hw) / 2.0, (strip - hh) / 2.0, hw, hh, help),
        ]
    }
}

// A record of the viewer's, its id from its name.
fn placed(name: &str, x: f64, y: f64, w: f64, h: f64, paint: Paint) -> Placed {
    Placed { id: key(&["mediaviewer", name]), kind: "mediaviewer".to_owned(), name: name.to_owned(), x, y, w, h, rotation: 0.0, rotation_origin: (0.5, 0.5), opacity: 1.0, paint }
}
