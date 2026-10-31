//! R0.4's timing of `float_lanes` (Sapphire_Redesign.md, R0 results): the adder, the multiplier and the multiply-add four
//! lanes at a time with the MAC flags, against `float`'s functions lane by lane, over operands drawn in the shares the
//! census of Sapphire_Speed.md §56.2 found. Independent vectors (throughput) and a chain through the accumulator
//! (latency); the forms without the MAC word, on the short path, and two operations at once in eight lanes; the median
//! of the rounds, in nanoseconds a lane operation. Graded bit-equal by the module's tests.
//!
//! `cargo run --release --features lanes --example lanes -- [rounds]`
//! `cargo run --release --features lanes --example lanes -- mix FILE [rounds]`: a stream of VU operations in the shares a
//! count gives (FILE's lines `pair KA LA KB LB N` and `single K L N`, kinds 0 add, 1 mul, 2 multiply-add, L the lanes
//! written), each kind's operations in a run as compiled code would hold them (shuffled with `shuffle`, which adds the
//! host's mispredicted dispatch); a pair of one kind runs in eight lanes, any other operation in four; ns a lane
//! operation over the lanes written, a multiply-add's counting twice, and over the lanes computed; `wide` instead of
//! `shuffle` runs the AVX-512VL form, `host` and `host2` `float_host`'s two forms, `floor` the stream alone, `rz`
//! `float_rz` through its entry points and `rzreg` its kernel in registers (Sapphire_Redesign.md, the VU's floats under
//! rounding towards zero). A fifth argument names a file of recorded VU1 operations (`recorded`) to take the operands
//! from instead of drawing them; with `rz` or `rzreg` it also prints their census.

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

/// One operation of the stream: whether two run at once, their kinds, and the field mask (the second's in bits 19-16).
#[derive(Clone, Copy)]
struct Op {
    paired: bool,
    kinds: [u8; 2],
    mask: u32,
}

const fn lane_mask(l: u32) -> u32 {
    [0, 0x8, 0xC, 0xE, 0xF][l as usize]
}

/// The stream run's body over one form's six entry points.
macro_rules! stream_body {
    ($add:path, $mul:path, $fused:path, $add2:path, $mul2:path, $fused2:path, $ops:expr, $a:expr, $b:expr, $c:expr, $split:expr) => {{
        let (ops, a, b, c, split): (&[Op], &Pools, &Pools, &Pools, bool) = ($ops, $a, $b, $c, $split);
        let (old4, old8) = ([0u32; 4], [0u32; 8]);
        let mut h = 0u32;
        let half = |v: &[u32; 8], i: usize| -> [u32; 4] { [v[4 * i], v[4 * i + 1], v[4 * i + 2], v[4 * i + 3]] };
        for (i, op) in ops.iter().enumerate() {
            let k = op.kinds[0] as usize;
            let j = i % a[k].len();
            let (x, y, z) = (&a[k][j], &b[k][j], &c[k][j]);
            h ^= if op.paired && op.kinds[0] == op.kinds[1] && !split {
                match op.kinds[0] {
                    0 => $add2(x, y, &old8, op.mask).mac,
                    1 => $mul2(x, y, &old8, op.mask).mac,
                    _ => $fused2(z, x, y, false, &old8, op.mask).value.mac,
                }
            } else {
                let mut m = 0;
                for (n, (kind, mask)) in [(op.kinds[0], op.mask & 0xF), (op.kinds[1], op.mask >> 16)].into_iter().enumerate().take(if op.paired { 2 } else { 1 }) {
                    let (p, q, r) = (half(x, n), half(y, n), half(z, n));
                    m ^= match kind {
                        0 => $add(&p, &q, &old4, mask).mac,
                        1 => $mul(&p, &q, &old4, mask).mac,
                        _ => $fused(&r, &p, &q, false, &old4, mask).value.mac,
                    };
                }
                m
            };
        }
        h
    }};
}

/// The operands of each kind of operation (add, mul, multiply-add): drawn, the same three; recorded, each kind's own.
type Pools = [Vec<[u32; 8]>; 3];

