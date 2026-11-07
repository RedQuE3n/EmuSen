use crate::world::SeededRandom;

/// The sample rate of every cue.
pub const RATE: u32 = 48_000;

/// One partial of a struck bell: its frequency as a multiple of the note, its amplitude and when it has died away.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Partial {
    pub ratio: f64,
    pub amplitude: f64,
    /// When, after the strike, it has fallen exponentially to a ten-thousandth.
    pub decay: f64,
}

/// A bell struck on a scale: the splash's sound as data (LunaP §219.4).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Chime {
    /// The first bell's pitch in hertz.
    pub root: f64,
    /// The scale's steps in semitones above the root within an octave; the bells climb it, an octave higher each time round.
    pub ladder: &'static [i32],
    pub partials: &'static [Partial],
    /// How long a partial takes to rise to its peak.
    pub attack: f64,
    /// How long a partial sounds after its decay before it stops.
    pub tail: f64,
    /// The widest random detune of a partial, as a fraction of its frequency either way.
    pub detune: f64,
    pub detune_seed: u32,
    /// The echo's delay in seconds; 0 for none, the sound fully dry.
    pub echo: f64,
    pub feedback: f64,
    pub gain: f64,
    /// The corner, in hertz, of the low-pass filter the whole mix passes through.
    pub lowpass: f64,
    /// The filter's Q in decibels, as a Web Audio low-pass reads it.
    pub lowpass_q: f64,
}

impl Chime {
    /// Moonrise's bell: G major pentatonic from G4, four inharmonic partials, dry, at the menus' level since 2026-11-05.
    pub const MOONRISE: Chime = Chime {
        root: 391.995,
        ladder: &[0, 2, 4, 7, 9],
        partials: &[
            Partial { ratio: 1.0, amplitude: 1.0, decay: 0.55 },
            Partial { ratio: 2.76, amplitude: 0.18, decay: 0.22 },
            Partial { ratio: 5.40, amplitude: 0.06, decay: 0.11 },
            Partial { ratio: 8.93, amplitude: 0.02, decay: 0.06 },
        ],
        attack: 0.018,
        tail: 0.05,
        detune: 0.002,
        detune_seed: 0x4265_6c6c,
        echo: 0.0,
        feedback: 0.0,
        gain: 0.2,
        lowpass: 2400.0,
        lowpass_q: 0.5,
    };

    /// The pitch of a bell along the ladder, in equal temperament from the root.
    pub fn note_of(&self, bell: usize) -> f64 {
        let steps = self.ladder.len();
        let semitones = 12 * (bell / steps) as i32 + self.ladder[bell % steps];
        self.root * 2f64.powf(semitones as f64 / 12.0)
    }
}

/// One strike on the ladder: which rung, when, and how it sounds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bell {
    /// The ladder rung struck, 0 for G4.
    pub note: usize,
    /// When it is struck, in seconds from the sound's start.
    pub start: f64,
    /// The factor the partials' falls are multiplied by.
    pub decay: f64,
    pub gain: f64,
    /// The corner, in hertz, of the low-pass the strike passes through.
    pub lowpass: f64,
}

impl Bell {
    pub const fn new(note: usize, start: f64, decay: f64, gain: f64) -> Bell {
        Bell { note, start, decay, gain, lowpass: 2400.0 }
    }

    pub const fn dull(note: usize, start: f64, decay: f64, gain: f64, lowpass: f64) -> Bell {
        Bell { note, start, decay, gain, lowpass }
    }
}

/// When a partial of a strike has fallen to a ten-thousandth: its decay scaled by the strike's, never under the attack and 10 ms.
fn fall(chime: &Chime, p: &Partial, b: &Bell) -> f64 {
    (p.decay * b.decay).max(chime.attack + 0.01)
}

/// A chime's bells, one per strike time in ladder order, as 48 kHz stereo samples starting at the first strike.
pub fn render(chime: &Chime, strikes: &[f64]) -> Vec<f32> {
    let bells: Vec<Bell> = strikes
        .iter()
        .enumerate()
        .map(|(n, at)| Bell { note: n, start: *at, decay: 1.0, gain: chime.gain, lowpass: chime.lowpass })
        .collect();
    render_bells(chime, &bells)
}

