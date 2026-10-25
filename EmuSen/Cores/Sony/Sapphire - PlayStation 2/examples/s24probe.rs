//! Writes stage S24's measurement pairs for the referees, as `examples/s22.rs` writes S22's: `s24probe <folder>` makes
//! `<name>.elf` and `<name>.expected` (Sapphire_Native.md §35).

#[path = "../tests/s21/mod.rs"]
mod s21;
#[path = "../tests/s22/mod.rs"]
mod s22;
#[path = "../tests/s24/mod.rs"]
mod s24;

fn main() {
    let dir = std::path::PathBuf::from(std::env::args().nth(1).expect("a folder"));
    std::fs::create_dir_all(&dir).expect("the folder");
    for p in [s24::sif1_slice(), s24::sif0_odd()] {
        std::fs::write(dir.join(format!("{}.elf", p.name)), sapphire::media::program(&p.image, s22::EE_ORIGIN)).expect("written");
        let text: String = p.expected.iter().map(|w| w.map_or("-\n".to_string(), |w| format!("{w:08x}\n"))).collect();
        std::fs::write(dir.join(format!("{}.expected", p.name)), text).expect("written");
        println!("{}: {} record words ({} the EE's)", p.name, p.expected.len(), p.ee_words);
    }
    {
        let p = s24::statx();
        let name = p.name;
        let (source, expected) = p.finish();
        let iop = s21::assemble(&source);
        let ee = s21::wrap(&iop, expected.len());
        let words = emusen_mips::assemble(&ee, s21::EE_ORIGIN, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        std::fs::write(dir.join(format!("{name}.elf")), sapphire::media::program(&words, s21::EE_ORIGIN)).expect("written");
        println!("{name}: {} record words", expected.len());
    }
}
