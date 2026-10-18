//! The recompiler's switch and counters. See Mars_Native.md §5.8.

use crate::cpu::blocks::Tier;
use crate::ffi::Core;

impl Core {
    /// The recompiler on or off, its verifier, and its tier, 0 the furthest built.
    pub fn set_recompiler(&mut self, on: bool, verify: bool, tier: u32) {
        self.machine.set_recompiler(on);
        let verify = verify || crate::cpu::blocks::Blocks::verify_by_default();
        self.machine.blocks.configure(Tier::from_number(tier & 0xF), verify);
    }
}
