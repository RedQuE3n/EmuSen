// GPU-tier mapping checks: video playback and shader-backed element properties that require Vulkan RHI.

mod common;

use common::mapping::{all_cases, is_gpu_case, run_cases, Case, H, W};
use emusen_esde::scene::mapping::{self, Status};
use emusen_qt_test::Harness;

#[test]
fn every_gpu_drawn_pair_changes_the_painted_pixels() {
    assert!(emusen_esde_qt::marks::mark_named("star").is_some(), "the module is linked into this test");
    let _kwin = common::start();
    // Held clocks are refused under a threaded render loop, so force the basic render loop.
    std::env::set_var("QSG_RENDER_LOOP", "basic");
    let only = std::env::var("ESDE_MAPPING_ONLY").ok();
    let cases: Vec<Case> = all_cases()
        .into_iter()
        .filter(is_gpu_case)
        .filter(|c| mapping::status(&c.kind, &c.property) == Some(&Status::Drawn))
        .filter(|c| only.as_deref().is_none_or(|o| o.split(',').any(|w| w == c.to_string())))
        .collect();
    assert!(!cases.is_empty());
    let mut harness = Harness::new();
    let mut scene = harness.load(common::VIEW, W, H);
    run_cases(&mut scene, "mapping-gpu", &cases);
}