/// The stream run: pairs of one kind in eight lanes, the rest in four (`split` runs every operation in four).
#[target_feature(enable = "avx2")]
fn stream_run(ops: &[Op], a: &Pools, b: &Pools, c: &Pools, split: bool) -> u32 {
    stream_body!(lanes::add, lanes::mul, lanes::fused, lanes::add2, lanes::mul2, lanes::fused2, ops, a, b, c, split)
}

/// The same on the AVX-512VL form.
#[target_feature(enable = "avx2,avx512f,avx512vl,avx512cd")]
fn stream_run512(ops: &[Op], a: &Pools, b: &Pools, c: &Pools, split: bool) -> u32 {
    use lanes::wide::{eight, four};
    stream_body!(four::add, four::mul, four::fused, eight::add2, eight::mul2, eight::fused2, ops, a, b, c, split)
}

/// The same on `float_host`'s `{rz-sae}` form.
#[target_feature(enable = "avx2,avx512f,avx512dq")]
fn stream_run_host(ops: &[Op], a: &Pools, b: &Pools, c: &Pools, split: bool) -> u32 {
    use sapphire::float_host::{eight, four};
    stream_body!(four::add, four::mul, four::fused, eight::add, eight::mul, eight::fused, ops, a, b, c, split)
}

/// The same on `float_host`'s AVX2 form.
#[target_feature(enable = "avx2")]
fn stream_run_host2(ops: &[Op], a: &Pools, b: &Pools, c: &Pools, split: bool) -> u32 {
    use sapphire::float_host::avx2 as h;
    stream_body!(h::add, h::mul, h::fused, h::add2, h::mul2, h::fused2, ops, a, b, c, split)
}

/// The same on `float_rz` through the entry points; MXCSR at `float_rz::MODE`.
#[target_feature(enable = "avx2")]
fn stream_run_rz(ops: &[Op], a: &Pools, b: &Pools, c: &Pools, split: bool) -> u32 {
    use sapphire::float_rz::{eight, four};
    stream_body!(four::add, four::mul, four::fused, eight::add, eight::mul, eight::fused, ops, a, b, c, split)
}

/// `float_rz`'s kernel as compiled code would hold it: operands loaded, the result stored, no MAC word, no field mask,
/// a vector left to the integer form counted and not computed; MXCSR at `float_rz::MODE`. Returns the vectors left.
#[target_feature(enable = "avx2")]
fn stream_regs(ops: &[Op], a: &Pools, b: &Pools, c: &Pools, out: &mut [[u32; 8]]) -> u32 {
    use sapphire::float_rz::raw;
    use std::arch::x86_64::*;
    let mut left = 0u32;
    for (i, op) in ops.iter().enumerate() {
        let k = op.kinds[0] as usize;
        let j = i % a[k].len();
        // SAFETY: eight lanes each.
        let (x, y, z) = unsafe { (_mm256_loadu_si256(a[k][j].as_ptr().cast()), _mm256_loadu_si256(b[k][j].as_ptr().cast()), _mm256_loadu_si256(c[k][j].as_ptr().cast())) };
        let n = if op.paired && op.kinds[0] != op.kinds[1] { 2 } else { 1 };
        let live = raw::on(op.mask);
        for kind in op.kinds.iter().take(n) {
            let (r, t) = match kind {
                0 => {
                    let (r, t) = raw::add(x, y);
                    (r, _mm256_and_si256(t, live))
                }
                1 => {
                    let (p, fix, t) = raw::mul(x, y);
                    (raw::fixed(p, x, y, _mm256_and_si256(fix, live)), _mm256_and_si256(t, live))
                }
                _ => {
                    let (r, _, t) = raw::fused(z, x, y, false, live);
                    (r, t)
                }
            };
            if raw::any(t) {
                left += 1;
            }
            // SAFETY: `out` holds a slot for each entry.
            unsafe { _mm256_storeu_si256(out[j].as_mut_ptr().cast(), r) };
        }
    }
    left
}

