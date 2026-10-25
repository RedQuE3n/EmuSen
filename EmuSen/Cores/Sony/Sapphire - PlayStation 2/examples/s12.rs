//! Stage S12's tools (Sapphire_Native.md §21.7): `s12 <folder>` writes each program as `<name>.elf`, its expected record
//! `<name>.expected` (one record word a line in hexadecimal, `-` where any value is expected) and its source;
//! `s12 explain <folder> <referee>` reads `<referee>-<program>.txt`, `ps2referee.py ram`'s output for each, and sorts
//! the FMAC and divider lanes where the referee differs by the class of their inputs, and the flags by bit.

#[path = "../tests/s6/mod.rs"]
mod s6;
#[path = "../tests/s9/mod.rs"]
mod s9;
#[path = "../tests/s12/mod.rs"]
mod s12;

use std::collections::BTreeMap;

use sapphire::float;

fn words_of(text: &str) -> Vec<u32> {
    let Some((_, rest)) = text.split_once("== RAM") else { return Vec::new() };
    rest.lines().skip(1).take_while(|l| !l.starts_with('{') && !l.starts_with("==")).flat_map(|l| l.split_whitespace().map(|w| u32::from_str_radix(w, 16).unwrap())).collect()
}

fn exp255(x: u32) -> bool {
    x >> 23 & 255 == 255
}
fn denormal(x: u32) -> bool {
    x >> 23 & 255 == 0 && x & 0x7F_FFFF != 0
}

/// The class of a lane the referee computes differently.
fn class(a: u32, y: u32, acc: u32, fused: bool, want: u32, got: u32) -> &'static str {
    let ins = [a, y].into_iter().chain(fused.then_some(acc));
    if ins.clone().any(exp255) || exp255(want) || exp255(got) {
        "exponent 255 (input or result)"
    } else if ins.clone().any(denormal) {
        "a would-be denormal input"
    } else if want & 0x7FFF_FFFF == 0x7FFF_FFFF || got & 0x7FFF_FFFF == 0x7FFF_FFFF {
        "overflow"
    } else if float::is_zero(want) != float::is_zero(got) {
        "underflow or zero"
    } else if want ^ got == 0x8000_0000 {
        "sign of a zero"
    } else if want.abs_diff(got) == 1 {
        "last bit"
    } else {
        "other"
    }
}

const STATUS: [&str; 12] = ["Z", "S", "U", "O", "I", "D", "ZS", "SS", "US", "OS", "IS", "DS"];

fn explain(dir: &std::path::Path, referee: &str) {
    for g in s12::all_gens() {
        let name = g.p.name;
        let short = name.trim_start_matches("s12-").to_string();
        let Ok(text) = std::fs::read_to_string(dir.join(format!("{referee}-{short}.txt"))) else { continue };
        let got = words_of(&text);
        let notes = g.notes;
        let (_, expected) = g.p.finish();
        let mut lanes: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut flags: BTreeMap<String, usize> = BTreeMap::new();
        let mut cases = 0;
        for n in &notes {
            let w = |i: usize| (got.get(n.at + i).copied(), expected.get(n.at + i).copied().flatten());
            let is_div = matches!(n.what.as_str(), "vdiv" | "vrsqrt" | "vsqrt");
            let acc_case = n.what.ends_with(" acc");
            let fused = n.what.starts_with("vmadd") || n.what.starts_with("vmsub");
            let (value_at, status_at, mac_at) = if is_div { (0, 2, None) } else if acc_case { (4, 0, Some(2)) } else { (0, 4, Some(6)) };
            cases += 1;
            for &(lane, a, y, acc) in &n.lanes {
                if let (Some(gv), Some(wv)) = w(value_at + if is_div { 0 } else { lane })
                    && let Ok(path) = std::env::var("S12_DUMP")
                {
                    use std::io::Write;
                    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path).expect("the dump");
                    writeln!(f, "{}\t{lane}\t{a:08x}\t{y:08x}\t{acc:08x}\t{wv:08x}\t{gv:08x}", n.what).expect("written");
                }
                if let (Some(gv), Some(wv)) = w(value_at + if is_div { 0 } else { lane }) && gv != wv {
                    let c = class(a, y, acc, fused, wv, gv);
                    let op = n.what.split(['.', ' ']).next().unwrap().to_string();
                    lanes.entry(format!("{op}: {c}")).or_default().push(format!("{} ({acc:08x}) {a:08x}, {y:08x}: {gv:08x}, expected {wv:08x}", n.what));
                }
            }
            let clean = n.lanes.iter().all(|&(_, a, y, acc)| ![a, y, acc].iter().any(|&x| exp255(x) || denormal(x)));
            if std::env::var("S12_FLAGS").is_ok() && clean {
                let (gs, ws) = w(status_at);
                let (gm, wm) = mac_at.map_or((None, None), w);
                if gs != ws || gm != wm {
                    println!("    {} {:x?}: status {gs:x?} want {ws:x?}, MAC {gm:x?} want {wm:x?}", n.what, n.lanes);
                }
            }
            for (at, names) in [(Some(status_at), &STATUS[..]), (mac_at, &[][..])] {
                let Some(at) = at else { continue };
                if let (Some(gv), Some(wv)) = w(at) {
                    let d = gv ^ wv;
                    for b in 0..16 {
                        if d >> b & 1 == 1 {
                            let name = names.get(b).map_or_else(|| format!("MAC bit {b} ({}{})", ["Z", "S", "U", "O"][b / 4], ["w", "z", "y", "x"][b % 4]), |s| format!("status {s}"));
                            let sense = if gv >> b & 1 == 1 { "set, expected clear" } else { "clear, expected set" };
                            *flags.entry(format!("{} {name}: {sense}", n.what.split(['.', ' ']).next().unwrap())).or_default() += 1;
                        }
                    }
                }
            }
        }
        println!("== {name} on {referee}: {cases} cases");
        for (k, v) in &lanes {
            println!("  {k}: {} lanes, e.g. {}", v.len(), v.iter().take(2).cloned().collect::<Vec<_>>().join("; "));
        }
        for (k, v) in &flags {
            println!("  {k}: {v}");
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("explain") {
        return explain(std::path::Path::new(&args[1]), &args[2]);
    }
    let dir = std::path::PathBuf::from(args.first().expect("a folder"));
    std::fs::create_dir_all(&dir).expect("the folder");
    for p in s12::all() {
        let name = p.name;
        let (source, expected) = p.finish();
        let words = emusen_mips::assemble(&source, s6::ORIGIN, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        std::fs::write(dir.join(format!("{name}.elf")), sapphire::media::program(&words, s6::ORIGIN)).expect("written");
        let text: String = expected.iter().map(|w| w.map_or("-\n".to_string(), |w| format!("{w:08x}\n"))).collect();
        std::fs::write(dir.join(format!("{name}.expected")), text).expect("written");
        std::fs::write(dir.join(format!("{name}.s")), &source).expect("written");
        let graded = expected.iter().filter(|w| w.is_some()).count();
        println!("{name}: {} words of code and data, {} record words, {graded} graded", words.len(), expected.len());
    }
}
