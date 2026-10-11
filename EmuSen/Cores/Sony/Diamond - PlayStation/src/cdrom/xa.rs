//! XA-ADPCM (psx-spx "CDROM XA Audio ADPCM Compression"): a form 2 sector's eighteen 128-byte portions decoded at
//! 37,800 or 18,900 Hz, 4- or 8-bit, mono or stereo, and the 25-point zigzag interpolation to 44,100 Hz. Diamond_CDROM.md
//! §6 records the choices where the document stops.

/// The four filters' coefficients over 64 (psx-spx "Pos/neg Tables").
const POS: [i32; 4] = [0, 60, 115, 98];
const NEG: [i32; 4] = [0, 0, -52, -55];

/// psx-spx's seven 29-tap tables, the zigzag interpolation from 37,800 to 44,100 Hz.
pub const ZIGZAG: [[i32; 29]; 7] = [
    [0, 0, 0, 0, 0, -0x0002, 0x000A, -0x0022, 0x0041, -0x0054, 0x0034, 0x0009, -0x010A, 0x0400, -0x0A78, 0x234C, 0x6794, -0x1780, 0x0BCD, -0x0623, 0x0350, -0x016D, 0x006B, 0x000A, -0x0010, 0x0011, -0x0008, 0x0003, -0x0001],
    [0, 0, 0, -0x0002, 0, 0x0003, -0x0013, 0x003C, -0x004B, 0x00A2, -0x00E3, 0x0132, -0x0043, -0x0267, 0x0C9D, 0x74BB, -0x11B4, 0x09B8, -0x05BF, 0x0372, -0x01A8, 0x00A6, -0x001B, 0x0005, 0x0006, -0x0008, 0x0003, -0x0001, 0],
    [0, 0, -0x0001, 0x0003, -0x0002, -0x0005, 0x001F, -0x004A, 0x00B3, -0x0192, 0x02B1, -0x039E, 0x04F8, -0x05A6, 0x7939, -0x05A6, 0x04F8, -0x039E, 0x02B1, -0x0192, 0x00B3, -0x004A, 0x001F, -0x0005, -0x0002, 0x0003, -0x0001, 0, 0],
    [0, -0x0001, 0x0003, -0x0008, 0x0006, 0x0005, -0x001B, 0x00A6, -0x01A8, 0x0372, -0x05BF, 0x09B8, -0x11B4, 0x74BB, 0x0C9D, -0x0267, -0x0043, 0x0132, -0x00E3, 0x00A2, -0x004B, 0x003C, -0x0013, 0x0003, 0, -0x0002, 0, 0, 0],
    [-0x0001, 0x0003, -0x0008, 0x0011, -0x0010, 0x000A, 0x006B, -0x016D, 0x0350, -0x0623, 0x0BCD, -0x1780, 0x6794, 0x234C, -0x0A78, 0x0400, -0x010A, 0x0009, 0x0034, -0x0054, 0x0041, -0x0022, 0x000A, -0x0001, 0, 0x0001, 0, 0, 0],
    [0x0002, -0x0008, 0x0010, -0x0023, 0x002B, 0x001A, -0x00EB, 0x027B, -0x0548, 0x0AFA, -0x16FA, 0x53E0, 0x3C07, -0x1249, 0x080E, -0x0347, 0x015B, -0x0044, -0x0017, 0x0046, -0x0023, 0x0011, -0x0005, 0, 0, 0, 0, 0, 0],
    [-0x0005, 0x0011, -0x0023, 0x0046, -0x0017, -0x0044, 0x015B, -0x0347, 0x080E, -0x1249, 0x3C07, 0x53E0, -0x16FA, 0x0AFA, -0x0548, 0x027B, -0x00EB, 0x001A, 0x002B, -0x0023, 0x0010, -0x0008, 0x0002, 0, 0, 0, 0, 0, 0],
];

