//! `mdboard.py`'s FM voices, built here as it builds them, and channel 1 of the YM2612 held to the board's pins
//! sample for sample: the chip is given each write at the cycle the board's bus carried it, so that nothing of the
//! 68000's or the Z80's timing is in the comparison (`board_fm.rs`; Nephrite_Disputes.md D-16 to D-19,
//! Nephrite_Native.md §21).

use crate::board_fm::{BLOCK, BoardVoice, REPLAY_WRITES, REPLAYS, VOICE_RELEASE, VOICE_WRITES, VOICES};
use crate::machine::Machine;
use crate::media::Media;
use crate::pictures::program;
use crate::ym2612::{SAMPLE, Ym2612};

/// An operator's registers `$30`-`$80`: detune and multiple, TL, key scaling and AR, DR, SR, SL and RR.
type Op = [u8; 6];
const SINE: Op = [0x01, 0x00, 0x1F, 0x00, 0x00, 0x0F];
const QUIET: Op = [0x01, 0x7F, 0x1F, 0x00, 0x00, 0x0F];
/// Full level with the other registers of `SINE`, for a given multiple and TL.
const fn level(mul: u8, tl: u8) -> Op {
    [mul, tl, 0x1F, 0, 0, 0x0F]
}

/// RESET released and the Z80's bus requested and granted, then `lea $A04000,a2`.
pub(crate) const Z80_BUS: [u16; 16] = [0x33FC, 0x0100, 0x00A1, 0x1200, 0x33FC, 0x0100, 0x00A1, 0x1100, 0x0839, 0x0000, 0x00A1, 0x1100, 0x66F6, 0x45F9, 0x00A0, 0x4000];

/// `mdboard.py`'s `ym_writes`: for each register, the busy flag waited out, the address and the data, four NOPs.
pub(crate) fn ym_writes(code: &mut Vec<u16>, pairs: &[(u8, u8)], part: u8) {
    for &(r, v) in pairs {
        code.extend_from_slice(&[0x4A12, 0x6BFC]);
        if part == 0 {
            code.extend_from_slice(&[0x14BC, r as u16, 0x157C, v as u16, 0x0001]);
        } else {
            code.extend_from_slice(&[0x157C, r as u16, 0x0002, 0x157C, v as u16, 0x0003]);
        }
        code.extend_from_slice(&[0x4E71; 4]);
    }
}

/// A voice on channel 1 as `mdboard.py`'s `fm_voice` writes it: its 37 register writes up to the key-on, and what
/// follows the key-on (a register, its value, thousands of `dbra` rounds to wait first).
struct Voice {
    writes: Vec<(u8, u8)>,
    extra: Vec<(u8, u8, u16)>,
}

/// S1-S4, the algorithm and feedback, the frequency, then every key on.
fn fm_voice(ops: [Op; 4], alg: u8, fb: u8, fnum: u16, block: u8, extra: &[(u8, u8, u16)]) -> Voice {
    let mut writes = vec![(0x22, 0), (0x27, 0), (0x28, 0), (0x2B, 0), (0xB0, fb << 3 | alg), (0xB4, 0xC0)];
    for (slot, op) in [0x00u8, 0x08, 0x04, 0x0C].into_iter().zip(ops) {
        for (k, v) in op.into_iter().enumerate() {
            writes.push((0x30 + 0x10 * k as u8 + slot, v));
        }
        writes.push((0x90 + slot, 0));
    }
    writes.extend_from_slice(&[(0xA4, block << 3 | (fnum >> 8) as u8), (0xA0, fnum as u8), (0x28, 0xF0)]);
    Voice { writes, extra: extra.to_vec() }
}

impl Voice {
    /// The 68000's program for it, as `mdboard.py` builds it.
    fn image(&self) -> Vec<u8> {
        self.image_then(&[])
    }

    /// The same with `tail` run after the voice's writes, before the idle loop.
    fn image_then(&self, tail: &[u16]) -> Vec<u8> {
        let mut code = Z80_BUS.to_vec();
        ym_writes(&mut code, &self.writes, 0);
        for &(r, v, wait) in &self.extra {
            code.extend_from_slice(&[0x303C, wait * 1000, 0x51C8, 0xFFFE]);
            ym_writes(&mut code, &[(r, v)], 0);
        }
        code.extend_from_slice(tail);
        code.push(0x60FE);
        program(&[0x8004, 0x8104, 0x8F02], &[], &code)
    }
}

