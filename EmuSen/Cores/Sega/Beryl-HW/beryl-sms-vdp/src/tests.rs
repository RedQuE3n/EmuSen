//! The chip's tests (Beryl_SmsVdp.md §7): the port protocol, the counters and interrupts, the picture drawn to the dot,
//! the variants, the state and rendering's neutrality.

use emusen_native::{StateReader, StateWriter};

use crate::timing::{LINE_CLOCKS, LINE_DOT, PIXEL_0_DOT};
use crate::{Vdp, Variant};

fn reg(v: &mut Vdp, r: u8, d: u8) {
    v.write_control(d);
    v.write_control(0x80 | r);
}

fn address(v: &mut Vdp, a: u16) {
    v.write_control(a as u8);
    v.write_control((a >> 8) as u8);
}

/// The clock at `dot` of `line`, counting lines from the power-on line, the frame's last.
fn at(line: u32, dot: u32) -> u64 {
    (line + 1) as u64 * LINE_CLOCKS + dot as u64 * 2
}

fn state(v: &Vdp) -> Vec<u8> {
    let mut w = StateWriter::counter();
    v.write_state(&mut w);
    let mut out = vec![0; w.len()];
    v.write_state(&mut StateWriter::new(&mut out));
    out
}

#[test]
fn the_command_word_sets_the_address_low_byte_first_and_code_0_fills_the_buffer() {
    let mut v = Vdp::new(Variant::SMS2, false);
    v.vram[0x1234] = 0xAA;
    v.vram[0x1235] = 0xBB;
    address(&mut v, 0x1234);
    assert_eq!((v.address, v.code, v.buffer), (0x1235, 0, 0xAA), "code 0 reads ahead and steps");
    assert_eq!((v.read_data(), v.read_data()), (0xAA, 0xBB));
    v.write_control(0x56);
    assert_eq!(v.address & 0xFF, 0x56, "the first byte lands at once");
    v.read_control();
    v.write_control(0x7F);
    v.write_control(0x40);
    assert_eq!((v.address, v.code), (0x007F, 1), "a status read reset the latch, so $7F was a first byte");
    v.write_data(0x11);
    assert_eq!((v.vram[0x007F], v.buffer), (0x11, 0x11), "a write fills the buffer");
    address(&mut v, 0x7FFF);
    v.write_data(1);
    assert_eq!((v.vram[0x3FFF], v.address), (1, 0), "the address wraps at $3FFF");
}

#[test]
fn registers_take_the_first_byte_and_numbers_11_to_15_do_nothing() {
    let mut v = Vdp::new(Variant::SMS2, false);
    reg(&mut v, 7, 0x0C);
    reg(&mut v, 0x3A, 0x05);
    assert_eq!((v.regs[7], v.regs[10]), (0x0C, 0x05), "bits 4-5 of the register number are ignored");
    let before = v.regs;
    reg(&mut v, 0x0B, 0xFF);
    reg(&mut v, 0x0F, 0xFF);
    assert_eq!(v.regs, before);
    assert_eq!(v.code, 2);
}

#[test]
fn cram_takes_six_bits_a_byte_and_the_game_gear_a_word_through_its_latch() {
    let mut v = Vdp::new(Variant::SMS2, false);
    address(&mut v, 0xC000 | 0x21);
    v.write_data(0xFF);
    assert_eq!(v.cram[1], 0x3F, "CRAM is 32 bytes of six bits");
    let mut g = Vdp::new(Variant::GG, false);
    g.gg_mode = true;
    address(&mut g, 0xC000);
    g.write_data(0xFF);
    assert_eq!(g.cram[0], 0, "an even address only latches");
    address(&mut g, 0xC021);
    g.write_data(0x0F);
    assert_eq!((g.cram[0x20], g.cram[0x21], g.cram[0]), (0xFF, 0x0F, 0), "the odd write stores the latch and itself");
}

#[test]
fn the_frame_interrupt_rises_as_line_c1_begins_and_a_status_read_clears_it() {
    let mut v = Vdp::new(Variant::SMS2, false);
    reg(&mut v, 1, 0x20);
    v.run(at(193, 0) - 1);
    assert!(!v.irq(), "not while line $C0 lasts");
    v.run(at(193, 0));
    assert!(v.irq() && v.read_v() == 0xC1);
    assert_eq!(v.read_control() & 0x80, 0x80);
    assert!(!v.irq() && v.read_control() & 0x80 == 0);
    reg(&mut v, 1, 0x00);
    v.run(at(193 + 262, 0));
    assert!(!v.irq() && v.frame_flag, "the flag is set with its enable clear, and holds no INT");
    reg(&mut v, 1, 0x20);
    assert!(v.irq(), "setting the enable raises INT for a pending flag");
}

