//! The NEC DSP command oracle over the low-level path (VenusRT_Native.md §37): `dsp_oracle sweep <chip> [sets]`,
//! `dsp_oracle versus <chip> <chip> [cases]`, `dsp_oracle ask <chip> <command> <inputs...> [/ <command> ...]` (transactions in turn), `dsp_oracle latency <chip> [cases]`, `dsp_oracle tables` (the
//! replacement's generated tables, for firmwarecheck.py). Images from EMUSEN_VENUSRT_FIRMWARE;
//! full reports to ~/.cache/emusen/probe/venusrt/dsp-hle/, counts and cycles only on stdout.
use std::collections::BTreeMap;
use std::fmt::Write as _;
use venusrt::chips::dsporacle::*;
use venusrt::chips::necdsp::Port;

fn cache() -> std::path::PathBuf {
    let home = std::env::var_os("HOME").expect("HOME");
    let dir = std::path::PathBuf::from(home).join(".cache/emusen/probe/venusrt/dsp-hle");
    std::fs::create_dir_all(&dir).expect("the cache folder");
    dir
}

fn host() -> Host {
    Host { cap: 2_000_000, max_steps: 1100, ..Host::steady() }
}

fn idle(stem: &str, offset: u32, h: &mut Host) -> Lle {
    let image = firmware(stem).unwrap_or_else(|| std::process::exit(1));
    idle_chip(stem, &image, h, offset).expect("the chip reaches idle")
}

fn inputs(seed: u64, n: usize) -> Vec<u16> {
    let mut p = Pcg::new(seed);
    (0..n).map(|_| p.word()).collect()
}

fn mail_inputs(seed: u64) -> Vec<(usize, u16)> {
    let mut p = Pcg::new(seed);
    (0..0x100).map(|w| (w, p.word())).filter(|&(w, _)| w != 0x10).collect()
}

/// A latency list as runs, "12,4x3,2".
fn runs(l: &[u32]) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < l.len() {
        let mut j = i;
        while j + 1 < l.len() && l[j + 1] == l[i] {
            j += 1;
        }
        out.push(if j > i { format!("{}x{}", l[i], j - i + 1) } else { l[i].to_string() });
        i = j + 1;
    }
    out.join(",")
}

fn shape_runs(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < b.len() {
        let mut j = i;
        while j + 1 < b.len() && b[j + 1] == b[i] {
            j += 1;
        }
        let _ = if j > i { write!(out, "{}{}", b[i] as char, j - i + 1) } else { write!(out, "{}", b[i] as char) };
        i = j + 1;
    }
    out
}

