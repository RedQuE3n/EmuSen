//! Stage S27's tools (Sapphire_Native.md §34.10): `s27 <folder>` writes each program as `<name>.elf` with its expected
//! record `<name>.expected` (one record word a line in hexadecimal, `-` where any value is taken); `s27 explain <folder>
//! <referee>` reads `<referee>-<name>.txt`, `ps2referee.py ram`'s output for each, and lists by command where the
//! referee's record differs, with the EE cycles each command took there.

#[path = "../tests/s27/mod.rs"]
mod s27;

fn words_of(text: &str) -> Vec<u32> {
    let Some((_, rest)) = text.split_once("== RAM") else { return Vec::new() };
    rest.lines().skip(1).take_while(|l| !l.starts_with('{') && !l.starts_with("==")).flat_map(|l| l.split_whitespace().map(|w| u32::from_str_radix(w, 16).unwrap())).collect()
}

fn explain(dir: &std::path::Path, referee: &str) {
    for p in s27::programs() {
        let Ok(text) = std::fs::read_to_string(dir.join(format!("{referee}-{}.txt", p.name))) else { continue };
        let got = words_of(&text);
        let mut differ = 0;
        let mut lines = Vec::new();
        let (mut at, mut cmd) = (0usize, 0usize);
        let mut cycles = Vec::new();
        while at < p.expected.len() {
            let out = p.out[cmd];
            let span = 8 + 4 * out;
            let bad: Vec<usize> = (at..at + span).filter(|&i| p.expected[i].is_some() && got.get(i).copied() != p.expected[i]).collect();
            cycles.push(got.get(at + 4).copied().unwrap_or(0));
            if !bad.is_empty() {
                differ += 1;
                if lines.len() < 12 {
                    let i = bad[0];
                    lines.push(format!(
                        "    command {cmd} ({:08x}): {} words differ, first word {} ({}): {} expected {:08x}",
                        p.words[cmd],
                        bad.len(),
                        i - at,
                        if i - at < 4 { "registers" } else { "output" },
                        got.get(i).map_or("none".into(), |g| format!("{g:08x}")),
                        p.expected[i].unwrap()
                    ));
                }
            }
            at += span;
            cmd += 1;
        }
        let total: u64 = cycles.iter().map(|&c| u64::from(c)).sum();
        println!("== {} on {referee}: {} commands, {differ} differ; {total} EE cycles in commands", p.name, p.commands);
        println!("    output lanes by difference (referee - expected): {}", lanes(&p, &got));
        for l in lines {
            println!("{l}");
        }
    }
}

/// The output's lanes (BDEC's 16-bit samples, RGB32's bytes, RGB16's 5-bit fields) by their difference from the
/// expected, as "difference:count" pairs; a lane of a command whose registers already differ is counted all the same.
fn lanes(p: &s27::program::Program, got: &[u32]) -> String {
    let mut hist = std::collections::BTreeMap::<i32, usize>::new();
    let mut at = 0;
    for (cmd, &out) in p.out.iter().enumerate() {
        let w = p.words[cmd];
        for q in 0..out {
            for k in 0..4 {
                let i = at + 8 + 4 * q + k;
                let (Some(e), Some(&g)) = (p.expected[i], got.get(i)) else { continue };
                let pairs: Vec<(i32, i32)> = match w >> 28 {
                    2 => (0..2).map(|j| ((e >> (16 * j)) as u16 as i16 as i32, (g >> (16 * j)) as u16 as i16 as i32)).collect(),
                    1 | 7 if w >> 27 & 1 == 0 => (0..4).map(|j| ((e >> (8 * j) & 0xFF) as i32, (g >> (8 * j) & 0xFF) as i32)).collect(),
                    _ => (0..2).flat_map(|j| (0..4).map(move |f| (j, f))).map(|(j, f)| {
                        let (eh, gh) = (e >> (16 * j) & 0xFFFF, g >> (16 * j) & 0xFFFF);
                        if f == 3 { ((eh >> 15) as i32, (gh >> 15) as i32) } else { ((eh >> (5 * f) & 31) as i32, (gh >> (5 * f) & 31) as i32) }
                    }).collect(),
                };
                for (e, g) in pairs {
                    *hist.entry((g - e).clamp(-9, 9)).or_default() += 1;
                }
            }
        }
        at += 8 + 4 * out;
    }
    hist.iter().map(|(d, n)| format!("{d}:{n}")).collect::<Vec<_>>().join(" ")
}

