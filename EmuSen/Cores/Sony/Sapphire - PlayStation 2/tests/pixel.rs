//! Stage S19, the GS's pixel operations and local-to-local overlap (Sapphire_Native.md §37): the project's programs
//! driven through the GS's host interface and graded by the manual's model (`tests/s17/manual.rs`'s `pixel_ops`), many
//! generated primitives under generated pixel-operation registers drawn by both, and the programs' assembly for the
//! referees.

mod s6;
mod s11;
mod s16;
mod s17;
mod s18;
mod s19;

use sapphire::board::Board;
use sapphire::clock;
use sapphire::ee::Bus;
use sapphire::gs::{HostInterface, privileged};
use s16::{Op, Script};

/// The script run on the board's GS, its host interface driven directly; its record, 64 bits an entry.
fn run(script: &Script) -> Vec<u64> {
    let mut b = Board::new(clock::NTSC);
    let at = privileged::address;
    let mut out = Vec::new();
    for op in &script.ops {
        match op {
            Op::Regs(list) => list.iter().for_each(|&(r, v)| b.gs.write(r, v)),
            Op::Image { seed, qwords } => s16::units(*seed, *qwords).into_iter().for_each(|u| b.gs.image(u)),
            Op::Finish => {
                b.write(at(s16::ga::CSR), 8, 2);
                b.gs.write(s16::r::FINISH, 0);
                b.sync_gs();
                b.write(at(s16::ga::CSR), 8, 2);
            }
            Op::Readback(q) => {
                b.write(at(s16::ga::BUSDIR), 8, 1);
                out.extend((0..2 * q).map(|_| b.gs.read().unwrap_or(0)));
                b.write(at(s16::ga::BUSDIR), 8, 0);
            }
            other => panic!("{other:?}"),
        }
    }
    out
}

fn grade(script: &Script) {
    let want = s17::expected(script);
    let got = run(script);
    assert_eq!(want.len(), got.len(), "{}: the record's length", script.name);
    let bad: Vec<_> = want.iter().zip(&got).enumerate().filter(|(_, (w, g))| w.is_some_and(|w| w != **g)).map(|(i, (w, g))| (i, w.unwrap(), *g)).collect();
    assert!(bad.is_empty(), "{}: {} of {} entries differ, the first {:x?}", script.name, bad.len(), want.len(), &bad[..bad.len().min(4)]);
}

#[test]
fn every_alpha_test_under_every_afail_leaves_effective_what_3_7_2_says() {
    grade(&s19::atests());
}

#[test]
fn the_destination_alpha_test_reads_the_frames_alpha_bit_in_each_format() {
    grade(&s19::dates());
}

#[test]
fn every_blend_selection_with_fix_follows_3_8_1_and_colclamp() {
    grade(&s19::blends());
}

#[test]
fn pabe_blends_only_pixels_whose_alpha_msb_is_set() {
    grade(&s19::pabes());
}

#[test]
fn fba_ors_the_alpha_msb_in_each_context() {
    grade(&s19::fbas());
}

#[test]
fn dithering_adds_dimx_at_y_and_x_modulo_4_before_the_clamp() {
    grade(&s19::dithers());
}

#[test]
fn fbmsk_and_zmsk_keep_their_bits_after_every_other_operation() {
    grade(&s19::masks());
}

#[test]
fn antialiasing_replaces_alpha_by_coverage_and_writes_no_z_below_it() {
    grade(&s19::antialias());
}

#[test]
fn overlapping_local_to_local_copies_run_in_trxpos_order() {
    grade(&s19::overlaps());
}

