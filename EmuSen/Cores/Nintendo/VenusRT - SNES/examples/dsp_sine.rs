//! Grades VenusRT_Native.md §40.1's sine family against the image's Triangle (04h): `dsp_sine <chip> [seeded]`
//! runs every angle at five radii and the seeded pairs, and prints the members that reproduce every case, or the
//! closest. Counts only; no value from the image is printed. DSP_SINE_BITS lists only members of that table size;
//! DSP_SINE_HISTOGRAM=<member> prints that member's differences by size and radius.
use venusrt::chips::dsporacle::*;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Member {
    bits: u32,
    full_amplitude: bool,
    half_up: bool,
    quarter: bool,
    lookup: u8,
    product_half_up: bool,
    /// §40.2's widening: a derivative table quantised by floor (false) or half up, and the step's rounding.
    taylor: Option<(bool, bool)>,
}

impl Member {
    /// The sine table, then for §40.2's members the derivative table after it.
    fn table(&self) -> Vec<i32> {
        let n = 1usize << self.bits;
        let mut t = self.wave(self.half_up, 1.0, 0.0);
        if let Some((d_half_up, _)) = self.taylor {
            t.extend(self.wave(d_half_up, 2.0 * std::f64::consts::PI / n as f64, 0.25));
        }
        t
    }

    /// Q(A·scale·sin(2π(i/N + phase))) for every i, direct or mirrored from a quarter wave.
    fn wave(&self, half_up: bool, scale: f64, phase: f64) -> Vec<i32> {
        let n = 1usize << self.bits;
        let amp = if self.full_amplitude { 32768.0 } else { 32767.0 } * scale;
        let q = |x: f64| if half_up { (x + 0.5).floor() } else { x.floor() };
        let shift = (phase * n as f64) as usize;
        (0..n)
            .map(|i0| {
                let i = (i0 + shift) % n;
                let v = if self.quarter {
                    let half = i % (n / 2);
                    let k = if half > n / 4 { n / 2 - half } else { half };
                    let m = q(amp * (2.0 * std::f64::consts::PI * k as f64 / n as f64).sin());
                    if i >= n / 2 { -m } else { m }
                } else {
                    q(amp * (2.0 * std::f64::consts::PI * i as f64 / n as f64).sin())
                };
                (v as i32).clamp(-0x8000, 0x7FFF)
            })
            .collect()
    }
}

fn members() -> Vec<Member> {
    let mut v = Vec::new();
    for bits in [6, 7, 8, 9, 10, 11, 12, 16] {
        for full_amplitude in [true, false] {
            for half_up in [false, true] {
                for quarter in [false, true] {
                    for lookup in 0..3u8 {
                        for product_half_up in [false, true] {
                            if bits == 16 && lookup != 0 {
                                continue;
                            }
                            v.push(Member { bits, full_amplitude, half_up, quarter, lookup, product_half_up, taylor: None });
                        }
                    }
                }
            }
        }
    }
    for bits in [7, 8, 9] {
        for full_amplitude in [true, false] {
            for half_up in [false, true] {
                for d_half_up in [false, true] {
                    for step_half_up in [false, true] {
                        for quarter in [false, true] {
                            for product_half_up in [false, true] {
                                v.push(Member { bits, full_amplitude, half_up, quarter, lookup: 3, product_half_up, taylor: Some((d_half_up, step_half_up)) });
                            }
                        }
                    }
                }
            }
        }
    }
    v
}

/// The member's sine of angle `a`: lookup 0 the top bits, 1 the nearest entry, 2 the top bits with linear interpolation.
fn sine(m: &Member, t: &[i32], a: u16) -> i64 {
    let shift = 16 - m.bits;
    let n = 1usize << m.bits;
    let i = (a as usize) >> shift;
    let frac = (a as i64) & ((1i64 << shift) - 1);
    (match m.lookup {
        0 => t[i],
        1 => t[((a as usize + ((1usize << shift) >> 1)) >> shift) % n],
        2 => t[i] + (((t[(i + 1) % n] - t[i]) as i64 * frac) >> shift) as i32,
        _ => {
            let f = frac << (15 - shift);
            let step = f * t[n + i] as i64;
            t[i] + (if m.taylor.is_some_and(|(_, h)| h) { (step + (1 << 14)) >> 15 } else { step >> 15 }) as i32
        }
    }) as i64
}

