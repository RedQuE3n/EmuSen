//! Dispute D-800's programs (Sapphire_Native.md §30.68): `d800 write <folder>` makes `<name>.elf` and
//! `<name>.expected` for the referees, as `examples/d700.rs` does; `d800 run <bios> <program.elf> <expected> [frames]`
//! enters the program by the fast boot on the player's BIOS given by path, runs to its done word and prints its record
//! against the expected words. The BIOS is handed to the machine and never read here; what is
//! printed is the program's record, the EE's exception counts, BadVAddr (a data address), Status, Cause and the INTC.

#[path = "../tests/d700/mod.rs"]
mod d700;
#[path = "../tests/d800/mod.rs"]
mod d800;
#[path = "../tests/s6/mod.rs"]
mod s6;

use sapphire::machine::{Machine, Market};
use sapphire::media::Media;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(String::as_str) {
        Some("write") => write(&a[2]),
        Some("run") => run(&a[2], &a[3], &a[4], a.get(5).map_or(600, |f| f.parse().unwrap())),
        _ => panic!("d800 write <folder> | d800 run <bios> <program.elf> <expected> [frames]"),
    }
}

fn write(folder: &str) {
    let dir = std::path::PathBuf::from(folder);
    std::fs::create_dir_all(&dir).expect("the folder");
    for p in d800::all() {
        let name = p.name;
        let (source, expected) = p.finish();
        let words = emusen_mips::assemble(&source, s6::ORIGIN, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        std::fs::write(dir.join(format!("{name}.elf")), sapphire::media::program(&words, s6::ORIGIN)).expect("written");
        let text: String = expected.iter().map(|w| w.map_or("-\n".to_string(), |w| format!("{w:08x}\n"))).collect();
        std::fs::write(dir.join(format!("{name}.expected")), text).expect("written");
        let graded = expected.iter().filter(|w| w.is_some()).count();
        println!("{name}: {} words of code and data, {} record words, {graded} graded", words.len(), expected.len());
    }
}

fn run(bios: &str, program: &str, expected: &str, frames: i64) {
    let bios = std::fs::read(bios).expect("the BIOS");
    let program = std::fs::read(program).expect("the program");
    let expected: Vec<Option<u32>> =
        std::fs::read_to_string(expected).expect("the expected words").lines().map(|l| u32::from_str_radix(l.trim(), 16).ok()).collect();
    let mut m = Machine::new(Media::read(&program).expect("an EE program"), Market::Americas, [Some("ps2.dualshock2"), None]);
    m.insert_bios(&bios);
    m.program = Some(program);
    while m.frames < frames && m.sys.peek(0x001F_FFF0, 4) as u32 != 0x5A99_D00E {
        m.advance();
    }
    let done = m.sys.peek(0x001F_FFF0, 4) as u32 == 0x5A99_D00E;
    println!(
        "done word {} at frame {}; EE exceptions {:?}; BadVAddr {:08x}; Status {:08x}; Cause {:08x}; I_STAT {:04x} I_MASK {:04x}",
        if done { "written" } else { "absent" },
        m.frames,
        m.sys.ee.exceptions,
        m.sys.ee.cop0[8],
        m.sys.ee.cop0[12],
        m.sys.ee.cop0[13],
        m.sys.board.intc.stat,
        m.sys.board.intc.mask
    );
    let mut differ = 0;
    for (i, want) in expected.iter().enumerate() {
        let got = m.sys.peek(0x0020_0000 + 4 * i as u32, 4) as u32;
        let mark = match want {
            Some(w) if *w != got => {
                differ += 1;
                " DIFFERS"
            }
            Some(_) => "",
            None => " (ungraded)",
        };
        println!("{i:3}: {got:08x}{mark}");
    }
    println!("{differ} graded words differ");
}