/// `mdboard.py`'s `ROM_VOICES`: S3's multiple, TL and decay rate, S4's multiple, the frequency and the block.
const ROM_VOICES: [(u8, u8, u8, u8, u16, u8); 10] = [
    (1, 0, 16, 1, 1081, 4), (1, 0, 16, 2, 1081, 4), (2, 0, 17, 1, 1081, 4), (1, 1, 16, 3, 1081, 4), (3, 0, 15, 0, 1081, 4),
    (1, 0, 16, 1, 1151, 3), (1, 0, 17, 2, 777, 4), (2, 2, 16, 1, 1999, 2), (1, 3, 15, 5, 613, 5), (1, 0, 18, 7, 1333, 3),
];

/// One of `mdboard.py`'s `SOUNDS` by its name there.
fn voice(name: &str) -> Voice {
    let s4 = |op: Op| fm_voice([QUIET, QUIET, QUIET, op], 7, 0, 1081, 4, &[]);
    let number = |prefix: &str| name.strip_prefix(prefix).and_then(|n| n.parse::<u8>().ok());
    // `rate_voice`: key scaling 3 at block 4, where the rate is twice the register plus a key code of 18 or 19.
    let by_rate = |rate: u8, attack: bool| {
        let (r, fnum) = ((rate - 18) / 2, if rate & 1 != 0 { 1152 } else { 1081 });
        let op = if attack { [0x01, 0x00, 0xC0 | r, 0x00, 0x00, 0x0F] } else { [0x01, 0x00, 0xDF, r, 0x00, 0xFF] };
        fm_voice([QUIET, QUIET, QUIET, op], 7, 0, fnum, 4, &[])
    };
    match name {
        "fm-sine" => s4(SINE),
        "fm-tl16" => s4(level(0x01, 0x10)),
        "fm-mul3" => s4(level(0x03, 0x00)),
        "fm-dt7" => s4(level(0x71, 0x00)),
        "fm-feedback5" => fm_voice([SINE, QUIET, QUIET, QUIET], 7, 5, 1081, 4, &[]),
        "fm-chain" => fm_voice([level(0x01, 0x30), level(0x02, 0x28), level(0x01, 0x20), SINE], 0, 0, 1081, 4, &[]),
        "fm-alg7" => fm_voice([level(0x01, 0x10), level(0x02, 0x14), level(0x03, 0x18), level(0x05, 0x1C)], 7, 4, 1081, 4, &[]),
        "fm-attack" => s4([0x01, 0x00, 0x0C, 0, 0, 0x0F]),
        "fm-envelope" => fm_voice([QUIET, QUIET, QUIET, [0x01, 0x00, 0x14, 0x14, 0x0E, 0x48]], 7, 0, 1081, 4, &[(0x28, 0x00, 43)]),
        "fm-envelope-ks" => fm_voice([QUIET, QUIET, QUIET, [0x01, 0x00, 0xD2, 0x10, 0x0C, 0x46]], 7, 0, 1081, 4, &[(0x28, 0x00, 43)]),
        _ => {
            if let Some(alg) = number("fm-alg") {
                fm_voice([level(0x01, 0x28), level(0x02, 0x24), level(0x03, 0x20), level(0x01, 0x08)], alg, 3, 1081, 4, &[])
            } else if let Some(rate) = number("fm-dr") {
                by_rate(rate, false)
            } else if let Some(rate) = number("fm-ar") {
                by_rate(rate, true)
            } else if let Some(i) = number("fm-rom") {
                let (mm, tl, dr, cm, fnum, block) = ROM_VOICES[i as usize];
                fm_voice([QUIET, QUIET, [mm, tl, 0x1F, dr, 0x00, 0xFF], level(cm, 0x00)], 4, 0, fnum, block, &[])
            } else {
                panic!("no voice named {name}")
            }
        }
    }
}

fn fnv(bytes: impl IntoIterator<Item = u8>) -> u32 {
    bytes.into_iter().fold(0x811C_9DC5u32, |h, b| (h ^ b as u32).wrapping_mul(0x0100_0193))
}

/// The machine after `frames` frames of `image`, and channel 1's output for every sample of them.
fn machine(image: &[u8], frames: i64) -> (Machine, Vec<i32>) {
    let mut m = Machine::new(image, Media::read(image));
    m.genesis.hw.sound.ym.trace = Some(Vec::new());
    while m.frames < frames {
        m.advance();
    }
    let trace = m.genesis.hw.sound.ym.trace.take().unwrap().iter().map(|c| c[0]).collect();
    (m, trace)
}

