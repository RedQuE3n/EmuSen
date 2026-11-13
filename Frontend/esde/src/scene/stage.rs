//! The system and gamelist views and the move between them, instant or a vertical pan as the theme's transition profile
//! says; the help bar, clock and status stay still while the views pan beneath them (EmuSen_BigPicture.md §14.7).

use crate::loader::Animation;

use super::data::Data;
use super::draw::{Drawn, Measures, Transit};
use super::frame::{Frame, Paint, Row, Rows};
use super::motion::{Glide, Time};
use super::view::{ElementState, SceneView};
use super::viewer::Viewer;

/// One view on the screen at a time: its elements and its vertical shift in pixels.
pub struct Layer {
    pub view: String,
    pub y: f64,
    pub elements: Vec<ElementState>,
}

pub struct Stage {
    pub current: SceneView,
    leaving: Option<SceneView>,
    slide: Glide,
    direction: f64,
    measures: Measures,
    /// ES-DE's media viewer over the gamelist, while it is open.
    pub viewer: Option<Viewer>,
}

// The elements that stay on the screen while the views pan.
fn overlay(kind: &str) -> bool {
    matches!(kind, "helpsystem" | "clock" | "systemstatus")
}

impl Stage {
    pub fn new(data: Data, view: &str, now: Time) -> Self {
        Stage {
            current: SceneView::new(data, view, now),
            leaving: None,
            slide: Glide::at(0.0),
            direction: 0.0,
            measures: Measures::new(),
            viewer: None,
        }
    }

    /// A new view over new data that keeps the painter's measurements, as a reload, a resize or another system asks (EmuSen_Qt.md §4.8.5).
    pub fn restart(&mut self, data: Data, view: &str, now: Time) {
        let measures = std::mem::take(&mut self.measures);
        *self = Stage { measures, ..Stage::new(data, view, now) };
    }

    pub fn is_moving(&self) -> bool {
        self.leaving.is_some() || self.current.is_moving()
    }

    /// The page count of the open viewer's manual, as the painter read it; whether it changed.
    pub fn manual_pages(&mut self, pages: usize) -> bool {
        self.viewer.as_mut().is_some_and(|v| v.set_pages(pages))
    }

    /// Opens the media viewer on the gamelist's game, as ES-DE's X does; nothing on a folder or a game with no media.
    pub fn open_viewer(&mut self) -> bool {
        self.viewer = (self.current.view == "gamelist").then(|| self.current.data.game().and_then(Viewer::of)).flatten();
        self.viewer.is_some()
    }

    /// Opens the other view with the current selection: the gamelist from the system view, or back.
    pub fn switch(&mut self, now: Time) {
        self.viewer = None;
        let to_gamelist = self.current.view == "system";
        let profile = &self.current.data.system().theme.transitions;
        let animation = if to_gamelist { profile.system_to_gamelist } else { profile.gamelist_to_system };
        let data = self.current.data.clone();
        let next = SceneView::new(data, if to_gamelist { "gamelist" } else { "system" }, now);
        let previous = std::mem::replace(&mut self.current, next);
        self.leaving = None;
        if animation == Animation::Slide && previous.motion.view_slide > 0 {
            self.direction = if to_gamelist { 1.0 } else { -1.0 };
            self.slide = Glide::new(0.0, 1.0, now, previous.motion.view_slide, previous.motion.view_slide_easing);
            self.leaving = Some(previous);
        }
        self.advance(now);
    }

    pub fn advance(&mut self, now: Time) {
        self.current.advance(now);
        if self.slide.is_settled_at(now) {
            self.leaving = None;
        }
    }

    /// What is on the screen now: the leaving view while a pan runs, then the current one, each with its shift.
    pub fn layers(&self) -> Vec<Layer> {
        let now = self.current.now();
        let mut layers = vec![];
        let h = self.current.data.height;
        let p = self.slide.value_at(now);
        let y = if self.leaving.is_some() { self.direction * (1.0 - p) * h } else { 0.0 };
        let transit = |v: &SceneView, e: &ElementState| Transit::of(&v.theme_view().elements[e.index]);
        if let Some(leaving) = &self.leaving {
            let leave_y = -self.direction * p * h;
            let elements = leaving
                .frame()
                .into_iter()
                .filter(|e| !overlay(&e.kind) && !transit(leaving, e).hidden)
                .map(|mut e| {
                    if transit(leaving, &e).stays {
                        e.rect.y -= leave_y;
                    }
                    e
                })
                .collect();
            layers.push(Layer { view: leaving.view.clone(), y: leave_y, elements });
        }
        let panning = self.leaving.is_some();
        let mut elements: Vec<ElementState> =
            self.current.frame().into_iter().filter(|e| !(panning && transit(&self.current, e).hidden)).collect();
        for e in elements.iter_mut().filter(|e| overlay(&e.kind) || (panning && transit(&self.current, e).stays)) {
            e.rect.y -= y;
        }
        layers.push(Layer { view: self.current.view.clone(), y, elements });
        layers
    }

