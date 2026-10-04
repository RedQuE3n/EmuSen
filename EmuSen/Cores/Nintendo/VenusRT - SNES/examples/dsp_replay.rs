//! The trace oracle (VenusRT_DspHle.md §4.2): a game's recorded DSP-1 transactions (dsp_trace) replayed on the
//! replacement in order, each compared with the image's recorded results. `dsp_replay <chip> <trace>`; per command, the
//! transactions, those whose results are all close (within 2, or 1/128 of the image's value; angles the short way
//! round for Gyrate) and all exact, and the largest difference. Counts only.
use std::collections::BTreeMap;
use venusrt::chips::dsphle::Program;
use venusrt::chips::dsporacle::*;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let program = Program::for_stem(&a[1]).expect("a DSP-1 program");
    let text = std::fs::read_to_string(&a[2]).expect("a trace");
    let mut hle = Hle::new(program);
    power_on(&mut hle, &Host::steady()).unwrap();
    let list = |s: &str| -> Vec<u16> { s.split(',').filter_map(|w| u16::from_str_radix(w.trim(), 16).ok()).collect() };
    // Per command: transactions, all close, all exact, largest difference.
    let mut t: BTreeMap<u8, [i64; 4]> = BTreeMap::new();
    // Per command: the replacement's longest latency in a transaction less the trace's, over transactions.
    let mut lag: BTreeMap<u8, Vec<i64>> = BTreeMap::new();
    for line in text.lines() {
        let mut words = line.split_whitespace();
        let (Some(_), Some(c)) = (words.next(), words.next()) else { continue };
        let Ok(command) = u8::from_str_radix(c, 16) else { continue };
        let (Some(i), Some(o)) = (line.find("in ["), line.find("out [")) else { continue };
        let lat: Vec<i64> = line.find("lat [").map(|l| line[l + 5..line[l..].find(']').unwrap() + l].split(',').filter_map(|w| w.trim().parse().ok()).collect()).unwrap_or_default();
        let ins = list(&line[i + 4..line[i..].find(']').unwrap() + i]);
        let outs = list(&line[o + 5..line[o..].find(']').unwrap() + o]);
        if command >= 0x40 {
            continue;
        }
        let run = command & 0xCF == 0x0A;
        // A raster run: the lines the game read, then the terminator over the next line's last result.
        let (host, image) = if run {
            let lines = outs.len().div_ceil(4).saturating_sub(1);
            (Host { write_from: 1 + 4 * lines, filler: 0x8000, ..Host::steady() }, outs[..4 * lines].to_vec())
        } else {
            (Host::steady(), outs.clone())
        };
        let got = transact(&mut hle, &mut host.clone(), command, &ins);
        if !matches!(got.end, End::Idle { .. }) {
            hle = Hle::new(program);
            power_on(&mut hle, &Host::steady()).unwrap();
        }
        let mine = got.outputs();
        if !run && !lat.is_empty() {
            let longest = got.latencies().iter().map(|&v| v as i64).max().unwrap_or(0);
            lag.entry(command).or_default().push(longest - lat.iter().copied().max().unwrap_or(0));
        }
        let angles = command == 0x14;
        let diff = |x: u16, y: u16| if angles { (x.wrapping_sub(y) as i16 as i64).abs() } else { (x as i16 as i64 - y as i16 as i64).abs() };
        let close = |x: u16, y: u16| diff(x, y) <= if angles { 2 } else { 2.max((x as i16 as i64).abs() / 128) };
        let e = t.entry(command).or_insert([0; 4]);
        e[0] += 1;
        e[1] += (mine.len() >= image.len() && image.iter().zip(&mine).all(|(&x, &y)| close(x, y))) as i64;
        e[2] += (mine.len() >= image.len() && image.iter().zip(&mine).all(|(&x, &y)| x == y)) as i64;
        e[3] = e[3].max(image.iter().zip(&mine).map(|(&x, &y)| diff(x, y)).max().unwrap_or(0));
    }
    let name = std::path::Path::new(&a[2]).file_stem().unwrap().to_string_lossy().into_owned();
    for (c, r) in t {
        let mut l = lag.remove(&c).unwrap_or_default();
        l.sort();
        let timing = if l.is_empty() { String::new() } else { format!("; latency against the trace: median {:+}, from {:+} to {:+}", l[l.len() / 2], l[0], l[l.len() - 1]) };
        println!("{name} {c:02X}: {} transactions, all close {}, all exact {}, largest {}{timing}", r[0], r[1], r[2], r[3]);
    }
}
