//! emusen-native's test core taking a few milliseconds a frame, so that an image's cases run longer than a short
//! deadline while every frame ends.

pub const TEST_ID: &str = "v1-slow-core";
pub const TEST_NAME: &str = "V1SlowCore";
pub const TEST_CAPABILITIES: u64 = 0;
pub const TEST_FAULTS: u64 = SLOW_FRAMES;

include!("../../emusen-native/examples/common/test_core.rs");

emusen_native::core_exports!(TestCore;);