fn product(m: &Member, r: u16, s: i64) -> u16 {
    let p = r as i16 as i64 * s;
    (if m.product_half_up { (p + (1 << 14)) >> 15 } else { p >> 15 }) as u16
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let stem = &a[1];
    let seeded: u64 = a.get(2).and_then(|c| c.parse().ok()).unwrap_or(1 << 20);
    let image = firmware(stem).unwrap_or_else(|| std::process::exit(1));
    let mut chip = idle_chip(stem, &image, &mut Host::steady(), 0).unwrap();
    let all = members();
    let tables: Vec<Vec<i32>> = all.iter().map(Member::table).collect();
    let mut hits = vec![0u64; all.len()];
    let mut worst = vec![0i64; all.len()];
    let mut total = 0u64;
    let mut p = Pcg::new(0x51_4E);
    let cases = (0..5 * 65536u64).map(|k| (k as u16, [0x7FFFu16, 0x4000, 1, 0x8000, 0xFFFF][(k >> 16) as usize])).chain((0..seeded).map(|_| (p.word(), p.word()))).collect::<Vec<_>>();
    for (angle, r) in cases {
        let got = transact(&mut chip, &mut Host::steady(), 0x04, &[angle, r]).outputs();
        total += 1;
        for (k, m) in all.iter().enumerate() {
            let t = &tables[k];
            let want = [product(m, r, sine(m, t, angle)), product(m, r, sine(m, t, angle.wrapping_add(0x4000)))];
            if got == want {
                hits[k] += 1;
            }
            let e = got.iter().zip(want).map(|(&g, w)| (g as i16 as i64 - w as i16 as i64).abs()).max().unwrap_or(0);
            worst[k] = worst[k].max(e);
        }
    }
    if let Ok(pick) = std::env::var("DSP_SINE_HISTOGRAM") {
        let k: usize = pick.parse().unwrap();
        let mut h = std::collections::BTreeMap::<(i64, u16), u64>::new();
        let mut p = Pcg::new(0x51_4E);
        let cases = (0..5 * 65536u64).map(|k| (k as u16, [0x7FFFu16, 0x4000, 1, 0x8000, 0xFFFF][(k >> 16) as usize])).chain((0..seeded).map(|_| (p.word(), p.word()))).collect::<Vec<_>>();
        let (m, t) = (&all[k], &tables[k]);
        for (angle, r) in cases {
            let got = transact(&mut chip, &mut Host::steady(), 0x04, &[angle, r]).outputs();
            let want = product(m, r, sine(m, t, angle));
            let rr = if [0x7FFF, 0x4000, 1, 0x8000, 0xFFFF].contains(&r) { r } else { 0x1234 };
            *h.entry(((got[0] as i16 as i64 - want as i16 as i64).clamp(-20, 20), rr)).or_default() += 1;
        }
        println!("{:?}: {:?}", m, h);
        return;
    }
    let mut order: Vec<usize> = (0..all.len()).collect();
    order.sort_by(|&x, &y| hits[y].cmp(&hits[x]));
    println!("{stem} 04h: {total} cases, {} members", all.len());
    let only: Option<u32> = std::env::var("DSP_SINE_BITS").ok().and_then(|b| b.parse().ok());
    for &k in order.iter().filter(|&&k| only.is_none_or(|b| all[k].bits == b)).take(8) {
        println!("  [{k}] {:?}: {} of {total}, largest difference {}{}", all[k], hits[k], worst[k], if hits[k] == total { "  all" } else { "" });
    }
}
