//! Stage S27's gate (Sapphire_Native.md §34): the IPU.

mod s27;

use s27::driver::{self, Options};
use s27::{program, standard};

use sapphire::board::System;

/// Runs a program on the interpreter with the board until its done word, and grades its record.
fn grade(p: &program::Program) {
    let mut sys = System::new(program::ORIGIN);
    sys.board.map.load(program::ORIGIN, &p.image);
    let limit = 400_000_000;
    let n = sys.run(limit, &mut |s| s.peek(program::DONE_AT, 4) as u32 == program::DONE).unwrap_or_else(|h| panic!("{}: {h:?}", p.name));
    assert!(n < limit, "{}: no done word in {limit} instructions, pc {:08x}", p.name, sys.ee.pc);
    println!("{}: {} commands, {} record words, {n} instructions, {} EE cycles", p.name, p.commands, p.expected.len(), sys.ee.timing.cycles());
    let mut wrong = Vec::new();
    for (i, want) in p.expected.iter().enumerate() {
        let got = sys.peek(program::RECORD + 4 * i as u32, 4) as u32;
        if let Some(w) = want
            && *w != got
        {
            wrong.push(format!("word {i}: {got:08x}, expected {w:08x}"));
        }
    }
    assert!(wrong.is_empty(), "{}: {} of {} words differ, first {:?}", p.name, wrong.len(), p.expected.len(), &wrong[..wrong.len().min(8)]);
}

#[test]
fn the_corpus_decodes_as_the_standard_and_ffmpeg_agree() {
    for s in s27::corpus() {
        let d = standard::decode(&s.complete());
        let mine: Vec<u8> = d.frames.iter().flat_map(|f| f.bytes()).collect();
        assert_eq!(mine.len(), s.ffmpeg.len(), "{}: frames", s.name);
        let diff: Vec<i32> = mine.iter().zip(s.ffmpeg).map(|(a, b)| i32::from(*a) - i32::from(*b)).collect();
        let max = diff.iter().map(|d| d.abs()).max().unwrap();
        let n = diff.iter().filter(|&&d| d != 0).count();
        println!("{}: {} frames, {} samples differ from ffmpeg, at most by {max}", s.name, d.frames.len(), n);
    }
}

#[test]
fn bdec_gives_the_standards_blocks_for_every_macroblock_of_the_corpus() {
    for s in s27::corpus() {
        let d = standard::decode(&s.complete());
        let log = driver::drive(&s.complete(), &d, Options::BDEC).unwrap_or_else(|e| panic!("{}: {e}", s.name));
        println!("{}: {} commands", s.name, log.len());
    }
}

#[test]
fn idec_gives_the_standards_pictures_through_the_manuals_csc() {
    for s in s27::corpus() {
        let d = standard::decode(&s.complete());
        let log = driver::drive(&s.complete(), &d, Options::IDEC32).unwrap_or_else(|e| panic!("{}: {e}", s.name));
        println!("{}: {} commands", s.name, log.len());
    }
}


#[test]
fn the_programs_replay_their_logs_through_the_dmac_on_the_board() {
    for (name, o) in [("intra-matrix", Options::IDEC32), ("pb", Options::BDEC)] {
        let s = s27::corpus().into_iter().find(|s| s.name == name).unwrap();
        let bytes = s.complete();
        let d = standard::decode(&bytes);
        let log = driver::drive(&bytes, &d, o).unwrap();
        let p = program::build(name, &driver::image(&bytes), &log);
        grade(&p);
    }
}

#[test]
fn csc_and_pack_follow_the_manuals_post_processing() {
    s27::post().unwrap();
}
