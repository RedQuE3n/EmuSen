//! The noise generator (psx-spx "SPU Noise Generator", Diamond_SPU.md §3.6): a timer less the step each sample, and
//! below zero a new bit, the parity of bits 15, 12, 11 and 10 inverted, the timer recharged by 20000h >> shift once
//! and again if need be. A core restarts the timer when its shift or step is written (Diamond's D-37).

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Noise {
    /// The level a voice in noise mode takes as its sample.
    pub level: i16,
    pub timer: i32,
}

impl Noise {
    /// One sample at `shift` (0..15) and `step` (0..3, acting as 4..7).
    #[inline]
    pub fn tick(&mut self, shift: u16, step: u16) {
        let step = (step & 3) as i32 + 4;
        let shift = (shift & 0xF) as i32;
        self.timer -= step;
        let n = self.level as u16;
        let parity = (n >> 15 ^ n >> 12 ^ n >> 11 ^ n >> 10 ^ 1) & 1;
        if self.timer < 0 {
            self.level = (n << 1 | parity) as i16;
            self.timer += 0x2_0000 >> shift;
            if self.timer < 0 {
                self.timer += 0x2_0000 >> shift;
            }
        }
    }
}
