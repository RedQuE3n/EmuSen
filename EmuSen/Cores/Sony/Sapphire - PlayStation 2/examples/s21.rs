//! Writes stage S21's programs for the referees: `s21 <folder>` makes `<name>.elf`, the EE program that places the IOP
//! program and copies its record (`tests/s21`'s `wrap`), and `<name>.expected`, the done word and the record, one word
//! a line in hexadecimal, `-` where any value is expected (Sapphire_Native.md §22).

#[path = "../tests/s21/mod.rs"]
mod s21;

fn main() {
    let dir = std::path::PathBuf::from(std::env::args().nth(1).expect("a folder"));
    std::fs::create_dir_all(&dir).expect("the folder");
    for p in s21::all() {
        let name = p.name;
        let (source, expected) = p.finish();
        let iop = s21::assemble(&source);
        let ee = s21::wrap(&iop, expected.len());
        let words = emusen_mips::assemble(&ee, s21::EE_ORIGIN, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        std::fs::write(dir.join(format!("{name}.elf")), sapphire::media::program(&words, s21::EE_ORIGIN)).expect("written");
        let text: String =
            std::iter::once(Some(s21::DONE)).chain(expected.iter().copied()).map(|w| w.map_or("-\n".to_string(), |w| format!("{w:08x}\n"))).collect();
        std::fs::write(dir.join(format!("{name}.expected")), text).expect("written");
        let graded = expected.iter().filter(|w| w.is_some()).count();
        println!("{name}: {} IOP words, {} record words, {graded} graded", iop.len(), expected.len());
    }
}
