//! A minimal core on the v1 ABI that claims every capability, so that `core_exports!` generates all 55 exports:
//! the crate's tests drive it in-process, `emusen-core-abi-check` checks its signatures against the header, and
//! CI's export check runs on its library. Its machine is a counter; nothing in it is a console.

pub const TEST_ID: &str = "v1-test-core";
pub const TEST_NAME: &str = "V1TestCore";
pub const TEST_CAPABILITIES: u64 = emusen_native::core::caps::RESET
    | emusen_native::core::caps::PRESENT
    | emusen_native::core::caps::SNAPSHOT
    | emusen_native::core::caps::AXES
    | emusen_native::core::caps::AUDIO_PEEK
    | emusen_native::core::caps::MUTES
    | emusen_native::core::caps::SETTINGS
    | emusen_native::core::caps::PHASES
    | emusen_native::core::caps::FRAME_SERIAL
    | emusen_native::core::caps::ROW_REPEAT
    | emusen_native::core::caps::BATTERY_DIRTY
    | emusen_native::core::caps::ROM_PATCHES
    | emusen_native::core::caps::DEBUG
    | emusen_native::core::caps::DEBUG_STACK
    | emusen_native::core::caps::CHEAT_POKES
    | emusen_native::core::caps::SETTING_NOTES
    | emusen_native::core::caps::DEBUG_REGISTERS
    | emusen_native::core::caps::DEBUG_DISASSEMBLE;

include!("common/test_core.rs");

emusen_native::core_exports!(TestCore; reset, present, phases, audio_peek, mutes, axes, settings, setting_notes, rom_patches, cheat_pokes, debug, debug_stack, debug_registers, debug_disassemble);
