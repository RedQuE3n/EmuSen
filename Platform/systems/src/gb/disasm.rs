//! The SM83's instructions as text: `silvermillennium-sm83`'s, the crate MercuryRT decodes with too, so the processor has one
//! table (EmuSen_Backend.md F8, §17.2 D10).

use crate::debug::Reference;

pub use silvermillennium_sm83::{CB_LENGTH, ILLEGAL, format_operand, length_of, mnemonic_of};

/// Only the forms whose target the bytes alone give.
pub fn classify(opcode: u8, operand: i32, address: i32) -> Option<(Reference, i32)> {
    silvermillennium_sm83::classify(opcode, operand, address).map(|(k, t)| {
        let kind = match k {
            silvermillennium_sm83::Reference::Call => Reference::Call,
            silvermillennium_sm83::Reference::Read => Reference::Read,
            silvermillennium_sm83::Reference::Write => Reference::Write,
        };
        (kind, t)
    })
}
