//! Endymion's logic half: what is decided between a pad or a sample stream and the core, with no device in it.
//!
//! The C# `EmuSen.Endymion` forwards here behind `EMUSEN_ENDYMION_NATIVE`, and a Rust DianaOS calls it directly.
//! Every rule reproduces the C#'s to the bit, which is its oracle until its gate; EmuSen_RustPlatform.md §12 says
//! how that was held.

pub mod pad;
pub mod rate_control;
pub mod resampler;
pub mod router;
pub mod slots;

/// `Math.Clamp` for doubles: below the low bound is the low bound, above the high is the high, and anything else, a NaN included, is itself.
pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}
