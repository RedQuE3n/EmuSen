//! `mdboard.py`'s FM voices, built here as it builds them, and channel 1 of the YM2612 held to the board's pins
//! sample for sample: the chip is given each write at the cycle the board's bus carried it, so that nothing of the
//! 68000's or the Z80's timing is in the comparison (`board_fm.rs`; Nephrite_Disputes.md D-16 to D-19,
//! Nephrite_Native.md §21).

use crate::board_chip::CHIP_VOICES;
use crate::board_fm::{BLOCK, BoardVoice, LFO_VOICES, REPLAY_WRITES, REPLAYS, VOICE_RELEASE, VOICE_WRITES, VOICES};
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
        ym_write_late(code, r, v, part, 0);
    }
}

/// One of them with `nops` NOPs between the wait for the busy flag and the address, which move it across the sample.
fn ym_write_late(code: &mut Vec<u16>, r: u8, v: u8, part: u8, nops: u16) {
    code.extend_from_slice(&[0x4A12, 0x6BFC]);
    code.extend(std::iter::repeat_n(0x4E71, nops as usize));
    if part == 0 {
        code.extend_from_slice(&[0x14BC, r as u16, 0x157C, v as u16, 0x0001]);
    } else {
        code.extend_from_slice(&[0x157C, r as u16, 0x0002, 0x157C, v as u16, 0x0003]);
    }
    code.extend_from_slice(&[0x4E71; 4]);
}

/// A voice as `mdboard.py`'s `fm_voice` writes it: its register writes up to the key-on, each with the part it goes
/// to and the NOPs before it, and what follows the key-on (a register, its value, thousands of `dbra` rounds to wait
/// first, NOPs).
struct Voice {
    writes: Vec<(u8, u8, u8, u16)>,
    extra: Vec<(u8, u8, u8, u16, u16)>,
}

/// The rest of `fm_voice`'s settings: the channel (0-5), `$22`, the channel's `$B4`, writes before the key-on, the
/// operators' SSG-EG registers, `$27`, the key's code (`$F0`, all four), and NOPs before the last write before the
/// key-on and before the key-on.
struct Setup {
    channel: u8,
    lfo: u8,
    b4: u8,
    pre: Vec<(u8, u8)>,
    ssg: [u8; 4],
    mode: u8,
    keys: u8,
    pre_nops: u16,
    key_nops: u16,
}

const PLAIN: Setup = Setup { channel: 0, lfo: 0, b4: 0xC0, pre: Vec::new(), ssg: [0; 4], mode: 0, keys: 0xF0, pre_nops: 0, key_nops: 0 };

/// S1-S4, the algorithm and feedback, the frequency, then every key on.
fn fm_voice(ops: [Op; 4], alg: u8, fb: u8, fnum: u16, block: u8, extra: &[(u8, u8, u16)]) -> Voice {
    fm_voice_on(ops, alg, fb, fnum, block, extra, PLAIN)
}

fn fm_voice_on(ops: [Op; 4], alg: u8, fb: u8, fnum: u16, block: u8, extra: &[(u8, u8, u16)], setup: Setup) -> Voice {
    let extra: Vec<_> = extra.iter().map(|&(r, v, wait)| (r, v, wait, 0)).collect();
    fm_voice_late(ops, alg, fb, fnum, block, &extra, setup)
}

/// The same with NOPs before any of the writes after the key-on.
fn fm_voice_late(ops: [Op; 4], alg: u8, fb: u8, fnum: u16, block: u8, extra: &[(u8, u8, u16, u16)], setup: Setup) -> Voice {
    let (part, i) = (setup.channel / 3, setup.channel % 3);
    let mut writes = vec![(0, 0x22, setup.lfo, 0), (0, 0x27, setup.mode, 0), (0, 0x28, 0, 0), (0, 0x2B, 0, 0), (part, 0xB0 + i, fb << 3 | alg, 0), (part, 0xB4 + i, setup.b4, 0)];
    for ((slot, op), eg) in [0x00u8, 0x08, 0x04, 0x0C].into_iter().zip(ops).zip(setup.ssg) {
        for (k, v) in op.into_iter().enumerate() {
            writes.push((part, 0x30 + 0x10 * k as u8 + slot + i, v, 0));
        }
        writes.push((part, 0x90 + slot + i, eg, 0));
    }
    writes.extend_from_slice(&[(part, 0xA4 + i, block << 3 | (fnum >> 8) as u8, 0), (part, 0xA0 + i, fnum as u8, 0)]);
    let last = setup.pre.len().saturating_sub(1);
    writes.extend(setup.pre.iter().enumerate().map(|(k, &(r, v))| (0, r, v, if k == last { setup.pre_nops } else { 0 })));
    writes.push((0, 0x28, setup.keys | part << 2 | i, setup.key_nops));
    let extra = extra.iter().map(|&(r, v, wait, nops)| (if r >= 0x30 { part } else { 0 }, r, v, wait, nops)).collect();
    Voice { writes, extra }
}

impl Voice {
    /// The 68000's program for it, as `mdboard.py` builds it.
    fn image(&self) -> Vec<u8> {
        self.image_then(&[])
    }

