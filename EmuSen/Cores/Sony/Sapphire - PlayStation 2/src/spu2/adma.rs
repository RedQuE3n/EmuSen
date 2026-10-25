//! AutoDMA (ps2tek "SPU2 AutoDMA"): signed 16-bit stereo PCM streamed by a core's DMA channel into its part of the
//! MEMIN region, double-buffered, a block of 1,024 bytes the left channel's 256 samples then the right's. The core plays
//! the region as its sound data input at one sample a sample, buffer 0 then 1; a buffer the core has left is free, and
//! a free buffer is what the channel's request asks to fill (Sapphire_Native.md §32.5).

/// A block's words: 512 bytes of left samples, then 512 of right [R].
pub const BLOCK_WORDS: u32 = 256;
/// The samples a buffer holds, a side.
pub const BUFFER: u32 = 256;

/// A core's MEMIN as halfword addresses: core 0's left buffers at 2000h and 2100h, its right at 2200h and 2300h, core
/// 1's from 2400h [R].
pub const fn left(core: usize, buffer: u32) -> u32 {
    0x2000 + 0x400 * core as u32 + 0x100 * buffer
}

pub const fn right(core: usize, buffer: u32) -> u32 {
    left(core, buffer) + 0x200
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Adma {
    /// The sample the core plays next, 0..511 across both buffers.
    pub play: u32,
    /// Which buffers hold samples the core has not yet played.
    pub filled: [bool; 2],
    /// The buffer the channel fills next, and the words of its block already moved.
    pub buffer: u32,
    pub words: u32,
}

impl Adma {
    /// The buffer the core is playing.
    pub fn playing(&self) -> u32 {
        self.play / BUFFER
    }

    /// AutoDMA started: the next block for the buffer the core is not playing, the one it plays busy until it is left
    /// [C, D-267].
    pub fn start(&mut self) {
        let p = self.playing();
        self.filled = [false; 2];
        self.filled[p as usize] = true;
        self.buffer = 1 - p;
        self.words = 0;
    }

    /// The words the channel may move now: the rest of a block while its buffer is free [C].
    pub fn room(&self) -> u32 {
        if self.filled[self.buffer as usize] { 0 } else { BLOCK_WORDS - self.words }
    }

    /// Where the block's next word goes, as a halfword address; the block's end marks its buffer filled.
    pub fn next_word(&mut self, core: usize) -> u32 {
        let w = self.words;
        let at = if w < BLOCK_WORDS / 2 { left(core, self.buffer) + 2 * w } else { right(core, self.buffer) + 2 * (w - BLOCK_WORDS / 2) };
        self.words += 1;
        if self.words == BLOCK_WORDS {
            self.filled[self.buffer as usize] = true;
            self.buffer = 1 - self.buffer;
            self.words = 0;
        }
        at
    }

    /// The sample's two halfword addresses, then the play position advanced; a buffer finished is free again.
    pub fn take(&mut self, core: usize) -> (u32, u32) {
        let (b, i) = (self.playing(), self.play % BUFFER);
        let at = (left(core, b) + i, right(core, b) + i);
        self.play = (self.play + 1) % (2 * BUFFER);
        if self.play.is_multiple_of(BUFFER) {
            self.filled[b as usize] = false;
        }
        at
    }
}
