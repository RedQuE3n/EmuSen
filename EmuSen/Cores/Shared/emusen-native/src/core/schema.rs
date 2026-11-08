//! The descriptor schemas the core ABI v1 commits beside its header, `abi/v1/*.schema.json`, as text. A host checks a
//! descriptor against them with `emusen-core-schema` (EmuSen_RustPlatform.md §47.22).

/// The schemas as committed beside the header.
pub const INFO: &str = include_str!("../../abi/v1/info.schema.json");
pub const MACHINE_INFO: &str = include_str!("../../abi/v1/machine-info.schema.json");
pub const SETTINGS: &str = include_str!("../../abi/v1/settings.schema.json");
pub const FIRMWARE: &str = include_str!("../../abi/v1/firmware.schema.json");
pub const SETTING_NOTES: &str = include_str!("../../abi/v1/setting-notes.schema.json");
pub const DISASSEMBLY: &str = include_str!("../../abi/v1/disassembly.schema.json");
