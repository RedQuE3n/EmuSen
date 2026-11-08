use super::body::{WorldBody, WorldShape, tallness};
use super::random::SeededRandom;
use super::shape::{Box2, Point};
use super::sky::{DRAFT_TURN, Star, place_stars};
use crate::motion::{Curve, spring};
use crate::tokens::Colour;

/// A part of the timeline: when it starts and ends, in the draft's own seconds before the pace.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span {
    pub from: f64,
    pub to: f64,
}

impl Span {
    pub const fn new(from: f64, to: f64) -> Span {
        Span { from, to }
    }

    /// How far the part has gone at a time, 0 before it starts and 1 once it has finished.
    pub fn at(&self, t: f64) -> f64 {
        if self.to <= self.from {
            return if t >= self.to { 1.0 } else { 0.0 };
        }
        ((t - self.from) / (self.to - self.from)).clamp(0.0, 1.0)
    }
}

/// The splash's whole timing as data, in the draft's seconds; `pace` stretches them and `length` holds the finished frame (LunaP §219.2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timeline {
    pub pace: f64,
    pub length: f64,
    pub stars: Span,
    pub stroke: Span,
    pub halo: Span,
    pub fill: Span,
    pub appear: Span,
    pub appear_stagger: f64,
    pub settle: Span,
    pub settle_rate: f64,
    pub settle_stagger: f64,
    pub orbit_swing: f64,
    pub sweep: Span,
    pub flare_width: f64,
    pub line: Span,
    pub words: Span,
    pub star_count: usize,
    pub seed: u32,
    /// How long the host fades the splash out at its end; cut under reduced motion.
    pub fade_out: f64,
}

impl Timeline {
    /// Moonrise, approved 2026-11-04: 8 s, the draft's spans stretched by 1.55.
    pub const MOONRISE: Timeline = Timeline {
        pace: 1.55,
        length: 8.0,
        stars: Span::new(0.0, 0.9),
        stroke: Span::new(0.4, 1.6),
        halo: Span::new(0.9, 2.0),
        fill: Span::new(1.2, 2.0),
        appear: Span::new(1.15, 1.6),
        appear_stagger: 0.03,
        settle: Span::new(1.2, 2.7),
        settle_rate: 1.25,
        settle_stagger: 0.02,
        orbit_swing: 2.6,
        sweep: Span::new(2.35, 3.65),
        flare_width: 34.0,
        line: Span::new(2.4, 3.7),
        words: Span::new(3.45, 4.15),
        star_count: 220,
        seed: 0x4d6f_6f6e,
        fade_out: 0.15,
    };

    /// The draft's own time for a time since the splash began: clamped to the length, then divided by the pace.
    pub fn draft_time(&self, seconds: f64) -> f64 {
        seconds.clamp(0.0, self.length) / self.pace
    }
}

/// One planet of the row: its name, its hue, and the body a picker draws for it (none draws the plain planet).
#[derive(Clone, Debug, PartialEq)]
pub struct SplashBody {
    pub name: String,
    pub hue: f64,
    pub body: Option<WorldBody>,
}

/// The draft's unit: positions are in units of the height over 720.
pub const UNIT: f64 = 720.0;
pub const MOON_Y: f64 = 0.36;
pub const MOON_RADIUS: f64 = 92.0;
pub const ROW_Y: f64 = 0.66;
pub const ROW_SHARE: f64 = 0.78;
pub const ROW_MOST: f64 = 980.0;
pub const TITLE_Y: f64 = 0.85;
pub const TITLE_SIZE: f64 = 78.0;
pub const CAPTION_Y: f64 = 0.91;
pub const CAPTION_SIZE: f64 = 18.0;

pub const HALO: Colour = Colour::rgba(201, 214, 255, 71);
/// The halo's outer edge: its colour at no alpha.
pub const HALO_EDGE: Colour = Colour::rgba(201, 214, 255, 0);
pub const OUTLINE: Colour = Colour::rgba(243, 239, 226, 242);
pub const LIT_INNER: Colour = Colour::hex("#FAF7EC");
pub const LIT_OUTER: Colour = Colour::hex("#D6D2C4");
pub const SHADOW: Colour = Colour::hex("#070A1C");
pub const SWEEP: Colour = Colour::rgba(214, 226, 255, 140);
pub const LINE: Colour = Colour::rgba(159, 180, 255, 89);
pub const TITLE: Colour = Colour::hex("#E7E9F4");
pub const CAPTION: Colour = Colour::hex("#8B90AD");

