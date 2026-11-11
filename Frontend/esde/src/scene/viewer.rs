//! ES-DE's media viewer (USERGUIDE.md, "Game media viewer"; EmuSen_Qt.md §4.10, §4.13): a game's video and pictures
//! full screen in ES-DE's order, left and right one at a time stopping at the ends, the triggers to the first and last;
//! up to the game's PDF manual, its pages browsed, zoomed by the shoulders and panned when zoomed; its help prompts in a
//! strip, as ES-DE 3.4.1's captures show it, under the player's viewer settings. Plain records out, drawn above the view.

use crate::loader::Color;

use super::data::{Data, Game};
use super::draw::{key, Measures};
use super::frame::{Background, Clip, Face, Fit, HAlign, Help, HelpEntry, PadButton, Page, Paint, Picture, Placed, Text, Tint, VAlign};

/// The types the viewer shows, in its order: the user guide's list, and the order its captures step through.
pub const ORDER: [&str; 8] = ["video", "cover", "backcover", "titlescreen", "screenshot", "fanart", "miximage", "custom"];

/// Each type's words where the help names it (USERGUIDE.md, "Display media types"), in `ORDER`'s order.
const TYPE_WORDS: [&str; 8] = ["VIDEO", "BOX COVER", "BOX BACK COVER", "TITLE SCREEN", "SCREENSHOT", "FAN ART", "MIXIMAGE", "CUSTOM"];

/// The media type that holds a game's PDF manual.
pub const MANUAL: &str = "manual";

// Measured on the captures of 2026-11-09 (EmuSen_Qt.md §4.10, §4.13): the strip's colour and the words' and icons', the
// words' size over the screen's height, the strip's height over the words' size, the left words' place over the
// screen's width, a zoom step over the screen's height, a pan step over the screen's width, and the video blur's Gaussian
// sigma over the screen's width.
const STRIP: u32 = 0x2222_22FF;
const INK: u32 = 0xAAAA_AAFF;
const WORDS: f64 = 0.03;
const STRIP_OVER_WORDS: f64 = 1.35;
const LEFT_WORDS: f64 = 0.01;
const ZOOM_STEP: f64 = 0.5;
const PAN_STEP: f64 = 0.125;
const BLUR_SIGMA: f64 = 1.0 / 512.0;

/// Where the help prompts stand (ES-DE's MediaViewerHelpPrompts).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HelpPlace {
    #[default]
    Top,
    Bottom,
    Off,
}

impl HelpPlace {
    /// ES-DE's word for it: top, bottom or disabled; any other is the default.
    pub fn of(word: &str) -> HelpPlace {
        match word {
            "bottom" => HelpPlace::Bottom,
            "disabled" => HelpPlace::Off,
            _ => HelpPlace::Top,
        }
    }
}

/// The player's viewer settings (USERGUIDE.md, "Media viewer settings"), with ES-DE's defaults; the scanlines are not
/// offered (EmuSen_Galaxia.md §3.15.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewerSettings {
    pub help: HelpPlace,
    pub show_types: bool,
    pub keep_video_running: bool,
    pub stretch_videos: bool,
    pub blur: bool,
    pub audio: bool,
}

impl Default for ViewerSettings {
    fn default() -> Self {
        ViewerSettings { help: HelpPlace::Top, show_types: false, keep_video_running: true, stretch_videos: false, blur: false, audio: true }
    }
}

/// One shown item: its type and its file.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub kind: String,
    pub source: String,
}

/// The manual's view: the page shown, the zoom steps taken and the pan steps, right and down.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ManualView {
    pub page: usize,
    pub zoom: u32,
    pub pan: (i32, i32),
}

/// A button the viewer reads; any other closes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Left,
    Right,
    Up,
    Down,
    First,
    Last,
    ZoomOut,
    ZoomIn,
}