fn sweep_dr(stem: &str, sets: u64) {
    let mut report = String::new();
    let a = idle(stem, 0, &mut host());
    let b = idle(stem, 10_007, &mut host());
    let zero = sweep(&a, &mut host(), &[]);
    let mut all = vec![zero.clone()];
    let (mut offset_diff, mut jitter_values, mut model_ok, mut model_bad, mut model_none) = (0, 0, 0, 0, 0);
    let mut jitter_bytes = std::collections::BTreeSet::new();
    for seed in 0..sets {
        let set = inputs(seed, 64);
        let x = sweep(&a, &mut host(), &set);
        let y = sweep(&b, &mut host(), &set);
        let mut jh = Host { cap: 2_000_000, max_steps: 1100, ..Host::jittered(seed + 100) };
        for c in 0..256 {
            offset_diff += (x[c] != y[c]) as usize;
            let j = transact(&mut a.clone(), &mut jh, c as u8, &set);
            let d = compare(&x[c], &j);
            if d.shape || d.values {
                jitter_values += 1;
                jitter_bytes.insert(c as u8);
            }
            match timing(&a, &host(), c as u8, &set) {
                Some(phases) if !d.shape && matches!(j.end, End::Idle { .. } | End::Ignored) => {
                    let answers: Vec<u32> = j.steps.iter().map(|s| s.answer).collect();
                    if predict(&phases, &answers) == j.latencies() { model_ok += 1 } else { model_bad += 1 }
                }
                _ => model_none += 1,
            }
        }
        all.push(x);
    }
    let classes = mirror_classes(&all);
    let _ = writeln!(report, "{stem}: sweep of 256 command bytes, zero inputs and {sets} seeded sets of 64 words");
    let _ = writeln!(report, "byte  shape(zero inputs)  latencies(zero)  shapes over seeded sets");
    for c in 0..256 {
        let t = &zero[c];
        let shapes: std::collections::BTreeSet<String> = all[1..].iter().map(|r| shape_runs(&r[c].shape())).collect();
        let _ = writeln!(report, "{c:02X}  {:<18}  {:<28}  {}", shape_runs(&t.shape()), runs(&t.latencies()), shapes.into_iter().collect::<Vec<_>>().join(" "));
    }
    let _ = writeln!(report, "mirror classes (alike over every set):");
    for m in &classes {
        let _ = writeln!(report, "  {}", m.iter().map(|c| format!("{c:02X}")).collect::<Vec<_>>().join(" "));
    }
    let _ = writeln!(report, "bytes whose values or transfers change with the S-CPU's answer time: {}", jitter_bytes.iter().map(|c| format!("{c:02X}")).collect::<Vec<_>>().join(" "));
    let stalled = zero.iter().filter(|t| t.end == End::Stalled).count();
    let capped = zero.iter().filter(|t| t.end == End::Capped).count();
    let distinct = 256 - classes.iter().map(|m| m.len() - 1).sum::<usize>();
    let summary = format!(
        "{stem}: {distinct} distinct behaviours over 256 bytes, {} mirror classes; zero inputs: {stalled} stall, {capped} pass {} transfers; {} cases: {offset_diff} differ from a start 10,007 cycles later, {jitter_values} in values or transfers under a jittered S-CPU ({} bytes); timing model: {model_ok} predicted exactly, {model_bad} not, {model_none} not modelled",
        classes.len(),
        host().max_steps,
        sets * 256,
        jitter_bytes.len()
    );
    let _ = writeln!(report, "{summary}");
    std::fs::write(cache().join(format!("sweep-{stem}.txt")), report).unwrap();
    println!("{summary}");
}

fn sweep_st010(sets: u64) {
    let mut report = String::new();
    let a = idle("st010", 0, &mut host());
    let b = idle("st010", 10_007, &mut Host::jittered(1));
    let mut all: Vec<Vec<Mailbox>> = Vec::new();
    let mut phase = 0;
    let mut shift = std::collections::BTreeSet::new();
    for seed in 0..sets {
        let set = mail_inputs(seed);
        let x = sweep_mailbox(&a, &mut host(), &set);
        let y = sweep_mailbox(&b, &mut host(), &set);
        phase += x.iter().zip(&y).filter(|(p, q)| p.ram != q.ram || p.latency.is_some() != q.latency.is_some()).count();
        for (p, q) in x.iter().zip(&y) {
            if let (Some(l), Some(m)) = (p.latency, q.latency) {
                shift.insert(m as i64 - l as i64);
            }
        }
        all.push(x);
    }
    let mut classes: Vec<(Vec<(Option<u32>, Vec<u16>)>, Vec<u8>)> = Vec::new();
    for c in 0..256 {
        let k: Vec<(Option<u32>, Vec<u16>)> = all.iter().map(|r| (r[c].latency, r[c].ram.clone())).collect();
        match classes.iter_mut().find(|(x, _)| *x == k) {
            Some((_, m)) => m.push(c as u8),
            None => classes.push((k, vec![c as u8])),
        }
    }
    let _ = writeln!(report, "st010: mailbox sweep of 256 command bytes over {sets} seeded RAM fills");
    for c in 0..256usize {
        let set = mail_inputs(0);
        let r = &all[0][c];
        let changed = (0..r.ram.len()).filter(|&w| set.iter().find(|x| x.0 == w).map_or(0, |x| x.1) != r.ram[w]).count();
        let lat: std::collections::BTreeSet<Option<u32>> = all.iter().map(|s| s[c].latency).collect();
        let _ = writeln!(report, "{c:02X}  latency {:?}  RAM words changed (set 0) {changed}", lat);
    }
    let _ = writeln!(report, "classes:");
    for (_, m) in &classes {
        let _ = writeln!(report, "  {}", m.iter().map(|c| format!("{c:02X}")).collect::<Vec<_>>().join(" "));
    }
    let summary = format!("st010: {} distinct behaviours over 256 bytes; a start 10,007 cycles later over {} cases: {phase} differ in RAM, latency shifted by {:?}", classes.len(), sets * 256, shift);
    let _ = writeln!(report, "{summary}");
    std::fs::write(cache().join("sweep-st010.txt"), report).unwrap();
    println!("{summary}");
}

