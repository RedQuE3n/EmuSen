//! The YM2612's FM unit, a sample at a time: each operator's phase generator, envelope generator and output through
//! the log-sine and exponent tables, the four operators of a channel in the order the chip evaluates them, and each
//! channel's nine-bit output. Its rules and their sources are Nephrite_Native.md §19; the register file is
//! `ym2612.rs`'s. SSG-EG, the LFO and CSM are stage 5's next step.

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
/// From rate 48 the increments are 1 plus these, doubling every four rates up to 60, where they are all 8.
const EXTRA: [[u8; 8]; 4] = [[0; 8], [0, 0, 0, 1, 0, 0, 0, 1], [0, 1, 0, 1, 0, 1, 0, 1], [0, 1, 1, 1, 0, 1, 1, 1]];

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

/// The log-sine and exponent tables, written out from their description (Nephrite_Native.md §19.2) so that every
/// machine has the same bits: a quarter-wave of 256 entries giving −log2 of the sine at each entry's middle in 4.8
/// fixed point, and 256 entries of 2^−(i+1)/256 as eleven bits whose top one is always set. A test builds them again.
pub const LOGSIN: [u16; 256] = [
    2137, 1731, 1543, 1419, 1326, 1252, 1190, 1137, 1091, 1050, 1013, 979, 949, 920, 894, 869,
    846, 825, 804, 785, 767, 749, 732, 717, 701, 687, 672, 659, 646, 633, 621, 609,
    598, 587, 576, 566, 556, 546, 536, 527, 518, 509, 501, 492, 484, 476, 468, 461,
    453, 446, 439, 432, 425, 418, 411, 405, 399, 392, 386, 380, 375, 369, 363, 358,
    352, 347, 341, 336, 331, 326, 321, 316, 311, 307, 302, 297, 293, 289, 284, 280,
    276, 271, 267, 263, 259, 255, 251, 248, 244, 240, 236, 233, 229, 226, 222, 219,
    215, 212, 209, 205, 202, 199, 196, 193, 190, 187, 184, 181, 178, 175, 172, 169,
    167, 164, 161, 159, 156, 153, 151, 148, 146, 143, 141, 138, 136, 134, 131, 129,
    127, 125, 122, 120, 118, 116, 114, 112, 110, 108, 106, 104, 102, 100, 98, 96,
    94, 92, 91, 89, 87, 85, 83, 82, 80, 78, 77, 75, 74, 72, 70, 69,
    67, 66, 64, 63, 62, 60, 59, 57, 56, 55, 53, 52, 51, 49, 48, 47,
    46, 45, 43, 42, 41, 40, 39, 38, 37, 36, 35, 34, 33, 32, 31, 30,
    29, 28, 27, 26, 25, 24, 23, 23, 22, 21, 20, 20, 19, 18, 17, 17,
    16, 15, 15, 14, 13, 13, 12, 12, 11, 10, 10, 9, 9, 8, 8, 7,
    7, 7, 6, 6, 5, 5, 5, 4, 4, 4, 3, 3, 3, 2, 2, 2,
    2, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0,
];
pub const EXP: [u16; 256] = [
    2042, 2037, 2031, 2026, 2020, 2015, 2010, 2004, 1999, 1993, 1988, 1983, 1977, 1972, 1966, 1961,
    1956, 1951, 1945, 1940, 1935, 1930, 1924, 1919, 1914, 1909, 1904, 1898, 1893, 1888, 1883, 1878,
    1873, 1868, 1863, 1858, 1853, 1848, 1843, 1838, 1833, 1828, 1823, 1818, 1813, 1808, 1803, 1798,
    1794, 1789, 1784, 1779, 1774, 1769, 1765, 1760, 1755, 1750, 1746, 1741, 1736, 1732, 1727, 1722,
    1717, 1713, 1708, 1704, 1699, 1694, 1690, 1685, 1681, 1676, 1672, 1667, 1663, 1658, 1654, 1649,
    1645, 1640, 1636, 1631, 1627, 1623, 1618, 1614, 1609, 1605, 1601, 1596, 1592, 1588, 1584, 1579,
    1575, 1571, 1566, 1562, 1558, 1554, 1550, 1545, 1541, 1537, 1533, 1529, 1525, 1520, 1516, 1512,
    1508, 1504, 1500, 1496, 1492, 1488, 1484, 1480, 1476, 1472, 1468, 1464, 1460, 1456, 1452, 1448,
    1444, 1440, 1436, 1433, 1429, 1425, 1421, 1417, 1413, 1409, 1406, 1402, 1398, 1394, 1391, 1387,
    1383, 1379, 1376, 1372, 1368, 1364, 1361, 1357, 1353, 1350, 1346, 1342, 1339, 1335, 1332, 1328,
    1324, 1321, 1317, 1314, 1310, 1307, 1303, 1300, 1296, 1292, 1289, 1286, 1282, 1279, 1275, 1272,
    1268, 1265, 1261, 1258, 1255, 1251, 1248, 1244, 1241, 1238, 1234, 1231, 1228, 1224, 1221, 1218,
    1214, 1211, 1208, 1205, 1201, 1198, 1195, 1192, 1188, 1185, 1182, 1179, 1176, 1172, 1169, 1166,
    1163, 1160, 1157, 1154, 1150, 1147, 1144, 1141, 1138, 1135, 1132, 1129, 1126, 1123, 1120, 1117,
    1114, 1111, 1108, 1105, 1102, 1099, 1096, 1093, 1090, 1087, 1084, 1081, 1078, 1075, 1072, 1069,
    1066, 1064, 1061, 1058, 1055, 1052, 1049, 1046, 1044, 1041, 1038, 1035, 1032, 1030, 1027, 1024,
];

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
        let mag = if shift >= 13 { 0 } else { ((self.exp[(level & 0xFF) as usize] as u32) << 2) >> shift } as i16;
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
    /// The stored outputs the chip keeps between operators: operator 1's last and the one before, operator 2's.
    pub old1: i16,
    pub out1: i16,
    pub out2: i16,
    /// The keys `$28` last wrote, S1-S4.
    pub keys: [bool; 4],
}

