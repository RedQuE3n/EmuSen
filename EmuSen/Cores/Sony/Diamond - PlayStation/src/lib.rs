//! Diamond, the Sony PlayStation core in Rust, on the core ABI v1. Written from hardware documents under
//! Diamond_Plan.md §1.3's clean-room protocol; the build record is Diamond_Native.md.
//!
//! Stage 4: the R3000A runs against the bus, with memory control, the I-cache, the interrupts, the timers, the seven
//! DMA channels, the video timing and the GPU, whose display is the picture, and SIO0 with port 1's digital pad,
//! brought forward from stage 6; the later stages' devices answer as register stubs, and the sound is silent until
//! stage 8. The processors come from the BlackMoon crates.

pub mod bus;
pub mod card;
pub mod cdrom;
pub mod devices;
pub mod dma;
pub mod exe;
pub mod gpu;
pub mod icache;
pub mod irq;
pub mod kernel;
pub mod machine;
pub mod mdec;
pub mod media;
pub mod medium;
pub mod memctl;
pub mod multitap;
pub mod pad;
pub mod sio0;
pub mod state;
pub mod timers;
pub mod v1;
pub mod video;
