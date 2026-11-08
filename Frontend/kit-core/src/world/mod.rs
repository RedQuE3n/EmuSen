//! The drawn world as data: bodies, the sky and Moonrise as vertices, colours and times, never pixels (EmuSen_Qt.md §1.4 and §10.4, EmuSen_Kit.md §4).

mod body;
mod moonrise;
mod random;
mod shape;
mod sky;

pub use body::{WorldBody, WorldRing, WorldShape, colours, facet_inks, geometry, glow, size_of, tallness, unit_box};
pub use moonrise::{
    CAPTION, CAPTION_SIZE, CAPTION_Y, Frame, HALO, HALO_EDGE, LINE, LIT_INNER, LIT_OUTER, Line, MOON_RADIUS, MOON_Y, Moon, OUTLINE,
    PlanetFrame, ROW_MOST, ROW_SHARE, ROW_Y, SHADOW, SWEEP, Span, Splash, SplashBody, Sweep, TITLE, TITLE_SIZE, TITLE_Y, Timeline, UNIT,
    Words, body_box, ease, round_spacing,
};
pub use random::SeededRandom;
pub use shape::{Box2, ELLIPSE_SEGMENTS, Paint, Point, Shape, Stroke, arc, circle, ellipse, into_box, quadratic, transform};
pub use sky::{
    DESIGN_HEIGHT, DRAFT_TURN, GLOW_RADII, GLOW_STRENGTH, Glow, HEIGHT_SHARE, MOONRISE_SKY, SCREEN_STARS_OPACITY, SCREENS_SKY, STAR_COLOUR,
    STAR_SIZE, SkyGlow, Star, StarPoint, glow_bounds, glow_stops, normal_hue, place_stars, star_points, twinkle, wrap_hue,
};
