//! Grades VenusRT's replacement against the image, command by command, through the ports (VenusRT_DspHle.md §4.1):
//! `dsp_grade <chip> <command> <cases|all> [threads]`. Values, transfers, SR and latency are compared under a steady
//! S-CPU and one whose answer time is drawn at every transfer. `all` runs every pair of a two-input command. Counts
//! and the first mismatches' inputs go to stdout and to ~/.cache/emusen/probe/venusrt/dsp-hle/grade-<chip>-<cc>.txt.
use std::fmt::Write as _;
use venusrt::chips::dsphle::Program;
use venusrt::chips::dsporacle::*;

struct Tally {
    cases: u64,
    shape: u64,
    values: u64,
    latency: u64,
    status: u64,
    /// The largest difference of a result word, as signed 16-bit values, and how many cases had each size up to 16.
    worst: i64,
    sizes: [u64; 17],
    first: Vec<Vec<u16>>,
}

fn grade(stem: &str, image: &[u8], command: u8, sets: impl Iterator<Item = Vec<u16>>, seed: u64, before: Option<(u8, usize)>) -> Tally {
    let nonzero = std::env::var_os("DSP_GRADE_NONZERO").is_some();
    // DSP_GRADE_FIRST=lo-hi[,lo-hi...] holds the first input words in those ranges.
    let first: Vec<(u16, u16)> = std::env::var("DSP_GRADE_FIRST").map(|v| {
        v.split(',').map(|r| {
            let (a, b) = r.split_once('-').unwrap();
            (u16::from_str_radix(a, 16).unwrap(), u16::from_str_radix(b, 16).unwrap())
        }).collect()
    }).unwrap_or_default();
    let program = Program::for_stem(stem).expect("a program with a replacement");
    let mut lle = idle_chip(stem, image, &mut Host::steady(), 0).unwrap();
    let mut hle = Hle::new(program);
    power_on(&mut hle, &Host::steady()).unwrap();
    let mut t = Tally { cases: 0, shape: 0, values: 0, latency: 0, status: 0, worst: 0, sizes: [0; 17], first: Vec::new() };
    // DSP_GRADE_WRITE_OVER, a bit mask in hex: the transfers answered with a write over the chip's word (the DSP-4).
    let write_over = std::env::var("DSP_GRADE_WRITE_OVER").ok().and_then(|v| u64::from_str_radix(&v, 16).ok()).unwrap_or(0);
    let steady = || Host { write_over, ..Host::steady() };
    let (mut hs, mut ha) = (steady(), steady());
    for (k, set) in sets.enumerate() {
        let jitter = k % 2 == 1;
        if jitter && k % 512 == 1 {
            hs = Host { write_over, ..Host::jittered(seed + k as u64) };
            ha = hs.clone();
        }
        let mut set = set;
        if nonzero && set[0] as u8 == 0 {
            set[0] |= 1;
        }
        for (k, &(lo, hi)) in first.iter().enumerate() {
            set[k] = lo + set[k] % (hi - lo + 1);
        }
        if let Some((before, k)) = before {
            let set = &set[set.len() - k..];
            transact(&mut lle, &mut Host::steady(), before, set);
            transact(&mut hle, &mut Host::steady(), before, set);
        }
        let (x, y) = if jitter {
            (transact(&mut lle, &mut hs, command, &set), transact(&mut hle, &mut ha, command, &set))
        } else {
            (transact(&mut lle, &mut steady(), command, &set), transact(&mut hle, &mut steady(), command, &set))
        };
        let d = compare(&x, &y);
        t.cases += 1;
        t.shape += d.shape as u64;
        t.values += d.values as u64;
        t.latency += d.latency as u64;
        t.status += d.status as u64;
        let e = x.outputs().iter().zip(y.outputs()).map(|(&a, b)| (a as i16 as i64 - b as i16 as i64).abs()).max().unwrap_or(0);
        t.worst = t.worst.max(e);
        t.sizes[e.min(16) as usize] += 1;
        if d.any() && t.first.len() < 10 {
            t.first.push(set.iter().take(12).copied().collect());
            if std::env::var_os("DSP_GRADE_SHOW").is_some() {
                eprintln!("{d:?}\n  image {x:?}\n  hle   {y:?}");
            }
        }
        if !matches!(x.end, End::Idle { .. }) || !matches!(y.end, End::Idle { .. }) {
            lle = idle_chip(stem, image, &mut Host::steady(), 0).unwrap();
            hle = Hle::new(program);
            power_on(&mut hle, &Host::steady()).unwrap();
        }
    }
    t
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let stem = a[1].clone();
    let command = u8::from_str_radix(&a[2], 16).unwrap();
    let all = a.get(3).map(String::as_str) == Some("all");
    let cases: u64 = a.get(3).and_then(|c| c.parse().ok()).unwrap_or(1 << 20);
    let threads: u64 = a.get(4).and_then(|c| c.parse().ok()).unwrap_or(1);
    let image = firmware(&stem).unwrap_or_else(|| std::process::exit(1));
    let n = transact(&mut idle_chip(&stem, &image, &mut Host::steady(), 0).unwrap(), &mut Host::steady(), command, &[0; 8]).inputs().max(1);
    // A variable-length command takes as many words as DSP_GRADE_WORDS gives; DSP_GRADE_BEFORE runs a command first.
    let n = n.max(std::env::var("DSP_GRADE_WORDS").ok().and_then(|v| v.parse().ok()).unwrap_or(if stem == "dsp2" { 300 } else { 8 }));
    // DSP_GRADE_BEFORE=cc[:k] runs command cc first with the case's last k words (1 by default).
    let before = std::env::var("DSP_GRADE_BEFORE").ok().map(|c| {
        let (c, k) = c.split_once(':').unwrap_or((&c, "1"));
        (u8::from_str_radix(c, 16).unwrap(), k.parse::<usize>().unwrap())
    });
    let start = std::time::Instant::now();
    let handles: Vec<_> = (0..threads)
        .map(|th| {
            let (stem, image) = (stem.clone(), image.clone());
            std::thread::spawn(move || {
                if all {
                    let span = (1u64 << 32) / threads;
                    let sets = (th * span..(th + 1) * span).map(|v| vec![(v >> 16) as u16, v as u16]);
                    grade(&stem, &image, command, sets, th, before)
                } else {
                    let mut p = Pcg::new(0x6EAD_0000 + th + 0x100 * command as u64);
                    let sets = (0..cases / threads).map(move |_| (0..n).map(|_| p.word()).collect());
                    grade(&stem, &image, command, sets, th, before)
                }
            })
        })
        .collect();
    let mut sum = Tally { cases: 0, shape: 0, values: 0, latency: 0, status: 0, worst: 0, sizes: [0; 17], first: Vec::new() };
    for h in handles {
        let t = h.join().unwrap();
        sum.cases += t.cases;
        sum.shape += t.shape;
        sum.values += t.values;
        sum.latency += t.latency;
        sum.status += t.status;
        sum.worst = sum.worst.max(t.worst);
        for k in 0..17 {
            sum.sizes[k] += t.sizes[k];
        }
        sum.first.extend(t.first.into_iter().take(10 - sum.first.len().min(10)));
    }
    let line = format!(
        "{stem} {command:02X}: {} cases{} in {:.0} s; differ in shape {}, values {}, latency {}, SR {}",
        sum.cases,
        if all { " (every pair)" } else { "" },
        start.elapsed().as_secs_f64(),
        sum.shape,
        sum.values,
        sum.latency,
        sum.status
    );
    let mut report = format!("{line}\n  largest result difference {}; cases by its size 0..15 and 16+: {:?}\n", sum.worst, sum.sizes);
    for s in &sum.first {
        let _ = writeln!(report, "  inputs {s:04X?}");
    }
    let dir = std::path::PathBuf::from(std::env::var_os("HOME").unwrap()).join(".cache/emusen/probe/venusrt/dsp-hle");
    let _ = std::fs::create_dir_all(&dir);
    std::fs::write(dir.join(format!("grade-{stem}-{command:02x}.txt")), &report).unwrap();
    print!("{report}");
}
