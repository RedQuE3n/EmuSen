//! The SPU's rules, each by its own test (Diamond_SPU.md).

use super::*;

const TICK: u64 = CYCLES_PER_SAMPLE;

fn silent() -> impl FnMut() -> [i16; 2] {
    || [0, 0]
}

/// A block at `at` of filter 0 and `shift`, its 28 nibbles `n(i)`, its flags `flags`.
fn block(spu: &mut Spu, at: usize, shift: u8, flags: u8, n: impl Fn(usize) -> u8) {
    spu.ram[at] = shift;
    spu.ram[at + 1] = flags;
    for i in 0..14 {
        spu.ram[at + 2 + i] = n(2 * i) & 0xF | (n(2 * i + 1) & 0xF) << 4;
    }
}

/// The SPU on, unmuted, at full main volume, voice `v` keyed on at `start` with `pitch`, its envelope's attack instant.
fn playing(v: usize, start: u32, pitch: u16) -> Spu {
    let mut s = Spu::new();
    let mut t = 0;
    for (r, val) in [(reg::ATTR, 0xC000), (reg::MVOL, 0x3FFF), (reg::MVOL + 1, 0x3FFF)] {
        s.write16(r as u32 * 2, val, t);
    }
    let base = (v * 16) as u32;
    for (r, val) in [(0, 0x3FFF), (2, 0x3FFF), (4, pitch), (6, (start / 8) as u16), (8, 0x00FF), (10, 0x0000)] {
        s.write16(base + r, val, t);
    }
    s.write16(reg::KON as u32 * 2 + if v >= 16 { 2 } else { 0 }, 1 << (v % 16), t);
    t += 1;
    s.run(t, &mut silent());
    s
}

fn ticks(s: &mut Spu, n: u64) {
    let t = s.next_tick + (n - 1) * TICK;
    s.run(t, &mut silent());
}

#[test]
fn filter_zero_decodes_each_nibble_shifted() {
    let mut s = Spu::new();
    block(&mut s, 0x1000, 4, 0, |i| (i % 16) as u8);
    let v = &mut s.voices[0];
    v.start = 0x1000 / 8;
    v.key_on(&s.ram);
    v.decode_word(&s.ram, 0);
    v.decode_word(&s.ram, 1);
    let want: Vec<i16> = (0..8).map(|i| (((i % 16) as i32) << 28 >> 16 >> 4) as i16).collect();
    assert_eq!(&v.samples[3..11], &want[..]);
}

#[test]
fn a_filter_predicts_from_the_two_samples_before_and_a_reserved_shift_acts_as_nine() {
    let mut s = Spu::new();
    block(&mut s, 0x1000, 0x10 | 13, 0, |i| if i == 0 { 4 } else { 0 });
    let v = &mut s.voices[0];
    v.start = 0x200;
    v.key_on(&s.ram);
    v.decode_word(&s.ram, 0);
    let first = (4 << 12) >> 9;
    assert_eq!(v.samples[3], first, "shift 13 as 9");
    assert_eq!(v.samples[4] as i32, first as i32 * 60 >> 6, "filter 1: 60/64 of the one before, each product shifted apart (D-34)");
    let mut s = Spu::new();
    block(&mut s, 0x1000, 0x20, 0, |i| [7, 9, 0, 0][i.min(3)]);
    let v = &mut s.voices[0];
    v.start = 0x200;
    v.key_on(&s.ram);
    v.decode_word(&s.ram, 0);
    let (a, b) = (v.samples[3] as i32, v.samples[4] as i32);
    assert_eq!(v.samples[5] as i32, (b * 115 >> 6) + (a * -52 >> 6), "filter 2, its two products truncated apart");
}

