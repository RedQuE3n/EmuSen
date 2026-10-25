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
    let words = run_words(run);
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

/// A run's record words from the text `ps2referee.py ram` printed, the done word first.
fn run_words(run: &str) -> Vec<u32> {
    std::fs::read_to_string(run)
        .expect("the run's text")
        .lines()
        .filter(|l| !l.starts_with('{') && !l.starts_with("==") && !l.trim().is_empty())
        .flat_map(|l| l.split_whitespace().map(|w| u32::from_str_radix(w, 16).expect("a word")).collect::<Vec<_>>())
        .collect()
}

/// For a check that failed, the alignment of the copy with its record that agrees best: its equal samples, the largest
/// difference, and the first differences, so a dispute can say how the referee departs.
fn diff(name: &str, run: &str) {
    let g = s26::gate::all().into_iter().find(|g| g.program.name == name).expect("a gate program of that name");
    let (_, checks) = g.finish();
    let words = run_words(run);
    let record = &words[1..];
    for c in &checks {
        let h = s26::gate::halves(&record[c.at..c.at + c.halves / 2]);
        let s26::gate::What::Window(model) = &c.what else { continue };
        if s26::gate::window(&h, model).is_some() {
            continue;
        }
        let n = h.len();
        let mut best = (0usize, 0usize, 0usize, i32::MAX);
        for r in 0..n {
            for o in 0..model.len().saturating_sub(n) {
                if (0..8).any(|j| (h[(r + j) % n] as i32 - model[o + j] as i32).abs() > 1024) {
                    continue;
                }
                let eq = (0..n).filter(|&j| h[(r + j) % n] == model[o + j]).count();
                let worst = (0..n).map(|j| (h[(r + j) % n] as i32 - model[o + j] as i32).abs()).max().unwrap_or(0);
                if eq > best.0 || (eq == best.0 && worst < best.3) {
                    best = (eq, r, o, worst);
                }
            }
        }
        let (eq, r, o, worst) = best;
        let firsts: Vec<String> = (0..n).filter(|&j| h[(r + j) % n] != model[o + j]).take(8).map(|j| format!("{}:{}/{}", j, h[(r + j) % n], model[o + j])).collect();
        println!("{}: best {eq} of {n} equal, largest difference {worst}, at record sample {o}; first (index:referee/record) {}", c.name, firsts.join(" "));
    }
}

/// Runs a gate program on Sapphire and prints its record as `ps2referee.py ram` prints a referee's, so that `grade`
/// and `diff` read either.
fn sapphire(name: &str) {
    let g = s26::gate::all().into_iter().find(|g| g.program.name == name).expect("a gate program of that name");
    let (program, _) = g.finish();
    let (source, expected) = program.finish();
    let mut b = sapphire::board::Board::new(sapphire::clock::NTSC);
    b.iop.bus.load(s21::ORIGIN, &s21::assemble(&source));
    b.iop.bus.load(0x80, &s21::vector());
    b.iop.enter(s21::K1 | s21::ORIGIN);
    let mut spr = vec![0u8; 16 << 10];
    let mut chunk = vec![0i16; 1 << 16];
    while b.iop.bus.word(s21::DONE_AT) != s21::DONE {
        let t = b.now + sapphire::clock::IOP * 20_000;
        b.run_to(t, &mut spr);
        b.iop.bus.spu2.out.drain(&mut chunk, usize::MAX);
    }
    println!("{{\"sapphire\": true}}");
    let words: Vec<u32> = std::iter::once(s21::DONE).chain((0..expected.len()).map(|i| b.iop.bus.word(s21::RECORD + 4 * i as u32))).collect();
    for row in words.chunks(8) {
        println!("{}", row.iter().map(|w| format!("{w:08X}")).collect::<Vec<_>>().join(" "));
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("sapphire") {
        return sapphire(&args[1]);
    }
    if args.first().map(String::as_str) == Some("grade") {
        return grade(&args[1], &args[2]);
    }
    if args.first().map(String::as_str) == Some("diff") {
        return diff(&args[1], &args[2]);
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
