//! The drawing time (Diamond_Plan.md §5.4), in 256ths of a GPU cycle of the video clock, fitted to ps1-tests'
//! `gpu/bandwidth` console log; how each figure was found, and which were chosen with no measurement, is
//! Diamond_GPU.md §8. A command's time is charged from its start, and the GPU takes its next word when it is spent.

use super::draw::Work;

/// One GPU cycle.
pub const CYCLE: u64 = 256;
/// A command's own time once its words are in.
pub const COMMAND: u64 = 2 * CYCLE;
/// A triangle's setup; a rectangle's row drawn; a line's setup [C].
pub const TRIANGLE: u64 = 876 * CYCLE;
pub const RECTANGLE_ROW: u64 = 791;
pub const LINE: u64 = 16 * CYCLE;
/// A pixel written; one read and written (semi-transparent, or the mask checked); a texel fetched, written or not.
pub const WRITE: u64 = 214;
pub const READ_WRITE: u64 = 324;
pub const TEXEL: u64 = 114;
/// A texture cache entry fetched, and a CLUT entry loaded [C].
pub const MISS: u64 = 5035;
pub const CLUT_ENTRY: u64 = CYCLE;
/// The fill: a row, and each 16 pixels of it.
pub const FILL_ROW: u64 = 765;
pub const FILL_16: u64 = 2 * CYCLE;
/// The copy, per pixel read and written; a CPU-to-VRAM data word, two pixels written; a VRAM-to-CPU word made ready.
pub const COPY_PIXEL: u64 = 545;
pub const WRITE_WORD: u64 = 2 * WRITE;
pub const READ_WORD: u64 = 894;
pub const READ_SETUP: u64 = 16 * CYCLE;

/// A rendering command's time from what it did.
pub fn drawing(setup: u64, w: Work) -> u64 {
    COMMAND + setup + w.texels as u64 * TEXEL + w.writes as u64 * WRITE + w.read_writes as u64 * READ_WRITE + w.misses as u64 * MISS
}

/// From CPU cycle `t` with `carry` left over, `cost` later: the cycle reached and what is left over, in 1/(256 × the
/// clock's numerator) of a CPU cycle, so that a run of short commands loses nothing to rounding.
pub fn advance(t: u64, carry: u64, cost: u64, clock: (u64, u64)) -> (u64, u64) {
    let unit = clock.0 as u128 * CYCLE as u128;
    let total = carry as u128 + cost as u128 * 2 * clock.1 as u128;
    (t + (total / unit) as u64, (total % unit) as u64)
}

/// GPU time in 256ths of a cycle to CPU cycles, `clock` being the video clock's half cycles per CPU cycle.
pub fn cpu_cycles(gpu: u64, clock: (u64, u64)) -> u64 {
    (gpu as u128 * 2 * clock.1 as u128).div_ceil(clock.0 as u128 * CYCLE as u128) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_gpu_cycle_is_a_video_clock() {
        let ntsc = crate::video::NTSC_CLOCK;
        assert_eq!(cpu_cycles(3_412_500 * CYCLE / 10, ntsc), 215_256, "a tenth of 1000 lines of 2,152.55 cycles, rounded up");
        let (t, carry) = (0..1000).fold((0, 0), |(t, c), _| advance(t, c, WRITE_WORD, ntsc));
        assert_eq!(t, cpu_cycles(1000 * WRITE_WORD, ntsc) - 1, "no rounding lost over a run, {carry} left over");
        assert_eq!(cpu_cycles(1, ntsc), 1);
    }
}