/// A held level comes out at the Gaussian table's gain, its four taps summing to 7F7Fh-7F81h of 8000h, through a full
/// envelope, the volume halved and the main volume halved.
#[test]
fn a_held_level_interpolates_at_the_tables_gain() {
    let mut s = Spu::new();
    for b in 0..8 {
        block(&mut s, 0x1000 + 16 * b, 0, if b == 7 { 3 } else { 0 }, |_| 4);
    }
    s.voices[0].repeat = 0x200;
    let mut s2 = playing(0, 0x1000, 0x1000);
    s2.ram.copy_from_slice(&s.ram);
    s2.voices[0].key_on(&s.ram);
    ticks(&mut s2, 200);
    let out = s2.voices[0].out as i32;
    let level = 4 << 12;
    assert!(((level * 0x7F7F >> 15) - 1..=level * 0x7F81 >> 15).contains(&out), "{out:X}: the four products summed, then shifted (D-35), then the envelope's");
    let mut buf = [0i16; 2];
    while s2.out.len() > 2 {
        s2.out.drain(&mut buf, 1);
    }
    s2.out.drain(&mut buf, 1);
    let through = (out * 0x7FFE >> 15) * 0x7FFE >> 15;
    assert_eq!(buf[0] as i32, through, "the voice's volume then the main volume, each 3FFFh doubled");
}

#[test]
fn the_loop_flags_jump_end_and_mute() {
    let mut s = playing(2, 0x2000, 0x4000);
    block(&mut s, 0x2000, 0, 4, |_| 1);
    block(&mut s, 0x2010, 0, 3, |_| 1);
    block(&mut s, 0x3000, 0, 1, |_| 1);
    s.voices[2].key_on(&s.ram);
    assert_eq!(s.voices[2].repeat, 0x2000 / 8, "the first block's loop start set LSAX at key on");
    ticks(&mut s, 7);
    assert_eq!(s.voices[2].address, 0x2010, "28 samples at four a sample");
    ticks(&mut s, 7);
    assert_eq!((s.voices[2].address, s.endx), (0x2000, 1 << 2), "end and repeat: back to LSAX, ENDX set");
    s.voices[2].repeat = 0x3000 / 8;
    ticks(&mut s, 14);
    assert_eq!(s.voices[2].address, 0x3000);
    assert!(s.voices[2].envelope.level > 0);
    ticks(&mut s, 7);
    assert_eq!((s.voices[2].phase, s.voices[2].envelope.level), (voice::Phase::Release, 0), "end and mute: released at zero");
    s.write16(reg::KON as u32 * 2, 1 << 2, s.next_tick - 1);
    ticks(&mut s, 1);
    assert_eq!(s.endx, 0, "a key on clears its ENDX bit");
}

#[test]
fn key_on_and_off_act_at_the_next_sample() {
    let mut s = playing(0, 0x1000, 0x1000);
    assert_eq!(s.voices[0].phase, voice::Phase::Release, "not before its sample");
    ticks(&mut s, 1);
    assert_eq!(s.voices[0].phase, voice::Phase::Attack);
    s.write16(reg::KOFF as u32 * 2, 1, s.next_tick - 5);
    assert_eq!(s.read16(reg::KOFF as u32 * 2), 1, "KOFF reads as written");
    assert_ne!(s.voices[0].phase, voice::Phase::Release);
    ticks(&mut s, 1);
    assert_eq!(s.voices[0].phase, voice::Phase::Release);
}

#[test]
fn the_attack_reaches_decay_then_sustains_at_its_level() {
    let mut s = playing(0, 0x1000, 0x1000);
    s.voices[0].adsr = [0x0003, 0x0000];
    ticks(&mut s, 3);
    assert_eq!((s.voices[0].phase, s.voices[0].envelope.level), (voice::Phase::Decay, 0x7FFF), "shift 0, +7: three steps");
    ticks(&mut s, 1);
    assert_eq!((s.voices[0].phase, s.voices[0].envelope.level), (voice::Phase::Decay, 0x3FFF), "decay shift 0 halves");
    ticks(&mut s, 1);
    assert_eq!((s.voices[0].phase, s.voices[0].envelope.level), (voice::Phase::Sustain, 0x1FFF), "at (3 + 1) × 800h or below");
}