/// The stream's own cost: entry points that read their operands and return them, no arithmetic.
mod floor {
    use sapphire::float_lanes::{FusedPair, FusedVector, Pair, Vector};
    #[inline(never)]
    pub fn add(a: &[u32; 4], b: &[u32; 4], _: &[u32; 4], m: u32) -> Vector {
        Vector { bits: [a[0] ^ b[0], a[1] ^ b[1], a[2] ^ b[2], a[3] ^ b[3]], mac: m }
    }
    pub use add as mul;
    #[inline(never)]
    pub fn fused(c: &[u32; 4], a: &[u32; 4], b: &[u32; 4], _: bool, o: &[u32; 4], m: u32) -> FusedVector {
        FusedVector { value: add(a, b, o, m), product: add(c, b, o, m) }
    }
    #[inline(never)]
    pub fn add2(a: &[u32; 8], b: &[u32; 8], _: &[u32; 8], m: u32) -> Pair {
        Pair { bits: std::array::from_fn(|i| a[i] ^ b[i]), mac: m }
    }
    pub use add2 as mul2;
    #[inline(never)]
    pub fn fused2(c: &[u32; 8], a: &[u32; 8], b: &[u32; 8], _: bool, o: &[u32; 8], m: u32) -> FusedPair {
        FusedPair { value: add2(a, b, o, m), product: add2(c, b, o, m) }
    }
}

/// The same on the floor's entry points.
fn stream_run_floor(ops: &[Op], a: &Pools, b: &Pools, c: &Pools, split: bool) -> u32 {
    stream_body!(floor::add, floor::mul, floor::fused, floor::add2, floor::mul2, floor::fused2, ops, a, b, c, split)
}

/// The shares the host path meets on the stream's operands: products exact, lanes its check refuses, vectors taken whole.
fn shares(ops: &[Op], a: &Pools, b: &Pools, c: &Pools) {
    use sapphire::float_host::{add_taken, fused_taken, mul_taken};
    let (mut products, mut exact, mut refused, mut vectors, mut whole) = (0u64, 0u64, 0u64, [0u64; 3], [0u64; 3]);
    for (i, op) in ops.iter().enumerate() {
        let k0 = op.kinds[0] as usize;
        let j = i % a[k0].len();
        let (a, b, c) = (&a[k0], &b[k0], &c[k0]);
        for (n, kind) in op.kinds.iter().enumerate().take(if op.paired { 2 } else { 1 }) {
            let lane = |l: usize| (a[j][4 * n + l], b[j][4 * n + l], c[j][4 * n + l]);
            let k = *kind as usize;
            vectors[k] += 1;
            whole[k] += u64::from((0..4).all(|l| {
                let (x, y, z) = lane(l);
                match k {
                    0 => add_taken(x, y),
                    1 => mul_taken(x, y),
                    _ => fused_taken(z, x, y, false),
                }
            }));
            if k == 0 {
                continue;
            }
            for l in 0..4 {
                let (x, y, _) = lane(l);
                if (x >> 23 & 0xFF) == 0 || (y >> 23 & 0xFF) == 0 {
                    continue;
                }
                products += 1;
                let p = u64::from(x & 0x7F_FFFF | 0x80_0000) * u64::from(y & 0x7F_FFFF | 0x80_0000);
                exact += u64::from(p & if p >> 47 == 1 { 0xFF_FFFF } else { 0x7F_FFFF } == 0);
                refused += u64::from(!mul_taken(x, y));
            }
        }
    }
    let pc = |n: u64, d: u64| 100.0 * n as f64 / d.max(1) as f64;
    println!(
        "shares\tproducts of two non-zero operands: {:.2} % exact, {:.2} % refused by the host's check\tvectors taken whole: add {:.1} %, mul {:.1} %, multiply-add {:.1} %",
        pc(exact, products), pc(refused, products), pc(whole[0], vectors[0]), pc(whole[1], vectors[1]), pc(whole[2], vectors[2])
    );
}

fn mix(path: &str, rounds: usize, shuffle: bool, wide: bool) {
    mix_on(path, rounds, shuffle, if wide { 1 } else { 0 }, None)
}

