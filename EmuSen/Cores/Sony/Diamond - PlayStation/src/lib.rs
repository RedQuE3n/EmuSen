//! Diamond, the Sony PlayStation core in Rust, on the core ABI v1. Written from hardware documents under
//! Diamond_Plan.md §1.3's clean-room protocol; the build record is Diamond_Native.md.
//!
//! Stage 1: the image is recognised and reported, the memories exist, the picture is black and the sound silent. The
//! processors come from the BlackMoon crates (`blackmoon-r3000`, `blackmoon-gte`) as they are built.

pub mod machine;
pub mod media;
pub mod state;
pub mod v1;
