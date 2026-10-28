//! R0.4's timing of `float_lanes` (Sapphire_Redesign.md, R0 results): the adder, the multiplier and the multiply-add four
//! lanes at a time with the MAC flags, against `float`'s functions lane by lane, over operands drawn in the shares the
//! census of Sapphire_Speed.md §56.2 found. Independent vectors (throughput) and a chain through the accumulator
//! (latency); the forms without the MAC word, on the short path, and two operations at once in eight lanes; the median
//! of the rounds, in nanoseconds a lane operation. Graded bit-equal by the module's tests.
//!
//! `cargo run --release --features lanes --example lanes -- [rounds]`

use std::hint::black_box;
use std::time::Instant;

use sapphire::float;
use sapphire::float_lanes as lanes;

const N: usize = 1024;
const REPEAT: usize = 4096;

fn stream(mut seed: u64) -> impl FnMut() -> u64 {
    move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    }
}

/// An operand in §56.2's shares: four in five normal near the centre, a tenth zero, a twentieth would-be denormals, the
/// rest any field; a random, zero or short fraction; either sign.
fn operand(r: u64) -> u32 {
    let e = match r % 100 {
        0..80 => 127 + (r >> 8 & 63) as u32 - 32,
        80..90 => 0,
        90..95 => 0,
        _ => (r >> 8 & 255) as u32,
    };
    let f = match r >> 20 & 3 {
        0 => 0,
        1 => (r >> 24) as u32 & 0x70_0000,
        _ => (r >> 24) as u32 & 0x7F_FFFF,
    };
    let f = if (80..90).contains(&(r % 100)) { 0 } else if (90..95).contains(&(r % 100)) { f | 1 } else { f };
    ((r >> 63) as u32) << 31 | e << 23 | f
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

#[target_feature(enable = "avx2")]
fn vector(op: u8, a: &[[u32; 4]], b: &[[u32; 4]], c: &[[u32; 4]]) -> u32 {
    let mut out = vec![lanes::Vector::default(); N];
    let old = [0u32; 4];
    for _ in 0..REPEAT {
        for i in 0..N {
            out[i] = match op {
                0 => lanes::add(&a[i], &b[i], &old, 0xF),
                1 => lanes::mul(&a[i], &b[i], &old, 0xF),
                _ => {
                    let f = lanes::fused(&c[i], &a[i], &b[i], false, &old, 0xF);
                    lanes::Vector { bits: f.value.bits, mac: f.value.mac ^ f.product.mac }
                }
            };
        }
        black_box(&mut out);
    }
    out.iter().fold(0, |h, v| h ^ v.bits[0] ^ v.mac)
}

/// The other four-lane forms: 3 `add_bits`, 4 `mul_bits`, 5 `add_fast`.
#[target_feature(enable = "avx2")]
fn forms(op: u8, a: &[[u32; 4]], b: &[[u32; 4]]) -> u32 {
    let mut out = vec![[0u32; 4]; N];
    let old = [0u32; 4];
    for _ in 0..REPEAT {
        for i in 0..N {
            out[i] = match op {
                3 => lanes::add_bits(&a[i], &b[i], &old, 0xF),
                4 => lanes::mul_bits(&a[i], &b[i], &old, 0xF),
                _ => {
                    let v = lanes::add_fast(&a[i], &b[i], &old, 0xF);
                    [v.bits[0], v.bits[1], v.bits[2], v.mac]
                }
            };
        }
        black_box(&mut out);
    }
    out.iter().fold(0, |h, v| h ^ v[0] ^ v[3])
}

/// Two operations at once in eight lanes: 0 `add2`, 1 `mul2`, 2 `fused2`, 3 `add2_bits`, 4 `mul2_bits`.
#[target_feature(enable = "avx2")]
fn paired(op: u8, a: &[[u32; 8]], b: &[[u32; 8]], c: &[[u32; 8]]) -> u32 {
    let mut out = vec![[0u32; 8]; N / 2];
    let old = [0u32; 8];
    let mask = 0xF000F;
    for _ in 0..REPEAT {
        for i in 0..N / 2 {
            out[i] = match op {
                0 => lanes::add2(&a[i], &b[i], &old, mask).bits,
                1 => lanes::mul2(&a[i], &b[i], &old, mask).bits,
                2 => {
                    let f = lanes::fused2(&c[i], &a[i], &b[i], false, &old, mask);
                    let mut v = f.value.bits;
                    v[7] ^= f.value.mac ^ f.product.mac;
                    v
                }
                3 => lanes::add2_bits(&a[i], &b[i], &old, mask),
                _ => lanes::mul2_bits(&a[i], &b[i], &old, mask),
            };
        }
        black_box(&mut out);
    }
    out.iter().fold(0, |h, v| h ^ v[0] ^ v[7])
}

#[target_feature(enable = "avx2")]
fn chain(a: &[[u32; 4]], b: &[[u32; 4]]) -> u32 {
    let mut acc = [0x3F80_0000u32; 4];
    let old = [0u32; 4];
    let mut mac = 0;
    for _ in 0..REPEAT {
        for i in 0..N {
            let f = lanes::fused(&acc, &a[i], &b[i], i & 1 == 1, &old, 0xF);
            acc = f.value.bits;
            mac ^= f.value.mac;
        }
    }
    acc[0] ^ acc[3] ^ mac
}

fn scalar(op: u8, a: &[[u32; 4]], b: &[[u32; 4]], c: &[[u32; 4]]) -> u32 {
    let mut h = 0u32;
    for _ in 0..REPEAT / 4 {
        for i in 0..N {
            for l in 0..4 {
                let o = match op {
                    0 => float::add(a[i][l], b[i][l]),
                    1 => float::mul(a[i][l], b[i][l]),
                    _ => float::fused(c[i][l], a[i][l], b[i][l], false).value,
                };
                h = h.rotate_left(5) ^ o.bits ^ u32::from(o.overflow) ^ u32::from(o.underflow) << 1;
            }
        }
    }
    h
}

fn main() {
    assert!(lanes::available(), "the kernel needs AVX2");
    let rounds: usize = std::env::args().nth(1).map_or(7, |a| a.parse().expect("a count of rounds"));
    let mut next = stream(0x5A99_41E5_0000_0004);
    let mut draw = || (0..N).map(|_| [0; 4].map(|_: u32| operand(next()))).collect::<Vec<[u32; 4]>>();
    let (a, b, c) = (draw(), draw(), draw());
    let lane_ops = (N * REPEAT * 4) as f64;
    let time = |f: &dyn Fn() -> u32, ops: f64| median((0..rounds).map(|_| {
        let t = Instant::now();
        black_box(f());
        t.elapsed().as_secs_f64() * 1e9 / ops
    }).collect());
    for (op, name) in [(0u8, "add"), (1, "mul"), (2, "madd")] {
        // SAFETY: AVX2 was asserted above.
        let v = time(&|| unsafe { vector(op, black_box(&a), black_box(&b), black_box(&c)) }, lane_ops);
        let s = time(&|| scalar(op, black_box(&a), black_box(&b), black_box(&c)), lane_ops / 4.0);
        println!("{name}\tlanes {v:.3} ns a lane operation\tfloat, lane by lane {s:.3}\t{:.1} times", s / v);
    }
    let pair = |v: &[[u32; 4]]| v.as_chunks::<2>().0.iter().map(|p| [p[0][0], p[0][1], p[0][2], p[0][3], p[1][0], p[1][1], p[1][2], p[1][3]]).collect::<Vec<[u32; 8]>>();
    let (a8, b8, c8) = (pair(&a), pair(&b), pair(&c));
    for (op, name) in [(3u8, "add without the MAC word"), (4, "mul without the MAC word"), (5, "add on the short path")] {
        // SAFETY: as above.
        let v = time(&|| unsafe { forms(op, black_box(&a), black_box(&b)) }, lane_ops);
        println!("{name}\tlanes {v:.3} ns a lane operation");
    }
    for (op, name) in [(0u8, "add"), (1, "mul"), (2, "madd"), (3, "add without the MAC word"), (4, "mul without the MAC word")] {
        // SAFETY: as above.
        let v = time(&|| unsafe { paired(op, black_box(&a8), black_box(&b8), black_box(&c8)) }, lane_ops);
        println!("{name}, two at once\tlanes {v:.3} ns a lane operation");
    }
    // SAFETY: as above.
    let l = time(&|| unsafe { chain(black_box(&a), black_box(&b)) }, lane_ops);
    println!("madd chained through the accumulator\tlanes {l:.3} ns a lane operation");
}
