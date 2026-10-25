//! S26's gate programs (Sapphire_Native.md §32.7). Each scene leaves its sound in one of the output areas (D-261),
//! which the program copies into its record: core 0's voice 1 at 400h for the voices, core 0's output at 800h and the
//! voice sums at 1000h for the mix, the effects area for the reverb, MEMIN for AutoDMA. A copy is graded by `Check`:
//! the area holds 200h samples written in a ring, so it must equal some 200h-sample window of the scene's record, read
//! from the ring's write position. The record of a voice is psx-spx's rules as `blackmoon-spu` holds them, run here on
//! the program's own blocks apart from `src/spu2/`; the mix is the documented and measured rules written out below.

use super::{dma, dump, r, rec_h, reg, reset, sh, vol};
use crate::s21::{self, Program};
use blackmoon_spu::noise::Noise;
use blackmoon_spu::voice::{self, Voice};

/// Samples an output area holds.
pub const RING: usize = 0x200;

/// What a copy of an area must hold.
pub enum What {
    /// A window of this record, read cyclically from the write position.
    Window(Vec<i16>),
    /// Zero throughout.
    Zero,
}

pub struct Check {
    pub name: String,
    /// The record word the copy begins at, and its halfwords.
    pub at: usize,
    pub halves: usize,
    pub what: What,
}

/// A gate program, its checks, and sound RAM as the program leaves it for the record's voices.
pub struct Gate {
    pub program: Program,
    pub checks: Vec<Check>,
    pub ram: Vec<u8>,
}

/// A voice of a scene: volumes, pitch, ADSR and start address (halfwords).
#[derive(Clone, Copy)]
pub struct Set {
    pub vols: [u16; 2],
    pub pitch: u16,
    pub adsr: [u16; 2],
    pub start: u32,
}

impl Set {
    pub const fn new(pitch: u16, adsr: [u16; 2], start: u32) -> Set {
        Set { vols: [0x3FFF, 0x3FFF], pitch, adsr, start }
    }

    fn voice(&self) -> Voice {
        Voice { volume: self.vols, pitch: self.pitch, adsr: self.adsr, start: self.start >> 2, ..Voice::default() }
    }
}

/// A rolling hash's base.
const BASE: u64 = 0x100_0000_01B3;

/// Where `ring`, read cyclically from some position, equals a window of `model`: the window's start, if any.
pub fn window(ring: &[i16], model: &[i16]) -> Option<usize> {
    let n = ring.len();
    if model.len() < n || n == 0 {
        return None;
    }
    let pow = (1..n).fold(1u64, |a, _| a.wrapping_mul(BASE));
    let v = |x: i16| x as u16 as u64 + 1;
    let mut rot = std::collections::HashMap::new();
    let mut h = 0u64;
    for &x in ring {
        h = h.wrapping_mul(BASE).wrapping_add(v(x));
    }
    for (r, &x) in ring.iter().enumerate() {
        rot.entry(h).or_insert(r);
        h = h.wrapping_sub(v(x).wrapping_mul(pow)).wrapping_mul(BASE).wrapping_add(v(x));
    }
    let mut h = 0u64;
    for &x in &model[..n] {
        h = h.wrapping_mul(BASE).wrapping_add(v(x));
    }
    for o in 0..=model.len() - n {
        if let Some(&r) = rot.get(&h)
            && (0..n).all(|j| ring[(r + j) % n] == model[o + j])
        {
            return Some(o);
        }
        if o + n < model.len() {
            h = h.wrapping_sub(v(model[o]).wrapping_mul(pow)).wrapping_mul(BASE).wrapping_add(v(model[o + n]));
        }
    }
    None
}

/// A record's halfwords from its words, low first.
pub fn halves(words: &[u32]) -> Vec<i16> {
    words.iter().flat_map(|w| [*w as u16 as i16, (*w >> 16) as u16 as i16]).collect()
}

/// Grades the checks against a run's record words: the failures, each named.
pub fn grade(checks: &[Check], record: &[u32]) -> Vec<String> {
    let mut bad = Vec::new();
    for c in checks {
        let h = halves(&record[c.at..c.at + c.halves / 2]);
        let ok = match &c.what {
            What::Window(model) => window(&h, model).is_some(),
            What::Zero => h.iter().all(|&x| x == 0),
        };
        if !ok {
            let nz = h.iter().position(|&x| x != 0).unwrap_or(0);
            bad.push(format!("{}: from {nz} {:?}", c.name, &h[nz..(nz + 12).min(h.len())]));
        }
    }
    bad
}