/// Each operator's register offset within its channel's: S1 at +0, S2 at +8, S3 at +4, S4 at +12.
const SLOT: [usize; 4] = [0, 8, 4, 12];

/// The FM unit's state: six channels, the envelope generator's cycle counter and the samples to its next cycle,
/// and the frequency registers' high-byte latches (one each for `$A4`-`$A6` and `$AC`-`$AE`, in each part).
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

    /// One sample: keys, the envelope's update every third sample, the phases and the operators, and each channel's
    /// nine-bit output, the top nine of its carriers' fourteen bits summed and saturated.
    pub fn sample(&mut self, regs: &Regs, t: &Tables) -> [i32; 6] {
        let update = self.eg_div == 0;
        self.eg_div = (self.eg_div + 1) % 3;
        if update {
            self.eg_counter = self.eg_counter.wrapping_add(1);
        }
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
                if update {
                    Self::envelope(op, r, base, kcs[o], self.eg_counter);
                }
            }
            let alg = r[0xB0 + i] & 7;
            let fb = r[0xB0 + i] >> 3 & 7;
            let ch = &mut self.ch[c];
            let att = |o: usize, ops: &[Op; 4]| -> u16 { (ops[o].att as u32 + ((r[0x40 + SLOT[o] + i] as u32 & 0x7F) << 3)).min(0x3FF) as u16 };
            let phase10 = |op: &Op| (op.phase >> 10) as u16;
            // The modulation of a phase: a sum of up to two 14-bit outputs shifted down by one (Nephrite_Native.md §19).
            let modu = |a: i32, b: i32| -> u16 { ((a + b) >> 1) as u16 };
            // Operator 1, from its own last two outputs.
            ch.out2 = ch.ops[1].out;
            let fbm = if fb == 0 { 0 } else { ((ch.old1 as i32 + ch.out1 as i32) >> (10 - fb)) as u16 };
            let o1 = t.output(phase10(&ch.ops[0]).wrapping_add(fbm), att(0, &ch.ops));
            ch.ops[0].out = o1;
            // Operator 3, from the stored operator 1 and operator 2.
            let m3 = match alg {
                0 => modu(ch.out2 as i32, 0),
                1 => modu(ch.out1 as i32, ch.out2 as i32),
                2 => modu(ch.out2 as i32, 0),
                3 => 0,
                4 => 0,
                5 => modu(ch.out1 as i32, 0),
                _ => 0,
            };
            let o3 = t.output(phase10(&ch.ops[2]).wrapping_add(m3), att(2, &ch.ops));
            ch.ops[2].out = o3;
            // Operator 2, from operator 1's newest output.
            let m2 = match alg {
                0 | 3 | 4 | 5 | 6 => modu(o1 as i32, 0),
                _ => 0,
            };
            let o2 = t.output(phase10(&ch.ops[1]).wrapping_add(m2), att(1, &ch.ops));
            ch.ops[1].out = o2;
            ch.old1 = ch.out1;
            ch.out1 = o1;
            // Operator 4, from the stored 1 and 2 and operator 3's newest output.
            let m4 = match alg {
                0 | 1 => modu(o3 as i32, 0),
                2 => modu(ch.out1 as i32, o3 as i32),
                3 => modu(ch.out2 as i32, o3 as i32),
                4 => modu(o3 as i32, 0),
                5 => modu(ch.out1 as i32, 0),
                _ => 0,
            };
            let o4 = t.output(phase10(&ch.ops[3]).wrapping_add(m4), att(3, &ch.ops));
            ch.ops[3].out = o4;
            let carriers: &[i16] = match alg {
                0..=3 => &[o4],
                4 => &[o2, o4],
                5 | 6 => &[o2, o3, o4],
                _ => &[o1, o2, o3, o4],
            };
            *channel_out = carriers.iter().map(|&o| (o >> 5) as i32).sum::<i32>().clamp(-256, 255);
            for o in 0..4 {
                let (fnum, block) = freq[o];
                let base = SLOT[o] + i;
                let inc = Self::increment(fnum, block, r[0x30 + base], kcs[o]);
                let op = &mut self.ch[c].ops[o];
                op.phase = (op.phase + inc) & 0xF_FFFF;
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
            ops.extend_from_slice(&[c.old1 as u16 as u64, c.out1 as u16 as u64, c.out2 as u16 as u64]);
            keys.extend(c.keys);
        }
        w.u64s("FmOperators", &ops);
        w.bools("FmKeys", &keys);
        w.u64s("FmEnvelope", &[self.eg_counter as u64, self.eg_div as u64]);
        w.bytes("FmLatches", &[self.latch[0][0], self.latch[0][1], self.latch[1][0], self.latch[1][1]]);
    }

    pub fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        let mut ops = [0u64; 6 * 23];
        r.u64s(&mut ops)?;
        let mut keys = [false; 24];
        r.bools(&mut keys)?;
        let mut e = [0u64; 2];
        r.u64s(&mut e)?;
        let mut l = [0u8; 4];
        r.bytes(&mut l)?;
        for (c, ch) in self.ch.iter_mut().enumerate() {
            let s = &ops[c * 23..];
            for (o, op) in ch.ops.iter_mut().enumerate() {
                let v = &s[o * 5..];
                *op = Op { phase: v[0] as u32, att: v[1] as u16, eg: [Eg::Attack, Eg::Decay, Eg::Sustain, Eg::Release][v[2] as usize & 3], key: v[3] != 0, out: v[4] as u16 as i16 };
            }
            (ch.old1, ch.out1, ch.out2) = (s[20] as u16 as i16, s[21] as u16 as i16, s[22] as u16 as i16);
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

    #[test]
    fn the_tables_are_their_description() {
        for i in 0..256 {
            let sine = (((i as f64) + 0.5) * std::f64::consts::PI / 512.0).sin();
            assert_eq!(LOGSIN[i], (-sine.log2() * 256.0).round() as u16, "log-sine {i}");
            assert_eq!(EXP[i], ((2f64).powf(-((i + 1) as f64) / 256.0) * 2048.0).round() as u16, "exponent {i}");
        }
    }

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
        assert_eq!(increment(51, 3), 2);
        assert_eq!(increment(55, 1), 4);
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
