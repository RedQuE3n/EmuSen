//! The inverse DCT as H.262 Annex A defines its reference, the saturated mathematical integer-number IDCT: the real
//! transform f(x, y) = 1/4 ΣΣ C(u)C(v)F(u, v)cos((2x+1)uπ/16)cos((2y+1)vπ/16), rounded to the nearest integer with
//! halves away from zero, saturated to [−256, 255]. It is evaluated in double precision, separably (rows, then
//! columns), from cosines written as the nearest doubles; a result within 2^-20 of a half-integer is taken as that
//! half, since integer coefficients give halves exactly (a DC alone, or with F(0,4), F(4,0), F(4,4), whose cosine
//! products are rational) and double precision misses them by about 10^-13. How the IPU's own transform departs from
//! this is Sapphire_Disputes.md D-300; the reading is Sapphire_Native.md §34.5.

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

/// How near a half-integer a result is taken to be one.
pub const TIE: f64 = 1.0 / (1 << 20) as f64;

/// round(), Annex A's: to the nearest integer, halves away from zero, a result within `TIE` of a half counted as one.
pub fn round(f: f64) -> i32 {
    let a = f.abs();
    let fl = a.floor();
    let m = if (a - fl - 0.5).abs() <= TIE { fl + 1.0 } else { a.round() };
    (if f < 0.0 { -m } else { m }) as i32
}

/// The real transform of `coef` (F[v][u] at index 8v + u) into `out` (f[y][x] at 8y + x).
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

/// The saturated integer transform: f''[y][x] in [−256, 255].
pub fn idct(coef: &[i32; 64], out: &mut [i16; 64]) {
    let mut f = [0.0f64; 64];
    real(coef, &mut f);
    for (o, v) in out.iter_mut().zip(f) {
        *o = round(v).clamp(-256, 255) as i16;
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
            for y in 0..8 {
                for x in 0..8 {
                    let c77 = ((2 * x + 1) as f64 * 7.0 * std::f64::consts::PI / 16.0).cos() * ((2 * y + 1) as f64 * 7.0 * std::f64::consts::PI / 16.0).cos();
                    let exact = f64::from(b[0]) / 8.0 + f64::from(b[63]) * c77 / 4.0;
                    let want = round(exact).clamp(-256, 255);
                    assert_eq!(i32::from(out[8 * y + x]), want, "block {i} at ({x}, {y})");
                }
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
}
