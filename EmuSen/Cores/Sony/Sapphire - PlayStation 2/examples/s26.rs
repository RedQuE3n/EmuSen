//! Stage S26's programs for the referees (Sapphire_Native.md §32): `s26 <folder>` makes `<name>.elf`, the EE program
//! that places the IOP program and copies its record (`s21::wrap`), and `<name>.expected`, the done word and the
//! record, one word a line in hexadecimal, `-` where any value is expected, for the gate programs and the measurement
//! programs; `s26 grade <name> <run>` grades a referee's run of a gate program, the text `ps2referee.py ram` printed,
//! word by word and check by check (`tests/s26/gate.rs`).

#[path = "../tests/s21/mod.rs"]
mod s21;
#[path = "../tests/s26/mod.rs"]
mod s26;

fn write(dir: &std::path::Path, p: s21::Program) {
    let name = p.name;
    let (source, expected) = p.finish();
    let iop = s21::assemble(&source);
    let ee = s21::wrap(&iop, expected.len());
    let words = emusen_mips::assemble(&ee, s21::EE_ORIGIN, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{name}: {e:?}"));
    std::fs::write(dir.join(format!("{name}.elf")), sapphire::media::program(&words, s21::EE_ORIGIN)).expect("written");
    let text: String = std::iter::once(Some(s21::DONE)).chain(expected.iter().copied()).map(|w| w.map_or("-\n".to_string(), |w| format!("{w:08x}\n"))).collect();
    std::fs::write(dir.join(format!("{name}.expected")), text).expect("written");
    let graded = expected.iter().filter(|w| w.is_some()).count();
    println!("{name}: {} IOP words, {} record words, {graded} graded", iop.len(), expected.len());
}

fn grade(name: &str, run: &str) {
    let g = s26::gate::all().into_iter().find(|g| g.program.name == name).expect("a gate program of that name");
    let (program, checks) = g.finish();
    let (_, expected) = program.finish();
    let text = std::fs::read_to_string(run).expect("the run's text");
    let words: Vec<u32> = text
        .lines()
        .filter(|l| !l.starts_with('{') && !l.starts_with("==") && !l.trim().is_empty())
        .flat_map(|l| l.split_whitespace().map(|w| u32::from_str_radix(w, 16).expect("a word")).collect::<Vec<_>>())
        .collect();
    assert_eq!(words.first(), Some(&s21::DONE), "the done word first");
    let record = &words[1..];
    assert!(record.len() >= expected.len(), "{} words of {}", record.len(), expected.len());
    let mut wrong = 0;
    for (i, w) in expected.iter().enumerate() {
        if let Some(w) = w
            && *w != record[i]
        {
            println!("word {i}: {:08x}, expected {w:08x}", record[i]);
            wrong += 1;
        }
    }
    let bad = s26::gate::grade(&checks, record);
    for b in &bad {
        println!("check {b}");
    }
    let graded = expected.iter().filter(|w| w.is_some()).count();
    println!("{name}: {} of {graded} graded words and {} of {} checks equal", graded - wrong, checks.len() - bad.len(), checks.len());
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("grade") {
        return grade(&args[1], &args[2]);
    }
    let dir = std::path::PathBuf::from(args.first().expect("a folder"));
    std::fs::create_dir_all(&dir).expect("the folder");
    for g in s26::gate::all() {
        write(&dir, g.program);
    }
    for p in s26::measures() {
        write(&dir, p);
    }
}
