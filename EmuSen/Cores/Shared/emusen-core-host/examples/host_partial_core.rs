//! A library with the version export and nothing else, which no host may load as a core.

#[unsafe(no_mangle)]
pub extern "C" fn emusen_core_abi_version() -> u32 {
    (1 << 16) | 14
}
