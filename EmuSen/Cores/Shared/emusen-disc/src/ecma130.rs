//! A raw 2,352-byte sector as ECMA-130 (2nd edition, 1996) defines it: the sync, the header (§14.1–14.2), the EDC
//! (§14.3), and the P- and Q-parity of the RSPC (Annex A); mode 1 and the CD-XA modes 2 form 1 and form 2, whose
//! subheader and EDC placement psx-spx's "CDROM Sector Encoding" gives. Written from the standard's definitions.

use crate::msf::Msf;

pub const RAW: usize = 2352;
pub const SYNC: [u8; 12] = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00];
pub const USER: usize = 2048;
pub const FORM2_USER: usize = 2324;

/// The subheader's submode bits (psx-spx "CDROM XA Subheader"): end of record, data, form 2, end of file.
pub mod submode {
    pub const EOR: u8 = 1 << 0;
    pub const VIDEO: u8 = 1 << 1;
    pub const AUDIO: u8 = 1 << 2;
    pub const DATA: u8 = 1 << 3;
    pub const FORM2: u8 = 1 << 5;
    pub const REAL_TIME: u8 = 1 << 6;
    pub const EOF: u8 = 1 << 7;
}

/// The EDC's check polynomial (x¹⁶ + x¹⁵ + x² + 1)(x¹⁶ + x² + x + 1), taken least significant bit first: reflected.
pub const EDC_POLY: u32 = 0xD801_8001;

const fn edc_table() -> [u32; 256] {
    let mut t = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut c = i as u32;
        let mut k = 0;
        while k < 8 {
            c = if c & 1 != 0 { c >> 1 ^ EDC_POLY } else { c >> 1 };
            k += 1;
        }
        t[i] = c;
        i += 1;
    }
    t
}

static EDC_TABLE: [u32; 256] = edc_table();

/// The 32-bit EDC of §14.3 over `data`.
pub fn edc(data: &[u8]) -> u32 {
    data.iter().fold(0, |c, &b| EDC_TABLE[((c ^ b as u32) & 0xFF) as usize] ^ c >> 8)
}

/// GF(2⁸) by x⁸ + x⁴ + x³ + x² + 1 (Annex A.3): α's powers and their logarithms.
const fn gf_tables() -> ([u8; 512], [u8; 256]) {
    let mut exp = [0u8; 512];
    let mut log = [0u8; 256];
    let mut x: u32 = 1;
    let mut i = 0;
    while i < 255 {
        exp[i] = x as u8;
        exp[i + 255] = x as u8;
        log[x as usize] = i as u8;
        x <<= 1;
        if x & 0x100 != 0 {
            x ^= 0x11D;
        }
        i += 1;
    }
    (exp, log)
}

static GF: ([u8; 512], [u8; 256]) = gf_tables();

pub fn gf_mul(a: u8, b: u8) -> u8 {
    if a == 0 || b == 0 { 0 } else { GF.0[GF.1[a as usize] as usize + GF.1[b as usize] as usize] }
}

/// αⁿ.
pub fn alpha(n: usize) -> u8 {
    GF.0[n % 255]
}

fn gf_div(a: u8, b: u8) -> u8 {
    if a == 0 { 0 } else { GF.0[(GF.1[a as usize] as usize + 255 - GF.1[b as usize] as usize) % 255] }
}

/// The word count of the RSPC (A.2): 1,170 two-byte words from byte 12.
const WORDS: usize = 1170;

/// Byte `half` (0 the LSB, 1 the MSB) of word `n` (A.2: S(n) = MSB B(2n+13), LSB B(2n+12)).
#[inline]
fn at(n: usize, half: usize) -> usize {
    12 + 2 * n + half
}

/// A P-vector's word indices (A.3): column `np` of the 43 × 26 matrix.
fn p_word(np: usize, mp: usize) -> usize {
    43 * mp + np
}

/// A Q-vector's word indices: diagonal `nq`, its 43 bytes then its two parity words.
fn q_word(nq: usize, mq: usize) -> usize {
    match mq {
        43 => 1118 + nq,
        44 => 1144 + nq,
        _ => (44 * mq + 43 * nq) % 1118,
    }
}