/// SPU-ADPCM with a pseudo-random sequence of nibbles: `blocks` blocks of the shifts and filters given in turn, the
/// first with the loop start flag, the last with loop end and repeat.
pub fn sample(blocks: usize, shifts: &[u8], filters: &[u8], seed: u32) -> Vec<u16> {
    let mut x = seed;
    let mut out = Vec::new();
    for b in 0..blocks {
        let flags = if b == 0 { 4 } else { 0 } | if b + 1 == blocks { 3 } else { 0 };
        let nib: Vec<u8> = (0..28)
            .map(|_| {
                x = x.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                (x >> 16) as u8 & 0xF
            })
            .collect();
        out.extend_from_slice(&super::block(shifts[b % shifts.len()], filters[b % filters.len()], flags, |i| nib[i]));
    }
    out
}

/// A constant block: 28 samples of nibble `n` at shift 0 and filter 0, looping on itself.
pub fn constant(n: u8) -> [u16; 8] {
    super::block(0, 0, 7, |_| n)
}

impl Gate {
    pub fn new(name: &'static str) -> Gate {
        let mut p = Program::new(name);
        reset(&mut p);
        for c in 0..2 {
            sh(&mut p, vol(c, 0), 0x3FFF);
            sh(&mut p, vol(c, 1), 0x3FFF);
        }
        Gate { program: p, checks: Vec::new(), ram: vec![0; sapphire::spu2::RAM_BYTES] }
    }

    pub fn upload(&mut self, at: u32, data: &[u16]) {
        super::upload(&mut self.program, 0, at, data);
        for (i, h) in data.iter().enumerate() {
            let b = 2 * (at as usize + i);
            self.ram[b..b + 2].copy_from_slice(&h.to_le_bytes());
        }
    }

    /// Copies the area at `at` into the record and checks it.
    pub fn check(&mut self, name: impl Into<String>, at: u32, what: What) {
        let i = self.program.expected.len();
        dump(&mut self.program, at, RING as u32);
        self.checks.push(Check { name: name.into(), at: i, halves: RING, what });
    }

    /// Keys every voice of both cores off with a fast release, and waits for silence.
    pub fn silence(&mut self) {
        let p = &mut self.program;
        for c in 0..2 {
            for v in 0..4 {
                sh(p, reg(c, r::voice(v, 4)), 0x0000);
            }
            sh(p, reg(c, r::KOFF), 0xFFFF);
            sh(p, reg(c, r::KOFF + 2), 0xFF);
            for o in [r::PMON, r::NON] {
                sh(p, reg(c, o), 0);
                sh(p, reg(c, o + 2), 0);
            }
        }
        p.delay(200_000);
    }

    pub fn set(&mut self, c: u32, v: u32, s: Set) {
        super::voice(&mut self.program, c, v, [s.vols[0] as u32, s.vols[1] as u32], s.pitch as u32, [s.adsr[0] as u32, s.adsr[1] as u32], s.start);
    }

    /// The record of core 0's voices from a key on of the voices in `sets` together (voice n modulated by voice n-1
    /// where `pmon` has bit n), `lead` silent samples first, then `n` samples, voice `which`'s output.
    pub fn model(&self, sets: &[Set], pmon: u32, which: usize, n: usize, lead: usize) -> Vec<i16> {
        let mut voices: Vec<Voice> = sets.iter().map(Set::voice).collect();
        for v in voices.iter_mut() {
            v.key_on(&self.ram);
        }
        let mut out = vec![0i16; lead];
        for _ in 0..n {
            let mut previous = 0i16;
            for (k, v) in voices.iter_mut().enumerate() {
                let step = voice::step(v.pitch, (k > 0 && pmon & 1 << k != 0).then_some(previous));
                v.tick(&self.ram, step, None, &mut ());
                previous = v.out;
            }
            out.push(voices[which].out);
        }
        out
    }

    /// A voice scene on core 0: the voices of `sets` (voice 0 first) keyed on together from silence, `pmon` written,
    /// a wait of `passes`, then voice 1's area checked against the record.
    pub fn voice_scene(&mut self, name: &str, sets: &[Set], pmon: u32, passes: u32) {
        self.silence();
        for (v, s) in sets.iter().enumerate() {
            self.set(0, v as u32, *s);
        }
        sh(&mut self.program, reg(0, r::PMON), pmon);
        sh(&mut self.program, reg(0, r::KON), (1 << sets.len()) - 1);
        self.program.delay(passes);
        let model = self.model(sets, pmon, 1, passes as usize * 4 / 768 * 4 + 4 * RING, 2 * RING);
        self.check(name, 0x400, What::Window(model));
    }