/// The viewer of one game: its items and the one shown, its manual and the manual's view while it is shown.
#[derive(Clone, Debug, PartialEq)]
pub struct Viewer {
    pub entries: Vec<Entry>,
    pub index: usize,
    pub manual: Option<String>,
    /// The manual's view while the manual is shown.
    pub reading: Option<ManualView>,
    /// The manual's pages, once the painter has opened it; 0 before.
    pub pages: usize,
}

impl Viewer {
    /// The viewer of a game's media; none for a folder or a game with no media of the viewer's types and no manual.
    pub fn of(game: &Game) -> Option<Viewer> {
        if game.folder {
            return None;
        }
        let mut entries: Vec<Entry> =
            ORDER.iter().filter_map(|k| game.media.get(*k).map(|s| Entry { kind: (*k).to_owned(), source: s.clone() })).collect();
        // With no video the screenshot stands first, where the video would (EmuSen_Qt.md §4.10).
        if !game.media.contains_key("video") {
            if let Some(at) = entries.iter().position(|e| e.kind == "screenshot") {
                let shot = entries.remove(at);
                entries.insert(0, shot);
            }
        }
        let manual = game.media.get(MANUAL).cloned();
        (!entries.is_empty() || manual.is_some()).then_some(Viewer { entries, index: 0, manual, reading: None, pages: 0 })
    }

    /// One item on, or back; at an end it stays. Whether it moved.
    pub fn browse(&mut self, step: i32) -> bool {
        if self.entries.is_empty() {
            return false;
        }
        let to = (self.index as i64 + i64::from(step)).clamp(0, self.entries.len() as i64 - 1) as usize;
        std::mem::replace(&mut self.index, to) != to
    }

    /// The first item, or the last. Whether it moved.
    pub fn jump(&mut self, last: bool) -> bool {
        if self.entries.is_empty() {
            return false;
        }
        let to = if last { self.entries.len() - 1 } else { 0 };
        std::mem::replace(&mut self.index, to) != to
    }

    /// A press, as the user guide's "Game media viewer" gives it: whether the view changed, or none when the button
    /// closes the viewer. Up opens the manual and down leaves it; in it, left and right turn the page and the triggers
    /// go to the first and last, the shoulders zoom, and while zoomed the d-pad pans and a trigger resets the view.
    pub fn press(&mut self, key: Key) -> Option<bool> {
        let Some(view) = self.reading.as_mut() else {
            return Some(match key {
                Key::Left | Key::Right => self.browse(if key == Key::Left { -1 } else { 1 }),
                Key::First | Key::Last => self.jump(key == Key::Last),
                Key::Up if self.manual.is_some() => {
                    self.reading = Some(ManualView::default());
                    true
                }
                Key::Up | Key::Down | Key::ZoomOut | Key::ZoomIn => false,
            });
        };
        let before = *view;
        let last = self.pages.saturating_sub(1);
        match (key, view.zoom) {
            (Key::Down, 0) => {
                self.reading = None;
                return Some(true);
            }
            (Key::Up, 0) => {}
            (Key::Left, 0) => view.page = view.page.saturating_sub(1),
            (Key::Right, 0) => view.page = (view.page + 1).min(last),
            (Key::First, 0) => view.page = 0,
            (Key::Last, 0) => view.page = last,
            (Key::Left, _) => view.pan.0 -= 1,
            (Key::Right, _) => view.pan.0 += 1,
            (Key::Up, _) => view.pan.1 -= 1,
            (Key::Down, _) => view.pan.1 += 1,
            (Key::First | Key::Last, _) => *view = ManualView { page: view.page, ..ManualView::default() },
            (Key::ZoomIn, _) => view.zoom += 1,
            (Key::ZoomOut, 0) => {}
            (Key::ZoomOut, _) => {
                view.zoom -= 1;
                if view.zoom == 0 {
                    view.pan = (0, 0);
                }
            }
        }
        Some(*view != before)
    }

    /// The manual's page count, as the painter read it; a page past the last is the last.
    pub fn set_pages(&mut self, pages: usize) -> bool {
        if let Some(view) = self.reading.as_mut() {
            view.page = view.page.min(pages.saturating_sub(1));
        }
        std::mem::replace(&mut self.pages, pages) != pages
    }

