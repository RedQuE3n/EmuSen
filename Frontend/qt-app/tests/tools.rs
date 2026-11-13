//! The tools of jobs in the program, over the platform library built from this commit and a scratch library: the pad
//! opens Library Manager from Home's place, starts its first job, sees the bar's progress and then the job's result
//! lines and outcome, and B returns to Home; Compressed Discs, from the Home card's Tools page, lists a generated disc
//! the same way, and B returns to Home.

mod common;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_qt::MAIN_URL;
use emusen_qt_test::Scene;

const SHEET: &str = "FILE \"Disc.bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n";

fn focused(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).unwrap_or_default()
}

/// The focused item's words, or those of the first item under it that has some.
fn focused_words(scene: &mut Scene<'_>) -> String {
    let tree = scene.tree();
    let Some(focus) = scene.focus(&tree) else { return String::new() };
    tree.nodes.iter().skip(focus as usize).find_map(|n| n.text.clone()).unwrap_or_default()
}

/// A on the focused job, then the model read every 10 ms until the job has ended: whether the bar showed a share done
/// while it ran, then the pane's sentences and the bar's line.
fn run_first_job(scene: &mut Scene<'_>) -> (bool, String, String) {
    assert_eq!(scene.property("jobsModel", "progress"), "-1", "no job runs before A");
    scene.pad(UiButton::Accept);
    let mut progressed = false;
    for _ in 0..500 {
        let busy = scene.property("jobsModel", "busy") == "true";
        let progress: f64 = scene.property("jobsModel", "progress").parse().unwrap();
        progressed |= busy && (0.0..=1.0).contains(&progress);
        if progressed && !busy {
            break;
        }
        scene.wait(10);
    }
    wait_for(scene, "jobsModel", "busy", "false", 5000);
    (progressed, scene.property("jobsModel", "sentences"), scene.property("jobsModel", "summary"))
}

/// B on the tool screen: Home again, holding the pad.
fn back_home(scene: &mut Scene<'_>) {
    scene.pad(UiButton::Back);
    wait_for(scene, "screens", "shown", "Home", 2000);
    for _ in 0..20 {
        if focused(scene).starts_with("place.") {
            break;
        }
        scene.wait(50);
    }
    assert!(focused(scene).starts_with("place."), "Home holds the pad: {}", focused(scene));
}

#[test]
fn library_manager_and_compressed_discs_run_a_job_from_the_pad_and_b_returns_home() {
    let world = sandbox("qt-tools", &[("nes", "Alpha.nes")]);
    let psx = world.dir("Roms/psx");
    std::fs::write(psx.join("Disc.cue"), SHEET).unwrap();
    std::fs::write(psx.join("Disc.bin"), vec![0u8; 2352 * 300]).unwrap();
    let mut harness = emusen_qt_test::Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert!(scene.set_property("program", "splash", "false"));
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "library", "loading", "false", 10000);
    for _ in 0..20 {
        if focused(&mut scene) == "place.0" {
            break;
        }
        scene.wait(50);
    }

    // Library Manager, Home's fourth place.
    for _ in 0..3 {
        scene.pad(UiButton::Right);
    }
    assert_eq!(focused(&mut scene), "place.3");
    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "screens", "shown", "Tool", 2000);
    wait_for(&mut scene, "jobsModel", "title", "Library Manager", 2000);
    scene.wait(200);
    assert!(focused_words(&mut scene).starts_with("Check every dump"), "the first job holds the focus: {}", focused_words(&mut scene));
    let (progressed, sentences, summary) = run_first_job(&mut scene);
    assert!(progressed, "the bar showed the job's share done while it ran");
    assert!(sentences.contains("psx/Disc.cue: ok") && sentences.contains("X lists all 2 lines."), "the pane's latest result line: {sentences}");
    assert!(summary.contains(" ok") && summary.ends_with(" s."), "the bar's outcome: {summary}");
    scene.pad(UiButton::Screensaver);
    wait_for(&mut scene, "jobsModel", "cardOpen", "true", 1000);
    let lines = scene.property("jobsModel", "cardRows");
    assert!(lines.contains("nes/Alpha.nes: ok") && lines.contains("psx/Disc.cue: ok"), "X lists every line: {lines}");
    scene.pad(UiButton::Back);
    wait_for(&mut scene, "jobsModel", "cardOpen", "false", 1000);
    back_home(&mut scene);

    // Compressed Discs, from the Home card's Tools page.
    scene.pad(UiButton::Menu);
    wait_for(&mut scene, "screens", "shown", "HomeCard", 2000);
    scene.wait(300);
    for row in ["Tools", "Compressed Discs"] {
        for _ in 0..12 {
            if focused_words(&mut scene).starts_with(row) {
                break;
            }
            scene.pad(UiButton::Down);
        }
        assert!(focused_words(&mut scene).starts_with(row), "no row reads {row}: {}", focused_words(&mut scene));
        scene.pad(UiButton::Accept);
        scene.wait(300);
    }
    wait_for(&mut scene, "screens", "shown", "Tool", 2000);
    wait_for(&mut scene, "jobsModel", "title", "Compressed Discs", 2000);
    scene.wait(200);
    assert!(focused_words(&mut scene).starts_with("List disc games"), "{}", focused_words(&mut scene));
    let (progressed, sentences, summary) = run_first_job(&mut scene);
    assert!(progressed, "the bar showed the job's share done while it ran");
    assert!(sentences.contains("psx/Disc.cue: not converted"), "the pane's result line: {sentences}");
    assert!(summary.starts_with("1 not converted; "), "the bar's outcome: {summary}");
    back_home(&mut scene);
}
