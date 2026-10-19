//! emusen-native's test core listing two systems, whose machine reports the system a host routed its image to, or
//! keeps its own under `KEEPS_ITS_RULE`; with core ABI 1.15's seeded faults behind EMUSEN_TEST_CORE_FAULTS (C24).

pub const TEST_ID: &str = "v1-two-system-core";
pub const TEST_NAME: &str = "V1TwoSystemCore";
pub const TEST_CAPABILITIES: u64 = 0;
pub const TEST_FAULTS: u64 = TWO_SYSTEMS | FAULT_ROUTED_HALF_KEPT | FAULT_ROUTED_RUNS_OTHERWISE | FAULT_ROUTED_OWN_REFUSED | KEEPS_ITS_RULE;

include!("../../emusen-native/examples/common/test_core.rs");

emusen_native::core_exports!(TestCore;);
