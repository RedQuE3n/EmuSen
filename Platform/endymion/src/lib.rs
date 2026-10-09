//! Endymion: what is decided between a pad or a sample stream and the core, and the devices on either side of it.
//!
//! The logic half (`pad`, `rate_control`, `resampler`, `router`, `slots`) has no device in it; the device half
//! (`sdl`, `devices`, `pads`, `audio`) reaches SDL3 or a simulated set of pads. The C# `EmuSen.Endymion` forwards here
//! behind `EMUSEN_ENDYMION_NATIVE`, and a Rust DianaOS calls it directly. Every rule reproduces the C#'s, which is its
//! oracle until its gate; EmuSen_RustPlatform.md §12 and §14 say how that was held.

pub mod audio;
pub mod devices;
pub mod pad;
pub mod pads;
pub mod rate_control;
pub mod resampler;
pub mod router;
pub mod sdl;
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
