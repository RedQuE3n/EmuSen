use std::time::Duration;

use super::random::SeededRandom;
use super::shape::{Box2, Point};
use crate::motion::{Curve, Glide, MotionToken};
use crate::tokens::Colour;

/// The design height the glow's radii and the stars' size are measured in on the night screens.
pub const DESIGN_HEIGHT: f64 = 800.0;

/// A night screen's star against the splash's.
pub const STAR_SIZE: f64 = 0.7;

/// A night screen's stars' opacity, held still.
pub const SCREEN_STARS_OPACITY: f64 = 0.85;

/// The share of the height the stars are spread over.
pub const HEIGHT_SHARE: f64 = 0.92;

/// The draft's turn: it placed phases and orbits with 6.28, not tau, and the same number keeps its frames comparable.
#[allow(clippy::approx_constant)]
pub const DRAFT_TURN: f64 = 6.28;

/// The splash's sky, top to bottom.
pub const MOONRISE_SKY: [(f64, Colour); 3] = [(0.0, Colour::hex("#05060F")), (0.7, Colour::hex("#0B1029")), (1.0, Colour::hex("#141B45"))];

/// The night screens' sky, top to bottom.
pub const SCREENS_SKY: [(f64, Colour); 3] = [(0.0, Colour::hex("#05060F")), (0.55, Colour::hex("#0A0D22")), (1.0, Colour::hex("#141A3C"))];

pub const STAR_COLOUR: Colour = Colour::hex("#DFE6FF");

/// A star: its place as shares of the width and height, its radius in units, and its twinkle's phase and rate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Star {
    pub x: f64,
    pub y: f64,
    pub r: f64,
    pub phase: f64,
    pub rate: f64,
}

/// Five draws a star, in Moonrise's order: x, y, radius, phase, twinkle rate (LunaP §221.1).
pub fn place_stars(random: &mut SeededRandom, count: usize) -> Vec<Star> {
    (0..count)
        .map(|_| Star {
            x: random.draw(),
            y: random.draw() * HEIGHT_SHARE,
            r: random.draw() * 1.2 + 0.2,
            phase: random.draw() * DRAFT_TURN,
            rate: 0.6 + random.draw() * 1.6,
        })
        .collect()
}

/// A star's brightness at a time: 0.55 to 1, a sine of its own phase and rate.
pub fn twinkle(star: &Star, time: f64) -> f64 {
    0.55 + 0.45 * (star.phase + time * star.rate).sin()
}

/// One star as drawn: a disc at a place with a radius and an opacity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StarPoint {
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub opacity: f64,
}

/// The stars of a frame into a host's buffer, so a frame allocates nothing: `size` is the screen's width and height, `unit` the height over the design height, `scale` a factor on every radius.
pub fn star_points(stars: &[Star], size: Point, unit: f64, scale: f64, opacity: f64, time: f64, out: &mut Vec<StarPoint>) {
    out.clear();
    if opacity <= 0.0 {
        return;
    }
    out.extend(stars.iter().map(|s| StarPoint {
        x: s.x * size[0],
        y: s.y * size[1],
        radius: s.r * unit * scale,
        opacity: opacity * twinkle(s, time),
    }));
}

/// The glow's radii on a night screen, in units of the height over 800.
pub const GLOW_RADII: (f64, f64) = (756.0, 450.0);

/// The glow's alpha at its centre.
pub const GLOW_STRENGTH: f64 = 0.2;

/// The soft glow's stops for a hue at an alpha: its centre, a dimmer middle, and clear from three quarters out (LunaP §221.2).
pub fn glow_stops(hue: f64, alpha: f64) -> [(f64, Colour); 4] {
    [
        (0.0, Colour::hsl(hue, 0.7, 0.5, alpha)),
        (0.45, Colour::hsl(hue, 0.7, 0.4, alpha * 0.35)),
        (0.75, Colour::hsl(hue, 0.7, 0.4, 0.0)),
        (1.0, Colour::hsl(hue, 0.7, 0.4, 0.0)),
    ]
}

