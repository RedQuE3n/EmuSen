//! How the scene moves, as ES-DE 3.4.1 was measured moving (EmuSen_BigPicture.md §14.6, §14.7): the clock's unit, the
//! easings, a value gliding between two numbers, a held direction's repeats, a self-scrolling text, and the durations.

/// A time on the host's clock in microseconds; the scene never reads wall time.
pub type Time = i64;

/// A time of so many milliseconds.
pub fn ms(milliseconds: f64) -> Time {
    (milliseconds * 1000.0).round() as Time
}

fn seconds(t: Time) -> f64 {
    t as f64 / 1_000_000.0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Easing {
    Linear,
    QuadraticOut,
    CubicOut,
    CubicInOut,
}

impl Easing {
    pub fn ease(self, p: f64) -> f64 {
        match self {
            Easing::Linear => p,
            Easing::QuadraticOut => -(p * (p - 2.0)),
            Easing::CubicOut => {
                let f = p - 1.0;
                f * f * f + 1.0
            }
            Easing::CubicInOut => {
                if p < 0.5 {
                    4.0 * p * p * p
                } else {
                    let f = 2.0 * p - 2.0;
                    0.5 * f * f * f + 1.0
                }
            }
        }
    }
}

/// A value moving from one number to another over a span of the host's clock (LunaP's Glide, §102.1 there).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glide {
    pub from: f64,
    pub to: f64,
    pub start: Time,
    pub duration: Time,
    pub easing: Easing,
}

impl Glide {
    pub fn at(value: f64) -> Self {
        Glide { from: value, to: value, start: 0, duration: 0, easing: Easing::Linear }
    }

    pub fn new(from: f64, to: f64, start: Time, duration: Time, easing: Easing) -> Self {
        Glide { from, to, start, duration, easing }
    }

    pub fn end(&self) -> Time {
        self.start + self.duration.max(0)
    }

    pub fn value_at(&self, now: Time) -> f64 {
        if self.duration <= 0 || now >= self.end() {
            return self.to;
        }
        if now <= self.start {
            return self.from;
        }
        let t = (now - self.start) as f64 / self.duration as f64;
        self.from + (self.to - self.from) * self.easing.ease(t)
    }

    pub fn is_settled_at(&self, now: Time) -> bool {
        now >= self.end()
    }

    pub fn toward(&self, to: f64, now: Time, duration: Time, easing: Easing) -> Self {
        Glide::new(self.value_at(now), to, now, duration, easing)
    }
}

/// A held direction's timing: the first repeat's delay, the interval, and from fast_after a faster interval, the switch moving fast_jump items at once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RepeatRule {
    pub delay: Time,
    pub interval: Time,
    pub fast_after: Option<Time>,
    pub fast_interval: Option<Time>,
    pub fast_jump: i32,
}

impl RepeatRule {
    pub fn new(delay: Time, interval: Time) -> Self {
        RepeatRule { delay, interval, fast_after: None, fast_interval: None, fast_jump: 1 }
    }

    pub fn fast(delay: Time, interval: Time, after: Time, fast: Time, jump: i32) -> Self {
        RepeatRule { delay, interval, fast_after: Some(after), fast_interval: Some(fast), fast_jump: jump }
    }
}

/// A held direction's repeats on the scene's clock (EmuSen_BigPicture.md §14.7).
#[derive(Clone, Debug)]
pub struct Repeat {
    rule: RepeatRule,
    direction: i32,
    fast: bool,
    since: Time,
    next: Time,
}

impl Default for Repeat {
    fn default() -> Self {
        Repeat { rule: RepeatRule::new(0, 0), direction: 0, fast: false, since: 0, next: 0 }
    }
}

impl Repeat {
    pub fn held(&self) -> bool {
        self.direction != 0
    }

    pub fn press(&mut self, direction: i32, now: Time, rule: RepeatRule) {
        self.rule = rule;
        self.direction = direction.signum();
        self.fast = false;
        self.since = now;
        self.next = now + rule.delay;
    }

    pub fn release(&mut self) {
        self.direction = 0;
    }

    /// The next repeat due by `now`, as a signed number of items and the time it fell due; the switch to the fast tier moves fast_jump items.
    pub fn next_due(&mut self, now: Time) -> Option<(i32, Time)> {
        if self.direction == 0 || self.rule.interval <= 0 {
            return None;
        }
        if let (false, Some(after), Some(fast)) = (self.fast, self.rule.fast_after, self.rule.fast_interval) {
            if self.since + after <= self.next && self.since + after <= now {
                self.fast = true;
                let at = self.since + after;
                self.next = at + fast;
                return Some((self.direction * self.rule.fast_jump.max(1), at));
            }
        }
        if self.next > now {
            return None;
        }
        let due = self.next;
        self.next = due + if self.fast { self.rule.fast_interval.unwrap_or(self.rule.interval) } else { self.rule.interval };
        Some((self.direction, due))
    }
}

