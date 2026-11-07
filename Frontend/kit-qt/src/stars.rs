//! The sky's state with no Qt: kit-core's stars placed once, their points each tick into one buffer, and the one glow (EmuSen_Kit.md §7.5).

use std::time::Duration;

use emusen_kit_core::tokens::Colour;
use emusen_kit_core::world::{self, Box2, Glow, Point, SkyGlow, Star, StarPoint, GLOW_RADII, MOONRISE_SKY, SCREENS_SKY, STAR_SIZE};

/// The glow as drawn: its box on the screen and its four stops.
#[derive(Clone, Debug, PartialEq)]
pub struct GlowDrawn {
    pub glow: Glow,
    pub bounds: Box2,
    pub stops: [(f64, Colour); 4],
}

#[derive(Clone, Debug)]
pub struct SkyState {
    stars: Vec<Star>,
    pub points: Vec<StarPoint>,
    glow: SkyGlow,
    moonrise: bool,
}

impl SkyState {
    pub fn new(seed: u32, count: usize, moonrise: bool) -> SkyState {
        let mut random = world::SeededRandom::new(seed);
        let stars = world::place_stars(&mut random, count);
        SkyState { points: Vec::with_capacity(stars.len()), stars, glow: SkyGlow::new(), moonrise }
    }

    pub fn stars(&self) -> &[Star] {
        &self.stars
    }

    /// The gradient's three stops: Moonrise's sky or the screens'.
    pub fn stops(&self) -> [(f64, Colour); 3] {
        if self.moonrise {
            MOONRISE_SKY
        } else {
            SCREENS_SKY
        }
    }

    /// The stars at a time for a screen size, into the buffer; the unit is the height over the design height.
    pub fn tick(&mut self, time: f64, size: Point, opacity: f64) {
        let unit = size[1] / world::DESIGN_HEIGHT;
        world::star_points(&self.stars, size, unit, STAR_SIZE, opacity, time, &mut self.points);
    }

    pub fn set_glow_hue(&mut self, hue: Option<f64>, now: Duration) {
        self.glow.set_hue(hue, now);
    }

    pub fn set_glow_centre(&mut self, centre: Point, now: Duration) {
        self.glow.set_centre(centre, now);
    }

    pub fn glow_at(&self, now: Duration, size: Point) -> Option<GlowDrawn> {
        let glow = self.glow.at(now)?;
        let bounds = world::glow_bounds(glow.centre, GLOW_RADII, size[0], size[1]);
        Some(GlowDrawn { glow, bounds, stops: world::glow_stops(glow.hue, glow.alpha) })
    }
}
