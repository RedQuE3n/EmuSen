//! Diamond, the Sony PlayStation core in Rust, on the core ABI v1. Written from hardware documents under
//! Diamond_Plan.md §1.3's clean-room protocol; the build record is Diamond_Native.md.
//!
//! Stage 3: the R3000A runs against the bus, with memory control, the I-cache, the interrupts, the timers, the seven
//! DMA channels and the video timing; the later stages' devices answer as register stubs. The picture is black and
//! the sound silent until stages 4 and 8. The processors come from the BlackMoon crates.

pub mod bus;
pub mod devices;
pub mod dma;
pub mod exe;
pub mod icache;
pub mod irq;
pub mod kernel;
pub mod machine;
pub mod media;
pub mod memctl;
pub mod state;
pub mod timers;
pub mod v1;
pub mod video;