    pub fn shown(&self) -> Option<&Entry> {
        self.entries.get(self.index)
    }

    /// The strip's height in pixels for a screen `h` high.
    pub fn strip(h: f64) -> f64 {
        (WORDS * h * STRIP_OVER_WORDS).round()
    }

    /// The help's entries for what is shown: the media's, with the manual's door where the game has one; the manual's;
    /// or, zoomed, the pan's.
    fn entries(&self) -> Vec<HelpEntry> {
        let entry = |label: &str, button: PadButton, pair: bool| HelpEntry { label: label.to_owned(), button, pair, icon: None };
        let browse = entry("BROWSE", PadButton::DpadLeftRight, true);
        let ends = [entry("FIRST", PadButton::LeftTrigger, false), entry("LAST", PadButton::RightTrigger, false)];
        let zoom = entry("ZOOM", PadButton::LeftShoulder, true);
        match self.reading {
            None if self.manual.is_some() => [vec![browse, entry("PDF MANUAL", PadButton::DpadUpDown, true)], ends.to_vec()].concat(),
            None => [vec![browse], ends.to_vec()].concat(),
            Some(view) if view.zoom == 0 => [vec![browse, entry("GAME MEDIA", PadButton::DpadUpDown, true)], ends.to_vec(), vec![zoom]].concat(),
            Some(_) => vec![entry("PAN", PadButton::Dpad, true), entry("RESET", PadButton::LeftTrigger, true), zoom],
        }
    }

    /// The words at the strip's left: the page and the pages in the manual, else the shown type's when the player asked
    /// for them.
    fn left_words(&self, settings: &ViewerSettings) -> Option<String> {
        match self.reading {
            Some(view) if self.pages > 0 => Some(format!("PAGE {} OF {}", view.page + 1, self.pages)),
            Some(_) => None,
            None if settings.show_types => self.shown().and_then(|e| ORDER.iter().position(|k| *k == e.kind)).map(|i| TYPE_WORDS[i].to_owned()),
            None => None,
        }
    }

    /// The manual page's box: fitted inside the area, each zoom step half the screen's height more of the page's
    /// height, centred, and moved by the pan steps; none before the painter has read the page's size.
    fn page_box(&self, view: ManualView, area: (f64, f64, f64, f64), screen: (f64, f64), measures: &Measures) -> (f64, f64, f64, f64) {
        let (ax, ay, aw, ah) = area;
        let Some(&(_, pw, ph)) = measures.get(&key(&["mediaviewer", "item"])).filter(|m| m.1 > 0.0 && m.2 > 0.0) else { return (ax, ay, 0.0, 0.0) };
        let scale = (aw / pw).min(ah / ph) + f64::from(view.zoom) * ZOOM_STEP * screen.1 / ph;
        let (w, h) = (pw * scale, ph * scale);
        let step = PAN_STEP * screen.0;
        (ax + (aw - w) / 2.0 - f64::from(view.pan.0) * step, ay + (ah - h) / 2.0 - f64::from(view.pan.1) * step, w, h)
    }

