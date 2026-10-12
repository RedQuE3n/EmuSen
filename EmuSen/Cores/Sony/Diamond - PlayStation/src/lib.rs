//! Diamond, the Sony PlayStation core in Rust, on the core ABI v1. Written from hardware documents under
//! Diamond_Plan.md §1.3's clean-room protocol; the build record is Diamond_Native.md.
//!
//! Stage 4: the R3000A runs against the bus, with memory control, the I-cache, the interrupts, the timers, the seven
//! DMA channels, the video timing and the GPU, whose display is the picture, and SIO0 with port 1's digital pad,
//! brought forward from stage 6; the CD-ROM controller (stage 7), the SPU (stage 8), the MDEC (stage 9) and the
//! debugger (stage 12); the serial port answers as a stub. The processors come from the BlackMoon crates.

pub mod bus;
pub mod card;
pub mod cdrom;
pub mod debugger;
pub mod devices;
pub mod dma;
pub mod exe;
pub mod gpu;
pub mod icache;
pub mod idle;
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
pub mod spu;
pub mod state;
pub mod timers;
pub mod v1;
pub mod video;
