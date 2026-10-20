//! The OPLL's rules, each held by a test (Endou_Opll.md §6).

use super::*;

/// The user instrument as a plain sine on the carrier: the modulator silent, the carrier at once to full level and held.
const SINE: [u8; 8] = [0x21, 0x21, 0x3F, 0x00, 0x00, 0xF0, 0x0F, 0x0F];

fn chip(user: &[u8; 8], fnum: u16, block: u8) -> Opll {
    let mut o = Opll::new();
    for (i, &v) in user.iter().enumerate() {
        o.write_address(i as u8);
        o.write_data(v);
    }
    for (a, v) in [(0x30, 0x00), (0x10, fnum as u8), (0x20, 0x10 | block << 1 | (fnum >> 8) as u8)] {
        o.write_address(a);
        o.write_data(v);
    }
    o
}

#[test]
fn the_tables_are_the_opl2_s_as_printed() {
    assert_eq!(&LOGSIN[..8], &[2137, 1731, 1543, 1419, 1326, 1252, 1190, 1137]);
    assert_eq!(&EXP[252..], &[1002, 1007, 1013, 1018]);
    for x in 0..256 {
        let p = (x as f64 + 0.5) * std::f64::consts::PI / 512.0;
        assert_eq!(LOGSIN[x], (-p.sin().log2() * 256.0).round() as u16);
        assert_eq!(EXP[x], (((x as f64 / 256.0).exp2() - 1.0) * 1024.0).round() as u16);
    }
}

#[test]
fn a_sine_at_full_level_peaks_at_the_operator_s_full_scale() {
    assert_eq!(operator(0x100, 0, false), 4084);
    assert_eq!(operator(0x300, 0, false), -4084);
    assert_eq!(operator(0x300, 0, true), 0);
    assert_eq!(operator(0x100, 16, false), 2042, "16 steps of 0.375 dB halve it");
}

#[test]
fn a_tone_comes_out_at_its_frequency() {
    let mut o = chip(&SINE, 0x120, 4);
    let mut crossings = Vec::new();
    let mut last = 0;
    for n in 0..49716 {
        let (m, _) = o.sample();
        if last < 0 && m >= 0 {
            crossings.push(n);
        }
        last = m;
    }
    let f = (crossings.len() - 1) as f64 * 49716.0 / (crossings[crossings.len() - 1] - crossings[0]) as f64;
    assert!((f - 288.0 * 49716.0 * 8.0 / 262144.0).abs() < 0.1, "{f}");
}

fn write(o: &mut Opll, a: u8, v: u8) {
    o.write_address(a);
    o.write_data(v);
}

// §3.2: the phase step keeps half a unit of its 19-bit cycle: F-number $101 in block 0 with MUL 0 steps 128, with MUL 1 257.
#[test]
fn the_phase_step_keeps_half_a_unit() {
    let mut o = Opll::new();
    write(&mut o, 0x10, 0x01);
    write(&mut o, 0x20, 0x01);
    let op = |mul: u8| Operator::of(&[0x20 | mul, 0x20 | mul, 0, 0, 0, 0, 0, 0], 1);
    assert_eq!((o.increment(0, &op(0)), o.increment(0, &op(1)), o.increment(0, &op(15))), (128, 257, 257 * 15));
    write(&mut o, 0x20, 0x03);
    assert_eq!(o.increment(0, &op(0)), 257, "block 1 doubles it");
}

// §3.4: KSL 1, 2 and 3 are 1.5, 3 and 6 dB an octave, measured; Table III-5's 21 dB at F-number $1A0 in block 7 is 3 dB an
// octave's 54 steps of 0.375 dB less 8 for each block below.
#[test]
fn the_level_key_scale_codes_are_1_5_3_and_6_db_an_octave() {
    let mut o = Opll::new();
    write(&mut o, 0x10, 0xA0);
    write(&mut o, 0x20, 0x0F);
    assert_eq!([0, 1, 2, 3].map(|k| o.ksl(0, k)), [0, 27, 54, 108]);
    write(&mut o, 0x20, 0x09);
    assert_eq!(o.ksl(0, 2), 54 - 3 * 8, "block 4");
    write(&mut o, 0x20, 0x01);
    assert_eq!(o.ksl(0, 3), 0, "never below 0");
}

fn run(o: &mut Opll, n: usize) -> Vec<i32> {
    (0..n).map(|_| o.sample().0).collect()
}

// §3.5: a decay whose sustain level is already reached does not step: SL 0 holds the attack's level 0 (a seeded fault
// stepped first and sounded 0.75 dB low).
#[test]
fn a_decay_at_its_sustain_level_does_not_step() {
    let mut o = chip(&[0x21, 0x21, 0x3F, 0x00, 0x00, 0xFF, 0x0F, 0x0F], 0x120, 4);
    run(&mut o, 2000);
    assert_eq!((o.slots[1].level, o.slots[1].stage), (0, Stage::Sustain));
    let mut o = chip(&[0x21, 0x21, 0x3F, 0x00, 0x00, 0xFF, 0x0F, 0x2F], 0x120, 4);
    run(&mut o, 2000);
    assert_eq!((o.slots[1].level, o.slots[1].stage), (16, Stage::Sustain), "SL 2 is 16 steps");
}

// §3.7: the carrier takes the modulator's output of the sample before.
#[test]
fn the_carrier_takes_the_modulator_s_output_of_the_sample_before() {
    let mut o = chip(&[0x21, 0x21, 0x00, 0x00, 0xF0, 0xF0, 0x0F, 0x0F], 0x120, 4);
    run(&mut o, 3000);
    for _ in 0..50 {
        let before = o.slots[0].out[0];
        let phase = o.slots[1].phase;
        let (m, _) = o.sample();
        assert_eq!(m, operator((phase >> 9) as i32 + before as i32, 0, false) as i32 >> 3);
    }
}

