//! Grades VenusRT_Native.md §50.2's families for the DSP-1's Target (0Eh) and Gyrate (14h) against the image:
//! `dsp_target_gyrate <chip> [cases] [trace]`, seeded cases through the ports and a trace's recorded transactions.
//! Counts only. DSP_TG_PORTS=1 grades the replacement itself through the ports, steady and jittered.
use venusrt::chips::dsphle::{Program, reciprocal, sine};
use venusrt::chips::dsporacle::*;

fn mul(a: i64, b: i64) -> i64 {
    (a * b) >> 15
}

/// num/den through the Inverse routine or exactly, floored or half up; saturated to 16 bits unless `wide`.
fn quot(routine: bool, half: bool, wide: bool, num: i128, den: i128) -> i64 {
    let lim = if wide { (i64::MIN / 4, i64::MAX / 4) } else { (-0x8000, 0x7FFF) };
    if den == 0 {
        return if num >= 0 { lim.1 } else { lim.0 };
    }
    let q = if routine {
        let (mut d, mut extra) = (den, 0u32);
        while d > i64::MAX as i128 / 4 || d < i64::MIN as i128 / 4 {
            d >>= 1;
            extra += 1;
        }
        let (r, shift) = reciprocal(d as i64).unwrap();
        let sh = shift + extra;
        let p = num * r as i128;
        if half { (p + (1i128 << (sh - 1))) >> sh } else { p >> sh }
    } else {
        let (n, d) = if den < 0 { (-num, -den) } else { (num, den) };
        if half { (2 * n + d).div_euclid(2 * d) } else { n.div_euclid(d) }
    };
    q.clamp(lim.0 as i128, lim.1 as i128) as i64
}

/// Target by member m (T1 wide, T2 combined, T3 half up), on §49.5's Parameter and Raster.
fn target(m: u32, p: &[i64], h: i64, v: i64) -> [i64; 2] {
    let (wide, combined, half) = (m & 1 != 0, m & 2 != 0, m & 4 != 0);
    let (a, z) = (p[5] as u16, p[6] as u16);
    let (sa, ca, sz, cz) = (sine(a), sine(a.wrapping_add(0x4000)), sine(z), sine(z.wrapping_add(0x4000)));
    let (lfe, les) = (p[3], p[4]);
    let ez = p[2] + mul(lfe, cz);
    let ex = p[0] - ((lfe * sz * sa) >> 30);
    let ey = p[1] + ((lfe * sz * ca) >> 30);
    let t = quot(true, true, false, ez as i128 * sz as i128, cz as i128);
    let (cx, cy) = ((ex + mul(t, sa)).clamp(-0x8000, 0x7FFF), (ey - mul(t, ca)).clamp(-0x8000, 0x7FFF));
    let n = mul(les, cz) + mul(v, sz);
    let k = quot(true, true, wide, 256 * ez as i128, n as i128);
    let kk = quot(true, true, wide, (k as i128) << 15, cz as i128);
    let div = |x: i128| -> i64 { (if half { (x + 128) >> 8 } else { x >> 8 }) as i64 };
    let (dx, dy) = if combined {
        let s = |x: i128| -> i128 { x >> 15 };
        (div(s(h as i128 * k as i128 * ca as i128) - s(v as i128 * kk as i128 * sa as i128)), div(s(v as i128 * kk as i128 * ca as i128) - s(h as i128 * k as i128 * sa as i128)))
    } else {
        let (aa, bb, cc, dd) = (mul(k, ca), mul(-kk, sa), mul(k, sa), mul(kk, ca));
        (div(h as i128 * aa as i128 + v as i128 * bb as i128), div(v as i128 * dd as i128 - h as i128 * cc as i128))
    };
    [(cx + dx).clamp(-0x8000, 0x7FFF), (cy + dy).clamp(-0x8000, 0x7FFF)]
}

/// Gyrate by member m (G1 routine, G2 terms floored, G3 each term divided, G4 half up).
fn gyrate(m: u32, i: &[i64]) -> [i64; 3] {
    let (routine, terms, each, half) = (m & 1 != 0, m & 2 != 0, m & 4 != 0, m & 8 != 0);
    let (az, ax, ay, u, f, l) = (i[0], i[1], i[2], i[3], i[4], i[5]);
    let (sx, cx) = (sine(ax as u16), sine((ax as u16).wrapping_add(0x4000)));
    let (sy, cy) = (sine(ay as u16), sine((ay as u16).wrapping_add(0x4000)));
    // Q15 products summed: each floored, or the sum scaled once.
    let sum2 = |a: i64, b: i64, c: i64, d: i64| if terms { mul(a, b) + mul(c, d) } else { (a * b + c * d) >> 15 };
    let sat = |v: i64| v.clamp(-0x8000, 0x7FFF);
    let q = |num: i128, den: i128| quot(routine, half, false, num, den);
    let daz = if each { sat(q((u * cy) as i128, cx as i128) - q((f * sy) as i128, cx as i128)) } else { q((u * cy - f * sy) as i128, cx as i128) };
    let dax = sat(sum2(u, sy, f, cy));
    let w = sum2(u, cy, f, sy);
    // tan Ax in Q15 at full width: past 45° it exceeds a word.
    let tan = quot(routine, half, true, (sx as i128) << 15, cx as i128);
    let day = sat(l - ((w as i128 * tan as i128) >> 15) as i64);
    let wrap = |a: i64, d: i64| (a + d) as u16 as i16 as i64;
    [wrap(az, sat(daz)), wrap(ax, dax), wrap(ay, day)]
}

