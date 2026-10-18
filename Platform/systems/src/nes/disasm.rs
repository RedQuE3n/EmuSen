//! The 2A03's instructions as text: the shared `emusen-6502` table, which MoonRT's `debug_disassemble` reads too, with
//! its references in the pack's words (Moon_Debug.md §5, EmuSen_Backend.md §21).

use crate::debug::Reference;

pub use emusen_6502::{Mode, NAMES, format_operand, length_of, mnemonic_of, mode_of};

/// Only the forms whose target the bytes alone give: an absolute or zero-page operand, by what the instruction does with it.
pub fn classify(opcode: u8, operand: i32) -> Option<(Reference, i32)> {
    emusen_6502::classify(opcode, operand).map(|(kind, target)| {
        let kind = match kind {
            emusen_6502::Reference::Call => Reference::Call,
            emusen_6502::Reference::Read => Reference::Read,
            emusen_6502::Reference::Write => Reference::Write,
        };
        (kind, target)
    })
}
