//! The 32X, the Genesis's attachment in its cartridge slot: a stub on the interface until stages 11 and 12 build it
//! (Nephrite_Attachments.md §3.9). It holds the three memories a 32X image's machine has had since stage 0 and its
//! facts for a frontend, answers nothing on the 68000's bus, and keeps no state of its own; a 32X image's machine still
//! counts frames only.

use emusen_native::core::{Firmware, Replacement};

use crate::interface::{Answer, Attachment, AttachmentState, AttachmentView, Master, Refusal, SpaceRef, StateReader, StateWriter, Steps, Truncated};

/// The 32X's memories by their space ids, from its range (Nephrite_Plan.md §4.3).
pub const MEMORIES: [(u32, &str, usize); 3] = [(34, "SDRAM", 0x4_0000), (35, "FRAMEBUFFER", 0x4_0000), (36, "PALETTE", 512)];
/// Its processors' ids, fixed whatever is fitted (decided 2026-10-15, Nephrite_Attachments.md Q7).
pub const MSH2: u32 = 3;
pub const SSH2: u32 = 4;

/// The extension its images carry.
pub const EXTENSIONS: [&str; 1] = [".32x"];

/// Whether a header's system field names the 32X.
pub fn is_32x(system_type: &str) -> bool {
    system_type.contains("32X")
}

const BOOT_COST: &str = "Nephrite's open replacements for the 32X's boot ROMs are not written yet (Nephrite_Plan.md §5.6): at this stage no 32X game runs, with the images or without them.";

/// Its three boot ROMs, the 68000's, the master SH-2's and the slave's, as the frontend's files `which`.
pub fn boot_firmware(which: [u32; 3]) -> Vec<Firmware> {
    let rom = |which, name: &str, label: &str, size| Firmware { which, name: name.into(), label: label.into(), size, required: false, parts: vec![vec![name.into()]], replacement: Some(Replacement::None { cost: BOOT_COST.into() }) };
    vec![
        rom(which[0], "32X_G_BIOS.BIN", "The 32X's 68000 boot ROM (256 bytes)", 256),
        rom(which[1], "32X_M_BIOS.BIN", "The 32X's master SH-2 boot ROM (2,048 bytes)", 2048),
        rom(which[2], "32X_S_BIOS.BIN", "The 32X's slave SH-2 boot ROM (1,024 bytes)", 1024),
    ]
}

/// The stub: its memories, each zeroed.
pub struct S32x {
    memories: Vec<Vec<u8>>,
}

impl S32x {
    pub fn new() -> S32x {
        S32x { memories: MEMORIES.iter().map(|&(_, _, size)| vec![0; size]).collect() }
    }
}

impl Default for S32x {
    fn default() -> S32x {
        S32x::new()
    }
}

impl Attachment for S32x {
    fn read8(&mut self, _: u32, _: Master) -> Answer<u8> {
        Answer::Pass
    }
    fn read16(&mut self, _: u32, _: Master) -> Answer<u16> {
        Answer::Pass
    }
    fn write(&mut self, _: u32, _: u16, _: u16, _: Master) -> bool {
        false
    }
    fn run_to(&mut self, _: Master) {}
    fn sound(&mut self, _: Master) -> Option<&mut Steps> {
        None
    }
}

/// No block of its own: its memories are the machine's spaces.
impl AttachmentState for S32x {
    type Saved = ();
    fn write_state(&self, _: &mut StateWriter) {}
    fn read_state(&self, _: &mut StateReader, _: i32) -> Result<(), Truncated> {
        Ok(())
    }
    fn check(&self, _: &()) -> Result<(), Refusal> {
        Ok(())
    }
    fn apply_state(&mut self, _: ()) {}
}

impl AttachmentView for S32x {
    fn spaces(&self, out: &mut Vec<SpaceRef>) {
        out.extend(MEMORIES.iter().map(|&(id, name, _)| SpaceRef { id, name, read_only: false }));
    }

    fn bytes(&self, id: u32) -> Option<&[u8]> {
        MEMORIES.iter().position(|m| m.0 == id).map(|i| self.memories[i].as_slice())
    }

    fn bytes_mut(&mut self, id: u32) -> Option<&mut [u8]> {
        MEMORIES.iter().position(|m| m.0 == id).map(|i| self.memories[i].as_mut_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interface::S32X_SPACES;

    // The stub lists its three memories from its range and answers nothing on the bus.
    #[test]
    fn the_stub_holds_its_memories_and_passes_every_access() {
        let mut s = S32x::new();
        let mut spaces = Vec::new();
        s.spaces(&mut spaces);
        assert_eq!(spaces.iter().map(|m| (m.id, m.name)).collect::<Vec<_>>(), [(34, "SDRAM"), (35, "FRAMEBUFFER"), (36, "PALETTE")]);
        assert!(spaces.iter().all(|m| S32X_SPACES.contains(&m.id)));
        assert_eq!((s.bytes(36).map(<[u8]>::len), s.bytes(37)), (Some(512), None));
        assert_eq!((s.read16(0x00_0000, 0), s.read8(0xA1_5100, 0), s.write(0x84_0000, 0, 0xFFFF, 0)), (Answer::Pass, Answer::Pass, false));
        assert!(is_32x("SEGA 32X") && !is_32x("SEGA GENESIS"));
    }
}