/// The splash's easing: cubic out.
pub fn ease(v: f64) -> f64 {
    Curve::CubicOut.at(v)
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Orbit {
    a0: f64,
    rx: f64,
    ry: f64,
}

/// A splash placed from its seed: the stars and each planet's orbit, from which every frame is a pure function of time and size (LunaP §219.3).
#[derive(Clone, Debug)]
pub struct Splash {
    timeline: Timeline,
    bodies: Vec<SplashBody>,
    stars: Vec<Star>,
    orbits: Vec<Orbit>,
}

/// The moon at a time: its place, its halo, how much of the outline is drawn, how far it has filled, and where the shadow that cuts the crescent sits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Moon {
    pub centre: Point,
    pub radius: f64,
    pub halo_opacity: f64,
    /// The halo is HALO out to `halo_inner_radius` and fades to nothing at `halo_radius`, the draft's gradient from 0.6 to 3.2 radii.
    pub halo_inner_radius: f64,
    pub halo_radius: f64,
    pub outline_drawn: f64,
    pub outline_width: f64,
    pub fill: f64,
    /// The lit face's two-circle gradient: LIT_INNER from a small circle up and to the left, LIT_OUTER at 1.1 radii about the centre.
    pub lit_focus: Point,
    pub lit_focus_radius: f64,
    pub lit_radius: f64,
    pub shadow_centre: Point,
    pub shadow_radius: f64,
}

impl Moon {
    /// The shadow as it is drawn, clipped to the moon: the outline of the two discs' overlap, `steps` points along each arc, into `out`.
    pub fn shadow_outline(&self, steps: usize, out: &mut Vec<Point>) {
        out.clear();
        let ([ax, ay], ar) = (self.centre, self.radius);
        let ([bx, by], br) = (self.shadow_centre, self.shadow_radius);
        let d = (bx - ax).hypot(by - ay);
        let arc = |out: &mut Vec<Point>, [cx, cy]: Point, r: f64, from: f64, to: f64| {
            for i in 0..=steps {
                let a = from + (to - from) * i as f64 / steps as f64;
                out.push([cx + r * a.cos(), cy + r * a.sin()]);
            }
        };
        if ar <= 0.0 || br <= 0.0 || d >= ar + br {
            return;
        }
        if d <= (ar - br).abs() {
            let (centre, r) = if br < ar { (self.shadow_centre, br) } else { (self.centre, ar) };
            arc(out, centre, r, 0.0, std::f64::consts::TAU);
            return;
        }
        let towards = (by - ay).atan2(bx - ax);
        let on_moon = ((d * d + ar * ar - br * br) / (2.0 * d * ar)).clamp(-1.0, 1.0).acos();
        let on_shadow = ((d * d + br * br - ar * ar) / (2.0 * d * br)).clamp(-1.0, 1.0).acos();
        arc(out, self.centre, ar, towards - on_moon, towards + on_moon);
        let back = towards + std::f64::consts::PI;
        arc(out, self.shadow_centre, br, back - on_shadow, back + on_shadow);
    }
}

/// One planet at a time: where it is, how large, how far it has appeared, its glow, and its flare as the sweep passes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlanetFrame {
    pub index: usize,
    pub centre: Point,
    pub radius: f64,
    pub opacity: f64,
    pub glow_radius: f64,
    pub glow_colour: Colour,
    pub flare: f64,
    pub flare_radius: f64,
    pub flare_opacity: f64,
    pub flare_colour: Colour,
    /// The box the body is drawn in about its centre: a disc of the radius, shards and a gem 2.5 radii tall.
    pub body_box: Box2,
}

/// The light sweeping along the row: its centre and the band drawn round it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sweep {
    pub x: f64,
    pub band: Box2,
}

/// The row's line growing from the centre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line {
    pub from_x: f64,
    pub to_x: f64,
    pub y: f64,
    pub opacity: f64,
}

/// The title and caption settling in: the title's letter spacing closes from 18 to 10 units and it rises 10 units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Words {
    pub opacity: f64,
    pub title_size: f64,
    pub title_spacing: f64,
    pub title_baseline_y: f64,
    pub caption_size: f64,
    pub caption_spacing: f64,
    pub caption_baseline_y: f64,
}

