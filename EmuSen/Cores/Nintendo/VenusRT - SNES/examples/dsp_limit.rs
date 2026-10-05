//! Grades VenusRT_Native.md §52.4's family, with §52.5's widening, for the DSP-1's branch past the limit angle against the image:
//! `dsp_limit <chip> [cases] [trace...]`, seeded cases through the ports and the traced commands past the limit.
//! Counts only. DSP_LIMIT_PORTS=1 grades the replacement itself through the ports, steady and jittered.
use venusrt::chips::dsphle::{Program, sine};
use venusrt::chips::dsporacle::*;

/// The limit angle on Azs (VenusRT_Native.md §52.3), the one constant §52.1 admits.
const LIMIT: u16 = 0x38CE;

fn mul(a: i64, b: i64) -> i64 {
    (a * b) >> 15
}

fn fdiv(num: i128, den: i128, wide: bool) -> i64 {
    let lim = if wide { (i64::MIN as i128 / 4, i64::MAX as i128 / 4) } else { (-0x8000, 0x7FFF) };
    if den == 0 {
        return if num >= 0 { lim.1 as i64 } else { lim.0 as i64 };
    }
    let (n, d) = if den < 0 { (-num, -den) } else { (num, den) };
    n.div_euclid(d).clamp(lim.0, lim.1) as i64
}

/// Member m: B1 Vof half up, B2 the screen at Les/cos Δ, B3 K with cos Δ, B4 Project on the virtual camera, B5
/// (§52.5) Vva rounded toward zero.
#[derive(Clone, Copy)]
struct Branch {
    half: bool,
    les_over_cos: bool,
    k_cos: bool,
    project_virtual: bool,
    toward_zero: bool,
}

impl Branch {
    fn of(m: u32) -> Branch {
        Branch { half: m & 1 != 0, les_over_cos: m & 2 != 0, k_cos: m & 4 != 0, project_virtual: m & 8 != 0, toward_zero: m & 16 != 0 }
    }
}

struct Cam {
    f: [i64; 3],
    lfe: i64,
    les: i64,
    sa: i64,
    ca: i64,
    /// The eye's angle, Azs itself.
    sz: i64,
    cz: i64,
    /// The view's angle, the limit's.
    sv: i64,
    cv: i64,
    cd: i64,
    vof: i64,
    /// Les_v·cos z_v in Q15.
    lv: i64,
    ez: i64,
    c: [i64; 2],
    out: [i64; 4],
}

fn camera(b: Branch, p: &[i64]) -> Cam {
    let w = |k: usize| p[k];
    let (a, z) = (p[5] as u16, p[6] as u16);
    let zv = if z < 0x8000 { LIMIT } else { LIMIT.wrapping_neg() };
    let d = z.wrapping_sub(zv);
    let (sa, ca, sz, cz) = (sine(a), sine(a.wrapping_add(0x4000)), sine(z), sine(z.wrapping_add(0x4000)));
    let (sv, cv, sd, cd) = (sine(zv), sine(zv.wrapping_add(0x4000)), sine(d), sine(d.wrapping_add(0x4000)));
    let les = w(4);
    let num = les as i128 * sd as i128;
    let vof = if b.half { fdiv(2 * num + cd as i128, 2 * cd as i128, false) } else { fdiv(num, cd as i128, false) };
    let lv = if b.les_over_cos { fdiv(((les * cv) as i128) << 15, cd as i128, true) } else { les * cv };
    let vva = if b.toward_zero { -fdiv(lv as i128, sv as i128, false) } else { fdiv(-(lv as i128), sv as i128, false) };
    let ez = w(2) + mul(w(3), cz);
    let ex = w(0) - ((w(3) * sz * sa) >> 30);
    let ey = w(1) + ((w(3) * sz * ca) >> 30);
    let t = fdiv(ez as i128 * sv as i128, cv as i128, false);
    let c = [(ex + mul(t, sa)).clamp(-0x8000, 0x7FFF), (ey - mul(t, ca)).clamp(-0x8000, 0x7FFF)];
    Cam { f: [w(0), w(1), w(2)], lfe: w(3), les, sa, ca, sz, cz, sv, cv, cd, vof, lv, ez, c, out: [vof, vva, c[0], c[1]] }
}

fn raster(b: Branch, c: &Cam, v: i64) -> [i64; 4] {
    let n = (c.lv >> 15) + mul(v, c.sv);
    let k = fdiv(256 * c.ez as i128, n as i128, false);
    let kk = if b.k_cos { fdiv(k as i128 * c.cd as i128, c.cv as i128, false) } else { fdiv((k as i128) << 15, c.cv as i128, false) };
    [mul(k, c.ca), mul(-kk, c.sa), mul(k, c.sa), mul(kk, c.ca)]
}

