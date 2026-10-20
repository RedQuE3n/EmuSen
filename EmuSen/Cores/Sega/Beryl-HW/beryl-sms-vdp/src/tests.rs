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

/// The Game Gear's own mode with M2 and M1 set: a 224-line display, as the 315-5246's, with its taller name table and no
/// end marker; the LCD's window moves to lines 40-183 (D-21).
#[test]
fn the_game_gear_s_own_mode_takes_the_224_line_display_and_its_window_stays_in_the_frame() {
    let mut v = screen(Variant::GG);
    v.gg_mode = true;
    reg(&mut v, 1, 0x70);
    for a in (0x3700..0x3800).step_by(2) {
        v.vram[a] = 2;
    }
    for row in 0..8 {
        v.vram[64 + row * 4 + 2] = 0xFF;
    }
    v.cram[8..10].copy_from_slice(&[0x5A, 0x0A]);
    assert_eq!(v.active_lines(), 224);
    v.run(at(262 + 225, 0));
    assert!(v.irq() && v.read_v() == 0xE1, "the frame interrupt on line $E1, as a 224-line display's");
    v.run(at(262 + 262, 0));
    assert_eq!(v.height, 224);
    assert_eq!(v.picture()[0], 0x0A5A, "row 0 of the taller layout at $3700, tile 2's colour 4, as a 12-bit word");
    assert!(v.overflow, "the sprite at $3F00's Y of $D0 ends nothing, so the 63 after it fill line 1");
    assert_eq!((crate::gg_window_top(192), crate::gg_window_top(224)), (24, 40));
}

