//! Grades VenusRT_Native.md §49.2's family for the DSP-1's Parameter (02h), Raster (0Ah) and Project (06h), with
//! §49.4's widening of Project, against the image: `dsp_projection <chip> [cases] [trace...]`. Seeded cases run on the
//! image through the ports; a trace's recorded transactions (dsp_trace) are the traced set. Counts only: per member,
//! the close results of each command and the largest difference. DSP_PROJ_SHOW=<pr>,<pj> prints those members'
//! results that are not close, to the terminal, for characterisation.
use venusrt::chips::dsphle::{reciprocal, sine};
use venusrt::chips::dsporacle::*;

/// One member: S1 the eye's height whole for Parameter and Raster, S2 the Inverse routine, S3 half up, P1 the offset
/// floored, R1 the denominator (0 the sum of floors, 1 the floor of the sum, 2 unquantised), J1 the view's elements
/// floored, J2 the terms floored, J3 Project's eye height and horizontal position with fractions, J4 w whole.
#[derive(Clone, Copy, Debug, Default)]
struct Member {
    whole: bool,
    routine: bool,
    half: bool,
    p_floored: bool,
    den: u8,
    j_floored: bool,
    j_terms: bool,
    z_frac: bool,
    h_frac: bool,
    w_whole: bool,
}

impl Member {
    /// Parameter's and Raster's 48.
    fn pr(k: u32) -> Member {
        Member { whole: k & 1 != 0, routine: k & 2 != 0, half: k & 4 != 0, p_floored: k & 8 != 0, den: (k / 16) as u8, ..Default::default() }
    }

    /// Project's 256: the shared S2, S3 and P1 with its own 32.
    fn pj(k: u32) -> Member {
        Member {
            routine: k & 1 != 0,
            half: k & 2 != 0,
            p_floored: k & 4 != 0,
            j_floored: k & 8 != 0,
            j_terms: k & 16 != 0,
            z_frac: k & 32 != 0,
            h_frac: k & 64 != 0,
            w_whole: k & 128 != 0,
            ..Default::default()
        }
    }
}

fn mul(a: i64, b: i64) -> i64 {
    (a * b) >> 15
}

fn sat(v: i64) -> i64 {
    v.clamp(-0x8000, 0x7FFF)
}

/// num / den by S2 and S3, saturated to 16 bits; a zero denominator saturates by the numerator's sign.
fn div(m: Member, num: i128, den: i128) -> i64 {
    if den == 0 {
        return if num >= 0 { 0x7FFF } else { -0x8000 };
    }
    let q = if m.routine {
        let (mut d, mut extra) = (den, 0u32);
        while d > i64::MAX as i128 / 4 || d < i64::MIN as i128 / 4 {
            d >>= 1;
            extra += 1;
        }
        let (r, shift) = reciprocal(d as i64).unwrap_or((0x7FFF, 0));
        let p = num * r as i128;
        let sh = shift + extra;
        if m.half { (p + (1i128 << (sh - 1))) >> sh } else { p >> sh }
    } else {
        let (n, d) = if den < 0 { (-num, -den) } else { (num, den) };
        if m.half { (2 * n + d).div_euclid(2 * d) } else { n.div_euclid(d) }
    };
    q.clamp(-0x8000, 0x7FFF) as i64
}

struct Cam {
    sa: i64,
    ca: i64,
    sz: i64,
    cz: i64,
    les: i64,
    lfe: i64,
    f: [i64; 3],
    /// The eye: horizontal whole by P1, height whole or Q15 by S1.
    ex: i64,
    ey: i64,
    ez: i64,
    out: [i64; 4],
}

