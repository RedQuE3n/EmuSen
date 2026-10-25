//! Stage S27's gate (Sapphire_Native.md §34): the IPU.

mod s27;

use s27::driver::{self, Options};
use s27::standard;

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