    pub fn finish(self) -> (Program, Vec<Check>) {
        (self.program, self.checks)
    }
}

/// A short wait, about 260 samples on either machine.
const SHORT: u32 = 66_000;
/// A long one, about 1,300.
const LONG: u32 = 330_000;

/// `s26-voices`: SPU-ADPCM through every filter and shift, the interpolation at several pitches, the pitch's limit,
/// and pitch modulation, each as core 0's voice 1 records it.
pub fn voices() -> Gate {
    let mut g = Gate::new("s26-voices");
    let adsr = [0x000F, 0x1FC0];
    let streams: [(&str, Vec<u16>); 4] = [
        ("filters 0-4 at shift 4", sample(10, &[4], &[0, 1, 2, 3, 4], 1)),
        ("shifts 0-12 at filter 1", sample(13, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12], &[1], 2)),
        ("reserved shifts 13-15 and filters 5-7", sample(6, &[13, 14, 15, 9], &[5, 6, 7, 2], 3)),
        ("filter 4 at shift 0, saturating", sample(8, &[0], &[4, 3], 4)),
    ];
    let mut at = 0x6000;
    let mut starts = Vec::new();
    for (_, data) in &streams {
        g.upload(at, data);
        starts.push(at);
        at += data.len() as u32 + 0x10;
    }
    g.upload(0x5000, &super::square());
    g.upload(0x5010, &constant(4));
    let quiet = Set::new(0, adsr, 0x5010);
    g.voice_scene("a key on's first samples", &[quiet, Set::new(0x1000, adsr, 0x5000)], 0, SHORT);
    for (k, (name, _)) in streams.iter().enumerate() {
        g.voice_scene(name, &[quiet, Set::new(0x1000, adsr, starts[k])], 0, LONG);
    }
    for pitch in [0x0400u16, 0x0C00, 0x1555, 0x2FFF, 0x4000, 0x7FFF] {
        g.voice_scene(&format!("pitch {pitch:04X}"), &[quiet, Set::new(pitch, adsr, starts[0])], 0, LONG);
    }
    g.voice_scene("pitch modulation by a constant voice", &[Set::new(0x1000, adsr, 0x5010), Set::new(0x0800, adsr, starts[1])], 2, LONG);
    g.voice_scene("pitch modulation by a square voice", &[Set::new(0x0700, adsr, 0x5000), Set::new(0x1000, adsr, starts[0])], 2, LONG);
    g.voice_scene("no modulation without PMON", &[Set::new(0x1000, adsr, 0x5010), Set::new(0x0800, adsr, starts[1])], 0, LONG);
    g
}

/// `s26-envelope`: ADSR's attack, decay, sustain and release in each mode, on a constant voice at pitch 1000h.
pub fn envelope() -> Gate {
    let mut g = Gate::new("s26-envelope");
    g.upload(0x5010, &constant(4));
    let quiet = Set::new(0, [0x000F, 0x1FC0], 0x5010);
    let scenes: [(&str, [u16; 2]); 8] = [
        ("linear attack, shift 10 step 1", [0x290F, 0x1FC0]),
        ("exponential attack, shift 8 step 0", [0xA00F, 0x1FC0]),
        ("decay shift 6 to level 7", [0x0067, 0x1FC0]),
        ("decay shift 9 to level 2", [0x0092, 0x1FC0]),
        ("sustain linear increase, shift 12", [0x0005, 0x0C00]),
        ("sustain linear decrease, shift 11", [0x000A, 0x4B00]),
        ("sustain exponential decrease, shift 8", [0x000F, 0xC840]),
        ("sustain exponential increase, shift 10", [0x0003, 0x8A00]),
    ];
    for (name, adsr) in scenes {
        g.voice_scene(name, &[quiet, Set::new(0x1000, adsr, 0x5010)], 0, LONG);
    }
    for (name, adsr2) in [("linear release, shift 9", 0x1FC9u16), ("exponential release, shift 6", 0x1FE6)] {
        g.silence();
        let s = Set::new(0x1000, [0x000F, adsr2], 0x5010);
        g.set(0, 0, quiet);
        g.set(0, 1, s);
        sh(&mut g.program, reg(0, r::KON), 2);
        g.program.delay(LONG);
        sh(&mut g.program, reg(0, r::KOFF), 2);
        g.program.delay(SHORT);
        let mut v = s.voice();
        v.key_on(&g.ram);
        let mut model = Vec::new();
        for k in 0..30_000 {
            if k == 20_000 {
                v.key_off();
            }
            v.tick(&g.ram, 0x1000, None, &mut ());
            model.push(v.out);
        }
        g.check(name, 0x400, What::Window(model));
    }
    g
}

