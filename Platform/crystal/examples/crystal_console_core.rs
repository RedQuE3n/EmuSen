//! emusen-native's v1 test core, its capabilities and exports the same, as a PlayStation that starts with nothing in it
//! (`AS_PLAYSTATION`): the client fixture's console core, for Start Console (EmuSen_Qt.md §7.9).

#[allow(dead_code)]
mod test_core {
    pub const TEST_ID: &str = "v1-console-core";
    pub const TEST_NAME: &str = "V1ConsoleCore";
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
        | emusen_native::core::caps::DEBUG_DISASSEMBLE
        | emusen_native::core::caps::MEDIA
        | emusen_native::core::caps::DEBUG_READS
        | emusen_native::core::caps::BORROW
        | emusen_native::core::caps::DEVICE_PICTURE
        | emusen_native::core::caps::RAW_PICTURE
        | emusen_native::core::caps::TELEMETRY
        | emusen_native::core::caps::PRESENCE
        | emusen_native::core::caps::DEBUG_TRANSLATE
        | emusen_native::core::caps::DEVICE_RING;

    pub const TEST_FAULTS: u64 = AS_PLAYSTATION;
    pub const PAINTER: Option<MakePainter> = None;

    include!("../../../EmuSen/Cores/Shared/emusen-native/examples/common/test_core.rs");

    emusen_native::core_exports!(TestCore; reset, present, phases, audio_peek, mutes, axes, settings, setting_notes, rom_patches, cheat_pokes, debug, debug_stack, debug_registers, debug_disassemble, debug_reads, media, device_picture, raw_picture, telemetry, presence, debug_translate, device_ring);
}
