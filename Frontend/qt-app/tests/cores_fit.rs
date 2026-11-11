//! Cores' list measured on the software backend (EmuSen_Qt.md §7.9), over the world esde.rs holds Cores in, in the own look and
//! under the ES-DE theme, at the gate's sizes: `ColumnFit` gives every content column one share of its words' width as
//! drawn, and the section strip, whose names run past it here, moves by the wheel. Whether the words are cut is the
//! gate's, which esde.rs holds; the measures and the findings are written to the comparison folder. cores_fit_gpu.rs holds the same on Vulkan.

mod common;

use common::cores_fit::{both_looks, shares, table, world};
use common::out;

#[test]
fn qt_7_9_cores_list_is_measured_as_drawn_and_its_strip_moves_by_the_wheel_on_the_software_backend() {
    let (world, cores) = world("qt-app-cores-fit");
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(emusen_qt::MAIN_URL, 1280, 800);
    let mut report = Vec::new();
    let (own, themed) = both_looks(&mut scene, &world, &cores, &mut report);
    std::fs::write(out().join("cores-fit-software.tsv"), format!("{}\n{}\n", table("own", &own), table("esde", &themed))).unwrap();
    assert!(!own.is_empty() && !themed.is_empty(), "cells measured");
    std::fs::write(out().join("cores-fit-software-findings.txt"), report.join("\n")).unwrap();
    // The section strip's names run past it in this world, and the wheel moves it.
    let wheel: Vec<_> = report.iter().filter(|f| f.contains("the wheel does not move")).collect();
    assert!(wheel.is_empty(), "the section strip's wheel: {wheel:#?}");
    // ColumnFit measures each column as its words are drawn: every content column is given one share of that width.
    for (size, shares) in shares(&own).into_iter().chain(shares(&themed)) {
        assert!(shares.len() >= 2 && shares.iter().all(|s| (s - shares[0]).abs() < 0.001 && *s <= 1.0), "at {size:?} the content columns' shares {shares:?}");
    }
    scene.assert_quiet();
}