/// `s26-noise`: voice 1 in noise mode at the fastest and a slower clock, against the noise generator's whole cycle.
pub fn noise() -> Gate {
    let mut g = Gate::new("s26-noise");
    g.upload(0x5010, &constant(4));
    for (shift, step) in [(15u16, 3u16), (12, 1)] {
        g.silence();
        g.set(0, 1, Set::new(0x1000, [0x000F, 0x1FC0], 0x5010));
        let a = shift << 10 | step << 8;
        sh(&mut g.program, reg(0, r::ATTR), a as u32);
        sh(&mut g.program, reg(0, r::NON), 2);
        sh(&mut g.program, reg(0, r::KON), 2);
        g.program.delay(LONG);
        let mut v = Set::new(0x1000, [0x000F, 0x1FC0], 0x5010).voice();
        v.key_on(&g.ram);
        for _ in 0..64 {
            v.tick(&g.ram, 0x1000, Some(0), &mut ());
        }
        let level = v.envelope.level as i32;
        // Each of the five phases a bit can begin with against the timer (D-269), a whole cycle of 65,535 bits each.
        let hold = (0x2_0000 >> shift) / (step as usize + 4) + 1;
        let mut model = Vec::new();
        for t in 0..5 {
            let mut n = Noise { level: 0, timer: t };
            model.extend((0..(0x1_0000 + 2 * RING) * hold).map(|_| {
                n.tick(shift, step);
                ((n.level as i32 * level) >> 15) as i16
            }));
        }
        g.check(format!("noise shift {shift} step {step}"), 0x400, What::Window(model));
    }
    sh(&mut g.program, reg(0, r::ATTR), 0);
    g
}

/// The record of core 0's output and voice sums for voice 0 alone: its volumes, the masks, MMIX and MVOL (levels from
/// the volume registers' direct form, D-262's mix).
fn mix_model(g: &Gate, s: Set, masks: [bool; 4], mmix: u32, mvol: [u16; 2], n: usize) -> [Vec<i16>; 6] {
    let mut v = s.voice();
    v.key_on(&g.ram);
    let mut out: [Vec<i16>; 6] = Default::default();
    for o in out.iter_mut() {
        o.extend(std::iter::repeat_n(0, 2 * RING));
    }
    let level = |reg: u16| (reg << 1) as i16 as i32;
    for _ in 0..n {
        v.tick(&g.ram, voice::step(v.pitch, None), None, &mut ());
        let lr = v.volumes();
        let dry = [if masks[0] { lr[0] } else { 0 }, if masks[1] { lr[1] } else { 0 }];
        let wet = [if masks[2] { lr[0] } else { 0 }, if masks[3] { lr[1] } else { 0 }];
        for s in 0..2 {
            let gated = if (mmix >> (11 - s)) & 1 != 0 { dry[s] } else { 0 };
            out[s].push(((gated.clamp(-0x8000, 0x7FFF) * level(mvol[s])) >> 15).clamp(-0x8000, 0x7FFF) as i16);
            out[2 + s].push(dry[s].clamp(-0x8000, 0x7FFF) as i16);
            out[4 + s].push(wet[s].clamp(-0x8000, 0x7FFF) as i16);
        }
    }
    out
}