fn close(image: i64, model: i64) -> bool {
    let d = (image - model).abs();
    d <= 2.max(image.abs() / 128)
}

/// An angle's result: the distance the long way round counts as its wrapped difference.
fn close_angle(image: i64, model: i64) -> bool {
    ((image - model) as i16 as i64).abs() <= 2
}

fn signed(v: u16) -> i64 {
    v as i16 as i64
}

fn trace(path: &str) -> Vec<(u8, Vec<i64>, Vec<i64>)> {
    let text = std::fs::read_to_string(path).expect("a trace");
    let list = |s: &str| -> Vec<i64> { s.split(',').filter_map(|w| u16::from_str_radix(w.trim(), 16).ok()).map(signed).collect() };
    text.lines()
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            words.next()?;
            let command = u8::from_str_radix(words.next()?, 16).ok()?;
            let i = line.find("in [")?;
            let o = line.find("out [")?;
            Some((command, list(&line[i + 4..line[i..].find(']')? + i]), list(&line[o + 5..line[o..].find(']')? + o])))
        })
        .collect()
}

#[derive(Clone, Copy, Default)]
struct Tally {
    close: u64,
    results: u64,
    exact: u64,
    worst: i64,
}

fn seeded_params(p: &mut Pcg) -> Vec<i64> {
    let mut w = |lo: i64, hi: i64| lo + p.within(0, (hi - lo) as u32) as i64;
    vec![w(-4096, 4096), w(-4096, 4096), w(0, 1000), w(0, 1024), w(64, 1024), w(0, 0xFFFF), w(0x0800, 0x3800)]
}

fn seeded_gyrate(p: &mut Pcg) -> Vec<i64> {
    let mut w = |lo: i64, hi: i64| lo + p.within(0, (hi - lo) as u32) as i64;
    let ax = w(-0x3555, 0x3555);
    vec![w(-0x8000, 0x7FFF), ax, w(-0x8000, 0x7FFF), w(-1024, 1024), w(-1024, 1024), w(-1024, 1024)]
}

