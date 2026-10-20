//! The YM2413's synthetic voices for the referee (Endou_Opll.md §7): `voices` prints them for the bench that drives
//! Nuked-OPLL through its header (`EmuSen.WiseMan/Reference/opll/build-opll.sh`); `compare <bench output>` runs Endou's
//! chip on the same writes and reports, a voice a line, the lag and scale that best fit the referee's samples and how many
//! samples are then equal. `cargo run --release --example opllref -- voices|compare <file>`.

use endou::opll::Opll;

/// A voice: its name, its length in samples, and its writes as (sample, address, data).
struct Voice {
    name: String,
    samples: u32,
    writes: Vec<(u32, u8, u8)>,
}

/// The original instrument at `fnum` and `block` on channel 1, keyed for `on` samples and released for `off`.
fn original(name: &str, patch: [u8; 8], fnum: u16, block: u8, on: u32, off: u32) -> Voice {
    let mut writes: Vec<(u32, u8, u8)> = patch.iter().enumerate().map(|(i, &v)| (0, i as u8, v)).collect();
    writes.extend([(0, 0x30, 0x00), (0, 0x10, fnum as u8), (1, 0x20, 0x10 | block << 1 | (fnum >> 8) as u8), (1 + on, 0x20, block << 1 | (fnum >> 8) as u8)]);
    Voice { name: name.into(), samples: 1 + on + off, writes }
}

fn voices() -> Vec<Voice> {
    spaced(voices_as_written())
}

fn voices_as_written() -> Vec<Voice> {
    let mut v = Vec::new();
    let sine = [0x21, 0x21, 0x3F, 0x00, 0x00, 0xF0, 0x0F, 0x0F];
    for (fnum, block) in [(0x120, 4), (0x1FF, 7), (0x0C0, 0), (0x155, 2)] {
        v.push(original(&format!("sine-{fnum:03x}-{block}"), sine, fnum, block, 6000, 2000));
    }
    for mul in 0..16u8 {
        v.push(original(&format!("mul-{mul}"), [0x21, 0x20 | mul, 0x3F, 0, 0, 0xF0, 0x0F, 0x0F], 0x120, 4, 3000, 500));
    }
    for tl in [0u8, 8, 16, 24, 32, 48] {
        v.push(original(&format!("mod-tl{tl}"), [0x21, 0x21, tl, 0, 0xF0, 0xF0, 0x0F, 0x0F], 0x120, 4, 3000, 500));
    }
    for fb in 1..8u8 {
        v.push(original(&format!("fb-{fb}"), [0x21, 0x21, 0x00, fb, 0xF0, 0xF0, 0x0F, 0x0F], 0x120, 4, 3000, 500));
    }
    v.push(original("half-both", [0x21, 0x21, 0x10, 0x18, 0xF0, 0xF0, 0x0F, 0x0F], 0x120, 4, 3000, 500));
    for ar in [1u8, 4, 8, 12, 14, 15] {
        v.push(original(&format!("attack-{ar}"), [0x21, 0x21, 0x3F, 0, 0, ar << 4, 0x0F, 0x0F], 0x120, 4, 40000, 500));
    }
    for dr in [1u8, 5, 9, 13, 15] {
        v.push(original(&format!("decay-{dr}"), [0x21, 0x01, 0x3F, 0, 0, 0xF0 | dr, 0x0F, 0xF0], 0x120, 4, 40000, 500));
    }
    for (eg, rr) in [(0x20u8, 4u8), (0x20, 10), (0x00, 4)] {
        v.push(original(&format!("release-{eg:02x}-{rr}"), [0x21, 0x01 | eg, 0x3F, 0, 0, 0xF0, 0x0F, 0x40 | rr], 0x120, 4, 4000, 40000));
    }
    for ksl in 1..4u8 {
        v.push(original(&format!("ksl-{ksl}"), [0x21, 0x21, 0x3F, ksl << 6, 0, 0xF0, 0x0F, 0x0F], 0x1A0, 7, 3000, 500));
    }
    v.push(original("ksr", [0x21, 0x31, 0x3F, 0, 0, 0x84, 0x0F, 0x24], 0x1A0, 7, 20000, 10000));
    v.push(original("am", [0x21, 0xA1, 0x3F, 0, 0, 0xF0, 0x0F, 0x0F], 0x120, 4, 30000, 500));
    v.push(original("vib", [0x21, 0x61, 0x3F, 0, 0, 0xF0, 0x0F, 0x0F], 0x120, 4, 30000, 500));
    for inst in 1..16u8 {
        let writes = vec![(0, 0x30, inst << 4), (0, 0x10, 0x20), (1, 0x20, 0x19), (20001, 0x20, 0x09)];
        v.push(Voice { name: format!("instrument-{inst}"), samples: 30000, writes });
    }
    let rhythm = vec![(0, 0x16, 0x20), (0, 0x17, 0x50), (0, 0x18, 0xC0), (0, 0x26, 0x05), (0, 0x27, 0x05), (0, 0x28, 0x01), (0, 0x36, 0x00), (0, 0x37, 0x00), (0, 0x38, 0x00)];
    for (bit, drum) in [(0x10u8, "bd"), (0x08, "sd"), (0x04, "tom"), (0x02, "tc"), (0x01, "hh")] {
        let mut writes = rhythm.clone();
        writes.extend([(1, 0x0E, 0x20 | bit), (12001, 0x0E, 0x20)]);
        v.push(Voice { name: format!("rhythm-{drum}"), samples: 20000, writes });
    }
    v
}

