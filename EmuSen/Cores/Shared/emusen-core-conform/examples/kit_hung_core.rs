//! emusen-native's test core whose tenth frame never ends on an image whose payload begins 0xEE, so that the kit's
//! deadline on progress has a hang to find beside images that run.

pub const TEST_ID: &str = "v1-hung-core";
pub const TEST_NAME: &str = "V1HungCore";
pub const TEST_CAPABILITIES: u64 = 0;
pub const TEST_FAULTS: u64 = FAULT_HANGS;

include!("../../emusen-native/examples/common/test_core.rs");

emusen_native::core_exports!(TestCore;);
