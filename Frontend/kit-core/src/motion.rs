//! Motion as data: the tokens' durations and curves, and reduced motion (EmuSen_Interface.md §5.5, LunaP §206.5).

use std::time::Duration;

/// What kind of change a movement is, which fixes its duration and curve.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MotionToken {
    /// No animation: a list's cursor, the help bar's change.
    Instant,
    /// 120 ms, quadratic out: the ring between tiles, a tile's scale, a stepped value.
    Quick,
    /// 200 ms, quadratic out: a card or a pane arriving.
    Enter,
    /// 160 ms, quadratic in: a card or a pane leaving.
    Exit,
    /// 320 ms, cubic out: moving between screens.
    Pan,
    /// 400 ms, quadratic out: a carousel or a shelf stepping, as ES-DE's own.
    Reel,
    /// 150 ms, linear: facts fading while a list scrolls.
    Fade,
    /// 1.5 s before a hero picture turns into its clip, which then cross-fades over Enter.
    Poster,
}

/// An easing curve, evaluated on a progress of 0 to 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Curve {
    Linear,
    QuadraticIn,
    QuadraticOut,
    CubicOut,
}

impl Curve {
    /// The eased progress for a linear one, both clamped to 0..=1.
    pub fn at(self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Curve::Linear => t,
            Curve::QuadraticIn => t * t,
            Curve::QuadraticOut => 1.0 - (1.0 - t) * (1.0 - t),
            Curve::CubicOut => 1.0 - (1.0 - t).powi(3),
        }
    }
}

/// How long a fade lasts while motion is reduced.
pub const REDUCED_FADE: Duration = Duration::from_millis(120);

/// How long the focus waits idle before its ring starts to breathe.
pub const BREATH_DELAY: Duration = Duration::from_secs(2);

/// One breath of the idle ring, full to dim and back.
pub const BREATH_PERIOD: Duration = Duration::from_millis(1600);

/// The idle ring's opacity at the bottom of a breath.
pub const BREATH_LOW: f64 = 0.7;

impl MotionToken {
    pub const ALL: [MotionToken; 8] = [
        MotionToken::Instant,
        MotionToken::Quick,
        MotionToken::Enter,
        MotionToken::Exit,
        MotionToken::Pan,
        MotionToken::Reel,
        MotionToken::Fade,
        MotionToken::Poster,
    ];

    /// The token's full duration, ignoring reduced motion.
    pub const fn nominal(self) -> Duration {
        Duration::from_millis(match self {
            MotionToken::Instant => 0,
            MotionToken::Quick => 120,
            MotionToken::Enter => 200,
            MotionToken::Exit => 160,
            MotionToken::Pan => 320,
            MotionToken::Reel => 400,
            MotionToken::Fade => 150,
            MotionToken::Poster => 1500,
        })
    }

    /// How long a movement of the token lasts: its nominal duration, or nothing while motion is reduced.
    pub const fn duration(self, reduced: bool) -> Duration {
        if reduced { Duration::ZERO } else { self.nominal() }
    }

    /// How long a fade of the token lasts: its nominal duration, or 120 ms while motion is reduced; nothing for Instant.
    pub const fn fade_duration(self, reduced: bool) -> Duration {
        match self {
            MotionToken::Instant => Duration::ZERO,
            _ if reduced => REDUCED_FADE,
            _ => self.nominal(),
        }
    }

    /// The token's curve.
    pub const fn curve(self) -> Curve {
        match self {
            MotionToken::Exit => Curve::QuadraticIn,
            MotionToken::Pan => Curve::CubicOut,
            MotionToken::Fade | MotionToken::Instant => Curve::Linear,
            _ => Curve::QuadraticOut,
        }
    }
}

/// A value moving from one number to another over a span of a host's clock, read at any time and never from wall time (LunaP §102.1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glide {
    pub from: f64,
    pub to: f64,
    pub start: Duration,
    pub duration: Duration,
    pub curve: Curve,
}

impl Glide {
    /// A value at rest.
    pub const fn still(value: f64) -> Glide {
        Glide { from: value, to: value, start: Duration::ZERO, duration: Duration::ZERO, curve: Curve::Linear }
    }

    pub fn end(&self) -> Duration {
        self.start + self.duration
    }

    /// The value at a time: the destination once the span has passed or while motion is reduced.
    pub fn value_at(&self, now: Duration, reduced: bool) -> f64 {
        if self.duration.is_zero() || now >= self.end() || reduced {
            return self.to;
        }
        if now <= self.start {
            return self.from;
        }
        let t = (now - self.start).as_secs_f64() / self.duration.as_secs_f64();
        self.from + (self.to - self.from) * self.curve.at(t)
    }

    pub fn is_settled(&self, now: Duration, reduced: bool) -> bool {
        now >= self.end() || reduced
    }

    /// A fresh glide from wherever this one is now to a new destination.
    pub fn toward(&self, to: f64, now: Duration, duration: Duration, curve: Curve) -> Glide {
        Glide { from: self.value_at(now, false), to, start: now, duration, curve }
    }
}

/// The splash's damped spring, 1 - cos(2.4 pi v) e^(-5 v): a planet settles 0.2% short of its place, as the draft drew it (LunaP §219.5).
pub fn spring(v: f64) -> f64 {
    let v = v.clamp(0.0, 1.0);
    1.0 - (v * std::f64::consts::PI * 2.4).cos() * (-5.0 * v).exp()
}

/// Whether the idle focus ring breathes: not while motion is reduced (WCAG 2.3.3).
pub const fn breathes(reduced: bool) -> bool {
    !reduced
}

/// The idle ring's opacity at a time since the focus last moved: 1 until the delay, then a breath down to BREATH_LOW and back each period.
pub fn breath_opacity(since_focus: Duration, reduced: bool) -> f64 {
    if reduced || since_focus < BREATH_DELAY {
        return 1.0;
    }
    let phase = (since_focus - BREATH_DELAY).as_secs_f64() / BREATH_PERIOD.as_secs_f64();
    let dip = (1.0 - (phase * std::f64::consts::TAU).cos()) / 2.0;
    1.0 - (1.0 - BREATH_LOW) * dip
}
