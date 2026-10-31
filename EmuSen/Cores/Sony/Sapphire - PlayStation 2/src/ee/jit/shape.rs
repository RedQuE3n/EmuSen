//! What a block of EE code may hold, and where it ends: the integer instructions, the multipliers and dividers of both
//! pipelines, SA's moves, SYNC, the traps and every branch and jump with its delay slot; the loads and stores of every
//! width and PREF; the multimedia instructions, COP1's and COP2's and CACHE, in place or as the interpreter's own operations.
//! What ends the block before it is the rest of COP0's and the system's, and the branches that test a unit (Sapphire_Speed.md §66).

use super::super::ops;

/// The most instructions a block holds: a 4 KiB page's, where a block ends if it has met no branch.
pub const MAX: usize = 1024;

/// What the lowering does with an instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// Compiled, and the next instruction follows it.
    Plain,
    /// A branch or jump: compiled with its delay slot, and the block ends after the slot.
    Branch,
    /// A load, a store or PREF: compiled like a plain instruction, its slow path the interpreter's operation.
    Memory,
    /// The interpreter's operation called from inside a block, which goes on after it.
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
    "pref", "lwc1", "swc1",
];

const BRANCH: &[&str] = &[
    "beq", "bne", "blez", "bgtz", "bltz", "bgez", "bltzal", "bgezal", "beql", "bnel", "blezl", "bgtzl", "bltzl", "bgezl", "bltzall",
    "bgezall", "j", "jal", "jr", "jalr", "bc1f", "bc1t", "bc1fl", "bc1tl",
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
    } else if MEMORY.contains(&n) || matches!(n, "lqc2" | "sqc2") {
        Class::Memory
    } else if ops::is_mmi(ops::index(word)) || ops::is_fpu_register(ops::index(word)) || ops::is_cop2_call(ops::index(word)) || n == "cache" {
        Class::Call
    } else {
        Class::Other
    }
}

/// Whether a class is compiled where it stands, the block going on after it.
fn straight(c: Class) -> bool {
    matches!(c, Class::Plain | Class::Memory | Class::Call)
}

/// The block's words from the run `words` holds: compiled instructions up to the first other, a branch only with its delay slot when the slot is compiled where it stands; none if the first instruction is not compiled.
pub fn shape(words: &[u32]) -> Option<Vec<u32>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < words.len() {
        match class(words[i]) {
            Class::Plain | Class::Memory | Class::Call => out.push(words[i]),
            Class::Branch => {
                if i + 1 < words.len() && straight(class(words[i + 1])) {
                    out.extend_from_slice(&words[i..i + 2]);
                }
                break;
            }
            Class::Other => break,
        }
        i += 1;
    }
    (!out.is_empty()).then_some(out)
}
