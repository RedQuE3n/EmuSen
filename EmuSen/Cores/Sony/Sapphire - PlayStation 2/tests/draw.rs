//! Stage S17, the GS's primitives, shading, scissoring and depth (Sapphire_Native.md §27): the project's programs
//! driven through the GS's host interface and graded by the manual's model (`tests/s17/manual.rs`), many generated
//! primitives drawn by both, and the programs' assembly for the referees.

mod s6;
mod s11;
mod s16;
mod s17;

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
fn points_take_the_nearest_pixel_a_half_rounding_up() {
    grade(&s17::points());
}

#[test]
fn sprites_cover_their_top_and_left_sides_and_not_their_bottom_and_right() {
    grade(&s17::sprites());
}

#[test]
fn lines_draw_the_diamonds_they_pass_through_but_their_end() {
    grade(&s17::lines());
}

#[test]
fn triangles_draw_left_and_top_sides_and_strips_fans_xyz3_and_prmode_as_section_3_2_says() {
    grade(&s17::triangles());
}

#[test]
fn gouraud_colour_and_z_are_interpolated_exactly_and_rounded_down() {
    grade(&s17::gouraud());
}

#[test]
fn every_frame_format_is_written_through_its_layout_under_fbmsk() {
    grade(&s17::formats());
}

#[test]
fn every_depth_test_holds_in_psmz32_and_psmz24() {
    grade(&s17::depth32());
}

#[test]
fn every_depth_test_holds_in_psmz16_and_psmz16s() {
    grade(&s17::depth16());
}

#[test]
fn z_beyond_a_formats_range_is_clamped() {
    grade(&s17::zrange());
}

#[test]
fn scissoring_is_inclusive_and_scanmsk_skips_rows() {
    grade(&s17::scissors());
}

#[test]
fn the_vertex_queue_prmode_and_the_contexts_follow_section_3_2() {
    grade(&s17::kick());
}

/// Primitives from a generator, every kind with both shadings, vertices anywhere in sixteenths about a 64×32 page and a
/// little outside it (in odd rounds on halves and whole pixels only), Z over the whole range, drawn with GEQUAL into
/// PSMZ32 over generated data: the source and the model agree on every pixel and every Z.
#[test]
fn generated_primitives_draw_as_the_model_draws_them() {
    use s17::*;
    let mut x = 0x2468_1357u32;
    let mut next = |m: u32| {
        x = x.wrapping_mul(0x41C6_4E6D).wrapping_add(0x3039);
        (x >> 8) % m
    };
    for round in 0..6u32 {
        let mut ops = Vec::new();
        begin(&mut ops);
        let f = Buf::new(200, 1, s16::manual::CT32, 64, 32);
        let z = Buf::new(201, 1, s16::manual::Z32, 64, 32);
        f.fill(&mut ops, 0x5151_0000 + round);
        z.fill(&mut ops, 0x6161_0000 + round);
        target(&mut ops, &f, Some((&z, false)), test(true, GEQUAL));
        let mut regs = Vec::new();
        for _ in 0..400 {
            let kind = u64::from(next(7));
            regs.push(prim(kind, if next(2) == 1 { GOURAUD } else { 0 }));
            let small = next(3) == 0;
            for _ in 0..[1, 2, 2, 3, 4, 5, 2][kind as usize] {
                regs.push(rgba([next(256), next(256), next(256), next(256)]));
                let (vx, vy) = if small { (next(160) as i32 + 300, next(160) as i32 + 200) } else { (next(1200) as i32 - 80, next(700) as i32 - 80) };
                // Odd rounds put every vertex on a half or a whole pixel, where the rules' ties fall.
                let snap = |v: i32| if round % 2 == 1 { v & !7 } else { v };
                regs.push(xyz2(snap(vx), snap(vy), next(u32::MAX >> 8) << 8 | next(256)));
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
    for script in s17::all() {
        let (source, expected) = s17::program(&script).finish();
        let words = emusen_mips::assemble(&source, s6::ORIGIN, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{}: {e:?}", script.name));
        assert!(words.len() < (1 << 18) - 64, "{}: the code fits below the record", script.name);
        assert!(expected.len() < (1 << 18), "{}: the record fits below the packets' buffer", script.name);
        assert!(sapphire::media::Media::read(&sapphire::media::program(&words, s6::ORIGIN)).is_ok(), "{}", script.name);
        assert_eq!(expected.len(), 2 * s17::expected(&script).len(), "{}", script.name);
    }
}