fn target(c: &Cam, h: i64, v: i64) -> [i64; 2] {
    let n = (c.lv >> 15) + mul(v, c.sv);
    let k = fdiv(256 * c.ez as i128, n as i128, true) as i128;
    let kk = fdiv(k << 15, c.cv as i128, true) as i128;
    let (h, v, sa, ca) = (h as i128, v as i128, c.sa as i128, c.ca as i128);
    let dx = (((h * k * ca) >> 15) - ((v * kk * sa) >> 15)) >> 8;
    let dy = (((v * kk * ca) >> 15) - ((h * k * sa) >> 15)) >> 8;
    [(c.c[0] as i128 + dx).clamp(-0x8000, 0x7FFF) as i64, (c.c[1] as i128 + dy).clamp(-0x8000, 0x7FFF) as i64]
}

/// §52.2's Project, its view at (sz, cz) or at the limit's, the eye from Azs.
fn project(b: Branch, c: &Cam, p: &[i64]) -> [i64; 3] {
    let (sz, cz) = if b.project_virtual { (c.sv, c.cv) } else { (c.sz, c.cz) };
    let (sa, ca) = (c.sa, c.ca);
    let m = |a: i64, b: i64| ((a * b) >> 15) << 15;
    let r = [ca << 15, sa << 15, 0];
    let u = [m(-sa, cz), m(ca, cz), -sz << 15];
    let f = [m(sa, sz), m(-ca, sz), -cz << 15];
    // The eye from Azs itself.
    let fe = [m(sa, c.sz), m(-ca, c.sz)];
    let e = [(c.f[0] << 15) - ((c.lfe * fe[0]) >> 15), (c.f[1] << 15) - ((c.lfe * fe[1]) >> 15), (c.f[2] << 15) + c.lfe * c.cz];
    let d = [(p[0] << 15) - e[0], (p[1] << 15) - e[1], (p[2] << 15) - e[2]];
    let dot = |row: [i64; 3]| (0..3).map(|k| d[k] as i128 * row[k] as i128).sum::<i128>() >> 30;
    let (x, y, w) = (dot(r), dot(u), (dot(f) >> 15) << 15);
    let les = c.les as i128;
    let v = fdiv(les * y, w, false);
    [fdiv(les * x, w, false), if b.project_virtual { (v + c.vof).clamp(-0x8000, 0x7FFF) } else { v }, fdiv((256 * les) << 15, w, false)]
}

fn close(image: i64, model: i64) -> bool {
    (image - model).abs() <= 2.max(image.abs() / 128)
}

fn signed(v: u16) -> i64 {
    v as i16 as i64
}

fn past(z: u16) -> bool {
    (LIMIT..=0x4800).contains(&z) || (0xB800..=LIMIT.wrapping_neg()).contains(&z)
}

#[derive(Clone, Copy, Default)]
struct Tally {
    close: [u64; 4],
    exact: [u64; 4],
    results: [u64; 4],
}

impl Tally {
    fn add(&mut self, k: usize, image: &[i64], model: &[i64]) {
        for (&a, &b) in image.iter().zip(model) {
            self.results[k] += 1;
            self.close[k] += close(a, b) as u64;
            self.exact[k] += (a == b) as u64;
        }
    }
}

struct Case {
    params: Vec<i64>,
    got: Vec<i64>,
    projects: Vec<(Vec<i64>, Vec<i64>)>,
    rasters: Vec<(i64, Vec<i64>)>,
    targets: Vec<(Vec<i64>, Vec<i64>)>,
}

fn grade(t: &mut [Tally], c: &Case) {
    // DSP_LIMIT_SHOW=<member> prints that member's results that are not close, for characterisation.
    let show = std::env::var("DSP_LIMIT_SHOW").ok().and_then(|v| v.parse::<usize>().ok());
    if let Some(m) = show {
        let b = Branch::of(m as u32);
        let cam = camera(b, &c.params);
        let bad = |x: &[i64], y: &[i64]| x.iter().zip(y).any(|(&a, &b)| !close(a, b));
        if bad(&c.got, &cam.out) {
            eprintln!("02 {:?} image {:?} model {:?}", c.params, c.got, cam.out);
        }
        for (line, image) in &c.rasters {
            let r = raster(b, &cam, *line);
            if bad(image, &r) {
                eprintln!("0A {:?} line {line} image {image:?} model {r:?}", c.params);
            }
        }
        for (hv, image) in &c.targets {
            let r = target(&cam, hv[0], hv[1]);
            if bad(image, &r) {
                eprintln!("0E {:?} {hv:?} image {image:?} model {r:?}", c.params);
            }
        }
    }
    for (m, t) in t.iter_mut().enumerate() {
        let b = Branch::of(m as u32);
        let cam = camera(b, &c.params);
        t.add(0, &c.got, &cam.out);
        for (line, image) in &c.rasters {
            t.add(1, image, &raster(b, &cam, *line));
        }
        for (pt, image) in &c.projects {
            t.add(2, image, &project(b, &cam, pt));
        }
        for (hv, image) in &c.targets {
            t.add(3, image, &target(&cam, hv[0], hv[1]));
        }
    }
}