/// A recorded operation of VU1: its kind (0 add, 1 mul, 2 multiply-add), its field mask, and its four lanes' operands;
/// a subtraction's second operand is recorded as written and negated here, so every operation is an add or a madd.
struct Recorded {
    kind: u8,
    mask: u32,
    a: [u32; 4],
    b: [u32; 4],
    acc: [u32; 4],
}

/// The record file: 13 words an operation (kind | subtract << 8 | mask << 16, a, b, acc), little-endian.
fn recorded(path: &str) -> Vec<Recorded> {
    let bytes = std::fs::read(path).expect("the recorded operands");
    bytes
        .chunks_exact(52)
        .map(|r| {
            let w = |i: usize| u32::from_le_bytes(r[4 * i..4 * i + 4].try_into().unwrap());
            let head = w(0);
            let sub = if head >> 8 & 1 == 1 { 0x8000_0000 } else { 0 };
            Recorded { kind: (head & 0xFF) as u8, mask: head >> 16 & 0xF, a: [w(1), w(2), w(3), w(4)], b: [w(5) ^ sub, w(6) ^ sub, w(7) ^ sub, w(8) ^ sub], acc: [w(9), w(10), w(11), w(12)] }
        })
        .collect()
}

/// What the recorded operations are, lane by lane, and what `float_rz` does with them: the lanes written, the vectors left
/// to the integer form, the product lanes `fixed` recomputes, and the adder's classes (a zero operand, the cut, no cut).
#[target_feature(enable = "avx2")]
fn census(recs: &[Recorded]) {
    let field = |x: u32| x >> 23 & 0xFF;
    let (mut ops, mut lanes_written, mut whole, mut products, mut fixed) = ([0u64; 3], [0u64; 3], [0u64; 3], 0u64, 0u64);
    let (mut sums, mut zero_sum, mut cut, mut flat, mut field255) = (0u64, 0u64, 0u64, 0u64, 0u64);
    let (mut quiet, mut short, mut fixed_short) = (0u64, 0u64, 0u64);
    for r in recs {
        let k = r.kind as usize;
        ops[k] += 1;
        lanes_written[k] += u64::from(r.mask.count_ones());
        let t = sapphire::float_rz::taken(r.kind, &r.acc, &r.a, &r.b);
        whole[k] += u64::from(t.whole);
        for l in 0..4 {
            if r.mask >> (3 - l) & 1 == 0 {
                continue;
            }
            let (a, b, acc) = (r.a[l], r.b[l], r.acc[l]);
            field255 += u64::from(field(a) == 255 || field(b) == 255 || (k == 2 && field(acc) == 255));
            if k >= 1 {
                products += 1;
                fixed += u64::from(t.fixed >> l & 1);
                fixed_short += u64::from(t.fixed >> l & 1 == 1 && a & 0x7FFF == 0);
                quiet += u64::from(b & 0xFFFF == 0);
                short += u64::from(a & 0x7FFF == 0);
            }
            let (x, y) = if k == 0 { (a, b) } else if k == 2 { (acc, float::mul(a, b).bits) } else { continue };
            sums += 1;
            let (fx, fy) = (field(x), field(y));
            if fx == 0 || fy == 0 {
                zero_sum += 1;
            } else if fx.abs_diff(fy) >= 2 {
                cut += 1;
            } else {
                flat += 1;
            }
        }
    }
    let pc = |n: u64, d: u64| 100.0 * n as f64 / d.max(1) as f64;
    let total: u64 = ops.iter().sum();
    println!(
        "recorded\t{total} operations: add {:.1} %, mul {:.1} %, multiply-add {:.1} %\tlanes written a vector: add {:.2}, mul {:.2}, multiply-add {:.2}\tvectors kept whole: add {:.3} %, mul {:.3} %, multiply-add {:.3} %",
        pc(ops[0], total),
        pc(ops[1], total),
        pc(ops[2], total),
        lanes_written[0] as f64 / ops[0].max(1) as f64,
        lanes_written[1] as f64 / ops[1].max(1) as f64,
        lanes_written[2] as f64 / ops[2].max(1) as f64,
        pc(whole[0], ops[0]),
        pc(whole[1], ops[1]),
        pc(whole[2], ops[2])
    );
    println!(
        "recorded\tproduct lanes written: {:.2} % recomputed by fixed ({:.2} % of them with a short multiplicand), {:.1} % with a quiet multiplier, {:.1} % with a short multiplicand\tsum lanes written: {:.1} % a zero operand, {:.1} % cut, {:.1} % no cut\tlanes with a field of 255: {:.4} %",
        pc(fixed, products),
        pc(fixed_short, fixed),
        pc(quiet, products),
        pc(short, products),
        pc(zero_sum, sums),
        pc(cut, sums),
        pc(flat, sums),
        pc(field255, lanes_written.iter().sum())
    );
}