fn parameter(m: Member, i: &[i64]) -> Cam {
    let (fx, fy, fz, lfe, les) = (i[0], i[1], i[2], i[3], i[4]);
    let (a, z) = (i[5] as u16, i[6] as u16);
    let (sa, ca, sz, cz) = (sine(a), sine(a.wrapping_add(0x4000)), sine(z), sine(z.wrapping_add(0x4000)));
    let ez = if m.whole { fz + mul(lfe, cz) } else { (fz << 15) + lfe * cz };
    let (ex, ey) = if m.p_floored {
        let h = mul(lfe, sz);
        (fx - mul(h, sa), fy + mul(h, ca))
    } else {
        (fx - ((lfe * sz * sa) >> 30), fy + ((lfe * sz * ca) >> 30))
    };
    let scale = if m.whole { 1 } else { 1 << 15 };
    let t = div(m, ez as i128 * sz as i128, cz as i128 * scale);
    let cx = sat(ex + mul(t, sa));
    let cy = sat(ey - mul(t, ca));
    let vva = div(m, -(les * cz) as i128, sz as i128);
    Cam { sa, ca, sz, cz, les, lfe, f: [fx, fy, fz], ex, ey, ez, out: [0, vva, cx, cy] }
}

fn raster(m: Member, c: &Cam, v: i64) -> [i64; 4] {
    let (n, nscale) = match m.den {
        0 => (mul(c.les, c.cz) + mul(v, c.sz), 1i128),
        1 => ((c.les * c.cz + v * c.sz) >> 15, 1),
        _ => (c.les * c.cz + v * c.sz, 1 << 15),
    };
    let escale: i128 = if m.whole { 1 } else { 1 << 15 };
    let k = div(m, 256 * c.ez as i128 * nscale, n as i128 * escale);
    let kk = div(m, (k as i128) << 15, c.cz as i128);
    [mul(k, c.ca), mul(-kk, c.sa), mul(k, c.sa), mul(kk, c.ca)]
}

/// Project, with `c` built by Parameter under a member sharing `m`'s S2, S3 and P1.
fn project(m: Member, c: &Cam, p: &[i64]) -> [i64; 3] {
    let (sa, ca, sz, cz) = (c.sa, c.ca, c.sz, c.cz);
    // The view's rows in Q30: elements as floored Q15 products (J1) or exact.
    let el = |a: i64, b: i64| if m.j_floored { mul(a, b) << 15 } else { a * b };
    let r = [ca << 15, sa << 15, 0];
    let u = [el(-sa, cz), el(ca, cz), -sz << 15];
    let f = [el(sa, sz), el(-ca, sz), -cz << 15];
    let frac = m.z_frac || m.h_frac;
    // The eye in Q15 (J3).
    let (ex, ey) = if m.h_frac {
        ((c.f[0] << 15) - ((c.lfe * f[0]) >> 15), (c.f[1] << 15) - ((c.lfe * f[1]) >> 15))
    } else {
        (c.ex << 15, c.ey << 15)
    };
    let ez = if m.z_frac { (c.f[2] << 15) + c.lfe * cz } else { (c.f[2] + mul(c.lfe, cz)) << 15 };
    let d = [(p[0] << 15) - ex, (p[1] << 15) - ey, (p[2] << 15) - ez];
    // Each dot product in Q15; J2 floors each term, at 2^-15 when a fraction is carried and at 1 otherwise.
    let unit = if frac { 30 } else { 45 };
    let dot = |row: [i64; 3]| -> i128 {
        if m.j_terms {
            (0..3).map(|k| ((d[k] as i128 * row[k] as i128) >> unit) << (unit - 30)).sum()
        } else {
            (0..3).map(|k| d[k] as i128 * row[k] as i128).sum::<i128>() >> 30
        }
    };
    let (x, y, mut w) = (dot(r), dot(u), dot(f));
    if m.w_whole {
        w = (w >> 15) << 15;
    }
    [div(m, c.les as i128 * x, w), div(m, c.les as i128 * y, w), div(m, (256 * c.les as i128) << 15, w)]
}

/// Close: within 2, or within 1/128 of the image's value where that is larger (§49.2).
fn close(image: i64, model: i64) -> bool {
    (image - model).abs() <= 2.max(image.abs() / 128)
}

#[derive(Clone, Copy, Default)]
struct Tally {
    close: u64,
    results: u64,
    exact: u64,
    worst: i64,
}