/// A text that scrolls by itself: a line in a loop, or a column run to its end (LunaP's TextScroll, §102.3 there).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextScroll {
    pub delay: Time,
    /// Pixels a second.
    pub speed: f64,
    /// Horizontal: pixels between the end of one copy and the next.
    pub gap: f64,
    /// Vertical: how long the last lines stay shown.
    pub end_pause: Time,
    /// Vertical: how long the text takes to fade in at the top after each pass.
    pub fade_in: Time,
    pub whole_pixels: bool,
}

impl TextScroll {
    fn step(&self, offset: f64) -> f64 {
        if self.whole_pixels {
            (offset + 1e-9).floor()
        } else {
            offset
        }
    }

    /// A line's leftward offset at a time since it was shown, for a line `width` pixels wide.
    pub fn loop_offset(&self, elapsed: Time, width: f64) -> f64 {
        let period = width + self.gap.max(0.0);
        if self.speed <= 0.0 || period <= 0.0 || elapsed <= self.delay {
            return 0.0;
        }
        let delay = seconds(self.delay);
        let cycle = delay + period / self.speed;
        let into = seconds(elapsed) % cycle;
        if into <= delay {
            0.0
        } else {
            (period.min((into - delay) * self.speed)) % period
        }
    }

    /// A column's upward offset and opacity at a time, for a column that must travel `travel` pixels for its last line to show.
    pub fn run_at(&self, elapsed: Time, travel: f64) -> (f64, f64) {
        if self.speed <= 0.0 || travel <= 0.0 || elapsed <= self.delay {
            return (0.0, 1.0);
        }
        let (delay, fade, pause) = (seconds(self.delay), seconds(self.fade_in).max(0.0), seconds(self.end_pause).max(0.0));
        let moving = travel / self.speed;
        let t = seconds(elapsed);
        let first = delay + moving + pause;
        if t < first {
            return (self.step(travel.min((t - delay).max(0.0) * self.speed)), 1.0);
        }
        let cycle = fade + delay + moving + pause;
        let mut into = (t - first) % cycle;
        if into < fade {
            return (0.0, into / fade);
        }
        into -= fade;
        if into <= delay {
            return (0.0, 1.0);
        }
        (self.step(travel.min((into - delay) * self.speed)), 1.0)
    }

    /// How long until a line `width` pixels wide next moves, 0 while it moves; None when it never does.
    pub fn loop_wait(&self, elapsed: Time, width: f64) -> Option<Time> {
        let period = width + self.gap.max(0.0);
        if self.speed <= 0.0 || period <= 0.0 {
            return None;
        }
        if elapsed <= self.delay {
            return Some(self.delay - elapsed);
        }
        let (delay, cycle) = (seconds(self.delay), seconds(self.delay) + period / self.speed);
        let into = seconds(elapsed) % cycle;
        Some(if into <= delay { micros(delay - into) } else { 0 })
    }

    /// How long until a column that travels `travel` pixels next moves or fades, 0 while it does; None when it never does.
    pub fn run_wait(&self, elapsed: Time, travel: f64) -> Option<Time> {
        if self.speed <= 0.0 || travel <= 0.0 {
            return None;
        }
        if elapsed <= self.delay {
            return Some(self.delay - elapsed);
        }
        let (delay, fade, pause) = (seconds(self.delay), seconds(self.fade_in).max(0.0), seconds(self.end_pause).max(0.0));
        let moving = travel / self.speed;
        let t = seconds(elapsed);
        let first = delay + moving + pause;
        if t < delay + moving {
            return Some(0);
        }
        if t < first {
            return Some(micros(first - t));
        }
        let into = (t - first) % (fade + delay + moving + pause);
        Some(if into < fade || (into >= fade + delay && into < fade + delay + moving) {
            0
        } else if into < fade + delay {
            micros(fade + delay - into)
        } else {
            micros(fade + delay + moving + pause - into)
        })
    }
}

fn micros(seconds: f64) -> Time {
    (seconds * 1_000_000.0).ceil().max(1.0) as Time
}

