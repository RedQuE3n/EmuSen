//! Stage S16, the GS's registers, local memory and transfers (Sapphire_Native.md §24): the layout against chapter 8's
//! figures and §8.6's conversion tables, the project's programs driven through the GS's host interface and graded by
//! the manual's model, the console texts of gs/label and gs/signal, and the GS on the board.

mod s6;
mod s11;
mod s16;

use sapphire::board::Board;
use sapphire::clock::{self, NTSC};
use sapphire::ee::Bus;
use sapphire::gs::local::{self, Address};
use sapphire::gs::psm::{ALL, Psm};
use sapphire::gs::{HostInterface, csr, privileged};
use s16::manual;
use s16::{Op, Script, ga};

const SPR: usize = 16 << 10;

fn code(p: Psm) -> u32 {
    p.code()
}

#[test]
fn every_formats_page_falls_as_chapter_8s_figures_show() {
    for p in ALL {
        let (pw, ph) = p.shape().page();
        for (bp, bw) in [(0, pw / 64), (1, pw / 64), (0x3FF0, 2 * pw / 64), (37, 5)] {
            for y in 0..2 * ph {
                for x in 0..2 * pw {
                    let a = local::address(p, bp, bw, x, y);
                    let (w, lane) = manual::locate(code(p), bp, bw, x, y);
                    assert_eq!(a, Address { word: w as u32, shift: lane }, "{p:?} bp {bp} bw {bw} at ({x}, {y})");
                }
            }
        }
    }
}

#[test]
fn every_word_of_a_page_is_reached_once_in_every_format() {
    for p in ALL {
        let (pw, ph) = p.shape().page();
        let mut seen = std::collections::HashMap::new();
        for y in 0..ph {
            for x in 0..pw {
                let a = local::address(p, 0, pw / 64, x, y);
                *seen.entry((a.word, a.shift)).or_insert(0) += 1;
            }
        }
        assert!(seen.values().all(|&n| n == 1), "{p:?}: a place reached twice");
        let words: std::collections::HashSet<u32> = seen.keys().map(|k| k.0).collect();
        assert_eq!(words.len(), 2048, "{p:?}: a page is 2,048 words");
        assert!(words.iter().all(|&w| w < 2048), "{p:?}: inside the page");
    }
}

/// §8.6's tables: the PSMCT32 pixels A to P of one column, and which pixels of the 4-bit, 8-bit and 16-bit columns
/// each word holds, from bit 31 down.
const T4_EVEN: &str = "92 24 84 16 76 8 68 0 / 93 25 85 17 77 9 69 1 / 94 26 86 18 78 10 70 2 / 95 27 87 19 79 11 71 3 / \
                       88 28 80 20 72 12 64 4 / 89 29 81 21 73 13 65 5 / 90 30 82 22 74 14 66 6 / 91 31 83 23 75 15 67 7 / \
                       124 56 116 48 108 40 100 32 / 125 57 117 49 109 41 101 33 / 126 58 118 50 110 42 102 34 / \
                       127 59 119 51 111 43 103 35 / 120 60 112 52 104 44 96 36 / 121 61 113 53 105 45 97 37 / \
                       122 62 114 54 106 46 98 38 / 123 63 115 55 107 47 99 39";
const T4_ODD: &str = "88 28 80 20 72 12 64 4 / 89 29 81 21 73 13 65 5 / 90 30 82 22 74 14 66 6 / 91 31 83 23 75 15 67 7 / \
                      92 24 84 16 76 8 68 0 / 93 25 85 17 77 9 69 1 / 94 26 86 18 78 10 70 2 / 95 27 87 19 79 11 71 3 / \
                      120 60 112 52 104 44 96 36 / 121 61 113 53 105 45 97 37 / 122 62 114 54 106 46 98 38 / \
                      123 63 115 55 107 47 99 39 / 124 56 116 48 108 40 100 32 / 125 57 117 49 109 41 101 33 / \
                      126 58 118 50 110 42 102 34 / 127 59 119 51 111 43 103 35";
const T8_EVEN: &str = "44 8 36 0 / 45 9 37 1 / 46 10 38 2 / 47 11 39 3 / 40 12 32 4 / 41 13 33 5 / 42 14 34 6 / 43 15 35 7 / \
                       60 24 52 16 / 61 25 53 17 / 62 26 54 18 / 63 27 55 19 / 56 28 48 20 / 57 29 49 21 / 58 30 50 22 / 59 31 51 23";
