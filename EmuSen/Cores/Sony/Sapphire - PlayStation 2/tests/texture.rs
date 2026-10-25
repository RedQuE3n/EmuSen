//! Stage S18, the GS's texture mapping and fog (Sapphire_Native.md §31): the project's programs driven through the
//! GS's host interface and graded by the manual's model (`tests/s18/manual.rs` beneath S17's), many generated textured
//! primitives drawn by both, and the programs' assembly for the referees.

mod s6;
mod s11;
mod s16;
mod s17;
mod s18;

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
fn every_texture_format_is_read_through_its_layout_and_converted_with_texa() {
    grade(&s18::formats());
}

#[test]
fn the_clut_buffer_loads_as_cld_says_and_keeps_what_it_loaded() {
    grade(&s18::clut());
}

#[test]
fn every_texture_function_with_tcc_follows_the_table_of_3_4_9() {
    grade(&s18::tfx());
}

#[test]
fn uv_coordinates_are_interpolated_exactly_on_every_primitive() {
    grade(&s18::uv_program());
}

#[test]
fn st_and_q_divide_exactly_after_interpolation() {
    grade(&s18::stq());
}

#[test]
fn bilinear_lod_and_the_mipmap_filters_work_in_sixteenths() {
    grade(&s18::filter());
}

#[test]
fn every_wrap_mode_acts_on_whole_texels_at_each_level() {
    grade(&s18::clamp_program());
}

#[test]
fn texa_expands_16_and_24_bit_alpha_with_aem() {
    grade(&s18::texa_program());
}

#[test]
fn fog_blends_fogcol_by_the_interpolated_coefficient() {
    grade(&s18::fog_program());
}

/// Textured primitives from a generator: every kind, UV or STQ (Q of either sign), every texture format with a CLUT
/// of either storage mode, every function, filter, LOD source and wrap mode, TEXA and fog, drawn into a page over
/// generated memory: the source and the model agree on every pixel.
#[test]
fn generated_textured_primitives_draw_as_the_model_draws_them() {
    use s16::manual::{CT16, CT16S, CT24, CT32, T4, T4HH, T4HL, T8, T8H, Z16, Z16S, Z24, Z32};
    use s18::*;
    let mut x = 0x1357_2468u32;
    let mut next = |m: u32| {
        x = x.wrapping_mul(0x41C6_4E6D).wrapping_add(0x3039);
        (x >> 8) % m
    };
    let formats = [CT32, CT24, CT16, CT16S, T8, T4, T8H, T4HL, T4HH, Z32, Z24, Z16, Z16S];
    for round in 0..6u32 {
        let mut ops = Vec::new();
        s18::begin(&mut ops);
        fill(&mut ops, 300, 16, 0x5151_1800 + round);
        let f = page(&mut ops, 200, 0x6161_1800 + round);
        let mut regs = Vec::new();
        for _ in 0..120 {
            let psm = formats[next(13) as usize];
            let wide = matches!(psm, T8 | T4);
            let t = Tex {
                tbp: 32 * (300 + next(8)) + next(32),
                tbw: if wide { 2 } else { 1 + next(2) },
                psm,
                tw: 2 + next(5),
                th: 2 + next(5),
                tcc: next(2) == 1,
                tfx: next(4),
                cbp: 32 * (308 + next(8)) + next(4),
                cpsm: [CT32, CT16, CT16S][next(3) as usize],
                csm2: false,
                csa: next(32),
                cld: [0, 1, 1, 2, 3, 4, 5][next(7) as usize],
            };
            let t = if next(4) == 0 { Tex { csm2: true, cpsm: CT16, csa: 0, ..t } } else { t };
            let k = next(160) as i32 - 48;
            regs.extend([
                (r::TEXCLUT, texclut(next(4), next(8), next(32))),
                (r::TEXA, texa(next(256), next(2) == 1, next(256))),
                (r::FOGCOL, u64::from(next(1 << 24))),
                (r::CLAMP_1, clamp(next(4), next(4), next(64), next(64), next(64), next(64))),
                (r::TEX1_1, tex1(next(2) == 1, next(7), next(2) == 1, next(6), false, next(4), k)),
                (r::MIPTBP1_1, miptbp([(300 + next(16), 1), (300 + next(16), 1), (300 + next(16), 2)])),
                (r::MIPTBP2_1, miptbp([(300 + next(16), 1), (300 + next(16), 1), (300 + next(16), 1)])),
                (r::TEX0_1, t.bits()),
            ]);
            let kind = u64::from(next(7));
            let uv_mode = next(2) == 1;
            let attributes = TME | if uv_mode { FST } else { 0 } | if next(2) == 1 { FGE } else { 0 } | if next(2) == 1 { GOURAUD } else { 0 };
            regs.push(s17::prim(kind, attributes));
            let neg = next(4) == 0;
            for _ in 0..[1, 2, 2, 3, 4, 5, 2][kind as usize] {
                let q = (next(3800) + 200) as f32 / 1000.0 * if neg { -1.0 } else { 1.0 };
                let s = (next(5000) as f32 - 2000.0) / 1000.0 * q;
                let tt = (next(5000) as f32 - 2000.0) / 1000.0 * q;
                regs.push(rgbaq([next(256), next(256), next(256), next(256)], q));
                regs.push(if uv_mode { uv(next(16384), next(16384)) } else { st(s, tt) });
                let small = next(3) == 0;
                let (vx, vy) = if small { (next(160) as i32 + 300, next(160) as i32 + 200) } else { (next(1200) as i32 - 80, next(700) as i32 - 80) };
                regs.push(xyzf2(vx, vy, next(1 << 24), next(256)));
            }
        }
        ops.push(Op::Regs(regs));
        f.read(&mut ops);
        grade(&Script { name: "generated", ops });
    }
}

#[test]
fn the_programs_assemble_and_their_records_end_in_the_done_word() {
    for script in s18::all() {
        let (source, expected) = s17::program(&script).finish();
        let words = emusen_mips::assemble(&source, s6::ORIGIN, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{}: {e:?}", script.name));
        assert!(words.len() < (1 << 18) - 64, "{}: the code fits below the record", script.name);
        assert!(expected.len() < (1 << 18), "{}: the record fits below the packets' buffer", script.name);
        assert!(sapphire::media::Media::read(&sapphire::media::program(&words, s6::ORIGIN)).is_ok(), "{}", script.name);
        assert_eq!(expected.len(), 2 * s17::expected(&script).len(), "{}", script.name);
    }
}