/// Each kind's pool of `n` entries: the drawn operands (the same three for every kind), or each kind's own recorded
/// operations, two an entry, cycled when a kind has few; with the records themselves when they were read.
fn pools(operands: Option<&str>, n: usize, next: &mut impl FnMut() -> u64) -> (Pools, Pools, Pools, Option<Vec<Recorded>>) {
    let same_pool = |v: Vec<[u32; 8]>| -> Pools { [v.clone(), v.clone(), v] };
    match operands {
        None => {
            let mut draw = || (0..n).map(|_| [0; 8].map(|_: u32| operand(next()))).collect::<Vec<[u32; 8]>>();
            (same_pool(draw()), same_pool(draw()), same_pool(draw()), None)
        }
        Some(path) => {
            let recs = recorded(path);
            let mut p: (Pools, Pools, Pools) = Default::default();
            for k in 0..3u8 {
                let of: Vec<&Recorded> = recs.iter().filter(|r| r.kind == k).collect();
                assert!(of.len() >= 2, "no recorded operations of kind {k}");
                for j in 0..n {
                    let (x, y) = (of[(2 * j) % of.len()], of[(2 * j + 1) % of.len()]);
                    let join = |x: [u32; 4], y: [u32; 4]| [x[0], x[1], x[2], x[3], y[0], y[1], y[2], y[3]];
                    p.0[k as usize].push(join(x.a, y.a));
                    p.1[k as usize].push(join(x.b, y.b));
                    p.2[k as usize].push(join(x.acc, y.acc));
                }
            }
            (p.0, p.1, p.2, Some(recs))
        }
    }
}

/// `float_rz`'s kernel alone: each kind on `KN` eight-lane vectors held in the first-level cache, loaded, computed and
/// stored with no dispatch, the field mask full; the multiply-add also chained through ACC (its latency); ns a lane
/// operation, a multiply-add's counting twice, the median of `rounds`; MXCSR at the kernel's mode.
#[target_feature(enable = "avx2")]
fn kernels(rounds: usize, a: &Pools, b: &Pools, c: &Pools) {
    use sapphire::float_rz::raw;
    use std::arch::x86_64::*;
    const KN: usize = 128;
    const KREPEAT: usize = 16384;
    let live = raw::k(!0);
    let mut out = vec![[0u32; 8]; KN];
    let load = |v: &[u32; 8]| {
        // SAFETY: eight lanes.
        unsafe { _mm256_loadu_si256(v.as_ptr().cast()) }
    };
    for (kind, name, weight) in [(0usize, "add", 1.0), (1, "mul", 1.0), (2, "madd", 2.0)] {
        let (a, b, c) = (&a[kind][..KN], &b[kind][..KN], &c[kind][..KN]);
        let mut left = 0u32;
        let ns = median((0..rounds).map(|_| {
            let t = Instant::now();
            for _ in 0..KREPEAT {
                for i in 0..KN {
                    let (x, y, z) = (load(&a[i]), load(&b[i]), load(&c[i]));
                    let (r, t) = match kind {
                        0 => raw::add(x, y),
                        1 => {
                            let (p, fix, t) = raw::mul(x, y);
                            (raw::fixed(p, x, y, fix), t)
                        }
                        _ => {
                            let (r, _, t) = raw::fused(z, x, y, false, live);
                            (r, t)
                        }
                    };
                    if raw::any(t) {
                        left += 1;
                    }
                    // SAFETY: `out` holds KN entries.
                    unsafe { _mm256_storeu_si256(out[i].as_mut_ptr().cast(), r) };
                }
                black_box(&mut out);
            }
            t.elapsed().as_secs_f64() * 1e9 / (KREPEAT * KN * 8) as f64 / weight
        }).collect());
        println!("kernel\t{name}\t{ns:.3} ns a lane operation\t{} vectors left to the integer form a round", left / rounds as u32);
    }
    // The chain: each multiply-add's ACC is the one before, as a VU microprogram's MADDA chain holds it.
    let (a, b) = (&a[2][..KN], &b[2][..KN]);
    let ns = median((0..rounds).map(|_| {
        let mut acc = raw::k(0x3F80_0000);
        let t = Instant::now();
        for _ in 0..KREPEAT {
            for i in 0..KN {
                let (r, _, _) = raw::fused(acc, load(&a[i]), load(&b[i]), i & 1 == 1, live);
                acc = r;
            }
        }
        black_box(acc);
        t.elapsed().as_secs_f64() * 1e9 / (KREPEAT * KN * 8) as f64 / 2.0
    }).collect());
    println!("kernel\tmadd chained through ACC\t{ns:.3} ns a lane operation");
}