/// The Web Audio graph of the draft: dry partials, a fed-back delay line and a low-pass biquad per corner, trimmed where the mix falls 60 dB under its loudest sample (LunaP §219.4, §220.1).
pub fn render_bells(chime: &Chime, bells: &[Bell]) -> Vec<f32> {
    if bells.is_empty() {
        return Vec::new();
    }
    let rate = RATE as f64;
    let start = bells.iter().map(|b| b.start).fold(f64::MAX, f64::min);
    let last_end =
        bells.iter().flat_map(|b| chime.partials.iter().map(move |p| b.start - start + fall(chime, p, b) + chime.tail)).fold(0.0, f64::max);
    let delay = if chime.echo > 0.0 { ((chime.echo * rate).round() as usize).max(1) } else { 0 };
    let frames = (last_end * rate).ceil() as usize + delay * 10 + 1;
    let mut dry: Vec<(f64, Vec<f64>)> = Vec::new();
    let mut detune = SeededRandom::new(chime.detune_seed);
    for b in bells {
        let line = match dry.iter().position(|(corner, _)| *corner == b.lowpass) {
            Some(i) => i,
            None => {
                dry.push((b.lowpass, vec![0.0; frames]));
                dry.len() - 1
            }
        };
        let buffer = &mut dry[line].1;
        let first = ((b.start - start) * rate).round_ties_even() as usize;
        for p in chime.partials {
            let hz = chime.note_of(b.note) * p.ratio * (1.0 + (detune.draw() - 0.5) * 2.0 * chime.detune);
            let amplitude = b.gain * p.amplitude;
            let fall = fall(chime, p, b);
            let w = 2.0 * std::f64::consts::PI * hz / rate;
            let slope = (0.0001 / p.amplitude).ln() / (fall - chime.attack);
            let count = ((fall + chime.tail) * rate) as usize;
            for i in 0..count {
                let Some(slot) = buffer.get_mut(first + i) else { break };
                let at = i as f64 / rate;
                let level = if at < chime.attack {
                    amplitude * at / chime.attack
                } else if at < fall {
                    amplitude * (slope * (at - chime.attack)).exp()
                } else {
                    b.gain * 0.0001
                };
                *slot += level * (w * i as f64).sin();
            }
        }
    }
    let mut mix = vec![0f32; frames];
    let mut echo = vec![0f64; frames];
    for (corner, line) in &dry {
        let w0 = 2.0 * std::f64::consts::PI * corner / rate;
        let (sin, cos) = w0.sin_cos();
        let alpha = sin / (2.0 * 10f64.powf(chime.lowpass_q / 20.0));
        let a0 = 1.0 + alpha;
        let (b0, b1) = ((1.0 - cos) / 2.0 / a0, (1.0 - cos) / a0);
        let b2 = b0;
        let (a1, a2) = (-2.0 * cos / a0, (1.0 - alpha) / a0);
        let (mut x1, mut x2, mut y1, mut y2) = (0.0, 0.0, 0.0, 0.0);
        echo.fill(0.0);
        for i in 0..frames {
            echo[i] = if delay > 0 && i >= delay { line[i - delay] + chime.feedback * echo[i - delay] } else { 0.0 };
            let x = line[i] + echo[i];
            let y = b0 * x + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2;
            (x2, x1, y2, y1) = (x1, x, y1, y);
            mix[i] += y as f32;
        }
    }
    let loudest = mix.iter().fold(0f32, |m, v| m.max(v.abs()));
    let mut end = frames;
    while end > 0 && mix[end - 1].abs() < loudest * 0.001 {
        end -= 1;
    }
    mix[..end].iter().flat_map(|v| [*v, *v]).collect()
}

/// The loudest sample of a rendering.
pub fn loudest(samples: &[f32]) -> f32 {
    samples.iter().fold(0f32, |m, v| m.max(v.abs()))
}
