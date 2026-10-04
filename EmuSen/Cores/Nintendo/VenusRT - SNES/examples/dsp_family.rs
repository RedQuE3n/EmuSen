//! Grades a declared family of variants for a DSP-1 command against the image (VenusRT_DspHle.md §1.3, R2):
//! `dsp_family <chip> <command> [cases]` prints, per variant, how many of the edge and seeded cases it reproduces.
//! Counts only; no value from the image is printed. See VenusRT_Native.md §39.
use venusrt::chips::dsporacle::*;

/// The edge values of plan §4.1: 0, ±1, the extremes, the powers of two and their neighbours.
fn edges() -> Vec<u16> {
    let mut v: Vec<u16> = vec![0, 1, 0xFFFF, 0x7FFF, 0x8000, 0x8001, 0x7FFE];
    for b in 0..16 {
        let p = 1u16 << b;
        v.extend([p, p.wrapping_sub(1), p.wrapping_add(1), p.wrapping_neg()]);
    }
    v.sort();
    v.dedup();
    v
}

#[derive(Clone, Copy, Debug)]
enum Round {
    Floor,
    HalfUp,
    HalfEven,
    TowardZero,
}

const ROUNDS: [Round; 4] = [Round::Floor, Round::HalfUp, Round::HalfEven, Round::TowardZero];

/// x / 2^n by the rounding rule.
fn shift(x: i64, n: u32, r: Round) -> i64 {
    let d = 1i64 << n;
    let floor = x.div_euclid(d);
    let rem = x.rem_euclid(d);
    match r {
        Round::Floor => floor,
        Round::HalfUp => (x + d / 2).div_euclid(d),
        Round::HalfEven => {
            if rem * 2 > d || rem * 2 == d && floor & 1 == 1 {
                floor + 1
            } else {
                floor
            }
        }
        Round::TowardZero => {
            if x < 0 && rem != 0 {
                floor + 1
            } else {
                floor
            }
        }
    }
}

fn fit16(v: i64, saturate: bool) -> u16 {
    if saturate { v.clamp(-0x8000, 0x7FFF) as u16 } else { v as u16 }
}

fn fit32(v: i64, saturate: bool) -> i64 {
    if saturate { v.clamp(-0x8000_0000, 0x7FFF_FFFF) } else { v as i32 as i64 }
}

