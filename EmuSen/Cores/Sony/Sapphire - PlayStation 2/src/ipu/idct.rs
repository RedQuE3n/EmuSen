//! The inverse DCT as H.262 Annex A defines its reference, the saturated mathematical integer-number IDCT: the real
//! transform f(x, y) = 1/4 ΣΣ C(u)C(v)F(u, v)cos((2x+1)uπ/16)cos((2y+1)vπ/16), rounded to the nearest integer with
//! halves away from zero, saturated to [−256, 255]. It is evaluated in double precision, separably (rows, then
//! columns), from cosines written as the nearest doubles, whose error is below 10^-10 for any block of the standard's
//! range. A value within 2^-20 of a half-integer is decided exactly instead: 8f(x, y) is an integer combination of
//! cos(kπ/16), k = 0 to 7, which are linearly independent over the rationals, so f is a half-integer only when every
//! coefficient but the rational one is zero; otherwise its side of the half is read from the combination with the
//! cosines in fixed point to 2^-100. How the IPU's own transform departs from this is Sapphire_Disputes.md D-300; the
//! reading is Sapphire_Native.md §34.5.

/// C(u)/2 · cos((2x + 1)uπ/16), by u then x, each the double nearest the real value.
const C: [[f64; 8]; 8] = [
    [0.3535533905932738, 0.3535533905932738, 0.3535533905932738, 0.3535533905932738, 0.3535533905932738, 0.3535533905932738, 0.3535533905932738, 0.3535533905932738],
    [0.4903926402016152, 0.4157348061512726, 0.2777851165098011, 0.09754516100806414, -0.09754516100806414, -0.2777851165098011, -0.4157348061512726, -0.4903926402016152],
    [0.46193976625564337, 0.1913417161825449, -0.1913417161825449, -0.46193976625564337, -0.46193976625564337, -0.1913417161825449, 0.1913417161825449, 0.46193976625564337],
    [0.4157348061512726, -0.09754516100806414, -0.4903926402016152, -0.2777851165098011, 0.2777851165098011, 0.4903926402016152, 0.09754516100806414, -0.4157348061512726],
    [0.3535533905932738, -0.3535533905932738, -0.3535533905932738, 0.3535533905932738, 0.3535533905932738, -0.3535533905932738, -0.3535533905932738, 0.3535533905932738],
    [0.2777851165098011, -0.4903926402016152, 0.09754516100806414, 0.4157348061512726, -0.4157348061512726, -0.09754516100806414, 0.4903926402016152, -0.2777851165098011],
    [0.1913417161825449, -0.46193976625564337, 0.46193976625564337, -0.1913417161825449, -0.1913417161825449, 0.46193976625564337, -0.46193976625564337, 0.1913417161825449],
    [0.09754516100806414, -0.2777851165098011, 0.4157348061512726, -0.4903926402016152, 0.4903926402016152, -0.4157348061512726, 0.2777851165098011, -0.09754516100806414],
];

/// cos(kπ/16) × 2^100, rounded, k = 0 to 7.
const COS_FIXED: [i128; 8] = [
    1267650600228229401496703205376,
    1243293049398167358316279903776,
    1171156443926508635938050024219,
    1054012953106854663288574085998,
    896364335596578238699711011639,
    704268939356236033728032079564,
    485108882735005078916833875245,
    247306363802463555846379077689,
];

/// How near a half-integer a double-precision result is decided exactly.
pub const NEAR: f64 = 1.0 / (1 << 20) as f64;

/// The real transform of `coef` (F[v][u] at index 8v + u) into `out` (f[y][x] at 8y + x), in double precision.
pub fn real(coef: &[i32; 64], out: &mut [f64; 64]) {
    let mut rows = [0.0f64; 64];
    for v in 0..8 {
        let f = &coef[8 * v..8 * v + 8];
        if f.iter().all(|&c| c == 0) {
            continue;
        }
        for x in 0..8 {
            let mut s = 0.0;
            for u in 0..8 {
                s += C[u][x] * f64::from(f[u]);
            }
            rows[8 * v + x] = s;
        }
    }
    for y in 0..8 {
        for x in 0..8 {
            let mut s = 0.0;
            for v in 0..8 {
                s += C[v][y] * rows[8 * v + x];
            }
            out[8 * y + x] = s;
        }
    }
}