/// What a sector's coding information byte says (psx-spx "4th Subheader byte").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Coding {
    pub stereo: bool,
    pub half_rate: bool,
    pub eight_bit: bool,
}

impl Coding {
    pub fn of(ci: u8) -> Coding {
        Coding { stereo: ci & 3 == 1, half_rate: ci >> 2 & 3 == 1, eight_bit: ci >> 4 & 3 == 1 }
    }

    /// Samples per channel a sector decodes to.
    pub fn samples(self) -> usize {
        18 * if self.eight_bit { 4 } else { 8 } * 28 / if self.stereo { 2 } else { 1 }
    }
}

/// The decoder's prediction history and the interpolator's ring for each channel.
#[derive(Clone, Debug)]
pub struct Xa {
    pub old: [[i32; 2]; 2],
    pub ring: [[i16; 32]; 2],
    pub p: u8,
    pub sixstep: u8,
}

impl Default for Xa {
    fn default() -> Xa {
        Xa { old: [[0; 2]; 2], ring: [[0; 32]; 2], p: 0, sixstep: 6 }
    }
}

/// One block's 28 samples (psx-spx "decode_28_nibbles"): the 4- or 8-bit value shifted to 16 bits, the filter's
/// prediction from the two before added, clamped.
fn block(portion: &[u8], blk: usize, nibble: usize, eight: bool, old: &mut [i32; 2], out: &mut impl FnMut(i16)) {
    let h = portion[4 + blk * 2 + nibble];
    let shift = match h & 0xF {
        s @ 0..=12 => s,
        _ => 9,
    } as u32;
    let filter = (h >> 4 & 3) as usize;
    for j in 0..28 {
        let word = &portion[16 + j * 4..20 + j * 4];
        let t = if eight { (word[blk * 2 + nibble] as i8 as i32) << 8 } else { ((word[blk] >> (4 * nibble) & 0xF) as i8 as i32) << 28 >> 16 };
        let s = (t >> shift) + ((old[0] * POS[filter] + old[1] * NEG[filter] + 32) >> 6);
        let s = s.clamp(-0x8000, 0x7FFF);
        old[1] = old[0];
        old[0] = s;
        out(s as i16);
    }
}

impl Xa {
    /// Decodes the 2,304 bytes after a form 2 sector's subheader (`data`, from byte 24) into each channel's samples at
    /// the sector's own rate: `emit(channel, sample)`, the right channel 1, mono on channel 0.
    pub fn decode(&mut self, data: &[u8], coding: Coding, mut emit: impl FnMut(usize, i16)) {
        let blocks = if coding.eight_bit { 4 } else { 8 };
        for portion in data.chunks_exact(128).take(18) {
            for b in 0..blocks {
                let (blk, nibble) = if coding.eight_bit { (b / 2, b % 2) } else { (b / 2, b % 2) };
                if coding.stereo {
                    let ch = b % 2;
                    let mut old = self.old[ch];
                    block(portion, blk, nibble, coding.eight_bit, &mut old, &mut |s| emit(ch, s));
                    self.old[ch] = old;
                } else {
                    let mut old = self.old[0];
                    block(portion, blk, nibble, coding.eight_bit, &mut old, &mut |s| emit(0, s));
                    self.old[0] = old;
                }
            }
        }
    }

