//! The OPLL's rules, each held by a test (Endou_Opll.md §6).

use super::*;

/// The original instrument as a plain sine on the carrier: the modulator silent, the carrier at once to full level and held.
const SINE: [u8; 8] = [0x21, 0x21, 0x3F, 0x00, 0x00, 0xF0, 0x0F, 0x0F];

fn chip(tone: &[u8; 8], fnum: u16, block: u8) -> Opll {
    let mut o = Opll::new();
    for (i, &v) in tone.iter().enumerate() {
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
    assert_eq!(operator(0x300, 0, false), -4085, "the negative half is the ones' complement");
    assert_eq!(operator(0x300, 0, true), -1, "a half wave's negative half is a negative zero");
    assert_eq!(operator(0x100, 16, false), 2042, "16 steps of 0.375 dB halve it");
    assert_eq!(operator(0x200, 127, false), -1, "past the exponent's range the sign stays");
    assert_eq!(operator(0x300, MUTE, false), 0, "a silent slot gives a positive zero");
}

// §3.3: the DAC takes a twelve-bit output as a sign and its top eight bits, offset by one, and a silent channel sums to 0.
#[test]
fn the_dac_is_sign_and_magnitude_offset_by_one() {
    assert_eq!([0, 15, 16, 4084].map(dac), [0, 0, 1, 255]);
    assert_eq!([-1, -16, -17, -4085].map(dac), [-4, -4, -5, -259]);
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
    assert_eq!((o.increment(1, &op(0)), o.increment(1, &op(1)), o.increment(1, &op(15))), (128, 257, 257 * 15));
    write(&mut o, 0x20, 0x03);
    assert_eq!(o.increment(1, &op(0)), 257, "block 1 doubles it");
    write(&mut o, 0x10, 230);
    write(&mut o, 0x20, 0x00);
    o.counter = 1024;
    let vib = Operator::of(&[0x67, 0x67, 0, 0, 0, 0, 0, 0], 1);
    assert_eq!(o.increment(1, &vib), (461 / 2) * 14 / 2, "an odd 2F + vibrato in block 0 loses its half before the multiple");
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

// §3.7: the carrier takes the modulator's output of the sample before, its lowest bit dropped.
#[test]
fn the_carrier_takes_the_modulator_s_output_of_the_sample_before() {
    let mut o = chip(&[0x21, 0x21, 0x00, 0x00, 0xF0, 0xF0, 0x0F, 0x0F], 0x120, 4);
    run(&mut o, 3000);
    for _ in 0..50 {
        let before = o.slots[0].out[0];
        let phase = o.slots[1].phase;
        let (m, _) = o.sample();
        assert_eq!(m, dac(operator((phase >> 9) as i32 + (before as i32 >> 1 << 1), 0, false)));
    }
}

// §3.3: feedback is the modulator's last two outputs, each with its lowest bit dropped, summed and shifted by 8 − FB.
#[test]
fn feedback_halves_each_output_before_the_sum() {
    let mut o = chip(&[0x21, 0x21, 0x00, 0x07, 0xF0, 0xF0, 0x0F, 0x0F], 0x120, 4);
    run(&mut o, 1000);
    for _ in 0..50 {
        let (out, phase) = (o.slots[0].out, o.slots[0].phase);
        o.sample();
        let feedback = ((out[0] as i32 >> 1) + (out[1] as i32 >> 1)) >> 1;
        assert_eq!(o.slots[0].out[0], operator((phase >> 9) as i32 + feedback, 0, false));
    }
}

// §3.5: a falling envelope at level 124 is cut to 127 on its next update, and a slot at 127 sounds nothing.
#[test]
fn a_slot_reaching_level_124_is_cut_to_silence() {
    let mut o = chip(&SINE, 0x120, 4);
    run(&mut o, 2000);
    o.slots[1].level = 123;
    assert!(run(&mut o, 200).iter().any(|&v| v != 0));
    o.slots[1].level = 124;
    o.slots[1].stage = Stage::Sustain;
    run(&mut o, 1);
    assert_eq!(o.slots[1].level, SILENT);
    assert!(run(&mut o, 200).iter().all(|&v| v == 0));
}

// §3.6: the tremolo is a triangle of 0-13 steps of 0.375 dB over 210 steps of 64 samples, 0 to 105 and back with no
// hold; the vibrato steps every 1,024 samples through 0, half, full, half and back below.
#[test]
fn the_lfos_have_their_measured_shapes() {
    let mut o = Opll::new();
    let mut levels = Vec::new();
    for _ in 0..13440 {
        o.sample();
        levels.push(o.am >> 3);
    }
    assert_eq!((levels.iter().max(), levels.iter().filter(|&&l| l == 13).count(), levels.iter().filter(|&&l| l == 0).count()), (Some(&13), 192, 960));
    assert_eq!((o.am, o.am_down), (0, false), "a whole cycle in 13,440 samples");
    let mut o = Opll::new();
    let steps: Vec<i32> = (0..8).map(|k| {
        o.counter = k << 10;
        o.vibrato(0x1FF, 1)
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

// §3.5: a key-on over a sounding slot takes it towards silence at rate 12 with the key scale, until level 124, then the
// attack starts with the phase at 0.
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
    assert!((320..340).contains(&n), "124 steps at rate 50, three in eight samples: {n}");
    assert_eq!(o.slots[1].phase, 4608, "restarted at 0 and stepped once in the sample the attack began");
}

/// A carrier's levels, one a sample, from a key-on on channel 0 with the given tone.
fn carrier_levels(tone: &[u8; 8], fnum: u16, block: u8, n: usize) -> Vec<u8> {
    let mut o = chip(tone, fnum, block);
    (0..n).map(|_| { o.sample(); o.slots[1].level }).collect()
}

// §3.5: an attack below rate 48 takes four steps of level / 16 + 1, one a sample, at each of its ticks.
#[test]
fn a_slow_attack_steps_four_samples_running() {
    let levels = carrier_levels(&[0x21, 0x21, 0x3F, 0, 0, 0x40, 0x0F, 0x0F], 0x1FF, 0, 1700);
    let changes: Vec<(usize, u8)> = levels.windows(2).enumerate().filter(|(_, w)| w[0] != w[1]).map(|(i, w)| (i + 1, w[1])).collect();
    assert_eq!(&changes[..4], &[(511, 119), (512, 111), (513, 104), (514, 97)], "127 less 8, 8, 7 and 7");
    assert_eq!(changes[4].0, 1535, "the next tick that steps, rate 16 stepping at every other");
}

// §3.5: from rate 48 an attack steps every sample, 1, 2 or 4 sixteenths, doubled in some groups of four samples.
#[test]
fn a_fast_attack_weighs_each_sample() {
    let mut o = Opll::new();
    o.counter = 0;
    assert_eq!((0..16).map(|_| { let w = o.attack_weight(48, 1); o.counter += 1; w }).collect::<Vec<_>>(), [1; 16]);
    o.counter = 0;
    let rate51: Vec<u32> = (0..16).map(|_| { let w = o.attack_weight(51, 1); o.counter += 1; w }).collect();
    assert_eq!(rate51, [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 2], "three groups of four doubled at rate 51");
    o.counter = 0;
    assert_eq!(o.attack_weight(56, 1), 4);
}

// §3.5: from rate 48 a decay steps on some samples of each four, by the rate's low bits.
#[test]
fn a_fast_decay_steps_by_its_groups() {
    let mut o = Opll::new();
    let steps = |o: &mut Opll, rate: u32| -> Vec<u8> {
        o.counter = 0;
        (0..16).map(|_| { let s = o.step(rate, 1); o.counter += 1; s }).collect()
    };
    assert_eq!(steps(&mut o, 48), [0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0]);
    assert_eq!(steps(&mut o, 51), [1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0]);
    assert_eq!(steps(&mut o, 53), [1, 1, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 1]);
    assert_eq!(steps(&mut o, 57), [2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2]);
}

// §3.5: a modulator reads the envelope's ticks and the LFOs a sample ahead of its carrier.
#[test]
fn a_modulator_reads_the_count_a_sample_ahead() {
    let mut o = Opll::new();
    o.counter = 100;
    assert_eq!((o.tick(0), o.tick(1)), (102, 101));
    o.counter = 1023;
    assert_eq!((o.vibrato(0x1FF, 0), o.vibrato(0x1FF, 1)), (3, 0));
    o.counter = 63;
    o.am = 8;
    assert_eq!((o.tremolo(0), o.tremolo(1)), (1, 1));
    o.am = 7;
    assert_eq!((o.tremolo(0), o.tremolo(1)), (1, 0));
}

// §3.5: at key-off a modulator holds its level, and only a percussive tone's rate-7 release steps on that update.
#[test]
fn a_key_off_releases_the_carrier_alone() {
    let mut o = chip(&[0x21, 0x21, 0x10, 0, 0xF4, 0xF0, 0xFF, 0x0F], 0x120, 4);
    run(&mut o, 3000);
    write(&mut o, 0x20, 0x09);
    let held = o.slots[0].level;
    run(&mut o, 3000);
    assert_eq!((o.slots[0].level, o.slots[0].stage), (held, Stage::Release));
    assert_eq!(o.slots[1].level, SILENT);
    for (eg, sus, rate, steps) in [(0x00u8, 0u8, 30, true), (0x00, 1, 22, false), (0x20, 0, 30, false)] {
        let mut o = chip(&[0x21, 0x01 | eg, 0x3F, 0, 0xF0, 0xF0, 0x0F, 0x07], 0x120, 4);
        run(&mut o, 100);
        while o.step(rate, 1) == 0 {
            o.sample();
        }
        write(&mut o, 0x20, sus << 5 | 0x09);
        let before = o.slots[1].level;
        o.sample();
        assert_eq!(o.slots[1].level != before, steps, "eg {eg:02x} sus {sus}");
    }
}

// §3.5: a modulator's phase restarts when its carrier's damp ends, whenever its own damp ended.
#[test]
fn a_modulator_restarts_its_phase_with_its_carrier() {
    let mut o = chip(&[0x21, 0x21, 0x10, 0, 0xF0, 0xF0, 0x00, 0x0F], 0x120, 4);
    run(&mut o, 1000);
    write(&mut o, 0x20, 0x09);
    run(&mut o, 100);
    assert_eq!((o.slots[0].level, o.slots[1].level), (0, SILENT), "the modulator held, the carrier released");
    write(&mut o, 0x20, 0x19);
    o.sample();
    assert_eq!((o.slots[0].stage, o.slots[1].stage), (Stage::Damp, Stage::Attack));
    assert_eq!((o.slots[0].phase, o.slots[1].phase), (4608, 4608), "both restarted with the carrier's attack");
}

// §3.5: a damp ends at level 124 and the attack starts from the level it reached; at rate 60 or more the attack's
// update only sets level 0, the decay starting on the next.
#[test]
fn a_damp_ends_at_124_and_the_attack_starts_there() {
    let mut o = chip(&[0x21, 0x21, 0x3F, 0, 0, 0x50, 0x0F, 0x00], 0x120, 4);
    run(&mut o, 4);
    o.slots[1].stage = Stage::Damp;
    o.slots[1].level = 124;
    o.sample();
    assert_eq!((o.slots[1].stage, o.slots[1].level), (Stage::Attack, 124));
    let mut o = chip(&SINE, 0x120, 4);
    o.sample();
    assert_eq!((o.slots[1].stage, o.slots[1].level), (Stage::Attack, 0));
    o.sample();
    assert_eq!((o.slots[1].stage, o.slots[1].level), (Stage::Decay, 0));
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

// §4: each of the fifteen instruments and the five drums sounds; the five measured exactly are the tone registers that
// sound the same, so the original instrument loaded with one sounds as the instrument does, sample for sample.
#[test]
fn every_instrument_and_drum_sounds_and_the_exact_ones_are_their_registers() {
    for inst in 1..16u8 {
        let mut o = Opll::new();
        for (a, v) in [(0x30, inst << 4), (0x10, 0x20), (0x20, 0x19)] {
            write(&mut o, a, v);
        }
        assert!(run(&mut o, 4000).iter().any(|&v| v.abs() > 20), "instrument {inst}");
    }
    for bit in [0x10u8, 0x08, 0x04, 0x02, 0x01] {
        let mut o = Opll::new();
        for (a, v) in [(0x16, 0x20), (0x17, 0x50), (0x18, 0xC0), (0x26, 0x05), (0x27, 0x05), (0x28, 0x01), (0x0E, 0x20 | bit)] {
            write(&mut o, a, v);
        }
        assert!((0..4000).map(|_| o.sample().1).any(|v| v.abs() > 20), "drum {bit:02x}");
    }
    for inst in [1u8, 2, 5, 6, 7, 8, 9, 10, 12, 13, 14] {
        let play = |original: bool| {
            let mut o = Opll::new();
            for (i, &v) in INSTRUMENTS[inst as usize - 1].iter().enumerate() {
                write(&mut o, i as u8, v);
            }
            for (a, v) in [(0x30, if original { 0 } else { inst << 4 }), (0x10, 0x20), (0x20, 0x19)] {
                write(&mut o, a, v);
            }
            run(&mut o, 6000)
        };
        assert_eq!(play(true), play(false), "instrument {inst}");
    }
}