#[test]
fn pitch_modulation_scales_by_the_voice_before_with_its_sign_glitch() {
    let mut s = playing(0, 0x1000, 0x1000);
    s.write16(1 << 4 | 4, 0x1000, 0);
    s.write16(reg::PMON as u32 * 2, 2, 0);
    s.voices[0].out = 0;
    let before = s.voices[1].counter;
    s.voices[0].pitch = 0;
    ticks(&mut s, 1);
    let out0 = s.voices[0].out as i64;
    let want = ((0x1000i64 * (out0 + 0x8000)) >> 15) as u32 & 0xFFFF;
    assert_eq!(s.voices[1].counter - before, want.min(0x4000));
}

#[test]
fn a_manual_write_drains_a_halfword_each_32_cycles() {
    let mut s = Spu::new();
    s.write16(reg::TSA as u32 * 2, 0x1000 / 8, 0);
    for k in 0..4u16 {
        s.write16(reg::DATA as u32 * 2, 0x1111 * (k + 1), 0);
    }
    s.write16(reg::ATTR as u32 * 2, 1 << 4, 10);
    s.run(TICK - 1, &mut silent());
    assert_eq!(s.status() & 0x43F, 0, "the mode waits for the next sample");
    s.run(TICK, &mut silent());
    assert_eq!(s.status() & 0x43F, 0x410, "applied, its first slot taken, and busy");
    s.run(TICK + 2 * 32, &mut silent());
    assert_eq!(&s.ram[0x1000..0x1008], &[0x11, 0x11, 0x22, 0x22, 0x33, 0x33, 0, 0]);
    assert_eq!(s.status() & 0x400, 0x400);
    s.run(TICK + 3 * 32, &mut silent());
    assert_eq!((s.ram[0x1006], s.status() & 0x400), (0x44, 0), "the fourth, and no longer busy");
}

#[test]
fn dma_write_requests_an_empty_fifo_and_read_a_full_one() {
    let mut s = Spu::new();
    s.write16(reg::TSA as u32 * 2, 0x1000 / 8, 0);
    s.write16(reg::ATTR as u32 * 2, 2 << 4, 0);
    s.run(TICK, &mut silent());
    assert!(s.dreq() && s.status() & 0x380 == 0x180);
    for w in 0..16 {
        s.dma_write(w, TICK);
    }
    assert!(!s.dreq());
    assert_eq!(s.dreq_change(), s.next_slot + 31 * 32);
    s.run(s.dreq_change(), &mut silent());
    assert!(s.dreq(), "empty again 32 halfwords later");
    s.write16(reg::TSA as u32 * 2, 0x1000 / 8, s.next_tick);
    s.write16(reg::ATTR as u32 * 2, 3 << 4, s.next_tick);
    s.run(s.next_tick, &mut silent());
    assert!(!s.dreq(), "the read mode fills first");
    s.run(s.dreq_change(), &mut silent());
    assert!(s.dreq() && s.status() & 0x380 == 0x280);
    assert_eq!(s.dma_read(0) & 0xFFFF, 0, "word 0's low half");
    assert_eq!(s.dma_read(0), 1, "word 1 as written");
}

#[test]
fn a_voice_reading_the_irq_address_raises_irq9_until_acknowledged() {
    let mut s = playing(5, 0x4000, 0x1000);
    s.write16(reg::IRQA as u32 * 2, 0x4008 / 8, 0);
    s.write16(reg::ATTR as u32 * 2, 0xC040, 0);
    let mut rose = false;
    for _ in 0..20 {
        let t = s.next_tick;
        rose |= s.run(t, &mut silent());
        if rose {
            break;
        }
    }
    assert!(rose && s.status() & 0x40 != 0, "the word at 4008h, samples 12-15, is read");
    assert_eq!(s.voices[5].counter >> 12, 13, "the sample whose word holds it");
    s.write16(reg::ATTR as u32 * 2, 0xC000, 0);
    assert_eq!(s.status() & 0x40, 0, "ATTR's bit 6 clear acknowledges");
}

