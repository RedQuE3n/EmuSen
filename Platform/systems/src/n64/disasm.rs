//! The N64's two processors' instructions as text, through `emusen-mips`, which owns the MIPS instruction set (EmuSen_Backend.md
//! F8): the VR4300's MIPS III, and the signal processor's scalar and vector units, each with the address a word names.

use crate::debug::Reference;
use emusen_mips::{rsp, vr4300};

/// One word at `address`: its mnemonic, its operands and the address it names where the word alone gives it.
pub fn decode(word: u32, address: u32) -> (String, String, Option<(Reference, u32)>) {
    text(vr4300::decode(word, address))
}

/// One signal-processor word at `address` in IMEM: its mnemonic, its operands and the IMEM or DMEM offset it names.
pub fn decode_rsp(word: u32, address: u32) -> (String, String, Option<(Reference, u32)>) {
    text(rsp::decode(word, address))
}

fn text(i: vr4300::Instruction) -> (String, String, Option<(Reference, u32)>) {
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