/// The family's members for a command, each a name and its outputs for the inputs.
fn family(command: u8) -> Vec<(String, Box<dyn Fn(&[u16]) -> Vec<u16>>)> {
    let s = |w: u16| w as i16 as i64;
    let mut out: Vec<(String, Box<dyn Fn(&[u16]) -> Vec<u16>>)> = Vec::new();
    match command {
        0x00 => {
            for r in ROUNDS {
                for sat in [false, true] {
                    out.push((format!("{r:?} {}", if sat { "saturate" } else { "wrap" }), Box::new(move |i: &[u16]| vec![fit16(shift(s(i[0]) * s(i[1]), 15, r), sat)])));
                }
            }
        }
        // §39.2's widening, once and in writing: the low bit forced, or one added.
        0x20 => {
            for (name, f) in [("Floor | 1", (|x: i64| x | 1) as fn(i64) -> i64), ("Floor + 1", |x: i64| x + 1)] {
                for sat in [false, true] {
                    out.push((format!("{name} {}", if sat { "saturate" } else { "wrap" }), Box::new(move |i: &[u16]| vec![fit16(f(shift(s(i[0]) * s(i[1]), 15, Round::Floor)), sat)])));
                }
            }
        }
        0x08 => {
            for sh in [-1i32, 0, 1] {
                for sat in [false, true] {
                    out.push((
                        format!("s={sh} {}", if sat { "saturate" } else { "wrap" }),
                        Box::new(move |i: &[u16]| {
                            let sum = s(i[0]) * s(i[0]) + s(i[1]) * s(i[1]) + s(i[2]) * s(i[2]);
                            let v = fit32(if sh < 0 { sum >> 1 } else { sum << sh }, sat);
                            vec![v as u16, (v >> 16) as u16]
                        }),
                    ));
                }
            }
        }
        0x18 | 0x38 => {
            for sh in [-1i32, 0, 1] {
                for sat in [false, true] {
                    for r in [Round::Floor, Round::HalfUp] {
                        out.push((
                            format!("s={sh} {} {r:?}", if sat { "saturate" } else { "wrap" }),
                            Box::new(move |i: &[u16]| {
                                let sum = s(i[0]) * s(i[0]) + s(i[1]) * s(i[1]) + s(i[2]) * s(i[2]) - s(i[3]) * s(i[3]);
                                let v = fit32(if sh < 0 { sum >> 1 } else { sum << sh }, sat);
                                vec![shift(v, 16, r) as u16]
                            }),
                        ));
                    }
                }
            }
            if command == 0x38 {
                for (name, up) in [("Floor + 1", false), ("Ceil", true)] {
                    for sat in [false, true] {
                        out.push((
                            format!("s=1 {} {name}", if sat { "saturate" } else { "wrap" }),
                            Box::new(move |i: &[u16]| {
                                let sum = s(i[0]) * s(i[0]) + s(i[1]) * s(i[1]) + s(i[2]) * s(i[2]) - s(i[3]) * s(i[3]);
                                let v = fit32(sum << 1, sat);
                                vec![if up { -((-v) >> 16) } else { (v >> 16) + 1 } as u16]
                            }),
                        ));
                    }
                }
            }
        }
        0x0F => {
            for c in [0x0000u16, 0x0001, 0x00FF, 0xFFFF] {
                out.push((format!("{c:04X}"), Box::new(move |_: &[u16]| vec![c])));
            }
        }
        _ => panic!("no family declared for {command:02X}"),
    }
    out
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let stem = &a[1];
    let command = u8::from_str_radix(&a[2], 16).unwrap();
    let cases: u64 = a.get(3).and_then(|c| c.parse().ok()).unwrap_or(1 << 20);
    let image = firmware(stem).unwrap_or_else(|| std::process::exit(1));
    let mut chip = idle_chip(stem, &image, &mut Host::steady(), 0).unwrap();
    let n = transact(&mut chip.clone(), &mut Host::steady(), command, &[0; 8]).inputs().max(1);
    let members = family(command);
    let mut hits = vec![0u64; members.len()];
    let mut total = 0u64;
    let e = edges();
    let mut p = Pcg::new(0xF4_0000 + command as u64);
    let mut host = Host { cap: 2_000_000, ..Host::steady() };
    let edge_cases = e.len().pow(n.min(2) as u32) as u64;
    for k in 0..edge_cases + cases {
        let set: Vec<u16> = if k < edge_cases {
            let mut v = vec![e[(k as usize) % e.len()]; n];
            if n > 1 {
                v[1] = e[(k as usize / e.len()) % e.len()];
            }
            for (j, w) in v.iter_mut().enumerate().skip(2) {
                *w = e[(k as usize + j * 7) % e.len()];
            }
            v
        } else {
            (0..n).map(|_| p.word()).collect()
        };
        let t = transact(&mut chip, &mut host, command, &set);
        assert!(matches!(t.end, End::Idle { .. }), "the chip did not return to idle");
        let got = t.outputs();
        total += 1;
        for (m, (_, f)) in members.iter().enumerate() {
            hits[m] += (f(&set) == got) as u64;
        }
    }
    println!("{stem} {command:02X}: {total} cases ({edge_cases} edge, {cases} seeded, {n} inputs)");
    for (m, (name, _)) in members.iter().enumerate() {
        println!("  {name:<28} {} of {total}{}", hits[m], if hits[m] == total { "  all" } else { "" });
    }
}
