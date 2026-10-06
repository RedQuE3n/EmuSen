//! The sound chips as the 68000's programs see them: the YM2612's busy flag and timers polled through `$A04000`,
//! the counts the loops leave in RAM checked against the rules of `ym2612.rs` (Nephrite_Native.md §18).

use crate::machine::Machine;
use crate::media::Media;
use crate::pictures::program;

/// RESET released and the Z80's bus requested and granted, then `lea $A04000,a2`.
const Z80_BUS: [u16; 16] = [0x33FC, 0x0100, 0x00A1, 0x1200, 0x33FC, 0x0100, 0x00A1, 0x1100, 0x0839, 0x0000, 0x00A1, 0x1100, 0x66F6, 0x45F9, 0x00A0, 0x4000];

/// `setup`, then a loop of `addq.w #1,d1; btst #bit,offset(a2); bne/beq` until the bit changes, the count stored
/// at `$FF0000`.
fn polls(setup: &[u16], offset: u16, bit: u16, until_set: bool) -> u16 {
    polls_placed(setup, offset, bit, until_set, false)
}

/// The same, placed as the board is against its 68000's start or not.
fn polls_placed(setup: &[u16], offset: u16, bit: u16, until_set: bool, board: bool) -> u16 {
    let mut code = Z80_BUS.to_vec();
    code.extend_from_slice(&[0x7200]);
    code.extend_from_slice(setup);
    // addq.w #1,d1 (4); btst #bit,d16(a2) (16); bne.s / beq.s back (10): 30 clocks a round.
    code.extend_from_slice(&[0x5241, 0x082A, bit, offset, if until_set { 0x67F6 } else { 0x66F6 }]);
    code.extend_from_slice(&[0x33C1, 0x00FF, 0x0000, 0x60FE]);
    let image = program(&[0x8004, 0x8104, 0x8F02], &[], &code);
    let mut m = Machine::new(&image, Media::read(&image));
    if board {
        crate::voices::place_as_the_board(&mut m);
    }
    while m.frames < 3 {
        m.advance();
    }
    u16::from_be_bytes([m.genesis.hw.wram[0], m.genesis.hw.wram[1]])
}

/// The busy flag reads set for 192 of the 68000's clocks after a data write, about six rounds of 30 clocks, and only
/// at port 0 of the discrete chip; an address write leaves it clear.
#[test]
fn a_program_sees_the_busy_flag_for_192_clocks_at_port_0_only() {
    let write = [0x14BC, 0x002A, 0x157C, 0x0080, 0x0001];
    let at_0 = polls(&write, 0, 7, false);
    assert!((6..=8).contains(&at_0), "{at_0} polls at port 0");
    assert_eq!(polls(&write, 2, 7, false), 1, "port 2 shows no busy flag on the discrete chip");
    assert_eq!(polls(&[0x14BC, 0x002A], 0, 7, false), 1, "an address write sets no busy flag");
}

/// Timer A at `$3F0` counts sixteen samples from the tick after its load. The board's program polls its flag 77 times,
/// and so does Nephrite, placed as the board is and at its own placement (Nephrite_Native.md §24, §25).
#[test]
fn a_program_sees_timer_a_overflow_after_its_period() {
    let start = [0x14BC, 0x0024, 0x157C, 0x00FC, 0x0001, 0x14BC, 0x0025, 0x157C, 0x0000, 0x0001, 0x14BC, 0x0027, 0x157C, 0x0005, 0x0001];
    assert_eq!(polls_placed(&start, 0, 0, true, true), 77, "polls until timer A's flag, as the board");
    assert_eq!(polls(&start, 0, 0, true), 77, "at Nephrite's placement");
}

/// A 68000 program writing `writes` (part, register, value) through `$A04000` and idling, run `frames` frames with the
/// YM2612's channel trace on; channel 1's samples from its first non-zero one.
fn voice(writes: &[(u8, u8, u8)], frames: i64) -> Vec<i32> {
    let mut code = Z80_BUS.to_vec();
    for &(part, r, v) in writes {
        code.extend_from_slice(&[0x157C, r as u16, 2 * part as u16, 0x157C, v as u16, 2 * part as u16 + 1]);
    }
    code.extend_from_slice(&[0x60FE]);
    let image = program(&[0x8004, 0x8104, 0x8F02], &[], &code);
    let mut m = Machine::new(&image, Media::read(&image));
    m.genesis.hw.sound.ym.trace = Some(Vec::new());
    while m.frames < frames {
        m.advance();
    }
    let t: Vec<i32> = m.genesis.hw.sound.ym.trace.take().unwrap().iter().map(|c| c[0]).collect();
    let first = t.iter().position(|&v| v != 0).unwrap_or(0);
    t[first..].to_vec()
}