/// Each byte's random cases of its own shape on two chips, and random command sequences without a reset.
fn versus(sa: &str, sb: &str, cases: u64) {
    let (a, b) = (idle(sa, 0, &mut host()), idle(sb, 0, &mut host()));
    let mut report = String::new();
    let _ = writeln!(report, "{sa} against {sb}: {cases} seeded cases per command byte");
    let mut differing = Vec::new();
    for c in 0..=255u8 {
        let (mut shape, mut values, mut latency, mut status) = (0, 0, 0, 0);
        let mut p = Pcg::new(0xD5B1_0000 + c as u64);
        for _ in 0..cases {
            let set: Vec<u16> = (0..16).map(|_| p.word()).collect();
            let x = transact(&mut a.clone(), &mut host(), c, &set);
            let y = transact(&mut b.clone(), &mut host(), c, &set);
            let d = compare(&x, &y);
            shape += d.shape as u64;
            values += d.values as u64;
            latency += d.latency as u64;
            status += d.status as u64;
        }
        if shape + values + latency + status > 0 {
            let _ = writeln!(report, "{c:02X}  shape {shape}  values {values}  latency {latency}  SR {status}  of {cases}");
            differing.push(c);
        }
    }
    // Pairs: a command after a possible setter, both from 00h-3Fh, the second compared.
    let mut pairs: BTreeMap<u8, Vec<u8>> = BTreeMap::new();
    let base: Vec<u8> = (0..0x40u8).filter(|c| !differing.contains(c)).collect();
    for &first in &base {
        for &second in &base {
            let mut p = Pcg::new(0xA11 + ((first as u64) << 8) + second as u64);
            for _ in 0..(cases / 128).max(4) {
                let s1: Vec<u16> = (0..16).map(|_| p.word()).collect();
                let s2: Vec<u16> = (0..16).map(|_| p.word()).collect();
                let (mut ca, mut cb) = (a.clone(), b.clone());
                let (x1, y1) = (transact(&mut ca, &mut host(), first, &s1), transact(&mut cb, &mut host(), first, &s1));
                if !matches!(x1.end, End::Idle { .. }) || compare(&x1, &y1).any() {
                    break;
                }
                let (x, y) = (transact(&mut ca, &mut host(), second, &s2), transact(&mut cb, &mut host(), second, &s2));
                if compare(&x, &y).any() {
                    pairs.entry(first).or_default().push(second);
                    break;
                }
            }
        }
    }
    let (mut ca, mut cb) = (a.clone(), b.clone());
    let mut p = Pcg::new(0x5E9);
    let mut first = None;
    let mut ha = host();
    let mut hb = host();
    let commands: Vec<u8> = (0..=255u8).filter(|c| !differing.contains(c) && !pairs.contains_key(c)).collect();
    for n in 0..cases * 16 {
        let c = commands[p.next() as usize % commands.len()];
        let set: Vec<u16> = (0..16).map(|_| p.word()).collect();
        let (x, y) = (transact(&mut ca, &mut ha, c, &set), transact(&mut cb, &mut hb, c, &set));
        if compare(&x, &y).any() {
            first = Some((n, c));
            break;
        }
        if !matches!(x.end, End::Idle { .. }) {
            ca = a.clone();
            cb = b.clone();
        }
    }
    let _ = writeln!(report, "pairs whose second command differs, by first command:");
    for (f, v) in &pairs {
        let _ = writeln!(report, "  {f:02X}: {}", v.iter().map(|c| format!("{c:02X}")).collect::<Vec<_>>().join(" "));
    }
    let summary = format!(
        "{sa} against {sb}: {} of 256 bytes differ ({}); {} first commands from 00h-3Fh change a later one; sequences of the bytes in neither list, {} commands: {}",
        differing.len(),
        differing.iter().map(|c| format!("{c:02X}")).collect::<Vec<_>>().join(" "),
        pairs.len(),
        cases * 16,
        match first {
            None => "no difference".to_string(),
            Some((n, c)) => format!("first difference at command {n} ({c:02X})"),
        }
    );
    let _ = writeln!(report, "{summary}");
    std::fs::write(cache().join(format!("versus-{sa}-{sb}.txt")), report).unwrap();
    println!("{summary}");
}

