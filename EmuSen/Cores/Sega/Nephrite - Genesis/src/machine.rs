//! The stage 0 machine: the image recognised, the memories of the three systems allocated and zeroed, a frame
//! counter, and a blank picture. Nothing is emulated (Nephrite_Native.md §1).

use crate::media::{Media, System};

/// A memory the machine holds, by its space name and id (Nephrite_Plan.md §4.3).
pub struct Memory {
    pub id: u32,
    pub name: &'static str,
    pub bytes: Vec<u8>,
    pub read_only: bool,
}

/// The Genesis's memories, then the Sega CD's, then the 32X's, as (id, name, size).
pub const MD_MEMORIES: [(u32, &str, usize); 5] = [(2, "WRAM", 0x1_0000), (3, "Z80RAM", 0x2000), (4, "VRAM", 0x1_0000), (5, "CRAM", 128), (6, "VSRAM", 80)];
pub const MCD_MEMORIES: [(u32, &str, usize); 4] = [(17, "PRGRAM", 0x8_0000), (18, "WORDRAM", 0x4_0000), (19, "PCMRAM", 0x1_0000), (20, "BRAM", 0x2000)];
pub const S32X_MEMORIES: [(u32, &str, usize); 3] = [(34, "SDRAM", 0x4_0000), (35, "FRAMEBUFFER", 0x4_0000), (36, "PALETTE", 512)];
pub const SRAM_ID: u32 = 7;
pub const ROM_ID: u32 = 8;

/// The picture before the video processor exists: H40, 28 rows of tiles.
pub const WIDTH: u32 = 320;
pub const HEIGHT: u32 = 224;

pub struct Machine {
    pub media: Media,
    pub memories: Vec<Memory>,
    pub frames: i64,
    pub picture: Vec<u8>,
    pub skip: bool,
    pub pads: [u32; 2],
    /// The firmware files given at create, by number; none is used yet.
    pub firmware: Vec<u32>,
}

impl Machine {
    /// Every memory the image's system has; the cartridge's ROM, and its battery RAM when the header declares one.
    pub fn new(image: &[u8], media: Media) -> Machine {
        let mut memories = Vec::new();
        let mut add = |list: &[(u32, &'static str, usize)]| {
            for &(id, name, size) in list {
                memories.push(Memory { id, name, bytes: vec![0; size], read_only: false });
            }
        };
        add(&MD_MEMORIES);
        match media.system {
            System::Mcd => add(&MCD_MEMORIES),
            System::S32x => add(&S32X_MEMORIES),
            System::Md => {}
        }
        if media.battery_bytes() > 0 {
            memories.push(Memory { id: SRAM_ID, name: "SRAM", bytes: vec![0; media.battery_bytes()], read_only: false });
        }
        if media.system != System::Mcd {
            memories.push(Memory { id: ROM_ID, name: "ROM", bytes: image.to_vec(), read_only: true });
        }
        memories.sort_by_key(|m| m.id);
        let picture = blank();
        Machine { media, memories, frames: 0, picture, skip: false, pads: [0; 2], firmware: Vec::new() }
    }

    pub fn memory(&self, id: u32) -> Option<&Memory> {
        self.memories.iter().find(|m| m.id == id)
    }

    pub fn memory_mut(&mut self, id: u32) -> Option<&mut Memory> {
        self.memories.iter_mut().find(|m| m.id == id)
    }

    /// The memory the battery file holds: the cartridge's save RAM, or the Sega CD's internal backup RAM.
    pub fn battery_id(&self) -> Option<u32> {
        match self.media.system {
            System::Mcd => Some(20),
            _ => self.memory(SRAM_ID).map(|m| m.id),
        }
    }

    /// One frame: the count moves and nothing else does.
    pub fn advance(&mut self) {
        self.frames += 1;
    }
}

/// An opaque black frame, RGBA.
pub fn blank() -> Vec<u8> {
    (0..(WIDTH * HEIGHT) as usize).flat_map(|_| [0, 0, 0, 255]).collect()
}