    /// The same with `tail` run after the voice's writes, before the idle loop.
    fn image_then(&self, tail: &[u16]) -> Vec<u8> {
        let mut code = Z80_BUS.to_vec();
        for &(part, r, v, nops) in &self.writes {
            ym_write_late(&mut code, r, v, part, nops);
        }
        for &(part, r, v, wait, nops) in &self.extra {
            code.extend_from_slice(&[0x303C, wait * 1000, 0x51C8, 0xFFFE]);
            ym_write_late(&mut code, r, v, part, nops);
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
            } else if let Some(v) = lfo_voice(name) {
                v
            } else if let Some(v) = chip_voice(name) {
                v
            } else if let Some(i) = number("fm-rom") {
                let (mm, tl, dr, cm, fnum, block) = ROM_VOICES[i as usize];
                fm_voice([QUIET, QUIET, [mm, tl, 0x1F, dr, 0x00, 0xFF], level(cm, 0x00)], 4, 0, fnum, block, &[])
            } else {
                panic!("no voice named {name}")
            }
        }
    }
}

/// `mdboard.py`'s LFO voices (Nephrite_Native.md §23).
fn lfo_voice(name: &str) -> Option<Voice> {
    const AM_S4: Op = [0x01, 0x00, 0x1F, 0x80, 0x00, 0x0F];
    let setup = |channel: u8, lfo: u8, b4: u8| Setup { channel, lfo, b4, ..PLAIN };
    let alone = |k: usize, op: Op| {
        let mut ops = [QUIET; 4];
        ops[k] = op;
        ops
    };
    let digit = |prefix: &str| name.strip_prefix(prefix).and_then(|n| n.parse::<u8>().ok());
    let a4 = |ops: [Op; 4], extra: &[(u8, u8, u16)], s: Setup| fm_voice_on(ops, 7, 0, 1081, 4, extra, s);
    let at_1500 = |ops: [Op; 4], s: Setup| fm_voice_on(ops, 7, 0, 1500, 4, &[], s);
    let special = |s1: u16, ch: u16, b4: u8| {
        let pre = vec![(0xAD, 4 << 3 | (s1 >> 8) as u8), (0xA9, s1 as u8), (0x27, 0x40)];
        fm_voice_on([SINE, QUIET, QUIET, QUIET], 7, 0, ch, 4, &[], Setup { channel: 2, lfo: 8 | 6, b4, pre, ..PLAIN })
    };
    // `fm-pm-c<n>s<k>` and `fm-am-c<n>s<k>`: operator k of channel n alone.
    let by_op = |prefix: &str| -> Option<(u8, usize)> {
        let rest = name.strip_prefix(prefix)?;
        let (c, k) = rest.split_once('s')?;
        Some((c.parse::<u8>().ok()? - 1, k.parse::<usize>().ok()? - 1))
    };
    Some(match name {
        "fm-am-off" => a4([QUIET, QUIET, QUIET, SINE], &[], setup(0, 8 | 7, 0xC0 | 3 << 4)),
        "fm-lfo-off" => a4([QUIET, QUIET, QUIET, AM_S4], &[], setup(0, 7, 0xC0 | 3 << 4)),
        "fm-lfo-restart" => a4([QUIET, QUIET, QUIET, AM_S4], &[(0x22, 0x07, 3), (0x22, 0x0E, 2), (0x22, 0x0B, 4)], setup(0, 8 | 6, 0xC0 | 3 << 4)),
        "fm-am-mod" => fm_voice_on([level(0x01, 0x30), level(0x02, 0x28), [0x01, 0x10, 0x1F, 0x80, 0, 0x0F], SINE], 0, 0, 1081, 4, &[], setup(0, 8 | 6, 0xC0 | 2 << 4)),
        "fm-pm-kc" => fm_voice_on([QUIET, QUIET, QUIET, [0x31, 0x00, 0x1F, 0, 0, 0x0F]], 7, 0, 1151, 4, &[], setup(0, 8 | 6, 0xC7)),
        "fm-pm-1500" => at_1500([QUIET, QUIET, QUIET, SINE], setup(0, 8 | 6, 0xC7)),
        "fm-pm-s1" => at_1500([SINE, QUIET, QUIET, QUIET], setup(0, 8 | 6, 0xC7)),
        "fm-pm-ch3-normal" => at_1500([SINE, QUIET, QUIET, QUIET], setup(2, 8 | 6, 0xC7)),
        "fm-pm-ch3" => special(1500, 1081, 0xC7),
        "fm-pm-ch3-p0" => special(1500, 1081, 0xC0),
        "fm-pm-ch3-swap" => special(1081, 1500, 0xC7),
        _ => {
            if let Some(f) = digit("fm-lfo") {
                a4([QUIET, QUIET, QUIET, AM_S4], &[], setup(0, 8 | f, 0xC0 | 3 << 4))
            } else if let Some(a) = digit("fm-ams") {
                a4([QUIET, QUIET, QUIET, AM_S4], &[], setup(0, 8 | 7, 0xC0 | a << 4))
            } else if let Some(p) = digit("fm-pms") {
                a4([QUIET, QUIET, QUIET, SINE], &[], setup(0, 8 | 6, 0xC0 | p))
            } else if let Some((c, k)) = by_op("fm-pm-c") {
                at_1500(alone(k, SINE), setup(c, 8 | 6, 0xC7))
            } else if let Some((c, k)) = by_op("fm-am-c") {
                a4(alone(k, AM_S4), &[], setup(c, 8 | 7, 0xC0 | 3 << 4))
            } else {
                let k = digit("fm-am-s")?;
                a4(alone(k as usize - 1, AM_S4), &[], setup(0, 8 | 7, 0xC0 | 3 << 4))
            }
        }
    })
}

