//! The static form's cycles proven against the issue model (EmuSen_Recompiler.md §14.3), and the lowering run over random blocks of the slice.

use emusen_mips_planner as planner;

use super::super::super::ops;
use super::super::super::timing::Timing;
use super::super::Options;
use super::{R5900Isa, compile, static_times};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u32 {
        (self.next() % n) as u32
    }
}

fn r(rng: &mut Rng) -> u32 {
    // A few registers, so that instructions depend on each other.
    [0, 2, 3, 4, 5, 8, 9, 29, 31][rng.below(9) as usize]
}

/// A word of E0's slice: ALU, shifts, loads and stores of every width.
fn slice_word(rng: &mut Rng) -> u32 {
    let (s, t, d) = (r(rng), r(rng), r(rng));
    let imm = rng.next() as u16 as u32;
    let sa = rng.below(32);
    match rng.below(14) {
        0 => s << 21 | t << 16 | d << 11 | [0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x0A, 0x0B][rng.below(16) as usize],
        1 => t << 16 | d << 11 | sa << 6 | [0x00, 0x02, 0x03, 0x38, 0x3A, 0x3B, 0x3C, 0x3E, 0x3F][rng.below(9) as usize],
        2 => s << 21 | t << 16 | d << 11 | [0x04, 0x06, 0x07, 0x14, 0x16, 0x17][rng.below(6) as usize],
        3 => [8u32, 9, 10, 11, 12, 13, 14, 24, 25][rng.below(9) as usize] << 26 | s << 21 | t << 16 | imm,
        4 => 0xF << 26 | t << 16 | imm,
        5..=9 => [0x20u32, 0x24, 0x21, 0x25, 0x23, 0x27, 0x37, 0x1E][rng.below(8) as usize] << 26 | s << 21 | t << 16 | (imm & 0x7F0),
        10..=12 => [0x28u32, 0x29, 0x2B, 0x3F, 0x1F][rng.below(5) as usize] << 26 | s << 21 | t << 16 | (imm & 0x7F0),
        _ => 0xF,
    }
}

fn branch_word(rng: &mut Rng) -> u32 {
    let (s, t) = (r(rng), r(rng));
    let off = rng.below(64) as u16 as u32;
    match rng.below(8) {
        0 => 4 << 26 | s << 21 | t << 16 | off,
        1 => 5 << 26 | s << 21 | t << 16 | off,
        2 => 0x14 << 26 | s << 21 | t << 16 | off,
        3 => 1 << 26 | s << 21 | [0, 1, 0x10, 0x11, 2, 3, 0x12, 0x13][rng.below(8) as usize] << 16 | off,
        4 => 2 << 26 | rng.below(1 << 20),
        5 => 3 << 26 | rng.below(1 << 20),
        6 => s << 21 | 8,
        _ => s << 21 | 31 << 11 | 9,
    }
}

fn random_block(rng: &mut Rng) -> Vec<u32> {
    let n = 1 + rng.below(20) as usize;
    let mut w: Vec<u32> = (0..n).map(|_| slice_word(rng)).collect();
    if rng.below(3) != 0 {
        w.push(branch_word(rng));
        w.push(slice_word(rng));
    }
    w
}

/// An entry model: a random prefix of instructions issued from rest, multipliers and dividers among them.
fn random_model(rng: &mut Rng) -> Timing {
    let mut t = Timing::default();
    let prefix = [0x0000_0018u32, 0x0000_001A, 0x0000_0012, 0x7000_0000, 0x0000_0021, 0x8C00_0000];
    for _ in 0..1 + rng.below(6) {
        let w = if rng.below(2) == 0 { prefix[rng.below(6) as usize] | r(rng) << 21 | r(rng) << 16 | r(rng) << 11 } else { slice_word(rng) };
        let mut i = ops::issue(ops::get(ops::index(w)), w);
        i.taken = rng.below(4) == 0;
        t.issue(i);
    }
    t
}

#[test]
fn the_static_form_is_the_issue_model_from_every_entry_at_rest() {
    let mut rng = Rng(0x5A5A_1234_8765_F00D);
    let (mut tried, mut rested) = (0, 0);
    for _ in 0..4000 {
        let words = random_block(&mut rng);
        let Some(words) = planner::shape(&R5900Isa, &words) else { continue };
        let Ok(plan) = planner::plan(&R5900Isa, &words, 0x1000) else { continue };
        for single in [false, true] {
            let options = Options { single, verify: false, fixed_cost: false, spill: false, memory: true, blocking: rng.below(2) == 0 };
            let Some(st) = static_times(&plan, options) else { panic!("a block of the slice has a static form") };
            for _ in 0..8 {
                let entry = random_model(&mut rng);
                tried += 1;
                if !entry.rests_for(st.exposed) {
                    continue;
                }
                rested += 1;
                for taken in [false, true] {
                    let mut t = entry.clone();
                    for (k, s) in plan.steps.iter().enumerate() {
                        let mut i = ops::issue(ops::get(ops::index(s.word)), s.word);
                        i.taken = taken && s.info.class == planner::Class::Branch;
                        (i.single, i.blocking) = (options.single, options.blocking);
                        let at = t.issue(i);
                        assert_eq!(at + 1, entry.last() + u64::from(st.times[k]), "step {k}'s cycles\n{plan}");
                        let mut folded = entry.clone();
                        folded.apply(entry.last(), &st.snaps[k + 1][usize::from(taken)]);
                        folded.instructions += k as u64 + 1;
                        assert_eq!(folded, t, "the model after {} steps, taken {taken}\n{plan}", k + 1);
                    }
                }
            }
        }
    }
    assert!(rested * 4 > tried, "most random entries are at rest: {rested} of {tried}");
}

#[test]
fn every_random_block_of_the_slice_lowers_in_both_terms() {
    let mut rng = Rng(0x0123_4567_89AB_CDEF);
    let options = Options { single: false, verify: false, fixed_cost: false, spill: false, memory: true, blocking: false };
    let mut n = 0;
    for _ in 0..3000 {
        let words = random_block(&mut rng);
        let Some(words) = planner::shape(&R5900Isa, &words) else { continue };
        if words.len() < 2 {
            continue;
        }
        let (exact, _) = compile(&words, 0x8000_1000, options, false).expect("a block of the slice lowers");
        let (proto, _) = compile(&words, 0x8000_1000, options, true).expect("on the prototype's terms too");
        assert!(proto.hot.len() < exact.hot.len(), "the prototype's terms drop the entry's test");
        assert_eq!(exact.hot_by_purpose.iter().sum::<u32>() as usize, exact.hot.len());
        n += 1;
    }
    assert!(n > 2000);
}
