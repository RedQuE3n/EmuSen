//! emusen-native's test core taking any non-empty image whole, as a core of an open image format may.

pub const TEST_ID: &str = "v1-lenient-core";
pub const TEST_NAME: &str = "V1LenientCore";
pub const TEST_CAPABILITIES: u64 = 0;
pub const TEST_FAULTS: u32 = LENIENT_IMAGES;

include!("../../emusen-native/examples/common/test_core.rs");

emusen_native::core_exports!(TestCore;);
