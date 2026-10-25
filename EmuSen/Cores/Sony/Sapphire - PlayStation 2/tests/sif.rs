//! Stage S22's gate (Sapphire_Native.md §26): the project's paired programs passing messages between the EE and the
//! IOP through the SIF's registers and its three DMA channels, run on the interpreter and `blackmoon-r3000` in one board
//! and graded by the records the documents give; and the FIFOs' rules no program can show a referee, checked on the
//! units directly.

mod s21;
mod s22;

use sapphire::board::System;
use sapphire::dmac;
use sapphire::sif::{self, FIFO_WORDS};

/// Runs a pair until the EE's done word: the EE entered at its program, the IOP entered at its own once the EE has
/// copied it and written the exception vector, as a referee's IOP enters it at its next interrupt.
fn run(p: &s22::Pair) -> System {
    let mut sys = System::new(s22::EE_ORIGIN);
    sys.board.map.load(s22::EE_ORIGIN, &p.image);
    let mut entered = false;
    let limit = 400_000_000;
    let n = sys
        .run(limit, &mut |s| {
            if !entered && s.board.iop.bus.word(0x88) == s21::vector()[2] {
                s.board.iop.enter(s21::K1 | s21::ORIGIN);
                entered = true;
            }
            s.peek(s22::EE_DONE_AT, 4) as u32 == s22::DONE
        })
        .unwrap_or_else(|h| panic!("{}: {h:?}", p.name));
    assert!(n < limit, "{}: no done word in {limit} instructions, EE pc {:08x}, IOP pc {:08x}", p.name, sys.ee.pc, sys.board.iop.cpu.regs.pc);
    sys
}

/// Runs a pair and grades the whole record, the EE's words and the IOP's after them.
fn grade(p: s22::Pair) -> System {
    let mut sys = run(&p);
    let mut wrong = Vec::new();
    for (i, want) in p.expected.iter().enumerate() {
        let got = sys.peek(s22::EE_RECORD + 4 * i as u32, 4) as u32;
        if let Some(w) = want
            && *w != got
        {
            let side = if i < p.ee_words { "EE" } else { "IOP" };
            wrong.push(format!("word {i} ({side}): {got:08x}, expected {w:08x}"));
        }
    }
    assert!(wrong.is_empty(), "{}: {} of {} words differ, first {:?}", p.name, wrong.len(), p.expected.len(), &wrong[..wrong.len().min(8)]);
    sys
}

#[test]
fn the_mailbox_registers_pass_values_and_flags_both_ways() {
    grade(s22::registers());
}

#[test]
fn sif1_carries_the_ees_packets_to_the_iop_in_every_mode() {
    let sys = grade(s22::sif1());
    assert!(sys.board.iop.bus.sif.fifo.iter().all(sif::Fifo::is_empty), "every FIFO drained");
}

#[test]
fn sif0_carries_the_iops_packets_to_the_ee_with_the_ees_tags() {
    let sys = grade(s22::sif0());
    assert!(sys.board.iop.bus.sif.fifo.iter().all(sif::Fifo::is_empty), "every FIFO drained");
}

#[test]
fn sif2_carries_data_both_ways() {
    grade(s22::sif2());
}

/// The EE's SIF1 with nobody reading: it moves whole quadwords until the FIFO's 32 words are full, then holds and
/// schedules nothing; once the IOP's channel starts, the rest crosses (D-140).
#[test]
fn a_full_fifo_holds_the_ees_channel_until_the_iop_takes_words() {
    use blackmoon_r3000::{Bus as _, Size};
    use sapphire::ee::Bus as _;
    let mut sys = System::new(s22::EE_ORIGIN);
    let b = &mut sys.board;
    let mut spr = vec![0u8; 16 << 10];
    let data: Vec<u32> = (0..76).map(|k| 0x5800_0000 + k).collect();
    b.map.load(0x30_0000, &[[0x1C_0000 | 1 << 31, 76, 0, 0].as_slice(), &data].concat());
    b.iop.bus.load(s21::ORIGIN, &s21::assemble("top: beq zero, zero, top\nnop\n"));
    b.iop.enter(s21::K1 | s21::ORIGIN);
    let ch = dmac::BASES[dmac::SIF1];
    for (reg, v) in [(dmac::D_CTRL, 1u32), (ch + 0x10, 0x30_0000), (ch + 0x20, 20), (ch, 0x101)] {
        b.write(reg, 4, u128::from(v));
    }
    b.run_to(10_000, &mut spr);
    assert_eq!(b.iop.bus.sif.fifo[sif::SIF1].len(), FIFO_WORDS, "the FIFO full");
    assert_eq!(b.dmac.ch[dmac::SIF1].qwc, 12, "eight of twenty quadwords moved");
    assert_eq!(b.dmac.next_event(), u64::MAX, "held, nothing scheduled");
    for (reg, v) in [(0xBF80_1570, 0x8800), (0xBF80_1578, 1), (0xBF80_1538, 0x0100_0600)] {
        b.iop.bus.write(reg, Size::Word, v);
    }
    for t in (10_000..1_000_000).step_by(4) {
        b.run_to(t, &mut spr);
    }
    assert_eq!(b.dmac.ch[dmac::SIF1].qwc, 0, "the rest crossed, the board brought on four cycles at a time as instructions do");
    assert!(b.iop.bus.sif.fifo[sif::SIF1].is_empty());
    assert_eq!((0..76).map(|k| b.iop.bus.word(0x1C_0000 + 4 * k)).collect::<Vec<_>>(), data);
}
