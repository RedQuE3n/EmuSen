//! The system and gamelist views and the move between them, instant, a vertical pan or a fade through black as the
//! theme's transition profile says; the help bar, clock and status stay still while the views pan beneath them
//! (EmuSen_BigPicture.md §14.7; EmuSen_Qt.md §4.8.8).

use crate::loader::Animation;

use super::data::Data;
use super::draw::{Drawn, Measures, Transit};
use super::fade::{self, ViewFade};
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
    /// A fade through black between views, while it runs; `leaving` is the view shown until the black covers it.
    fade: Option<ViewFade>,
}

/// The elements ES-DE draws over its views, apart from them: the help, the clock and the status.
pub(super) fn overlay(kind: &str) -> bool {
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
            fade: None,
        }
    }

    /// The program's first view, faded in from black when the theme's profile fades at startup (ES-DE's
    /// `goToSystemView` and `goToGamelist` from `ViewMode::NOTHING`).
    pub fn opening(data: Data, view: &str, now: Time) -> Self {
        let mut stage = Stage::new(data, view, now);
        let profile = &stage.current.data.system().theme.transitions;
        let animation = if view == "gamelist" { profile.startup_to_gamelist } else { profile.startup_to_system };
        if animation == Animation::Fade {
            stage.fade = Some(ViewFade::new(now, 0.0, &stage.current.motion));
        }
        stage.advance(now);
        stage
    }

    /// Another system's gamelist from a gamelist, as ES-DE's quick system select opens it: a fade when the profile's
    /// gamelistToGamelist fades, else at once, keeping the painter's measurements.
    pub fn open_gamelist(&mut self, data: Data, now: Time) {
        let animation = self.current.data.system().theme.transitions.gamelist_to_gamelist;
        let next = SceneView::new(data, "gamelist", now);
        self.viewer = None;
        let previous = std::mem::replace(&mut self.current, next);
        self.slide = Glide::at(0.0);
        self.fade_from(previous, animation, now);
        self.advance(now);
    }

    // A fade from the view on the screen when the profile fades, else none; a fade begun over another starts from its black.
    fn fade_from(&mut self, previous: SceneView, animation: Animation, now: Time) {
        let running = self.fade.take();
        if animation != Animation::Fade {
            self.leaving = None;
            return;
        }
        let black = running.and_then(|f| f.black(now)).unwrap_or(0.0);
        let still_leaving = running.is_some_and(|f| f.shows_leaving(now));
        if !still_leaving {
            self.leaving = Some(previous);
        }
        self.fade = Some(ViewFade::new(now, black, &self.current.motion));
    }

    /// A new view over new data that keeps the painter's measurements, as a reload, a resize or another system asks (EmuSen_Qt.md §4.8.5).
    pub fn restart(&mut self, data: Data, view: &str, now: Time) {
        let measures = std::mem::take(&mut self.measures);
        *self = Stage { measures, ..Stage::new(data, view, now) };
    }

    pub fn is_moving(&self) -> bool {
        self.leaving.is_some() || self.fade.is_some() || self.current.is_moving()
    }

    // The view leaving in a pan, not in a fade.
    fn panning(&self) -> Option<&SceneView> {
        self.leaving.as_ref().filter(|_| self.fade.is_none())
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
        if animation == Animation::Slide && previous.motion.view_slide > 0 {
            self.fade = None;
            self.direction = if to_gamelist { 1.0 } else { -1.0 };
            self.slide = Glide::new(0.0, 1.0, now, previous.motion.view_slide, previous.motion.view_slide_easing);
            self.leaving = Some(previous);
        } else {
            self.slide = Glide::at(0.0);
            self.fade_from(previous, animation, now);
        }
        self.advance(now);
    }

    pub fn advance(&mut self, now: Time) {
        self.current.advance(now);
        match self.fade {
            Some(f) if f.black(now).is_none() => {
                self.fade = None;
                self.leaving = None;
            }
            Some(f) if !f.shows_leaving(now) => self.leaving = None,
            Some(_) => {}
            None if self.slide.is_settled_at(now) => self.leaving = None,
            None => {}
        }
    }

    /// What is on the screen now: the leaving view while a pan runs, then the current one, each with its shift.
    pub fn layers(&self) -> Vec<Layer> {
        let now = self.current.now();
        let mut layers = vec![];
        let h = self.current.data.height;
        let p = self.slide.value_at(now);
        let y = if self.panning().is_some() { self.direction * (1.0 - p) * h } else { 0.0 };
        let transit = |v: &SceneView, e: &ElementState| Transit::of(&v.theme_view().elements[e.index]);
        if let Some(leaving) = self.panning() {
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
        let panning = self.panning().is_some();
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
        if self.panning().is_some() {
            (-self.direction * p * h, self.direction * (1.0 - p) * h)
        } else {
            (0.0, 0.0)
        }
    }

    fn drawn(&self, measures: &Measures) -> (Vec<Drawn>, Frame) {
        if let Some(f) = self.fade {
            return self.faded(f, measures);
        }
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

    // A fade between views: the view left until the black covers it, then the new one, under the black; the clock and
    // status over it, as ES-DE's Window draws them after the views (ViewController.cpp 1336-1386, Window.cpp 715-735).
    fn faded(&self, f: ViewFade, measures: &Measures) -> (Vec<Drawn>, Frame) {
        let now = self.current.now();
        let mut frame = Frame::default();
        let drawn = if f.shows_leaving(now) {
            // At startup nothing is in front of the camera yet.
            self.leaving.as_ref().map_or(vec![], |leaving| {
                let (drawn, primary) = leaving.draw(measures, true);
                frame.leaving_primary = primary;
                drawn
            })
        } else {
            let (drawn, primary) = self.current.draw(measures, false);
            frame.primary = primary;
            drawn
        };
        let over = |d: &Drawn| matches!(d.placed.kind.as_str(), "clock" | "systemstatus");
        let (mut all, above): (Vec<Drawn>, Vec<Drawn>) = drawn.into_iter().partition(|d| !over(d));
        let black = f.black(now).unwrap_or(0.0);
        let data = &self.current.data;
        all.push(Drawn { content: 0, placed: fade::veil("stage", data.width, data.height, black), wait: None, transit: Transit::default() });
        all.extend(above);
        (all, frame)
    }

    /// Everything on the screen at the stage's time, in window pixels (EmuSen_Qt.md §4.5.5).
    pub fn frame(&self) -> Frame {
        let (drawn, mut frame) = self.drawn(&self.measures);
        let waits: Vec<Time> = drawn.iter().filter_map(|d| d.wait).collect();
        frame.moving = self.leaving.is_some() || self.fade.is_some() || self.current.is_animating() || waits.contains(&0);
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