impl Tally {
    fn add(&mut self, image: &[i64], model: &[i64]) {
        for (&a, &b) in image.iter().zip(model) {
            self.results += 1;
            self.close += close(a, b) as u64;
            self.exact += (a == b) as u64;
            self.worst = self.worst.max((a - b).abs());
        }
    }

    fn merge(&mut self, o: &Tally) {
        self.close += o.close;
        self.results += o.results;
        self.exact += o.exact;
        self.worst = self.worst.max(o.worst);
    }
}

fn signed(v: u16) -> i64 {
    v as i16 as i64
}

/// A trace's transactions: (command, inputs, outputs).
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
            let ins = list(&line[i + 4..line[i..].find(']')? + i]);
            let outs = list(&line[o + 5..line[o..].find(']')? + o]);
            Some((command, ins, outs))
        })
        .filter(|(c, _, _)| matches!(c, 0x02 | 0x06 | 0x0A))
        .collect()
}

/// One case's outputs, from the image or a trace: Parameter's inputs and outputs, then Project's and Raster's.
struct Case {
    params: Vec<i64>,
    got: Vec<i64>,
    projects: Vec<(Vec<i64>, Vec<i64>)>,
    rasters: Vec<(i64, Vec<i64>)>,
}

/// Per member index: Parameter's and Raster's tallies over the 48, Project's over the 256.
struct Tallies {
    param: Vec<Tally>,
    raster: Vec<Tally>,
    project: Vec<Tally>,
}

impl Tallies {
    fn new() -> Tallies {
        Tallies { param: vec![Tally::default(); 48], raster: vec![Tally::default(); 48], project: vec![Tally::default(); 256] }
    }

    fn add(&mut self, c: &Case, show: Option<(u32, u32)>) {
        for k in 0..48 {
            let m = Member::pr(k);
            let cam = parameter(m, &c.params);
            if show.is_some_and(|s| s.0 == k) && c.got.iter().zip(&cam.out).any(|(&a, &b)| !close(a, b)) {
                eprintln!("02 {:?} image {:?} model {:?}", c.params, c.got, cam.out);
            }
            self.param[k as usize].add(&c.got, &cam.out);
            for (line, image) in &c.rasters {
                let got = raster(m, &cam, *line);
                if show.is_some_and(|s| s.0 == k) && image.iter().zip(&got).any(|(&a, &b)| !close(a, b)) {
                    eprintln!("A {:?} line {line} image {image:?} model {got:?}", c.params);
                }
                self.raster[k as usize].add(image, &got);
            }
        }
        for k in 0..256 {
            let m = Member::pj(k);
            let base = Member { whole: true, ..m };
            let cam = parameter(base, &c.params);
            for (point, image) in &c.projects {
                let got = project(m, &cam, point);
                if show.is_some_and(|s| s.1 == k) && image.iter().zip(&got).any(|(&a, &b)| !close(a, b)) {
                    eprintln!("P {:?} {point:?} image {image:?} model {got:?}", c.params);
                }
                self.project[k as usize].add(image, &got);
            }
        }
    }
}