/// The glow's box for a centre given as shares of the width and height, its radii scaled by the height over 800.
pub fn glow_bounds(centre: Point, radii: (f64, f64), w: f64, h: f64) -> Box2 {
    let u = h / DESIGN_HEIGHT;
    let (rx, ry) = (radii.0 * u, radii.1 * u);
    [centre[0] * w - rx, centre[1] * h - ry, 2.0 * rx, 2.0 * ry]
}

/// A hue difference folded to the shorter way round, -180 to 180.
pub fn wrap_hue(difference: f64) -> f64 {
    (difference.rem_euclid(360.0) + 540.0).rem_euclid(360.0) - 180.0
}

pub fn normal_hue(hue: f64) -> f64 {
    hue.rem_euclid(360.0)
}

/// The glow as drawn at an instant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glow {
    pub hue: f64,
    /// The alpha at the centre, the strength times how far the glow has come in.
    pub alpha: f64,
    pub centre: Point,
}

/// One soft glow whose hue follows the focus, easing the shorter way round over Pan, on the host's clock (LunaP §221.2).
#[derive(Clone, Debug)]
pub struct SkyGlow {
    hue: Option<f64>,
    shown_hue: Glide,
    strength: Glide,
    x: Glide,
    y: Glide,
    pub radii: (f64, f64),
    pub glow_strength: f64,
}

impl Default for SkyGlow {
    fn default() -> Self {
        SkyGlow::new()
    }
}

impl SkyGlow {
    /// No glow, centred.
    pub fn new() -> SkyGlow {
        SkyGlow {
            hue: None,
            shown_hue: Glide::still(0.0),
            strength: Glide::still(0.0),
            x: Glide::still(0.5),
            y: Glide::still(0.5),
            radii: GLOW_RADII,
            glow_strength: GLOW_STRENGTH,
        }
    }

    fn span() -> (Duration, Curve) {
        (MotionToken::Pan.nominal(), Curve::CubicOut)
    }

    /// The hue to glow in, or none for no glow; a change eases the shorter way round over Pan.
    pub fn set_hue(&mut self, hue: Option<f64>, now: Duration) {
        if hue == self.hue {
            return;
        }
        let (span, curve) = Self::span();
        match hue {
            Some(to) => {
                if self.hue.is_none() && self.strength.value_at(now, false) <= 0.0 {
                    self.shown_hue = Glide::still(to);
                } else {
                    let from = self.shown_hue.value_at(now, false);
                    self.shown_hue = self.shown_hue.toward(from + wrap_hue(to - from), now, span, curve);
                }
                self.strength = self.strength.toward(1.0, now, span, curve);
            }
            None => self.strength = self.strength.toward(0.0, now, span, curve),
        }
        self.hue = hue;
    }

    /// The centre as shares of the width and height; a change eases with the hue.
    pub fn set_centre(&mut self, centre: Point, now: Duration) {
        let (span, curve) = Self::span();
        self.x = self.x.toward(centre[0], now, span, curve);
        self.y = self.y.toward(centre[1], now, span, curve);
    }

    /// Puts the centre at once, with no easing, as a screen first shown does.
    pub fn place(&mut self, centre: Point) {
        self.x = Glide::still(centre[0]);
        self.y = Glide::still(centre[1]);
    }

    /// The glow at a time, or none while nothing shows.
    pub fn at(&self, now: Duration) -> Option<Glow> {
        let alpha = self.strength.value_at(now, false) * self.glow_strength;
        if alpha <= 0.0 {
            return None;
        }
        Some(Glow {
            hue: normal_hue(self.shown_hue.value_at(now, false)),
            alpha,
            centre: [self.x.value_at(now, false), self.y.value_at(now, false)],
        })
    }

    /// The hue drawn now, or none while no glow shows.
    pub fn shown_hue(&self, now: Duration) -> Option<f64> {
        (self.strength.value_at(now, false) > 0.0 || self.hue.is_some()).then(|| normal_hue(self.shown_hue.value_at(now, false)))
    }

    pub fn is_settled(&self, now: Duration) -> bool {
        self.shown_hue.is_settled(now, false)
            && self.strength.is_settled(now, false)
            && self.x.is_settled(now, false)
            && self.y.is_settled(now, false)
    }
}