/// Per command byte of a distinct behaviour, each phase's work and notice (`dsporacle::timing`) over seeded inputs: a
/// constant, or its range and count of values.
fn latency(stem: &str, cases: u64) {
    let a = idle(stem, 0, &mut host());
    let zero = sweep(&a, &mut host(), &[]);
    let classes = mirror_classes(&[zero.clone(), sweep(&a, &mut host(), &inputs(1, 64))]);
    let mut report = String::new();
    let _ = writeln!(report, "{stem}: per-phase work/notice in chip cycles over {cases} seeded cases of 16 words; 'a..b(n)' a range of n values, '-' work hidden by the fastest answer");
    let (mut constant, mut variable, mut unmodelled) = (0, 0, 0);
    for c in 0..=255u8 {
        if classes.iter().any(|m| m[0] != c && m.contains(&c)) {
            continue;
        }
        let mut per: BTreeMap<String, Vec<(std::collections::BTreeSet<u32>, std::collections::BTreeSet<u32>)>> = BTreeMap::new();
        let mut none = 0;
        let mut p = Pcg::new(0x1A7 + c as u64);
        for _ in 0..cases {
            let set: Vec<u16> = (0..16).map(|_| p.word()).collect();
            let Some(phases) = timing(&a, &host(), c, &set) else {
                none += 1;
                continue;
            };
            let shape = transact(&mut a.clone(), &mut host(), c, &set).shape();
            let e = per.entry(shape).or_insert_with(|| vec![Default::default(); phases.len()]);
            for (k, ph) in phases.iter().enumerate() {
                e[k].0.insert(ph.work);
                e[k].1.insert(ph.notice);
            }
        }
        let fmt = |v: &std::collections::BTreeSet<u32>| match v.len() {
            1 if v.contains(&0) => "-".to_string(),
            1 => v.iter().next().unwrap().to_string(),
            _ => format!("{}..{}({})", v.first().unwrap(), v.last().unwrap(), v.len()),
        };
        let mut line = format!("{c:02X}");
        if none > 0 {
            let _ = write!(line, "  unmodelled in {none} of {cases}");
            unmodelled += 1;
        }
        for (shape, phases) in &per {
            let all_const = phases.iter().all(|(w, n)| w.len() == 1 && n.len() == 1);
            if all_const { constant += 1 } else { variable += 1 }
            let cells: Vec<String> = phases.iter().map(|(w, n)| format!("{}/{}", fmt(w), fmt(n))).collect();
            let _ = write!(line, "  [{}] {}", shape_runs(shape), compress(&cells));
        }
        let _ = writeln!(report, "{line}");
    }
    let summary = format!("{stem}: {constant} command shapes with every phase constant, {variable} with a phase that varies, {unmodelled} bytes not always modelled");
    let _ = writeln!(report, "{summary}");
    std::fs::write(cache().join(format!("latency-{stem}.txt")), report).unwrap();
    println!("{summary}");
}

/// A command after another on two chips over seeded cases: how many of the second's runs differ, and how.
fn pair(sa: &str, sb: &str, first: u8, second: u8, cases: u64) {
    let (a, b) = (idle(sa, 0, &mut host()), idle(sb, 0, &mut host()));
    let mut p = Pcg::new(0xBA1 + ((first as u64) << 8) + second as u64);
    let mut d = Difference::default();
    let (mut n, mut differ) = (0, 0);
    let mut h = Host { max_steps: 64, ..host() };
    for _ in 0..cases {
        let s1: Vec<u16> = (0..16).map(|_| p.word()).collect();
        let s2: Vec<u16> = (0..16).map(|_| p.word()).collect();
        let (mut ca, mut cb) = (a.clone(), b.clone());
        let (x1, y1) = (transact(&mut ca, &mut h, first, &s1), transact(&mut cb, &mut h, first, &s1));
        if compare(&x1, &y1).any() {
            continue;
        }
        n += 1;
        let (x, y) = (transact(&mut ca, &mut h, second, &s2), transact(&mut cb, &mut h, second, &s2));
        let e = compare(&x, &y);
        if e.any() {
            differ += 1;
            d.shape |= e.shape;
            d.values |= e.values;
            d.latency |= e.latency;
            d.status |= e.status;
        }
    }
    println!("{sa} against {sb}: {first:02X} then {second:02X}: {differ} of {n} differ ({d:?})");
}

