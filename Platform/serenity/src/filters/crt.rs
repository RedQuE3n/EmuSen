//! A CRT computed from its display chain: the console's encoder, the receiver's decoder, the beam, the phosphors, the
//! mask and the glass, each pass's GLSL written from the parameters' values. Every figure is sourced in EmuSen_CRT.md
//! (§3, §5, §11, §12, §16, §18, §20, §22 to §24 and §27); the numbers are written as the C# that first wrote these
//! passes did, so a preset is the same text whichever program writes it.

use super::number::fixed;
use super::{ANY_CONSOLE, Axis, Filter, Pass, Scale, Setting, Size, Values, choice, glsl, screens, setting};
use std::collections::BTreeMap;
use std::f64::consts::PI;

pub const NAME: &str = "CRT";

/// The signal's samples across the active picture on the Accurate tier.
pub const SIGNAL_SAMPLES: i32 = 2048;

/// The glare map's size, in cells across the tube's face.
pub const GLARE_WIDTH: i32 = 128;
pub const GLARE_HEIGHT: i32 = 96;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Signal {
    Rgb,
    SVideo,
    Composite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mask {
    None,
    ApertureGrille,
    Slot,
    Dot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decoder {
    Notch,
    Comb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quality {
    Performance,
    Balanced,
    Accurate,
}

/// One class of set: its picture, mask, spot, receiver and glass.
#[derive(Clone, Copy, Debug)]
pub struct Screen {
    pub label: &'static str,
    pub width_millimetres: f64,
    pub mask: Mask,
    pub triads_across: f64,
    pub spot_minimum: f64,
    pub spot_maximum: f64,
    pub decoder: Decoder,
    pub chroma_megahertz: f64,
    pub video_megahertz: f64,
    pub cylinder: bool,
    pub radius_over_width: f64,
    pub convergence_centre: f64,
    pub convergence_edge: f64,
}

#[allow(clippy::too_many_arguments)]
const fn screen(label: &'static str, width_millimetres: f64, mask: Mask, triads_across: f64, spot_minimum: f64, spot_maximum: f64, decoder: Decoder, chroma_megahertz: f64, video_megahertz: f64, cylinder: bool, radius_over_width: f64, convergence_centre: f64, convergence_edge: f64) -> Screen {
    Screen { label, width_millimetres, mask, triads_across, spot_minimum, spot_maximum, decoder, chroma_megahertz, video_megahertz, cylinder, radius_over_width, convergence_centre, convergence_edge }
}

pub const SCREENS: [Screen; 6] = [
    screen("Consumer TV, 14-inch", 264.0, Mask::Slot, 440.0, 0.158, 0.316, Decoder::Notch, 0.5, 4.2, false, 1.9, 0.6, 1.0),
    screen("Consumer TV, 20-inch", 386.0, Mask::Slot, 515.0, 0.108, 0.217, Decoder::Notch, 0.5, 4.2, false, 1.9, 0.6, 1.0),
    screen("Consumer Trinitron, 27-inch", 549.0, Mask::ApertureGrille, 732.0, 0.076, 0.153, Decoder::Comb, 0.5, 4.2, true, 2.5, 0.6, 1.0),
    screen("Professional monitor, 14-inch", 266.0, Mask::ApertureGrille, 1064.0, 0.126, 0.252, Decoder::Comb, 1.3, 10.0, true, 2.5, 0.4, 0.5),
    screen("Professional monitor, 20-inch", 386.0, Mask::ApertureGrille, 1245.0, 0.087, 0.173, Decoder::Comb, 1.3, 10.0, true, 2.5, 0.5, 0.7),
    screen("PC monitor, 15-inch", 280.0, Mask::Dot, 1167.0, 0.09, 0.18, Decoder::Comb, 1.3, 30.0, false, 3.3, 0.3, 0.4),
];

pub const DEFAULT_SCREEN: i32 = 1;

/// A phosphor set and its white, as chromaticities.
#[derive(Clone, Copy, Debug)]
pub struct Colour {
    pub label: &'static str,
    pub rx: f64,
    pub ry: f64,
    pub gx: f64,
    pub gy: f64,
    pub bx: f64,
    pub by: f64,
    pub wx: f64,
    pub wy: f64,
}

#[allow(clippy::too_many_arguments)]
const fn colour(label: &'static str, rx: f64, ry: f64, gx: f64, gy: f64, bx: f64, by: f64, wx: f64, wy: f64) -> Colour {
    Colour { label, rx, ry, gx, gy, bx, by, wx, wy }
}

pub const COLOURS: [Colour; 3] = [
    colour("North America", 0.630, 0.340, 0.310, 0.595, 0.155, 0.070, 0.3127, 0.3290),
    colour("Japan", 0.618, 0.350, 0.280, 0.605, 0.152, 0.063, 0.281, 0.311),
    colour("Europe", 0.640, 0.330, 0.290, 0.600, 0.150, 0.060, 0.3127, 0.3290),
];

/// The display's own primaries, both with a D65 white.
pub const DISPLAYS: [Colour; 2] = [
    colour("sRGB", 0.640, 0.330, 0.300, 0.600, 0.150, 0.060, 0.3127, 0.3290),
    colour("Display P3", 0.680, 0.320, 0.265, 0.690, 0.150, 0.060, 0.3127, 0.3290),
];

/// Two exponentials fitted to Kuhn's measured P22 tails.
pub const TAIL_FAST: f64 = 0.590;
pub const TAIL_SLOW: f64 = 0.945;
pub const GREEN_FAST: f64 = 0.0679;
pub const GREEN_SLOW: f64 = 0.0094;
pub const BLUE_FAST: f64 = 0.0601;
pub const BLUE_SLOW: f64 = 0.0081;

/// The two decays' ratios a frame and their weights, on a standard's field.
#[derive(Clone, Copy, Debug)]
pub struct Tails {
    pub fast: f64,
    pub slow: f64,
    pub green_fast: f64,
    pub green_slow: f64,
    pub blue_fast: f64,
    pub blue_slow: f64,
}

impl Tails {
    pub fn of(pal: bool) -> Tails {
        if !pal {
            return Tails { fast: TAIL_FAST, slow: TAIL_SLOW, green_fast: GREEN_FAST, green_slow: GREEN_SLOW, blue_fast: BLUE_FAST, blue_slow: BLUE_SLOW };
        }
        let t = PAL_FIELD_MILLISECONDS / NTSC_FIELD_MILLISECONDS;
        let (fast, slow) = (TAIL_FAST.powf(t), TAIL_SLOW.powf(t));
        let (kf, ks) = ((1.0 - fast) / (1.0 - TAIL_FAST), (1.0 - slow) / (1.0 - TAIL_SLOW));
        Tails { fast, slow, green_fast: GREEN_FAST * kf, green_slow: GREEN_SLOW * ks, blue_fast: BLUE_FAST * kf, blue_slow: BLUE_SLOW * ks }
    }
}

pub const TAIL_SCALE: f64 = 0.1606;
pub const GLARE_SIGMA: f64 = 0.0668;
pub const GLARE_WEIGHT: f64 = 0.0299;
pub const BLACK_LEVEL: f64 = 0.01 / 176.0;
pub const STANDARD_ACTIVE_MICROSECONDS: f64 = 52.66;
pub const SUBCARRIER_MEGAHERTZ: f64 = 3.579545;
pub const BRIGHT_WHITE: f64 = 0.6;
pub const FINEST_TRIAD_PIXELS: f64 = 3.0;
pub const LEAST_TRIAD_PIXELS: f64 = 1.5;

/// A demodulator's R-Y and G-Y axes, as angles from B-Y and gains against B-Y's.
#[derive(Clone, Copy, Debug)]
pub struct Demodulator {
    pub label: &'static str,
    pub red_degrees: f64,
    pub red_gain: f64,
    pub green_degrees: f64,
    pub green_gain: f64,
}

pub fn demodulators() -> [Demodulator; 3] {
    [
        Demodulator {
            label: "The standard's",
            red_degrees: 90.0,
            red_gain: 0.492 / 0.877,
            green_degrees: f64::atan2(-0.299 / 0.587 * 0.492 / 0.877, -0.114 / 0.587) * 180.0 / PI,
            green_gain: ((0.114 / 0.587f64).powf(2.0) + (0.299 / 0.587 * 0.492 / 0.877f64).powf(2.0)).sqrt(),
        },
        Demodulator { label: "Sony CXA2025AS, US axis", red_degrees: 112.0, red_gain: 0.83, green_degrees: 252.0, green_gain: 0.30 },
        Demodulator { label: "Sony CXA2025AS, Japan axis", red_degrees: 95.0, red_gain: 0.78, green_degrees: 240.0, green_gain: 0.30 },
    ]
}

/// Rows of R-Y, G-Y and B-Y over B-Y and R-Y as the standard's axes give them.
pub fn axes_matrix(d: &Demodulator) -> [[f64; 2]; 3] {
    let k = 0.877 / 0.492;
    let (r, g) = (d.red_degrees * PI / 180.0, d.green_degrees * PI / 180.0);
    [[d.red_gain * r.cos(), d.red_gain * k * r.sin()], [d.green_gain * g.cos(), d.green_gain * k * g.sin()], [1.0, 0.0]]
}

/// The television standard's parameter: 0 NTSC, 1 PAL, 2 the game's.
pub const STANDARD: &str = "standard";
pub const FOLLOWS_GAME: f32 = 2.0;

pub fn is_pal(value: f32) -> bool {
    value > 0.5 && value < 1.5
}

pub fn settings() -> Vec<Setting> {
    let screens: Vec<&str> = SCREENS.iter().map(|s| s.label).collect();
    let colours: Vec<&str> = COLOURS.iter().map(|c| c.label).collect();
    let displays: Vec<&str> = DISPLAYS.iter().map(|c| c.label).collect();
    let axes: Vec<&str> = demodulators().iter().map(|d| d.label).collect();
    vec![
        choice("quality", "Quality", Quality::Balanced as i32, &["Performance", "Balanced", "Accurate"]),
        choice("signal", "Signal", Signal::Composite as i32, &["RGB", "S-Video", "Composite"]),
        choice(STANDARD, "Television standard", FOLLOWS_GAME as i32, &["NTSC", "PAL", "The game's"]),
        choice("palDecoder", "PAL colour decoding", 0, &["Delay line", "Simple"]),
        choice("screen", "Screen", DEFAULT_SCREEN, &screens),
        choice("colour", "Colour", 0, &colours),
        setting("curvature", "Curvature", 1.0, 0.0, 2.0, 0.05),
        choice("level", "Picture brightness", 0, &["Bright", "The tube's own"]),
        setting("displayNits", "Display brightness, cd/m²", 300.0, 100.0, 1000.0, 10.0),
        choice("subpixels", "Display subpixels", 1, &["None", "RGB", "BGR"]),
        choice("gamut", "Display colours", 0, &displays),
        choice("mask", "Mask", 0, &["The screen's own", "None", "Aperture grille", "Slot mask", "Dot mask"]),
        setting("maskPitch", "Mask pitch, relative to the screen's", 1.0, 0.5, 3.0, 0.05),
        choice("maskLimit", "Mask on a large display", 1, &["Enlarged with the picture", "Kept fine"]),
        setting("maskDepth", "Mask depth", 1.0, 0.0, 1.0, 0.05),
        setting("spotSize", "Beam spot size, relative to the screen's", 1.0, 0.5, 2.0, 0.05),
        setting("spotGrowth", "Beam spot growth with brightness", 1.0, 0.0, 2.0, 0.05),
        choice("decoder", "Luma and chroma separation", 0, &["The screen's own", "Notch", "Comb"]),
        choice("axes", "Colour demodulator", 0, &axes),
        setting("chroma", "Chroma bandwidth, relative to the screen's", 1.0, 0.5, 3.0, 0.1),
        setting("glare", "Glare in the glass", 0.1, 0.0, 3.0, 0.1),
        setting("persistence", "Phosphor persistence", 0.1, 0.0, 6.0, 0.1),
        setting("convergence", "Convergence error", 1.0, 0.0, 3.0, 0.1),
        setting("tubeNits", "Tube white, cd/m²", 100.0, 50.0, 300.0, 5.0),
        setting("gamma", "Tube gamma", 2.4, 2.0, 2.8, 0.05),
        setting("contrast", "Contrast", 1.0, 0.5, 1.5, 0.01),
        setting("brightness", "Brightness", 0.0, -0.1, 0.1, 0.005),
        choice("interlace", "Interlaced pictures", 0, &["Woven", "Fields"]),
    ]
}

/// The settings that decide which passes exist or what is written into them as constants.
pub const STRUCTURAL: [&str; 11] = ["quality", "signal", "standard", "palDecoder", "screen", "colour", "gamut", "mask", "decoder", "axes", "chroma"];

/// One standard's line on a console.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineTiming {
    pub active_microseconds: f64,
    pub cycles_across: f64,
    pub phase_line: f64,
    pub phase_frame: f64,
    pub phase_crawl: f64,
    pub switch_frame: bool,
}

const fn line(active_microseconds: f64, cycles_across: f64, phase_line: f64, phase_frame: f64) -> LineTiming {
    LineTiming { active_microseconds, cycles_across, phase_line, phase_frame, phase_crawl: 0.0, switch_frame: false }
}

const LINE_KEYS: [&str; 6] = ["activeUs", "cyclesAcross", "phaseLine", "phaseFrame", "phaseCrawl", "switchFrame"];

fn timing(ntsc: LineTiming, pal: LineTiming) -> Values {
    let mut values = Values::new();
    values.insert("lumaMHz".into(), 5.0);
    values.insert("chromaMHz".into(), 1.3);
    for (prefix, l) in [("", ntsc), ("pal.", pal)] {
        let numbers = [l.active_microseconds, l.cycles_across, l.phase_line, l.phase_frame, l.phase_crawl, if l.switch_frame { 1.0 } else { 0.0 }];
        for (key, number) in LINE_KEYS.iter().zip(numbers) {
            values.insert(format!("{prefix}{key}"), number as f32);
        }
    }
    values
}

/// A console's line in the standard chosen, from its constants.
pub fn line_of(values: &Values, pal: bool) -> LineTiming {
    let prefix = if pal { "pal." } else { "" };
    let of = |key: &str| values.get(&format!("{prefix}{key}")).copied().unwrap_or(0.0);
    LineTiming {
        active_microseconds: of("activeUs") as f64,
        cycles_across: of("cyclesAcross") as f64,
        phase_line: of("phaseLine") as f64,
        phase_frame: of("phaseFrame") as f64,
        phase_crawl: of("phaseCrawl") as f64,
        switch_frame: of("switchFrame") > 0.5,
    }
}

pub const PAL_SUBCARRIER_MEGAHERTZ: f64 = 4.43361875;
pub const PAL_ACTIVE_MICROSECONDS: f64 = 52.0;
pub const PAL_CYCLES_PER_LINE: f64 = 283.7516;

/// The Sony CXA2025AS's luma trap's lower -3 dB edge.
pub const TRAP_LOWER_EDGE_MEGAHERTZ: f64 = 2.70;

pub fn trap_q() -> f64 {
    SUBCARRIER_MEGAHERTZ * TRAP_LOWER_EDGE_MEGAHERTZ / (SUBCARRIER_MEGAHERTZ * SUBCARRIER_MEGAHERTZ - TRAP_LOWER_EDGE_MEGAHERTZ * TRAP_LOWER_EDGE_MEGAHERTZ)
}

fn trap_step_scale() -> f64 {
    let q = trap_q();
    1.0 / (q * q - 0.25).sqrt()
}

/// The band the trap takes away, as taps back along a line: a bilinear notch with the analogue trap's centre and lower edge.
pub fn trap_taps(samples_per_microsecond: f64, centre: f64) -> Vec<f64> {
    let c0 = (PI * centre / samples_per_microsecond).tan();
    let c1 = (PI * centre * TRAP_LOWER_EDGE_MEGAHERTZ / SUBCARRIER_MEGAHERTZ / samples_per_microsecond).tan();
    let q = c0 * c1 / (c0 * c0 - c1 * c1);
    let w0 = 2.0 * PI * centre / samples_per_microsecond;
    let alpha = w0.sin() / (2.0 * q);
    let a0 = 1.0 + alpha;
    let (b0, b1) = (1.0 / a0, -2.0 * w0.cos() / a0);
    let (a1, a2) = (b1, (1.0 - alpha) / a0);
    let mut taps = Vec::new();
    let (mut x1, mut x2, mut y1, mut y2) = (0.0, 0.0, 0.0, 0.0);
    for n in 0..256 {
        let x = if n == 0 { 1.0 } else { 0.0 };
        let y = b0 * x + b1 * x1 + b0 * x2 - a1 * y1 - a2 * y2;
        (x2, x1, y2, y1) = (x1, x, y1, y);
        taps.push(x - y);
        if n > 2 && y.abs() < 1e-5 && y2.abs() < 1e-5 {
            break;
        }
    }
    taps
}

pub const NTSC_FIELD_MILLISECONDS: f64 = 1000.0 / 59.94;
pub const PAL_FIELD_MILLISECONDS: f64 = 1000.0 / 50.0;
pub const SG1000_PAL_CYCLES: f64 = PAL_SUBCARRIER_MEGAHERTZ / (10.738635 / 2.0);
pub const PS2_VIDEO_MEGAHERTZ: f64 = 54.0;

/// Each console's subcarrier against its pixels on both standards; "" is a standard line.
pub fn console_timing() -> BTreeMap<String, Values> {
    let ntsc_pixels = line(256.0 / 5.3693175, 256.0 * 2.0 / 3.0, 0.0, 0.0);
    let pal_pixels_switched = LineTiming { switch_frame: true, ..line(256.0 / 5.3203424, 256.0 * 5.0 / 6.0, 0.0, 0.0) };
    let mut nes = timing(line(256.0 / 5.3693175, 256.0 * 2.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0), line(256.0 / 5.3203424, 256.0 * 5.0 / 6.0, 1.0 / 6.0, 0.0));
    nes.insert("ppuSignal".into(), 1.0);
    let all = [
        ("NES", nes),
        ("SNES", timing(line(256.0 / 5.3693175, 256.0 * 2.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0), line(256.0 / 5.3203425, 256.0 * 5.0 / 6.0, 1.0 / 6.0, 0.0))),
        ("Genesis", timing(ntsc_pixels, pal_pixels_switched)),
        (
            "N64",
            timing(
                line(640.0 / 12.1704545, 640.0 * 4.0 / 13.6, 0.5, 0.5),
                LineTiming { phase_crawl: 0.25, switch_frame: true, ..line(640.0 / 12.41413250, 640.0 * 4.0 / 11.2, 0.75, 0.0) },
            ),
        ),
        ("PlayStation", timing(line(2560.0 / 53.693175, 2560.0 / 15.0, 0.5, 0.5), LineTiming { phase_crawl: 0.5, ..line(2560.0 / 53.203425, 2560.0 / 12.0, 0.75, 0.0) })),
        (
            "PlayStation 2",
            timing(
                line(2560.0 / PS2_VIDEO_MEGAHERTZ, 2560.0 * 227.5 / 3432.0, 0.5, 263.0 * 227.5 % 1.0),
                LineTiming {
                    phase_crawl: 314.0 * PAL_CYCLES_PER_LINE % 1.0,
                    ..line(2560.0 / PS2_VIDEO_MEGAHERTZ, 2560.0 / PS2_VIDEO_MEGAHERTZ * PAL_SUBCARRIER_MEGAHERTZ, PAL_CYCLES_PER_LINE % 1.0, 0.0)
                },
            ),
        ),
        ("Master System", timing(ntsc_pixels, pal_pixels_switched)),
        (
            "SG-1000",
            timing(
                ntsc_pixels,
                LineTiming {
                    phase_crawl: (313 * 342) as f64 * SG1000_PAL_CYCLES % 1.0,
                    switch_frame: true,
                    ..line(256.0 / 5.3693175, 256.0 * SG1000_PAL_CYCLES, 342.0 * SG1000_PAL_CYCLES % 1.0, 0.0)
                },
            ),
        ),
        (
            ANY_CONSOLE,
            timing(
                line(STANDARD_ACTIVE_MICROSECONDS, STANDARD_ACTIVE_MICROSECONDS * SUBCARRIER_MEGAHERTZ, 0.5, 0.5),
                LineTiming { phase_crawl: 0.25, switch_frame: true, ..line(PAL_ACTIVE_MICROSECONDS, PAL_ACTIVE_MICROSECONDS * PAL_SUBCARRIER_MEGAHERTZ, PAL_CYCLES_PER_LINE % 1.0, 0.0) },
            ),
        ),
    ];
    all.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
}

/// The 4:3 picture of a television.
pub const TELEVISION_ASPECT: f64 = 4.0 / 3.0;

pub fn filter() -> Filter {
    let consoles = screens::televisions();
    let mut f = Filter::plain(NAME, NAME, Vec::new(), Some(&consoles), "EmuSen's own, from the physics and measurements cited in EmuSen_CRT.md", settings());
    f.build = Some(passes);
    f.raw_encoding = Some("nes.ppu".to_string());
    f.structural = Some(STRUCTURAL.iter().map(|s| s.to_string()).collect());
    f.console_defaults = Some(console_timing());
    f.aspect = Some(TELEVISION_ASPECT);
    f.rows_once = true;
    f.crops_itself = true;
    f
}

fn n(value: f64) -> String {
    fixed(value, 8)
}

/// .NET's `Math.Round` and `MathF.Round`: halves to the even neighbour.
fn round(value: f64) -> f64 {
    value.round_ties_even()
}

fn at(values: &Values, id: &str, count: i32) -> i32 {
    (values.get(id).copied().unwrap_or(0.0).round_ties_even() as i32).clamp(0, count - 1)
}

/// What the screen and the player's overrides come to.
#[derive(Clone, Copy, Debug)]
pub struct Resolved {
    pub screen: Screen,
    pub signal: Signal,
    pub mask: Mask,
    pub decoder: Decoder,
    pub chroma_cycles: i32,
    pub colour: Colour,
    pub display: Colour,
    pub quality: Quality,
    pub samples: i32,
    pub line: LineTiming,
    pub pal: bool,
    pub delay_line: bool,
    pub luma_megahertz: f64,
    pub demodulator: Demodulator,
    pub encoder_chroma_megahertz: f64,
}

impl Resolved {
    pub fn subcarrier(&self) -> f64 {
        if self.pal { PAL_SUBCARRIER_MEGAHERTZ } else { SUBCARRIER_MEGAHERTZ }
    }

    pub fn tail(&self) -> Tails {
        Tails::of(self.pal)
    }
}

pub fn resolve(values: &Values) -> Resolved {
    let screen = SCREENS[at(values, "screen", SCREENS.len() as i32) as usize];
    let quality = [Quality::Performance, Quality::Balanced, Quality::Accurate][at(values, "quality", 3) as usize];
    let (mask, decoder) = (at(values, "mask", 5), at(values, "decoder", 3));
    let megahertz = screen.chroma_megahertz * values.get("chroma").copied().unwrap_or(0.0) as f64;
    let pal = values.get(STANDARD).is_some_and(|s| is_pal(*s));
    let line = line_of(values, pal);

    let phase = line.phase_line * 2.0 * PI;
    let comb_gain = (1.0 - phase.cos()) / 2.0;
    let wanted = match decoder {
        0 => screen.decoder,
        1 => Decoder::Notch,
        _ => Decoder::Comb,
    };
    let demodulators = demodulators();
    Resolved {
        screen,
        signal: [Signal::Rgb, Signal::SVideo, Signal::Composite][at(values, "signal", 3) as usize],
        mask: if mask == 0 { screen.mask } else { [Mask::None, Mask::ApertureGrille, Mask::Slot, Mask::Dot][(mask - 1) as usize] },
        decoder: if wanted == Decoder::Comb && (comb_gain < 0.25 || pal) { Decoder::Notch } else { wanted },
        chroma_cycles: (round(0.72 * if pal { PAL_SUBCARRIER_MEGAHERTZ } else { SUBCARRIER_MEGAHERTZ } / megahertz) as i32).clamp(1, 6),
        colour: COLOURS[at(values, "colour", COLOURS.len() as i32) as usize],
        display: DISPLAYS[at(values, "gamut", DISPLAYS.len() as i32) as usize],
        quality,
        samples: if quality == Quality::Accurate { SIGNAL_SAMPLES } else { round(4.0 * line.cycles_across) as i32 },
        line,
        pal,
        delay_line: pal && !values.get("palDecoder").is_some_and(|v| *v > 0.5),
        luma_megahertz: values.get("lumaMHz").map_or(5.0, |v| *v as f64),
        demodulator: if pal { demodulators[0] } else { demodulators[values.get("axes").map_or(0, |a| (a.round_ties_even() as i32).clamp(0, 2)) as usize] },
        encoder_chroma_megahertz: values.get("chromaMHz").map_or(1.3, |v| *v as f64),
    }
}

fn source_pass(glsl: String, name: &str) -> Pass {
    let mut p = Pass::new(glsl, Scale::Source);
    p.name = Some(name.to_string());
    p.float = true;
    p
}

pub fn passes(values: &Values) -> Vec<Pass> {
    let r = resolve(values);
    let ppu = values.get("ppuSignal").is_some_and(|v| *v > 0.5);
    let colours = ppu.then(|| source_pass(ppu_colours(&r), "colours"));
    let face_pass = |linear: &[&str]| {
        let mut p = Pass::new(face(&r), Scale::Viewport);
        p.linear = linear.iter().map(|s| s.to_string()).collect();
        p
    };
    let mut passes = Vec::new();
    passes.extend(colours);
    if r.quality == Quality::Performance {
        let mut gun = source_pass(blend(&r, ppu), "gun");
        gun.width = Some(Size::of(Axis::Source, 2.0));
        passes.push(gun);
        passes.push(face_pass(&["gun"]));
        return passes;
    }
    let signal = Size::of(Axis::Fixed, r.samples as f32);
    let line = |glsl: String, name: &str, feedback: bool, linear: &[&str]| {
        let mut p = source_pass(glsl, name);
        p.width = Some(signal);
        p.feedback = feedback;
        p.linear = linear.iter().map(|s| s.to_string()).collect();
        p
    };
    let small = |glsl: String, name: &str, linear: &[&str]| {
        let mut p = source_pass(glsl, name);
        p.width = Some(Size::of(Axis::Fixed, GLARE_WIDTH as f32));
        p.height = Some(Size::of(Axis::Fixed, GLARE_HEIGHT as f32));
        p.linear = linear.iter().map(|s| s.to_string()).collect();
        p
    };
    if r.signal == Signal::Rgb {
        passes.push(line(encode(&r, ppu), "gun", false, &[]));
    } else {
        passes.push(line(encode(&r, ppu), "signal", false, &[]));
        passes.push(line(separate(&r, ppu), "parts", false, &[]));
        passes.push(line(drive(&r), "gun", false, &[]));
    }
    passes.push(line(state(&r), "state", true, &[]));
    let accurate = r.quality == Quality::Accurate;
    if accurate {
        passes.push(line(beam(&r, false), "beam", false, &[]));
        passes.push(line(beam(&r, true), "spot", false, &[]));
    }
    let lit = if accurate { "beam" } else { "gun" };
    passes.push(small(glare_gather(lit), "small", &[lit]));
    passes.push(small(glare_blur(true), "wide", &[]));
    passes.push(small(glare_blur(false), "haze", &[]));
    passes.push(face_pass(if accurate { &["beam", "spot", "haze"] } else { &["gun", "state", "haze"] }));
    passes
}

fn transfer() -> String {
    format!(
        "
uniform float gamma;
uniform float contrast;
uniform float brightness;
vec3 light(vec3 v) {{
    float lb = pow({}, 1.0 / gamma);
    float b = lb / (1.0 - lb);
    float a = pow(1.0 - lb, gamma);
    return a * pow(max(contrast * v + brightness + b, vec3(0.0)), vec3(gamma));
}}
",
        n(BLACK_LEVEL)
    )
}

fn constants(r: &Resolved) -> String {
    format!(
        "
const float activeUs = {};
const float cyclesAcross = {};
const float phaseLine = {};
const float phaseFrame = {};
const float phaseCrawl = {};
const float switchFrame = {};
const float lumaMHz = {};
const float chromaMHz = {};
",
        n(r.line.active_microseconds),
        n(r.line.cycles_across),
        n(r.line.phase_line),
        n(r.line.phase_frame),
        n(r.line.phase_crawl),
        if r.line.switch_frame { "1.0" } else { "0.0" },
        n(r.luma_megahertz),
        n(r.encoder_chroma_megahertz)
    )
}

/// The NES PPU's terminated levels, low and high for each row of the palette, its black and its 100 IRE.
pub const PPU_LOW: [f64; 4] = [0.228, 0.312, 0.552, 0.880];
pub const PPU_HIGH: [f64; 4] = [0.616, 0.840, 1.100, 1.100];
pub const PPU_BLACK: f64 = 0.312;
pub const PPU_IRE100: f64 = (1.100 - 0.312) / 1.10;
pub const PPU_BURST_LOW: f64 = 0.148;
pub const PPU_BURST_HIGH: f64 = 0.524;
pub const PPU_ATTENUATION: f64 = 0.816328;
pub const PPU_ROW_SKEW_DEGREES: f64 = -5.0;
pub const PPU_PAL_AXIS_STEP: f64 = 9.5;

pub fn ppu_chroma_gain() -> f64 {
    0.2 / (2.0 / PI * (PPU_BURST_HIGH - PPU_BURST_LOW) / PPU_IRE100)
}

pub fn ppu_chroma_gain_pal() -> f64 {
    3.0 / 14.0 / (2.0 / PI * (PPU_BURST_HIGH - PPU_BURST_LOW) / PPU_IRE100)
}

fn ppu_level(volts: f64) -> f64 {
    (volts - PPU_BLACK) / PPU_IRE100
}

fn ppu(pal: bool) -> String {
    let lo: Vec<String> = PPU_LOW.iter().map(|v| n(ppu_level(*v))).collect();
    let hi: Vec<String> = PPU_HIGH.iter().map(|v| n(ppu_level(*v))).collect();
    let mut t = String::new();
    t += "\nuniform sampler2D raw;\nuniform vec2 rawSize;\nvec2 ppuLevels(float row, float hue) {\n";
    t += &format!("    float lo = row < 0.5 ? {} : row < 1.5 ? {} : row < 2.5 ? {} : {};\n", lo[0], lo[1], lo[2], lo[3]);
    t += &format!("    float hi = row < 0.5 ? {} : row < 1.5 ? {} : row < 2.5 ? {} : {};\n", hi[0], hi[1], hi[2], hi[3]);
    t += "    if (hue > 13.5) return vec2(0.0);
    if (hue < 0.5) lo = hi;
    if (hue > 12.5) hi = lo;
    return vec2(lo, hi);
}
bool inPhase(float hue, float p) { return mod(hue + p, 12.0) < 6.0; }
";
    t += if pal {
        "// The 2C07 makes PAL's switch by pairing its oscillators, k with 5 - k, on the lines whose V is inverted; its red and green bits change places.
float osc(float k, float s) { return s < 0.0 ? mod(5.0 - k, 12.0) : k; }
float ppuAt(float colour, float emphasis, float p, float s) {
    float hue = mod(colour, 16.0);
    float row = floor(colour / 16.0);
    vec2 levels = ppuLevels(row, hue);
    float v = inPhase(hue > 0.5 && hue < 12.5 ? osc(hue, s) : hue, p) ? levels.y : levels.x;
    bool dim = (mod(floor(emphasis / 2.0), 2.0) > 0.5 && inPhase(osc(0.0, s), p)) || (mod(emphasis, 2.0) > 0.5 && inPhase(osc(4.0, s), p)) || (emphasis > 3.5 && inPhase(osc(8.0, s), p));
"
    } else {
        "float ppuAt(float colour, float emphasis, float p) {
    float hue = mod(colour, 16.0);
    float row = floor(colour / 16.0);
    vec2 levels = ppuLevels(row, hue);
    float v = inPhase(hue, p) ? levels.y : levels.x;
    bool dim = (mod(emphasis, 2.0) > 0.5 && inPhase(0.0, p)) || (mod(floor(emphasis / 2.0), 2.0) > 0.5 && inPhase(4.0, p)) || (emphasis > 3.5 && inPhase(8.0, p));
"
    };
    t += &format!(
        "    return dim ? ({} * (v * {} + {}) - {}) / {} : v;\n}}\n",
        n(PPU_ATTENUATION),
        n(PPU_IRE100),
        n(PPU_BLACK),
        n(PPU_BLACK),
        n(PPU_IRE100)
    );
    t += "vec2 ppuPixel(vec2 at) {
    vec4 c = raw_eval(at);
    return vec2(floor(c.r * 255.0 + 0.5), floor(c.g * 255.0 + 0.5));
}
";
    t += &format!("vec3 ppuRgb(vec2 pixel{}) {{\n", if pal { ", float s" } else { "" });
    t += "    float y = 0.0;
    float u = 0.0;
    float v = 0.0;
    for (int i = 0; i < 12; i++) {
        float p = float(i);
";
    t += &format!("        float level = ppuAt(pixel.x, pixel.y, p{});\n", if pal { ", 1.0" } else { "" });
    t += &format!("        float a = 6.2831853 * (p - {}) / 12.0;\n", if pal { n(PPU_PAL_AXIS_STEP) } else { "10.0".to_string() });
    t += &format!("        float b = 6.2831853 * (p - {}) / 12.0;\n", if pal { n(PPU_PAL_AXIS_STEP - 1.0) } else { "9.0".to_string() });
    t += "        y += level / 12.0;
        u += level * (cos(a) - cos(b)) / 3.14159265;
        v += level * (sin(b) - sin(a)) / 3.14159265;
    }
";
    t += &format!("    float skew = {} * floor(pixel.x / 16.0){};\n", n(PPU_ROW_SKEW_DEGREES * PI / 180.0), if pal { " * s" } else { "" });
    t += &format!(
        "    vec2 uv = {} * vec2(u * cos(skew) - v * sin(skew), u * sin(skew) + v * cos(skew));\n",
        n(if pal { ppu_chroma_gain_pal() } else { ppu_chroma_gain() })
    );
    t += "    float ry = uv.y / 0.877;
    float by = uv.x / 0.492;
    return vec3(y + ry, y - (0.299 * ry + 0.114 * by) / 0.587, y + by);
}
";
    t
}

