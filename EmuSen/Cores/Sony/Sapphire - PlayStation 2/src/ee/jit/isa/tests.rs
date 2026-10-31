//! A block's folded cost against the issue model, and the lowering run over random blocks.

use emusen_mips_planner as planner;

use super::super::super::ops;
use super::super::super::timing::Timing;
use super::super::Options;
use super::{R5900Isa, compile, cost};

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
    let f = |rng: &mut Rng| rng.below(4);
    match rng.below(21) {
        // COP2's transfers, loads, stores and an operation of VU0's registers.
        20 => match rng.below(4) {
            0 => 0x12 << 26 | [1u32, 5][rng.below(2) as usize] << 21 | t << 16 | f(rng) << 11,
            1 => [0x36u32, 0x3E][rng.below(2) as usize] << 26 | s << 21 | f(rng) << 16 | (imm & 0x7F0),
            _ => 0x12 << 26 | 1 << 25 | 0xF << 21 | f(rng) << 16 | f(rng) << 11 | f(rng) << 6 | [0x28u32, 0x2C, 0x2A][rng.below(3) as usize],
        },
        // The calls (Sapphire_Speed.md §65): COP1's operations and moves, and multimedia instructions, the FPRs and the FPU's divider in the model.
        18 => match rng.below(4) {
            0 => 0x11 << 26 | [0u32, 4][rng.below(2) as usize] << 21 | t << 16 | f(rng) << 11,
            _ => 0x11 << 26 | 0x10 << 21 | f(rng) << 16 | f(rng) << 11 | f(rng) << 6 | [0u32, 1, 2, 3, 4, 5, 6, 7, 0x16, 0x18, 0x1A, 0x1C, 0x1E, 0x24, 0x28, 0x32][rng.below(16) as usize],
        },
        19 => 0x1C << 26 | s << 21 | t << 16 | d << 11 | [0x12u32, 0x16, 0x1A, 0x0E][rng.below(4) as usize] << 6 | [0x08u32, 0x28, 0x09, 0x29][rng.below(4) as usize],
        // E1's classes: HI and LO, the multipliers and dividers of both pipelines, SA, the traps, the unaligned forms and PREF.
        14 => s << 21 | t << 16 | d << 11 | [0x10, 0x11, 0x12, 0x13, 0x18, 0x19, 0x1A, 0x1B, 0x28, 0x29, 0x30, 0x31, 0x32, 0x33, 0x34, 0x36][rng.below(16) as usize],
        15 => 0x1C << 26 | s << 21 | t << 16 | d << 11 | [0x00, 0x01, 0x10, 0x11, 0x12, 0x13, 0x18, 0x19, 0x1A, 0x1B, 0x20, 0x21][rng.below(12) as usize],
        16 => 1 << 26 | s << 21 | [8u32, 9, 10, 11, 12, 14, 0x18, 0x19][rng.below(8) as usize] << 16 | imm,
        17 => [0x22u32, 0x26, 0x1A, 0x1B, 0x2A, 0x2E, 0x2C, 0x2D, 0x33][rng.below(9) as usize] << 26 | s << 21 | t << 16 | (imm & 0x7FF),
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

#[test]
fn a_blocks_cost_is_the_issue_model_from_rest() {
    let mut rng = Rng(0x5A5A_1234_8765_F00D);
    let mut tried = 0;
    for _ in 0..4000 {
        let words = random_block(&mut rng);
        let Some(words) = planner::shape(&R5900Isa, &words) else { continue };
        let Ok(plan) = planner::plan(&R5900Isa, &words, 0x1000) else { continue };
        let options = Options { single: rng.below(2) == 0, blocking: rng.below(2) == 0 };
        let c = cost(&plan, options);
        let mut t = Timing::at_rest(1 << 40);
        for (k, s) in plan.steps.iter().enumerate() {
            let mut i = ops::issue(ops::get(ops::index(s.word)), s.word);
            (i.single, i.blocking) = (options.single, options.blocking);
            let at = t.issue(i);
            if k + 1 < plan.steps.len() {
                assert_eq!(at + 1 - (1 << 40), u64::from(c.times[k]), "step {k}\n{plan}");
            }
        }
        assert!(c.taken >= c.times[plan.steps.len().saturating_sub(2).min(c.times.len() - 1)]);
        tried += 1;
    }
    assert!(tried > 2000, "{tried}");
}

#[test]
fn every_random_block_lowers() {
    let mut rng = Rng(0x1234_5678_9ABC_DEF1);
    let mut done = 0;
    for _ in 0..4000 {
        let words = random_block(&mut rng);
        let Some(words) = planner::shape(&R5900Isa, &words) else { continue };
        let options = Options { single: false, blocking: false };
        let code = compile(&words, 0x1000, options).expect("the block lowers");
        assert!(!code.hot.is_empty());
        done += 1;
    }
    assert!(done > 2000, "{done}");
}