/// Cases a second for one command run back to back on one chip, the way an exhaustive pass would run them.
fn rate(stem: &str, command: u8, cases: u64) {
    let mut chip = idle(stem, 0, &mut host());
    let mut h = host();
    let mut p = Pcg::new(0x7A7E);
    let n = transact(&mut chip.clone(), &mut h, command, &[0; 16]).inputs();
    let start = std::time::Instant::now();
    let mut cycles = chip.dsp.cycles;
    for _ in 0..cases {
        let set: Vec<u16> = (0..n).map(|_| p.word()).collect();
        let t = transact(&mut chip, &mut h, command, &set);
        assert!(matches!(t.end, End::Idle { .. }));
    }
    let s = start.elapsed().as_secs_f64();
    cycles = chip.dsp.cycles - cycles;
    println!("{stem} {command:02X}: {cases} cases in {s:.2} s, {:.0} cases/s, {:.0} chip cycles a case, {:.1} M cycles/s", cases as f64 / s, cycles as f64 / cases as f64, cycles as f64 / s / 1e6);
}

fn compress(p: &[String]) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < p.len() {
        let mut j = i;
        while j + 1 < p.len() && p[j + 1] == p[i] {
            j += 1;
        }
        out.push(if j > i { format!("{}x{}", p[i], j - i + 1) } else { p[i].clone() });
        i = j + 1;
    }
    out.join(",")
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let num = |i: usize, d: u64| a.get(i).and_then(|v| v.parse().ok()).unwrap_or(d);
    match a.get(1).map(String::as_str) {
        Some("sweep") if a[2] == "st010" => sweep_st010(num(3, 4)),
        Some("sweep") => sweep_dr(&a[2], num(3, 4)),
        Some("versus") => versus(&a[2], &a[3], num(4, 256)),
        Some("pair") => pair(&a[2], &a[3], u8::from_str_radix(&a[4], 16).unwrap(), u8::from_str_radix(&a[5], 16).unwrap(), num(6, 4096)),
        Some("tables") => {
            let path = cache().join("dsp1.tables.bin");
            std::fs::write(&path, venusrt::chips::dsphle::tables_image()).unwrap();
            println!("{} bytes to {}", venusrt::chips::dsphle::tables_image().len(), path.display());
        }
        Some("rate") => rate(&a[2], u8::from_str_radix(&a[3], 16).unwrap(), num(4, 1 << 20)),
        Some("latency") => latency(&a[2], num(3, 256)),
        Some("phases") => {
            let set: Vec<u16> = a[4..].iter().map(|v| u16::from_str_radix(v, 16).unwrap()).collect();
            let chip = idle(&a[2], 0, &mut host());
            let t = transact(&mut chip.clone(), &mut host(), u8::from_str_radix(&a[3], 16).unwrap(), &set);
            match handshakes(&chip, &host(), u8::from_str_radix(&a[3], 16).unwrap(), &set, 30_000) {
                None => println!("{}: not described", shape_runs(&t.shape())),
                Some(h) => {
                    let cells: Vec<String> = h.iter().map(|p| if p.waits { format!("{}/{}", p.work, p.notice) } else { format!("{}!", p.work) }).collect();
                    println!("{}: {}", shape_runs(&t.shape()), compress(&cells));
                }
            }
        }
        Some("batch") => {
            // One case a line on stdin, its input words in hex; per line its outputs, end and phases.
            use std::io::BufRead;
            let chip = idle(&a[2], 0, &mut host());
            let command = u8::from_str_radix(&a[3], 16).unwrap();
            for line in std::io::stdin().lock().lines() {
                let set: Vec<u16> = line.unwrap().split_whitespace().map(|v| u16::from_str_radix(v, 16).unwrap()).collect();
                let t = transact(&mut chip.clone(), &mut host(), command, &set);
                let h = handshakes(&chip, &host(), command, &set, 30_000).map(|h| {
                    let cells: Vec<String> = h.iter().map(|p| if p.waits { format!("{}/{}", p.work, p.notice) } else { format!("{}!", p.work) }).collect();
                    compress(&cells)
                });
                let end = match t.end { End::Idle { value, .. } => format!("{value:04X}"), e => format!("{e:?}") };
                println!("{} | {} | {end} | {}", shape_runs(&t.shape()), t.outputs().iter().map(|v| format!("{v:02X}")).collect::<Vec<_>>().join(" "), h.unwrap_or_default());
            }
        }
        Some("mailbatch") => {
            // One ST010 case a line on stdin, `word=value` pairs in hex, each from the same idle chip; per line the cycles
            // and every word that changed.
            use std::io::BufRead;
            let chip = idle("st010", 0, &mut host());
            let command = u8::from_str_radix(&a[2], 16).unwrap();
            for line in std::io::stdin().lock().lines() {
                let set: Vec<(usize, u16)> = line.unwrap().split_whitespace().map(|kv| {
                    let (k, v) = kv.split_once('=').unwrap();
                    (usize::from_str_radix(k, 16).unwrap(), u16::from_str_radix(v, 16).unwrap())
                }).collect();
                let mut before = chip.dsp.ram.to_vec();
                for &(w, v) in &set {
                    before[w] = v;
                }
                let m = mailbox(&mut chip.clone(), &mut host(), command, &set);
                let changed: Vec<String> = (0..m.ram.len()).filter(|&w| w != 0x10 && m.ram[w] != before[w]).map(|w| format!("{w:03X}={:04X}", m.ram[w])).collect();
                println!("{} | {}", m.latency.map_or(-1, |l| l as i64), changed.join(" "));
            }
        }
        Some("mail") => {
            // An ST010 command through its mailbox, with RAM words set as word=value (hex); prints the cycles and every
            // word that changed. MAIL_FILL=seed fills the other words first.
            let mut chip = idle("st010", 0, &mut host());
            let mut fill: Vec<(usize, u16)> = match std::env::var("MAIL_FILL").ok().and_then(|v| v.parse().ok()) {
                Some(seed) => {
                    let mut p = Pcg::new(seed);
                    (0..0x800).map(|w| (w, p.word())).filter(|&(w, _)| w != 0x10).collect()
                }
                None => Vec::new(),
            };
            for part in a[2..].split(|w| w == "/") {
                let command = u8::from_str_radix(&part[0], 16).unwrap();
                let set: Vec<(usize, u16)> = part[1..].iter().map(|kv| {
                    let (k, v) = kv.split_once('=').unwrap();
                    (usize::from_str_radix(k, 16).unwrap(), u16::from_str_radix(v, 16).unwrap())
                }).collect();
                fill.extend(set);
                for _ in 0..std::env::var("MAIL_DELAY").ok().and_then(|v| v.parse().ok()).unwrap_or(0u32) {
                    chip.tick();
                }
                let before = { let mut c = chip.clone(); for &(w, v) in &fill { c.write(Port::Ram(w * 2), v as u8); c.write(Port::Ram(w * 2 + 1), (v >> 8) as u8); } c.dsp.ram.to_vec() };
                let m = mailbox(&mut chip, &mut host(), command, &fill);
                fill.clear();
                let changed: Vec<String> = (0..m.ram.len()).filter(|&w| w != 0x10 && m.ram[w] != before[w]).map(|w| format!("{w:03X}={:04X}", m.ram[w])).collect();
                println!("{command:02X}: {:?} cycles; {} words changed: {}", m.latency, changed.len(), changed.join(" "));
            }
        }
        Some("ask") => {
            let mut chip = idle(&a[2], 0, &mut host());
            for part in a[3..].split(|w| w == "/") {
                let set: Vec<u16> = part[1..].iter().map(|v| u16::from_str_radix(v, 16).unwrap()).collect();
                let t = transact(&mut chip, &mut host(), u8::from_str_radix(&part[0], 16).unwrap(), &set);
                let sr: Vec<String> = t.steps.iter().map(|s| format!("{:02X}/{:02X}", s.sr_rise, s.sr_access)).collect();
                println!("{} out {:04X?} lat {:?} end {:?} sr {}", shape_runs(&t.shape()), t.outputs(), t.latencies(), t.end, compress(&sr));
            }
        }
        _ => eprintln!("dsp_oracle sweep <chip> [sets] | versus <chip> <chip> [cases] | latency <chip> [cases]"),
    }
}
