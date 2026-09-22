//! A block's shape: where it ends and whether it is refused, decided from its words. C#'s `BlockShape`; see Mars_Recompiler.md §2.1 and Mars_Native.md §5.8.

use crate::cpu::interp::{DIVIDE_DOUBLE_STALL, DIVIDE_STALL, MULTIPLY_DOUBLE_STALL, MULTIPLY_STALL};

/// The most instructions a block holds, never between a branch and its slot.
pub const MAX_WORDS: u32 = 64;

/// What an instruction does to the shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// Runs on to the next word.
    Plain,
    /// A branch or jump: its slot follows, and the block ends after it.
    Branch,
    /// Changes the mode, the timer, the TLB or the interrupt check's inputs, or always raises: the block ends after it.
    Ender,
}

/// How `Cpu::execute` treats a word in kernel mode, for the shape's purposes.
pub fn class(i: u32) -> Class {
    use Class::*;
    let rt = (i >> 16) & 0x1F;
    let rs = (i >> 21) & 0x1F;
    match i >> 26 {
        0x00 => match i & 0x3F {
            0x08 | 0x09 => Branch,
            0x0C | 0x0D => Ender,
            0x00 | 0x02..=0x04 | 0x06 | 0x07 | 0x0F | 0x10..=0x14 | 0x16..=0x27 | 0x2A..=0x34 | 0x36 | 0x38 | 0x3A..=0x3C | 0x3E | 0x3F => Plain,
            _ => Ender,
        },
        0x01 => match rt {
            0x00..=0x03 | 0x10..=0x13 => Branch,
            0x08..=0x0C | 0x0E => Plain,
            _ => Ender,
        },
        0x02..=0x07 | 0x14..=0x17 => Branch,
        0x10 => {
            if rs & 0x10 != 0 {
                Ender
            } else {
                match rs {
                    0x00 | 0x01 | 0x02 | 0x06 | 0x08 => Plain,
                    _ => Ender,
                }
            }
        }
        0x11 if rs == 0x08 => Branch,
        0x08..=0x0F | 0x11 | 0x12 | 0x18..=0x1B | 0x20..=0x2F | 0x30 | 0x31 | 0x34 | 0x35 | 0x37..=0x39 | 0x3C | 0x3D | 0x3F => Plain,
        _ => Ender,
    }
}

/// The cycles an instruction ticks: one, and the vendor's stall for the multiplies and divides.
pub fn cycles(i: u32) -> i64 {
    if i >> 26 != 0 {
        return 1;
    }
    1 + match i & 0x3F {
        0x18 | 0x19 => MULTIPLY_STALL,
        0x1C | 0x1D => MULTIPLY_DOUBLE_STALL,
        0x1A | 0x1B => DIVIDE_STALL,
        0x1E | 0x1F => DIVIDE_DOUBLE_STALL,
        _ => 0,
    } as i64
}

/// A shape: its length in words, or refused, which leaves its first word to the interpreter every time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shape {
    pub words: u32,
    pub refused: bool,
}

/// Scans from the first word; `limit` is how many words may be read, the end of RDRAM or of a mapped page.
pub fn shape(word: impl Fn(u32) -> u32, limit: u32) -> Shape {
    let limit = limit.min(MAX_WORDS);
    let mut n = 0;
    while n < limit {
        match class(word(n)) {
            Class::Plain => n += 1,
            Class::Ender => return Shape { words: n + 1, refused: false },
            Class::Branch => {
                // The slot must fit, and must not itself be a branch or an ender; otherwise the branch is left to the interpreter.
                if n + 1 >= limit || class(word(n + 1)) != Class::Plain {
                    return if n == 0 { Shape { words: 1, refused: true } } else { Shape { words: n, refused: false } };
                }
                return Shape { words: n + 2, refused: false };
            }
        }
    }
    if n == 0 { Shape { words: 1, refused: true } } else { Shape { words: n, refused: false } }
}