fn ppu_looks(r: &Resolved) -> i32 {
    (round(144.0 * r.line.cycles_across / r.samples as f64) as i32).max(12)
}

fn ppu_colours(r: &Resolved) -> String {
    let mut t = String::from("\nuniform sampler2D original;\n");
    t += &ppu(r.pal);
    if r.pal {
        t += &constants(r);
        t += glsl::PHASE;
    }
    t += "\nvec4 filter_main(vec2 coord) {\n    if (rawSize.x < 0.5) return original_eval(coord);\n";
    t += &format!("    return vec4(ppuRgb(ppuPixel(coord){}), 1.0);\n}}", if r.pal { ", palSwitch(floor(coord.y))" } else { "" });
    t
}

const RGB_FROM_COLOURS: &str = "(rawSize.x > 0.5 ? colours_eval(vec2(k + 0.5, row)) : original_eval(vec2(k + 0.5, row))).rgb";
const RGB_FROM_ORIGINAL: &str = "original_eval(vec2(k + 0.5, row)).rgb";

fn encode(r: &Resolved, ppu_signal: bool) -> String {
    let rgb = r.signal == Signal::Rgb;
    let composite = r.signal == Signal::Composite;
    let mut t = String::from("\nuniform sampler2D original;\n");
    if ppu_signal {
        t += "uniform sampler2D colours;\n";
        if composite {
            t += &ppu(r.pal);
        } else {
            t += "uniform vec2 rawSize;\n";
        }
    }
    t += "\nuniform vec2 originalSize;\nuniform vec2 outputSize;\n";
    t += &constants(r);
    t += glsl::FUNCTIONS;
    if rgb {
        t += &transfer();
    } else {
        t += glsl::PHASE;
    }
    t += "
vec4 filter_main(vec2 coord) {
    float width = originalSize.x;
    float x = coord.x / outputSize.x * width;
    float row = floor(coord.y) + 0.5;
";
    if ppu_signal && composite {
        let looks = ppu_looks(r);
        t += "    // The PPU's own composite signal: its square waves, twelve looks to each phase step so the rows' skew falls on a look.
    if (rawSize.x > 0.5) {
        float x0 = floor(coord.x) / outputSize.x * width;
        float dx = width / outputSize.x;
        float level = 0.0;
";
        t += &format!("        for (int j = 0; j < {looks}; j++) {{\n");
        t += &format!("            float at = x0 + (float(j) + 0.5) / {looks}.0 * dx;\n");
        t += "            vec2 pixel = ppuPixel(vec2(floor(at) + 0.5, row));\n";
        t += &format!(
            "            float q = 12.0 * (at * cyclesAcross / width + fract(floor(coord.y) * phaseLine) + mod(frameCount, 2.0) * phaseFrame) + {} + floor(pixel.x / 16.0) * {};\n",
            if r.pal { n(PPU_PAL_AXIS_STEP) } else { "10.0".to_string() },
            n(PPU_ROW_SKEW_DEGREES / 30.0)
        );
        t += &format!("            level += ppuAt(pixel.x, pixel.y, floor(mod(q, 12.0)){}) / {looks}.0;\n", if r.pal { ", palSwitch(floor(coord.y))" } else { "" });
        t += "        }
        return vec4(level, 0.0, 0.0, 1.0);
    }
";
    }
    t += "\n    float rate = width / activeUs;\n";
    t += &format!(
        "    float sy = 0.132532 * rate * {};\n",
        if rgb { format!("sqrt(1.0 / (lumaMHz * lumaMHz) + {})", n(1.0 / (r.screen.video_megahertz * r.screen.video_megahertz))) } else { "(1.0 / lumaMHz)".to_string() }
    );
    t += "    float sc = 0.132532 / chromaMHz * rate;
    float first = floor(x);
    vec3 wide = vec3(0.0);
    vec3 narrow = vec3(0.0);
    for (int i = -4; i <= 4; i++) {
        float k = first + float(i);
        if (k < 0.0 || k >= width) continue;
";
    t += &format!("        vec3 rgb = {};\n", if ppu_signal { RGB_FROM_COLOURS } else { RGB_FROM_ORIGINAL });
    t += "        float y = dot(rgb, vec3(0.299, 0.587, 0.114));
        float wy = cdf((k + 1.0 - x) / sy) - cdf((k - x) / sy);
        float wc = cdf((k + 1.0 - x) / sc) - cdf((k - x) / sc);
";
    if rgb {
        t += "
        wide += wy * rgb;
    }
    return vec4(light(wide), 1.0);
}";
    } else {
        t += "
        wide.x += wy * y;
        narrow += wc * vec3(0.0, 0.492 * (rgb.b - y), 0.877 * (rgb.r - y));
    }
    float angle = 6.2831853 * phaseAt(floor(coord.x), floor(coord.y), outputSize.x);
";
        t += &format!("    float c = narrow.y * sin(angle) + {}narrow.z * cos(angle);\n", if r.pal { "palSwitch(floor(coord.y)) * " } else { "" });
        t += if composite { "    return vec4(wide.x + c, 0.0, 0.0, 1.0);\n}" } else { "    return vec4(wide.x, c, 0.0, 1.0);\n}" };
    }
    t
}

