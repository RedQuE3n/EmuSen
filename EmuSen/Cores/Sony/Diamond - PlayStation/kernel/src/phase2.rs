//! The calls whose devices the kernel does not have: the ROM's character sets, which an open kernel does not carry;
//! each returns the failure psx-spx documents for it. Diamond_OpenKernel.md §5.

/// The calls that return an address or a count: -1.
pub extern "C" fn failed() -> u32 {
    u32::MAX
}
