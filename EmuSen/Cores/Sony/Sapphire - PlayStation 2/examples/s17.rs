//! Writes stage S17's programs for the referees: `s17 <folder>` makes `<name>.elf` and `<name>.expected` (one record
//! word a line in hexadecimal, `-` where any value is expected) for each, and `<name>.background`, the record with no
//! vertex kicked, so that a referee's difference can be read as a pixel drawn, not drawn or drawn otherwise
//! (Sapphire_Native.md §27.7).

#[path = "../tests/s6/mod.rs"]
mod s6;
#[path = "../tests/s11/mod.rs"]
mod s11;
#[path = "../tests/s16/mod.rs"]
mod s16;
#[path = "../tests/s17/mod.rs"]
mod s17;

fn main() {
    let dir = std::path::PathBuf::from(std::env::args().nth(1).expect("a folder"));
    std::fs::create_dir_all(&dir).expect("the folder");
    for script in s17::all() {
        let name = script.name;
        let (source, expected) = s17::program(&script).finish();
        let words = emusen_mips::assemble(&source, s6::ORIGIN, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        std::fs::write(dir.join(format!("{name}.elf")), sapphire::media::program(&words, s6::ORIGIN)).expect("written");
        let text: String = expected.iter().map(|w| w.map_or("-\n".to_string(), |w| format!("{w:08x}\n"))).collect();
        std::fs::write(dir.join(format!("{name}.expected")), text).expect("written");
        let quiet = s16::Script {
            name,
            ops: script.ops.iter().map(|op| match op {
                s16::Op::Regs(list) => s16::Op::Regs(list.iter().copied().filter(|&(r, _)| !matches!(r, 0x04 | 0x05 | 0x0C | 0x0D)).collect()),
                other => other.clone(),
            }).collect(),
        };
        let back: String = s17::expected(&quiet).iter().flat_map(|w| [w.map(|w| w as u32), w.map(|w| (w >> 32) as u32)]).map(|w| format!("{:08x}\n", w.unwrap_or(0))).collect();
        std::fs::write(dir.join(format!("{name}.background")), back).expect("written");
        let graded = expected.iter().filter(|w| w.is_some()).count();
        println!("{name}: {} words of code and data, {} record words, {graded} graded", words.len(), expected.len());
    }
}
