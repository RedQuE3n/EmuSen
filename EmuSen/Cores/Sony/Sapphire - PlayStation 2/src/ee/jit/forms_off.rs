//! The emitter's measurement forms compiled out: every lever off (`forms.rs`).

use cranelift_frontend::FunctionBuilder;

pub const CACHE: u8 = 1;
pub const COUNT: u8 = 8;

pub fn kinds() -> i64 {
    0
}

#[inline(always)]
pub fn on(_: u8) -> bool {
    false
}

#[inline(always)]
pub fn is_static() -> bool {
    false
}

#[inline(always)]
pub fn tag(_: &mut FunctionBuilder, _: u32) {}

#[inline(always)]
pub fn push(_: &mut FunctionBuilder, _: u32) -> u32 {
    0
}