fn seeded(p: &mut Pcg) -> Vec<i64> {
    let mut w = |lo: i64, hi: i64| lo + p.within(0, (hi - lo) as u32) as i64;
    vec![w(-4096, 4096), w(-4096, 4096), w(0, 1000), w(0, 1024), w(64, 1024), w(0, 0xFFFF), w(LIMIT as i64, 0x4800)]
}

fn ports(stem: &str, cases: u64) {
    let image = firmware(stem).unwrap_or_else(|| std::process::exit(1));
    let mut lle = idle_chip(stem, &image, &mut Host::steady(), 0).unwrap();
    let mut hle = Hle::new(Program::for_stem(stem).unwrap());
    power_on(&mut hle, &Host::steady()).unwrap();
    let mut p = Pcg::new(0x38CE_0001);
    let mut t = [[0i64; 5]; 4];
    for k in 0..cases {
        let host = |h: Host| if k % 2 == 1 { Host { write_from: h.write_from, filler: h.filler, ..Host::jittered(k) } } else { h };
        let params: Vec<u16> = seeded(&mut p).iter().map(|&v| v as u16).collect();
        let mut run = |c: usize, command: u8, inputs: &[u16], h: Host, lle: &mut Lle, hle: &mut Hle| -> Vec<u16> {
            let (x, y) = (transact(lle, &mut host(h.clone()), command, inputs), transact(hle, &mut host(h), command, inputs));
            let d = compare(&x, &y);
            t[c][0] += 1;
            t[c][1] += x.outputs().iter().zip(y.outputs()).all(|(&a, b)| close(signed(a), signed(b))) as i64;
            t[c][2] += (x.outputs() == y.outputs()) as i64;
            t[c][3] += (d.shape || d.status) as i64;
            t[c][4] += d.latency as i64;
            x.outputs()
        };
        let got = run(0, 0x02, &params, Host::steady(), &mut lle, &mut hle);
        let vva = signed(got[1]);
        let lo = (vva + 2).clamp(-112, 112);
        let mut w = |lo: i64, hi: i64| (lo + p.within(0, (hi - lo).max(0) as u32) as i64) as u16;
        let point = [got[2].wrapping_add(w(-500, 500)), got[3].wrapping_add(w(-500, 500)), w(0, 200)];
        run(2, 0x06, &point, Host::steady(), &mut lle, &mut hle);
        let line = w(lo, 112);
        run(1, 0x0A, &[line], Host { write_from: 9, filler: 0x8000, ..Host::steady() }, &mut lle, &mut hle);
        let hv = [w(-128, 128), w(lo, 112)];
        run(3, 0x0E, &hv, Host::steady(), &mut lle, &mut hle);
    }
    for (c, name) in ["02h Parameter", "0Ah Raster (two lines)", "06h Project", "0Eh Target"].iter().enumerate() {
        let r = t[c];
        println!("{stem} past the limit, {name}: {} cases, all close {}, all exact {}; transfers or SR differ {}, latency {}", r[0], r[1], r[2], r[3], r[4]);
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let stem = &a[1];
    let cases: u64 = a.get(2).and_then(|c| c.parse().ok()).unwrap_or(1 << 16);
    if std::env::var_os("DSP_LIMIT_PORTS").is_some() {
        ports(stem, cases);
        return;
    }
    let mut sets: Vec<(String, Vec<Tally>)> = Vec::new();
    if cases > 0 {
        let image = firmware(stem).unwrap_or_else(|| std::process::exit(1));
        let mut chip = idle_chip(stem, &image, &mut Host::steady(), 0).unwrap();
        let mut p = Pcg::new(0x38CE_0000);
        let mut t = vec![Tally::default(); 32];
        for _ in 0..cases {
            let params = seeded(&mut p);
            let words: Vec<u16> = params.iter().map(|&v| v as u16).collect();
            let got: Vec<i64> = transact(&mut chip, &mut Host::steady(), 0x02, &words).outputs().iter().map(|&v| signed(v)).collect();
            let lo = (got[1] + 2).clamp(-112, 112);
            let mut w = |lo: i64, hi: i64| lo + p.within(0, (hi - lo).max(0) as u32) as i64;
            let point = vec![got[2] + w(-500, 500), got[3] + w(-500, 500), w(0, 200)];
            let pw: Vec<u16> = point.iter().map(|&v| v as u16).collect();
            let proj = transact(&mut chip, &mut Host::steady(), 0x06, &pw).outputs().iter().map(|&v| signed(v)).collect();
            let line = w(lo, 112);
            let ras = transact(&mut chip, &mut Host { write_from: 5, filler: 0x8000, ..Host::steady() }, 0x0A, &[line as u16]).outputs().iter().map(|&v| signed(v)).collect();
            let hv = vec![w(-128, 128), w(lo, 112)];
            let tg = transact(&mut chip, &mut Host::steady(), 0x0E, &[hv[0] as u16, hv[1] as u16]).outputs().iter().map(|&v| signed(v)).collect();
            grade(&mut t, &Case { params, got, projects: vec![(point, proj)], rasters: vec![(line, ras)], targets: vec![(hv, tg)] });
        }
        sets.push(("seeded".into(), t));
    }
    let list = |s: &str| -> Vec<i64> { s.split(',').filter_map(|w| u16::from_str_radix(w.trim(), 16).ok()).map(signed).collect() };
    for path in a.iter().skip(3) {
        let mut t = vec![Tally::default(); 32];
        let mut case: Option<Case> = None;
        for line in std::fs::read_to_string(path).expect("a trace").lines() {
            let mut words = line.split_whitespace();
            let (Some(_), Some(c)) = (words.next(), words.next()) else { continue };
            let Ok(command) = u8::from_str_radix(c, 16) else { continue };
            let (Some(i), Some(o)) = (line.find("in ["), line.find("out [")) else { continue };
            let ins = list(&line[i + 4..line[i..].find(']').unwrap() + i]);
            let outs = list(&line[o + 5..line[o..].find(']').unwrap() + o]);
            match command {
                0x02 if ins.len() == 7 && outs.len() == 4 => {
                    if let Some(c) = case.take() {
                        grade(&mut t, &c);
                    }
                    if past(ins[6] as u16) {
                        case = Some(Case { params: ins, got: outs, projects: Vec::new(), rasters: Vec::new(), targets: Vec::new() });
                    }
                }
                0x06 if ins.len() == 3 && outs.len() == 3 => {
                    if let Some(c) = case.as_mut() {
                        c.projects.push((ins, outs));
                    }
                }
                0x0E if ins.len() == 2 && outs.len() == 2 => {
                    if let Some(c) = case.as_mut() {
                        c.targets.push((ins, outs));
                    }
                }
                0x0A if ins.len() == 1 => {
                    if let Some(c) = case.as_mut() {
                        let lines = outs.len().div_ceil(4).saturating_sub(1);
                        for (k, l) in outs.chunks_exact(4).take(lines).enumerate() {
                            c.rasters.push((ins[0] + k as i64, l.to_vec()));
                        }
                    }
                }
                _ => {}
            }
        }
        if let Some(c) = case.take() {
            grade(&mut t, &c);
        }
        sets.push((std::path::Path::new(path).file_stem().unwrap().to_string_lossy().into_owned(), t));
    }
    let total = |m: usize, exact: bool| sets.iter().map(|s| if exact { s.1[m].exact.iter().sum::<u64>() } else { s.1[m].close.iter().sum::<u64>() }).sum::<u64>();
    let mut order: Vec<usize> = (0..32).collect();
    order.sort_by(|&x, &y| total(y, true).cmp(&total(x, true)).then(total(y, false).cmp(&total(x, false))));
    for &m in order.iter().take(6) {
        let b = Branch::of(m as u32);
        println!("member {m:05b} (Vof half up {}, screen Les/cos Δ {}, K with cos Δ {}, Project virtual {}, Vva toward zero {}): exact {}, close {}", b.half, b.les_over_cos, b.k_cos, b.project_virtual, b.toward_zero, total(m, true), total(m, false));
        for (name, t) in &sets {
            let x = t[m];
            println!(
                "  {name}: Parameter {}/{} ({} exact), Raster {}/{} ({}), Project {}/{} ({}), Target {}/{} ({})",
                x.close[0], x.results[0], x.exact[0], x.close[1], x.results[1], x.exact[1], x.close[2], x.results[2], x.exact[2], x.close[3], x.results[3], x.exact[3]
            );
        }
    }
}