/// Everything drawn at one time, in the control's pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    /// The draft's time the frame is drawn for.
    pub draft_time: f64,
    pub unit: f64,
    pub stars_opacity: f64,
    pub moon: Moon,
    pub planets: Vec<PlanetFrame>,
    pub sweep: Option<Sweep>,
    pub line: Option<Line>,
    pub words: Option<Words>,
    pub finished: bool,
}

impl Splash {
    /// Places the stars and the orbits from the timeline's seed, in the draft's order.
    pub fn new(timeline: Timeline, bodies: Vec<SplashBody>) -> Splash {
        let mut random = SeededRandom::new(timeline.seed);
        let stars = place_stars(&mut random, timeline.star_count);
        let orbits = bodies
            .iter()
            .map(|_| Orbit { a0: random.draw() * DRAFT_TURN, rx: 0.32 + random.draw() * 0.16, ry: 0.12 + random.draw() * 0.1 })
            .collect();
        Splash { timeline, bodies, stars, orbits }
    }

    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    pub fn bodies(&self) -> &[SplashBody] {
        &self.bodies
    }

    pub fn stars(&self) -> &[Star] {
        &self.stars
    }

    /// The row's width at a size: the lesser of 78% of the width and 980 units.
    pub fn row_width(w: f64, h: f64) -> f64 {
        (w * ROW_SHARE).min(ROW_MOST * h / UNIT)
    }

    /// A planet's radius: four sizes in turn, 1.25 times the draft's.
    pub fn planet_radius(index: usize, unit: f64) -> f64 {
        (5.0 + (index % 4) as f64 * 1.6) * unit * 1.25
    }

    /// Where along the sweep a planet's place in the row is, 0 to 1.
    fn sweep_share(index: usize, count: usize, row_w: f64, unit: f64) -> f64 {
        let along = if count == 1 { row_w / 2.0 } else { row_w * index as f64 / (count - 1) as f64 };
        (along + 60.0 * unit) / (row_w + 120.0 * unit)
    }

    /// The sweep's centre: from 60 units left of the row to 60 right of it, at a steady speed.
    fn sweep_x(sweep: f64, w: f64, row_w: f64, unit: f64) -> f64 {
        w / 2.0 - row_w / 2.0 - 60.0 * unit + (row_w + 120.0 * unit) * sweep
    }

    /// A planet's resting place in the row.
    fn slot(index: usize, count: usize, w: f64, row_w: f64) -> f64 {
        if count == 1 { w / 2.0 } else { w / 2.0 - row_w / 2.0 + row_w * index as f64 / (count - 1) as f64 }
    }

    /// A planet's centre at a draft time, from its orbit into its place in the row by a damped spring.
    fn planet_at(&self, index: usize, t: f64, w: f64, h: f64, row_w: f64) -> Point {
        let tl = &self.timeline;
        let (mx, my) = (w / 2.0, h * MOON_Y);
        let local = (tl.settle.at(t) * tl.settle_rate - index as f64 * tl.settle_stagger).clamp(0.0, 1.0);
        let orbit = self.orbits[index];
        let angle = orbit.a0 + (1.0 - ease(local)) * tl.orbit_swing;
        let (ox, oy) = (mx + angle.cos() * w * orbit.rx, my + angle.sin() * h * orbit.ry);
        let (tx, ty) = (Self::slot(index, self.orbits.len(), w, row_w), h * ROW_Y);
        let k = spring(local);
        [ox + (tx - ox) * k, oy + (ty - oy) * k]
    }

    /// When the sweep reaches each planet's place at a size, in seconds since the splash began: the times its bells ring (LunaP §219.4).
    pub fn strike_times(&self, w: f64, h: f64) -> Vec<f64> {
        if h <= 0.0 {
            return vec![0.0; self.bodies.len()];
        }
        let unit = h / UNIT;
        let row_w = Self::row_width(w, h);
        let tl = &self.timeline;
        (0..self.bodies.len())
            .map(|i| (tl.sweep.from + Self::sweep_share(i, self.bodies.len(), row_w, unit) * (tl.sweep.to - tl.sweep.from)) * tl.pace)
            .collect()
    }

