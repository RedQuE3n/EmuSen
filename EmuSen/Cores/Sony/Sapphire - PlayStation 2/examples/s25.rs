//! Writes stage S25's programs for the referees: `s25 <folder>` makes `<name>.elf`, the EE program that places the IOP
//! program and copies its record (`tests/s21`'s `wrap`), and `<name>.expected`, the done word and the record, one word
//! a line in hexadecimal, `-` where any value is expected (Sapphire_Native.md §30). `s25 run <name> <cards>` runs one
//! on Sapphire's board and prints its record the same way, `<cards>` two letters for the slots, `c` an erased card and
//! `-` none (Sapphire_Disputes.md D-383).

#[path = "../tests/s21/mod.rs"]
mod s21;
#[path = "../tests/s25/mod.rs"]
mod s25;

use sapphire::board::Board;
use sapphire::clock::{self, IOP};
use sapphire::sio2::card::MemoryCard;

fn run(name: &str, cards: &str) {
    let p = s25::all().into_iter().find(|p| p.name == name).expect("a program of S25's");
    let (source, expected) = p.finish();
    let words = s21::assemble(&source);
    let mut b = Board::new(clock::NTSC);
    b.iop.bus.load(s21::ORIGIN, &words);
    b.iop.bus.load(0x80, &s21::vector());
    for (s, c) in b.iop.bus.sio2.slots.iter_mut().zip(cards.chars()) {
        s.cards[0] = (c == 'c').then(MemoryCard::erased);
    }
    b.iop.enter(s21::K1 | s21::ORIGIN);
    let mut spr = vec![0u8; 16 << 10];
    while b.iop.bus.word(s21::DONE_AT) != s21::DONE {
        assert!(b.now < IOP * 2_000_000_000, "no done word");
        let t = b.now + IOP * 200_000;
        b.run_to(t, &mut spr);
    }
    println!("{:08x}", s21::DONE);
    for i in 0..expected.len() {
        println!("{:08x}", b.iop.bus.word(s21::RECORD + 4 * i as u32));
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("run") {
        return run(&args[2], args.get(3).map_or("cc", String::as_str));
    }
    let dir = std::path::PathBuf::from(args.get(1).expect("a folder"));
    std::fs::create_dir_all(&dir).expect("the folder");
    for p in s25::all() {
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
