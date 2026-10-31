//! The tailored code's memory: stubs reached, blocks laid side by side, a label's address in the data.

use emusen_assembler::x64::{self, X64};
use emusen_assembler::{Assembler, Code, Cond, Op, Section, Src, Width};

use crate::native::Memory;

fn stub() -> Code {
    let mut a = X64::new();
    let e = a.label();
    a.bind(e);
    a.alu(Op::Add, Width::W64, x64::RAX, x64::RAX, Src::Imm(1));
    a.ret();
    a.finish(e)
}

/// A block that answers its word plus one through the stub, or through a jump by a data word holding its cold label where its argument is 0.
fn block(word: i32) -> Code {
    let mut a = X64::new();
    let (entry, slow) = (a.label(), a.label());
    let slot = a.data_address(slow);
    a.bind(entry);
    a.mov_imm(x64::RAX, i64::from(word));
    a.cmp(Width::W64, x64::RDI, Src::Imm(0));
    let go = a.label();
    a.branch(Cond::Ne, go);
    a.jump_via_data(slot);
    a.bind(go);
    a.jump_stub(0);
    a.section(Section::Cold);
    a.bind(slow);
    a.alu(Op::Add, Width::W64, x64::RAX, x64::RAX, Src::Imm(100));
    a.jump_stub(0);
    a.finish(entry)
}

#[test]
fn blocks_are_laid_side_by_side_and_run_as_they_are_added() {
    let mut m = Memory::new(vec![stub()]);
    let mut entries = Vec::new();
    for k in 0..2000 {
        let e = m.place(block(k)).expect("placed");
        entries.push(e);
        // SAFETY: the entry of a block just placed, of this signature.
        let f = unsafe { std::mem::transmute::<u64, extern "C" fn(u64) -> u64>(e) };
        assert_eq!(f(1), k as u64 + 1);
        assert_eq!(f(0), k as u64 + 101, "through the data word");
    }
    // SAFETY: as above.
    let f = unsafe { std::mem::transmute::<u64, extern "C" fn(u64) -> u64>(entries[0]) };
    assert_eq!(f(1), 1, "the first block still runs");
    assert!(m.held() < 2000 * 256, "a block costs its bytes, not its pages: {}", m.held());
}
