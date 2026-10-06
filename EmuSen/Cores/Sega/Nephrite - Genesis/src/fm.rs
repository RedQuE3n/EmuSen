//! The YM2612's FM unit, a sample at a time: each operator's phase generator, envelope generator and output through
//! the log-sine and exponent tables, the four operators of a channel in the order the chip evaluates them, and each
//! channel's nine-bit output. Its rules and their sources are Nephrite_Native.md §19, and §21 for those the board's
//! pins settled (Nephrite_Disputes.md D-17, D-18); the register file is `ym2612.rs`'s. SSG-EG, the LFO and CSM are
//! stage 5's next step.

use emusen_native::{StateReader, StateWriter, Truncated};

/// The detune step for each key code and detune magnitude: the YM2608 manual's Table 2-6, its values in Hz divided
/// by 0.053 Hz, one step of the phase increment at the manual's 8 MHz clock.
const DETUNE: [[u8; 4]; 32] = [
    [0, 0, 1, 2], [0, 0, 1, 2], [0, 0, 1, 2], [0, 0, 1, 2], [0, 1, 2, 2], [0, 1, 2, 3], [0, 1, 2, 3], [0, 1, 2, 3],
    [0, 1, 2, 4], [0, 1, 3, 4], [0, 1, 3, 4], [0, 1, 3, 5], [0, 2, 4, 5], [0, 2, 4, 6], [0, 2, 4, 6], [0, 2, 5, 7],
    [0, 2, 5, 8], [0, 3, 6, 8], [0, 3, 6, 9], [0, 3, 7, 10], [0, 4, 8, 11], [0, 4, 8, 12], [0, 4, 9, 13], [0, 5, 10, 14],
    [0, 5, 11, 16], [0, 6, 12, 17], [0, 6, 13, 19], [0, 7, 14, 20], [0, 8, 16, 22], [0, 8, 16, 22], [0, 8, 16, 22], [0, 8, 16, 22],
];

/// For each rate, the envelope update's shift (an update every 2^shift envelope cycles) and its eight-cycle loop of
/// increments: Nemesis's Table 1 and Table 2 (SpritesMind topic 386), measured on the YM2612.
const SHIFT: [u8; 16] = [11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 0, 0, 0];
const STEPS: [[u8; 8]; 4] = [[0, 1, 0, 1, 0, 1, 0, 1], [0, 1, 0, 1, 1, 1, 0, 1], [0, 1, 1, 1, 0, 1, 1, 1], [0, 1, 1, 1, 1, 1, 1, 1]];
/// From rate 48 the increments are 1 plus these, doubling every four rates up to 60, where they are all 8. The
/// cycles the doubled steps fall on are the board's (Nephrite_Disputes.md D-18).
const EXTRA: [[u8; 8]; 4] = [[0; 8], [1, 0, 0, 0, 1, 0, 0, 0], [1, 0, 1, 0, 1, 0, 1, 0], [1, 1, 1, 0, 1, 1, 1, 0]];

fn increment(rate: u8, cycle: usize) -> u16 {
    match rate {
        0..=1 => 0,
        2..=5 => STEPS[0][cycle] as u16,
        6..=7 => STEPS[2][cycle] as u16,
        8..=47 => STEPS[(rate & 3) as usize][cycle] as u16,
        48..=59 => (1 + EXTRA[(rate & 3) as usize][cycle] as u16) << ((rate - 48) / 4),
        _ => 8,
    }
}

// The log-sine and exponent ROMs (`fm_tables.rs`, whose sources Nephrite_OperatorTables.md gives).
use crate::fm_tables::{EXP, LOGSIN};

pub struct Tables {
    pub logsin: [u16; 256],
    pub exp: [u16; 256],
}

impl Tables {
    pub fn new() -> Tables {
        Tables { logsin: LOGSIN, exp: EXP }
    }

    /// One operator's output: the phase with its modulation added, attenuated by `att` (ten bits, 4.6 fixed point),
    /// as a 14-bit signed value.
    pub fn output(&self, phase: u16, att: u16) -> i16 {
        let p = phase & 0x3FF;
        let i = if p & 0x100 != 0 { 0xFF - (p & 0xFF) } else { p & 0xFF } as usize;
        let level = self.logsin[i] as u32 + ((att as u32) << 2);
        let shift = level >> 8;
        // The ROM rises with its index and leaves its top bit implied: 2^-x for a fraction f is EXP[255 - f] + 1024.
        let mantissa = self.exp[255 - (level & 0xFF) as usize] as u32 + 1024;
        let mag = if shift >= 13 { 0 } else { (mantissa << 2) >> shift } as i16;
        if p & 0x200 != 0 { -mag } else { mag }
    }
}