fn separate(r: &Resolved, ppu_signal: bool) -> String {
    let theta = r.line.phase_line * 2.0 * PI;
    let (fr, fi) = ((1.0 - theta.cos()) / 2.0, theta.sin() / 2.0);
    let gain = fr * fr + fi * fi;
    let comb = r.signal == Signal::Composite && r.decoder == Decoder::Comb;
    let trap = r.signal == Signal::Composite && r.decoder == Decoder::Notch;
    let read = if r.signal == Signal::SVideo { "source_eval(vec2(x + 0.5, row + 0.5)).g" } else { "source_eval(vec2(x + 0.5, row + 0.5)).r" };
    let above = "(row < 1.0 ? 0.0 : source_eval(vec2(x + 0.5, row - 0.5)).r)";
    let reach = (r.chroma_cycles.max(2) as f64 * r.samples as f64 / r.line.cycles_across / 2.0).ceil() as i32;
    let mut band = String::new();
    if trap {
        let taps = trap_taps(r.samples as f64 / r.line.active_microseconds, r.subcarrier());
        for (j, tap) in taps.iter().enumerate() {
            band += &if j == 0 { "    ".to_string() } else { format!("    if (s >= {j}.0) ") };
            band += &format!("band += {} * source_eval(vec2(s - {j}.0 + 0.5, row + 0.5)).r;\n", n(*tap));
        }
    }
    let ppu_composite = ppu_signal && r.signal == Signal::Composite;
    let mut t = String::from("\nuniform sampler2D source;\nuniform vec2 inputSize;\n");
    if ppu_composite {
        t += "uniform vec2 rawSize;\n";
    }
    t += &constants(r);
    t += glsl::PHASE;
    t += "
vec4 filter_main(vec2 coord) {
    float s = floor(coord.x);
    float row = floor(coord.y);
    float perCycle = inputSize.x / cyclesAcross;
";
    t += &format!("    float chromaHalf = {} * perCycle * 0.5;\n", n(r.chroma_cycles as f64));
    t += "    float notchHalf = perCycle;
    vec2 chroma = vec2(0.0);
    vec2 notch = vec2(0.0);
    float chromaSum = 0.0;
    float notchSum = 0.0;
";
    t += &format!("    for (int j = -{reach}; j <= {reach}; j++) {{\n");
    t += "        float x = s + float(j);
        float t = abs(float(j));
        if (x < 0.0 || x >= inputSize.x || t >= max(chromaHalf, notchHalf)) continue;
";
    t += &format!("        float v = {};\n", if comb { format!("0.5 * ({read} - {above})") } else { read.to_string() });
    t += "        float angle = 6.2831853 * phaseAt(x, row, inputSize.x);
        vec2 axes = vec2(sin(angle), cos(angle));
        float wc = t < chromaHalf ? 0.5 + 0.5 * cos(3.14159265 * t / chromaHalf) : 0.0;
        chroma += wc * v * axes;
        chromaSum += wc;
";
    if comb {
        t += "        float wn = t < notchHalf ? 0.5 + 0.5 * cos(3.14159265 * t / notchHalf) : 0.0;
        notch += wn * v * axes;
        notchSum += wn;
";
    }
    t += "    }\n";
    t += "    chroma *= 2.0 / max(chromaSum, 0.0001)";
    if ppu_composite {
        t += &format!(" * (rawSize.x > 0.5 ? {} : 1.0)", n(if r.pal { ppu_chroma_gain_pal() } else { ppu_chroma_gain() }));
    }
    t += ";\n    notch *= 2.0 / max(notchSum, 0.0001);\n";
    if r.pal {
        t += "    chroma.y *= palSwitch(row);\n";
    }
    if comb {
        let (a, b) = (n(fr / gain), n(fi / gain));
        t += &format!("\n    chroma = vec2({a} * chroma.x + {b} * chroma.y, {a} * chroma.y - {b} * chroma.x);\n");
        t += &format!("    notch = vec2({a} * notch.x + {b} * notch.y, {a} * notch.y - {b} * notch.x);\n");
    }
    t += "
    float here = 6.2831853 * phaseAt(s, row, inputSize.x);
    float band = 0.0;
";
    t += &band;
    t += &format!(
        "    float y = source_eval(vec2(s + 0.5, row + 0.5)).r{};\n",
        if comb { " - (notch.x * sin(here) + notch.y * cos(here))" } else if trap { " - band" } else { "" }
    );
    t += "    return vec4(y, chroma.x, chroma.y, 1.0);\n}";
    t
}

fn demodulate(r: &Resolved, by: &str, ry: &str) -> String {
    let m = axes_matrix(&r.demodulator);
    let row = |i: usize| format!("{} * {by} + {} * {ry}", n(m[i][0]), n(m[i][1]));
    format!("vec3({}, {}, {by})", row(0), row(1))
}

fn drive(r: &Resolved) -> String {
    let mut t = String::from("\nuniform sampler2D source;\nuniform vec2 inputSize;\n");
    t += &constants(r);
    t += &transfer();
    t += "
vec4 filter_main(vec2 coord) {
    float s = floor(coord.x);
    float row = floor(coord.y) + 0.5;
";
    t += &format!("    float sigma = 0.132532 / {} * inputSize.x / activeUs;\n", n(r.screen.video_megahertz));
    t += "    float y = 0.0;
    float sum = 0.0;
    for (int j = -6; j <= 6; j++) {
        float x = s + float(j);
        if (x < 0.0 || x >= inputSize.x) continue;
        float w = exp(-0.5 * float(j * j) / (sigma * sigma));
        y += w * source_eval(vec2(x + 0.5, row)).r;
        sum += w;
    }
    y /= sum;
    vec4 here = source_eval(vec2(s + 0.5, row));
";
    if r.delay_line {
        t += "    // PAL's delay line: each line's colour averaged with the line's before it.\n    if (row > 1.0) here.gb = 0.5 * (here.gb + source_eval(vec2(s + 0.5, row - 1.0)).gb);\n";
    }
    t += "    float ry = here.b / 0.877;\n    float by = here.g / 0.492;\n";
    t += &format!("    return vec4(light(y + {}), 1.0);\n}}", demodulate(r, "by", "ry"));
    t
}

fn blend(r: &Resolved, ppu_signal: bool) -> String {
    let video = r.screen.video_megahertz;
    let chroma = 0.72 * r.subcarrier() / r.chroma_cycles as f64;
    let rgb = r.signal == Signal::Rgb;
    let notch = r.signal == Signal::Composite && r.decoder == Decoder::Notch;
    let encoder = r.luma_megahertz;
    let through = (-0.5 * 2f64.ln() * r.subcarrier() * r.subcarrier() * (1.0 / (encoder * encoder) + 1.0 / (video * video))).exp();
    let q = trap_q();
    let mut t = String::from("\nuniform sampler2D original;\n");
    if ppu_signal {
        t += "uniform sampler2D colours;\nuniform vec2 rawSize;\n";
    }
    t += "uniform vec2 originalSize;\n";
    t += &constants(r);
    t += glsl::FUNCTIONS;
    t += &transfer();
    t += "

// The receiver's trap band's response to a step, a second-order band-pass's, from where the pixel starts - see EmuSen_CRT.md §24.
float band(float u, float a, float wd) {
";
    t += &format!("    return u <= 0.0 ? 0.0 : {} * exp(-a * u) * sin(wd * u);\n}}\n", n(trap_step_scale()));
    t += "
vec4 filter_main(vec2 coord) {
    float width = originalSize.x;
    float x = coord.x * 0.5;
    float row = floor(coord.y) + 0.5;
    float rate = width / activeUs;
";
    t += &format!("    float a = {} / rate;\n", n(PI * r.subcarrier() / q));
    t += &format!("    float wd = {} / rate;\n", n(2.0 * PI * r.subcarrier() * (1.0 - 1.0 / (4.0 * q * q)).sqrt()));
    t += &format!("    float sy = 0.132532 * rate * sqrt(1.0 / (lumaMHz * lumaMHz) + {});\n", n(1.0 / (video * video)));
    t += &format!("    float sc = 0.132532 * rate * sqrt(1.0 / (chromaMHz * chromaMHz) + {});\n", n(1.0 / (chroma * chroma)));
    t += "    float first = floor(x);
    vec3 wide = vec3(0.0);
    vec2 narrow = vec2(0.0);
    for (int i = -4; i <= 4; i++) {
        float k = first + float(i);
        if (k < 0.0 || k >= width) continue;
";
    t += &format!("        vec3 rgb = {};\n", if ppu_signal { RGB_FROM_COLOURS } else { RGB_FROM_ORIGINAL });
    t += &format!(
        "        float wy = cdf((k + 1.0 - x) / sy) - cdf((k - x) / sy){};\n",
        if notch { format!(" - {} * (band(x - k, a, wd) - band(x - k - 1.0, a, wd))", n(through)) } else { String::new() }
    );
    if rgb {
        t += "        wide += wy * rgb;
    }
    return vec4(light(wide), 1.0);
}";
        return t;
    }
    t += "        float wc = cdf((k + 1.0 - x) / sc) - cdf((k - x) / sc);
        float y = dot(rgb, vec3(0.299, 0.587, 0.114));
        wide.x += wy * y;
        narrow += wc * vec2(rgb.b - y, rgb.r - y);
";
    if r.delay_line {
        t += if ppu_signal {
            "        vec2 up = vec2(k + 0.5, max(row - 1.0, 0.5));
        vec3 above = (rawSize.x > 0.5 ? colours_eval(up) : original_eval(up)).rgb;"
        } else {
            "        vec3 above = original_eval(vec2(k + 0.5, max(row - 1.0, 0.5))).rgb;"
        };
        t += "\n        narrow += wc * vec2(above.b, above.r) - wc * dot(above, vec3(0.299, 0.587, 0.114));\n";
    }
    t += "    }\n";
    if r.delay_line {
        t += "    narrow *= 0.5;\n";
    }
    t += &format!("    return vec4(light(wide.x + {}), 1.0);\n}}", demodulate(r, "narrow.x", "narrow.y"));
    t
}

fn state(r: &Resolved) -> String {
    let tail = r.tail();
    format!(
        "
uniform sampler2D source;
uniform sampler2D feedback;
vec4 filter_main(vec2 coord) {{
    vec3 e = source_eval(coord).rgb;
    vec4 f = feedback_eval(coord);
    return vec4(e.g + {} * f.r, e.g + {} * f.g, e.b + {} * f.b, e.b + {} * f.a);
}}",
        n(tail.fast),
        n(tail.slow),
        n(tail.fast),
        n(tail.slow)
    )
}

/// The light at a sample with its phosphors' tails, shared by the Balanced face and the Accurate beam.
fn light_at(r: &Resolved) -> String {
    let tail = r.tail();
    format!(
        "    vec3 e = gun_eval(p).rgb;
    vec4 st = state_eval(p);
    vec2 fast = persistence * vec2({}, {});
    vec2 slow = persistence * vec2({}, {});
    vec2 kept = 1.0 / (1.0 + fast * {} + slow * {});
    vec2 now = vec2(e.g, e.b);
    vec2 lit = kept * (now + fast * (vec2(st.r, st.b) - now) + slow * (vec2(st.g, st.a) - now));
    return vec3(e.r, lit.x, lit.y);
}}
",
        n(TAIL_SCALE * tail.green_fast),
        n(TAIL_SCALE * tail.blue_fast),
        n(TAIL_SCALE * tail.green_slow),
        n(TAIL_SCALE * tail.blue_slow),
        n(tail.fast / (1.0 - tail.fast)),
        n(tail.slow / (1.0 - tail.slow))
    )
}

fn spot(r: &Resolved) -> String {
    let mut t = String::from(
        "
uniform sampler2D gun;
uniform sampler2D state;
uniform float persistence;
uniform float spotSize;
uniform float spotGrowth;
vec3 lightAt(vec2 p) {
",
    );
    t += &light_at(r);
    t += &format!(
        "vec3 variance(vec3 l) {{
    float low = {};
    float high = {};
    return spotSize * spotSize * (low + (high - low) * spotGrowth * clamp(l, 0.0, 1.5));
}}
",
        n(r.screen.spot_minimum * r.screen.spot_minimum),
        n(r.screen.spot_maximum * r.screen.spot_maximum)
    );
    t
}

