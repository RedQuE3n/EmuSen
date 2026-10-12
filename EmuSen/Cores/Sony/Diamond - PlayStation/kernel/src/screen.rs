//! The screen with no disc. Diamond_OpenKernel.md §16.

use crate::layout::io;
use crate::mem::{rd, wr};
use crate::{card, entry, pad};

/// `frames` vblanks waited: counted by the shared element when it runs, else from I_STAT's vblank bit.
pub fn pause(frames: u32) {
    for _ in 0..frames {
        if pad::queued() && rd(io::I_MASK) & 1 != 0 && entry::interrupts_on() {
            let v = card::vblanks();
            while card::vblanks() == v {}
        } else {
            wr(io::I_STAT, !1);
            while rd(io::I_STAT) & 1 == 0 {}
        }
    }
}

/// Until a disc may have been put in.
pub fn no_disc() {
    pause(60);
}
