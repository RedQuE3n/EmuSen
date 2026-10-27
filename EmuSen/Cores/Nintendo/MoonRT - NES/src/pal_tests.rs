//! The PAL NES and the Dendy measured on synthetic programs, beside the NTSC NES as the control. See Moon_PAL.md §8.

use emusen_native::ffi::{StateMachine, Status};
use crate::apu::Apu;
use crate::machine::Machine;
use crate::memory::cartridge::Cartridge;
use crate::model::Model;
use crate::state::{STATUS_OTHER_MODEL, StateError};

const NMI_AT: u16 = 0xC100;
const IRQ_AT: u16 = 0xC200;

/// An NROM image: `reset` at $C000, `nmi` at $C100, `irq` at $C200; NES 2.0 with byte 12 `timing` when given, iNES 1.0
/// with byte 9's TV bit set otherwise, so that the unreliable flag is seen to be ignored.
fn image(timing: Option<u8>, reset: &[u8], nmi: &[u8], irq: &[u8]) -> Vec<u8> {
    let mut image = vec![0x4E, 0x45, 0x53, 0x1A, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    match timing {
        Some(t) => (image[7], image[12]) = (0x08, t),
        None => image[9] = 0x01,
    }
    let mut prg = vec![0xEAu8; 0x4000];
    prg[..reset.len()].copy_from_slice(reset);
    prg[0x100..0x100 + nmi.len()].copy_from_slice(nmi);
    prg[0x200..0x200 + irq.len()].copy_from_slice(irq);
    prg[0x3FFA..].copy_from_slice(&[0x00, 0xC1, 0x00, 0xC0, 0x00, 0xC2]);
    image.extend(prg);
    image.extend((0..0x2000).map(|i| i as u8));
    image
}

/// `code` then a jump to itself, after SEI, CLD and the stack.
fn program(code: &[u8]) -> Vec<u8> {
    let mut p = vec![0x78, 0xD8, 0xA2, 0xFF, 0x9A];
    p.extend_from_slice(code);
    let here = 0xC000 + p.len() as u16;
    p.extend([0x4C, here as u8, (here >> 8) as u8]);
    p
}

fn machine(model: Model, reset: &[u8], nmi: &[u8], irq: &[u8]) -> Machine {
    Machine::load_rom_as(&image(None, reset, nmi, irq), model).unwrap()
}

/// The CPU cycle and the PPU's line at each entry to the handler at `at`, over `entries` entries.
fn entries(m: &mut Machine, at: u16, entries: usize) -> Vec<(i64, i32)> {
    let mut seen = Vec::new();
    while seen.len() < entries {
        m.step();
        if m.cpu.pc == at {
            seen.push((m.cpu.cycles, m.bus.ppu.scanline));
        }
    }
    seen
}

const MODELS: [Model; 3] = [Model::Ntsc, Model::Pal, Model::Dendy];

// 3.2 dots a CPU cycle on the 2A07, exactly 3 on the 2A03 and the Dendy: over 200,000 cycles the PPU's clock keeps to the
// CPU's within one long cycle's dot.
#[test]
fn the_ppu_runs_its_dots_per_cpu_cycle() {
    for (model, num, den) in [(Model::Ntsc, 3, 1), (Model::Pal, 16, 5), (Model::Dendy, 3, 1)] {
        let mut m = machine(model, &program(&[]), &[0x40], &[0x40]);
        let (c0, d0) = (m.cpu.cycles, m.bus.ppu.ppu_clock);
        let mut worst = 0i64;
        while m.cpu.cycles - c0 < 200_000 {
            m.step();
            let drift = (m.bus.ppu.ppu_clock - d0) * den - (m.cpu.cycles - c0) * num;
            worst = worst.max(drift.abs());
        }
        assert!(worst <= num, "{model:?}: the PPU drifted {worst}/{den} dots from {num}/{den} a cycle");
        println!("{model:?}: at most {worst}/{den} dots from {num}/{den} a cycle over 200,000 cycles");
    }
}

// The 2C07 and 6538 draw 312 lines of 341 dots with no dot dropped, rendering or not; the 2C02 drops one on odd frames
// while rendering, the control that shows the count can see a dropped dot.
#[test]
fn a_frame_is_the_consoles_lines_and_only_the_2c02_drops_a_dot() {
    for model in MODELS {
        let mut m = machine(model, &program(&[0xA9, 0x1E, 0x8D, 0x01, 0x20]), &[0x40], &[0x40]);
        let dots = |m: &Machine| m.bus.ppu.ppu_clock - (m.bus.ppu.scanline as i64 * 341 + m.bus.ppu.cycle as i64);
        m.run_frame().unwrap();
        m.run_frame().unwrap();
        let (f0, d0) = (m.bus.ppu.frame_count, dots(&m));
        for _ in 0..20 {
            m.run_frame().unwrap();
        }
        let frames = m.bus.ppu.frame_count - f0;
        let per_frame = model.scanlines() as i64 * 341;
        let dropped = frames * per_frame - (dots(&m) - d0);
        assert_eq!(frames, 20);
        assert_eq!(dropped, if model == Model::Ntsc { 10 } else { 0 }, "{model:?}");
    }
}

// The vblank flag rises on the vblank line and falls on the pre-render line: 20 lines later on the NTSC NES and the
// Dendy, 70 on the PAL NES, which is 7,459 CPU cycles (nesdev's cycle reference chart).
#[test]
fn vblank_starts_on_the_consoles_line_and_lasts_its_lines() {
    for (model, line, lines) in [(Model::Ntsc, 241, 20), (Model::Pal, 241, 70), (Model::Dendy, 291, 20)] {
        let mut m = machine(model, &program(&[]), &[0x40], &[0x40]);
        let (mut rose, mut spans) = (None, Vec::new());
        let mut was = false;
        while spans.len() < 4 {
            m.step();
            let now = m.bus.ppu.vblank_flag();
            if now && !was {
                assert_eq!(m.bus.ppu.scanline, line, "{model:?}: the flag rose on line {}", m.bus.ppu.scanline);
                rose = Some(m.cpu.cycles);
            } else if was && !now {
                assert_eq!(m.bus.ppu.scanline, model.pre_render_scanline(), "{model:?}");
                spans.push(m.cpu.cycles - rose.unwrap());
            }
            was = now;
        }
        let dots_per_cycle = if model == Model::Pal { 3.2 } else { 3.0 };
        let expected = (lines * 341) as f64 / dots_per_cycle;
        assert!(spans.iter().all(|&s| (s as f64 - expected).abs() <= 4.0), "{model:?}: {spans:?} against {expected}");
        println!("{model:?}: vblank from line {line}, {spans:?} cycles against {expected:.1}");
    }
}

// The NMI enters on the vblank line, and two frames are 66,495 CPU cycles on the PAL NES (33,247.5 each), 70,928 on the
// Dendy and, rendering off, 59,561 on the NTSC NES.
#[test]
fn the_nmi_comes_once_a_frame_on_the_vblank_line() {
    for (model, two_frames) in [(Model::Ntsc, 59_561), (Model::Pal, 66_495), (Model::Dendy, 70_928)] {
        let mut m = machine(model, &program(&[0xA9, 0x80, 0x8D, 0x00, 0x20]), &[0x40], &[0x40]);
        let seen = entries(&mut m, NMI_AT, 21);
        assert!(seen.iter().all(|&(_, line)| line == model.vblank_scanline()), "{model:?}: {seen:?}");
        let span = seen[20].0 - seen[0].0;
        assert!((span - 10 * two_frames).abs() <= 7, "{model:?}: 20 frames in {span} cycles");
        println!("{model:?}: NMI on line {}, 20 frames in {span} cycles, {:?}", seen[0].1, seen.windows(2).take(6).map(|w| w[1].0 - w[0].0).collect::<Vec<_>>());
        for pair in seen.windows(3) {
            assert!((pair[2].0 - pair[0].0 - two_frames).abs() <= 7, "{model:?}: {pair:?}");
        }
    }
}

// The frame counter's IRQ, four steps and uninhibited, every 33,254 CPU cycles on the 2A07 and 29,830 on the 2A03 and the
// Dendy's CPU (nesdev's APU Frame Counter), seen by the CPU's IRQ handler.
#[test]
fn the_frame_irq_comes_at_the_consoles_period() {
    let code = [0xA9, 0x00, 0x8D, 0x17, 0x40, 0x58]; // LDA #0, STA $4017, CLI
    let ack = [0xAD, 0x15, 0x40, 0x40]; // LDA $4015, RTI
    for (model, period) in [(Model::Ntsc, 29_830), (Model::Pal, 33_254), (Model::Dendy, 29_830)] {
        let mut m = machine(model, &program(&code), &[0x40], &ack);
        let seen = entries(&mut m, IRQ_AT, 21);
        println!("{model:?}: 20 frame IRQs in {} cycles", seen[20].0 - seen[0].0);
        assert!((seen[20].0 - seen[0].0 - 20 * period).abs() <= 3, "{model:?}: {:?}", seen.windows(2).map(|w| w[1].0 - w[0].0).collect::<Vec<_>>());
    }
}

/// The CPU cycles between successive events `changed` sees in an APU stepped alone.
fn apu_periods(model: Model, writes: &[(u16, u8)], changed: impl Fn(&Apu, &Apu) -> bool) -> Vec<i64> {
    let mut apu = Apu::default();
    apu.set_model(model, 44100);
    apu.reset();
    for &(a, v) in writes {
        apu.write_register(a, v);
    }
    let mut times = Vec::new();
    for cycle in 0..200_000i64 {
        let before = apu.clone();
        apu.step(1);
        if changed(&before, &apu) {
            times.push(cycle);
        }
        if (cycle & 0x3FF) == 0 {
            apu.read_status();
        }
    }
    times.windows(2).map(|w| w[1] - w[0]).collect()
}

// Each noise period and DMC rate is the console's table entry in CPU cycles: the 2A07's for the PAL NES, the 2A03's for
// the NTSC NES and the Dendy (nesdev's APU Noise and APU DMC).
#[test]
fn the_noise_and_dmc_count_the_consoles_tables() {
    for model in MODELS {
        let pal = model == Model::Pal;
        for (index, ntsc_period, pal_period) in [(4u8, 64, 60), (13, 1016, 944)] {
            let periods = apu_periods(model, &[(0x400E, index)], |a, b| a.noise.shift != b.noise.shift);
            let want = if pal { pal_period } else { ntsc_period };
            assert!(periods.len() > 10 && periods.iter().all(|&p| p == want), "{model:?} noise {index}: {:?}", &periods[..4]);
        }
        for (index, ntsc_rate, pal_rate) in [(0u8, 428, 398), (15, 54, 50)] {
            let periods = apu_periods(model, &[(0x4010, index)], |a, b| a.dmc.bits_remaining != b.dmc.bits_remaining);
            let want = if pal { pal_rate } else { ntsc_rate };
            assert!(periods.len() > 10 && periods.iter().all(|&p| p == want), "{model:?} dmc {index}: {:?}", &periods[..4]);
        }
    }
}

// The four-step sequence's flag, set across its last three cycles, first rises 33,255 cycles after $4017 is written with 0
// on the 2A07 (blargg's pal_apu_tests, 07.irq_flag_timing) and 29,831 on the 2A03, then every 33,254 or 29,830.
#[test]
fn the_frame_counter_runs_the_consoles_sequence() {
    for (model, first, period) in [(Model::Ntsc, 29_831, 29_830), (Model::Pal, 33_255, 33_254)] {
        let mut apu = Apu::default();
        apu.set_model(model, 44100);
        apu.reset();
        for _ in 0..10 {
            apu.step(1);
        }
        apu.write_register(0x4017, 0x00);
        let mut rises = Vec::new();
        let mut was = false;
        for cycle in 1..=200_000i64 {
            apu.step(1);
            if apu.frame_irq_pending && !was {
                rises.push(cycle);
            }
            was = apu.frame_irq_pending;
            if was && apu.frame_step == 0 {
                apu.read_status();
                was = false;
            }
        }
        println!("{model:?}: the flag first rose {} cycles after the write, then every {:?}", rises[0], rises.windows(2).map(|w| w[1] - w[0]).take(3).collect::<Vec<_>>());
        assert!((rises[0] - first).abs() <= 1, "{model:?}: first at {}", rises[0]);
        assert!(rises.windows(2).all(|w| w[1] - w[0] == period), "{model:?}: {rises:?}");
    }
}

// 44,100 stereo samples a second of the console's own time: a frame of the PAL NES is 33,247.5 cycles of 1.662607 MHz,
// so 100 frames make 88,187.7 samples, against 73,377 for the NTSC NES's 100.
#[test]
fn the_audio_rate_follows_the_consoles_clock() {
    for model in MODELS {
        let mut m = machine(model, &program(&[]), &[0x40], &[0x40]);
        m.run_frame().unwrap();
        m.bus.apu.mixer.samples.clear();
        let c0 = m.cpu.cycles;
        let mut out = vec![0i16; 8192];
        let mut drained = 0;
        for _ in 0..100 {
            m.run_frame().unwrap();
            drained += m.bus.apu.drain(&mut out, 4096);
        }
        let seconds = (m.cpu.cycles - c0) as f64 / model.cpu_clock_hz();
        let pairs = drained as f64 / 2.0;
        assert!((pairs - seconds * 44100.0).abs() <= 1.5, "{model:?}: {pairs} pairs in {seconds} s");
        let per_frame = model.frame_rate().1 as f64 / model.frame_rate().0 as f64 * 44100.0;
        println!("{model:?}: {pairs} pairs in {seconds:.6} s, {} a frame", pairs / 100.0);
        assert!((pairs / 100.0 - per_frame).abs() < 0.05, "{model:?}: {} a frame against {per_frame}", pairs / 100.0);
    }
}

// The header chooses the console only in NES 2.0's byte 12; iNES 1.0's byte 9 is ignored, as nesdev advises.
#[test]
fn the_header_names_the_console_only_in_nes_2_0() {
    for (timing, model) in [(Some(0), Model::Ntsc), (Some(1), Model::Pal), (Some(2), Model::Ntsc), (Some(3), Model::Dendy), (None, Model::Ntsc)] {
        let img = image(timing, &program(&[]), &[0x40], &[0x40]);
        assert_eq!(Cartridge::header_model(&img), model, "{timing:?}");
        assert_eq!(*Machine::load_rom(&img).unwrap().model, model, "{timing:?}");
    }
}

// The 2C07's border: the top line and two pixels at each side black, $0E with no emphasis in the raw picture whatever
// PPUMASK says; the 2C02 draws the backdrop there.
#[test]
fn the_2c07_draws_its_black_border() {
    let code = [
        0x2C, 0x02, 0x20, 0x10, 0xFB, 0x2C, 0x02, 0x20, 0x10, 0xFB, // two vblanks
        0xA9, 0x3F, 0x8D, 0x06, 0x20, 0xA9, 0x00, 0x8D, 0x06, 0x20, 0xA9, 0x16, 0x8D, 0x07, 0x20, // backdrop $16
        0xA9, 0x00, 0x8D, 0x06, 0x20, 0x8D, 0x06, 0x20, // $2006 cleared
        0xA9, 0xE0, 0x8D, 0x01, 0x20, // all three emphasis bits, rendering off: the backdrop everywhere
    ];
    for model in MODELS {
        let mut m = machine(model, &program(&code), &[0x40], &[0x40]);
        m.bus.ppu.select_raw(true);
        for _ in 0..4 {
            m.bus.ppu.begin_raw(true);
            m.run_frame().unwrap();
            m.bus.ppu.end_raw();
        }
        let raw = m.bus.ppu.raw_picture().expect("the raw picture kept");
        let value = |x: usize, y: usize| u16::from_le_bytes([raw[(y * 256 + x) * 2], raw[(y * 256 + x) * 2 + 1]]);
        let rgb = |x: usize, y: usize| &m.bus.ppu.frame_rgba[(y * 256 + x) * 4..(y * 256 + x) * 4 + 3];
        let border = model != Model::Ntsc;
        for y in 0..240 {
            for x in 0..256 {
                let edge = border && (y == 0 || x < 2 || x >= 254);
                let want = if edge { 0x000E } else { 0x16 | 7 << 6 };
                assert_eq!(value(x, y), want, "{model:?} ({x}, {y})");
                assert_eq!(rgb(x, y) == [0, 0, 0], edge, "{model:?} ({x}, {y}): {:?}", rgb(x, y));
            }
        }
    }
}

// The 2A07 halts for DMA only on an instruction's first cycle, so a DMC fetch never lands on a read of $4016 and never
// clocks the pad an extra bit (nesdev's APU DMC). On the 2A03 it does, which is the control.
#[test]
fn only_the_2a03s_dmc_fetches_corrupt_a_pad_read() {
    let code = [
        0xA9, 0x4F, 0x8D, 0x10, 0x40, // DMC: loop, the fastest rate
        0xA9, 0x00, 0x8D, 0x12, 0x40, 0xA9, 0xFF, 0x8D, 0x13, 0x40, // from $C000, 4,081 bytes
        0xA9, 0x10, 0x8D, 0x15, 0x40, // start it
        // $C019: strobe the pad, read eight bits into $00, count a pass in $04-$05 and a wrong byte in $01-$02.
        0xA9, 0x01, 0x8D, 0x16, 0x40, 0xA9, 0x00, 0x8D, 0x16, 0x40, 0xA2, 0x08,
        0xAD, 0x16, 0x40, 0x4A, 0x26, 0x00, 0xCA, 0xD0, 0xF7, // LDA $4016, LSR, ROL $00, DEX, BNE
        0xE6, 0x04, 0xD0, 0x02, 0xE6, 0x05, // INC $04, BNE +2, INC $05
        0xA5, 0x00, 0xC9, 0x80, 0xF0, 0x06, 0xE6, 0x01, 0xD0, 0x02, 0xE6, 0x02, // LDA $00, CMP #$80, BEQ +6, INC $01, BNE +2, INC $02
        0xA5, 0x06, 0x4C, 0x19, 0xC0, // LDA $06, JMP: a pass of 163 cycles, prime, so a fetch every 432 or 400 lands on each of its cycles in turn
    ];
    for (model, corrupts) in [(Model::Ntsc, true), (Model::Pal, false)] {
        let mut m = machine(model, &program(&code), &[0x40], &[0x40]);
        m.bus.controller1.state = 0x01;
        for _ in 0..60 {
            m.run_frame().unwrap();
        }
        let (passes, wrong) = (u16::from_le_bytes([m.bus.ram[4], m.bus.ram[5]]), u16::from_le_bytes([m.bus.ram[1], m.bus.ram[2]]));
        println!("{model:?}: {wrong} wrong bytes in {passes} reads of the pad");
        assert!(passes > 10_000, "{model:?}: the loop ran {passes} times");
        assert_eq!(wrong != 0, corrupts, "{model:?}: {wrong} wrong bytes");
    }
}

fn save(m: &Machine) -> Vec<u8> {
    let mut out = vec![0u8; m.state_size()];
    m.save_state(&mut out).unwrap();
    out
}

// A state records its console: the NTSC NES's is version 6 byte for byte with no model in it, the others version 7 with
// the model and the long cycle's phase; each loads into its own console and is refused, changing nothing, by another.
#[test]
fn a_state_records_its_console_and_loads_only_into_it() {
    let img = image(None, &program(&[0xA9, 0x80, 0x8D, 0x00, 0x20]), &[0x40], &[0x40]);
    let mut states = Vec::new();
    for model in MODELS {
        let mut m = Machine::load_rom_as(&img, model).unwrap();
        for _ in 0..3 {
            m.run_frame().unwrap();
        }
        m.step();
        let state = save(&m);
        let version = i32::from_le_bytes(state[4..8].try_into().unwrap());
        let layout = m.layout();
        assert_eq!(version, if model == Model::Ntsc { 6 } else { 7 }, "{model:?}");
        assert_eq!(layout.lines().any(|l| l.ends_with(" Model")), model != Model::Ntsc, "{model:?}");
        let mut back = Machine::load_rom_as(&img, model).unwrap();
        back.load_state(&state).unwrap();
        assert_eq!(save(&back), state, "{model:?}");
        assert_eq!(*back.bus.dot_phase, *m.bus.dot_phase);
        back.run_frame().unwrap();
        m.run_frame().unwrap();
        assert_eq!(save(&back), save(&m), "{model:?}: the same frame after the load");
        states.push((model, state));
    }
    for (saved, state) in &states {
        for running in MODELS.into_iter().filter(|r| r != saved) {
            let mut m = Machine::load_rom_as(&img, running).unwrap();
            let before = save(&m);
            let refused = m.load_state(state);
            assert_eq!(refused, Err(StateError::OtherModel { saved: saved.code(), running: running.code() }));
            assert_eq!(refused.map_err(|e| e.status()), Err(STATUS_OTHER_MODEL));
            assert_eq!(save(&m), before, "{saved:?} into {running:?} changed the machine");
        }
    }
}