/// The Game Gear in the TMS9918's modes shows CRAM's entries 16-31, as words in its own mode; its border is the backdrop's
/// entry in either [R MacDonald §14].
#[test]
fn the_game_gear_shows_the_tms9918_s_codes_in_cram_16_to_31() {
    let mut v = Vdp::new(Variant::GG, false);
    v.gg_mode = true;
    for r in [(0, 0x00), (1, 0x40), (2, 0x0E), (3, 0x80), (4, 0x00), (7, 0xF4)] {
        reg(&mut v, r.0, r.1);
    }
    for row in 0..8 {
        v.vram[row] = 0xF0;
    }
    v.vram[0x2000] = 0x6C;
    v.cram[2 * 22..2 * 22 + 2].copy_from_slice(&[0x23, 0x01]);
    v.cram[2 * 28..2 * 28 + 2].copy_from_slice(&[0x56, 0x04]);
    v.cram[2 * 20..2 * 20 + 2].copy_from_slice(&[0x89, 0x07]);
    v.run(at(262 + 262, 0));
    assert_eq!((v.picture()[0], v.picture()[4]), (0x0123, 0x0456), "codes 6 and 12 as entries 22 and 28");
    assert_eq!(v.border(), 0x0789, "the backdrop, code 4, as entry 20");
    v.gg_mode = false;
    v.cram[20] = 0x15;
    assert_eq!(v.border(), 0x15, "a Master System cartridge's mode reads bytes");
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

/// The TMS9918A's Graphics I with name 0 everywhere, colour 6 on 12, and `sprites` 8 by 8 sprites of pattern 1 (all
/// set) on lines 50-57 at X 0, 20, 40...; the sprite table at $3B00, its patterns at $0000.
fn tms_screen(variant: Variant, sprites: usize) -> Vdp {
    let mut v = Vdp::new(variant, false);
    for (r, d) in [(0, 0x00), (1, 0xC0), (2, 0x0E), (3, 0x80), (4, 0x00), (5, 0x76), (6, 0x00), (7, 0xF4)] {
        reg(&mut v, r, d);
    }
    for row in 0..8 {
        v.vram[row] = 0xF0;
        v.vram[8 + row] = 0xFF;
    }
    v.vram[0x2000] = 0x6C;
    for i in 0..sprites {
        v.vram[0x3B00 + 4 * i..0x3B00 + 4 * i + 4].copy_from_slice(&[49, 20 * i as u8, 1, 0x0A]);
    }
    v.vram[0x3B00 + 4 * sprites] = 0xD0;
    v
}

#[test]
fn the_tms9918a_takes_bit_7_as_a_register_write_of_bits_2_to_0_and_has_no_cram() {
    let mut v = Vdp::new(Variant::TMS9918A, false);
    reg(&mut v, 0x0F, 0x5A);
    assert_eq!((v.regs[7], v.regs[0x0F & 7]), (0x5A, 0x5A), "register 15 is register 7");
    v.write_control(0x34);
    v.write_control(0xC2);
    assert_eq!((v.regs[2], v.code), (0x34, 2), "$C0 is a register write too");
    address(&mut v, 0xC010);
    v.write_data(0x77);
    assert_eq!((v.vram[0x0010], v.cram[0x10]), (0x77, 0), "no CRAM: the write lands in VRAM");
    assert_eq!(v.status_bits(), 0xFF);
}

#[test]
fn the_tms9918a_flags_the_fifth_sprite_while_the_frame_flag_is_clear_and_names_it() {
    let mut v = tms_screen(Variant::TMS9918A, 6);
    reg(&mut v, 0, 0x10);
    reg(&mut v, 10, 0);
    v.run(at(262 + 20, 0));
    assert!(!v.irq(), "no line interrupt on the TMS9918A");
    v.read_control();
    v.run(at(262 + 58, 0));
    assert_eq!(v.read_control(), 0x40 | 4, "the fifth sprite, number 4");
    v.run(at(262 * 2 + 58, 0));
    assert_eq!(v.read_control(), 0x80 | 6, "the frame flag was set through lines 50-57: no fifth sprite flagged, and the later lines' scans ended at the terminator, 6");
    v.vram[0x3B00 + 16] = 0xD0;
    v.run(at(262 * 3 + 58, 0));
    assert_eq!(v.read_control() & 0x5F, 4, "four sprites and the terminator at 4: the scan's end");
}

#[test]
fn the_tms9918a_draws_codes_and_the_sega_chips_their_fixed_colours_with_sprites_over_the_pattern_plane() {
    for (variant, want) in [(Variant::TMS9918A, [6u16, 12, 10, 4]), (Variant::SMS2, [0x01, 0x04, 0x05, 0x10]), (Variant::SMS1, [0x01, 0x04, 0x05, 0x10])] {
        let mut v = tms_screen(variant, 1);
        v.run(at(262 * 2, 0));
        let p = v.picture();
        assert_eq!((p[10 * 256], p[10 * 256 + 4], p[50 * 256 + 1], p[50 * 256 + 4]), (want[0], want[1], want[2], want[2]), "{variant:?}");
        reg(&mut v, 1, 0x80);
        v.run(at(262 * 3, 0));
        assert_eq!(v.picture()[0], want[3], "{variant:?}: blanked, the backdrop's code 4");
    }
}

#[test]
fn the_tms9918a_s_4k_mode_moves_the_cpu_s_cells_and_its_own_alike() {
    let mut v = tms_screen(Variant::TMS9918A, 0);
    address(&mut v, 0x4000 | 0x3E00);
    v.write_data(0xA5);
    assert_eq!(v.vram[0x3E00], 0xA5);
    reg(&mut v, 1, 0x40);
    address(&mut v, 0x3E00);
    assert_eq!(v.read_data(), v.vram[crate::tms::address_4k(0x3E00) as usize], "the 4K mode reads its own cell");
    address(&mut v, 0x4000 | 0x3E00);
    v.write_data(0x5A);
    assert_eq!((v.vram[crate::tms::address_4k(0x3E00) as usize], v.vram[0x3E00]), (0x5A, 0xA5), "a write reaches the 4K cell too");
    address(&mut v, 0x4000 | 0x0001);
    v.write_data(0x0F);
    assert_eq!(v.vram[1], 0x0F, "an address below $40 is the same cell in either mode");
}

#[test]
fn a_tms_mode_state_round_trips_and_skipping_its_picture_leaves_the_state_alike() {
    let mut a = tms_screen(Variant::TMS9918A, 6);
    let mut b = tms_screen(Variant::TMS9918A, 6);
    a.vram[0x3B00 + 4..0x3B00 + 8].copy_from_slice(&[49, 4, 1, 0x00]);
    b.vram[0x3B00 + 4..0x3B00 + 8].copy_from_slice(&[49, 4, 1, 0x00]);
    b.skip = true;
    for v in [&mut a, &mut b] {
        v.run(at(262 + 52, 30));
    }
    assert_eq!(state(&a), state(&b));
    assert!(a.collision && a.overflow, "a colour 0 sprite at X 4 collides with sprite 0");
    let saved = state(&a);
    let mut c = Vdp::new(Variant::TMS9918A, false);
    assert!(c.read_state(&mut StateReader::new(&saved)).unwrap());
    assert_eq!((state(&c), c.sprites.pixels), (saved, a.sprites.pixels), "the sprites' pixels are made again by the TMS9918's rules");
}