/// The two parity bytes that satisfy H = [1 … 1; αⁿ⁻¹ … α 1] for a vector of `n` bytes whose last two are parity:
/// p₀ = (S₀ + S₁)/(α + 1), p₁ = S₀ + p₀.
fn solve(s0: u8, s1: u8) -> (u8, u8) {
    let p0 = gf_div(s0 ^ s1, alpha(1) ^ 1);
    (p0, s0 ^ p0)
}

/// Writes the P- then the Q-parity of `s`, computed over bytes 12 onward as they stand.
pub fn parity(s: &mut [u8; RAW]) {
    for half in 0..2 {
        for np in 0..43 {
            let (mut s0, mut s1) = (0u8, 0u8);
            for mp in 0..24 {
                let v = s[at(p_word(np, mp), half)];
                s0 ^= v;
                s1 ^= gf_mul(v, alpha(25 - mp));
            }
            let (p0, p1) = solve(s0, s1);
            s[at(p_word(np, 24), half)] = p0;
            s[at(p_word(np, 25), half)] = p1;
        }
        for nq in 0..26 {
            let (mut s0, mut s1) = (0u8, 0u8);
            for mq in 0..43 {
                let v = s[at(q_word(nq, mq), half)];
                s0 ^= v;
                s1 ^= gf_mul(v, alpha(44 - mq));
            }
            let (p0, p1) = solve(s0, s1);
            s[at(q_word(nq, 43), half)] = p0;
            s[at(q_word(nq, 44), half)] = p1;
        }
    }
}

/// Whether every P- and Q-vector of `s` meets both rows of its check matrix (A.3), the encoder's equations checked
/// apart from their solution.
pub fn parity_holds(s: &[u8; RAW]) -> bool {
    debug_assert!(WORDS == 1118 + 52);
    let holds = |n: usize, word: &dyn Fn(usize) -> usize, half: usize| {
        let (mut s0, mut s1) = (0u8, 0u8);
        for i in 0..n {
            let v = s[at(word(i), half)];
            s0 ^= v;
            s1 ^= gf_mul(v, alpha(n - 1 - i));
        }
        s0 == 0 && s1 == 0
    };
    (0..2).all(|h| (0..43).all(|np| holds(26, &|mp| p_word(np, mp), h)) && (0..26).all(|nq| holds(45, &|mq| q_word(nq, mq), h)))
}

/// The sync and the header: the absolute time of sector `at` in BCD and the mode.
pub fn header(at: u32, mode: u8, s: &mut [u8; RAW]) {
    s[..12].copy_from_slice(&SYNC);
    s[12..15].copy_from_slice(&Msf::of(at).bcd());
    s[15] = mode;
}

/// A mode 1 sector of `user`'s 2,048 bytes, with its EDC over bytes 0–2,063, eight zeros and its parity.
pub fn mode1(at: u32, user: &[u8], s: &mut [u8; RAW]) {
    header(at, 1, s);
    s[16..2064].copy_from_slice(&user[..USER]);
    let e = edc(&s[..2064]);
    s[2064..2068].copy_from_slice(&e.to_le_bytes());
    s[2068..2076].fill(0);
    parity(s);
}

/// A mode 2 form 1 sector: the subheader twice, 2,048 bytes, the EDC over the subheader and data, and the parity
/// computed with the header taken as zero.
pub fn mode2_form1(at: u32, sub: [u8; 4], user: &[u8], s: &mut [u8; RAW]) {
    header(at, 2, s);
    s[16..20].copy_from_slice(&sub);
    s[20..24].copy_from_slice(&sub);
    s[24..2072].copy_from_slice(&user[..USER]);
    let e = edc(&s[16..2072]);
    s[2072..2076].copy_from_slice(&e.to_le_bytes());
    let h = [s[12], s[13], s[14], s[15]];
    s[12..16].fill(0);
    parity(s);
    s[12..16].copy_from_slice(&h);
}

/// A mode 2 form 2 sector: the subheader twice, 2,324 bytes and the EDC over the subheader and data; no parity.
pub fn mode2_form2(at: u32, sub: [u8; 4], user: &[u8], s: &mut [u8; RAW]) {
    header(at, 2, s);
    s[16..20].copy_from_slice(&sub);
    s[20..24].copy_from_slice(&sub);
    s[24..2348].copy_from_slice(&user[..FORM2_USER]);
    let e = edc(&s[16..2348]);
    s[2348..2352].copy_from_slice(&e.to_le_bytes());
}