/// `form`: 0 `float_lanes`' AVX2 form, 1 its AVX-512VL form, 2 `float_host`'s `{rz-sae}` form, 3 `float_host`'s AVX2 form,
/// 4 the stream alone, 5 `float_rz` through its entry points, 6 `float_rz`'s kernel in registers. `operands`: a record
/// file (`recorded`) whose operations fill each kind's pool instead of the drawn operands.
fn mix_on(path: &str, rounds: usize, shuffle: bool, form: u8, operands: Option<&str>) {
    let text = std::fs::read_to_string(path).expect("the count");
    let mut groups: Vec<(Op, u64)> = Vec::new();
    for line in text.lines() {
        let v: Vec<&str> = line.split_whitespace().collect();
        let n = |i: usize| v[i].parse::<u32>().expect("a number");
        match v.first() {
            Some(&"pair") => groups.push((Op { paired: true, kinds: [n(1) as u8, n(3) as u8], mask: lane_mask(n(2)) | lane_mask(n(4)) << 16 }, u64::from(n(5)))),
            Some(&"single") => groups.push((Op { paired: false, kinds: [n(1) as u8, 0], mask: lane_mask(n(2)) }, u64::from(n(3)))),
            _ => {}
        }
    }
    let total: u64 = groups.iter().map(|g| g.1).sum();
    let want = 1u64 << 20;
    let mut ops: Vec<Op> = groups.iter().flat_map(|&(op, n)| std::iter::repeat_n(op, ((n as f64 * want as f64 / total as f64).round() as usize).max(1))).collect();
    let mut next = stream(0x0000_5EED_0000_0006);
    if shuffle {
        for i in (1..ops.len()).rev() {
            ops.swap(i, next() as usize % (i + 1));
        }
    }
    let lanes_of = |op: &Op| -> f64 {
        let w = |m: u32, k: u8| f64::from((m & 0xF).count_ones()) * if k == 2 { 2.0 } else { 1.0 };
        w(op.mask, op.kinds[0]) + if op.paired { w(op.mask >> 16, op.kinds[1]) } else { 0.0 }
    };
    let lane_ops: f64 = ops.iter().map(lanes_of).sum();
    let computed: f64 = ops.iter().map(|o| 4.0 * (if o.kinds[0] == 2 { 2.0 } else { 1.0 } + if o.paired { if o.kinds[1] == 2 { 2.0 } else { 1.0 } } else { 0.0 })).sum();
    let same = ops.iter().filter(|o| o.paired && o.kinds[0] == o.kinds[1]).count();
    let pairs = ops.iter().filter(|o| o.paired).count();
    let (a, b, c, recs) = pools(operands, N, &mut next);
    if let Some(recs) = &recs
        && (form == 5 || form == 6)
    {
        // SAFETY: AVX2 asserted by main; the mode is restored at once.
        unsafe {
            let old = sapphire::float_rz::enter();
            census(recs);
            sapphire::float_rz::leave(old);
        }
    }
    if form == 2 || form == 3 {
        shares(&ops, &a, &b, &c);
    }
    let mut out = vec![[0u32; 8]; N];
    // SAFETY: the forms on `float_rz` need MXCSR at its mode; nothing else here computes on floats until `leave`.
    let old = if form >= 5 { Some(unsafe { sapphire::float_rz::enter() }) } else { None };
    for split in [false, true] {
        if form == 6 && split {
            continue;
        }
        let mut left = 0;
        let ns = median((0..rounds).map(|_| {
            let t = Instant::now();
            // SAFETY: AVX2 was asserted, and AVX-512 for the forms that need it.
            black_box(unsafe {
                match form {
                    1 => stream_run512(black_box(&ops), &a, &b, &c, split),
                    2 => stream_run_host(black_box(&ops), &a, &b, &c, split),
                    3 => stream_run_host2(black_box(&ops), &a, &b, &c, split),
                    4 => stream_run_floor(black_box(&ops), &a, &b, &c, split),
                    5 => stream_run_rz(black_box(&ops), &a, &b, &c, split),
                    6 => {
                        left = stream_regs(black_box(&ops), &a, &b, &c, &mut out);
                        out.iter().fold(left, |h, v| h ^ v[0] ^ v[7])
                    }
                    _ => stream_run(black_box(&ops), &a, &b, &c, split),
                }
            });
            t.elapsed().as_secs_f64() * 1e9
        }).collect());
        println!(
            "mix\t{} entries, {pairs} of them pairs, {same} of one kind\t{:.2} lane operations written an entry\t{}{}{}\t{:.3} ns a lane operation written\t{:.3} a lane computed{}",
            ops.len(),
            lane_ops / ops.len() as f64,
            if split { "every operation in four lanes" } else { "pairs of one kind in eight lanes" },
            if shuffle { ", shuffled" } else { ["", ", AVX-512VL", ", host floats {rz-sae}", ", host floats AVX2", ", the stream alone", ", host floats towards zero", ", towards zero in registers"][form as usize] },
            if operands.is_some() { ", recorded operands" } else { "" },
            ns / lane_ops,
            ns / computed,
            if form == 6 { format!("\t{left} vectors left to the integer form") } else { String::new() }
        );
    }
    if let Some(old) = old {
        // SAFETY: restores what `enter` saved.
        unsafe { sapphire::float_rz::leave(old) };
    }
}

