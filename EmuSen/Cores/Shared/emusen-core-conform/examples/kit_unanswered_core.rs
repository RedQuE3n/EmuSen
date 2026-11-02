//! emusen-native's test core refusing a state under a host of minor 99 with a status no text names, which only C12 reaches.

pub const TEST_ID: &str = "v1-unanswered-core";
pub const TEST_NAME: &str = "V1UnansweredCore";
pub const TEST_CAPABILITIES: u64 = 0;
pub const TEST_FAULTS: u64 = FAULT_OLD_HOST_UNANSWERED;
pub const PAINTER: Option<MakePainter> = None;

include!("../../emusen-native/examples/common/test_core.rs");

emusen_native::core_exports!(TestCore;);