/// `s26-mix`: core 0's voice 0 through its dry and wet masks by side, MMIX's voice gates and the master volume, as the
/// output and sum areas record them; core 1's voice sums and its voice 1 likewise.
pub fn mixing() -> Gate {
    let mut g = Gate::new("s26-mix");
    g.upload(0x5000, &super::square());
    let base = Set { vols: [0x1000, 0x2000], ..Set::new(0x1000, [0x000F, 0x1FC0], 0x5000) };
    let scenes: [(&str, [bool; 4], u32, [u16; 2]); 4] = [
        ("dry both sides, MMIX's voice gates, MVOL full", [true, true, false, false], 0xC00, [0x3FFF, 0x3FFF]),
        ("dry left, wet right, the left gate alone, MVOL half", [true, false, false, true], 0x800, [0x2000, 0x2000]),
        ("wet both, the right gate alone", [false, false, true, true], 0x400, [0x3FFF, 0x1000]),
        ("dry right, no gate", [false, true, true, false], 0x000, [0x3FFF, 0x3FFF]),
    ];
    let names = ["out L", "out R", "dry L", "dry R", "wet L", "wet R"];
    for (name, masks, mmix, mvol) in scenes {
        g.silence();
        g.set(0, 0, base);
        for (k, o) in [r::VMIXL, r::VMIXR, r::VMIXEL, r::VMIXER].into_iter().enumerate() {
            sh(&mut g.program, reg(0, o), masks[k] as u32);
        }
        sh(&mut g.program, reg(0, r::MMIX), mmix);
        sh(&mut g.program, vol(0, 0), mvol[0] as u32);
        sh(&mut g.program, vol(0, 1), mvol[1] as u32);
        sh(&mut g.program, reg(0, r::KON), 1);
        g.program.delay(LONG);
        let models = mix_model(&g, base, masks, mmix, mvol, LONG as usize * 4 / 768 * 4 + 4 * RING);
        for (k, m) in models.into_iter().enumerate() {
            g.check(format!("{name}: {}", names[k]), [0x800, 0xA00, 0x1000, 0x1200, 0x1400, 0x1600][k], What::Window(m));
        }
    }
    // Core 1: voice 0 dry left and wet right, voice 1 keyed with it.
    g.silence();
    let v1 = Set::new(0x0C00, [0x000F, 0x1FC0], 0x5000);
    super::voice(&mut g.program, 1, 0, [0x1000, 0x2000], 0x1000, [0x000F, 0x1FC0], 0x5000);
    g.set(1, 1, v1);
    for (o, v) in [(r::VMIXL, 1), (r::VMIXR, 0), (r::VMIXEL, 0), (r::VMIXER, 1)] {
        sh(&mut g.program, reg(1, o), v);
    }
    sh(&mut g.program, reg(1, r::KON), 3);
    g.program.delay(LONG);
    let n = LONG as usize * 4 / 768 * 4 + 4 * RING;
    let models = mix_model(&g, base, [true, false, false, true], 0, [0, 0], n);
    let [_, _, dry_l, dry_r, wet_l, wet_r] = models;
    g.check("core 1: dry L", 0x1800, What::Window(dry_l));
    g.check("core 1: dry R", 0x1A00, What::Window(dry_r));
    g.check("core 1: wet L", 0x1C00, What::Window(wet_l));
    g.check("core 1: wet R", 0x1E00, What::Window(wet_r));
    let m = g.model(&[base, v1], 0, 1, n, 2 * RING);
    g.check("core 1: voice 1", 0xC00, What::Window(m));
    g.check("core 0's voice 3, silent", 0x600, What::Zero);
    g
}

/// The record of the effects area after a long constant input: every halfword the reverb writes settles to one
/// value, which `blackmoon-spu`'s reverb gives when run here on the same registers from silence.
fn reverb_model(params: &blackmoon_spu::reverb::Params, area: (u32, u32), input: i32, samples: usize) -> Vec<i16> {
    let mut ram = vec![0u8; sapphire::spu2::RAM_BYTES];
    let mut r = blackmoon_spu::reverb::Reverb { address: area.0 * 2, ..Default::default() };
    let a = blackmoon_spu::reverb::Area { start: area.0 * 2, end: (area.1 + 1) * 2 };
    for _ in 0..samples {
        r.tick(&mut ram, &mut (), params, a, true, [input, input]);
    }
    let n = (area.1 + 1 - area.0) as usize;
    let mut out: Vec<i16> = (0..n).map(|k| i16::from_le_bytes([ram[2 * (area.0 as usize + k)], ram[2 * (area.0 as usize + k) + 1]])).collect();
    let copy = out.clone();
    out.extend(copy);
    out
}