    // The pan's progress and the two views' shifts: the leaving view's, then the current view's.
    fn shifts(&self) -> (f64, f64) {
        let h = self.current.data.height;
        let p = self.slide.value_at(self.current.now());
        if self.leaving.is_some() {
            (-self.direction * p * h, self.direction * (1.0 - p) * h)
        } else {
            (0.0, 0.0)
        }
    }

    fn drawn(&self, measures: &Measures) -> (Vec<Drawn>, Frame) {
        let (leave_y, y) = self.shifts();
        let mut frame = Frame::default();
        let mut all = vec![];
        if let Some(leaving) = &self.leaving {
            let (drawn, primary) = leaving.draw(measures, true);
            frame.leaving_primary = primary;
            all.extend(drawn.into_iter().filter(|d| !overlay(&d.placed.kind) && !d.transit.hidden).map(|mut d| {
                if !d.transit.stays {
                    d.placed.y += leave_y;
                }
                d
            }));
        }
        let panning = self.leaving.is_some();
        let (drawn, primary) = self.current.draw(measures, false);
        frame.primary = primary;
        all.extend(drawn.into_iter().filter(|d| !(panning && d.transit.hidden)).map(|mut d| {
            if !overlay(&d.placed.kind) && !(panning && d.transit.stays) {
                d.placed.y += y;
            }
            d
        }));
        if let Some(viewer) = &self.viewer {
            all.extend(viewer.placed(&self.current.data, measures).into_iter().map(|placed| Drawn { content: placed.id, placed, wait: None, transit: Transit::default() }));
        }
        (all, frame)
    }

    /// Everything on the screen at the stage's time, in window pixels (EmuSen_Qt.md §4.5.5).
    pub fn frame(&self) -> Frame {
        let (drawn, mut frame) = self.drawn(&self.measures);
        let waits: Vec<Time> = drawn.iter().filter_map(|d| d.wait).collect();
        frame.moving = self.leaving.is_some() || self.current.is_animating() || waits.contains(&0);
        let now = self.current.data.now;
        let clock = drawn.iter().any(|d| d.placed.kind == "clock").then(|| {
            let seconds = self.current.theme_view().of_kind("clock").any(|e| e.string("format").is_some_and(|f| f.contains("%S")));
            if seconds {
                1_000_000
            } else {
                (60 - now.second.min(59) as Time) * 1_000_000
            }
        });
        frame.next_change_ms =
            waits.iter().copied().filter(|w| *w > 0).chain(clock).min().map(|w| ((w + 999) / 1000).min(u32::MAX as Time) as u32);
        frame.overlay = self.current.scroll_letters();
        frame.elements = drawn.into_iter().map(|d| d.placed).collect();
        frame
    }

    /// The painter's measurement of an element's content, kept while that content is shown; true when the frame changes (EmuSen_Qt.md §4.5.5).
    pub fn measured(&mut self, id: u64, w: f64, h: f64) -> bool {
        let (drawn, _) = self.drawn(&Measures::new());
        let Some(d) = drawn.iter().find(|d| d.placed.id == id) else { return false };
        let entry = (d.content, w, h);
        if self.measures.get(&id) == Some(&entry) {
            return false;
        }
        // The same picture decoded at another width is its aspect again, to the pixel's rounding: the first stands, so a
        // box is the same share of every window (EmuSen_Qt.md §4.8.5).
        if let (Paint::Picture(_), Some(&(k, ow, oh))) = (&d.placed.paint, self.measures.get(&id)) {
            if k == d.content && ow > 0.0 && oh > 0.0 && ((oh * w / ow - h).abs() <= 1.0 || (ow * h / oh - w).abs() <= 1.0) {
                return false;
            }
        }
        self.measures.insert(id, entry);
        true
    }

    /// The rows a primary element shown now lists.
    pub fn rows(&self, rows: Rows) -> Option<Vec<Row>> {
        std::iter::once(&self.current).chain(self.leaving.as_ref()).find(|v| v.rows_id() == rows).map(|v| v.rows_of())
    }
}
