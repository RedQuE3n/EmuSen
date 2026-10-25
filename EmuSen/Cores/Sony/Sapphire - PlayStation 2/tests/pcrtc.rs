//! Stage S20, the PCRTC (Sapphire_Native.md §29): the project's display programs driven on the board and their pictures
//! graded by the manual's model (`s20::manual`), a register changed in the middle of a field, and the timing program
//! run on the interpreter against the documents' fields.

mod s6;
mod s11;
mod s16;
mod s20;

use sapphire::board::{Board, System};
use sapphire::clock::NTSC;
use sapphire::ee::Bus;
use sapphire::gs::{HostInterface, privileged};
use s16::{Op, Script, ga};
use s20::manual;

const SPR: usize = 16 << 10;

fn at(g: u32) -> u32 {
    privileged::address(g)
}

/// The script on the board, the host interface driven directly; the readback, and the time of each privileged write.
fn drive(b: &mut Board, spr: &mut [u8], script: &Script) -> (Vec<u64>, Vec<(u32, u64)>) {
    let mut out = Vec::new();
    let mut writes = Vec::new();
    for op in &script.ops {
        match op {
            Op::Regs(list) => list.iter().for_each(|&(r, v)| b.gs.write(r, v)),
            Op::Image { seed, qwords } => s16::units(*seed, *qwords).into_iter().for_each(|u| b.gs.image(u)),
            Op::Priv(g, v) => {
                writes.push((*g, b.now));
                b.write(at(*g), 8, u128::from(*v));
            }
            Op::Finish => {
                b.write(at(ga::CSR), 8, 2);
                b.gs.write(s16::r::FINISH, 0);
                b.write(at(ga::CSR), 8, 2);
            }
            Op::Readback(q) => {
                b.write(at(ga::BUSDIR), 8, 1);
                out.extend((0..2 * q).map(|_| b.gs.read().unwrap_or(0)));
                b.write(at(ga::BUSDIR), 8, 0);
            }
            Op::Wait(g, mask) => {
                let limit = b.now + 2 * NTSC.cycles(NTSC.field(true));
                while b.read(at(*g), 8) as u64 & mask == 0 {
                    assert!(b.now < limit, "{}: waited two fields for {mask:x}", script.name);
                    let t = b.now + 200;
                    b.run_to(t, spr);
                }
            }
            _ => panic!("{}: an op the display programs do not use", script.name),
        }
        b.sync_gs();
    }
    (out, writes)
}

/// Runs until `n` more fields have begun.
fn fields(b: &mut Board, spr: &mut [u8], n: u64) {
    let want = b.pcrtc.fields + n;
    while b.pcrtc.fields < want {
        let t = b.now + 500;
        b.run_to(t, spr);
    }
}

fn differ(name: &str, w: u32, got: &[u8], want: &[u8]) -> String {
    let bad: Vec<_> = got.chunks(4).zip(want.chunks(4)).enumerate().filter(|(_, (g, w))| g != w).map(|(i, (g, v))| (i as u32 % w, i as u32 / w, g.to_vec(), v.to_vec())).collect();
    format!("{name}: {} of {} pixels differ (sizes {} and {}), first {:?}", bad.len(), want.len() / 4, got.len(), want.len(), &bad[..bad.len().min(3)])
}

#[test]
fn the_display_programs_pictures_equal_the_manuals() {
    for script in s20::displays() {
        let mut b = Board::new(NTSC);
        let mut spr = vec![0u8; SPR];
        let (record, _) = drive(&mut b, &mut spr, &script);
        let want_record: Vec<u64> = s16::expected(&script).into_iter().map(|w| w.unwrap()).collect();
        assert!(record == want_record, "{}: the memory read back", script.name);
        fields(&mut b, &mut spr, 3);
        let gs = s20::memory(&script);
        let interlaced = b.pcrtc.timing.mode(b.now).1;
        let (x0, pitch, w, y0, h) = manual::rectangle(&gs);
        let g = b.pcrtc.picture.geometry;
        assert_eq!((g.x0, g.pitch, g.width, u64::from(g.y0), g.height), (x0, pitch, w, y0, h), "{}: the picture's rectangle", script.name);
        let want = manual::picture(&gs, interlaced);
        assert!(b.pcrtc.picture.rgba == want, "{}", differ(script.name, w, &b.pcrtc.picture.rgba, &want));
    }
}