fn beam(r: &Resolved, spot_pass: bool) -> String {
    let mut t = String::from("\nuniform vec2 outputSize;\n");
    t += glsl::FUNCTIONS;
    t += &spot(r);
    t += "
vec4 filter_main(vec2 coord) {
    float s = floor(coord.x);
    float row = floor(coord.y) + 0.5;
    float perLine = outputSize.x * 0.75 / (outputSize.y > 300.0 ? outputSize.y * 0.5 : outputSize.y);
    vec3 sum = vec3(0.0);
    vec3 weighted = vec3(0.0);
    for (int t = -8; t <= 8; t++) {
        float x = s + float(t);
        if (x < 0.0 || x >= outputSize.x) continue;
        vec3 l = lightAt(vec2(x + 0.5, row));
        vec3 v = variance(l);
        vec3 sigma = sqrt(v) * perLine;
        vec3 w = cdf3((0.5 - float(t)) / sigma) - cdf3((-0.5 - float(t)) / sigma);
        sum += l * w;
        weighted += l * w * v;
    }
";
    t += if spot_pass { "    return vec4(weighted / max(sum, vec3(0.000001)) + step(sum, vec3(0.000001)) * variance(vec3(0.0)), 1.0);\n}" } else { "    return vec4(sum, 1.0);\n}" };
    t
}

const RASTER: &str = "
uniform vec4 crop;
vec2 pictureOf(vec2 face) {
    return crop.xy + (face / vec2(1.0, 0.75) * 0.5 + 0.5) * crop.zw;
}
";

fn glare_gather(lit: &str) -> String {
    let mut t = String::from("\nuniform sampler2D beam;\nuniform vec2 beamSize;\nuniform vec2 outputSize;\n");
    t += RASTER;
    t += "
vec4 filter_main(vec2 coord) {
    vec3 sum = vec3(0.0);
    for (int i = 0; i < 6; i++) {
        for (int j = 0; j < 3; j++) {
            vec2 cell = (floor(coord) + vec2((float(i) + 0.5) / 6.0, (float(j) + 0.5) / 3.0)) / outputSize;
            vec2 p = pictureOf((cell * 2.0 - 1.0) * vec2(1.0, 0.75));
            if (p.x < 0.0 || p.x > 1.0 || p.y < 0.0 || p.y > 1.0) continue;
            sum += beam_eval(p * beamSize).rgb;
        }
    }
    return vec4(sum / 18.0, 1.0);
}";
    t.replace("beam", lit)
}

fn glare_blur(horizontal: bool) -> String {
    let spread = GLARE_SIGMA * GLARE_WIDTH as f64 * GLARE_SIGMA * GLARE_WIDTH as f64;
    let mut total = 0.0;
    for t in -24..=24 {
        let t = t as f64;
        total += (-0.5 * t * t / spread).exp();
    }
    format!(
        "
uniform sampler2D source;
uniform vec2 inputSize;
vec4 filter_main(vec2 coord) {{
    float sigma = {} * inputSize.x;
    vec3 sum = vec3(0.0);
    for (int t = -24; t <= 24; t++) {{
        vec2 p = floor(coord) + 0.5 + {};
        if (p.x < 0.0 || p.x > inputSize.x || p.y < 0.0 || p.y > inputSize.y) continue;
        sum += source_eval(p).rgb * exp(-0.5 * float(t * t) / (sigma * sigma));
    }}
    return vec4(sum / {}, 1.0);
}}",
        n(GLARE_SIGMA),
        if horizontal { "vec2(float(t), 0.0)" } else { "vec2(0.0, float(t))" },
        n(total)
    )
}

/// One colour's stripes or dots: where its lit run starts in the triad, how wide it is, and whether its columns sit half a pitch down.
struct Layout {
    width: f64,
    start: [f64; 3],
    shifted: [bool; 3],
    vertical_pitch: f64,
    vertical_lit: f64,
}

impl Layout {
    fn lit_fraction(&self) -> f64 {
        self.width * if self.vertical_pitch > 0.0 { self.vertical_lit } else { 1.0 }
    }
}

pub const STRIPE_WIDTH: f64 = 0.199;
pub const SLOT_WIDTH: f64 = 0.19;
pub const SLOT_PITCH: f64 = 0.81;
pub const SLOT_LIT: f64 = 0.76;
pub const DOT_WIDTH: f64 = 0.35;
pub const CORNER_RADIUS: f64 = 0.04;

fn thirds(width: f64) -> [f64; 3] {
    [(1.0 / 3.0 - width) / 2.0, 1.0 / 3.0 + (1.0 / 3.0 - width) / 2.0, 2.0 / 3.0 + (1.0 / 3.0 - width) / 2.0]
}

fn layout_of(mask: Mask) -> Layout {
    match mask {
        Mask::ApertureGrille => Layout { width: STRIPE_WIDTH, start: thirds(STRIPE_WIDTH), shifted: [false; 3], vertical_pitch: 0.0, vertical_lit: 1.0 },
        Mask::Slot => Layout { width: SLOT_WIDTH, start: [0.15 - SLOT_WIDTH / 2.0, 0.45 - SLOT_WIDTH / 2.0, 0.75 - SLOT_WIDTH / 2.0], shifted: [false; 3], vertical_pitch: SLOT_PITCH, vertical_lit: SLOT_LIT },
        Mask::Dot => Layout {
            width: DOT_WIDTH,
            start: [(1.0 / 3.0 - DOT_WIDTH) / 2.0, 2.0 / 3.0 + (1.0 / 3.0 - DOT_WIDTH) / 2.0, 1.0 / 3.0 + (1.0 / 3.0 - DOT_WIDTH) / 2.0],
            shifted: [false, false, true],
            vertical_pitch: 2.0 / 3f64.sqrt(),
            vertical_lit: DOT_WIDTH * 3f64.sqrt() / 2.0,
        },
        Mask::None => Layout { width: 1.0, start: [0.0; 3], shifted: [false; 3], vertical_pitch: 0.0, vertical_lit: 1.0 },
    }
}

/// Rows of the matrix from the tube's guns to the display's channels, in linear light, with no adaptation between their whites.
pub fn guns_to_display(tube: &Colour, display: &Colour) -> [[f64; 3]; 3] {
    let mut m = multiply(&invert(&to_xyz(display)), &to_xyz(tube));
    let peak = m.iter().map(|row| row[0] + row[1] + row[2]).fold(f64::NEG_INFINITY, f64::max);
    if peak > 1.0 {
        for row in m.iter_mut() {
            for v in row.iter_mut() {
                *v /= peak;
            }
        }
    }
    m
}

fn to_xyz(c: &Colour) -> [[f64; 3]; 3] {
    let p = [[c.rx / c.ry, c.gx / c.gy, c.bx / c.by], [1.0, 1.0, 1.0], [(1.0 - c.rx - c.ry) / c.ry, (1.0 - c.gx - c.gy) / c.gy, (1.0 - c.bx - c.by) / c.by]];
    let white = [c.wx / c.wy, 1.0, (1.0 - c.wx - c.wy) / c.wy];
    let inverse = invert(&p);
    let scale: Vec<f64> = inverse.iter().map(|row| row[0] * white[0] + row[1] * white[1] + row[2] * white[2]).collect();
    p.map(|row| [row[0] * scale[0], row[1] * scale[1], row[2] * scale[2]])
}

fn multiply(a: &[[f64; 3]; 3], b: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut m = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            m[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    m
}

fn invert(m: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let [[a, b, c], [d, e, f], [g, h, i]] = *m;
    let det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
    [
        [(e * i - f * h) / det, (c * h - b * i) / det, (b * f - c * e) / det],
        [(f * g - d * i) / det, (a * i - c * g) / det, (c * d - a * f) / det],
        [(d * h - e * g) / det, (b * g - a * h) / det, (a * e - b * d) / det],
    ]
}

fn row(r: &[f64; 3]) -> String {
    format!("vec3({}, {}, {})", n(r[0]), n(r[1]), n(r[2]))
}

const ACCURATE_BEAMS: &str = "
uniform sampler2D beam;
uniform sampler2D spot;
uniform vec2 beamSize;
float beamOf(vec2 picture, float tall, float channel, float field, inout float lineMean, inout float linePeak) {
    vec2 p = picture * beamSize;
    if (picture.x < 0.0 || picture.x > 1.0) return 0.0;
    float scale = beamSize.y > 300.0 ? 2.0 : 1.0;
    float sum = 0.0;
    // A pixel taller than the five lines read is the mean of the lines it covers, five groups of them each read once.
    float stride = ceil(tall / 4.0);
    if (stride > 1.5) {
        float top = p.y - 0.5 * tall;
        float g0 = floor(top / stride);
        for (int j = 0; j <= 4; j++) {
            float lo = max((g0 + float(j)) * stride, 0.0);
            float hi = min((g0 + float(j) + 1.0) * stride, beamSize.y);
            float overlap = min(hi, p.y + 0.5 * tall) - max(lo, top);
            if (overlap <= 0.0) continue;
            vec4 l = beam_eval(vec2(p.x, floor(0.5 * (lo + hi)) + 0.5));
            float light = channel < 0.5 ? l.r : channel < 1.5 ? l.g : l.b;
            sum += light * overlap / tall;
            linePeak = max(linePeak, light);
        }
        lineMean += sum;
        return sum;
    }
    float first = floor(p.y);
    for (int j = -2; j <= 2; j++) {
        float row = first + float(j);
        if (row < 0.0 || row >= beamSize.y) continue;
        vec2 at = vec2(p.x, row + 0.5);
        vec4 l = beam_eval(at);
        vec4 v = spot_eval(at);
        float light = channel < 0.5 ? l.r : channel < 1.5 ? l.g : l.b;
        float sigma = scale * sqrt(max(channel < 0.5 ? v.r : channel < 1.5 ? v.g : v.b, 0.000001));
        float d = p.y - (row + 0.5);
        float lit = scale > 1.5 && interlace > 0.5 ? (mod(row + field, 2.0) < 0.5 ? 2.0 : 0.0) : 1.0;
        sum += lit * light * (cdf((d + 0.5 * tall) / sigma) - cdf((d - 0.5 * tall) / sigma)) / tall;
        float near = max(1.0 - abs(d) / scale, 0.0);
        lineMean += lit * light * near / scale;
        if (near > 0.0) linePeak = max(linePeak, lit * light / (sigma * 2.5066283));
    }
    return sum;
}
";

fn gun_beams(r: &Resolved, performance: bool) -> String {
    let mut t = String::from(
        "
uniform sampler2D gun;
uniform vec2 gunSize;
uniform float spotSize;
uniform float spotGrowth;
",
    );
    if performance {
        t += "\nvec3 lightAt(vec2 p) { return gun_eval(p).rgb; }\n";
    } else {
        t += "\nuniform sampler2D state;\nuniform float persistence;\nvec3 lightAt(vec2 p) {\n";
        t += &light_at(r);
    }
    t += "
vec3 beamsOf(vec2 picture, float tall, vec3 across, float field, inout vec3 lineMean, inout vec3 linePeak) {
    vec2 p = picture * gunSize;
    if (picture.x < 0.0 || picture.x > 1.0) return vec3(0.0);
    float scale = gunSize.y > 300.0 ? 2.0 : 1.0;
    vec3 sum = vec3(0.0);
    // A pixel taller than a line is the mean of the lines it covers, in as many groups as lines are read.
";
    t += &format!("    float stride = ceil(tall / {});\n", if performance { "1.0" } else { "2.0" });
    t += "    if (stride > 1.5) {
        float top = p.y - 0.5 * tall;
        float g0 = floor(top / stride);
";
    t += &format!("        for (int j = 0; j <= {}; j++) {{\n", if performance { "1" } else { "2" });
    t += "            float lo = max((g0 + float(j)) * stride, 0.0);
            float hi = min((g0 + float(j) + 1.0) * stride, gunSize.y);
            float overlap = min(hi, p.y + 0.5 * tall) - max(lo, top);
            if (overlap <= 0.0) continue;
            float row = floor(0.5 * (lo + hi)) + 0.5;
";
    t += &format!(
        "            vec3 l = {};\n",
        if performance { "lightAt(vec2(p.x, row))" } else { "vec3(lightAt(vec2(across.r * gunSize.x, row)).r, lightAt(vec2(across.g * gunSize.x, row)).g, lightAt(vec2(across.b * gunSize.x, row)).b)" }
    );
    t += "            sum += l * overlap / tall;
            linePeak = max(linePeak, l);
        }
        lineMean += sum;
        return sum;
    }
    float first = floor(p.y - 0.5);
";
    t += &format!("    for (int j = {}; j <= 1; j++) {{\n", if performance { "0" } else { "-1" });
    t += &format!("        float row = first + float(j){};\n", if performance { "" } else { " + 0.0" });
    t += "        if (row < 0.0 || row >= gunSize.y) continue;\n";
    t += &format!(
        "        vec3 l = {};\n",
        if performance {
            "lightAt(vec2(p.x, row + 0.5))"
        } else {
            "vec3(lightAt(vec2(across.r * gunSize.x, row + 0.5)).r, lightAt(vec2(across.g * gunSize.x, row + 0.5)).g, lightAt(vec2(across.b * gunSize.x, row + 0.5)).b)"
        }
    );
    let kind = if performance { "float" } else { "vec3" };
    t += &format!(
        "        {kind} v = spotSize * spotSize * ({} + {} * spotGrowth * clamp({}, 0.0, 1.5));\n",
        n(r.screen.spot_minimum * r.screen.spot_minimum),
        n(r.screen.spot_maximum * r.screen.spot_maximum - r.screen.spot_minimum * r.screen.spot_minimum),
        if performance { "dot(l, vec3(0.299, 0.587, 0.114))" } else { "l" }
    );
    t += &format!("        {kind} sigma = scale * sqrt(v);\n");
    t += "        float d = p.y - (row + 0.5);
        float lit = scale > 1.5 && interlace > 0.5 ? (mod(row + field, 2.0) < 0.5 ? 2.0 : 0.0) : 1.0;
";
    t += &format!(
        "        sum += lit * l * ({}) / tall;\n",
        if performance { "cdf((d + 0.5 * tall) / sigma) - cdf((d - 0.5 * tall) / sigma)" } else { "cdf3((d + 0.5 * tall) / sigma) - cdf3((d - 0.5 * tall) / sigma)" }
    );
    t += "        float near = max(1.0 - abs(d) / scale, 0.0);
        lineMean += lit * l * near / scale;
        if (near > 0.0) linePeak = max(linePeak, lit * l / (sigma * 2.5066283));
    }
    return sum;
}
";
    t
}

