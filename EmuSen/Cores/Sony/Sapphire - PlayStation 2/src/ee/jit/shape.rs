//! What a block of EE code may hold at S36, and where it ends: the integer instructions, the multipliers and dividers of
//! both pipelines, SA's moves, SYNC, the traps and every branch and jump with its delay slot. Everything else ends the
//! block before it and is the interpreter's (Sapphire_Native.md §44.3).

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
    /// Not compiled at S36: the block ends before it.
    Other,
}

const PLAIN: &[&str] = &[
    "add", "addu", "sub", "subu", "addi", "addiu", "dadd", "daddu", "dsub", "dsubu", "daddi", "daddiu", "and", "or", "xor", "nor", "andi",
    "ori", "xori", "lui", "slt", "sltu", "slti", "sltiu", "movz", "movn", "sll", "srl", "sra", "sllv", "srlv", "srav", "dsll", "dsrl",
    "dsra", "dsll32", "dsrl32", "dsra32", "dsllv", "dsrlv", "dsrav", "mfhi", "mflo", "mthi", "mtlo", "mfhi1", "mflo1", "mthi1", "mtlo1",
    "mult", "multu", "mult1", "multu1", "madd", "maddu", "madd1", "maddu1", "div", "divu", "div1", "divu1", "mfsa", "mtsa", "mtsab",
    "mtsah", "tge", "tgeu", "tlt", "tltu", "teq", "tne", "tgei", "tgeiu", "tlti", "tltiu", "teqi", "tnei", "sync", "sync.p",
];

const BRANCH: &[&str] = &[
    "beq", "bne", "blez", "bgtz", "bltz", "bgez", "bltzal", "bgezal", "beql", "bnel", "blezl", "bgtzl", "bltzl", "bgezl", "bltzall",
    "bgezall", "j", "jal", "jr", "jalr",
];

/// The decoder's name for a word, as the interpreter's table holds it.
pub fn name(word: u32) -> &'static str {
    ops::get(ops::index(word)).name
}

pub fn class(word: u32) -> Class {
    let n = name(word);
    if PLAIN.contains(&n) {
        Class::Plain
    } else if BRANCH.contains(&n) {
        Class::Branch
    } else {
        Class::Other
    }
}

/// Every mnemonic a block compiles, for the tests that cover each one.
pub fn compiled() -> impl Iterator<Item = &'static str> {
    PLAIN.iter().chain(BRANCH).copied()
}

/// The block's words from the run `words` holds (consecutive instructions from the block's start, as far as they are
/// known): compiled instructions up to the first other, a branch only with its delay slot when the slot is a plain
/// instruction, at most `MAX`; none if the first instruction is not compiled. With it, whether the run ended before the
/// block did, so that more words could have made it longer.
pub fn shape(words: &[u32]) -> (Option<Vec<u32>>, bool) {
    let mut out = Vec::new();
    let mut i = 0;
    let mut short = true;
    while i < words.len() && out.len() < MAX {
        match class(words[i]) {
            Class::Plain => out.push(words[i]),
            Class::Branch => {
                short = i + 1 >= words.len();
                if !short && out.len() + 2 <= MAX && class(words[i + 1]) == Class::Plain {
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
