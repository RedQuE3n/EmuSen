//! The system and gamelist views and the move between them, instant or a vertical pan as the theme's transition profile
//! says; the help bar, clock and status stay still while the views pan beneath them (EmuSen_BigPicture.md §14.7).

use crate::loader::Animation;

use super::data::Data;
use super::motion::{Glide, Time};
use super::view::{ElementState, SceneView};

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
}

// The elements that stay on the screen while the views pan.
fn overlay(kind: &str) -> bool {
    matches!(kind, "helpsystem" | "clock" | "systemstatus")
}

impl Stage {
    pub fn new(data: Data, view: &str, now: Time) -> Self {
        Stage { current: SceneView::new(data, view, now), leaving: None, slide: Glide::at(0.0), direction: 0.0 }
    }

    pub fn is_moving(&self) -> bool {
        self.leaving.is_some() || self.current.is_moving()
    }

    /// Opens the other view with the current selection: the gamelist from the system view, or back.
    pub fn switch(&mut self, now: Time) {
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
        if let Some(leaving) = &self.leaving {
            let elements = leaving.frame().into_iter().filter(|e| !overlay(&e.kind)).collect();
            layers.push(Layer { view: leaving.view.clone(), y: -self.direction * p * h, elements });
        }
        let mut elements = self.current.frame();
        for e in elements.iter_mut().filter(|e| overlay(&e.kind)) {
            e.rect.y -= y;
        }
        layers.push(Layer { view: self.current.view.clone(), y, elements });
        layers
    }
}
