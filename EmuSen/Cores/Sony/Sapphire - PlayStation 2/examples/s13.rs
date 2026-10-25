//! Stage S13's tools (Sapphire_Native.md §23): `s13 <folder>` writes each program as `<name>.elf`, its expected record
//! `<name>.expected` (one record word a line in hexadecimal, `-` where any value is expected) and its source; `s13
//! explain <folder> <referee>` reads `<referee>-<program>.txt`, `ps2referee.py ram`'s output for each, and lists each
//! result quadword the referee gives otherwise, by the case's note.

#[path = "../tests/s6/mod.rs"]
mod s6;
#[path = "../tests/s9/mod.rs"]
mod s9;
#[path = "../tests/s12/mod.rs"]
mod s12;
#[path = "../tests/s13/mod.rs"]
mod s13;

fn words_of(text: &str) -> Vec<u32> {
    let Some((_, rest)) = text.split_once("== RAM") else { return Vec::new() };
    rest.lines().skip(1).take_while(|l| !l.starts_with('{') && !l.starts_with("==")).flat_map(|l| l.split_whitespace().map(|w| u32::from_str_radix(w, 16).unwrap())).collect()
}

/// Each record quadword's note, in the record's order.
fn notes(segs: &[s13::Seg]) -> Vec<String> {
    segs.iter()
        .flat_map(|s| {
            let mut v: Vec<String> = s.notes.iter().map(|(_, n)| n.clone()).collect();
            if s.tpc.is_some() {
                v.push("TPC".into());
            }
            v
        })
        .collect()
}

fn explain(dir: &std::path::Path, referee: &str) {
    for (name, segs) in s13::all_segs() {
        let short = name.trim_start_matches("s13-");
        let Ok(text) = std::fs::read_to_string(dir.join(format!("{referee}-{short}.txt"))) else { continue };
        let got = words_of(&text);
        let notes = notes(&segs);
        let (_, expected) = s13::program(name, &segs).finish();
        let mut differ = Vec::new();
        for (q, note) in notes.iter().enumerate() {
            let at = 4 * q;
            let g: Vec<_> = (0..4).map(|i| got.get(at + i).copied()).collect();
            let w: Vec<_> = (0..4).map(|i| expected.get(at + i).copied().flatten()).collect();
            if (0..4).any(|i| w[i].is_some() && g[i] != w[i]) {
                let show = |v: &[Option<u32>]| v.iter().map(|x| x.map_or("-".to_string(), |x| format!("{x:08x}"))).collect::<Vec<_>>().join(" ");
                differ.push(format!("    {note}: {} (expected {})", show(&g), show(&w)));
            }
        }
        println!("== {name} on {referee}: {} results, {} differ", notes.len(), differ.len());
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
    for p in s13::all() {
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