/// Every duration, curve and rate the scene moves by.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Motion {
    pub carousel_step: Time,
    pub carousel_easing: Easing,
    pub carousel_repeat: RepeatRule,
    pub carousel_fast_repeat: RepeatRule,
    pub list_repeat: RepeatRule,
    /// The gamelist's metadata and media fade out from the first repeat of a held direction, and back in when scrolling stops.
    pub metadata_fade_out: Time,
    pub metadata_fade_in: Time,
    /// An image or video with scrollFadeIn rises from scroll_fade_in_from to full over scroll_fade_in when the game changes.
    pub scroll_fade_in: Time,
    pub scroll_fade_in_from: f64,
    /// The textlist's selected name: font sizes a second at speed 1, and the gap as seconds of that travel per unit of textHorizontalScrollGap.
    pub marquee_speed_per_em: f64,
    pub marquee_gap_seconds: f64,
    pub horizontal_container_speed_per_em: f64,
    pub horizontal_container_gap_seconds: f64,
    pub vertical_container_speed_per_em: f64,
    pub vertical_container_fade_in: Time,
    pub grid_step: Time,
    pub grid_easing: Easing,
    pub grid_repeat: RepeatRule,
    /// The camera pan between the system and gamelist views, a screen height, the gamelist below.
    pub view_slide: Time,
    pub view_slide_easing: Easing,
}

impl Motion {
    /// The values measured from ES-DE 3.4.1 (EmuSen_BigPicture.md §14.7, §16).
    pub fn esde() -> Self {
        Motion {
            carousel_step: ms(400.0),
            carousel_easing: Easing::QuadraticOut,
            carousel_repeat: RepeatRule::new(ms(500.0), ms(200.0)),
            carousel_fast_repeat: RepeatRule::fast(ms(500.0), ms(180.0), ms(1580.0), ms(80.0), 1),
            list_repeat: RepeatRule::fast(ms(500.0), ms(114.0), ms(1703.0), ms(15.9), 4),
            metadata_fade_out: ms(149.0),
            metadata_fade_in: ms(150.0),
            scroll_fade_in: ms(326.0),
            scroll_fade_in_from: 0.5,
            marquee_speed_per_em: 131.5 / 30.0,
            marquee_gap_seconds: 1.0,
            horizontal_container_speed_per_em: 131.5 / 30.0,
            horizontal_container_gap_seconds: 1.0,
            vertical_container_speed_per_em: 37.03 / 30.0,
            vertical_container_fade_in: ms(298.0),
            grid_step: ms(250.0),
            grid_easing: Easing::QuadraticOut,
            grid_repeat: RepeatRule::new(ms(500.0), ms(200.0)),
            view_slide: ms(402.0),
            view_slide_easing: Easing::CubicOut,
        }
    }
}

/// A GIF's frame at a time since its restart, as ES-DE 3.4.1's `GIFAnimComponent` steps it (`setAnimation`, `update`,
/// `render`): every frame for the first one's delay over the speed, whole milliseconds; `normal` and `reverse` start
/// again from their first frame, the alternating forms turn at each end without showing it twice; after
/// `iterations` plays (0 for ever) nothing is drawn. Returns the frame and the time until it changes, or None when done.
pub fn gif_frame(since: Time, frames: u32, delay_ms: u32, speed: f64, direction: &str, iterations: u32) -> Option<(u32, Option<Time>)> {
    if frames == 0 {
        return None;
    }
    let pacing = ((f64::from(delay_ms) / speed.clamp(0.2, 3.0)) as i64).max(1) * 1000;
    let wait = Some(pacing - since.max(0) % pacing);
    if frames == 1 {
        return Some((0, None));
    }
    // The file's first frame stands for the first pacing; each later one shows the next frame of the sequence, since
    // `render` reads the frame it is at and then counts on.
    let paced = since.max(0) / pacing;
    if paced == 0 {
        return Some((0, wait));
    }
    let step = paced - 1;
    let n = i64::from(frames);
    let alternate = direction.starts_with("alternate");
    let reverse = direction == "reverse" || direction == "alternateReverse";
    // A play's frames: n from the start; turning, the end frame is not shown again, so n - 1 after the first.
    let (play, frame) = if !alternate {
        let (play, at) = (step / n, step % n);
        (play, if reverse { n - 1 - at } else { at })
    } else if step < n {
        (0, if reverse { n - 1 - step } else { step })
    } else {
        let rest = step - n;
        let (k, at) = (rest / (n - 1), rest % (n - 1));
        let backwards = (k % 2 == 0) != reverse;
        (k + 1, if backwards { n - 2 - at } else { at + 1 })
    };
    if iterations > 0 && play >= i64::from(iterations) {
        return None;
    }
    Some((frame as u32, wait))
}
