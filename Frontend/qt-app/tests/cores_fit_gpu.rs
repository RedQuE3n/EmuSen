//! Cores' list measured on Vulkan (EmuSen_Qt.md §7.9), in a compositor of the test's own, as the acceptance runs, over the world esde.rs holds Cores in, in the own look and
//! under the ES-DE theme, at the gate's sizes: `ColumnFit` gives every content column one share of its words' width as
//! drawn, and the section strip, whose names run past it here, moves by the wheel. Whether the words are cut is the
//! gate's, which esde.rs holds; the measures and the findings are written to the comparison folder. cores_fit.rs holds the same on the software backend. A GPU job.

mod common;

use common::cores_fit::{both_looks, shares, table, world};
use common::out;
use emusen_client_fixture::library;
use emusen_platform_client::Platform;
use emusen_qt_play::host::Host;
use emusen_qt_test::compositor::Compositor;

#[test]
fn qt_7_9_cores_list_is_measured_as_drawn_and_its_strip_moves_by_the_wheel_on_vulkan() {
    unsafe { std::env::set_var("SDL_AUDIODRIVER", "dummy") };
    let kwin = Compositor::start(3840, 2160);
    kwin.watch(300);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    let mut harness = emusen_qt_test::Harness::new();
    let platform = Platform::load(library()).unwrap();
    let gpu = std::env::var("EMUSEN_GPU_DEVICE").ok();
    let host = Host::new(&platform, "wayland", gpu.as_deref(), false, &std::env::temp_dir().join("qt-app-cores-fit-presets")).expect("a Vulkan device");
    let adoption = host.adoption();
    let (world, cores) = world("qt-app-cores-fit-gpu");
    let mut scene = harness.load_prepared(emusen_qt::MAIN_URL, 1280, 800, |window| unsafe { adoption.adopt(window.cast()) }.expect("Qt adopts the device"));
    assert!(scene.set_property("program", "feed", &host.feed.address().to_string()));
    let mut report = Vec::new();
    let (own, themed) = both_looks(&mut scene, &world, &cores, &mut report);
    std::fs::write(out().join("cores-fit-vulkan.tsv"), format!("{}\n{}\n", table("own", &own), table("esde", &themed))).unwrap();
    assert!(!own.is_empty() && !themed.is_empty(), "cells measured");
    std::fs::write(out().join("cores-fit-vulkan-findings.txt"), report.join("\n")).unwrap();
    // The section strip's names run past it in this world, and the wheel moves it.
    let wheel: Vec<_> = report.iter().filter(|f| f.contains("the wheel does not move")).collect();
    assert!(wheel.is_empty(), "the section strip's wheel: {wheel:#?}");
    // ColumnFit measures each column as its words are drawn: every content column is given one share of that width.
    for (size, shares) in shares(&own).into_iter().chain(shares(&themed)) {
        assert!(shares.len() >= 2 && shares.iter().all(|s| (s - shares[0]).abs() < 0.001 && *s <= 1.0), "at {size:?} the content columns' shares {shares:?}");
    }
    scene.assert_quiet();
    assert!(scene.set_property("program", "visible", "false"));
    scene.wait(50);
    drop(harness);
    drop(host);
}
