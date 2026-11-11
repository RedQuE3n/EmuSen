//! Cores' list measured on Vulkan (EmuSen_Qt.md §7.9), in a compositor of the test's own, as the acceptance runs, over the world esde.rs holds Cores in, in the own look and
//! under the ES-DE theme, at the gate's sizes: the gate finds nothing, every cell's words fit, the status column is given its
//! widest words, and the console column gives way to the first console and a count (decided 2026-11-11); the measures
//! and the findings are written to the comparison folder. cores_fit.rs holds the same on the software backend. A GPU job.

mod common;

use common::cores_fit::{both_looks, misfits, table, world};
use common::out;
use emusen_client_fixture::library;
use emusen_platform_client::Platform;
use emusen_qt_play::host::Host;
use emusen_qt_test::compositor::Compositor;

#[test]
fn qt_7_9_cores_list_holds_every_word_whole_or_gives_way_on_vulkan() {
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
    // The gate finds nothing on Cores in either look, at any size: no cut, no overlap, no reach, the wheel moving the strip.
    assert!(report.is_empty(), "the gate on Cores: {report:#?}");
    // Every cell's words fit; the console column gave way to the first console and a count where the whole list did not.
    let several = [("Genesis, Sega CD, Master System, SG-1000", "Genesis +3"), ("Game Boy, Game Boy Color", "Game Boy +1")];
    let misfits: Vec<String> = misfits(&own, &several).into_iter().chain(misfits(&themed, &several)).collect();
    assert!(misfits.is_empty(), "{misfits:#?}");
    scene.assert_quiet();
    assert!(scene.set_property("program", "visible", "false"));
    scene.wait(50);
    drop(harness);
    drop(host);
}