/// Channel 1 with operators S1-S4 given as (detune and multiple, TL, KS and AR, AM and D1R, D2R, SL and RR), algorithm
/// and feedback, at frequency 1,081 in block 4 (439.3 Hz), keyed on.
fn channel_one(ops: [[u8; 6]; 4], alg: u8, fb: u8) -> Vec<(u8, u8, u8)> {
    let mut w = vec![(0, 0x22, 0), (0, 0x27, 0), (0, 0x2B, 0), (0, 0xB0, fb << 3 | alg), (0, 0xB4, 0xC0)];
    for (slot, op) in [0x00u8, 0x08, 0x04, 0x0C].iter().zip(ops) {
        for (base, v) in [0x30u8, 0x40, 0x50, 0x60, 0x70, 0x80].iter().zip(op) {
            w.push((0, base + slot, v));
        }
    }
    w.extend([(0, 0xA4, 4 << 3 | (1081 >> 8) as u8), (0, 0xA0, (1081 & 0xFF) as u8), (0, 0x28, 0xF0)]);
    w
}

const QUIET: [u8; 6] = [0x01, 0x7F, 0x1F, 0, 0, 0x0F];

/// The amplitude of `f` Hz in the YM2612's own samples (53,267 a second on NTSC).
fn amplitude(x: &[i32], f: f64) -> f64 {
    let rate = 4_725_000_000.0 / 88.0 / 1008.0;
    let (mut re, mut im) = (0.0, 0.0);
    for (n, &v) in x.iter().enumerate() {
        let a = 2.0 * std::f64::consts::PI * f * n as f64 / rate;
        re += v as f64 * a.cos();
        im += v as f64 * a.sin();
    }
    2.0 * (re * re + im * im).sqrt() / x.len() as f64
}

/// A sine at TL 0 is 439.3 Hz (frequency 1,081 in block 4), as all four references play it, with a peak of 255 of
/// the nine-bit output, half the DAC's swing as they all have it; TL 16, 12 dB, is a quarter of it (Nephrite_Native.md
/// §19.3).
#[test]
fn a_sine_plays_at_its_frequency_and_level() {
    let sine = [0x01, 0x00, 0x1F, 0, 0, 0x0F];
    let x = voice(&channel_one([QUIET, QUIET, QUIET, sine], 7, 0), 4);
    let ups: Vec<usize> = (1..x.len()).filter(|&n| x[n - 1] < 0 && x[n] >= 0).collect();
    let period = (ups[ups.len() - 1] - ups[0]) as f64 / (ups.len() - 1) as f64;
    let rate = 4_725_000_000.0 / 88.0 / 1008.0;
    assert!((rate / period - 439.3).abs() < 0.2, "{} Hz", rate / period);
    assert_eq!((*x.iter().max().unwrap(), *x.iter().min().unwrap()), (255, -256));
    let quarter = voice(&channel_one([QUIET, QUIET, QUIET, [0x01, 0x10, 0x1F, 0, 0, 0x0F]], 7, 0), 4);
    assert_eq!(*quarter.iter().max().unwrap(), 63);
}

/// Operator 1 alone with feedback 5: its harmonics 2-5 relative to the fundamental as Genesis Plus GX and PicoDrive
/// play them (0.597, 0.310, 0.211, 0.167 and 0.585, 0.310, 0.211, 0.171), within 3%.
#[test]
fn feedback_5_has_the_references_harmonics() {
    let s1 = [0x01, 0x00, 0x1F, 0, 0, 0x0F];
    let x = voice(&channel_one([s1, QUIET, QUIET, QUIET], 7, 5), 70);
    let x = &x[x.len() - 53_267..];
    let f0 = 4_725_000_000.0 / 88.0 / 1008.0 * 1081.0 * 8.0 / (1u64 << 20) as f64;
    let a1 = amplitude(x, f0);
    for (h, reference) in [(2.0, 0.59), (3.0, 0.310), (4.0, 0.211), (5.0, 0.169)] {
        let r = amplitude(x, f0 * h) / a1;
        assert!((r / reference - 1.0).abs() < 0.03, "harmonic {h}: {r} against {reference}");
    }
}
