//! The emitter's measurement forms compiled out: every lever off (`forms.rs`).

use cranelift_frontend::FunctionBuilder;

pub const CACHE: u8 = 1;
pub const CHANGED: u8 = 2;
pub const RELOAD: u8 = 4;
pub const PRESERVE: u8 = 32;
pub const PINNED: u8 = 64;
pub const COUNT: u8 = 8;

pub struct Watched;

#[inline(always)]
pub fn trampolines(helpers: [super::Helper; 3]) -> [super::Helper; 3] {
    helpers
}

#[inline(always)]
pub fn watch(_: &crate::ee::Ee, _: &super::Context) -> Option<Watched> {
    None
}

#[inline(always)]
pub fn watched(_: Option<Watched>, _: &crate::ee::Ee, _: &super::Context, _: usize) {}

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
