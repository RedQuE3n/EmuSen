//! Stage S15, the GIF (Sapphire_Native.md §33): the project's programs sending GS packets down each path and
//! interleaving them, run on the interpreter with the board and graded by the records `s15::manual` gives from the EE
//! User's Manual's chapter 7 over S16's and S17's models of the GS; S16's and S17's own programs, until now graded by
//! driving the GS's host interface directly, run whole through PATH3 and VIF1; and what no program can show a
//! referee (each packing conversion's output, the FIFO's depth against the DMAC, the paths' cost), on the units.

mod s6;
mod s11;
mod s14;
mod s15;
mod s16;
mod s17;

use sapphire::board::System;

use s6::{DONE, DONE_AT, ORIGIN, Program, RECORD};

fn assemble(source: &str) -> Vec<u32> {
    emusen_mips::assemble(source, ORIGIN, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{e:?}"))
}

/// Runs a program on the interpreter and the board, and returns the system with every record word graded.
fn grade(p: Program) -> System {
    let name = p.name;
    let (source, expected) = p.finish();
    let mut sys = System::new(ORIGIN);
    sys.board.map.load(ORIGIN, &assemble(&source));
    let limit = 400_000_000;
    let n = sys.run(limit, &mut |s| s.peek(DONE_AT, 4) as u32 == DONE).unwrap_or_else(|h| panic!("{h:?}"));
    assert!(n < limit, "{name}: no done word in {limit} instructions, pc {:08x}", sys.ee.pc);
    let mut wrong = Vec::new();
    for (i, want) in expected.iter().enumerate() {
        let got = sys.peek(RECORD + 4 * i as u32, 4) as u32;
        if let Some(w) = want
            && *w != got
        {
            wrong.push(format!("word {i} (record {}): {got:08x}, expected {w:08x}", i / 2));
        }
    }
    assert!(wrong.is_empty(), "{name}: {} of {} words differ, first {:?}", wrong.len(), expected.len(), &wrong[..wrong.len().min(8)]);
    sys
}

#[test]
fn s16s_programs_run_whole_through_path3_and_vif1() {
    for script in s16::all() {
        grade(s16::program(&script));
    }
}

#[test]
fn s17s_programs_draw_through_path3() {
    for script in s17::all() {
        grade(s17::program(&script));
    }
}