fn ports(stem: &str, cases: u64) {
    let image = firmware(stem).unwrap_or_else(|| std::process::exit(1));
    let mut lle = idle_chip(stem, &image, &mut Host::steady(), 0).unwrap();
    let mut hle = Hle::new(Program::for_stem(stem).unwrap());
    power_on(&mut hle, &Host::steady()).unwrap();
    let mut p = Pcg::new(0x0E14_0000);
    let mut t = [[0i64; 6]; 2];
    for k in 0..cases {
        let host = || if k % 2 == 1 { Host::jittered(k) } else { Host::steady() };
        let params: Vec<u16> = seeded_params(&mut p).iter().map(|&v| v as u16).collect();
        let got = transact(&mut lle, &mut Host::steady(), 0x02, &params).outputs();
        transact(&mut hle, &mut Host::steady(), 0x02, &params);
        let vva = got[1] as i16 as i64;
        let hv = [(p.within(0, 256) as i64 - 128) as u16, (vva + 2).clamp(-112, 112).max(-112) as u16];
        let hv = [hv[0], ((hv[1] as i16 as i64) + p.within(0, (112 - hv[1] as i16 as i64).max(0) as u32) as i64) as u16];
        let g: Vec<u16> = seeded_gyrate(&mut p).iter().map(|&v| v as u16).collect();
        for (c, command, inputs, angles) in [(0, 0x0Eu8, hv.to_vec(), false), (1, 0x14, g, true)] {
            let (x, y) = (transact(&mut lle, &mut host(), command, &inputs), transact(&mut hle, &mut host(), command, &inputs));
            let d = compare(&x, &y);
            let ok = x.outputs().iter().zip(y.outputs()).all(|(&a, b)| if angles { close_angle(signed(a), signed(b)) } else { close(signed(a), signed(b)) });
            let e = x.outputs().iter().zip(y.outputs()).map(|(&a, b)| if angles { ((a as i64 - b as i64) as i16 as i64).abs() } else { (signed(a) - signed(b)).abs() }).max().unwrap_or(0);
            t[c][0] += 1;
            t[c][1] += ok as i64;
            t[c][2] += (x.outputs() == y.outputs()) as i64;
            t[c][3] += (d.shape || d.status) as i64;
            t[c][4] += d.latency as i64;
            t[c][5] = t[c][5].max(e);
        }
    }
    for (c, name) in ["0Eh Target", "14h Gyrate"].iter().enumerate() {
        let r = t[c];
        println!("{stem} {name}: {} cases, all close {}, all exact {}; transfers or SR differ {}, latency {}; largest {}", r[0], r[1], r[2], r[3], r[4], r[5]);
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let stem = &a[1];
    let cases: u64 = a.get(2).and_then(|c| c.parse().ok()).unwrap_or(1 << 16);
    if std::env::var_os("DSP_TG_PORTS").is_some() {
        ports(stem, cases);
        return;
    }
    // Per set: Target's 8 members and Gyrate's 16.
    let mut sets: Vec<(String, Vec<Tally>, Vec<Tally>)> = Vec::new();
    let add = |t: &mut Tally, image: &[i64], model: &[i64], angles: bool| {
        for (&x, &y) in image.iter().zip(model) {
            t.results += 1;
            t.close += if angles { close_angle(x, y) } else { close(x, y) } as u64;
            t.exact += (x == y) as u64;
            t.worst = t.worst.max(if angles { ((x - y) as i16 as i64).abs() } else { (x - y).abs() });
        }
    };
    if cases > 0 {
        let image = firmware(stem).unwrap_or_else(|| std::process::exit(1));
        let mut chip = idle_chip(stem, &image, &mut Host::steady(), 0).unwrap();
        let mut p = Pcg::new(0x0E14_0000);
        let (mut tt, mut gt) = (vec![Tally::default(); 8], vec![Tally::default(); 16]);
        for _ in 0..cases {
            let params = seeded_params(&mut p);
            let words: Vec<u16> = params.iter().map(|&v| v as u16).collect();
            let got = transact(&mut chip, &mut Host::steady(), 0x02, &words).outputs();
            let vva = got[1] as i16 as i64;
            let h = p.within(0, 256) as i64 - 128;
            let lo = (vva + 2).clamp(-112, 112);
            let v = lo + p.within(0, (112 - lo) as u32) as i64;
            let image: Vec<i64> = transact(&mut chip, &mut Host::steady(), 0x0E, &[h as u16, v as u16]).outputs().iter().map(|&w| signed(w)).collect();
            for (m, t) in tt.iter_mut().enumerate() {
                add(t, &image, &target(m as u32, &params, h, v), false);
            }
            let g = seeded_gyrate(&mut p);
            let gw: Vec<u16> = g.iter().map(|&v| v as u16).collect();
            let image: Vec<i64> = transact(&mut chip, &mut Host::steady(), 0x14, &gw).outputs().iter().map(|&w| signed(w)).collect();
            for (m, t) in gt.iter_mut().enumerate() {
                add(t, &image, &gyrate(m as u32, &g), true);
            }
            if std::env::var_os("DSP_TG_SHOW").is_some() {
                let model = gyrate(1, &g);
                if image.iter().zip(&model).any(|(&x, &y)| !close_angle(x, y)) {
                    eprintln!("G {g:?} image {image:?} model {model:?}");
                }
            }
        }
        sets.push(("seeded".into(), tt, gt));
    }
    if let Some(path) = a.get(3) {
        let (mut tt, mut gt) = (vec![Tally::default(); 8], vec![Tally::default(); 16]);
        let mut params: Option<Vec<i64>> = None;
        for (command, ins, outs) in trace(path) {
            match command {
                0x02 if ins.len() == 7 && outs.len() == 4 => params = Some(ins),
                0x0E if ins.len() == 2 && outs.len() == 2 => {
                    if let Some(p) = &params {
                        let model = target(3, p, ins[0], ins[1]);
                        if std::env::var_os("DSP_TG_SHOW").is_some() && outs.iter().zip(&model).any(|(&x, &y)| !close(x, y)) {
                            eprintln!("T {p:?} {ins:?} image {outs:?} model {model:?}");
                        }
                        for (m, t) in tt.iter_mut().enumerate() {
                            add(t, &outs, &target(m as u32, p, ins[0], ins[1]), false);
                        }
                    }
                }
                0x14 if ins.len() == 6 && outs.len() == 3 => {
                    for (m, t) in gt.iter_mut().enumerate() {
                        add(t, &outs, &gyrate(m as u32, &ins), true);
                    }
                }
                _ => {}
            }
        }
        sets.push(("traced".into(), tt, gt));
    }
    for (which, n) in [("Target", 8usize), ("Gyrate", 16)] {
        let mut order: Vec<usize> = (0..n).collect();
        let pick = |s: &(String, Vec<Tally>, Vec<Tally>), m: usize| if which == "Target" { s.1[m] } else { s.2[m] };
        let total = |m: usize| sets.iter().map(|s| pick(s, m).close).sum::<u64>();
        let worst = |m: usize| sets.iter().map(|s| pick(s, m).worst).max().unwrap_or(0);
        order.sort_by(|&x, &y| total(y).cmp(&total(x)).then(worst(x).cmp(&worst(y))));
        println!("{stem} {which}:");
        for &m in order.iter().take(n.min(6)) {
            let line: Vec<String> = sets.iter().map(|s| { let t = pick(s, m); format!("{} {}/{} ({} exact, largest {})", s.0, t.close, t.results, t.exact, t.worst) }).collect();
            println!("  member {m:04b}: {}", line.join("; "));
        }
    }
}