/// 8f(x, y) as integers n_k of cos(kπ/16), k = 0 to 7: each term C(u)C(v)F cos(A)cos(B)/4 is F[cos(A + B) +
/// cos(A − B)]/8, A the angle of u's factor in sixteenths of π, (2x + 1)u, or 4 for u = 0 since C(0) is cos(π/4); B
/// likewise for v and y.
pub fn combination(coef: &[i32; 64], x: usize, y: usize) -> [i64; 8] {
    let mut n = [0i64; 8];
    let mut add = |m: i64, f: i64| {
        let m = m.rem_euclid(32);
        let m = if m > 16 { 32 - m } else { m };
        match m {
            0..=7 => n[m as usize] += f,
            8 => {}
            _ => n[(16 - m) as usize] -= f,
        }
    };
    for v in 0..8 {
        for u in 0..8 {
            let f = i64::from(coef[8 * v + u]);
            if f != 0 {
                let a = if u == 0 { 4 } else { ((2 * x + 1) * u) as i64 };
                let b = if v == 0 { 4 } else { ((2 * y + 1) * v) as i64 };
                add(a + b, f);
                add(a - b, f);
            }
        }
    }
    n
}

/// round() of f(x, y), decided exactly from `combination` (halves away from zero).
pub fn exact(coef: &[i32; 64], x: usize, y: usize) -> i32 {
    let n = combination(coef, x, y);
    if n[1..].iter().all(|&k| k == 0) {
        let a = (n[0].abs() + 4) / 8;
        return (if n[0] < 0 { -a } else { a }) as i32;
    }
    let v: i128 = n.iter().zip(COS_FIXED).map(|(&k, c)| i128::from(k) * c).sum();
    let a = (v.abs() + (1i128 << 102)) >> 103;
    (if v < 0 { -a } else { a }) as i32
}

/// The saturated integer transform: f''[y][x] in [−256, 255].
pub fn idct(coef: &[i32; 64], out: &mut [i16; 64]) {
    let mut f = [0.0f64; 64];
    real(coef, &mut f);
    for (k, (o, v)) in out.iter_mut().zip(f).enumerate() {
        let a = v.abs();
        let r = if (a - a.floor() - 0.5).abs() <= NEAR { exact(coef, k % 8, k / 8) } else { v.round() as i32 };
        *o = r.clamp(-256, 255) as i16;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn annex_a_item_4_blocks_give_the_exact_saturated_transform() {
        for i in 0..4096i32 {
            let mut b = [0i32; 64];
            b[0] = i - 2048;
            b[63] = i32::from(b[0] % 2 == 0);
            let mut out = [0i16; 64];
            idct(&b, &mut out);
            for k in 0..64 {
                assert_eq!(i32::from(out[k]), exact(&b, k % 8, k / 8).clamp(-256, 255), "block {i} at {k}");
            }
        }
    }

    #[test]
    fn a_dc_alone_rounds_its_halves_away_from_zero() {
        let mut out = [0i16; 64];
        for (dc, want) in [(4, 1), (-4, -1), (12, 2), (-12, -2), (3, 0), (5, 1), (2047, 255), (-2048, -256)] {
            let mut b = [0i32; 64];
            b[0] = dc;
            idct(&b, &mut out);
            assert!(out.iter().all(|&o| i32::from(o) == want), "DC {dc}: {:?}", &out[..4]);
        }
    }

    #[test]
    fn the_rational_terms_give_exact_halves_and_the_exact_reading_agrees_everywhere() {
        let mut b = [0i32; 64];
        b[0] = 4;
        b[4] = 8;
        b[32] = -8;
        b[36] = 16;
        for k in 0..64 {
            let n = combination(&b, k % 8, k / 8);
            assert!(n[1..].iter().all(|&c| c == 0), "F(0,0), F(0,4), F(4,0), F(4,4) are rational at {k}");
        }
        let mut seed = 0x5EED_u64;
        let mut out = [0i16; 64];
        for _ in 0..3000 {
            let mut b = [0i32; 64];
            for c in b.iter_mut() {
                seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
                let r = (seed >> 33) as i32;
                *c = if r % 3 == 0 { (r >> 4) % 64 - 32 } else { 0 };
            }
            idct(&b, &mut out);
            for k in 0..64 {
                assert_eq!(i32::from(out[k]), exact(&b, k % 8, k / 8).clamp(-256, 255), "at {k}");
            }
        }
    }
}
