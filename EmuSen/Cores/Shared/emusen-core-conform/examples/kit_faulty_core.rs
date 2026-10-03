//! emusen-native's test core with both seeded faults on, so that this crate's tests see the kit fail it.

pub const TEST_ID: &str = "v1-faulty-core";
pub const TEST_NAME: &str = "V1FaultyCore";
pub const TEST_CAPABILITIES: u64 = 0;
pub const TEST_FAULTS: u32 = FAULT_MACHINES_DIFFER | FAULT_TAKES_TRUNCATED_STATE;

include!("../../emusen-native/examples/common/test_core.rs");

emusen_native::core_exports!(TestCore;);
