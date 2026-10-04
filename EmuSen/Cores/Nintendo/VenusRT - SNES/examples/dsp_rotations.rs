//! Grades VenusRT_Native.md §40.1's Rotate (0Ch) and Polar (1Ch) variants against the image with the replacement's
//! sine: `dsp_rotations <chip> [cases]`. Since that sine is not the chip's, a member is judged by the share of cases
//! within §40.3's Triangle bound (12) and its largest difference; counts only. DSP_ROTATIONS_SMALL keeps the
//! coordinates under 2^11; DSP_ROTATIONS_POLAR_WIDE grades §40.3's widened Polar family instead.
use venusrt::chips::dsphle::sine;
use venusrt::chips::dsporacle::*;

/// Polar compounds three sines, so its bound is wider than Triangle's.
const POLAR_BOUND: i64 = 64;

fn scale(v: i64, saturate: bool) -> i64 {
    let r = v >> 15;
    if saturate { r.clamp(-0x8000, 0x7FFF) } else { r as i16 as i64 }
}

/// One 2D rotation of (a, b) by angle `t`: counter-clockwise, or its transpose; each product scaled, or the sum once.
fn turn(a: i64, b: i64, t: u16, ccw: bool, each: bool, sat: bool) -> (i64, i64) {
    let (s, c) = (sine(t), sine(t.wrapping_add(0x4000)));
    let s = if ccw { s } else { -s };
    if each {
        let f = |v: i64| v >> 15;
        (scale((f(a * c) - f(b * s)) << 15, sat), scale((f(a * s) + f(b * c)) << 15, sat))
    } else {
        (scale(a * c - b * s, sat), scale(a * s + b * c, sat))
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let stem = &a[1];
    let cases: u64 = a.get(2).and_then(|c| c.parse().ok()).unwrap_or(65536);
    let image = firmware(stem).unwrap_or_else(|| std::process::exit(1));
    let mut chip = idle_chip(stem, &image, &mut Host::steady(), 0).unwrap();
    let mut p = Pcg::new(0x0C1C);
    let w = |v: u16| v as i16 as i64;
    let mut rot = vec![(0u64, 0i64); 8];
    let mut pol = vec![(0u64, 0i64); 16];
    for _ in 0..cases {
        let small = std::env::var("DSP_ROTATIONS_SMALL").is_ok();
        let set: Vec<u16> = (0..6).map(|k| if small && k >= 3 { (p.word() as i16 >> 4) as u16 } else { p.word() }).collect();
        let got: Vec<i64> = transact(&mut chip, &mut Host::steady(), 0x0C, &set[..3]).outputs().iter().map(|&v| w(v)).collect();
        for m in 0..8 {
            let (x, y) = turn(w(set[1]), w(set[2]), set[0], m & 1 == 0, m & 2 != 0, m & 4 != 0);
            let e = (got[0] - x).abs().max((got[1] - y).abs());
            rot[m].0 += (e <= 12) as u64;
            rot[m].1 = rot[m].1.max(e);
        }
        let got: Vec<i64> = transact(&mut chip, &mut Host::steady(), 0x1C, &set).outputs().iter().map(|&v| w(v)).collect();
        for m in 0..16 {
            let (transpose, each, sat, reverse) = (m & 1 != 0, m & 2 != 0, m & 4 != 0, m & 8 != 0);
            // SNESdev's matrices as written, the row vector on the left; I3 about Y, I2 about X, I1 about Z.
            let mat = |axis: u8, t: u16| -> [[i64; 3]; 3] {
                let (s, c) = (sine(t), sine(t.wrapping_add(0x4000)));
                let one = 0x8000i64;
                let m = match axis {
                    b'y' => [[c, 0, s], [0, one, 0], [-s, 0, c]],
                    b'x' => [[one, 0, 0], [0, c, -s], [0, s, c]],
                    _ => [[c, -s, 0], [s, c, 0], [0, 0, one]],
                };
                if transpose { [[m[0][0], m[1][0], m[2][0]], [m[0][1], m[1][1], m[2][1]], [m[0][2], m[1][2], m[2][2]]] } else { m }
            };
            let order = if reverse { [(b'z', set[0]), (b'x', set[1]), (b'y', set[2])] } else { [(b'y', set[2]), (b'x', set[1]), (b'z', set[0])] };
            let mut v = [w(set[3]), w(set[4]), w(set[5])];
            for (axis, t) in order {
                let mm = mat(axis, t);
                let mut o = [0i64; 3];
                for j in 0..3 {
                    let acc: i64 = if each { (0..3).map(|k| (v[k] * mm[k][j]) >> 15).sum::<i64>() << 15 } else { (0..3).map(|k| v[k] * mm[k][j]).sum() };
                    o[j] = scale(acc, sat);
                }
                v = o;
            }
            let e = (got[0] - v[0]).abs().max((got[1] - v[1]).abs()).max((got[2] - v[2]).abs());
            pol[m].0 += (e <= POLAR_BOUND) as u64;
            pol[m].1 = pol[m].1.max(e);
        }
    }
    if std::env::var("DSP_ROTATIONS_POLAR_WIDE").is_ok() {
        polar_wide(&mut chip, cases);
        return;
    }
    println!("{stem} 0Ch over {cases} cases (ccw, each-product, saturate): within 12 / largest");
    for (m, (n, e)) in rot.iter().enumerate() {
        println!("  {} {} {}: {n} / {e}", m & 1 == 0, m & 2 != 0, m & 4 != 0);
    }
    println!("{stem} 1Ch (transposed, each-product, saturate, reversed order): within 12 / largest");
    for (m, (n, e)) in pol.iter().enumerate() {
        println!("  {} {} {} {}: {n} / {e}", m & 1 != 0, m & 2 != 0, m & 4 != 0, m & 8 != 0);
    }
}

/// §40.3's widening: every order of the three axis rotations, every assignment of I1-I3 to the axes, each matrix as
/// written or transposed; the row vector on the left, the sum scaled once, wrap. Small coordinates, then full ones.
fn polar_wide(chip: &mut Lle, cases: u64) {
    let w = |v: u16| v as i16 as i64;
    let perms: [[usize; 3]; 6] = [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]];
    let axes = [b'z', b'x', b'y'];
    for small in [true, false] {
        let mut p = Pcg::new(0x1C1C + small as u64);
        let mut hits = vec![(0u64, 0i64); 72];
        for _ in 0..cases {
            let set: Vec<u16> = (0..6).map(|k| if small && k >= 3 { (p.word() as i16 >> 4) as u16 } else { p.word() }).collect();
            let got: Vec<i64> = transact(chip, &mut Host::steady(), 0x1C, &set).outputs().iter().map(|&v| w(v)).collect();
            for m in 0..72 {
                let (order, assign, transpose) = (perms[m / 12], perms[(m / 2) % 6], m % 2 == 1);
                let mut v = [w(set[3]), w(set[4]), w(set[5])];
                for &step in &order {
                    let axis = axes[step];
                    let t = set[assign[step]];
                    let (s, c) = (sine(t), sine(t.wrapping_add(0x4000)));
                    let one = 0x8000i64;
                    let mut mm = match axis {
                        b'y' => [[c, 0, s], [0, one, 0], [-s, 0, c]],
                        b'x' => [[one, 0, 0], [0, c, -s], [0, s, c]],
                        _ => [[c, -s, 0], [s, c, 0], [0, 0, one]],
                    };
                    if transpose {
                        mm = [[mm[0][0], mm[1][0], mm[2][0]], [mm[0][1], mm[1][1], mm[2][1]], [mm[0][2], mm[1][2], mm[2][2]]];
                    }
                    let mut o = [0i64; 3];
                    for j in 0..3 {
                        o[j] = scale((0..3).map(|k| v[k] * mm[k][j]).sum(), false);
                    }
                    v = o;
                }
                let e = (got[0] - v[0]).abs().max((got[1] - v[1]).abs()).max((got[2] - v[2]).abs());
                hits[m].0 += (e <= POLAR_BOUND) as u64;
                hits[m].1 = hits[m].1.max(e);
            }
        }
        let mut order: Vec<usize> = (0..72).collect();
        order.sort_by(|&a, &b| hits[b].0.cmp(&hits[a].0));
        println!("1Ch widened, {} coordinates, {cases} cases: within {POLAR_BOUND} / largest", if small { "small" } else { "full" });
        for &m in order.iter().take(4) {
            println!("  order {:?} angles {:?} transposed {}: {} / {}", perms[m / 12].map(|k| axes[k] as char), perms[(m / 2) % 6], m % 2 == 1, hits[m].0, hits[m].1);
        }
    }
}
