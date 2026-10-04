//! Grades VenusRT_Native.md §40.1's sine family against the image's Triangle (04h): `dsp_sine <chip> [seeded]`
//! runs every angle at five radii and the seeded pairs, and prints the members that reproduce every case, or the
//! closest. Counts only; no value from the image is printed.
use venusrt::chips::dsporacle::*;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Member {
    bits: u32,
    full_amplitude: bool,
    half_up: bool,
    quarter: bool,
    lookup: u8,
    product_half_up: bool,
}

impl Member {
    fn table(&self) -> Vec<i32> {
        let n = 1usize << self.bits;
        let amp = if self.full_amplitude { 32768.0 } else { 32767.0 };
        let q = |x: f64| if self.half_up { (x + 0.5).floor() } else { x.floor() };
        (0..n)
            .map(|i| {
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
                            v.push(Member { bits, full_amplitude, half_up, quarter, lookup, product_half_up });
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
    let n = t.len();
    let i = (a as usize) >> shift;
    let frac = (a as i64) & ((1i64 << shift) - 1);
    (match m.lookup {
        0 => t[i],
        1 => t[((a as usize + ((1usize << shift) >> 1)) >> shift) % n],
        _ => t[i] + (((t[(i + 1) % n] - t[i]) as i64 * frac) >> shift) as i32,
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
    let mut total = 0u64;
    let mut p = Pcg::new(0x51_4E);
    let cases = (0..5 * 65536u64).map(|k| (k as u16, [0x7FFFu16, 0x4000, 1, 0x8000, 0xFFFF][(k >> 16) as usize])).chain((0..seeded).map(|_| (p.word(), p.word()))).collect::<Vec<_>>();
    for (angle, r) in cases {
        let got = transact(&mut chip, &mut Host::steady(), 0x04, &[angle, r]).outputs();
        total += 1;
        for (k, m) in all.iter().enumerate() {
            let t = &tables[k];
            if got == [product(m, r, sine(m, t, angle)), product(m, r, sine(m, t, angle.wrapping_add(0x4000)))] {
                hits[k] += 1;
            }
        }
    }
    let mut order: Vec<usize> = (0..all.len()).collect();
    order.sort_by(|&x, &y| hits[y].cmp(&hits[x]));
    println!("{stem} 04h: {total} cases, {} members", all.len());
    let only: Option<u32> = std::env::var("ZZ_BITS").ok().and_then(|b| b.parse().ok());
    for &k in order.iter().filter(|&&k| only.is_none_or(|b| all[k].bits == b)).take(8) {
        println!("  {:?}: {} of {total}{}", all[k], hits[k], if hits[k] == total { "  all" } else { "" });
    }
}
