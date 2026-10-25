//! Writes stage S12's programs for the referees: `s12 <folder>` makes `<name>.elf`, `<name>.expected` (one record word
//! a line in hexadecimal, `-` where any value is expected) and `<name>.s` for each (Sapphire_Native.md §21.7).

#[path = "../tests/s6/mod.rs"]
mod s6;
#[path = "../tests/s9/mod.rs"]
mod s9;
#[path = "../tests/s12/mod.rs"]
mod s12;

fn main() {
    let dir = std::path::PathBuf::from(std::env::args().nth(1).expect("a folder"));
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
