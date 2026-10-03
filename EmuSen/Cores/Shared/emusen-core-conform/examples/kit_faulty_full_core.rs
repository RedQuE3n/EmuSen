//! emusen-native's test core with every capability and the seeded faults C9 and C12-C15 are for, so that this crate's
//! tests see each of those cases fail it.

pub const TEST_ID: &str = "v1-faulty-full-core";
pub const TEST_NAME: &str = "V1FaultyFullCore";
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
pub const TEST_FAULTS: u32 =
    FAULT_EXACT_SETTING_CHANGES_STATE | FAULT_ONLY_HOST_1_0 | FAULT_INFO_DRIFTS | FAULT_READ_ONLY_WRITABLE | FAULT_OBSERVED_FRAME_DIFFERS;

include!("../../emusen-native/examples/common/test_core.rs");

emusen_native::core_exports!(TestCore; reset, present, phases, audio_peek, mutes, axes, settings, setting_notes, rom_patches, cheat_pokes, debug, debug_stack, debug_registers, debug_disassemble);
