//! What a block of EE code may hold, and where it ends: the integer instructions, the multipliers and dividers of both
//! pipelines, SA's moves, SYNC, the traps and every branch and jump with its delay slot (S36); the loads and stores of
//! every width and PREF (S37); in the tailored tier's blocks also the multimedia instructions, COP1's and COP2's, run as the interpreter's
//! own operations (Sapphire_Speed.md §65). Everything else ends the block before it and is the interpreter's
//! (Sapphire_Native.md §44.3, Sapphire_Recompiler.md §48.2).

use super::super::ops;

/// The most instructions a block holds.
pub const MAX: usize = 64;

/// What the lowering does with an instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// Compiled, and the next instruction follows it.
    Plain,
    /// A branch or jump: compiled with its delay slot, and the block ends after the slot.
    Branch,
    /// A load, a store or PREF (S37): compiled like a plain instruction, leaving the block before it at run time where
    /// its access is not one a block makes.
    Memory,
    /// The interpreter's operation called from inside a tailored tier's block, which goes on after it (Sapphire_Speed.md §65).
    Call,
    /// Not compiled: the block ends before it.
    Other,
}

const PLAIN: &[&str] = &[
    "add", "addu", "sub", "subu", "addi", "addiu", "dadd", "daddu", "dsub", "dsubu", "daddi", "daddiu", "and", "or", "xor", "nor", "andi",
    "ori", "xori", "lui", "slt", "sltu", "slti", "sltiu", "movz", "movn", "sll", "srl", "sra", "sllv", "srlv", "srav", "dsll", "dsrl",
    "dsra", "dsll32", "dsrl32", "dsra32", "dsllv", "dsrlv", "dsrav", "mfhi", "mflo", "mthi", "mtlo", "mfhi1", "mflo1", "mthi1", "mtlo1",
    "mult", "multu", "mult1", "multu1", "madd", "maddu", "madd1", "maddu1", "div", "divu", "div1", "divu1", "mfsa", "mtsa", "mtsab",
    "mtsah", "tge", "tgeu", "tlt", "tltu", "teq", "tne", "tgei", "tgeiu", "tlti", "tltiu", "teqi", "tnei", "sync", "sync.p",
];

const MEMORY: &[&str] = &[
    "lb", "lbu", "lh", "lhu", "lw", "lwu", "ld", "lq", "lwl", "lwr", "ldl", "ldr", "sb", "sh", "sw", "sd", "sq", "swl", "swr", "sdl", "sdr",
    "pref",
];

const BRANCH: &[&str] = &[
    "beq", "bne", "blez", "bgtz", "bltz", "bgez", "bltzal", "bgezal", "beql", "bnel", "blezl", "bgtzl", "bltzl", "bgezl", "bltzall",
    "bgezall", "j", "jal", "jr", "jalr",
];

/// The decoder's name for a word, as the interpreter's table holds it.
pub fn name(word: u32) -> &'static str {
    ops::get(ops::index(word)).name
}

/// A word's class through a memo of the words seen on this thread: the decode and the name's lookup are a pure function
/// of the word, and the dispatcher shapes a block at every entry that waits for a line (Sapphire_Recompiler.md §48.4).
pub fn class(word: u32) -> Class {
    const SLOTS: usize = 4096;
    thread_local! {
        static MEMO: std::cell::RefCell<Box<[(u32, Class)]>> = std::cell::RefCell::new(vec![(0, decode(0)); SLOTS].into_boxed_slice());
    }
    MEMO.with(|m| {
        let mut m = m.borrow_mut();
        let slot = &mut m[(word ^ word >> 12 ^ word >> 24) as usize % SLOTS];
        if slot.0 != word {
            *slot = (word, decode(word));
        }
        slot.1
    })
}

fn decode(word: u32) -> Class {
    let n = name(word);
    if PLAIN.contains(&n) {
        Class::Plain
    } else if BRANCH.contains(&n) {
        Class::Branch
    } else if MEMORY.contains(&n) {
        Class::Memory
    } else if ops::is_mmi(ops::index(word)) || ops::is_fpu_register(ops::index(word)) || ops::is_cop2_in_blocks(ops::index(word), word) {
        Class::Call
    } else {
        Class::Other
    }
}

/// Whether a block's word reaches memory: a load, a store, PREF, LQC2 or SQC2.
pub fn touches_memory(word: u32) -> bool {
    class(word) == Class::Memory || matches!(name(word), "lqc2" | "sqc2")
}

/// Every mnemonic a block compiles, for the tests that cover each one.
pub fn compiled() -> impl Iterator<Item = &'static str> {
    PLAIN.iter().chain(BRANCH).chain(MEMORY).copied()
}

/// The class a block gives a word: a load or store is another instruction where `memory` is off (S36's blocks), a call where `calls` is.
pub fn class_in(word: u32, memory: bool, calls: bool) -> Class {
    match class(word) {
        Class::Memory if !memory => Class::Other,
        Class::Call if !calls => Class::Other,
        c => c,
    }
}

/// Whether a class is compiled where it stands, the block going on after it.
fn straight(c: Class) -> bool {
    matches!(c, Class::Plain | Class::Memory | Class::Call)
}

/// The block's words from the run `words` holds (consecutive instructions from the block's start, as far as they are
/// known): compiled instructions up to the first other, a branch only with its delay slot when the slot is a plain or
/// memory instruction, at most `MAX`; none if the first instruction is not compiled. With it, whether the run ended
/// before the block did, so that more words could have made it longer. `memory` and `calls` as in `class_in`.
pub fn shape(words: &[u32], memory: bool, calls: bool) -> (Option<Vec<u32>>, bool) {
    let mut out = Vec::new();
    let mut i = 0;
    let mut short = true;
    while i < words.len() && out.len() < MAX {
        match class_in(words[i], memory, calls) {
            Class::Plain | Class::Memory | Class::Call => out.push(words[i]),
            Class::Branch => {
                short = i + 1 >= words.len();
                if !short && out.len() + 2 <= MAX && straight(class_in(words[i + 1], memory, calls)) {
                    out.extend_from_slice(&words[i..i + 2]);
                }
                return ((!out.is_empty()).then_some(out), short);
            }
            Class::Other => {
                short = false;
                break;
            }
        }
        i += 1;
    }
    if out.len() == MAX {
        short = false;
    }
    ((!out.is_empty()).then_some(out), short)
}