#[test]
fn the_capture_buffers_take_the_cd_and_voices_1_and_3_each_sample() {
    let mut s = playing(1, 0x1000, 0x1000);
    for b in 0..4 {
        block(&mut s, 0x1000 + 16 * b, 0, 0, |_| 3);
    }
    s.voices[1].key_on(&s.ram);
    let mut n = 0i16;
    let mut cd = || {
        n += 1;
        [n, -n]
    };
    for _ in 0..5 {
        let t = s.next_tick;
        s.run(t, &mut cd);
    }
    let half = |a: usize| i16::from_le_bytes([s.ram[a], s.ram[a + 1]]);
    assert_eq!([half(0), half(2), half(8), half(0x400), half(0x408)], [1, 2, 5, -1, -5], "CD left and right, before the volume");
    assert_eq!(half(0x808), s.voices[1].out);
    assert_eq!(s.capture, 5);
}

#[test]
fn the_cd_input_is_mixed_by_its_volume_only_when_enabled() {
    let mut s = Spu::new();
    s.write16(reg::MVOL as u32 * 2, 0x3FFF, 0);
    s.write16(reg::MVOL as u32 * 2 + 2, 0x3FFF, 0);
    s.write16(reg::AVOL as u32 * 2, 0x4000, 0);
    s.write16(reg::AVOL as u32 * 2 + 2, 0x7FFF, 0);
    let mut cd = || [0x2000, 0x2000];
    s.run(TICK, &mut cd);
    s.write16(reg::ATTR as u32 * 2, attr::CD_IN, TICK);
    s.run(3 * TICK, &mut cd);
    let mut out = [0i16; 6];
    s.out.drain(&mut out, 3);
    assert_eq!(out[..2], [0, 0], "off");
    assert_eq!(out[4..], [(0x1000 * 0x7FFE >> 15) as i16, (0x1FFF * 0x7FFE >> 15) as i16], "and on, with the SPU itself off");
}

/// The noise generator's level, step 4 and shift 15: a new bit every sample, psx-spx's parity of bits 15, 12, 11, 10.
#[test]
fn the_noise_shifts_in_its_parity() {
    let mut s = Spu::new();
    s.regs[reg::ATTR] = 0x3C00;
    let mut levels = Vec::new();
    for _ in 0..6 {
        s.step_noise(s.regs[reg::ATTR]);
        levels.push(s.noise as u16);
    }
    assert_eq!(levels, [1, 3, 7, 15, 31, 63]);
    s.noise = 0x1C00u16 as i16;
    s.step_noise(0x3C00);
    assert_eq!(s.noise as u16, 0x3800, "bits 12, 11, 10 set: parity 0");
}

/// With vAPF2 zero the output is the buffer at mLAPF2-dAPF2 (psx-spx "Reverb Disable"): an impulse placed in the work
/// area comes out on the left when the buffer address reaches it, one step every two samples, through the filter.
#[test]
fn the_reverb_output_reads_the_work_area_as_the_buffer_address_moves() {
    let mut s = Spu::new();
    let mut regs = [0u16; 32];
    regs[28] = 0x10;
    let base = 0x7000 * 8;
    s.reverb.set_base(0x7000);
    s.ram[base + 0x80 + 2 * 100..][..2].copy_from_slice(&0x4000i16.to_le_bytes());
    let mut watch = Watch::default();
    let out: Vec<[i32; 2]> = (0..400).map(|_| s.reverb.tick(&mut s.ram, &mut watch, &regs, [0x7FFF, 0x7FFF], 0x7000, false, [0, 0])).collect();
    let peak = (0..400).max_by_key(|&t| out[t][0].abs()).unwrap_or(0);
    assert_eq!((peak, out[peak][0]), (200 + 19, 0x3FFF), "step 100's left cycle is sample 200, the filter's centre 19 later");
    assert_eq!(out[peak][1], 0, "the right side reads its own address, where nothing was placed");
    assert_eq!(s.reverb.address, base as u32 + 400, "two bytes a step");
}