/// What a raw sector holds, read from its header and subheader.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    Mode0,
    Mode1,
    Form1,
    Form2,
    /// No sync pattern: audio, or nothing readable as data.
    NoSync,
}

pub fn form(s: &[u8]) -> Form {
    if s.len() < 24 || s[..12] != SYNC {
        return Form::NoSync;
    }
    match s[15] {
        1 => Form::Mode1,
        2 if s[18] & submode::FORM2 != 0 => Form::Form2,
        2 => Form::Form1,
        _ => Form::Mode0,
    }
}

/// Whether a sector's EDC is right for its form: the CRC over the covered bytes and the stored EDC, taken together,
/// leaves no remainder (§14.3's "divisible by the check polynomial"). A form 2 EDC of zero means none was written.
pub fn edc_holds(s: &[u8; RAW]) -> bool {
    match form(s) {
        Form::Mode1 => edc(&s[..2068]) == 0,
        Form::Form1 => edc(&s[16..2076]) == 0,
        Form::Form2 => s[2348..] == [0; 4] || edc(&s[16..2352]) == 0,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Polynomial product over GF(2), bit i the coefficient of xⁱ.
    fn mul(a: u64, b: u64) -> u64 {
        (0..64).filter(|i| b >> i & 1 != 0).fold(0, |p, i| p ^ a << i)
    }

    #[test]
    fn the_edc_polynomial_is_the_standards_product_reflected() {
        let p = mul(1 << 16 | 1 << 15 | 1 << 2 | 1, 1 << 16 | 1 << 2 | 1 << 1 | 1);
        assert_eq!(p >> 32, 1, "degree 32");
        let low = p as u32;
        assert_eq!(low.reverse_bits(), EDC_POLY);
    }

    /// The table against a bit-at-a-time division of the message, least significant bit first, by P(x).
    #[test]
    fn the_edc_is_the_remainder_of_a_bitwise_division() {
        let data: Vec<u8> = (0..2064u32).map(|i| (i * 37 + 11) as u8).collect();
        let mut r = 0u32;
        for &b in &data {
            for k in 0..8 {
                let bit = (b >> k & 1) as u32 ^ (r & 1);
                r = (r >> 1) ^ if bit != 0 { EDC_POLY } else { 0 };
            }
        }
        assert_eq!(edc(&data), r);
        let mut with = data.clone();
        with.extend_from_slice(&r.to_le_bytes());
        assert_eq!(edc(&with), 0, "§14.3: the codeword divides by P(x), x⁰ in byte 2,067's top bit");
    }

    #[test]
    fn alpha_generates_the_field() {
        assert_eq!(alpha(8), 0x1D, "α⁸ = x⁴ + x³ + x² + 1");
        let mut seen = [false; 256];
        for i in 0..255 {
            seen[alpha(i) as usize] = true;
        }
        assert_eq!(seen.iter().filter(|&&s| s).count(), 255);
        assert_eq!(gf_mul(alpha(200), alpha(100)), alpha(45));
    }

    #[test]
    fn every_form_meets_its_checks_and_a_changed_byte_breaks_them() {
        let user: Vec<u8> = (0..FORM2_USER).map(|i| (i * 7 + 3) as u8).collect();
        let mut s = [0u8; RAW];
        mode1(16 + 150, &user, &mut s);
        assert!(edc_holds(&s) && parity_holds(&s) && form(&s) == Form::Mode1);
        assert_eq!(&s[12..16], &[0x00, 0x02, 0x16, 0x01]);
        s[500] ^= 1;
        assert!(!edc_holds(&s) && !parity_holds(&s));

        let mut f1 = [0u8; RAW];
        mode2_form1(1000, [1, 2, submode::DATA, 0], &user, &mut f1);
        assert!(edc_holds(&f1) && form(&f1) == Form::Form1);
        assert!(!parity_holds(&f1), "form 1's parity is over a zero header");
        let mut zeroed = f1;
        zeroed[12..16].fill(0);
        assert!(parity_holds(&zeroed));

        let mut f2 = [0u8; RAW];
        mode2_form2(1000, [1, 2, submode::AUDIO | submode::FORM2 | submode::REAL_TIME, 0], &user, &mut f2);
        assert!(edc_holds(&f2) && form(&f2) == Form::Form2);
        f2[100] ^= 0x80;
        assert!(!edc_holds(&f2));
    }
}