#[test]
fn a_register_changed_mid_field_shows_from_the_next_line() {
    let script = s20::midframe();
    let mut b = Board::new(NTSC);
    let mut spr = vec![0u8; SPR];
    let (_, writes) = drive(&mut b, &mut spr, &script);
    let switch = writes.last().filter(|w| w.0 == ga::DISPFB1).expect("the switch").1;
    let start = b.pcrtc.timing.field(switch).start;
    fields(&mut b, &mut spr, 1);
    let mut gs = s20::memory(&script);
    let before = {
        gs.write_privileged(ga::DISPFB1, s20::dispfb(0, 1, s16::manual::CT32, 0, 0));
        manual::picture(&gs, false)
    };
    gs.write_privileged(ga::DISPFB1, s20::dispfb(1, 1, s16::manual::CT32, 0, 0));
    let after = manual::picture(&gs, false);
    let (_, _, w, y0, h) = manual::rectangle(&gs);
    let row = (w * 4) as usize;
    let mut want = Vec::new();
    let mut first_after = None;
    for r in 0..h as u64 {
        let read = manual::cycles(start, (y0 + r) * manual::LINE);
        let from = if read <= switch { &before } else { first_after.get_or_insert(y0 + r); &after };
        want.extend_from_slice(&from[r as usize * row..(r as usize + 1) * row]);
    }
    let first = first_after.expect("the switch inside the rectangle");
    assert!((s20::MIDFRAME_DY as u64..(s20::MIDFRAME_DY + 32) as u64).contains(&first), "switched at raster {first}");
    assert_eq!(first, manual::VSYNC + s20::SWITCH_AFTER as u64 + 1, "the line after the fortieth H-sync past the V-sync");
    assert!(b.pcrtc.picture.rgba == want, "{}", differ("s20-midframe", w, &b.pcrtc.picture.rgba, &want));
}

#[test]
fn the_picture_is_described_to_the_frontend_with_its_size_and_shape() {
    let mut b = Board::new(NTSC);
    let info = b.pcrtc.picture.frame_info();
    assert_eq!((info.width, info.height, info.aspect_num, info.aspect_den), (640, 263, 4, 3), "no circuit on: the mode's picture, a non-interlaced field's rasters");
    let mut spr = vec![0u8; SPR];
    let script = &s20::displays()[4];
    drive(&mut b, &mut spr, script);
    fields(&mut b, &mut spr, 2);
    let info = b.pcrtc.picture.frame_info();
    assert_eq!((info.width, info.height, info.stride), (64, 64, 256), "s20-field: 64 pixels of 4 VCK, 64 rasters woven");
    assert_eq!(b.pcrtc.picture.rgba.len(), 64 * 64 * 4);
    b.set_mode(sapphire::pcrtc::timing::DTV1080I);
    fields(&mut b, &mut spr, 2);
    assert_eq!(b.pcrtc.picture.frame_info().aspect_num, 16, "1080i's shape");
}

#[test]
fn the_even_field_by_csr_reads_the_even_rasters() {
    let script = &s20::displays()[4];
    let mut b = Board::new(NTSC);
    let mut spr = vec![0u8; SPR];
    drive(&mut b, &mut spr, script);
    fields(&mut b, &mut spr, 2);
    let before = b.pcrtc.picture.rgba.clone();
    let csr = at(ga::CSR);
    loop {
        b.write(csr, 8, 8);
        while b.read(csr, 8) & 8 == 0 {
            let t = b.now + 200;
            b.run_to(t, &mut spr);
        }
        if b.read(csr, 8) & 1 << 13 == 0 {
            break;
        }
    }
    b.write(at(ga::DISPFB1), 8, u128::from(s20::dispfb(8, 1, s16::manual::CT32, 0, 0)));
    fields(&mut b, &mut spr, 1);
    let g = b.pcrtc.picture.geometry;
    let row = (g.width * 4) as usize;
    for r in 0..g.height as usize {
        let got = &b.pcrtc.picture.rgba[r * row..(r + 1) * row];
        if (g.y0 as usize + r) % 2 == 0 {
            assert!(got.chunks(4).all(|p| p == [0, 0, 0, 255]), "raster {}: the even field shows the empty buffer", g.y0 as usize + r);
        } else {
            assert_eq!(got, &before[r * row..(r + 1) * row], "raster {}: the odd field's rows kept", g.y0 as usize + r);
        }
    }
    assert!(!b.pcrtc.picture.odd);
}

