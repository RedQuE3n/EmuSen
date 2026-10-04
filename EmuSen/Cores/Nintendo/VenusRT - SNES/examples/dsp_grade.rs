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
    first: Vec<Vec<u16>>,
}

fn grade(stem: &str, image: &[u8], command: u8, sets: impl Iterator<Item = Vec<u16>>, seed: u64) -> Tally {
    let program = Program::for_stem(stem).expect("a program with a replacement");
    let mut lle = idle_chip(stem, image, &mut Host::steady(), 0).unwrap();
    let mut hle = Hle::new(program);
    power_on(&mut hle, &Host::steady()).unwrap();
    let mut t = Tally { cases: 0, shape: 0, values: 0, latency: 0, status: 0, first: Vec::new() };
    let (mut hs, mut ha) = (Host::steady(), Host::steady());
    for (k, set) in sets.enumerate() {
        let jitter = k % 2 == 1;
        if jitter && k % 512 == 1 {
            hs = Host::jittered(seed + k as u64);
            ha = hs.clone();
        }
        let (x, y) = if jitter {
            (transact(&mut lle, &mut hs, command, &set), transact(&mut hle, &mut ha, command, &set))
        } else {
            (transact(&mut lle, &mut Host::steady(), command, &set), transact(&mut hle, &mut Host::steady(), command, &set))
        };
        let d = compare(&x, &y);
        t.cases += 1;
        t.shape += d.shape as u64;
        t.values += d.values as u64;
        t.latency += d.latency as u64;
        t.status += d.status as u64;
        if d.any() && t.first.len() < 10 {
            t.first.push(set.clone());
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
    let start = std::time::Instant::now();
    let handles: Vec<_> = (0..threads)
        .map(|th| {
            let (stem, image) = (stem.clone(), image.clone());
            std::thread::spawn(move || {
                if all {
                    let span = (1u64 << 32) / threads;
                    let sets = (th * span..(th + 1) * span).map(|v| vec![(v >> 16) as u16, v as u16]);
                    grade(&stem, &image, command, sets, th)
                } else {
                    let mut p = Pcg::new(0x6EAD_0000 + th + 0x100 * command as u64);
                    let sets = (0..cases / threads).map(move |_| (0..n).map(|_| p.word()).collect());
                    grade(&stem, &image, command, sets, th)
                }
            })
        })
        .collect();
    let mut sum = Tally { cases: 0, shape: 0, values: 0, latency: 0, status: 0, first: Vec::new() };
    for h in handles {
        let t = h.join().unwrap();
        sum.cases += t.cases;
        sum.shape += t.shape;
        sum.values += t.values;
        sum.latency += t.latency;
        sum.status += t.status;
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
    let mut report = format!("{line}\n");
    for s in &sum.first {
        let _ = writeln!(report, "  inputs {s:04X?}");
    }
    let dir = std::path::PathBuf::from(std::env::var_os("HOME").unwrap()).join(".cache/emusen/probe/venusrt/dsp-hle");
    let _ = std::fs::create_dir_all(&dir);
    std::fs::write(dir.join(format!("grade-{stem}-{command:02x}.txt")), &report).unwrap();
    print!("{report}");
}