const T8_ODD: &str = "40 12 32 4 / 41 13 33 5 / 42 14 34 6 / 43 15 35 7 / 44 8 36 0 / 45 9 37 1 / 46 10 38 2 / 47 11 39 3 / \
                      56 28 48 20 / 57 29 49 21 / 58 30 50 22 / 59 31 51 23 / 60 24 52 16 / 61 25 53 17 / 62 26 54 18 / 63 27 55 19";
const T16: &str = "8 0 / 9 1 / 10 2 / 11 3 / 12 4 / 13 5 / 14 6 / 15 7 / 24 16 / 25 17 / 26 18 / 27 19 / 28 20 / 29 21 / 30 22 / 31 23";

#[test]
fn the_conversion_tables_of_8_6_hold_in_every_column() {
    for column in 0..4u32 {
        let odd = column % 2 == 1;
        for (psm, table, width, bits) in [
            (Psm::T4, if odd { T4_ODD } else { T4_EVEN }, 32, 4),
            (Psm::T8, if odd { T8_ODD } else { T8_EVEN }, 16, 8),
            (Psm::Ct16, T16, 16, 16),
        ] {
            let rows = psm.shape().column_height();
            for (letter, pixels) in manual::grid(table).iter().enumerate() {
                let (lx, ly) = (letter as u32 % 8, letter as u32 / 8);
                let word = local::address(Psm::Ct32, 0, 1, lx, 2 * column + ly).word;
                for (j, &n) in pixels.iter().enumerate() {
                    let at = local::address(psm, 0, 2, n % width, rows * column + n / width);
                    let lane = bits * (pixels.len() as u32 - 1 - j as u32);
                    assert_eq!(at, Address { word, shift: lane }, "{psm:?} column {column}: pixel {n} in letter {letter}");
                }
            }
        }
    }
}

/// The script run on the board, the GS's host interface driven directly; its record, 64 bits an entry.
fn run(script: &Script) -> Vec<u64> {
    let mut b = Board::new(clock::NTSC);
    let mut spr = vec![0u8; SPR];
    let mut out = Vec::new();
    let at = |g: u32| privileged::address(g);
    for op in &script.ops {
        match op {
            Op::Regs(list) => {
                for &(r, v) in list {
                    b.gs.write(r, v);
                }
                b.sync_gs();
            }
            Op::Image { seed, qwords } => {
                for u in s16::units(*seed, *qwords) {
                    b.gs.image(u);
                }
                b.sync_gs();
            }
            Op::Priv(g, v) => b.write(at(*g), 8, u128::from(*v)),
            Op::PrivAt(a, bytes, v) => b.write(*a, *bytes, u128::from(*v)),
            Op::Finish => {
                b.write(at(ga::CSR), 8, 2);
                b.gs.write(s16::r::FINISH, 0);
                b.sync_gs();
                assert_ne!(b.read(at(ga::CSR), 8) & 2, 0, "FINISH at once");
                b.write(at(ga::CSR), 8, 2);
            }
            Op::Readback(q) => {
                b.write(at(ga::BUSDIR), 8, 1);
                out.extend((0..2 * q).map(|_| b.gs.read().unwrap_or(0)));
                b.write(at(ga::BUSDIR), 8, 0);
            }
            Op::Record(g, mask, _) => out.push(b.read(at(*g), 8) as u64 & mask),
            Op::RecordAt(a, bytes, mask, _) => out.push(b.read(*a, *bytes) as u64 & mask),
            Op::Intc => {
                out.push(u64::from(b.intc.stat & 1));
                b.write(sapphire::intc::I_STAT, 4, 1);
            }
            Op::ClearIntc => b.write(sapphire::intc::I_STAT, 4, 1),
            Op::Probe(g, pattern) => {
                out.push(b.read(at(*g), 8) as u64);
                b.write(at(*g), 8, u128::from(*pattern));
                out.push(b.read(at(*g), 8) as u64 & 0xC01F);
                b.write(at(*g), 8, 0);
            }
            Op::Wait(g, mask) => {
                let limit = b.now + 2 * NTSC.cycles(NTSC.field(false));
                while b.read(at(*g), 8) as u64 & mask == 0 {
                    assert!(b.now < limit, "{}: waited two fields for {mask:x}", script.name);
                    let t = b.now + 1000;
                    b.run_to(t, &mut spr);
                }
            }
        }
    }
    out
}