/// `s26-reverb`: core 0's reverb in an area of 1000h halfwords at 1F000h, on with a constant voice wet until the area
/// settles, then the area against the record; with the reverb off, an area untouched.
pub fn reverb() -> Gate {
    let mut g = Gate::new("s26-reverb");
    g.upload(0x5010, &constant(4));
    let offsets: [u32; 22] = [0x80, 0x40, 0x200, 0x210, 0x300, 0x310, 0x340, 0x350, 0x100, 0x110, 0x400, 0x410, 0x380, 0x390, 0x3C0, 0x3D0, 0x180, 0x190, 0x600, 0x610, 0x700, 0x710];
    let volumes: [u16; 10] = [0x6000, 0x4000, 0x3000, 0x2000, 0x1000, 0x4000, 0x3000, 0x2800, 0x7FFF, 0x7FFF];
    let program_area = |g: &mut Gate, esa: u32| {
        let p = &mut g.program;
        sh(p, reg(0, r::ESAH), esa >> 16);
        sh(p, reg(0, r::ESAL), esa & 0xFFFF);
        sh(p, reg(0, r::EEA), esa >> 16);
        for (k, o) in offsets.iter().enumerate() {
            sh(p, reg(0, r::REVERB + 4 * k as u32), o >> 16);
            sh(p, reg(0, r::REVERB + 4 * k as u32 + 2), o & 0xFFFF);
        }
        for (k, v) in volumes.iter().enumerate() {
            sh(p, vol(0, 10 + k as u32), *v as u32);
        }
        sh(p, vol(0, 2), 0x7FFF);
        sh(p, vol(0, 3), 0x7FFF);
    };
    // Off: the area at 2F000h stays as it was, zero.
    g.silence();
    program_area(&mut g, 0x2F000);
    let s = Set::new(0x1000, [0x000F, 0x1FC0], 0x5010);
    g.set(0, 0, s);
    sh(&mut g.program, reg(0, r::VMIXEL), 1);
    sh(&mut g.program, reg(0, r::VMIXER), 1);
    sh(&mut g.program, reg(0, r::MMIX), 0x300);
    sh(&mut g.program, reg(0, r::KON), 1);
    g.program.delay(LONG);
    g.check("reverb off: the area untouched", 0x2F000, What::Zero);
    // On: the area at 1F000h-1FFFFh.
    program_area(&mut g, 0x1F000);
    sh(&mut g.program, reg(0, r::ATTR), 0x0080);
    g.program.delay(LONG * 40);
    sh(&mut g.program, reg(0, r::ATTR), 0);
    let mut v = s.voice();
    v.key_on(&g.ram);
    for _ in 0..64 {
        v.tick(&g.ram, 0x1000, None, &mut ());
    }
    let input = v.volumes()[0];
    let s2 = |x: u16| x as i16 as i32;
    let m = |k: usize| offsets[k] as i32 * 2;
    let params = blackmoon_spu::reverb::Params {
        input: [s2(volumes[8]), s2(volumes[9])],
        iir: s2(volumes[0]),
        wall: s2(volumes[5]),
        comb: [s2(volumes[1]), s2(volumes[2]), s2(volumes[3]), s2(volumes[4])],
        apf: [s2(volumes[6]), s2(volumes[7])],
        output: [0x7FFF, 0x7FFF],
        d_apf: [m(0), m(1)],
        same: [m(2), m(3)],
        comb_at: [[m(4), m(5)], [m(6), m(7)], [m(12), m(13)], [m(14), m(15)]],
        d_same: [m(8), m(9)],
        diff: [m(10), m(11)],
        d_diff: [m(16), m(17)],
        apf_at: [[m(18), m(19)], [m(20), m(21)]],
    };
    let model = reverb_model(&params, (0x1F000, 0x1FFFF), input, 400_000);
    let i = g.program.expected.len();
    dump(&mut g.program, 0x1F000, 0x1000);
    g.checks.push(Check { name: "reverb on: the area, settled".into(), at: i, halves: 0x1000, what: What::Window(model) });
    g
}