/// DSP_PROJ_TIMING=<cmd>: each phase's work and notice over seeded cases after a Parameter, as medians and ranges.
fn timings(stem: &str, command: u8, cases: u64) {
    let image = firmware(stem).unwrap_or_else(|| std::process::exit(1));
    let idle = idle_chip(stem, &image, &mut Host::steady(), 0).unwrap();
    let mut p = Pcg::new(0x0206_7100 + command as u64);
    let mut phases: Vec<(Vec<u32>, Vec<u32>)> = Vec::new();
    for _ in 0..cases {
        let w = |p: &mut Pcg, lo: i64, hi: i64| (lo + p.within(0, (hi - lo) as u32) as i64) as u16;
        let params = [w(&mut p, -4096, 4096), w(&mut p, -4096, 4096), w(&mut p, 0, 1000), w(&mut p, 0, 1024), w(&mut p, 64, 1024), p.word(), w(&mut p, 0x0800, 0x3800)];
        let mut after = idle.clone();
        let got = transact(&mut after, &mut Host::steady(), 0x02, &params).outputs();
        let (host, inputs) = match command {
            0x02 => (Host::steady(), params.to_vec()),
            0x06 => (Host::steady(), vec![got[2].wrapping_add(w(&mut p, -500, 500)), got[3].wrapping_add(w(&mut p, -500, 500)), w(&mut p, 0, 200)]),
            0x0E => (Host::steady(), vec![w(&mut p, -128, 128), w(&mut p, (got[1] as i16 as i64 + 2).clamp(-112, 112), 112)]),
            0x14 => (Host::steady(), vec![p.word(), w(&mut p, -0x3555, 0x3555), p.word(), w(&mut p, -1024, 1024), w(&mut p, -1024, 1024), w(&mut p, -1024, 1024)]),
            _ => (Host { write_from: 9, filler: 0x8000, ..Host::steady() }, vec![w(&mut p, (got[1] as i16 as i64 + 2).clamp(-112, 112), 112)]),
        };
        let from = if command == 0x02 { &idle } else { &after };
        let Some(t) = timing(from, &host, command, &inputs) else { continue };
        phases.resize(phases.len().max(t.len()), (Vec::new(), Vec::new()));
        for (k, ph) in t.iter().enumerate() {
            phases[k].0.push(ph.work);
            phases[k].1.push(ph.notice);
        }
    }
    let stat = |v: &mut Vec<u32>| {
        v.sort();
        format!("{} ({}-{})", v[v.len() / 2], v[0], v[v.len() - 1])
    };
    for (k, (w, n)) in phases.iter_mut().enumerate() {
        println!("  phase {k}: work {} notice {}", stat(w), stat(n));
    }
}