/// A chip whose reset line is asserted at master clock `asserted` and released at `released`, then given each of
/// `writes` (a master clock, the port, the value): channel 1's samples from its first that is neither 0 nor -1.
fn chip(asserted: u64, released: u64, writes: impl Iterator<Item = (u64, u16, u8)>, samples: usize) -> Vec<i32> {
    let mut y = Ym2612::new(true);
    y.run(asserted, |_, _| {});
    y.reset_line(true, asserted);
    y.run(released, |_, _| {});
    y.reset_line(false, released);
    y.trace = Some(Vec::new());
    let mut last = released;
    for (t, port, v) in writes {
        y.run(t, |_, _| {});
        y.write(port, v, t);
        last = t;
    }
    // A slow attack is some 1,300 samples leaving rest.
    y.run(last + (samples as u64 + 2048) * SAMPLE, |_, _| {});
    let all: Vec<i32> = y.trace.take().unwrap().iter().map(|c| c[0]).collect();
    let first = all.iter().position(|&x| x != 0 && x != -1).expect("the voice leaves rest");
    all[first..].to_vec()
}

/// How many of `samples` of ours are the board's, by its blocks of 64.
fn equal(ours: &[i32], samples: usize, blocks: &[u32]) -> usize {
    assert!(ours.len() >= samples, "{} samples made of {samples}", ours.len());
    let block = |k: usize| &ours[k * BLOCK..samples.min((k + 1) * BLOCK)];
    blocks.iter().enumerate().filter(|&(k, &hash)| fnv(block(k).iter().flat_map(|&x| (x as i16).to_le_bytes())) == hash).map(|(k, _)| block(k).len()).sum()
}

/// A voice on the chip, its writes at the cycles the board's 68000 made them: how many samples are the board's. The
/// program is checked to be the one the board ran.
fn equal_samples(v: &BoardVoice) -> usize {
    let voice = voice(v.name);
    assert_eq!(fnv(voice.image()), v.image, "{}: the program is not the one the board ran", v.name);
    let ports = voice.writes.iter().flat_map(|&(r, value)| [(0u16, r), (1, value)]);
    let writes = VOICE_WRITES.iter().zip(ports).map(|(&t, (port, value))| (t / 2, port, value));
    equal(&chip(0, VOICE_RELEASE / 2, writes, v.samples), v.samples, v.blocks)
}

/// The voices whose names begin with one of `prefixes`: (samples equal to the board's, samples held).
fn held(prefixes: &[&str]) -> (usize, usize) {
    let mut totals = (0, 0);
    for v in VOICES.iter().filter(|v| prefixes.iter().any(|p| v.name.starts_with(p))) {
        let equal = equal_samples(v);
        assert_eq!(equal, v.samples, "{}: {equal} of {} samples are the board's", v.name, v.samples);
        totals = (totals.0 + equal, totals.1 + v.samples);
    }
    totals
}

#[test]
fn the_board_holds_62_voices_and_200_220_samples() {
    assert_eq!((VOICES.len(), VOICES.iter().map(|v| v.samples).sum::<usize>()), (62, 200_220));
    assert!(VOICES.iter().all(|v| v.blocks.len() == v.samples.div_ceil(BLOCK)));
}

/// Every algorithm with feedback, the levels, a multiple and a detune: operator 1's output reaches operators 2 and 4
/// and the sum a sample late and operator 3 a sample later (D-17), without which 43,200 of these are the board's.
#[test]
fn the_algorithms_and_levels_are_the_boards_sample_for_sample() {
    assert_eq!(held(&["fm-sine", "fm-tl16", "fm-mul3", "fm-dt7", "fm-feedback5", "fm-chain", "fm-alg"]), (67_482, 67_482));
}

/// An attack, a whole envelope to its key-off and one with key scaling: the envelope's step shows from the sample
/// after its cycle (D-17), and the last attacks at rate 54 (D-18).
#[test]
fn the_envelopes_are_the_boards_sample_for_sample() {
    assert_eq!(held(&["fm-attack", "fm-envelope"]), (11_700, 11_700));
}

