//! Stage S26's gate (Sapphire_Native.md §32.7): the project's IOP programs on the SPU2, run on `blackmoon-r3000` with
//! Sapphire's IOP bus inside the board, their records graded by the documents and the rulings of D-260 on, and their
//! sound, copied from the output areas into the record, by the record `tests/s26/gate.rs` makes; and the rules no
//! program can show, checked on the unit directly.

mod s21;
mod s26;

use s26::gate::{self, Gate};
use sapphire::board::Board;
use sapphire::clock::{self, IOP};
use sapphire::spu2::{self, Spu2};

/// Runs an IOP program in a board until its done word; the board, the expected record and every stereo sample the
/// SPU2 made, interleaved.
pub fn run(p: s21::Program) -> (Board, Vec<Option<u32>>, Vec<i16>) {
    let name = p.name;
    let (source, expected) = p.finish();
    let words = s21::assemble(&source);
    let mut b = Board::new(clock::NTSC);
    b.iop.bus.load(s21::ORIGIN, &words);
    b.iop.bus.load(0x80, &s21::vector());
    b.iop.enter(s21::K1 | s21::ORIGIN);
    let mut spr = vec![0u8; 16 << 10];
    let mut sound = Vec::new();
    let mut chunk = vec![0i16; 1 << 16];
    let limit = IOP * 4_000_000_000;
    while b.iop.bus.word(s21::DONE_AT) != s21::DONE {
        assert!(b.now < limit, "{name}: no done word by EE cycle {limit}, IOP pc {:08x}", b.iop.cpu.regs.pc);
        let t = b.now + IOP * 20_000;
        b.run_to(t, &mut spr);
        let n = b.iop.bus.spu2.out.drain(&mut chunk, usize::MAX);
        sound.extend_from_slice(&chunk[..n]);
    }
    (b, expected, sound)
}

fn record(b: &Board, n: usize) -> Vec<u32> {
    (0..n).map(|i| b.iop.bus.word(s21::RECORD + 4 * i as u32)).collect()
}

/// Runs a gate program and grades its record's words and its checks.
fn grade(g: Gate) -> (Board, Vec<i16>) {
    let name = g.program.name;
    let (program, checks) = g.finish();
    let (b, expected, sound) = run(program);
    let got = record(&b, expected.len());
    let mut wrong: Vec<String> = expected
        .iter()
        .enumerate()
        .filter_map(|(i, w)| w.filter(|&w| w != got[i]).map(|w| format!("word {i}: {:08x}, expected {w:08x}", got[i])))
        .collect();
    wrong.extend(gate::grade(&checks, &got));
    assert!(wrong.is_empty(), "{name}: {} of {} words and {} checks wrong: {:#?}", wrong.len(), expected.len(), checks.len(), &wrong[..wrong.len().min(12)]);
    (b, sound)
}

#[test]
fn the_voices_decode_interpolate_and_modulate_as_the_shared_voice_does() {
    grade(gate::voices());
}

#[test]
fn each_envelope_phase_and_mode_runs_as_the_shared_voice_does() {
    grade(gate::envelope());
}

#[test]
fn the_noise_runs_through_its_whole_cycle_at_each_clock() {
    grade(gate::noise());
}

#[test]
fn the_masks_mmix_and_master_volume_mix_as_recorded_in_the_output_areas() {
    grade(gate::mixing());
}

#[test]
fn the_reverb_settles_in_its_area_and_writes_nothing_when_off() {
    grade(gate::reverb());
}

#[test]
fn autodma_fills_memin_as_ps2tek_lays_it_out_and_feeds_the_sound_data_input() {
    grade(gate::adma());
}

#[test]
fn the_irq_address_is_met_by_voices_and_transfers_on_either_core() {
    grade(gate::irq());
}

#[test]
fn the_registers_read_back_as_the_rulings_say() {
    grade(gate::registers());
}

/// The output: 48,000 samples a second exactly, one each 768 IOP cycles, whatever runs.
#[test]
fn the_output_is_48000_hz_exactly() {
    let mut s = Spu2::new();
    s.run(clock::EE_HZ / IOP);
    assert_eq!(s.out.len(), 2 * 48_000);
    assert_eq!(spu2::CYCLES_PER_SAMPLE * 48_000 * IOP, clock::EE_HZ);
}

/// Core 0's output is core 1's external input: through core 1's AVOL and MMIX's external dry gates into the output,
/// which the output areas cannot show.
#[test]
fn core_0s_output_reaches_the_output_through_core_1() {
    let mut s = Spu2::new();
    let block = s26::constant_block();
    s.write16(0x1A8, 0);
    s.write16(0x1AA, 0x5000);
    for h in block {
        s.write16(0x1AC, h);
    }
    for (i, v) in [0x3FFFu16, 0x3FFF, 0x1000, 0x000F, 0x1FC0].iter().enumerate() {
        s.write16(2 * i as u32, *v);
    }
    s.write16(0x1C2, 0x5000);
    s.write16(0x188, 1);
    s.write16(0x190, 1);
    s.write16(0x198, 0xC00);
    for c in 0..2u32 {
        s.write16(0x760 + 0x28 * c, 0x3FFF);
        s.write16(0x762 + 0x28 * c, 0x3FFF);
    }
    s.write16(0x1A0, 1);
    let mut out = vec![0i16; 64];
    let mut step = |s: &mut Spu2, n: u64| {
        let t = s.next_tick + (n - 1) * spu2::CYCLES_PER_SAMPLE;
        s.run(t);
        let k = s.out.drain(&mut out, usize::MAX);
        out[k - 2..k].to_vec()
    };
    step(&mut s, 32);
    assert_eq!(step(&mut s, 1), [0, 0], "core 1's MMIX closes its external input");
    s.write16(0x400 + 0x198, 0x8);
    s.write16(0x768 + 0x28, 0x3FFF);
    let first = s.cores[0].out;
    let lr = step(&mut s, 2);
    assert_ne!(first[0], 0);
    let through = |x: i32| (x * 0x7FFE >> 15) * 0x7FFE >> 15;
    assert_eq!(lr, [through(s.cores[0].out[0]) as i16, 0], "the left external dry gate alone, AVOLL full");
}