/// `mdboard.py`'s voices of SSG-EG, CSM, the timers, the test register and the channel's sum (Nephrite_Native.md §24).
fn chip_voice(name: &str) -> Option<Voice> {
    const AM_S4: Op = [0x01, 0x00, 0x1F, 0x80, 0x00, 0x0F];
    const PLUCK: Op = [0x01, 0x00, 0x1F, 0x00, 0x00, 0x0A];
    const FALL: Op = [0x01, 0x00, 0x1F, 20, 0x00, 0xFA];
    const CHAIN: [Op; 4] = [level(0x01, 0x30), level(0x02, 0x28), level(0x01, 0x20), SINE];
    let alone = |k: usize, op: Op| {
        let mut ops = [QUIET; 4];
        ops[k] = op;
        ops
    };
    let hex = |prefix: &str| name.strip_prefix(prefix).and_then(|n| u8::from_str_radix(n, 16).ok());
    let a4 = |ops: [Op; 4], alg: u8, extra: &[(u8, u8, u16)], s: Setup| fm_voice_on(ops, alg, 0, 1081, 4, extra, s);
    let with = |pre: Vec<(u8, u8)>| Setup { pre, ..PLAIN };
    // `ssg_s4`: S4 alone with `$9C`, an instant attack and a decay of rate 42 unless told otherwise.
    let ssg_s4 = |eg: u8, op: Op, extra: &[(u8, u8, u16)]| a4([QUIET, QUIET, QUIET, op], 7, extra, Setup { ssg: [0, 0, 0, eg], ..PLAIN });
    let s4 = |ar: u8, dr: u8, sr: u8, sl_rr: u8| [0x01, 0x00, ar, dr, sr, sl_rr];
    let plain = s4(0x1F, 20, 0, 0xF8);
    // `csm`: channel 3's operators keyed by timer A every `period` samples, `$27` as `mode`.
    let csm = |ops: [Op; 4], period: u16, mode: u8, keys: u8, mut pre: Vec<(u8, u8)>, extra: &[(u8, u8, u16)], pre_nops: u16, key_nops: u16| {
        let value = 1024 - period;
        pre.extend_from_slice(&[(0x24, (value >> 2) as u8), (0x25, (value & 3) as u8), (0x27, mode)]);
        a4(ops, 7, extra, Setup { channel: 2, keys, pre, pre_nops, key_nops, ..PLAIN })
    };
    let own_1500 = || {
        let mut pre: Vec<(u8, u8)> = [0xA9u8, 0xAA, 0xA8].iter().map(|&a| (a + 4, 4 << 3 | (1500u16 >> 8) as u8)).collect();
        pre.extend([0xA9u8, 0xAA, 0xA8].iter().map(|&a| (a, 1500u16 as u8)));
        pre
    };
    let pairs = |ws: &[(u8, u8)]| -> Vec<(u8, u8, u16)> { ws.iter().enumerate().map(|(k, &(r, v))| (r, v, if k == 0 { 20 } else { 5 })).collect() };
    let loud = |mul: u8| level(mul, 0x00);
    let soft = |mul: u8| level(mul, 0x18);
    Some(match name {
        "fm-ssg-tl" => ssg_s4(14, [0x01, 0x18, 0x1F, 20, 0, 0xF8], &[]),
        "fm-ssg-am" => a4([QUIET, QUIET, QUIET, s4(0x1F, 0x80 | 20, 0, 0xF8)], 7, &[], Setup { ssg: [0, 0, 0, 14], lfo: 8 | 7, b4: 0xC0 | 3 << 4, ..PLAIN }),
        "fm-ssg-on" => ssg_s4(0, s4(0x1F, 13, 0, 0xF8), &[(0x9C, 0x08, 20)]),
        "fm-ssg-flip" => ssg_s4(8, plain, &[(0x9C, 0x0C, 20)]),
        "fm-ssg-off" => ssg_s4(12, plain, &[(0x9C, 0x00, 20)]),
        "fm-ssg-hold-off" => ssg_s4(9, plain, &[(0x9C, 0x00, 20)]),
        "fm-ssg-hold-att" => ssg_s4(9, plain, &[(0x9C, 0x0D, 20)]),
        "fm-ssg-a-to-b" => ssg_s4(10, plain, &[(0x9C, 0x0B, 20)]),
        "fm-ssg-b-to-a" => ssg_s4(11, plain, &[(0x9C, 0x0A, 21)]),
        "fm-ssg-ar-relc-3" | "fm-ssg-ar-relc-5" | "fm-ssg-ar-rele-3" | "fm-ssg-ar-rele-5" => {
            let eg = if name.contains("relc") { 12 } else { 14 };
            ssg_s4(eg, s4(0x12, 20, 0, 0xFA), &[(0x28, 0x00, if name.ends_with('3') { 3 } else { 5 })])
        }
        "fm-ssg-mod8" | "fm-ssg-mode" => {
            let eg = if name == "fm-ssg-mod8" { 8 } else { 14 };
            a4([QUIET, QUIET, [0x01, 0x10, 0x1F, 20, 0, 0xF8], SINE], 4, &[], Setup { ssg: [0, 0, eg, 0], ..PLAIN })
        }
        "fm-csm" => csm([QUIET, QUIET, QUIET, PLUCK], 100, 0x81, 0, vec![], &[], 0, 0),
        "fm-csm-all" => csm([[0x01, 0x10, 0x1F, 0, 0, 0x0A], [0x02, 0x14, 0x1F, 0, 0, 0x0A], [0x03, 0x18, 0x1F, 0, 0, 0x0A], [0x05, 0x1C, 0x1F, 0, 0, 0x0A]], 100, 0x81, 0, vec![], &[], 0, 0),
        "fm-csm-ar" | "fm-csm-ar-p1" => csm([QUIET, QUIET, QUIET, [0x01, 0x00, 0x14, 0, 0, 0x0A]], if name == "fm-csm-ar" { 100 } else { 1 }, 0x81, 0, vec![], &[], 0, 0),
        "fm-csm-11" => csm([QUIET, QUIET, QUIET, PLUCK], 100, 0xC1, 0, vec![], &[], 0, 0),
        "fm-csm-noload" => csm([QUIET, QUIET, QUIET, PLUCK], 100, 0x80, 0, vec![], &[], 0, 0),
        "fm-csm-flags" => csm([QUIET, QUIET, QUIET, PLUCK], 100, 0x8F, 0, vec![], &[], 0, 0),
        "fm-csm-key" => csm([QUIET, QUIET, QUIET, FALL], 100, 0x81, 0xF0, vec![], &[(0x28, 0x02, 20)], 0, 18),
        "fm-csm-key-plain" => csm([QUIET, QUIET, QUIET, FALL], 100, 0x01, 0xF0, vec![], &[], 0, 18),
        "fm-csm-off" => csm([QUIET, QUIET, QUIET, PLUCK], 100, 0x81, 0, vec![], &[(0x27, 0x01, 20), (0x27, 0x81, 5)], 0, 0),
        "fm-test-timer" => csm([QUIET, QUIET, QUIET, PLUCK], 100, 0x81, 0, vec![(0x21, 0x04)], &[], 0, 0),
        "fm-test-lfo-am" => a4([QUIET, QUIET, QUIET, AM_S4], 7, &[], Setup { lfo: 8 | 7, b4: 0xC0 | 3 << 4, pre: vec![(0x21, 0x02)], ..PLAIN }),
        "fm-test-lfo-pm" => a4([QUIET, QUIET, QUIET, SINE], 7, &[], Setup { lfo: 8 | 6, b4: 0xC7, pre: vec![(0x21, 0x02)], ..PLAIN }),
        "fm-test-lfo-late" => a4([QUIET, QUIET, QUIET, AM_S4], 7, &pairs(&[(0x21, 0x02), (0x21, 0x00)]), Setup { lfo: 8 | 7, b4: 0xC0 | 3 << 4, ..PLAIN }),
        "fm-test-pg" => a4([QUIET, QUIET, QUIET, SINE], 7, &pairs(&[(0x21, 0x08), (0x21, 0x00)]), PLAIN),
        "fm-test-eg" => a4([QUIET, QUIET, QUIET, s4(0x1F, 13, 0, 0xFA)], 7, &pairs(&[(0x21, 0x20), (0x21, 0x00)]), PLAIN),
        "fm-test-ugly" => a4([QUIET, QUIET, QUIET, SINE], 7, &pairs(&[(0x21, 0x10), (0x21, 0x00)]), PLAIN),
        "fm-test-ugly-chain" => a4(CHAIN, 0, &pairs(&[(0x21, 0x10), (0x21, 0x00)]), PLAIN),
        "fm-test-pg0" => a4([QUIET, QUIET, QUIET, SINE], 7, &[(0x21, 0x00, 20)], with(vec![(0x21, 0x08)])),
        "fm-test-eg0" => a4([QUIET, QUIET, QUIET, s4(0x1F, 20, 0, 0xFA)], 7, &[], with(vec![(0x21, 0x20)])),
        "fm-test-ugly0" => a4([QUIET, QUIET, QUIET, SINE], 7, &[], with(vec![(0x21, 0x10)])),
        "fm-test-ugly0-chain" => a4(CHAIN, 0, &[], with(vec![(0x21, 0x10)])),
        "fm-test-ugly0-fb" => fm_voice_on([SINE, QUIET, QUIET, QUIET], 7, 5, 1081, 4, &[], with(vec![(0x21, 0x10)])),
        "fm-test-dac0" => a4([QUIET, QUIET, QUIET, SINE], 7, &[], with(vec![(0x2A, 0xA0), (0x2B, 0x80), (0x2C, 0x20)])),
        "fm-test-dac0-off" => a4([QUIET, QUIET, QUIET, SINE], 7, &[], with(vec![(0x2A, 0xA0), (0x2C, 0x20)])),
        "fm-test-read" => a4([QUIET, QUIET, QUIET, SINE], 7, &pairs(&[(0x21, 0x01), (0x21, 0x40), (0x21, 0x80), (0x21, 0xC1), (0x21, 0x00)]), PLAIN),
        "fm-test-2c" => a4([QUIET, QUIET, QUIET, SINE], 7, &pairs(&[(0x2C, 0x10), (0x2C, 0x20), (0x2C, 0x40), (0x2C, 0x80), (0x2C, 0x00)]), PLAIN),
        "fm-test-2c-dac" => a4([QUIET, QUIET, QUIET, SINE], 7, &pairs(&[(0x2C, 0x20), (0x2C, 0x00)]), with(vec![(0x2A, 0xA0), (0x2B, 0x80)])),
        "fm-clip7" => a4([loud(1), loud(2), loud(3), loud(5)], 7, &[], PLAIN),
        "fm-clip5" => a4([soft(1), loud(2), loud(3), loud(5)], 5, &[], PLAIN),
        "fm-clip4" => a4([soft(1), loud(1), soft(3), loud(3)], 4, &[], PLAIN),
        _ => {
            // `fm-ssg<pattern>`, `fm-ssg-rel<pattern>`, `fm-ssg-rekey<pattern>`, `fm-ssg-ar<pattern>`, the SL and fast
            // decays, each operator's place, the envelope by operator, CSM by operator and period, the timers' test bit.
            if let Some(m) = hex("fm-ssg-rekey") {
                ssg_s4(m, s4(0x1F, 20, 0, 0xFA), &[(0x28, 0x00, 20), (0x28, 0xF0, 3)])
            } else if let Some(m) = hex("fm-ssg-rel") {
                ssg_s4(m, s4(0x1F, 20, 0, 0xFA), &[(0x28, 0x00, 20)])
            } else if let Some(m) = hex("fm-ssg-ar") {
                ssg_s4(m, s4(0x12, 20, 0, 0xF8), &[])
            } else if let Some(m) = hex("fm-ssg-sl") {
                ssg_s4(m, s4(0x1F, 24, 14, 0x48), &[])
            } else if let Some(m) = hex("fm-ssg-fast") {
                ssg_s4(m, s4(0x1F, 31, 0, 0xF8), &[])
            } else if let Some(m) = hex("fm-ssg") {
                ssg_s4(m, plain, &[])
            } else if let Some(rest) = name.strip_prefix("fm-ssg-c") {
                let (c, k, m) = (rest[0..1].parse::<u8>().ok()? - 1, rest[2..3].parse::<usize>().ok()? - 1, u8::from_str_radix(&rest[4..], 16).ok()?);
                let mut ssg = [0; 4];
                ssg[k] = m;
                a4(alone(k, plain), 7, &[], Setup { channel: c, ssg, ..PLAIN })
            } else if let Some(rest) = name.strip_prefix("fm-eg-c") {
                let (c, k) = (rest[0..1].parse::<u8>().ok()? - 1, rest[2..3].parse::<usize>().ok()? - 1);
                a4(alone(k, s4(0x12, 20, 8, 0x4A)), 7, &[(0x28, ((c / 3) << 2) | (c % 3), 43)], Setup { channel: c, ..PLAIN })
            } else if let Some(k) = name.strip_prefix("fm-csm-o").and_then(|k| k.parse::<usize>().ok()) {
                csm(alone(k - 1, PLUCK), 100, 0x81, 0, own_1500(), &[], 12, 0)
            } else if let Some(p) = name.strip_prefix("fm-csm-p").and_then(|p| p.parse::<u16>().ok()) {
                csm([QUIET, QUIET, QUIET, FALL], p, 0x81, 0, vec![], &[], 0, 0)
            } else if let Some(p) = name.strip_prefix("fm-test-timer-").and_then(|p| p.parse::<u16>().ok()) {
                csm([QUIET, QUIET, QUIET, PLUCK], p, 0x81, 0, vec![(0x21, 0x04)], &[], 0, 0)
            } else {
                let k = name.strip_prefix("fm-test-ugly0-s").and_then(|k| k.parse::<usize>().ok())?;
                a4(alone(k - 1, SINE), 7, &[], with(vec![(0x21, 0x10)]))
            }
        }
    })
}

