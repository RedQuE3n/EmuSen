//! Writes stage S22's paired programs for the referees: `s22 <folder>` makes `<name>.elf`, the EE program with the IOP
//! program inside it (`tests/s22`'s `pair`), and `<name>.expected`, the record one word a line in hexadecimal, `-`
//! where any value is expected (Sapphire_Native.md §26).

#[path = "../tests/s21/mod.rs"]
mod s21;
#[path = "../tests/s22/mod.rs"]
mod s22;

fn main() {
    let dir = std::path::PathBuf::from(std::env::args().nth(1).expect("a folder"));
    std::fs::create_dir_all(&dir).expect("the folder");
    for p in s22::all() {
        std::fs::write(dir.join(format!("{}.elf", p.name)), sapphire::media::program(&p.image, s22::EE_ORIGIN)).expect("written");
        let text: String = p.expected.iter().map(|w| w.map_or("-\n".to_string(), |w| format!("{w:08x}\n"))).collect();
        std::fs::write(dir.join(format!("{}.expected", p.name)), text).expect("written");
        let graded = p.expected.iter().filter(|w| w.is_some()).count();
        println!("{}: {} words of image, {} record words ({} the EE's), {graded} graded", p.name, p.image.len(), p.expected.len(), p.ee_words);
    }
}