// §3.5: from envelope level 124 a slot sounds nothing, though its volume alone would let it sound.
#[test]
fn a_slot_past_level_124_is_silent() {
    let mut o = chip(&SINE, 0x120, 4);
    run(&mut o, 2000);
    o.slots[1].level = 123;
    assert!(run(&mut o, 200).iter().any(|&v| v != 0));
    o.slots[1].level = 124;
    o.slots[1].stage = Stage::Sustain;
    assert!(run(&mut o, 200).iter().all(|&v| v == 0));
}

// §3.6: the tremolo is a triangle of 0-13 steps of 0.375 dB over 210 steps of 64 samples, each end held twice; the
// vibrato steps every 1,024 samples through 0, half, full, half and back below.
#[test]
fn the_lfos_have_their_measured_shapes() {
    let mut o = Opll::new();
    let mut levels = Vec::new();
    for _ in 0..13440 {
        o.sample();
        levels.push(o.am >> 3);
    }
    assert_eq!((levels.iter().max(), levels.iter().filter(|&&l| l == 13).count(), levels.iter().filter(|&&l| l == 0).count()), (Some(&13), 128, 1024));
    assert_eq!((o.am, o.am_down), (0, false), "a whole cycle in 13,440 samples");
    let mut o = Opll::new();
    let steps: Vec<i32> = (0..8).map(|k| {
        o.counter = k << 10;
        o.vibrato(0x1FF)
    }).collect();
    assert_eq!(steps, [0, 3, 7, 3, 0, -3, -7, -3]);
}

// §3.5: key-off releases a sustained tone at its RR, a percussive one at 7, and either at 5 with SUS.
#[test]
fn the_release_rate_follows_the_tone_and_sus() {
    for (eg, sus, rate) in [(0x20u8, 0u8, 4 * 3), (0x00, 0, 4 * 7), (0x20, 1, 4 * 5), (0x00, 1, 4 * 5)] {
        let mut o = chip(&[0x21, 0x01 | eg, 0x3F, 0, 0, 0xF0, 0x0F, 0x03], 0x0C0, 0);
        run(&mut o, 500);
        write(&mut o, 0x20, sus << 5);
        let start = o.slots[1].level;
        let n = 1 << (13 - rate / 4 + 3);
        run(&mut o, n);
        assert_eq!(o.slots[1].level - start, 4, "eg {eg:02x} sus {sus}: four steps in {n} samples at rate {rate}");
    }
}

// §3.5: a key-on over a sounding slot takes it to silence at rate 48 first, then the attack starts with the phase at 0.
#[test]
fn a_key_on_damps_and_restarts_the_phase() {
    let mut o = chip(&[0x21, 0x21, 0x3F, 0, 0, 0xF0, 0x0F, 0x00], 0x120, 4);
    run(&mut o, 2000);
    write(&mut o, 0x20, 0x09);
    run(&mut o, 10);
    write(&mut o, 0x20, 0x19);
    run(&mut o, 2);
    assert_eq!(o.slots[1].stage, Stage::Damp);
    let mut n = 0;
    while o.slots[1].stage == Stage::Damp {
        run(&mut o, 1);
        n += 1;
    }
    assert!((480..540).contains(&n), "127 steps at a quarter a sample: {n}");
    assert_eq!(o.slots[1].phase, 4608, "restarted at 0 and stepped once in the sample the attack began");
}

// §3.8: rhythm mode keys its slots from $0E: the bass drum both of channel 7's, the others one each.
#[test]
fn the_rhythm_bits_key_their_slots() {
    let mut o = Opll::new();
    write(&mut o, 0x0E, 0x20 | 0x10 | 0x01);
    assert_eq!((12..18).map(|s| o.keyed(s)).collect::<Vec<_>>(), [true, true, true, false, false, false]);
    write(&mut o, 0x0E, 0x20 | 0x08 | 0x04 | 0x02);
    assert_eq!((12..18).map(|s| o.keyed(s)).collect::<Vec<_>>(), [false, false, false, true, true, true]);
    write(&mut o, 0x0E, 0x1F);
    assert!((12..18).all(|s| !o.keyed(s)), "without bit 5 the bits key nothing");
}

#[test]
fn an_address_the_map_does_not_name_takes_nothing() {
    let mut o = Opll::new();
    write(&mut o, 0x40, 0x55);
    write(&mut o, 0xFF, 0x55);
    assert_eq!(o, Opll::new().tap(|n| n.address = 0xFF));
}

trait Tap: Sized {
    fn tap(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}
impl Tap for Opll {}

#[test]
fn a_state_continues_the_chip_exactly() {
    let mut a = chip(&[0x61, 0xA1, 0x10, 0x05, 0xC4, 0xD3, 0x25, 0x36], 0x155, 3);
    write(&mut a, 0x0E, 0x3F);
    run(&mut a, 7777);
    let mut bytes = vec![0u8; 512];
    let mut w = StateWriter::new(&mut bytes);
    a.write_state(&mut w);
    let n = w.len();
    let mut b = Opll::new();
    assert!(b.read_state(&mut StateReader::new(&bytes[..n])).unwrap());
    assert_eq!(a, b);
    assert_eq!(run(&mut a, 5000), run(&mut b, 5000));
    bytes[1 + REGISTERS + 72] = 200;
    assert!(!Opll::new().read_state(&mut StateReader::new(&bytes[..n])).unwrap(), "a level past 127 is no chip's");
}