fn fnv(bytes: impl IntoIterator<Item = u8>) -> u32 {
    bytes.into_iter().fold(0x811C_9DC5u32, |h, b| (h ^ b as u32).wrapping_mul(0x0100_0193))
}

/// The machine after `frames` frames of `image`, and channel 1's output for every sample of them.
fn machine(image: &[u8], frames: i64) -> (Machine, Vec<i32>) {
    machine_channel(image, frames, 0)
}

/// The same for any channel's output.
fn machine_channel(image: &[u8], frames: i64, channel: usize) -> (Machine, Vec<i32>) {
    machine_placed(image, frames, channel, false)
}

/// Where the board has its bus refresh and its YM2612's first deadline against its 68000's start, in Nephrite's
/// clock: placed there, the whole machine makes each YM2612 write of a program on part 0 at the board's master clock
/// (Nephrite_Native.md §24; §22.3 for the placement against the picture that Nephrite keeps).
const BOARD_REFRESH: u64 = 98 + 777;
const BOARD_YM: u64 = 910;

/// A machine just made, its bus refresh and its YM2612's cycle moved to where the board has them against its 68000.
pub(crate) fn place_as_the_board(m: &mut Machine) {
    m.genesis.hw.refresh_at = BOARD_REFRESH;
    let ym = &mut m.genesis.hw.sound.ym;
    (ym.next, ym.timers_next, ym.held_at) = (BOARD_YM, BOARD_YM, BOARD_YM);
}

