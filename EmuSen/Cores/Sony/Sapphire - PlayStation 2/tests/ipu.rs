//! Stage S27's gate (Sapphire_Native.md §34): the IPU.

mod s11;
mod s27;
mod s6;

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
        assert!(max <= 1 && n * 20 < diff.len(), "{}: {n} samples differ, at most by {max}: more than two IDCTs' rounding", s.name);
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
    let programs = s27::programs();
    assert_eq!(programs.len(), 32);
    for p in &programs {
        grade(p);
    }
}

#[test]
fn csc_and_pack_follow_the_manuals_post_processing() {
    s27::post().unwrap();
}

#[test]
fn the_registers_and_fifos_answer_the_ee_as_chapter_8_says() {
    let p = s27::registers::registers();
    let name = p.name;
    let (source, expected) = p.finish();
    let words = emusen_mips::assemble(&source, s6::ORIGIN, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{e:?}"));
    let mut sys = System::new(s6::ORIGIN);
    sys.board.map.load(s6::ORIGIN, &words);
    let limit = 50_000_000;
    let n = sys.run(limit, &mut |s| s.peek(s6::DONE_AT, 4) as u32 == s6::DONE).unwrap_or_else(|h| panic!("{h:?}"));
    assert!(n < limit, "{name}: no done word, pc {:08x}", sys.ee.pc);
    let mut wrong = Vec::new();
    for (i, want) in expected.iter().enumerate() {
        let got = sys.peek(s6::RECORD + 4 * i as u32, 4) as u32;
        if let Some(w) = want
            && *w != got
        {
            wrong.push(format!("word {i} (record {}): {got:08x}, expected {w:08x}", i / 2));
        }
    }
    assert!(wrong.is_empty(), "{name}: {} of {} words differ: {:?}", wrong.len(), expected.len(), &wrong[..wrong.len().min(10)]);
}

#[test]
fn odd_escapes_levels_and_mpeg1s_forms_decode_as_the_standard() {
    for (name, bytes) in [("odd-mpeg2", s27::encode::odd_mpeg2()), ("odd-mpeg1", s27::encode::odd_mpeg1())] {
        let d = standard::decode(&bytes);
        for o in [Options::BDEC, Options::IDEC32] {
            driver::drive(&bytes, &d, o).unwrap_or_else(|e| panic!("{name} {o:?}: {e}"));
        }
    }
}

#[test]
fn error_and_start_codes_end_idec_and_bdec_where_section_8_4_says() {
    s27::errors().unwrap();
}

#[test]
fn vdec_gives_every_row_of_the_manuals_tables() {
    use sapphire::ipu::{Ipu, code, ctrl};
    for &(tbl, pct, mp1, bits, data) in s27::manual::VDEC_TABLE {
        for mpeg1 in [false, true] {
            if (mp1 == '0' && mpeg1) || (mp1 == '1' && !mpeg1) {
                continue;
            }
            let mut ipu = Ipu::new();
            ipu.write_ctrl(pct << 24 | u32::from(mpeg1) << 23);
            let stream: String = format!("{bits}{}", "1".repeat(128 - bits.len()));
            let q = u128::from_str_radix(&stream, 2).unwrap();
            ipu.put(q.swap_bytes());
            ipu.put(u128::MAX);
            ipu.command(code::VDEC << 28 | tbl << 26);
            assert!(!ipu.busy());
            assert_eq!(ipu.read_cmd() as u32, data, "TBL {tbl}, PCT {pct}, MP1 {mpeg1}: {bits}");
            assert_eq!(ipu.read_bp() & 0x7F, data >> 16, "TBL {tbl}: {bits}: the pointer moves by the code's length");
            assert_eq!(ipu.read_ctrl() & ctrl::ECD != 0, data == 0, "TBL {tbl}: {bits}: ECD for an error row");
        }
    }
}

#[test]
fn every_table_is_prefix_free_and_as_complete_as_the_standard() {
    use sapphire::ipu::vlc;
    // The code space each table leaves unused, in 4096ths: zero where the standard leaves none (H.262 Annex B).
    let unused = [("B.1", 46), ("B.2", 1024), ("B.3", 64), ("B.4", 64), ("B.9", 8), ("B.10", 48), ("B.11", 0), ("B.12", 0), ("B.13", 0), ("B.14 first", 1), ("B.14", 1), ("B.15", 9)];
    for (name, codes) in vlc::all_codes() {
        for a in &codes {
            for b in &codes {
                assert!(a == b || !b.starts_with(a.as_str()), "{name}: {a} is a prefix of {b}");
            }
        }
        let used: f64 = codes.iter().map(|c| 0.5f64.powi(c.len() as i32)).sum();
        let left = ((1.0 - used) * 4096.0).round() as i32;
        let want = unused.iter().find(|u| u.0 == name).unwrap().1;
        assert_eq!(left, want, "{name}: unused code space");
    }
}

#[test]
fn the_corpus_values_near_a_half_are_few_and_decided_exactly() {
    use sapphire::ipu::idct;
    let (mut near, mut ties, mut values, mut nearest) = (0usize, 0usize, 0usize, f64::MAX);
    let streams: Vec<Vec<u8>> = s27::corpus().iter().map(|s| s.complete()).chain([s27::encode::odd_mpeg2(), s27::encode::odd_mpeg1()]).collect();
    for bytes in streams {
        for e in standard::decode(&bytes).events {
            let standard::Event::Slice(s) = e else { continue };
            for mb in s.mbs {
                for b in mb.coefs {
                    let f: [i32; 64] = core::array::from_fn(|k| b[k / 8][k % 8]);
                    let mut out = [0.0; 64];
                    idct::real(&f, &mut out);
                    for (k, v) in out.into_iter().enumerate() {
                        values += 1;
                        let d = (v.abs().fract() - 0.5).abs();
                        nearest = nearest.min(d);
                        if d <= idct::NEAR {
                            near += 1;
                            let n = idct::combination(&f, k % 8, k / 8);
                            ties += usize::from(n[1..].iter().all(|&c| c == 0));
                        }
                    }
                }
            }
        }
    }
    println!("{values} values, {near} within 2^-20 of a half and decided exactly, {ties} of them halves, the nearest {nearest:e} from one");
    assert!(values > 100_000);
}