fn face(r: &Resolved) -> String {
    let layout = layout_of(r.mask);
    let m = guns_to_display(&r.colour, &r.display);
    let (radius, distance) = (r.screen.radius_over_width * 2.0, 6.0);
    let half_width = r.screen.width_millimetres / 2.0;
    let accurate = r.quality == Quality::Accurate;
    let performance = r.quality == Quality::Performance;

    let cover = if accurate { "covered" } else { "boxed" };
    let rows_of = if layout.vertical_pitch <= 0.0 {
        String::new()
    } else {
        format!(
            "    float rowLevel = {cover}(triad.y, footprint.y, 0.0, {}, {});\n    float rowDown = {cover}(triad.y, footprint.y, {}, {}, {});\n",
            n(layout.vertical_pitch * layout.vertical_lit),
            n(layout.vertical_pitch),
            n(layout.vertical_pitch * 0.5),
            n(layout.vertical_pitch * layout.vertical_lit),
            n(layout.vertical_pitch)
        )
    };
    let columns = |by: &str, reach: &str| -> String {
        let mut text = String::new();
        for c in 0..3 {
            let channel = ["r", "g", "b"][c];
            let phase = format!("(triad.x + shift.{channel} * stepX)");
            if layout.vertical_pitch <= 0.0 {
                text += &format!("        cover.{channel} = {by}({phase}, {reach}, {}, {}, 1.0);\n", n(layout.start[c]), n(layout.width));
                continue;
            }
            let even = format!("{by}({phase}, {reach}, {}, {}, 2.0)", n(layout.start[c]), n(layout.width));
            let odd = format!("{by}({phase}, {reach}, {}, {}, 2.0)", n(layout.start[c] + 1.0), n(layout.width));
            text += &if layout.shifted[c] { format!("        cover.{channel} = {even} * rowDown + {odd} * rowLevel;\n") } else { format!("        cover.{channel} = {even} * rowLevel + {odd} * rowDown;\n") };
        }
        text
    };
    let mask = if r.mask == Mask::None {
        "    cover = vec3(1.0);\n".to_string()
    } else if accurate {
        format!("{rows_of}    if (flatMask) {{\n{}    }} else {{\n{}    }}\n", columns("boxed", "footprint.x * 0.5"), columns("covered", "footprint.x"))
    } else {
        format!("{rows_of}{}", columns("boxed", "footprint.x"))
    };

    let mut beams = if accurate { ACCURATE_BEAMS.to_string() } else { gun_beams(r, performance) };
    if !accurate && !performance {
        beams = beams.replace("float first = floor(p.y - 0.5);", "float first = floor(p.y);");
    }

    let spread = format!(
        "    float spread = convergence * ({} + {} * dot(face, face) / 1.5625);\n",
        n(r.screen.convergence_centre / half_width / 2.0),
        n((r.screen.convergence_edge - r.screen.convergence_centre) / half_width / 2.0)
    );
    let guns = if accurate {
        format!(
            "\n{spread}    vec2 miss = vec2(pictureOf(face + vec2(spread, 0.0)).x - before.x, 0.0);
    vec3 lineMean = vec3(0.0);
    vec3 linePeak = vec3(0.0);
    vec3 guns = vec3(
        beamOf(before + vec2(shift.r * perPixel.x, 0.0) - miss, tall, 0.0, frameCount, lineMean.r, linePeak.r),
        beamOf(before + vec2(shift.g * perPixel.x, 0.0), tall, 1.0, frameCount, lineMean.g, linePeak.g),
        beamOf(before + vec2(shift.b * perPixel.x, 0.0) + miss, tall, 2.0, frameCount, lineMean.b, linePeak.b));
"
        )
    } else if performance {
        "
    vec3 lineMean = vec3(0.0);
    vec3 linePeak = vec3(0.0);
    vec3 guns = beamsOf(before, tall, vec3(before.x), frameCount, lineMean, linePeak);
"
        .to_string()
    } else {
        format!(
            "\n{spread}    float miss = pictureOf(face + vec2(spread, 0.0)).x - before.x;
    vec3 lineMean = vec3(0.0);
    vec3 linePeak = vec3(0.0);
    vec3 guns = beamsOf(before, tall, before.x + shift * perPixel.x + vec3(-miss, 0.0, miss), frameCount, lineMean, linePeak);
"
        )
    };
    let rows = if accurate { "beamSize.y" } else { "gunSize.y" };
    let (m0, m1, m2) = (row(&m[0]), row(&m[1]), row(&m[2]));
    let dots = |of: &str| format!("vec3(dot({m0}, {of}), dot({m1}, {of}), dot({m2}, {of}))");
    let glass = if performance {
        format!("\n    vec3 direct = {};\n    vec3 even = {};\n    vec3 peak = {};\n    vec3 veil = vec3(0.0);\n", dots("guns"), dots("lineMean"), dots("linePeak"))
    } else {
        format!(
            "\n    float spill = {} * glare;\n    vec3 scattered = haze_eval((face / vec2(1.0, 0.75) * 0.5 + 0.5) * hazeSize).rgb;\n    vec3 direct = (1.0 - spill) * {};\n    vec3 even = (1.0 - spill) * {};\n    vec3 peak = (1.0 - spill) * {};\n    vec3 veil = spill * {};\n",
            n(GLARE_WEIGHT),
            dots("guns"),
            dots("lineMean"),
            dots("linePeak"),
            dots("scattered")
        )
    };

    let mut t = String::from(
        "
uniform vec2 outputSize;
uniform float pixelScale;
uniform vec2 pixelOrigin;
uniform float frameCount;
uniform float displayNits;
uniform float tubeNits;
uniform float level;
uniform float subpixels;
uniform float maskPitch;
uniform float maskLimit;
uniform float maskDepth;
uniform float interlace;
",
    );
    t += if performance { "uniform float curvature;\n" } else { "uniform sampler2D haze;\nuniform vec2 hazeSize;\nuniform float curvature;\nuniform float glare;\n" };
    if !performance {
        t += "uniform float convergence;\n";
    }
    t += glsl::FUNCTIONS;
    t += RASTER;
    t += glsl::COVERAGE;
    t += &beams;
    t += "\n\nvec2 faceOf(vec2 screen) {\n    if (curvature <= 0.0) return screen;\n";
    t += &format!("    float radius = {} / curvature;\n", n(radius));
    t += &format!("    float eye = {};\n", n(distance));
    t += "    float edgeX = radius - sqrt(radius * radius - 1.0);\n";
    t += &format!("    float edgeY = {};\n", if r.screen.cylinder { "0.0" } else { "radius - sqrt(radius * radius - 0.5625)" });
    t += "    vec2 s = screen * vec2(eye / (eye + edgeX), eye / (eye + edgeY));\n";
    t += &format!("    float a = {} + eye * eye;\n", if r.screen.cylinder { "s.x * s.x" } else { "dot(s, s)" });
    t += "    float b = eye * (eye + radius);
    float c = eye * eye + 2.0 * eye * radius;
    float t = (b - sqrt(max(b * b - a * c, 0.0))) / a;
    return s * t;
}