/// The writes spaced two samples apart at least, so that each keeps the chip's 84 clocks after its data (the manual's
/// Table II-2) on the bench as here.
fn spaced(mut v: Vec<Voice>) -> Vec<Voice> {
    for voice in &mut v {
        let mut next = 0;
        for w in &mut voice.writes {
            w.0 = w.0.max(next);
            next = w.0 + 2;
        }
        voice.samples = voice.samples.max(next + 1);
    }
    v
}

fn endou(voice: &Voice) -> Vec<i64> {
    let mut o = Opll::new();
    let mut k = 0;
    (0..voice.samples)
        .map(|s| {
            while k < voice.writes.len() && voice.writes[k].0 <= s {
                o.write_address(voice.writes[k].1);
                o.write_data(voice.writes[k].2);
                k += 1;
            }
            let (m, r) = o.sample();
            (m + r) as i64
        })
        .collect()
}

/// Voices as `voices` prints them, read back, for voices written by hand.
fn read_voices(path: &str) -> Vec<Voice> {
    let mut out = Vec::new();
    for line in std::fs::read_to_string(path).expect("the voices").lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        match f.first() {
            Some(&"voice") => out.push(Voice { name: f[1].into(), samples: f[2].parse().unwrap(), writes: Vec::new() }),
            Some(&"w") => out.last_mut().unwrap().writes.push((f[1].parse().unwrap(), f[2].parse().unwrap(), f[3].parse().unwrap())),
            _ => {}
        }
    }
    out
}

fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    let custom = args.iter().position(|a| a == "--voices").map(|i| { let p = args[i + 1].clone(); args.drain(i..i + 2); p });
    let voices = || custom.as_deref().map_or_else(voices, read_voices);
    match args.get(1).map(String::as_str) {
        Some("voices") => {
            for v in voices() {
                println!("voice {} {}", v.name, v.samples);
                for (s, a, d) in &v.writes {
                    println!("w {s} {a} {d}");
                }
                println!("end");
            }
        }
        Some("dump") => {
            // dump <voice> <slot>: a slot's output a sample, as the chip holds it after each sample
            let v = voices().into_iter().find(|v| v.name == args[2]).expect("a voice");
            let slot: usize = args[3].parse().expect("a slot");
            let mut o = Opll::new();
            let mut k = 0;
            for s in 0..v.samples {
                while k < v.writes.len() && v.writes[k].0 <= s {
                    o.write_address(v.writes[k].1);
                    o.write_data(v.writes[k].2);
                    k += 1;
                }
                o.sample();
                println!("{}", o.slots[slot].out[0]);
            }
        }
        Some("diff") => {
            // diff <bench output> <voice> [lag]: the samples where Endou and the referee differ, at the lag given (1 by default)
            let text = std::fs::read_to_string(&args[2]).expect("the bench's output");
            let lines: Vec<&str> = text.lines().filter(|l| !l.starts_with("calls")).collect();
            let at = lines.iter().position(|l| *l == args[3]).expect("the voice");
            let v = voices().into_iter().find(|v| v.name == args[3]).expect("a voice");
            let theirs: Vec<i64> = lines[at + 1..at + 1 + v.samples as usize].iter().map(|l| l.parse::<i64>().unwrap() - 36).collect();
            let ours = endou(&v);
            let lag: usize = args.get(4).map_or(1, |l| l.parse().unwrap());
            let mut shown = 0;
            for i in 0..ours.len().min(theirs.len() - lag) {
                if ours[i] != theirs[i + lag] && shown < 30 {
                    println!("{i}: ours {} theirs {}  (ours around {:?} theirs {:?})", ours[i], theirs[i + lag], &ours[i.saturating_sub(2)..(i + 3).min(ours.len())], &theirs[(i + lag).saturating_sub(2)..(i + lag + 3).min(theirs.len())]);
                    shown += 1;
                }
            }
        }
        Some("modprobe") => {
            // modprobe <bench output> <voice> <lag>: channel 1's carrier, the offsets near the modulator's output that give the referee's sample
            let text = std::fs::read_to_string(&args[2]).expect("the bench's output");
            let lines: Vec<&str> = text.lines().filter(|l| !l.starts_with("calls")).collect();
            let at = lines.iter().position(|l| *l == args[3]).expect("the voice");
            let v = voices().into_iter().find(|v| v.name == args[3]).expect("a voice");
            let theirs: Vec<i64> = lines[at + 1..at + 1 + v.samples as usize].iter().map(|l| l.parse::<i64>().unwrap() - 36).collect();
            let lag: usize = args[4].parse().unwrap();
            let mut o = Opll::new();
            let mut k = 0;
            for s in 0..v.samples.min(400) {
                while k < v.writes.len() && v.writes[k].0 <= s {
                    o.write_address(v.writes[k].1);
                    o.write_data(v.writes[k].2);
                    k += 1;
                }
                let used = o.slots[0].out[0] as i32;
                let phase = (o.slots[1].phase >> 9) as i32;
                let level = o.slots[1].level as u32;
                o.sample();
                if s > 30 {
                    let t = theirs[s as usize + lag];
                    let fits: Vec<i32> = (-12..=12).filter(|d| endou::opll::dac(endou::opll::operator(phase + used + d, level, false)) as i64 == t).collect();
                    println!("{s}: used {used} fits {:?}", fits);
                }
            }
        }
        Some("egprobe") => {
            // egprobe <bench output> <voice> <lag> <from> <to>: channel 1's carrier level here, and the levels that give the referee's sample
            let text = std::fs::read_to_string(&args[2]).expect("the bench's output");
            let lines: Vec<&str> = text.lines().filter(|l| !l.starts_with("calls")).collect();
            let at = lines.iter().position(|l| *l == args[3]).expect("the voice");
            let v = voices().into_iter().find(|v| v.name == args[3]).expect("a voice");
            let theirs: Vec<i64> = lines[at + 1..at + 1 + v.samples as usize].iter().map(|l| l.parse::<i64>().unwrap() - 36).collect();
            let lag: usize = args[4].parse().unwrap();
            let (from, to): (u32, u32) = (args[5].parse().unwrap(), args[6].parse().unwrap());
            let mut o = Opll::new();
            let mut k = 0;
            for s in 0..to {
                while k < v.writes.len() && v.writes[k].0 <= s {
                    o.write_address(v.writes[k].1);
                    o.write_data(v.writes[k].2);
                    k += 1;
                }
                let phase = (o.slots[1].phase >> 9) as i32;
                let level = o.slots[1].level;
                let stage = o.slots[1].stage;
                o.sample();
                if s >= from {
                    let t = theirs[s as usize + lag];
                    let fits: Vec<u32> = (0..128).filter(|&l| endou::opll::dac(endou::opll::operator(phase, l, false)) as i64 == t).collect();
                    let r = if fits.is_empty() { "none".to_string() } else { format!("{}-{}", fits[0], fits[fits.len() - 1]) };
                    println!("{s}: ours {level} {stage:?} theirs {r} (t {t})");
                }
            }
        }
        Some("whatif") => {
            // whatif <bench output> <voice> <from> <to> [s,slot,field,value]...: ours with slots overridden, against theirs
            let text = std::fs::read_to_string(&args[2]).expect("the bench's output");
            let lines: Vec<&str> = text.lines().filter(|l| !l.starts_with("calls")).collect();
            let at = lines.iter().position(|l| *l == args[3]).expect("the voice");
            let v = voices().into_iter().find(|v| v.name == args[3]).expect("a voice");
            let theirs: Vec<i64> = lines[at + 1..at + 1 + v.samples as usize].iter().map(|l| l.parse::<i64>().unwrap() - 36).collect();
            let (from, to): (u32, u32) = (args[4].parse().unwrap(), args[5].parse().unwrap());
            let ops: Vec<(u32, usize, String, u32)> = args[6..].iter().map(|a| { let f: Vec<&str> = a.split(',').collect(); (f[0].parse().unwrap(), f[1].parse().unwrap(), f[2].to_string(), f[3].parse().unwrap()) }).collect();
            let mut o = Opll::new();
            let mut k = 0;
            let mut out = Vec::new();
            for s in 0..to {
                while k < v.writes.len() && v.writes[k].0 <= s {
                    o.write_address(v.writes[k].1);
                    o.write_data(v.writes[k].2);
                    k += 1;
                }
                for (t, slot, field, value) in &ops {
                    if *t == s {
                        match field.as_str() {
                            "phase" => o.slots[*slot].phase = *value,
                            "level" => o.slots[*slot].level = *value as u8,
                            "addphase" => o.slots[*slot].phase = (o.slots[*slot].phase + *value) & 0x7FFFF,
                            _ => panic!("a field"),
                        }
                    }
                }
                let (m, r) = o.sample();
                if s >= from {
                    out.push(format!("{}/{}", m + r, theirs[s as usize + 1]));
                }
            }
            println!("{}", out.join(" "));
        }
        Some("trace") => {
            // trace <voice> <from> <to>: channel 0's two slots, level, stage and phase, before each sample
            let v = voices().into_iter().find(|v| v.name == args[2]).expect("a voice");
            let (from, to): (u32, u32) = (args[3].parse().unwrap(), args[4].parse().unwrap());
            let mut o = Opll::new();
            let mut k = 0;
            for s in 0..to {
                while k < v.writes.len() && v.writes[k].0 <= s {
                    o.write_address(v.writes[k].1);
                    o.write_data(v.writes[k].2);
                    k += 1;
                }
                let tc: usize = std::env::var("TRACE_CH").map(|v| v.parse().unwrap()).unwrap_or(0);
                let (a, b) = (o.slots[2 * tc], o.slots[2 * tc + 1]);
                let (m, r) = o.sample();
                if s >= from {
                    println!("{s}: mod {} {:?} {} car {} {:?} {} out {}", a.level, a.stage, a.phase >> 9, b.level, b.stage, b.phase >> 9, m + r);
                }
            }
        }
        Some("fit") => {
            // fit <bench output> <instrument> <patch hex>: the original-instrument bytes that best give the bench's instrument
            let text = std::fs::read_to_string(&args[2]).expect("the bench's output");
            let inst: u8 = args[3].parse().unwrap();
            let mut lines = text.lines().filter(|l| !l.starts_with("calls"));
            let set: Vec<(Voice, Vec<i64>)> = voices()
                .into_iter()
                .map(|v| {
                    assert_eq!(lines.next(), Some(v.name.as_str()));
                    let t = (0..v.samples).map(|_| lines.next().unwrap().parse::<i64>().unwrap() - 36).collect();
                    (v, t)
                })
                .collect();
            let sse = std::env::var("FIT_SSE").is_ok();
            let limit: usize = std::env::var("FIT_N").map(|v| v.parse().unwrap()).unwrap_or(usize::MAX);
            let score = |p: &[u8; 8]| -> i64 {
                set.iter()
                    .map(|(v, t)| {
                        let mut o = Opll::new();
                        if inst >= 16 {
                            o.rhythm_patches[inst as usize - 16] = *p;
                        } else {
                            o.regs[..8].copy_from_slice(p);
                        }
                        let mut k = 0;
                        let mut n = 0;
                        for s in 0..(v.samples as usize).min(limit) {
                            while k < v.writes.len() && v.writes[k].0 as usize <= s {
                                let (a, mut d) = (v.writes[k].1, v.writes[k].2);
                                if (0x30..0x39).contains(&a) && d >> 4 == inst {
                                    d &= 0x0F;
                                }
                                o.write_address(a);
                                o.write_data(d);
                                k += 1;
                            }
                            let (m, r) = o.sample();
                            if let Some(&x) = t.get(s + 1) {
                                let d = x - (m + r) as i64;
                                n += if sse { -d * d } else { (d == 0) as i64 };
                            }
                        }
                        n
                    })
                    .sum()
            };
            let mut p: [u8; 8] = (0..8).map(|i| u8::from_str_radix(&args[4][2 * i..2 * i + 2], 16).unwrap()).collect::<Vec<_>>().try_into().unwrap();
            let total: usize = set.iter().map(|(v, _)| v.samples as usize - 1).sum();
            let fields: [(usize, u8); 22] = [(0, 0x80), (0, 0x40), (0, 0x20), (0, 0x10), (0, 0x0F), (1, 0x80), (1, 0x40), (1, 0x20), (1, 0x10), (1, 0x0F), (2, 0xC0), (2, 0x3F), (3, 0xC0), (3, 0x10), (3, 0x08), (3, 0x07), (4, 0xF0), (4, 0x0F), (5, 0xF0), (5, 0x0F), (6, 0xF0), (6, 0x0F)];
            let fields: Vec<(usize, u8)> = fields.into_iter().chain([(7, 0xF0), (7, 0x0F)]).collect();
            let mut best = score(&p);
            let mut rng = 0x2545F4914F6CDD1Du64;
            let restarts: usize = std::env::var("FIT_RESTARTS").map(|v| v.parse().unwrap()).unwrap_or(0);
            let mut round = 0;
            loop {
                let before = best;
                for &(b, mask) in &fields {
                    let shift = mask.trailing_zeros();
                    for v in 0..=(mask >> shift) {
                        let mut q = p;
                        q[b] = (q[b] & !mask) | (v << shift);
                        if q != p {
                            let s = score(&q);
                            if s > best {
                                best = s;
                                p = q;
                            }
                        }
                    }
                }
                eprintln!("{} of {total}: {}", best, p.iter().map(|b| format!("{b:02X}")).collect::<String>());
                if best == before {
                    // A local best: try pairs of fields from a random kick, and keep any gain.
                    if round >= restarts {
                        break;
                    }
                    round += 1;
                    let mut q = p;
                    for _ in 0..3 {
                        rng ^= rng << 13;
                        rng ^= rng >> 7;
                        rng ^= rng << 17;
                        let (b, mask) = fields[(rng % fields.len() as u64) as usize];
                        q[b] = (q[b] & !mask) | ((rng >> 32) as u8 & mask);
                    }
                    let mut qb = score(&q);
                    loop {
                        let qbefore = qb;
                        for &(b, mask) in &fields {
                            let shift = mask.trailing_zeros();
                            for v in 0..=(mask >> shift) {
                                let mut r = q;
                                r[b] = (r[b] & !mask) | (v << shift);
                                if r != q {
                                    let sc = score(&r);
                                    if sc > qb {
                                        qb = sc;
                                        q = r;
                                    }
                                }
                            }
                        }
                        if qb == qbefore {
                            break;
                        }
                    }
                    if qb > best {
                        best = qb;
                        p = q;
                    }
                }
            }
            if std::env::var("FIT_PAIRS").is_ok() {
                let mut improved = true;
                while improved {
                    improved = false;
                    for i in 0..fields.len() {
                        for j in i + 1..fields.len() {
                            let ((bi, mi), (bj, mj)) = (fields[i], fields[j]);
                            let (si, sj) = (mi.trailing_zeros(), mj.trailing_zeros());
                            for vi in 0..=(mi >> si) {
                                for vj in 0..=(mj >> sj) {
                                    let mut q = p;
                                    q[bi] = (q[bi] & !mi) | (vi << si);
                                    q[bj] = (q[bj] & !mj) | (vj << sj);
                                    if q != p {
                                        let sc = score(&q);
                                        if sc > best {
                                            best = sc;
                                            p = q;
                                            improved = true;
                                            eprintln!("pair {} of {total}: {}", best, p.iter().map(|b| format!("{b:02X}")).collect::<String>());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            println!("{inst} {} {best} of {total}", p.iter().map(|b| format!("{b:02X}")).collect::<String>());
            // Where the best still differs: each voice's first differing sample, and how many differ.
            for (v, t) in &set {
                let mut o = Opll::new();
                if inst >= 16 {
                    o.rhythm_patches[inst as usize - 16] = p;
                } else {
                    o.regs[..8].copy_from_slice(&p);
                }
                let mut k = 0;
                let (mut first, mut n) = (None, 0);
                let mut shown = Vec::new();
                for s in 0..v.samples as usize {
                    while k < v.writes.len() && v.writes[k].0 as usize <= s {
                        let (a, mut d) = (v.writes[k].1, v.writes[k].2);
                        if (0x30..0x39).contains(&a) && d >> 4 == inst {
                            d &= 0x0F;
                        }
                        o.write_address(a);
                        o.write_data(d);
                        k += 1;
                    }
                    let (m, r) = o.sample();
                    if t.get(s + 1).is_some_and(|&x| x != (m + r) as i64) {
                        n += 1;
                        first.get_or_insert(s);
                        if shown.len() < 6 {
                            shown.push(format!("{s}:{}/{}", m + r, t[s + 1]));
                        }
                    }
                }
                println!("  {} differs {n}, first at {first:?}, key-off at {}: {}", v.name, v.writes.last().map_or(0, |w| w.0), shown.join(" "));
            }
        }
        Some("compare") => {
            let text = std::fs::read_to_string(&args[2]).expect("the bench's output");
            let mut lines = text.lines().filter(|l| !l.starts_with("calls"));
            for v in voices() {
                assert_eq!(lines.next(), Some(v.name.as_str()), "the bench's voices are these, in this order");
                let theirs: Vec<i64> = (0..v.samples).map(|_| lines.next().expect("a sample").parse().expect("a number")).collect();
                let ours = endou(&v);
                // The bench's sums hold eighteen turns of two pins, an idle turn 2: less 36, a silent chip is 0.
                let best = (-4i64..=4)
                    .map(|lag| {
                        let equal = (0..v.samples as i64).filter(|&i| usize::try_from(i + lag).ok().and_then(|j| theirs.get(j)).is_some_and(|&t| t - 36 == ours[i as usize])).count();
                        (equal, lag, 1.0)
                    })
                    .max_by_key(|t| t.0)
                    .expect("a lag");
                println!("{} lag {} equal {} of {}", v.name, best.1, best.0, v.samples);
            }
        }
        _ => eprintln!("usage: opllref voices | compare <bench output>"),
    }
}
