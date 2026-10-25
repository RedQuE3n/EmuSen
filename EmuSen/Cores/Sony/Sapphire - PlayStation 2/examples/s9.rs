//! Stage S9's tools (Sapphire_Native.md §19.6, §19.3): `s9 <folder>` writes each program as `<name>.elf`, its expected
//! record `<name>.expected` (one record word a line in hexadecimal, `-` where any value is expected) and its source;
//! `s9 texts <ee_fpu folder>` grades `sapphire::float` against ps2autotests' `cpu/ee_fpu` console texts (the
//! `.expected` files only, never the sources) and prints every line it does not reproduce.

#[path = "../tests/s6/mod.rs"]
mod s6;
#[path = "../tests/s9/mod.rs"]
mod s9;

use sapphire::float;

/// The constants as the console's own `mov` lines in arithmetic.expected print them; CF_NEGZERO from the add lines.
fn constant(t: &str) -> Option<u32> {
    Some(match t {
        "CF_ZERO" => 0,
        "CF_NEGZERO" => 0x8000_0000,
        "CF_ONE" => 0x3F80_0000,
        "CF_NEGONE" => 0xBF80_0000,
        "CF_MAX_MANTISSA" => 0x3FFF_FFFF,
        "CF_MAX_EXP" => 0x7F80_0001,
        "CF_MIN_EXP" => 0x0000_0001,
        "CF_MAX" => 0x7FFF_FFFF,
        "CF_MIN" => 0xFFFF_FFFF,
        "CF_GARBAGE1" => 0x0000_1337,
        "CF_GARBAGE2" => 0xDEAD_BEEF,
        _ => return None,
    })
}
fn value(t: &str) -> u32 {
    let t = t.trim();
    constant(t).unwrap_or_else(|| u32::from_str_radix(t.split('/').next().unwrap(), 16).unwrap_or_else(|_| panic!("{t}")))
}

/// One console line's result as the module computes it, or none for an operation it does not grade.
fn compute(op: &str, acc: Option<u32>, o: &[u32]) -> Option<String> {
    let hex = |v: u32| format!("{v:08x}");
    let truth = |b: bool| if b { "true".to_string() } else { "false".to_string() };
    Some(match op {
        "abs" => hex(o[0] & float::MAX),
        "neg" => hex(o[0] ^ float::SIGN),
        "mov" => hex(o[0]),
        "add" | "adda" => hex(float::add(o[0], o[1]).bits),
        "sub" | "suba" => hex(float::sub(o[0], o[1]).bits),
        "mul" | "mula" => hex(float::mul(o[0], o[1]).bits),
        "div" => hex(float::div(o[0], o[1]).value.bits),
        "max" => hex(float::max(o[0], o[1])),
        "min" => hex(float::min(o[0], o[1])),
        "madd" | "madda" | "msub" | "msuba" => hex(float::fused(acc?, o[0], o[1], op.starts_with("msub")).value.bits),
        "sqrt" => hex(float::sqrt(o[0]).bits),
        "rsqrt" => hex(float::rsqrt(o[0], o[1]).value.bits),
        "cvt.w.s" => hex(float::to_word(o[0])),
        "cvt.s.w" => hex(float::from_word(o[0])),
        "eq" => truth(float::order(o[0]) == float::order(o[1])),
        "lt" => truth(float::order(o[0]) < float::order(o[1])),
        "le" => truth(float::order(o[0]) <= float::order(o[1])),
        "f" => truth(false),
        _ => return None,
    })
}

fn texts(dir: &std::path::Path) {
    let (mut graded, mut missed) = (0, 0);
    for file in ["arithmetic", "muldiv", "compare", "convert", "sqrt"] {
        let text = std::fs::read_to_string(dir.join(format!("{file}.expected"))).expect("the console text");
        for line in text.replace('\r', "").lines() {
            let Some(rest) = line.strip_prefix("  ") else { continue };
            let Some((op, rest)) = rest.split_once(' ') else { continue };
            let Some((args, result)) = rest.rsplit_once(": ") else { continue };
            let (acc, args) = match args.strip_prefix('(') {
                Some(r) => {
                    let (a, b) = r.split_once(") ").expect("an accumulator");
                    (Some(value(a)), b)
                }
                None => (None, args),
            };
            let operands: Vec<u32> = args.split(", ").map(value).collect();
            let want = result.split(' ').next().unwrap().split('/').next().unwrap();
            let Some(got) = compute(op, acc, &operands) else { continue };
            graded += 1;
            if got != want {
                missed += 1;
                println!("{file}: {line}   module: {got}");
            }
        }
    }
    println!("{graded} lines graded, {missed} not reproduced");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("texts") {
        return texts(std::path::Path::new(args.get(1).expect("the ee_fpu folder")));
    }
    let dir = std::path::PathBuf::from(args.first().expect("a folder"));
    std::fs::create_dir_all(&dir).expect("the folder");
    for p in s9::all() {
        let name = p.name;
        let (source, expected) = p.finish();
        let words = emusen_mips::assemble(&source, s6::ORIGIN, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        std::fs::write(dir.join(format!("{name}.elf")), sapphire::media::program(&words, s6::ORIGIN)).expect("written");
        let text: String = expected.iter().map(|w| w.map_or("-\n".to_string(), |w| format!("{w:08x}\n"))).collect();
        std::fs::write(dir.join(format!("{name}.expected")), text).expect("written");
        std::fs::write(dir.join(format!("{name}.s")), &source).expect("written");
        println!("{name}: {} words of code and data, {} record words", words.len(), expected.len());
    }
}