vec3 encode(vec3 c) {
    return vec3(c.r <= 0.0031308 ? c.r * 12.92 : 1.055 * pow(c.r, 0.41666667) - 0.055,
                  c.g <= 0.0031308 ? c.g * 12.92 : 1.055 * pow(c.g, 0.41666667) - 0.055,
                  c.b <= 0.0031308 ? c.b * 12.92 : 1.055 * pow(c.b, 0.41666667) - 0.055);
}

vec4 filter_main(vec2 coord) {
    vec2 screen = (coord / outputSize * 2.0 - 1.0) * vec2(1.0, 0.75);
    vec2 pixel = 2.0 / (outputSize * pixelScale) * vec2(1.0, 0.75);
    vec2 face = faceOf(screen);
    vec2 grain = vec2(faceOf(screen + vec2(pixel.x, 0.0)).x - face.x, faceOf(screen + vec2(0.0, pixel.y)).y - face.y);

";
    t += &format!("    vec2 corner = max(abs(face) - vec2(1.0, 0.75) + {}, 0.0);\n", n(CORNER_RADIUS));
    t += &format!("    float inside = clamp(({} - length(corner)) / max(grain.x, 0.000001) + 0.5, 0.0, 1.0);\n", n(CORNER_RADIUS));
    t += "    if (inside <= 0.0) return vec4(0.0, 0.0, 0.0, 1.0);

    vec3 shift = subpixels < 0.5 ? vec3(0.0) : subpixels < 1.5 ? vec3(-1.0, 0.0, 1.0) / 3.0 : vec3(1.0, 0.0, -1.0) / 3.0;
    // A triad's width in display pixels: the screen's own, kept fine on a large display; up to that limit a whole number of the display's own pixels, and let go below one and a float - see EmuSen_CRT.md §18 and §20.
    float wide = outputSize.x * pixelScale;
";
    t += &format!("    float triadPixels = wide * maskPitch / {};\n", n(r.screen.triads_across));
    t += &format!("    float finest = max(floor({} * maskPitch + 0.5), 2.0);\n", n(FINEST_TRIAD_PIXELS));
    t += "    if (maskLimit > 0.5) triadPixels = min(triadPixels, finest);\n";
    t += &format!("    float drawn = clamp((triadPixels - {}) / 0.25, 0.0, 1.0);\n", n(LEAST_TRIAD_PIXELS - 0.25));
    t += &format!("    bool flatMask = triadPixels < max(finest, {}) + 0.001;\n", n(FINEST_TRIAD_PIXELS));
    t += "    vec2 triad;
    vec2 footprint;
    float stepX;
    if (flatMask) {
        triadPixels = max(floor(triadPixels + 0.5), 2.0);
";
    t += &format!(
        "        float rowsDown = {};\n",
        if layout.vertical_pitch > 0.0 { format!("max(floor({} * triadPixels + 0.5), 2.0) / ({} * triadPixels)", n(layout.vertical_pitch), n(layout.vertical_pitch)) } else { "1.0".to_string() }
    );
    t += "        triad = (coord * pixelScale + pixelOrigin) / triadPixels * vec2(1.0, 1.0 / rowsDown);\n";
    t += &format!("        footprint = vec2(1.0, 1.0 / rowsDown) / triadPixels{};\n", if accurate { "" } else { " * 0.5" });
    t += "        stepX = 1.0 / triadPixels;
    } else {
        float across = 0.5 * wide / triadPixels;
        triad = face * across;
";
    t += &format!("        footprint = grain * across{};\n", if accurate { "" } else { " * 0.5" });
    t += "        stepX = grain.x * across;
    }
    vec3 cover;
";
    t += &mask;
    t += &format!("\n    float litFraction = {};\n", n(layout.lit_fraction()));
    t += &format!(
        "
    vec2 before = pictureOf(face);
    vec2 perPixel = vec2(pictureOf(face + vec2(grain.x, 0.0)).x - before.x, pictureOf(face + vec2(0.0, grain.y)).y - before.y);
    float tall = perPixel.y * {rows};
"
    );
    t += &guns;
    t += &glass;
    t += "\n    float luminance = dot(direct, vec3(0.2126, 0.7152, 0.0722));
    float lowest = min(direct.r, min(direct.g, direct.b));
    if (lowest < 0.0) direct = luminance + (direct - luminance) * (luminance / max(luminance - lowest, 0.000001));

";
    t += &format!("    float headroom = level < 0.5 ? {} : displayNits / tubeNits;\n", n(1.0 / BRIGHT_WHITE));
    t += "
    // A scanline brighter at its centre than the display can go is drawn flatter, toward its line's even light, by just enough for its peak.
    direct = even + (direct - even) * clamp((headroom - even) / max(peak - even, vec3(0.000001)), 0.0, 1.0);
    vec3 depth = drawn * maskDepth * clamp((headroom / max(direct, vec3(0.000001)) - 1.0) / (1.0 / litFraction - 1.0 + 0.000001), 0.0, 1.0);
    vec3 shown = (direct * (1.0 + depth * (cover / litFraction - 1.0)) + veil) / headroom;
";
    t += if performance {
        "    return vec4(encode(clamp(shown * inside, 0.0, 1.0)), 1.0);\n}"
    } else {
        "
    vec2 onDisplay = coord * pixelScale + pixelOrigin;
    float noise = fract(52.9829189 * fract(dot(floor(onDisplay), vec2(0.06711056, 0.00583715)))) - 0.5;
    return vec4(clamp(encode(clamp(shown * inside, 0.0, 1.0)) + noise / 255.0, 0.0, 1.0), 1.0);
}"
    };
    t
}