/// Primitives of every kind, flat, Gouraud and textured, each under generated pixel-operation registers (TEST whole,
/// ALPHA with FIX, PABE, FBA, DTHE and DIMX, COLCLAMP, FBMSK, ZMSK, ABE, AA1) over generated frame and Z data in every colour
/// format FRAME names: the source and the model agree on every pixel and every Z.
#[test]
fn generated_primitives_under_generated_pixel_operations_draw_as_the_model_draws_them() {
    use s16::manual::{CT16, CT16S, CT24, CT32, Z32};
    use s17::{Buf, prim, r, xyz2};
    let mut x = 0x1357_2468u32;
    let mut next = |m: u32| {
        x = x.wrapping_mul(0x41C6_4E6D).wrapping_add(0x3039);
        (x >> 8) % m
    };
    for round in 0..8u32 {
        let psm = [CT32, CT24, CT16, CT16S][(round % 4) as usize];
        let mut ops = Vec::new();
        s19::begin(&mut ops);
        let h = if matches!(psm, CT16 | CT16S) { 64 } else { 32 };
        let f = Buf::new(200, 1, psm, 64, h);
        let z = Buf::new(202, 1, Z32, 64, 32);
        f.fill(&mut ops, 0x5252_0000 + round);
        z.fill(&mut ops, 0x6262_0000 + round);
        s17::target(&mut ops, &f, Some((&z, false)), s17::test(true, s17::ALWAYS));
        ops.push(Op::Regs(vec![(r::SCISSOR_1, s17::scissor(0, 63, 0, 31))]));
        let mut regs = Vec::new();
        for _ in 0..300 {
            let t = u64::from(next(1 << 16)) | u64::from(next(8) | 1) << 16;
            let al = u64::from(next(256)) | u64::from(next(256)) << 32;
            let mask = if next(3) == 0 { next(u32::MAX >> 8) << 8 | next(256) } else { 0 };
            regs.extend([
                (r::TEST_1, t),
                (r::ALPHA_1, al),
                (r::PABE, u64::from(next(4) == 0)),
                (r::FBA_1, u64::from(next(4) == 0)),
                (r::DTHE, u64::from(next(3) == 0)),
                (s19::DIMX, u64::from(next(u32::MAX >> 8) << 8 | next(256)) | u64::from(next(u32::MAX >> 8) << 8 | next(256)) << 32),
                (r::COLCLAMP, u64::from(next(2))),
                (r::FRAME_1, s17::frame(f.page, 1, psm, mask)),
                (r::ZBUF_1, s17::zbuf(z.page, Z32, next(4) == 0)),
            ]);
            let kind = u64::from(next(7));
            let mut attributes = if next(2) == 1 { s17::GOURAUD } else { 0 } | if next(3) != 0 { s19::ABE } else { 0 } | if next(4) == 0 { s19::AA1 } else { 0 };
            let textured = next(3) == 0;
            if textured {
                attributes |= s18::TME | s18::FST;
            }
            regs.push(prim(kind, attributes));
            for _ in 0..[1, 2, 2, 3, 4, 5, 2][kind as usize] {
                regs.push(s18::rgbaq([next(256), next(256), next(256), next(256)], 1.0));
                if textured {
                    regs.push(s18::uv(next(64 * 16), next(32 * 16)));
                }
                regs.push(xyz2(next(1100) as i32 - 40, next(600) as i32 - 40, next(u32::MAX >> 8) << 8 | next(256)));
            }
        }
        ops.push(Op::Regs(regs));
        f.read(&mut ops);
        z.read(&mut ops);
        grade(&Script { name: "generated", ops });
    }
}

#[test]
fn the_programs_assemble_and_their_records_end_in_the_done_word() {
    for script in s19::all() {
        let (source, expected) = s17::program(&script).finish();
        let words = emusen_mips::assemble(&source, s6::ORIGIN, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{}: {e:?}", script.name));
        assert!(words.len() < (1 << 18) - 64, "{}: the code fits below the record", script.name);
        assert!(expected.len() < (1 << 18), "{}: the record fits below the packets' buffer", script.name);
        assert!(sapphire::media::Media::read(&sapphire::media::program(&words, s6::ORIGIN)).is_ok(), "{}", script.name);
        assert_eq!(expected.len(), 2 * s17::expected(&script).len(), "{}", script.name);
    }
}
