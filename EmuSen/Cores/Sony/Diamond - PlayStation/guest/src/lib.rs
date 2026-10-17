//! What the open kernel and its shell share, so that each fact has one owner inside the console: volatile access to
//! fixed addresses, the 5x8 font, Shift-JIS's ASCII forms and the GPU's drawing primitives. Diamond_Shell.md §1.3.

#![no_std]

pub mod font;
pub mod gpu;
pub mod mem;
pub mod sjis;
