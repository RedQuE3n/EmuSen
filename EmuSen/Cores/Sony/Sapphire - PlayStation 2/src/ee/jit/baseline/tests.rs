//! The static form's cycles proven against the issue model (EmuSen_Recompiler.md §14.3), and the lowering run over random blocks of the slice.

use emusen_mips_planner as planner;

use super::super::super::ops;
use super::super::super::timing::Timing;
use super::super::Options;
use super::{R5900Isa, compile, prefix, static_times};

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
    match rng.below(20) {
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
    assert!(rested * 8 > tried, "many random entries are at rest: {rested} of {tried}");
}

/// A block's prefix table in 8-aligned memory, as its data holds it.
fn table_buffer(st: &super::Static, options: Options) -> Vec<u64> {
    let bytes = prefix::table_bytes(st, options.single, options.blocking);
    let mut buf = vec![0u64; bytes.len().div_ceil(8)];
    for (k, b) in bytes.iter().enumerate() {
        buf[k / 8] |= u64::from(*b) << (8 * (k % 8));
    }
    buf
}

#[test]
fn the_prefix_then_the_static_form_is_the_issue_model_from_every_entry() {
    let mut rng = Rng(0x0BAD_5EED_1357_9ACE);
    let (mut moved, mut converged, mut late, mut never) = (0, 0, 0, 0);
    for _ in 0..3000 {
        let words = random_block(&mut rng);
        let Some(words) = planner::shape(&R5900Isa, &words) else { continue };
        let Ok(plan) = planner::plan(&R5900Isa, &words, 0x1000) else { continue };
        let n = plan.steps.len();
        for single in [false, true] {
            let options = Options { single, verify: false, fixed_cost: false, spill: false, memory: true, blocking: rng.below(2) == 0 };
            let st = static_times(&plan, options).expect("a block of the slice has a static form");
            let buf = table_buffer(&st, options);
            // SAFETY: the buffer is 8-aligned and holds the table's bytes for the loop's length.
            let t = unsafe { prefix::Table::at(buf.as_ptr().cast()) };
            for _ in 0..8 {
                let entry = random_model(&mut rng);
                for taken in [false, true] {
                    let mut direct = entry.clone();
                    let (mut times, mut models, mut unpaired) = (Vec::new(), vec![entry.clone()], vec![0u8]);
                    for s in &plan.steps {
                        let mut i = ops::issue(ops::get(ops::index(s.word)), s.word);
                        i.taken = taken && s.info.class == planner::Class::Branch;
                        (i.single, i.blocking) = (options.single, options.blocking);
                        times.push(direct.issue(i) + 1);
                        unpaired.push(unpaired.last().unwrap() + u8::from(!direct.paired()));
                        models.push(direct.clone());
                    }
                    for j in 1..=n {
                        let mut d = entry.clone();
                        let p = prefix::run(&t, &mut d, j, taken);
                        for (k, &c) in times.iter().enumerate().take(j) {
                            let got = if k < p.at { p.times[k] } else { p.base.expect("rebased past the prefix") + u64::from(st.times[k]) };
                            assert_eq!(got, c, "step {k}'s cycles at an exit after {j}, prefix {}\n{plan}", p.at);
                        }
                        let mut core = entry.clone();
                        let (u, paired) = prefix::model(&mut core, &d, &p, &t, &st.snaps[j][usize::from(taken)], j);
                        core.instructions += j as u64;
                        assert_eq!(core, models[j], "the model at an exit after {j}, taken {taken}, prefix {}\n{plan}", p.at);
                        assert_eq!((u, paired), (unpaired[j], models[j].paired()), "the unpaired cycles and the pairing after {j}");
                        if j == n && !taken && !entry.rests_for(st.exposed) {
                            moved += 1;
                            match p.base {
                                Some(_) if p.at > 1 => late += 1,
                                Some(_) => converged += 1,
                                None => never += 1,
                            }
                        }
                    }
                }
            }
        }
    }
    println!("random entries not at rest {moved}: converged at 1 {converged}, later {late}, never {never}");
    assert!(converged > 0 && late > 0 && never > 0, "every way the prefix ends was tried: {moved} entries not at rest, {converged} at 1, {late} later, {never} never");
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
        let exact = compile(&words, 0x8000_1000, options, false).expect("a block of the slice lowers");
        let proto = compile(&words, 0x8000_1000, options, true).expect("on the prototype's terms too");
        assert!(proto.hot.len() < exact.hot.len(), "the prototype's terms drop the entry's test");
        assert_eq!(exact.hot_by_purpose.iter().sum::<u32>() as usize, exact.hot.len());
        n += 1;
    }
    assert!(n > 2000);
}