/// DSP_PROJ_PORTS=1: the replacement against the image through the ports over the seeded cases, steady and jittered:
/// per command the cases whose values are all close, all exact, and whose transfers, SR and latency agree.
fn ports(stem: &str, cases: u64) {
    use venusrt::chips::dsphle::Program;
    let image = firmware(stem).unwrap_or_else(|| std::process::exit(1));
    let mut lle = idle_chip(stem, &image, &mut Host::steady(), 0).unwrap();
    let mut hle = Hle::new(Program::for_stem(stem).unwrap());
    power_on(&mut hle, &Host::steady()).unwrap();
    let mut p = Pcg::new(0x0206_0A00);
    // Per command: cases, all close, all exact, shape differs, latency differs, SR differs, largest difference.
    let mut t = [[0i64; 7]; 3];
    for k in 0..cases {
        let w = |p: &mut Pcg, lo: i64, hi: i64| (lo + p.within(0, (hi - lo) as u32) as i64) as u16;
        let params = [w(&mut p, -4096, 4096), w(&mut p, -4096, 4096), w(&mut p, 0, 1000), w(&mut p, 0, 1024), w(&mut p, 64, 1024), p.word(), w(&mut p, 0x0800, 0x3800)];
        let host = |extra: Host| if k % 2 == 1 { Host { write_from: extra.write_from, filler: extra.filler, ..Host::jittered(k) } } else { extra };
        let mut run = |c: usize, command: u8, inputs: &[u16], h: Host, lle: &mut Lle, hle: &mut Hle| -> Vec<u16> {
            let x = transact(lle, &mut host(h.clone()), command, inputs);
            let y = transact(hle, &mut host(h), command, inputs);
            if !matches!(x.end, End::Idle { .. }) || !matches!(y.end, End::Idle { .. }) {
                eprintln!("case {k} {command:02X} {inputs:04X?}: ends {:?} / {:?}", x.end, y.end);
            }
            let d = compare(&x, &y);
            let (xo, yo) = (x.outputs(), y.outputs());
            let diffs: Vec<i64> = xo.iter().zip(&yo).map(|(&a, &b)| (a as i16 as i64 - b as i16 as i64).abs()).collect();
            t[c][0] += 1;
            t[c][1] += xo.iter().zip(&yo).all(|(&a, &b)| close(a as i16 as i64, b as i16 as i64)) as i64;
            t[c][2] += (xo == yo) as i64;
            t[c][3] += d.shape as i64;
            t[c][4] += d.latency as i64;
            t[c][5] += d.status as i64;
            t[c][6] = t[c][6].max(diffs.into_iter().max().unwrap_or(0));
            xo
        };
        let got = run(0, 0x02, &params, Host::steady(), &mut lle, &mut hle);
        let point = [got[2].wrapping_add(w(&mut p, -500, 500)), got[3].wrapping_add(w(&mut p, -500, 500)), w(&mut p, 0, 200)];
        run(2, 0x06, &point, Host::steady(), &mut lle, &mut hle);
        let line = w(&mut p, (got[1] as i16 as i64 + 2).clamp(-112, 112), 112);
        let l = run(1, 0x0A, &[line], Host { write_from: 9, filler: 0x8000, ..Host::steady() }, &mut lle, &mut hle);
        if l.len() != 8 || t[1][3] + t[0][3] + t[2][3] > 0 {
            eprintln!("  after Parameter {params:04X?}");
            lle = idle_chip(stem, &image, &mut Host::steady(), 0).unwrap();
            hle = Hle::new(Program::for_stem(stem).unwrap());
            power_on(&mut hle, &Host::steady()).unwrap();
        }
    }
    for (c, name) in ["02h Parameter", "0Ah Raster (two lines)", "06h Project"].iter().enumerate() {
        let r = t[c];
        println!("{stem} {name}: {} cases, all close {}, all exact {}; shape differs {}, latency {}, SR {}; largest {}", r[0], r[1], r[2], r[3], r[4], r[5], r[6]);
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let stem = &a[1];
    if std::env::var_os("DSP_PROJ_PORTS").is_some() {
        ports(stem, a.get(2).and_then(|c| c.parse().ok()).unwrap_or(1 << 16));
        return;
    }
    if let Some(c) = std::env::var("DSP_PROJ_TIMING").ok().and_then(|c| u8::from_str_radix(&c, 16).ok()) {
        timings(stem, c, a.get(2).and_then(|c| c.parse().ok()).unwrap_or(4096));
        return;
    }
    let cases: u64 = a.get(2).and_then(|c| c.parse().ok()).unwrap_or(1 << 16);
    let show = std::env::var("DSP_PROJ_SHOW").ok().and_then(|v| v.split_once(',').map(|(x, y)| (x.parse().unwrap(), y.parse().unwrap())));
    let mut sets: Vec<(String, Tallies)> = Vec::new();
    if cases > 0 {
        let mut t = Tallies::new();
        let image = firmware(stem).unwrap_or_else(|| std::process::exit(1));
        let mut chip = idle_chip(stem, &image, &mut Host::steady(), 0).unwrap();
        let mut p = Pcg::new(0x0206_0A00);
        for _ in 0..cases {
            let w = |p: &mut Pcg, lo: i64, hi: i64| lo + p.within(0, (hi - lo) as u32) as i64;
            let params = vec![w(&mut p, -4096, 4096), w(&mut p, -4096, 4096), w(&mut p, 0, 1000), w(&mut p, 0, 1024), w(&mut p, 64, 1024), p.word() as i64, w(&mut p, 0x0800, 0x3800)];
            let words: Vec<u16> = params.iter().map(|&v| v as u16).collect();
            let got: Vec<i64> = transact(&mut chip, &mut Host::steady(), 0x02, &words).outputs().iter().map(|&v| signed(v)).collect();
            if got.len() < 4 {
                continue;
            }
            let point = vec![got[2] + w(&mut p, -500, 500), got[3] + w(&mut p, -500, 500), w(&mut p, 0, 200)];
            let pw: Vec<u16> = point.iter().map(|&v| v as u16).collect();
            let proj: Vec<i64> = transact(&mut chip, &mut Host::steady(), 0x06, &pw).outputs().iter().map(|&v| signed(v)).collect();
            let line = w(&mut p, (got[1] + 2).clamp(-112, 112), 112);
            let ras: Vec<i64> = transact(&mut chip, &mut Host { write_from: 5, filler: 0x8000, ..Host::steady() }, 0x0A, &[line as u16]).outputs().iter().map(|&v| signed(v)).collect();
            t.add(&Case { params, got, projects: vec![(point, proj)], rasters: vec![(line, ras)] }, show);
        }
        sets.push(("seeded".into(), t));
    }
    for path in a.iter().skip(3) {
        let mut t = Tallies::new();
        let mut case: Option<Case> = None;
        for (command, ins, outs) in trace(path) {
            match command {
                0x02 if ins.len() == 7 && outs.len() == 4 => {
                    if let Some(c) = case.take() {
                        t.add(&c, show);
                    }
                    case = Some(Case { params: ins, got: outs, projects: Vec::new(), rasters: Vec::new() });
                }
                0x06 if ins.len() == 3 && outs.len() == 3 => {
                    if let Some(c) = case.as_mut() {
                        c.projects.push((ins, outs));
                    }
                }
                0x0A if ins.len() == 1 => {
                    if let Some(c) = case.as_mut() {
                        // The run's last line is where the game writes its terminator over the results: not graded.
                        let lines = outs.len().div_ceil(4).saturating_sub(1);
                        for (k, line) in outs.chunks_exact(4).take(lines).enumerate() {
                            c.rasters.push((ins[0] + k as i64, line.to_vec()));
                        }
                    }
                }
                _ => {}
            }
        }
        if let Some(c) = case.take() {
            t.add(&c, show);
        }
        let name = std::path::Path::new(path).file_stem().unwrap().to_string_lossy().into_owned();
        sets.push((name, t));
    }
    // Every combination of a Parameter-Raster member and a Project member sharing S2, S3 and P1.
    let mut combos: Vec<(u32, u32, u64, i64)> = Vec::new();
    for k in 0..48u32 {
        for j in 0..256u32 {
            let (m, n) = (Member::pr(k), Member::pj(j));
            if (m.routine, m.half, m.p_floored) != (n.routine, n.half, n.p_floored) {
                continue;
            }
            let mut sum = Tally::default();
            for (_, t) in &sets {
                sum.merge(&t.param[k as usize]);
                sum.merge(&t.raster[k as usize]);
                sum.merge(&t.project[j as usize]);
            }
            combos.push((k, j, sum.close, sum.worst));
        }
    }
    combos.sort_by(|x, y| y.2.cmp(&x.2).then(x.3.cmp(&y.3)));
    println!("{stem}: {} members", combos.len());
    // DSP_PROJ_LIST=pr,pj[;pr,pj...] reports those members too, with their rank.
    let listed: Vec<(u32, u32)> = std::env::var("DSP_PROJ_LIST").map(|v| v.split(';').filter_map(|m| m.split_once(',').map(|(x, y)| (x.parse().unwrap(), y.parse().unwrap()))).collect()).unwrap_or_default();
    let picked: Vec<(usize, (u32, u32, u64, i64))> = combos.iter().copied().enumerate().filter(|(r, c)| *r < 6 || listed.contains(&(c.0, c.1))).collect();
    for (rank, (k, j, close, worst)) in picked {
        println!("  rank {rank}:");
        let (m, n) = (Member::pr(k), Member::pj(j));
        println!(
            "  member {k},{j}: S1 whole {} S2 routine {} S3 half {} P1 floored {} R1 {} J1 floored {} J2 terms {} J3 height frac {} horizontal frac {} J4 w whole {}: close {close}, largest {worst}",
            m.whole, m.routine, m.half, m.p_floored, m.den, n.j_floored, n.j_terms, n.z_frac, n.h_frac, n.w_whole
        );
        for (name, t) in &sets {
            let (pa, ra, pj) = (t.param[k as usize], t.raster[k as usize], t.project[j as usize]);
            println!(
                "    {name}: Parameter {}/{} ({} exact), Raster {}/{} ({}), Project {}/{} ({})",
                pa.close, pa.results, pa.exact, ra.close, ra.results, ra.exact, pj.close, pj.results, pj.exact
            );
        }
    }
}
