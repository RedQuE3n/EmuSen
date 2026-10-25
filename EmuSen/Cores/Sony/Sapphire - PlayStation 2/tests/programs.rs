//! The referees' programs (Sapphire_Referees.md §4): each assembles with emusen-mips, and its ELF is one the core reads.

use sapphire::media::{self, Media, PROGRAM_ORIGIN};

fn programs() -> Vec<(String, Vec<u32>)> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("programs");
    let mut out: Vec<_> = std::fs::read_dir(&dir).expect("programs/").map(|e| e.expect("an entry").path()).filter(|p| p.extension().is_some_and(|x| x == "s")).collect();
    out.sort();
    out.into_iter()
        .map(|p| {
            let text = std::fs::read_to_string(&p).expect("the source");
            let words = emusen_mips::assemble(&text, PROGRAM_ORIGIN, &emusen_mips::Dialect::MIPS_I).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            (p.file_name().unwrap().to_string_lossy().into_owned(), words)
        })
        .collect()
}

#[test]
fn each_program_assembles_into_an_elf_the_core_reads() {
    let all = programs();
    assert!(all.iter().any(|(n, _)| n == "record.s"), "the gate's program");
    for (name, words) in all {
        match Media::read(&media::program(&words, PROGRAM_ORIGIN)) {
            Ok(Media::Elf(elf)) => {
                assert_eq!(elf.entry, PROGRAM_ORIGIN, "{name}");
                assert_eq!(elf.segments, vec![(PROGRAM_ORIGIN, 84, words.len() as u32 * 4, words.len() as u32 * 4)], "{name}");
            }
            other => panic!("{name}: {other:?}"),
        }
    }
}

#[test]
fn the_record_ends_with_its_done_word_and_a_branch_to_itself() {
    let (_, words) = programs().into_iter().find(|(n, _)| n == "record.s").unwrap();
    let n = words.len();
    assert_eq!(&words[n - 2..], &[0x1000_FFFF, 0], "end: beq zero, zero, end; nop");
    assert_eq!(words[n - 3], 0xAC88_FFF0, "sw t0, -16(a0): the done word to 001FFFF0h");
    assert_eq!(&words[n - 5..n - 3], &[0x3C08_5A99, 0x3508_D00E], "t0 = 5A99D00Eh");
}

#[test]
fn the_channel_control_writes_its_line_to_kputchar_a_byte_at_a_time_then_its_done_word() {
    let (_, words) = programs().into_iter().find(|(n, _)| n == "kputchar.s").unwrap();
    let line = b"EMUSEN KPUTCHAR CONTROL\n";
    assert_eq!(words[0], 0x3C05_B001, "lui a1, 0xB001");
    let bytes: Vec<u8> = words[1..].chunks(2).take_while(|p| p.len() == 2 && p[1] == 0xA0A8_F180).map(|p| { assert_eq!(p[0] & 0xFFFF_FF00, 0x2408_0000, "addiu t0, zero, c"); p[0] as u8 }).collect();
    assert_eq!(bytes, line, "each byte stored to B000F180h in order");
    let n = words.len();
    assert_eq!(&words[n - 6..], &[0x3C04_0020, 0x3C08_5A99, 0x3508_D00E, 0xAC88_FFF0, 0x1000_FFFF, 0], "the done word to 001FFFF0h, then a branch to itself");
}