#[test]
fn the_line_counter_steps_on_lines_0_to_192_and_its_underflow_is_the_line_interrupt() {
    for every in [1u32, 3] {
        let mut v = Vdp::new(Variant::SMS2, false);
        reg(&mut v, 0, 0x16);
        reg(&mut v, 10, every as u8 - 1);
        let mut lines = Vec::new();
        for line in 0..262 {
            v.run(at(line, LINE_DOT));
            if v.irq() {
                lines.push(v.read_v());
                v.read_control();
            }
        }
        let want: Vec<u8> = (0..=192).filter(|l| l % every == every - 1).map(|l| l as u8).collect();
        assert_eq!(lines, want, "register 10 at {}: lines through 192", every - 1);
    }
}

#[test]
fn the_y_scroll_is_taken_as_a_frame_begins_and_the_x_scroll_as_each_line_does() {
    let mut v = screen(Variant::SMS2);
    v.run(at(262 + 100, 100));
    reg(&mut v, 9, 8);
    reg(&mut v, 8, 4);
    assert_eq!((v.vscroll, v.hscroll), (0, 0), "neither at once");
    v.run(at(262 + 101, LINE_DOT));
    assert_eq!((v.vscroll, v.hscroll), (0, 4), "the X scroll for the next line");
    v.run(at(262 * 2, 0));
    assert_eq!(v.vscroll, 8, "the Y scroll at the next frame");
}

#[test]
fn the_h_counter_is_latched_on_th_rising_and_holds_its_value() {
    let mut v = Vdp::new(Variant::SMS2, false);
    v.run(at(10, PIXEL_0_DOT + 101));
    v.latch_h();
    assert_eq!(v.read_h(), 56, "pixel 101 is H counter 111, and the latch takes it a dot later, 112");
    v.run(at(11, 0));
    assert_eq!(v.read_h(), 56);
}

/// A 192-line display of tile 1 everywhere, whose rows are colour 1 on the left half and colour 2 on the right, and
/// CRAM entries 1 and 2 at $11 and $22.
fn screen(variant: Variant) -> Vdp {
    let mut v = Vdp::new(variant, false);
    for r in [(0, 0x06), (1, 0x40), (2, 0xFF), (3, 0xFF), (4, 0xFF), (5, 0xFF), (6, 0xFB), (7, 0x00)] {
        reg(&mut v, r.0, r.1);
    }
    for a in (0x3800..0x3F00).step_by(2) {
        v.vram[a] = 1;
    }
    for row in 0..8 {
        v.vram[32 + row * 4] = 0xF0;
        v.vram[32 + row * 4 + 1] = 0x0F;
    }
    v.vram[0x3F00] = 0xD0;
    v.cram[1] = 0x11;
    v.cram[2] = 0x22;
    v.cram[16] = 0x33;
    v.cram[17] = 0x2A;
    v
}

#[test]
fn the_picture_is_cram_entries_and_a_write_takes_effect_at_its_dot() {
    let mut v = screen(Variant::SMS2);
    v.run(at(262 + 50, PIXEL_0_DOT + 100));
    address(&mut v, 0xC001);
    v.write_data(0x15);
    v.run(at(262 + 262, 0));
    let p = v.picture();
    assert_eq!(v.height, 192);
    assert_eq!((p[49 * 256], p[49 * 256 + 4]), (0x11, 0x22));
    assert_eq!((p[50 * 256 + 96], p[50 * 256 + 99], p[50 * 256 + 100], p[50 * 256 + 104]), (0x11, 0x11, 0x22, 0x15), "entry 1 changed under pixel 100: the next colour-1 pixel, 104, has it");
    assert_eq!(p[51 * 256], 0x15);
}