/// Decays at rates 38 to 61 from an instant attack: rates 48 to 59 double their step on the board's cycles (D-18).
#[test]
fn the_decays_by_rate_are_the_boards_sample_for_sample() {
    assert_eq!(held(&["fm-dr"]), (31_038, 31_038));
}

/// Attacks at rates 44 to 59, which a step placed a sample out leaves almost wholly unequal.
#[test]
fn the_attacks_by_rate_are_the_boards_sample_for_sample() {
    assert_eq!(held(&["fm-ar"]), (48_000, 48_000));
}

/// A decaying S3 into S4, where every bit of S3's output moves S4's phase: these are what hold the two ROMs' every
/// entry to the board's (D-16).
#[test]
fn a_modulator_into_a_carrier_is_the_boards_sample_for_sample() {
    assert_eq!(held(&["fm-rom"]), (42_000, 42_000));
}

/// The Z80's reset line held from 0.28 to 2.82 samples, forty times, and the same decaying voice written after each
/// release: where the release and the key-on fall in the cycle the assertion started decides which sample the
/// envelope's counter starts with and which takes the key (D-19).
#[test]
fn a_voice_replayed_after_a_reset_is_the_boards_wherever_the_release_falls() {
    let mut wrong = Vec::new();
    for r in REPLAYS {
        let (asserted, released) = (50_000, 50_000 + r.held / 2);
        let writes = r.times.iter().zip(REPLAY_WRITES).map(|(&t, (port, value))| (released + t / 2, port as u16, value));
        let ours = chip(asserted, released, writes, 512);
        if equal(&ours, 512, r.blocks) != 512 {
            wrong.push((r.held, equal(&ours, 512, r.blocks)));
        }
    }
    assert!(wrong.is_empty(), "replays unlike the board's, by the cycles the line was held and the samples equal: {wrong:?}");
    assert_eq!(REPLAYS.len(), 40);
}

/// The whole machine running the 68000's programs, on its own timing: the algorithm and level voices, in which no
/// envelope moves, are the board's all the same. The others are not yet, their key-on coming three and a half
/// samples sooner than the board's 68000 makes it (Nephrite_Native.md §21.4).
#[test]
fn the_whole_machine_plays_the_algorithms_as_the_board_does() {
    let (mut equal_samples, mut samples) = (0, 0);
    for v in VOICES.iter().filter(|v| !["fm-attack", "fm-envelope", "fm-dr", "fm-ar", "fm-rom"].iter().any(|p| v.name.starts_with(p))) {
        let (_, ours) = machine(&voice(v.name).image(), (v.samples as i64 + 400) / 880 + 2);
        let first = ours.iter().position(|&x| x != 0 && x != -1).expect("the voice leaves rest");
        equal_samples += equal(&ours[first..], v.samples, v.blocks);
        samples += v.samples;
    }
    assert_eq!((equal_samples, samples), (67_482, 67_482));
}

/// The 68000 asserts the Z80's reset under a sounding voice and releases it: the voice stops, the registers are as
/// at power-on, and the chip takes writes again (D-19).
#[test]
fn the_z80s_reset_line_resets_the_ym2612() {
    // dbra 3,000 rounds; move.w #0,$A11200; dbra 300 rounds; move.w #$100,$A11200; $2A = $55 once the busy flag clears.
    let mut tail = vec![0x303C, 3000, 0x51C8, 0xFFFE, 0x33FC, 0x0000, 0x00A1, 0x1200, 0x303C, 300, 0x51C8, 0xFFFE, 0x33FC, 0x0100, 0x00A1, 0x1200];
    ym_writes(&mut tail, &[(0x2A, 0x55)], 0);
    let (m, ours) = machine(&voice("fm-sine").image_then(&tail), 3);
    let last_sound = ours.iter().rposition(|&x| x != 0 && x != -1).unwrap();
    assert!(ours[..last_sound].iter().filter(|&&x| x.abs() > 200).count() > 50, "the voice sounded");
    assert!((250..400).contains(&last_sound), "it stopped at the pulse, about 62 + 208 samples in: {last_sound}");
    let ym = &m.genesis.hw.sound.ym;
    assert!(!ym.held);
    assert_eq!((ym.regs[0][0x3C], ym.regs[0][0x4C], ym.regs[0][0xA4], ym.regs[0][0xB4]), (0, 0, 0, 0xC0), "the voice's registers are gone");
    assert_eq!(ym.regs[0][0x2A], 0x55, "a write after the release lands");
}