/// The whole machine, at Nephrite's placement or at the board's against its 68000's start.
fn machine_placed(image: &[u8], frames: i64, channel: usize, board: bool) -> (Machine, Vec<i32>) {
    let mut m = Machine::new(image, Media::read(image));
    if board {
        place_as_the_board(&mut m);
    }
    m.genesis.hw.sound.ym.trace = Some(Vec::new());
    while m.frames < frames {
        m.advance();
    }
    let trace = m.genesis.hw.sound.ym.trace.take().unwrap().iter().map(|c| c[channel]).collect();
    (m, trace)
}

/// A chip whose reset line is asserted at master clock `asserted` and released at `released`, then given each of
/// `writes` (a master clock, the port, the value): channel 1's samples from its first that is neither 0 nor -1.
fn chip(asserted: u64, released: u64, writes: impl Iterator<Item = (u64, u16, u8)>, samples: usize) -> Vec<i32> {
    let all = chip_channel(asserted, released, writes, samples, 0);
    let first = all.iter().position(|&x| x != 0 && x != -1).expect("the voice leaves rest");
    all[first..].to_vec()
}

/// The board's first sample deadline after power-on, in master clocks, where the reset line starts asserted: each
/// key of the landing sweeps is in the board's sample with it there, and none 7 master clocks either side
/// (Nephrite_Native.md §24).
const BOARD_POWER_ON: u64 = 537;

