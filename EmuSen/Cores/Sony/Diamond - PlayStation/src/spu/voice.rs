//! The voices are `blackmoon-spu`'s (BlackMoon_SPU.md); what is Diamond's is the IRQ address's comparison, which
//! watches every access a voice, the reverb, the capture buffers and the transfer make (Diamond_SPU.md §7).

pub use blackmoon_spu::voice::{BLOCK_SAMPLES, Phase, Voice, step, sweep};

/// What an access to sound RAM meets: the IRQ address, as its 8-byte unit, and whether it was hit.
#[derive(Clone, Copy, Debug, Default)]
pub struct Watch {
    pub unit: u32,
    pub armed: bool,
    pub hit: bool,
}

impl Watch {
    #[inline]
    pub fn touch(&mut self, address: u32) {
        if self.armed && address >> 3 & 0xFFFF == self.unit {
            self.hit = true;
        }
    }
}

impl blackmoon_spu::Touch for Watch {
    #[inline]
    fn touch(&mut self, address: u32) {
        Watch::touch(self, address);
    }
}