fn grade(script: &Script) {
    let want = s16::expected(script);
    let got = run(script);
    assert_eq!(want.len(), got.len(), "{}: the record's length", script.name);
    let bad: Vec<_> = want.iter().zip(&got).enumerate().filter(|(_, (w, g))| w.is_some_and(|w| w != **g)).collect();
    assert!(bad.is_empty(), "{}: {} of {} entries differ, the first {:?}", script.name, bad.len(), want.len(), &bad[..bad.len().min(4)]);
}

#[test]
fn every_format_round_trips_and_lies_in_memory_as_the_manual_says() {
    use manual::*;
    grade(&s16::formats("s16-formats-c", &[CT32, CT24, CT16, CT16S]));
    grade(&s16::formats("s16-formats-z", &[Z32, Z24, Z16, Z16S]));
    grade(&s16::formats("s16-formats-t", &[T8, T8H, T4, T4HL, T4HH]));
}

#[test]
fn transfers_take_their_places_wrap_restart_and_copy_as_chapter_4_says() {
    grade(&s16::places());
}

#[test]
fn the_registers_csr_imr_and_the_signals_behave_as_chapters_6_and_7_say() {
    grade(&s16::regs());
}

#[test]
fn the_programs_assemble_and_their_records_end_in_the_done_word() {
    for script in s16::all() {
        let (source, expected) = s16::program(&script).finish();
        let words = emusen_mips::assemble(&source, s6::ORIGIN, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{}: {e:?}", script.name));
        assert!(sapphire::media::Media::read(&sapphire::media::program(&words, s6::ORIGIN)).is_ok(), "{}", script.name);
        assert_eq!(expected.len(), 2 * s16::expected(&script).len(), "{}", script.name);
    }
}

/// gs/label's and gs/signal's console texts (ps2autotests' `.expected`), reached by the sequence their values imply
/// [A: the programs' sources are not read]: SIGLBLID written whole, then a LABEL or SIGNAL whose IDMSK covers bits 4
/// to 7, with SIGNAL's flag set either way.
#[test]
fn the_console_texts_of_gs_label_and_gs_signal_follow_from_the_masked_updates() {
    let mut b = Board::new(clock::NTSC);
    let at = privileged::address(privileged::SIGLBLID);
    b.write(at, 8, 0x0000_0000_7654_3210);
    b.gs.write(s16::r::LABEL, 0x0000_00F0_FFFF_FFFF);
    assert_eq!(b.read(at, 8), 0x0000_00F0_7654_3210, "Set label bits");
    b.write(at, 8, 0x3333_3333_3333_3333);
    b.gs.write(s16::r::LABEL, 0x0000_00F0_0000_0000);
    assert_eq!(b.read(at, 8), 0x3333_3303_3333_3333, "Clear label bits");
    b.write(at, 8, 0x7654_3210_0000_0000);
    b.gs.write(s16::r::SIGNAL, 0x0000_00F0_FFFF_FFFF);
    assert_eq!((b.read(privileged::address(privileged::CSR), 8) & 1, b.read(at, 8)), (1, 0x7654_3210_0000_00F0), "Set signal bits");
    b.write(privileged::address(privileged::CSR), 8, 1);
    b.write(at, 8, 0x3333_3333_3333_3333);
    b.gs.write(s16::r::SIGNAL, 0x0000_00F0_0000_0000);
    assert_eq!((b.read(privileged::address(privileged::CSR), 8) & 1, b.read(at, 8)), (1, 0x3333_3333_3333_3303), "Clear signal bits");
}

#[test]
fn the_privileged_registers_answer_their_images_and_above_them_a_bus_error() {
    let mut b = Board::new(clock::NTSC);
    b.write(0x1200_1080, 8, 0x0123_4567_89AB_CDEF);
    for a in [0x1200_1480, 0x1200_1880, 0x1200_1C80, 0x1200_3080, 0x1200_F080] {
        assert_eq!(b.read(a, 8), 0x0123_4567_89AB_CDEF, "SIGLBLID at {a:x}");
    }
    let csr_value = u128::from(0x4000 | csr::REV << 16 | csr::ID << 24);
    for a in [0x1200_0000, 0x1200_0010, 0x1200_00E0, 0x1200_00F0, 0x1200_1010, 0x1200_1040, 0x1200_1050] {
        assert_eq!(b.read(a, 8), csr_value, "{a:x} reads CSR (D-100)");
    }
    assert!(!b.error(0x1200_FFF0) && b.error(0x1201_0000) && b.error(0x13FF_FFF0) && !b.error(0x1400_0000));
    b.write(0x1200_1000, 4, 0xFFFF_FFFF);
    assert_eq!(b.read(0x1200_1000, 4) & 0xFFFF, 0x4000, "a 32-bit store acts on CSR's low word");
}

#[test]
fn the_gs_interrupt_is_one_line_the_intc_latches_at_its_rise() {
    let mut b = Board::new(clock::NTSC);
    let (csr_at, imr_at) = (privileged::address(privileged::CSR), privileged::address(privileged::IMR));
    b.gs.write(s16::r::FINISH, 0);
    b.sync_gs();
    assert_eq!(b.intc.stat & 1, 0, "masked after reset");
    b.write(imr_at, 8, 0x7D00);
    assert_eq!(b.intc.stat & 1, 1, "unmasked with FINISH set: the line rises");
    b.write(sapphire::intc::I_STAT, 4, 1);
    b.gs.write(s16::r::SIGNAL, 0);
    b.write(imr_at, 8, 0x7C00);
    b.sync_gs();
    assert_eq!(b.intc.stat & 1, 0, "a second event while the line is high gives no edge");
    b.write(csr_at, 8, 3);
    b.gs.write(s16::r::SIGNAL, 0);
    b.sync_gs();
    assert_eq!(b.intc.stat & 1, 1, "cleared, then raised again: an edge");
}

#[test]
fn hsint_and_vsint_follow_the_pcrtcs_timing() {
    let mut b = Board::new(clock::NTSC);
    let mut spr = vec![0u8; SPR];
    let csr_at = privileged::address(privileged::CSR);
    let line = NTSC.cycles(NTSC.line);
    b.write(csr_at, 8, 0xC);
    b.run_to(line - 1, &mut spr);
    assert_eq!(b.read(csr_at, 8) & 0xC, 0);
    b.run_to(line, &mut spr);
    assert_eq!(b.read(csr_at, 8) & 0xC, 4, "HSINT at the next line");
    b.write(csr_at, 8, 4);
    b.run_to(NTSC.cycles(4 * NTSC.line), &mut spr);
    assert_eq!(b.read(csr_at, 8) & 0xC, 0xC, "VSINT at the V-sync four lines in, HSINT again with the line");
    b.write(privileged::address(privileged::IMR), 8, 0x7700);
    assert_eq!(b.intc.stat & 1, 1, "VSINT unmasked reaches the INTC");
}

#[test]
fn the_host_interface_stalls_on_a_second_signal_and_reads_only_when_reversed() {
    let mut b = Board::new(clock::NTSC);
    b.gs.write(s16::r::SIGNAL, 0);
    assert!(!b.gs.stalled());
    b.gs.write(s16::r::SIGNAL, 0);
    assert!(b.gs.stalled(), "the second SIGNAL waits on the first's flag");
    b.gs.write(s16::r::FINISH, 0);
    assert_eq!(b.read(privileged::address(privileged::CSR), 8) & 0xC003, 1, "FINISH held behind it; the FIFO not empty");
    b.write(privileged::address(privileged::CSR), 8, 1);
    assert!(!b.gs.stalled());
    assert_eq!(b.read(privileged::address(privileged::CSR), 8) & 0xC003, 0x4003);
    b.gs.write(s16::r::BITBLTBUF, s16::bitbltbuf(0, 1, 0, 0, 0, 0));
    b.gs.write(s16::r::TRXREG, s16::trxreg(2, 1));
    b.gs.write(s16::r::TRXDIR, 1);
    assert_eq!(b.gs.read(), None, "not reversed");
    b.write(privileged::address(privileged::BUSDIR), 8, 1);
    assert!(b.gs.reversed());
    assert_eq!(b.gs.read(), Some(0));
    assert_eq!(b.gs.read(), None, "two pixels of 32 bits, one unit");
    b.gs.write(s16::r::FINISH, 0);
    b.write(privileged::address(privileged::CSR), 8, 2);
    b.gs.write(s16::r::FINISH, 0);
    assert_eq!(b.read(privileged::address(privileged::CSR), 8) & 2, 0, "the general port is unreachable while reversed");
}
