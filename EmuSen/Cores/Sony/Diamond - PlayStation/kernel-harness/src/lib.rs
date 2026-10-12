//! The open kernel's host side (Diamond_OpenKernel.md §6, §7): the committed image and its rebuild, the call table
//! read from the kernel's source, and a machine of blackmoon-r3000 on a bus of RAM, the BIOS, a TTY capture and a memory
//! card's protocol on SIO0, which is not Diamond's bus and models no device's timing.

pub mod calls;
pub mod image;
pub mod machine;
pub mod psexe;
pub mod sha256;
pub mod sio0;