/// `s26-adma`: four AutoDMA blocks streamed on core 0, each written while the core plays the other buffer, its sound
/// data input through MMIX's dry gates and BVOL into core 0's output, which must have played the first three in order
/// and not yet the fourth when the transfer ends; then MEMIN as ps2tek lays it out, holding the last two.
pub fn adma() -> Gate {
    let mut g = Gate::new("s26-adma");
    let blocks = 4u32;
    let pcm = |b: u32, side: u32, i: u32| -> u16 { (0x1000 * (b + 1) + 0x4000 * side + 3 * i) as u16 ^ if side == 1 { 0x8000 } else { 0 } };
    for b in 0..blocks {
        for side in 0..2u32 {
            for i in 0..128u32 {
                let at = s21::K1 | (s21::AREA + 0x1000 + 0x400 * b + 0x200 * side + 4 * i);
                g.program.poke(at, pcm(b, side, 2 * i) as u32 | (pcm(b, side, 2 * i + 1) as u32) << 16);
            }
        }
    }
    g.silence();
    sh(&mut g.program, reg(0, r::MMIX), 0xC0);
    sh(&mut g.program, vol(0, 6), 0x3FFF);
    sh(&mut g.program, vol(0, 7), 0x2000);
    sh(&mut g.program, reg(0, r::ADMA), 1);
    dma(&mut g.program, 0, s21::AREA + 0x1000, 16, 16 * blocks, true);
    g.program.delay(16_000);
    let gains = [(0x3FFFu16 << 1) as i16 as i32, (0x2000u16 << 1) as i16 as i32];
    for (s, (name, at)) in [("the blocks in order, left, through BVOLL", 0x800u32), ("the blocks in order, right, through BVOLR", 0xA00)].into_iter().enumerate() {
        // The last block is written as the one before it begins, so it has not yet sounded when the transfer ends.
        let mut m = vec![0i16; 2 * RING];
        for b in 0..blocks - 1 {
            m.extend((0..256).map(|i| ((((pcm(b, s as u32, i) as i16 as i32 * gains[s]) >> 15) * 0x7FFE) >> 15) as i16));
        }
        g.check(name, at, What::Window(m));
    }
    rec_h(&mut g.program, reg(0, r::ADMA), Some(1));
    sh(&mut g.program, reg(0, r::ADMA), 0);
    for (side, name) in ["MEMIN left", "MEMIN right"].iter().enumerate() {
        let m: Vec<i16> = (blocks - 2..blocks).flat_map(|b| (0..256).map(move |i| pcm(b, side as u32, i) as i16)).collect();
        let doubled = [m.clone(), m].concat();
        g.check(*name, 0x2000 + 0x200 * side as u32, What::Window(doubled));
    }
    g
}

/// `s26-irq`: the IRQ address met by a voice's header and its sample words, by a transfer's halfword and not its
/// neighbour, by core 0's voice on core 1's address, by every output area's writes and by a reverb's reads; the flags
/// in 7C2h, IRQ 9 in I_STAT, and ATTR's enable clearing a flag (Sapphire_Disputes.md D-265).
pub fn irq() -> Gate {
    let mut g = Gate::new("s26-irq");
    g.upload(0x5000, &super::square());
    // Both reverbs' areas out of the way at 70000h-7FFFFh, since their reads are watched too.
    for c in 0..2 {
        sh(&mut g.program, reg(c, r::ESAH), 7);
        sh(&mut g.program, reg(c, r::ESAL), 0);
        sh(&mut g.program, reg(c, r::EEA), 7);
    }
    g.silence();
    g.set(0, 0, Set::new(0x1000, [0x000F, 0x1FC0], 0x5000));
    sh(&mut g.program, reg(0, r::KON), 1);
    g.program.delay(SHORT);
    let p = &mut g.program;
    let arm = |p: &mut Program, c: u32, at: u32| {
        sh(p, reg(0, r::ATTR), 0);
        sh(p, reg(1, r::ATTR), 0);
        p.poke(s21::reg::I_STAT, 0);
        sh(p, reg(c, r::IRQAH), at >> 16);
        sh(p, reg(c, r::IRQAL), at & 0xFFFF);
        sh(p, reg(c, r::ATTR), 0x0040);
    };
    for (at, hit) in [(0x5000u32, true), (0x5005, true), (0x5008, false), (0x0410, true), (0x0C10, true), (0x1010, true), (0x1C10, true), (0x0810, true), (0x0A10, true)] {
        arm(p, 0, at);
        p.delay(200_000);
        p.rec_masked(s21::reg::I_STAT, 0x200, if hit { 0x200 } else { 0 });
        rec_h(p, super::SPU2 + 0x7C2, Some(if hit { 4 } else { 0 }));
    }
    // A transfer through core 0's data port over 6000h-6003h.
    for (at, hit) in [(0x6003u32, true), (0x6004, false)] {
        arm(p, 0, at);
        sh(p, reg(0, r::TSAH), 0);
        sh(p, reg(0, r::TSAL), 0x6000);
        for _ in 0..4 {
            sh(p, reg(0, r::DATA), 0x1234);
        }
        p.delay(2_000);
        p.rec_masked(s21::reg::I_STAT, 0x200, if hit { 0x200 } else { 0 });
        rec_h(p, super::SPU2 + 0x7C2, Some(if hit { 4 } else { 0 }));
    }
    // A reverb's reads: core 0's area shrunk to 7FF00h-7FFFFh, whose every halfword its buffer address passes.
    arm(p, 0, 0x7FF80);
    sh(p, reg(0, r::ESAH), 7);
    sh(p, reg(0, r::ESAL), 0xFF00);
    p.delay(200_000);
    p.rec_masked(s21::reg::I_STAT, 0x200, 0x200);
    rec_h(p, super::SPU2 + 0x7C2, Some(4));
    arm(p, 0, 0x7FF80);
    sh(p, reg(0, r::ATTR), 0x00C0);
    p.delay(200_000);
    p.rec_masked(s21::reg::I_STAT, 0x200, 0x200);
    rec_h(p, super::SPU2 + 0x7C2, Some(4));
    sh(p, reg(0, r::ESAL), 0);
    // Core 1's address met by core 0's voice; ATTR's enable written clear clears the flag.
    arm(p, 1, 0x5003);
    p.delay(20_000);
    p.rec_masked(s21::reg::I_STAT, 0x200, 0x200);
    rec_h(p, super::SPU2 + 0x7C2, Some(8));
    // STATX with the flag up: psx-spx's bit 6 (D-329); PCSX2 reads 0.
    rec_h(p, reg(1, r::STATX), Some(0x40));
    sh(p, reg(1, r::ATTR), 0);
    rec_h(p, super::SPU2 + 0x7C2, Some(0));
    g
}