impl Default for Tables {
    fn default() -> Self {
        Tables::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Eg {
    Attack,
    Decay,
    Sustain,
    #[default]
    Release,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Op {
    /// The 20-bit phase counter.
    pub phase: u32,
    /// The 10-bit attenuation, its phase, and the key's state the last sample saw.
    pub att: u16,
    pub eg: Eg,
    pub key: bool,
    /// The last output, 14 bits signed.
    pub out: i16,
}

impl Default for Op {
    fn default() -> Self {
        Op { phase: 0, att: 0x3FF, eg: Eg::Release, key: false, out: 0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Channel {
    /// Operators S1-S4, numbered as algorithm 0 chains them.
    pub ops: [Op; 4],
    /// Operator 1's output of the last sample and of the one before: its feedback, and what the others take of it.
    pub old1: i16,
    pub out1: i16,
    /// The keys `$28` last wrote, S1-S4.
    pub keys: [bool; 4],
}

/// Each operator's register offset within its channel's: S1 at +0, S2 at +8, S3 at +4, S4 at +12.
const SLOT: [usize; 4] = [0, 8, 4, 12];

/// The FM unit's state: six channels, the envelope generator's cycle counter and the samples before its next cycle
/// (a cycle every third), and the frequency registers' high-byte latches (one each for `$A4`-`$A6` and `$AC`-`$AE`,
/// in each part).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Fm {
    pub ch: [Channel; 6],
    pub eg_counter: u32,
    pub eg_div: u8,
    pub latch: [[u8; 2]; 2],
}

/// What a register file read gives the FM unit: both parts' registers.
pub type Regs = [[u8; 256]; 2];

impl Fm {
    /// `$28`: the keys of the operators of one channel; channel codes 3 and 7 name none.
    pub fn key(&mut self, v: u8) {
        let c = (v & 7) as usize;
        if c & 3 == 3 {
            return;
        }
        let ch = (c >> 2) * 3 + (c & 3);
        for k in 0..4 {
            self.ch[ch].keys[k] = v >> (4 + k) & 1 != 0;
        }
    }

    /// A frequency write: the high bytes wait in their part's latch until the low byte's write carries them in.
    pub fn frequency(&mut self, regs: &mut Regs, part: usize, a: usize, v: u8) {
        match a {
            0xA4..=0xA6 => self.latch[part][0] = v,
            0xAC..=0xAE => self.latch[part][1] = v,
            0xA0..=0xA2 => {
                regs[part][a] = v;
                regs[part][a + 4] = self.latch[part][0];
            }
            0xA8..=0xAA => {
                regs[part][a] = v;
                regs[part][a + 4] = self.latch[part][1];
            }
            _ => {}
        }
    }

    /// The frequency number and block an operator plays: its channel's, or in channel 3's special modes, S1-S3's own.
    fn fnum_block(regs: &Regs, ch: usize, op: usize) -> (u16, u8) {
        let (part, i) = (ch / 3, ch % 3);
        let special = ch == 2 && regs[0][0x27] & 0xC0 != 0 && op < 3;
        let (lo, hi) = if special {
            let a = [0xA9, 0xAA, 0xA8][op];
            (regs[0][a], regs[0][a + 4])
        } else {
            (regs[part][0xA0 + i], regs[part][0xA4 + i])
        };
        ((hi as u16 & 7) << 8 | lo as u16, hi >> 3 & 7)
    }

    fn key_code(fnum: u16, block: u8) -> u8 {
        let f = |b: u16| fnum >> (b - 1) & 1;
        let n4 = f(11);
        let n3 = (f(11) & (f(10) | f(9) | f(8))) | ((1 - f(11)) & f(10) & f(9) & f(8));
        block << 2 | (n4 << 1 | n3) as u8
    }

    /// One sample: the keys, the operators at the attenuation they have, each channel's nine-bit output (the top
    /// nine of its carriers' fourteen bits summed and saturated), the phases' advance, and last, every third sample,
    /// the envelope's cycle, whose step shows from the next sample (Nephrite_Disputes.md D-17).
    pub fn sample(&mut self, regs: &Regs, t: &Tables) -> [i32; 6] {
        let update = self.eg_div == 0;
        self.eg_div = if update { 2 } else { self.eg_div - 1 };
        if update {
            self.eg_counter = self.eg_counter.wrapping_add(1);
        }
        let counter = self.eg_counter;
        let mut outs = [0i32; 6];
        for (c, channel_out) in outs.iter_mut().enumerate() {
            let (part, i) = (c / 3, c % 3);
            let r = &regs[part];
            let mut kcs = [0u8; 4];
            let mut freq = [(0u16, 0u8); 4];
            for o in 0..4 {
                let (fnum, block) = Self::fnum_block(regs, c, o);
                (kcs[o], freq[o]) = (Self::key_code(fnum, block), (fnum, block));
                let base = SLOT[o] + i;
                let key = self.ch[c].keys[o];
                let op = &mut self.ch[c].ops[o];
                if key && !op.key {
                    op.eg = Eg::Attack;
                    op.phase = 0;
                    if Self::rate(r[0x50 + base] & 0x1F, r[0x50 + base] >> 6, kcs[o]) >= 62 {
                        op.att = 0;
                    }
                } else if !key && op.key {
                    op.eg = Eg::Release;
                }
                op.key = key;
            }
            let alg = r[0xB0 + i] & 7;
            let fb = r[0xB0 + i] >> 3 & 7;
            let ch = &mut self.ch[c];
            let att = |o: usize, ops: &[Op; 4]| -> u16 { (ops[o].att as u32 + ((r[0x40 + SLOT[o] + i] as u32 & 0x7F) << 3)).min(0x3FF) as u16 };
            let phase10 = |op: &Op| (op.phase >> 10) as u16;
            // The modulation of a phase: a sum of up to two 14-bit outputs shifted down by one (Nephrite_Native.md §19).
            let modu = |a: i32, b: i32| -> u16 { ((a + b) >> 1) as u16 };
            // Operator 1's output of the last sample and of the one before, and operator 2's of the last: what this
            // sample's operators and sum take of them (Nephrite_Disputes.md D-17).
            let (before1, last1, last2) = (ch.old1 as i32, ch.out1 as i32, ch.ops[1].out as i32);
            // Operator 1, from its own last two outputs.
            let fbm = if fb == 0 { 0 } else { ((before1 + last1) >> (10 - fb)) as u16 };
            let o1 = t.output(phase10(&ch.ops[0]).wrapping_add(fbm), att(0, &ch.ops));
            ch.ops[0].out = o1;
            (ch.old1, ch.out1) = (ch.out1, o1);
            // Operator 3, from operator 1's output of two samples ago and operator 2's of the last.
            let m3 = match alg {
                0 | 2 => modu(last2, 0),
                1 => modu(before1, last2),
                5 => modu(before1, 0),
                _ => 0,
            };
            let o3 = t.output(phase10(&ch.ops[2]).wrapping_add(m3), att(2, &ch.ops));
            ch.ops[2].out = o3;
            // Operator 2, from operator 1's output of the last sample.
            let m2 = match alg {
                0 | 3 | 4 | 5 | 6 => modu(last1, 0),
                _ => 0,
            };
            let o2 = t.output(phase10(&ch.ops[1]).wrapping_add(m2), att(1, &ch.ops));
            ch.ops[1].out = o2;
            // Operator 4, from operator 1's and operator 2's outputs of the last sample and operator 3's of this one.
            let m4 = match alg {
                0 | 1 | 4 => modu(o3 as i32, 0),
                2 => modu(last1, o3 as i32),
                3 => modu(last2, o3 as i32),
                5 => modu(last1, 0),
                _ => 0,
            };
            let o4 = t.output(phase10(&ch.ops[3]).wrapping_add(m4), att(3, &ch.ops));
            ch.ops[3].out = o4;
            let carriers: &[i16] = match alg {
                0..=3 => &[o4],
                4 => &[o2, o4],
                5 | 6 => &[o2, o3, o4],
                _ => &[last1 as i16, o2, o3, o4],
            };
            *channel_out = carriers.iter().map(|&o| (o >> 5) as i32).sum::<i32>().clamp(-256, 255);
            for o in 0..4 {
                let (fnum, block) = freq[o];
                let base = SLOT[o] + i;
                let inc = Self::increment(fnum, block, r[0x30 + base], kcs[o]);
                let op = &mut self.ch[c].ops[o];
                op.phase = (op.phase + inc) & 0xF_FFFF;
                if update {
                    Self::envelope(op, r, base, kcs[o], counter);
                }
            }
        }
        outs
    }

    /// The phase increment: the frequency number shifted by the block into 17 bits, the detune added there, then the
    /// multiple applied (a half for 0) into 20 bits (Nemesis's account of the phase generator, topic 386).
    pub fn increment(fnum: u16, block: u8, dt_mul: u8, kc: u8) -> u32 {
        let shifted = ((fnum as u32) << block) >> 1;
        let dt = dt_mul >> 4 & 7;
        let step = DETUNE[kc as usize][(dt & 3) as usize] as u32;
        let detuned = if dt & 4 != 0 { shifted.wrapping_sub(step) } else { shifted + step } & 0x1_FFFF;
        let mul = (dt_mul & 15) as u32;
        (if mul == 0 { detuned >> 1 } else { detuned * mul }) & 0xF_FFFF
    }

    /// The envelope's rate: 0 for a register value of 0, else twice it plus the key-scaled key code, at most 63.
    fn rate(r: u8, ks: u8, kc: u8) -> u8 {
        if r == 0 { 0 } else { (2 * r + (kc >> (3 - ks))).min(63) }
    }

    /// One envelope cycle for an operator: its phase's change, then its rate's increment on the cycles its shift
    /// selects (Nemesis, topic 386, with his 2010 corrections).
    fn envelope(op: &mut Op, r: &[u8; 256], base: usize, kc: u8, counter: u32) {
        let sl = match r[0x80 + base] >> 4 {
            15 => 0x3FF,
            s => (s as u16) << 5,
        };
        if op.eg == Eg::Attack && op.att == 0 {
            op.eg = Eg::Decay;
        }
        if op.eg == Eg::Decay && op.att >= sl {
            op.eg = Eg::Sustain;
        }
        let ks = r[0x50 + base] >> 6;
        let reg = match op.eg {
            Eg::Attack => r[0x50 + base] & 0x1F,
            Eg::Decay => r[0x60 + base] & 0x1F,
            Eg::Sustain => r[0x70 + base] & 0x1F,
            Eg::Release => (r[0x80 + base] & 15) << 1 | 1,
        };
        let rate = Self::rate(reg, ks, kc);
        let shift = SHIFT[(rate >> 2) as usize] as u32;
        if counter & ((1 << shift) - 1) != 0 {
            return;
        }
        let cycle = (counter >> shift & 7) as usize;
        let inc = increment(rate, cycle);
        if op.eg == Eg::Attack {
            if rate < 62 && inc != 0 {
                let a = op.att as i32;
                op.att = (a + ((!a * inc as i32) >> 4)).clamp(0, 0x3FF) as u16;
            }
        } else {
            op.att = (op.att + inc).min(0x3FF);
        }
    }

    pub fn write_state(&self, w: &mut StateWriter) {
        let mut ops = Vec::new();
        let mut keys = Vec::new();
        for c in &self.ch {
            for o in &c.ops {
                ops.extend_from_slice(&[o.phase as u64, o.att as u64, o.eg as u64, o.key as u64, o.out as u16 as u64]);
            }
            ops.extend_from_slice(&[c.old1 as u16 as u64, c.out1 as u16 as u64]);
            keys.extend(c.keys);
        }
        w.u64s("FmOperators", &ops);
        w.bools("FmKeys", &keys);
        w.u64s("FmEnvelope", &[self.eg_counter as u64, self.eg_div as u64]);
        w.bytes("FmLatches", &[self.latch[0][0], self.latch[0][1], self.latch[1][0], self.latch[1][1]]);
    }

    pub fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        let mut ops = [0u64; 6 * 22];
        r.u64s(&mut ops)?;
        let mut keys = [false; 24];
        r.bools(&mut keys)?;
        let mut e = [0u64; 2];
        r.u64s(&mut e)?;
        let mut l = [0u8; 4];
        r.bytes(&mut l)?;
        for (c, ch) in self.ch.iter_mut().enumerate() {
            let s = &ops[c * 22..];
            for (o, op) in ch.ops.iter_mut().enumerate() {
                let v = &s[o * 5..];
                *op = Op { phase: v[0] as u32, att: v[1] as u16, eg: [Eg::Attack, Eg::Decay, Eg::Sustain, Eg::Release][v[2] as usize & 3], key: v[3] != 0, out: v[4] as u16 as i16 };
            }
            (ch.old1, ch.out1) = (s[20] as u16 as i16, s[21] as u16 as i16);
            ch.keys.copy_from_slice(&keys[c * 4..c * 4 + 4]);
        }
        (self.eg_counter, self.eg_div) = (e[0] as u32, e[1] as u8);
        self.latch = [[l[0], l[1]], [l[2], l[3]]];
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Nemesis's worked examples of the phase increment (topic 386): block 5 shifts 0x100 to 0x1000, detune 3 at key
    /// code 0x14 adds 11 and detune 7 takes 11 away in 17 bits; multiple 8 of 0x1FF at block 4 is 0x7FC0, multiple 0
    /// halves 0xFF8; detune below zero wraps in 17 bits.
    #[test]
    fn the_phase_increment_follows_the_worked_examples() {
        assert_eq!(Fm::key_code(0x100, 5), 0x14);
        assert_eq!(Fm::increment(0x100, 5, 0x01, 0x14), 0x1000);
        assert_eq!(Fm::increment(0x100, 5, 0x31, 0x14), 0x1000 + 11);
        assert_eq!(Fm::increment(0x100, 5, 0x71, 0x14), 0x1000 - 11);
        assert_eq!(Fm::increment(0x1FF, 4, 0x08, Fm::key_code(0x1FF, 4)), 0x7FC0);
        assert_eq!(Fm::increment(0x1FF, 4, 0x00, Fm::key_code(0x1FF, 4)), 0x7FC);
        assert_eq!(Fm::increment(0, 0, 0x71, 0), 0x1_FFFE, "0 less 2 in 17 bits");
        assert_eq!(Fm::increment(1, 0, 0x01, 0), 0, "block 0 loses the frequency's low bit");
        assert_eq!(Fm::increment(3, 0, 0x01, 0), 1);
    }

    #[test]
    fn the_rate_doubles_the_register_adds_the_key_scale_and_stops_at_63() {
        assert_eq!(Fm::rate(0, 3, 31), 0, "a register of 0 is no rate whatever the key code");
        assert_eq!(Fm::rate(12, 0, 18), 26);
        assert_eq!(Fm::rate(12, 3, 18), 42);
        assert_eq!(Fm::rate(31, 3, 31), 63);
        assert_eq!(increment(48, 3), 1);
        assert_eq!((0..4).map(|c| increment(49, c)).collect::<Vec<_>>(), [2, 1, 1, 1], "the board's cycles (D-18)");
        assert_eq!((0..4).map(|c| increment(50, c)).collect::<Vec<_>>(), [2, 1, 2, 1]);
        assert_eq!((0..8).map(|c| increment(51, c)).collect::<Vec<_>>(), [2, 2, 2, 1, 2, 2, 2, 1]);
        assert_eq!(increment(55, 1), 4);
        assert_eq!((0..8).map(|c| increment(47, c)).collect::<Vec<_>>(), [0, 1, 1, 1, 1, 1, 1, 1], "the cycle rate 47 misses numbers them");
        assert_eq!(increment(60, 0), 8);
        assert_eq!(increment(7, 0), 0);
    }

    /// The operator's sine: positive in the first half of the phase, mirrored in each quarter, and zero once the
    /// attenuation passes 13 octaves of the exponent's shift.
    #[test]
    fn the_operator_is_a_sine_by_quarters() {
        let t = Tables::new();
        assert_eq!(t.output(0x100, 0), t.output(0x0FF, 0));
        assert_eq!(t.output(0x200, 0), -t.output(0, 0));
        assert_eq!(t.output(0x100, 0), 8168);
        assert_eq!(t.output(0x300, 0), -8168);
        assert_eq!(t.output(0x100, 0x3FF), 0);
        assert_eq!(t.output(0x100, 0x40), 4084, "an attenuation of 0x40, one octave in 4.6 fixed point, halves it");
    }
}