fn main() {
    assert!(lanes::available(), "the kernel needs AVX2");
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("kernels") {
        let rounds = args.get(1).map_or(7, |a| a.parse().expect("rounds"));
        let mut next = stream(0x0000_5EED_0000_0006);
        let (a, b, c, _) = pools(args.get(2).map(String::as_str), N, &mut next);
        // SAFETY: AVX2 was asserted; the mode is restored after.
        unsafe {
            let old = sapphire::float_rz::enter();
            kernels(rounds, &a, &b, &c);
            sapphire::float_rz::leave(old);
        }
        return;
    }
    if args.first().map(String::as_str) == Some("mix") {
        let wide = args.get(3).is_some_and(|a| a == "wide");
        assert!(!wide || lanes::available512(), "the AVX-512VL form needs AVX-512VL and CD");
        let (file, rounds) = (args.get(1).expect("a count's file"), args.get(2).map_or(7, |a| a.parse().expect("rounds")));
        let operands = args.get(4).map(String::as_str);
        match args.get(3).map(String::as_str) {
            Some("host") => {
                assert!(sapphire::float_host::available512(), "the host form needs AVX-512F and DQ");
                mix_on(file, rounds, false, 2, operands);
            }
            Some("host2") => mix_on(file, rounds, false, 3, operands),
            Some("floor") => mix_on(file, rounds, false, 4, operands),
            Some("rz") => mix_on(file, rounds, false, 5, operands),
            Some("rzreg") => mix_on(file, rounds, false, 6, operands),
            _ => mix(file, rounds, args.get(3).is_some_and(|a| a == "shuffle"), wide),
        }
        return;
    }
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