/// For an IDEC program of RGB32 output: each pixel's Y, Cb and Cr, the expected RGBA and the referee's, for the first
/// `n` pixels that differ, so that a formula can be fitted to the referee's conversion.
fn pixels(dir: &std::path::Path, referee: &str, stream: &str, n: usize) {
    let s = s27::corpus().into_iter().find(|s| s.name == stream).expect("a stream");
    let bytes = s.complete();
    let d = s27::standard::decode(&bytes);
    let name = format!("s27-idec-{stream}");
    let p = s27::programs().into_iter().find(|p| p.name == name).expect("the program");
    let got = words_of(&std::fs::read_to_string(dir.join(format!("{referee}-{name}.txt"))).expect("the referee's record"));
    let mut slices = d.events.iter().filter_map(|e| if let s27::standard::Event::Slice(s) = e { Some(s) } else { None });
    let mut at = 0;
    let mut shown = 0;
    for (cmd, &out) in p.out.iter().enumerate() {
        if p.words[cmd] >> 28 == 1 {
            let slice = slices.next().unwrap();
            for (k, mb) in slice.mbs.iter().enumerate() {
                let raw = s27::manual::raw8(&mb.blocks, mb.dct_type);
                for px in 0..256 {
                    let i = at + 8 + 256 * k + px;
                    let (x, y) = (px % 16, px / 16);
                    let c = 8 * (y / 2) + x / 2;
                    let (e, g) = (p.expected[i].unwrap(), got.get(i).copied().unwrap_or(0));
                    if e != g && shown < n {
                        shown += 1;
                        println!("Y {:3} Cb {:3} Cr {:3}  expected {:08x}  {referee} {:08x}", raw[px], raw[256 + c], raw[320 + c], e, g);
                    }
                }
            }
        }
        at += 8 + 4 * out;
    }
}

/// The post program's first CSC (RGB32, no thresholds): every pixel as "Y Cb Cr expected referee", for fitting.
fn csc(dir: &std::path::Path, referee: &str) {
    let got = words_of(&std::fs::read_to_string(dir.join(format!("{referee}-s27-post.txt"))).expect("the referee's record"));
    let p = s27::programs().into_iter().find(|p| p.name == "s27-post").unwrap();
    let at = 8 * 2 + 8;
    for (k, m) in s27::raw8_inputs().iter().enumerate() {
        for px in 0..256 {
            let (x, y) = (px % 16, px / 16);
            let c = 8 * (y / 2) + x / 2;
            let i = at + 256 * k + px;
            println!("{} {} {} {:08x} {:08x}", m[px], m[256 + c], m[320 + c], p.expected[i].unwrap(), got[i]);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("csc") {
        return csc(std::path::Path::new(&args[1]), &args[2]);
    }
    if args.first().map(String::as_str) == Some("pixels") {
        return pixels(std::path::Path::new(&args[1]), &args[2], &args[3], args[4].parse().unwrap());
    }
    if args.first().map(String::as_str) == Some("explain") {
        return explain(std::path::Path::new(&args[1]), &args[2]);
    }
    let dir = std::path::PathBuf::from(args.first().expect("a folder"));
    std::fs::create_dir_all(&dir).expect("the folder");
    for p in s27::programs() {
        std::fs::write(dir.join(format!("{}.elf", p.name)), sapphire::media::program(&p.image, s27::program::ORIGIN)).expect("written");
        let text: String = p.expected.iter().map(|w| w.map_or("-\n".to_string(), |w| format!("{w:08x}\n"))).collect();
        std::fs::write(dir.join(format!("{}.expected", p.name)), text).expect("written");
        let graded = p.expected.iter().filter(|w| w.is_some()).count();
        println!("{}: {} commands, {} words of image, {} record words, {graded} graded", p.name, p.commands, p.image.len(), p.expected.len());
    }
}