fn assemble(source: &str) -> Vec<u32> {
    emusen_mips::assemble(source, s6::ORIGIN, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{e:?}"))
}

/// `s20-timing` on the interpreter: the record's doublewords.
fn timing_record() -> Vec<u64> {
    let (source, expected) = s20::timing().finish();
    let mut sys = System::new(s6::ORIGIN);
    sys.board.map.load(s6::ORIGIN, &assemble(&source));
    let n = sys.run(400_000_000, &mut |s| s.peek(s6::DONE_AT, 4) as u32 == s6::DONE).unwrap_or_else(|h| panic!("{h:?}"));
    assert!(n < 400_000_000, "no done word, pc {:08x}", sys.ee.pc);
    (0..expected.len() / 2).map(|i| sys.peek(s6::RECORD + 8 * i as u32, 8) as u64).collect()
}

#[test]
fn the_timing_program_sees_the_documents_fields() {
    let d = timing_record();
    let per = s20::PER_SETTING;
    let line = manual::cycles(0, manual::LINE) as i64;
    for (s, setting) in s20::SETTINGS.iter().enumerate() {
        let r = &d[s * per..(s + 1) * per];
        let interlaced = setting.is_some_and(|v| v & 1 == 1);
        let f = |k: usize, i: usize| r[7 * k + i] as i64;
        for k in 0..s20::FIELDS - 1 {
            let len = (f(k + 1, 0) - f(k, 0)) as u32 as i64;
            let ticks = (f(k + 1, 1) - f(k, 1)) & 0xFFFF;
            let want_len = manual::cycles(0, manual::field_vck(interlaced) * 2) as i64 / 2;
            assert!((len - want_len).abs() < line / 2 + 100, "setting {s}, field {k}: {len} EE cycles, about {want_len}");
            let lines = if interlaced { [262, 263] } else { [263, 263] };
            assert!(lines.contains(&ticks), "setting {s}, field {k}: {ticks} H-blanks");
            let vblank = (f(k, 5) - f(k, 0)) as u32 as i64;
            assert!((vblank - manual::VBLANK as i64 * line).abs() < 200, "setting {s}: V-blank {vblank}");
            let field_bits = (f(k, 4), f(k, 6));
            assert_ne!(field_bits.0, field_bits.1, "setting {s}: FIELD changes inside V-blank, at the V-sync, as on both referees");
            assert_ne!(f(k, 4), f(k + 1, 4), "setting {s}: FIELD alternates");
        }
        let gated = (r[3 + 7] - r[3]) as i64 & 0xFFFF;
        let all = (r[2 + 7] - r[2]) as i64 & 0xFFFF;
        let blank = 1.0 - gated as f64 / all as f64;
        assert!((blank - manual::HBLANK as f64 / manual::LINE as f64).abs() < 0.01, "setting {s}: H-blank {blank:.3} of the line");
        let base = 7 * s20::FIELDS;
        let hs: Vec<i64> = r[base + 1..base + 1 + s20::TICKS].iter().map(|&x| x as i64).collect();
        assert!(hs.windows(2).skip(1).all(|p| (p[1] - p[0] - line).abs() < 60), "setting {s}: H-syncs a line apart {hs:?}");
        assert!(matches!(r[base + 1 + 2 * s20::TICKS], 262 | 263), "setting {s}: {} H-syncs a field", r[base + 1 + 2 * s20::TICKS]);
        let (vs, fl) = (r[per - 3] as u32 as i32 as i64, r[per - 2] as u32 as i32 as i64);
        assert!((vs - manual::VSYNC as i64 * line).abs() < 120 && (fl - vs).abs() < 60, "setting {s}: VSINT {vs} and FIELD {fl} after V-blank's start");
    }
}