/// `s26-registers`: the registers read back: a voice's eight, its addresses' widths, NAX as a voice plays, the
/// transfer address after the data port, ENDX's clearing by a key on, ps2tek's key on row (D-260), ESA and EEA.
pub fn registers() -> Gate {
    let mut g = Gate::new("s26-registers");
    g.upload(0x5000, &super::square());
    g.upload(0x5008, &super::block(0, 0, 1, |_| 0));
    g.silence();
    let p = &mut g.program;
    rec_h(p, reg(0, r::TSAH), Some(0));
    rec_h(p, reg(0, r::TSAL), Some(0x5010));
    for (i, v) in [0x1000u32, 0x2000, 0x1000, 0x000F, 0x1FC0].iter().enumerate() {
        sh(p, reg(0, r::voice(2, i as u32)), *v);
    }
    for i in 0..4 {
        sh(p, reg(0, r::addr(2, i)), 0xFFFF);
        rec_h(p, reg(0, r::addr(2, i)), if i % 2 == 0 { Some(0xF) } else { None });
    }
    sh(p, reg(0, r::addr(2, 0)), 0);
    sh(p, reg(0, r::addr(2, 1)), 0x5008);
    p.delay(2_000);
    for (i, want) in [(0u32, 0x1000u32), (1, 0x2000), (2, 0x1000), (3, 0x000F), (4, 0x1FC0), (6, 0x2000), (7, 0x4000)] {
        rec_h(p, reg(0, r::voice(2, i)), Some(want));
    }
    // ps2tek's key on row keys nothing on (D-260); 1A0h does.
    sh(p, reg(0, r::KON_PS2TEK), 4);
    p.delay(2_000);
    rec_h(p, reg(0, r::voice(2, 5)), Some(0));
    sh(p, reg(0, r::KON), 4);
    p.delay(20_000);
    rec_h(p, reg(0, r::voice(2, 5)), Some(0));
    super::lh(p, reg(0, r::ENDX));
    p.emit("andi t2, t2, 4");
    p.record("t2", Some(4));
    sh(p, reg(0, r::addr(2, 1)), 0x5000);
    sh(p, reg(0, r::KON), 4);
    p.delay(2_000);
    rec_h(p, reg(0, r::voice(2, 5)), Some(0x3FFF));
    rec_h(p, reg(0, r::addr(2, 4)), Some(0));
    super::lh(p, reg(0, r::addr(2, 5)));
    p.emit("andi t2, t2, 0xfff8");
    p.record("t2", Some(0x5000));
    rec_h(p, reg(0, r::addr(2, 2)), Some(0));
    rec_h(p, reg(0, r::addr(2, 3)), Some(0x5000));
    super::lh(p, reg(0, r::ENDX));
    p.emit("andi t2, t2, 4");
    p.record("t2", Some(0));
    for (o, v) in [(r::ESAH, 0x1u32), (r::ESAL, 0x2345), (r::EEA, 0x3)] {
        sh(p, reg(0, o), v);
        rec_h(p, reg(0, o), Some(v));
    }
    rec_h(p, reg(0, r::STATX), Some(0));
    g
}

pub fn all() -> Vec<Gate> {
    vec![voices(), envelope(), noise(), mixing(), reverb(), adma(), irq(), registers()]
}