    /// The viewer's records over a screen of the data's size: the black ground, the item (or the manual's page) in the
    /// screen less the strip, the strip, the words at its left and the help centred in it once the painter has measured
    /// it. A video kept running while a picture is shown stays drawn, unseen.
    pub fn placed(&self, data: &Data, measures: &Measures) -> Vec<Placed> {
        let settings = &data.viewer;
        let (w, h) = (data.width, data.height);
        let strip = if settings.help == HelpPlace::Off { 0.0 } else { Viewer::strip(h) };
        let strip_y = if settings.help == HelpPlace::Bottom { h - strip } else { 0.0 };
        let area = (0.0, if settings.help == HelpPlace::Top { strip } else { 0.0 }, w, h - strip);
        let words = |text: &str, color: u32, background: Option<Background>| Text {
            text: text.to_owned(),
            font: Face { file: data.default_font.clone(), own: true },
            px: WORDS * h,
            color: Color(color),
            h_align: HAlign::Left,
            v_align: VAlign::Center,
            line_spacing: 0.0,
            wrap: false,
            ellipsis: false,
            every_line: false,
            snap_rows: false,
            background,
            scroll: None,
        };
        let fill = |name: &str, y: f64, height: f64, color: u32| {
            let tint = Tint { color: Color(color), end: Color(color), vertical: false };
            placed(name, 0.0, y, w, height, 1.0, Paint::Text(words("", color, Some(Background { tint, radius: 0.0, padding: [0.0; 4] }))))
        };
        let mut out = vec![fill("ground", 0.0, h, 0x0000_00FF)];
        let video = self.entries.iter().find(|e| e.kind == "video");
        let blur = if settings.blur { BLUR_SIGMA * w } else { 0.0 };
        let clip = |source: &str| Paint::Clip(Clip { source: source.to_owned(), stretch: settings.stretch_videos, audio: settings.audio, blur });
        let (ax, ay, aw, ah) = area;
        match (self.reading, self.shown()) {
            (Some(view), _) => {
                let (x, y, pw, ph) = self.page_box(view, area, (w, h), measures);
                let source = self.manual.clone().unwrap_or_default();
                out.push(placed("item", x, y, pw, ph, 1.0, Paint::Page(Page { source, page: view.page })));
            }
            (None, Some(e)) if e.kind == "video" => out.push(placed("video", ax, ay, aw, ah, 1.0, clip(&e.source))),
            (None, Some(e)) => {
                if let Some(v) = video.filter(|_| settings.keep_video_running) {
                    out.push(placed("video", ax, ay, aw, ah, 0.0, clip(&v.source)));
                }
                let picture = Picture {
                    source: e.source.clone(),
                    fit: Fit::Contain,
                    crop_pos: (0.5, 0.5),
                    tile_size: (0.0, 0.0),
                    tile_align: (HAlign::Left, VAlign::Top),
                    tint: Tint::WHITE,
                    saturation: 1.0,
                    corner_radius: 0.0,
                    nearest: false,
                    frame: None,
                };
                out.push(placed("item", ax, ay, aw, ah, 1.0, Paint::Picture(picture)));
            }
            (None, None) => {}
        }
        if settings.help == HelpPlace::Off {
            return out;
        }
        out.push(fill("strip", strip_y, strip, STRIP));
        if let Some(text) = self.left_words(settings) {
            out.push(placed("left", (LEFT_WORDS * w).round(), strip_y, w / 2.0, strip, 1.0, Paint::Text(words(&text, INK, None))));
        }
        let spacing = 0.00833 * w;
        let help = Paint::Help(Help {
            entries: self.entries(),
            font: super::paint::es_de_help_face(data),
            px: WORDS * h,
            label_color: Color(INK),
            icon_color: Color(INK),
            background: None,
            entry_spacing: spacing,
            icon_gap: 0.00416 * w,
            scale: 1.0,
            filled_glyphs: true,
        });
        // Measured as ES-DE's grid, its trailing spacing in; the strip seen is centred.
        let (hw, hh) = measures.get(&key(&["mediaviewer", "help"])).map_or((0.0, 0.0), |m| ((m.1 - spacing).max(0.0), m.2));
        out.push(placed("help", (w - hw) / 2.0, strip_y + (strip - hh) / 2.0, hw, hh, 1.0, help));
        out
    }
}

// A record of the viewer's, its id from its name.
fn placed(name: &str, x: f64, y: f64, w: f64, h: f64, opacity: f64, paint: Paint) -> Placed {
    Placed { id: key(&["mediaviewer", name]), kind: "mediaviewer".to_owned(), name: name.to_owned(), x, y, w, h, rotation: 0.0, rotation_origin: (0.5, 0.5), opacity, paint }
}
