//! The test core's machine claiming no capability: only the 32 required exports, so that a host's neutral answers
//! for a missing capability can be shown, and the conformance kit run on a core with nothing optional.

pub const TEST_ID: &str = "v1-plain-core";
pub const TEST_NAME: &str = "V1PlainCore";
pub const TEST_CAPABILITIES: u64 = 0;

pub const TEST_FAULTS: u32 = 0;

include!("common/test_core.rs");

emusen_native::core_exports!(TestCore;);
