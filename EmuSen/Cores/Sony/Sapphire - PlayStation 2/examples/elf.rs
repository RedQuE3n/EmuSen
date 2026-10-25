//! Assembles a test program with `emusen-mips` into an EE program: `elf <source.s> <out.elf>`, loaded and entered at
//! 00100000h (Sapphire_Referees.md §4.1).

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let source = std::fs::read_to_string(&args[0]).expect("the source");
    let words = emusen_mips::assemble(&source, sapphire::media::PROGRAM_ORIGIN, &emusen_mips::Dialect::MIPS_I).unwrap_or_else(|e| panic!("{}: {e}", args[0]));
    std::fs::write(&args[1], sapphire::media::program(&words, sapphire::media::PROGRAM_ORIGIN)).expect("written");
}