#[test]
fn the_left_column_mask_and_a_blanked_display_show_the_backdrop() {
    let mut v = screen(Variant::SMS2);
    reg(&mut v, 0, 0x26);
    v.run(at(262 + 10, 0));
    reg(&mut v, 1, 0x00);
    v.run(at(262 + 262, 0));
    let p = v.picture();
    assert_eq!((p[0], p[7], p[8]), (0x33, 0x33, 0x11));
    assert!(p[10 * 256..11 * 256].iter().all(|&e| e == 0x33), "the line the display went off on is the backdrop");
    let mut gg = screen(Variant::GG);
    reg(&mut gg, 0, 0x26);
    gg.run(at(262 + 1, 0));
    assert_eq!(gg.picture()[0], 0x11, "the Game Gear has no mask");
}

#[test]
fn sprites_draw_over_the_background_collide_and_overflow_past_eight() {
    let mut v = screen(Variant::SMS2);
    // Nine sprites on line 20, two of them at the same place.
    for i in 0..9 {
        v.vram[0x3F00 + i] = 19;
        v.vram[0x3F80 + 2 * i] = if i == 1 { 40 } else { 40 + 20 * i as u8 };
        v.vram[0x3F81 + 2 * i] = 2;
    }
    v.vram[0x3F09] = 0xD0;
    for row in 0..8 {
        v.vram[64 + row * 4] = 0xFF;
    }
    v.run(at(262 + 19, 0));
    v.read_control();
    v.run(at(262 + 20, 0));
    assert!(!v.collision && v.overflow, "the overflow flag as the line begins");
    v.run(at(262 + 20, PIXEL_0_DOT + 40));
    assert_eq!(v.read_control() & 0x60, 0x40, "the collision not yet");
    v.run(at(262 + 20, PIXEL_0_DOT + 42));
    assert_eq!(v.read_control() & 0x20, 0x20, "the collision at pixel 40, once it is drawn");
    v.run(at(262 + 262, 0));
    let p = &v.picture()[20 * 256..21 * 256];
    assert_eq!((p[39], p[40], p[47], p[48]), (0x22, 0x2A, 0x2A, 0x11), "sprite colour 1 is CRAM entry 17");
}

#[test]
fn the_taller_displays_are_the_315_5246_s_and_the_first_chip_has_none() {
    for (variant, m1, m3, lines) in [(Variant::SMS2, 1, 0, 224), (Variant::SMS2, 0, 1, 240), (Variant::SMS2, 1, 1, 192), (Variant::SMS1, 0, 1, 192), (Variant::GG, 0, 1, 192), (Variant::GG, 1, 0, 224)] {
        let mut v = Vdp::new(variant, true);
        reg(&mut v, 0, 0x06);
        reg(&mut v, 1, 0x40 | m1 << 4 | m3 << 3);
        assert_eq!(v.active_lines(), lines, "{variant:?} {m1} {m3}");
    }
    let mut v = Vdp::new(Variant::SMS1, false);
    reg(&mut v, 0, 0x06);
    reg(&mut v, 1, 0x50);
    assert!(!v.mode4(), "M1 on the 315-5124 is the invalid text mode");
}

#[test]
fn a_state_round_trips_and_carries_on_alike() {
    let mut a = screen(Variant::SMS1);
    reg(&mut a, 1, 0x60);
    reg(&mut a, 8, 5);
    reg(&mut a, 9, 3);
    a.run(at(262 + 100, 77));
    a.latch_h();
    let saved = state(&a);
    let mut b = Vdp::new(Variant::SMS1, false);
    assert!(b.read_state(&mut StateReader::new(&saved)).unwrap());
    assert_eq!(state(&b), saved);
    for v in [&mut a, &mut b] {
        v.run(at(262 * 3, 5));
    }
    assert_eq!((state(&a), a.picture()), (state(&b), b.picture()));
    let mut other = Vdp::new(Variant::SMS2, false);
    assert!(!other.read_state(&mut StateReader::new(&saved)).unwrap(), "another chip's state is refused");
    assert!(other.read_state(&mut StateReader::new(&saved[..10])).is_err());
}

#[test]
fn skipping_the_picture_leaves_the_state_as_drawing_it() {
    let mut a = screen(Variant::SMS2);
    let mut b = screen(Variant::SMS2);
    b.skip = true;
    for v in [&mut a, &mut b] {
        for i in 0..9 {
            v.vram[0x3F00 + i] = 30;
            v.vram[0x3F81 + 2 * i] = 1;
        }
        v.run(at(262 * 2, 0));
    }
    assert_eq!(state(&a), state(&b));
    assert!(a.overflow && a.collision);
}
