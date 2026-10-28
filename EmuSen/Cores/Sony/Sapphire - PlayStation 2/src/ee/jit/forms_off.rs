//! The emitter's measurement forms compiled out: every lever off (`forms.rs`).

use cranelift_frontend::FunctionBuilder;

pub const CACHE: u16 = 1;
pub const CHANGED: u16 = 2;
pub const RELOAD: u16 = 4;
pub const PRESERVE: u16 = 32;
pub const PINNED: u16 = 64;
pub const COUNT: u16 = 8;
pub const PAGE: u16 = 128;
pub const LINE: u16 = 256;
pub const STUB: u16 = 512;

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
pub fn on(_: u16) -> bool {
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