    /// The frame for a time since the splash began at a size: a pure function of the timeline, the bodies, the time and the size.
    pub fn frame(&self, seconds: f64, w: f64, h: f64) -> Frame {
        let tl = &self.timeline;
        let t = tl.draft_time(seconds);
        let unit = h / UNIT;
        let (mx, my, mr) = (w / 2.0, h * MOON_Y, MOON_RADIUS * unit);
        let fill = ease(tl.fill.at(t));
        let moon = Moon {
            centre: [mx, my],
            radius: mr,
            halo_opacity: ease(tl.halo.at(t)),
            halo_inner_radius: mr * 0.6,
            halo_radius: mr * 3.2,
            outline_drawn: ease(tl.stroke.at(t)),
            outline_width: 2.2 * unit,
            fill,
            lit_focus: [mx - mr * 0.35, my - mr * 0.3],
            lit_focus_radius: mr * 0.1,
            lit_radius: mr * 1.1,
            shadow_centre: [mx + mr * (0.42 + (1.0 - fill) * 0.7), my - mr * 0.12],
            shadow_radius: mr * 0.96,
        };
        let row_y = h * ROW_Y;
        let row_w = Self::row_width(w, h);
        let sweep = tl.sweep.at(t);
        let sweep_x = Self::sweep_x(sweep, w, row_w, unit);
        let sweeping = sweep > 0.0 && sweep < 1.0;
        let planets = (0..self.bodies.len())
            .filter_map(|i| {
                let appear = ease(tl.appear.at(t - i as f64 * tl.appear_stagger));
                if appear <= 0.0 {
                    return None;
                }
                let centre = self.planet_at(i, t, w, h, row_w);
                let r = Self::planet_radius(i, unit);
                let hue = self.bodies[i].hue;
                let hit = if sweeping && unit > 0.0 { (-((centre[0] - sweep_x) / (tl.flare_width * unit)).powi(2)).exp() } else { 0.0 };
                let shape = self.bodies[i].body.map_or(WorldShape::Planet, |b| b.shape);
                Some(PlanetFrame {
                    index: i,
                    centre,
                    radius: r,
                    opacity: appear,
                    glow_radius: r * 2.6,
                    glow_colour: Colour::hsl(hue, 0.8, 0.7, 0.18),
                    flare: hit,
                    flare_radius: r * (2.0 + 2.2 * hit),
                    flare_opacity: 0.55 * hit,
                    flare_colour: Colour::hsl(hue, 0.9, 0.86, 1.0),
                    body_box: body_box(shape, centre, r),
                })
            })
            .collect();
        let line = ease(tl.line.at(t));
        let word = ease(tl.words.at(t));
        Frame {
            draft_time: t,
            unit,
            stars_opacity: ease(tl.stars.at(t)) * 0.9,
            moon,
            planets,
            sweep: sweeping.then_some(Sweep { x: sweep_x, band: [sweep_x - 90.0 * unit, row_y - 26.0 * unit, 180.0 * unit, 52.0 * unit] }),
            line: (line > 0.0).then(|| Line {
                from_x: w / 2.0 - row_w / 2.0 * line,
                to_x: w / 2.0 + row_w / 2.0 * line,
                y: row_y,
                opacity: line,
            }),
            words: (word > 0.0).then(|| Words {
                opacity: word,
                title_size: TITLE_SIZE * unit,
                title_spacing: round_spacing((18.0 - 8.0 * word) * unit),
                title_baseline_y: h * TITLE_Y + (1.0 - word) * 10.0 * unit,
                caption_size: CAPTION_SIZE * unit,
                caption_spacing: round_spacing(6.0 * unit),
                caption_baseline_y: h * CAPTION_Y,
            }),
            finished: seconds >= tl.length,
        }
    }
}

/// The box a body is drawn in about its centre: a planet's disc of radius r, shards and a gem 2.5 r tall (LunaP §222.6).
pub fn body_box(shape: WorldShape, centre: Point, r: f64) -> Box2 {
    let h = if shape == WorldShape::Planet { r / 0.44 } else { 2.5 * r };
    let w = h / tallness(shape);
    let top = if shape == WorldShape::Planet { h / 2.0 } else { h * 80.0 / 160.0 };
    [centre[0] - w / 2.0, centre[1] - top, w, h]
}

/// Letter spacing rounded to a quarter unit, so a layout is made again only when the rounded value moves.
pub fn round_spacing(v: f64) -> f64 {
    (v * 4.0).round() / 4.0
}
