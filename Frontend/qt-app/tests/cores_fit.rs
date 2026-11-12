//! Cores' list measured on the software backend (EmuSen_Qt.md §7.9), over the world esde.rs holds Cores in, in the own look and
//! under the ES-DE theme, at the gate's sizes: the gate finds nothing, every cell's words fit, the status column is given its
//! widest words, and the console column gives way to the first console and a count (decided 2026-11-11); the measures
//! and the findings are written to the comparison folder. cores_fit_gpu.rs holds the same on Vulkan.

mod common;

use common::cores_fit::{both_looks, both_sides, misfits, table, world};
use common::out;

#[test]
fn qt_7_9_cores_list_holds_every_word_whole_or_gives_way_on_the_software_backend() {
    let (world, cores) = world("qt-app-cores-fit");
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(emusen_qt::MAIN_URL, 1280, 800);
    let mut report = Vec::new();
    let (own, themed) = both_looks(&mut scene, &world, &cores, &mut report);
    std::fs::write(out().join("cores-fit-software.tsv"), format!("{}\n{}\n", table("own", &own), table("esde", &themed))).unwrap();
    assert!(!own.is_empty() && !themed.is_empty(), "cells measured");
    std::fs::write(out().join("cores-fit-software-findings.txt"), report.join("\n")).unwrap();
    // The gate finds nothing on Cores in either look, at any size: no cut, no overlap, no reach, the wheel moving the strip.
    assert!(report.is_empty(), "the gate on Cores: {report:#?}");
    // Every cell's words fit; the console column gave way to the first console and a count where the whole list did not.
    let several = [("Genesis, Sega CD, Master System, SG-1000", "Genesis +3"), ("Game Boy, Game Boy Color", "Game Boy +1")];
    let misfits: Vec<String> = misfits(&own, &several).into_iter().chain(misfits(&themed, &several)).collect();
    assert!(misfits.is_empty(), "{misfits:#?}");
    // Both sides of giving way: two consoles whole where their room holds them and their first and a count where it does
    // not, in each look; a core of one console whole everywhere.
    let sides: Vec<String> = [&own, &themed].iter().flat_map(|cells| both_sides(cells, "Game Boy, Game Boy Color", "Game Boy +1", &["NES", "Super Nintendo"])).collect();
    assert!(sides.is_empty(), "{sides:#?}");
    scene.assert_quiet();
}
