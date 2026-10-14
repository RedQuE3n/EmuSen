//! emusen-native's test core whose picture settles three frames after a load, as an enhancement redrawing from the
//! machine does, with the faults of that rule, each chosen by `EMUSEN_TEST_CORE_FAULTS`.

pub const TEST_ID: &str = "v1-settling-core";
pub const TEST_NAME: &str = "V1SettlingCore";
pub const TEST_CAPABILITIES: u64 = 0;
pub const TEST_FAULTS: u32 = SETTLES | FAULT_NEVER_SETTLES | FAULT_SETTLES_UNDECLARED | FAULT_SOUND_DIFFERS_WHILE_SETTLING | FAULT_SETTLES_PAST_THE_MAXIMUM;

include!("../../emusen-native/examples/common/test_core.rs");

emusen_native::core_exports!(TestCore;);