/// The same side's reflection writes the input, taken down through the filter and by vLIN, where mLSAME points.
#[test]
fn the_reverb_writes_its_filtered_input_at_msame() {
    let mut s = Spu::new();
    let mut regs = [0u16; 32];
    regs[2] = 0x7FFF;
    regs[10] = 0x40;
    regs[30] = 0x7FFF;
    s.reverb.set_base(0x7000);
    let mut watch = Watch::default();
    for t in 0..24 {
        let x = if t == 0 { 0x4000 } else { 0 };
        s.reverb.tick(&mut s.ram, &mut watch, &regs, [0, 0], 0x7000, true, [x, x]);
    }
    let at = |step: usize| 0x7000 * 8 + 0x40 * 8 + 2 * step;
    let half = |a: usize| i16::from_le_bytes([s.ram[a], s.ram[a + 1]]) as i32;
    let tap = 0x2806 * 0x4000 >> 15;
    for step in [9, 10] {
        assert!((half(at(step)) - (tap * 0x7FFF >> 15) * 0x7FFF / 0x8000).abs() <= 2, "step {step}: {:X}", half(at(step)));
    }
    assert!(half(at(8)) < 0, "tap 16 is -B90h: {:X}", half(at(8)));
}

#[test]
fn the_registers_read_back_and_the_unused_tail_reads_ffff() {
    let mut s = Spu::new();
    s.write16(reg::MVOL as u32 * 2, 0x1234, 0);
    assert_eq!(s.read16(reg::MVOL as u32 * 2), 0x1234);
    assert_eq!(s.read16(0x280), 0xFFFF);
    s.write16(0x260, 0xBEEF, 0);
    assert_eq!(s.read16(0x260), 0xBEEF, "1F801E60h holds what is written");
}

/// D-36: a sample leaves through the envelope as it stood before the sample's step, so that the key on's own sample
/// is silent and the first step shows in the next, as DuckStation's capture buffers record it.
#[test]
fn the_envelope_steps_after_the_sample_it_scales() {
    let mut s = playing(1, 0x1000, 0x1000);
    block(&mut s, 0x1000, 0, 7, |_| 7);
    s.voices[1].adsr = [0x000F, 0x1FC0];
    let outs: Vec<i16> = (0..5).map(|_| {
        ticks(&mut s, 1);
        s.voices[1].out
    }).collect();
    let g = |taps: &[usize]| taps.iter().map(|&k| tables::GAUSS[k] * 0x7000).sum::<i32>() >> 15;
    let (g1, g2, g3) = (g(&[0x100, 0]), g(&[0x1FF, 0x100, 0]), g(&[0xFF, 0x1FF, 0x100, 0]));
    assert_eq!(outs, [0, (g1 * 14336 >> 15) as i16, (g2 * 28672 >> 15) as i16, (g3 * 0x7FFF >> 15) as i16, (g3 * 0x3FFF >> 15) as i16]);
}

/// D-37: a write of ATTR that changes the noise's shift or step starts the noise timer again, so that the new rate
/// holds from the next sample whatever the old one had left.
#[test]
fn a_new_noise_rate_holds_at_once() {
    let mut s = Spu::new();
    s.write16(reg::ATTR as u32 * 2, 0xC000, 0);
    for _ in 0..10 {
        let t = s.next_tick;
        s.run(t, &mut silent());
    }
    assert!(s.noise_timer > 0x1_0000, "shift 0 reloads 20000h");
    let before = s.noise;
    s.write16(reg::ATTR as u32 * 2, 0xFF00, s.next_tick - 1);
    let t = s.next_tick;
    s.run(t, &mut silent());
    assert_ne!(s.noise, before, "a new bit at once");
}
