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

fn main() {
    let args: Vec<String> = std::env::args().collect();
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
        Some("compare") => {
            let text = std::fs::read_to_string(&args[2]).expect("the bench's output");
            let mut lines = text.lines().filter(|l| !l.starts_with("calls"));
            for v in voices() {
                assert_eq!(lines.next(), Some(v.name.as_str()), "the bench's voices are these, in this order");
                let theirs: Vec<i64> = (0..v.samples).map(|_| lines.next().expect("a sample").parse().expect("a number")).collect();
                let ours = endou(&v);
                let best = (-8i64..=8)
                    .map(|lag| {
                        let pairs: Vec<(i64, i64)> = (0..v.samples as i64).filter_map(|i| Some((ours[i as usize], *theirs.get(usize::try_from(i + lag).ok()?)?))).collect();
                        let (sx, sy) = pairs.iter().fold((0f64, 0f64), |(a, b), &(x, y)| (a + (x * x) as f64, b + (x * y) as f64));
                        let scale = if sx == 0.0 { 1.0 } else { sy / sx };
                        let equal = pairs.iter().filter(|&&(x, y)| (x as f64 * scale).round() as i64 == y).count();
                        (equal, lag, scale)
                    })
                    .max_by_key(|t| t.0)
                    .expect("a lag");
                println!("{} lag {} scale {:.4} equal {} of {}", v.name, best.1, best.2, best.0, v.samples);
            }
        }
        _ => eprintln!("usage: opllref voices | compare <bench output>"),
    }
}
