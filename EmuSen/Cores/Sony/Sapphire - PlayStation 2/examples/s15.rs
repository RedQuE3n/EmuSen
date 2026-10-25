//! Stage S15's tools (Sapphire_Native.md §33): `s15 <folder>` writes each program as `<name>.elf`, its expected record
//! `<name>.expected` (one record word a line in hexadecimal, `-` where any value is expected) and its source; `s15
//! explain <folder> <referee>` reads `<referee>-<program>.txt`, `ps2referee.py ram`'s output for each, and lists each
//! record (a doubleword) the referee gives otherwise.

#[path = "../tests/s6/mod.rs"]
mod s6;
#[path = "../tests/s11/mod.rs"]
mod s11;
#[path = "../tests/s14/mod.rs"]
mod s14;
#[path = "../tests/s15/mod.rs"]
mod s15;
#[path = "../tests/s16/mod.rs"]
mod s16;
#[path = "../tests/s17/mod.rs"]
mod s17;

fn words_of(text: &str) -> Vec<u32> {
    let Some((_, rest)) = text.split_once("== RAM") else { return Vec::new() };
    rest.lines().skip(1).take_while(|l| !l.starts_with('{') && !l.starts_with("==")).flat_map(|l| l.split_whitespace().map(|w| u32::from_str_radix(w, 16).unwrap())).collect()
}

fn explain(dir: &std::path::Path, referee: &str) {
    for p in s15::all().into_iter().chain((0..4).map(s15::second_direct)) {
        let name = p.name;
        let short = name.trim_start_matches("s15-");
        let Ok(text) = std::fs::read_to_string(dir.join(format!("{referee}-{short}.txt"))) else { continue };
        let got = words_of(&text);
        let (_, expected) = p.finish();
        let mut differ = Vec::new();
        for (i, w) in expected.iter().enumerate().step_by(2) {
            let g = got.get(i).copied();
            if let Some(w) = w
                && g != Some(*w)
            {
                differ.push(format!("    record {}: {} (expected {w:08x})", i / 2, g.map_or("none".to_string(), |g| format!("{g:08x}"))));
            }
        }
        println!("== {name} on {referee}: {} records, {} differ", expected.len() / 2, differ.len());
        for d in differ {
            println!("{d}");
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
    for p in s15::all().into_iter().chain((0..4).map(s15::second_direct)) {
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
