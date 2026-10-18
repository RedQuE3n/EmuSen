//! The VR4300's instructions as text, through `emusen-mips`, which owns the MIPS instruction set for both MIPS
//! processors (EmuSen_Backend.md F8): MIPS III in the VR4300's spelling, with the address a word names.

use crate::debug::Reference;
use emusen_mips::vr4300;

/// One word at `address`: its mnemonic, its operands and the address it names where the word alone gives it.
pub fn decode(word: u32, address: u32) -> (String, String, Option<(Reference, u32)>) {
    let i = vr4300::decode(word, address);
    let r = i.reference.map(|(k, t)| {
        let kind = match k {
            vr4300::Reference::Call => Reference::Call,
            vr4300::Reference::Read => Reference::Read,
            vr4300::Reference::Write => Reference::Write,
        };
        (kind, t)
    });
    (i.mnemonic, i.operands, r)
}
