//! Unit tests of the machine core, with values the C# WiseMan tests give; the corpus behind `EMUSEN_MARSRT_CORPUS`.

mod blocks;
mod bus;
mod cpu;
mod debug;
mod devices;
mod fpu;
mod games;
mod gpu;
mod multiple;
mod rom;
mod rom_patches;
mod rsp;
mod rsp_blocks;
mod save;
mod sites;
mod support;
mod threads;
mod tlb;

use std::sync::Arc;

use crate::machine::Machine;
use crate::rom::RomImage;

/// The corpus's own last words, which are what the run being finished looks like.
const FINISHED: &str = "Base: Failed ";

/// `n64-systemtest` through the IS-Viewer, as `MarsCorpusTests` runs it: the interpreter alone, 400M steps at most.
#[test]
fn the_corpus_reports_what_the_csharp_core_reports() {
    let Ok(path) = std::env::var("EMUSEN_MARSRT_CORPUS") else { return };
    let image = RomImage::from_image(&std::fs::read(path).unwrap()).unwrap();
    let mut machine = Machine::boot(Arc::new(image), true);
    let started = std::time::Instant::now();
    for _ in 0..400 {
        machine.run_steps(1 << 20);
        if String::from_utf8_lossy(&machine.bus.is_viewer.text).contains(FINISHED) {
            break;
        }
    }
    let text = String::from_utf8_lossy(&machine.bus.is_viewer.text).into_owned();
    if let Ok(out) = std::env::var("EMUSEN_MARSRT_CORPUS_OUT") {
        std::fs::write(out, &text).unwrap();
    }
    eprintln!("{} instructions in {:?}", machine.cpu.instructions, started.elapsed());
    let at = text.find(FINISHED).expect("the corpus did not finish");
    let summary = &text[at + 6..at + text[at..].find(" tests").unwrap() + 6];
    assert_eq!(summary, "Failed 46 of 4637 tests");

    // Through the recompiler at each tier: the transcript line for line the interpreter's, which the WiseMan test holds to C#'s.
    let through = |text: &str| text[..text.find(FINISHED).map_or(text.len(), |at| at + text[at..].find('\n').unwrap_or(text.len() - at))].to_string();
    for tier in blocks::tiers() {
        let mut machine = Machine::boot(Arc::new(RomImage::from_image(&std::fs::read(std::env::var("EMUSEN_MARSRT_CORPUS").unwrap()).unwrap()).unwrap()), true);
        machine.set_recompiler(true);
        machine.blocks.tier = tier;
        let started = std::time::Instant::now();
        for _ in 0..400 {
            machine.run_steps(1 << 20);
            if String::from_utf8_lossy(&machine.bus.is_viewer.text).contains(FINISHED) {
                break;
            }
        }
        let got = String::from_utf8_lossy(&machine.bus.is_viewer.text).into_owned();
        let (want, got) = (through(&text), through(&got));
        for (n, (a, b)) in want.lines().zip(got.lines()).enumerate() {
            assert_eq!(a, b, "{tier:?}: line {}", n + 1);
        }
        assert_eq!(want.lines().count(), got.lines().count(), "{tier:?}");
        eprintln!("{tier:?}: {} lines identical in {:?}; blocks {:?}", got.lines().count(), started.elapsed(), machine.blocks.counters());
    }
}