    /// One 37,800 Hz frame into the interpolator; every sixth, the seven 44,100 Hz frames it makes.
    pub fn resample(&mut self, frame: [i16; 2], mut out: impl FnMut([i16; 2])) {
        let p = self.p as usize;
        self.ring[0][p & 31] = frame[0];
        self.ring[1][p & 31] = frame[1];
        self.p = (p as u8).wrapping_add(1);
        self.sixstep -= 1;
        if self.sixstep == 0 {
            self.sixstep = 6;
            let p = self.p as usize;
            for table in &ZIGZAG {
                let mut o = [0i16; 2];
                for (c, v) in o.iter_mut().enumerate() {
                    let mut sum: i32 = 0;
                    for (i, &k) in table.iter().enumerate() {
                        sum += self.ring[c][(p.wrapping_sub(i + 1)) & 31] as i32 * k >> 15;
                    }
                    *v = sum.clamp(-0x8000, 0x7FFF) as i16;
                }
                out(o);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A portion whose eight 4-bit blocks use filter 0 and shift `shift`: each sample is its nibble shifted, exactly.
    fn portion(nibbles: impl Fn(usize, usize) -> u8, shift: u8) -> [u8; 128] {
        let mut p = [0u8; 128];
        for b in 0..8 {
            p[4 + b] = shift;
        }
        for j in 0..28 {
            for blk in 0..4 {
                p[16 + j * 4 + blk] = nibbles(blk * 2, j) & 0xF | (nibbles(blk * 2 + 1, j) & 0xF) << 4;
            }
        }
        p
    }

    #[test]
    fn filter_zero_decodes_its_source_exactly_in_each_layout() {
        let src = |b: usize, j: usize| ((b * 3 + j) % 16) as u8;
        let data: Vec<u8> = (0..18).flat_map(|_| portion(src, 0)).collect();
        let expect = |b: usize, j: usize| ((src(b, j) as i8) << 4 >> 4) as i16 * 0x1000;
        let mut mono = Vec::new();
        Xa::default().decode(&data, Coding::of(0), |c, s| mono.push((c, s)));
        assert_eq!(mono.len(), 18 * 8 * 28);
        let first: Vec<(usize, i16)> = (0..8).flat_map(|b| (0..28).map(move |j| (0, expect(b, j)))).collect();
        assert_eq!(&mono[..224], &first[..], "blocks in order, 28 samples each");
        let mut stereo = Vec::new();
        Xa::default().decode(&data, Coding::of(1), |c, s| stereo.push((c, s)));
        assert_eq!((stereo.iter().filter(|s| s.0 == 0).count(), stereo.iter().filter(|s| s.0 == 1).count()), (2016, 2016));
        assert_eq!(stereo[28], (1, expect(1, 0)), "block 1 is the right channel");
        assert_eq!(Coding::of(1).samples(), 2016);
        assert_eq!(Coding::of(0x14).samples(), 2016, "8-bit mono at half rate");
    }

    #[test]
    fn the_filters_predict_from_the_two_before() {
        let mut p = portion(|_, j| if j == 0 { 4 } else { 0 }, 0);
        p[4] = 1 << 4 | 2;
        let mut x = Xa::default();
        let mut got = Vec::new();
        x.decode(&[p; 18].concat(), Coding::of(0), |_, s| got.push(s));
        let mut want = vec![0x1000i32];
        for _ in 1..28 {
            let prev = *want.last().unwrap();
            want.push((prev * 60 + 32) >> 6);
        }
        assert_eq!(&got[..28].iter().map(|&s| s as i32).collect::<Vec<_>>()[..], &want[..]);
    }

    /// A held level comes out at each table's gain, psx-spx's tables summing to about 0.906 of 8000h.
    #[test]
    fn a_held_level_interpolates_to_the_tables_gain() {
        let mut x = Xa::default();
        let mut out = Vec::new();
        for _ in 0..600 {
            x.resample([0x4000, -0x4000], |f| out.push(f));
        }
        assert_eq!(out.len(), 700, "seven out for every six in: 37,800 to 44,100 Hz");
        for (n, f) in out[600..].iter().enumerate() {
            let gain: i32 = ZIGZAG[n % 7].iter().sum();
            assert!((0x73E0..=0x7420).contains(&gain));
            let want = 0x4000 * gain >> 15;
            assert!((f[0] as i32 - want).abs() <= 29 && (f[1] as i32 + want).abs() <= 29, "{n}: {f:?} against {want}");
        }
    }
}