/// The same for any channel, from the chip's first sample; a chip asserted at 0 is the board's at power-on.
fn chip_channel(asserted: u64, released: u64, writes: impl Iterator<Item = (u64, u16, u8)>, samples: usize, channel: usize) -> Vec<i32> {
    let mut y = Ym2612::new(true);
    if asserted == 0 {
        (y.held, y.next, y.timers_next, y.held_at) = (true, BOARD_POWER_ON, BOARD_POWER_ON, BOARD_POWER_ON);
    } else {
        y.run(asserted, |_, _| {});
        y.reset_line(true, asserted);
    }
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
    y.trace.take().unwrap().iter().map(|c| c[channel]).collect()
}

/// A voice on the chip alone, each write at the master clock the board's 68000 made it (`board_chip.rs`): the chip's
/// samples on the voice's channel.
fn on_the_chip(name: &str, channel: usize, samples: usize) -> Vec<i32> {
    let v = voice(name);
    let times = crate::board_chip::WRITE_TIMES.iter().find(|w| w.0 == name).unwrap_or_else(|| panic!("{name}: no write times")).1;
    let ports: Vec<(u16, u8)> = v.writes.iter().map(|&(part, r, value, _)| (part, r, value)).chain(v.extra.iter().map(|&(part, r, value, _, _)| (part, r, value)))
        .flat_map(|(part, r, value)| [(2 * part as u16, r), (2 * part as u16 + 1, value)]).collect();
    assert_eq!(times.len(), ports.len(), "{name}: the board's writes are not the program's");
    let mut t = crate::board_chip::WRITES_RELEASE;
    let writes = times.iter().zip(ports).map(move |(&d, (port, value))| {
        t += d as u64;
        (t / 2, port, value)
    });
    chip_channel(0, crate::board_chip::WRITES_RELEASE / 2, writes, samples, channel)
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
    let ports = voice.writes.iter().flat_map(|&(part, r, value, _)| [(2 * part as u16, r), (2 * part as u16 + 1, value)]);
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

/// The whole machine running the 68000's programs on its own timing, every voice the board's to the sample at
/// Nephrite's placement and at the board's: the 68000's five-clock access to the Z80's area and the refreshes it
/// waits for (D-2, D-14), the busy flag's length (D-13) and the chip's cycle against the 68000 (D-19, D-24) put each
/// write in the board's sample.
#[test]
fn the_whole_machine_plays_every_voice_as_the_board_does() {
    for board in [false, true] {
        let mut wrong = Vec::new();
        let (mut equal_samples, mut samples) = (0, 0);
        for v in VOICES {
            let (_, ours) = machine_placed(&voice(v.name).image(), (v.samples as i64 + 2500) / 880 + 2, 0, board);
            let first = ours.iter().position(|&x| x != 0 && x != -1).expect("the voice leaves rest");
            let e = equal(&ours[first..], v.samples, v.blocks);
            if e != v.samples {
                wrong.push((v.name, e, v.samples));
            }
            (equal_samples, samples) = (equal_samples + e, samples + v.samples);
        }
        assert!(wrong.is_empty(), "voices unlike the board's, placed as the board is: {board}: {wrong:?}");
        assert_eq!((equal_samples, samples), (200_220, 200_220));
    }
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

/// The LFO's voices on the whole machine against the board's (Nephrite_Native.md §23, §24), whose chip is the board's
/// in all of them but `fm-lfo-restart` (`the_lfo_voices_on_the_chip_are_the_boards`). Placed as the board is against
/// its 68000's start, the whole machine is the board's in all but that one and nine on part 1, whose writes the 68000
/// makes a poll of the busy flag early or late; at Nephrite's placement against the picture, in all but that one and
/// channel 3's two in its special mode, which then take the vibrato's count a sample out.
#[test]
fn the_lfo_voices_are_the_boards_sample_for_sample() {
    for (board, known) in [
        (true, &["fm-lfo-restart", "fm-pm-c4s1", "fm-pm-c4s2", "fm-pm-c4s3", "fm-pm-c4s4", "fm-pm-c5s1", "fm-pm-c5s2", "fm-pm-c5s3", "fm-pm-c5s4", "fm-am-c4s1"][..]),
        (false, &["fm-lfo-restart", "fm-pm-ch3-swap", "fm-pm-ch3"][..]),
    ] {
        let mut wrong = Vec::new();
        for v in LFO_VOICES {
            let image = voice(v.name).image();
            assert_eq!(fnv(image.iter().copied()), v.image, "{}: the program is not the one the board ran", v.name);
            let (_, ours) = machine_placed(&image, (v.samples as i64 + 2500) / 880 + 2, v.channel, board);
            let first = ours.iter().position(|&x| x != 0 && x != -1).expect("the voice leaves rest");
            if equal(&ours[first..], v.samples, v.blocks) != v.samples {
                wrong.push(v.name);
            }
        }
        assert_eq!(wrong, known, "LFO voices unlike the board's, placed as the board is: {board}");
    }
}

/// A chip voice on the whole machine, at the board's placement or Nephrite's: the samples held that are the board's,
/// from the first sample that is not at rest (Nephrite_Native.md §24).
fn chip_equal(v: &crate::board_chip::BoardChipVoice, board: bool) -> usize {
    let image = voice(v.name).image();
    assert_eq!(fnv(image.iter().copied()), v.image, "{}: the program is not the one the board ran", v.name);
    let (_, ours) = machine_placed(&image, 8600 / 880 + 5, v.channel, board);
    let rest: &[i32] = if v.name.starts_with("fm-test-ugly0") { &[0, -1, -256] } else { &[0, -1] };
    match ours.iter().position(|x| !rest.contains(x)) {
        None => if v.samples == 0 { 1 } else { 0 },
        Some(_) if v.samples == 0 => 0,
        Some(first) => equal(&ours[first..], v.samples, v.blocks),
    }
}

/// The program of one of `mdboard.py`'s landing sweeps: S1-S4 of channels 1-3 keyed, or CSM's timer loaded, `k`
/// NOPs late.
fn sweep_image(kind: &str, k: u16) -> Vec<u8> {
    if kind == "csm" {
        const PLUCK: Op = [0x01, 0x00, 0x1F, 0x00, 0x00, 0x0A];
        let pre = vec![(0x24, ((1024 - 100) >> 2) as u8), (0x25, 0), (0x27, 0x81)];
        return fm_voice_on([QUIET, QUIET, QUIET, PLUCK], 7, 0, 1081, 4, &[], Setup { channel: 2, keys: 0, pre, pre_nops: k, ..PLAIN }).image();
    }
    let (c, o) = (kind[3..4].parse::<u8>().unwrap() - 1, kind[5..6].parse::<usize>().unwrap() - 1);
    let mut ops = [QUIET; 4];
    ops[o] = SINE;
    fm_voice_on(ops, 7, 0, 1081, 4, &[], Setup { channel: c, key_nops: k, ..PLAIN }).image()
}

/// The same voices on the whole machine. Placed as the board is against its 68000's start, 108 are the board's to the
/// sample: the test register's nine and three envelopes on part 1, whose writes the 68000 makes a poll of the busy
/// flag early or late there, are not.
#[test]
fn the_chip_voices_on_the_whole_machine_at_the_boards_placement() {
    let exact = CHIP_VOICES.iter().filter(|v| chip_equal(v, true) == v.samples.max(1)).count();
    assert_eq!(exact, 108);
}

/// The same voices at Nephrite's own placement, the YM2612's cycle against the picture (§22.3): a third of a sample
/// from where the board has it against the 68000, so that a write the 68000 times lands in another sample now and
/// then, CSM's timer load and the test register's among them. Held as a count.
#[test]
fn the_chip_voices_at_nephrites_placement() {
    let exact = CHIP_VOICES.iter().filter(|v| chip_equal(v, false) == v.samples.max(1)).count();
    assert_eq!(exact, 89);
}

/// Each key of S1-S4 on channels 1-3, and CSM's timer, moved by NOPs across the sample: on the whole machine placed
/// as the board is, each lands in the board's sample. The key is taken a slot later on each channel, S1's a sample
/// after the others', and the timer's load in the sample its tick follows (Nephrite_Native.md §24).
#[test]
fn keys_and_the_timers_load_land_in_the_boards_samples() {
    let mut wrong = Vec::new();
    for &(kind, board) in crate::board_chip::SWEEPS {
        let channel = if kind == "csm" { 2 } else { kind[3..4].parse::<usize>().unwrap() - 1 };
        let firsts: Vec<usize> = (0..40)
            .map(|k| {
                let (_, ours) = machine_placed(&sweep_image(kind, k), 3, channel, true);
                ours.iter().position(|&x| x != 0 && x != -1).expect("the voice sounds")
            })
            .collect();
        let ours: Vec<i8> = firsts.iter().map(|&f| (f as i64 - firsts[0] as i64) as i8).collect();
        if ours != board {
            wrong.push((kind, ours));
        }
    }
    assert!(wrong.is_empty(), "sweeps unlike the board's: {wrong:?}");
}

/// How many of a voice's held samples the chip alone makes as the board did, from the first sample not at rest.
fn equal_on_the_chip(name: &str, channel: usize, samples: usize, blocks: &[u32]) -> usize {
    let ours = on_the_chip(name, channel, samples.max(8600));
    let rest: &[i32] = if name.starts_with("fm-test-ugly0") { &[0, -1, -256] } else { &[0, -1] };
    match ours.iter().position(|x| !rest.contains(x)) {
        None => usize::from(samples == 0),
        Some(_) if samples == 0 => 0,
        Some(first) => equal(&ours[first..], samples, blocks),
    }
}

/// The test register's writes land in another sample on the board than at once (Nephrite_Disputes.md D-15): the nine
/// voices that write it as they sound differ there, each held to the samples it has now so that a change is seen.
const TEST_LANDINGS: [(&str, usize); 9] = [
    ("fm-test-lfo-late", 1408), ("fm-test-pg", 1728), ("fm-test-eg", 3992), ("fm-test-ugly", 8536), ("fm-test-ugly-chain", 8472),
    ("fm-test-pg0", 1408), ("fm-test-eg0", 0), ("fm-test-2c", 8472), ("fm-test-2c-dac", 8472),
];

/// The voices of SSG-EG, CSM, the timers' test bit, the test register and the channel's sum on the chip alone, each
/// write at the master clock the board's 68000 made it (Nephrite_Native.md §24): every SSG-EG pattern, its attack,
/// release, hold and its register written as it runs, on a modulator and on each operator's place; CSM by period,
/// operator and mode, with a manual key; the timers counting a slot at a time; the test register's bits; carriers
/// past nine bits. All the board's to the sample but the nine of `TEST_LANDINGS`.
#[test]
fn the_chip_voices_are_the_boards_sample_for_sample() {
    let (mut wrong, mut exact, mut samples) = (Vec::new(), 0, 0);
    for v in CHIP_VOICES {
        let e = equal_on_the_chip(v.name, v.channel, v.samples, v.blocks);
        match TEST_LANDINGS.iter().find(|k| k.0 == v.name) {
            Some(&(_, held)) if e != held => wrong.push((v.name, e, held)),
            Some(_) => {}
            None if e != v.samples.max(1) => wrong.push((v.name, e, v.samples)),
            None => (exact, samples) = (exact + 1, samples + v.samples),
        }
    }
    assert!(wrong.is_empty(), "voices unlike the board's: {wrong:?}");
    assert_eq!((CHIP_VOICES.len(), exact, samples), (120, 111, 891_386));
}

/// The LFO's voices on the chip alone: every one the board's to the sample, channel 3's special mode under the
/// vibrato among them, but `fm-lfo-restart`, whose writes to `$22` as it sounds land in another sample (D-15).
#[test]
fn the_lfo_voices_on_the_chip_are_the_boards() {
    let (mut wrong, mut samples) = (Vec::new(), 0);
    for v in LFO_VOICES {
        let e = equal_on_the_chip(v.name, v.channel, v.samples, v.blocks);
        match v.name {
            "fm-lfo-restart" => assert_eq!(e, 2944, "{}", v.name),
            _ if e != v.samples => wrong.push((v.name, e, v.samples)),
            _ => samples += e,
        }
    }
    assert!(wrong.is_empty(), "LFO voices unlike the board's: {wrong:?}");
    assert_eq!(samples, 591_800 - 17_000);
}
