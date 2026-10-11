//! Assembles a test program with `emusen-mips` into a PS-EXE: `psexe <source.s> <out.psexe>`, loaded and entered at
//! 80010000h with the stack at 801FFFF0h, no market marker (Diamond_Native.md §11.11).

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let source = std::fs::read_to_string(&args[0]).expect("the source");
    let words = emusen_mips::assemble(&source, 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).unwrap_or_else(|e| panic!("{}: {e}", args[0]));
    std::fs::write(&args[1], diamond::exe::build(&words)).expect("written");
}
