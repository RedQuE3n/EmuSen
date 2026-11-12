//! The gates' lists of what a ruling accepted (EmuSen_Kit.md §7.102, §7.109): one file a gate in one folder, so no gate's
//! list overwrites another's, and a list that cannot be written fails its gate.

use emusen_qt_test::excuse::{accepted_list, write_accepted};

#[test]
fn kit_7_102_each_gate_s_accepted_list_survives_the_others() {
    let dir = std::env::temp_dir().join(format!("qt-test-accepted-{}", std::process::id()));
    let gates = ["gate", "screens", "settings", "qt-app-acceptance"];
    for gate in gates {
        write_accepted(&dir, gate, &[format!("{gate}'s line")]);
    }
    for gate in gates {
        assert_eq!(std::fs::read_to_string(accepted_list(&dir, gate)).unwrap(), format!("{gate}'s line"), "{gate}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
#[should_panic(expected = "accepted list not written")]
fn kit_7_109_a_list_that_cannot_be_written_fails_its_gate() {
    let file = std::env::temp_dir().join(format!("qt-test-accepted-file-{}", std::process::id()));
    std::fs::write(&file, "a file where the folder should be").unwrap();
    write_accepted(&file, "gate", &["a line".to_owned()]);
}

#[test]
fn kit_7_109_each_test_program_lists_its_count_of_the_main_thread_line_under_its_ruling() {
    use emusen_qt_test::excuse::{tally_tests_only, tests_only_list};
    let dir = std::env::temp_dir().join(format!("qt-test-tests-only-{}", std::process::id()));
    let line = "QApplication was not created in the main() thread";
    tally_tests_only(&dir, "emusen-qt::acceptance", "run-1", "one", &[("R-20261111-110", line, 1)]);
    tally_tests_only(&dir, "emusen-qt::acceptance", "run-1", "two", &[("R-20261111-110", line, 2)]);
    tally_tests_only(&dir, "emusen-qt::acceptance", "run-1", "three", &[("R-20261111-110", line, 0)]);
    let file = tests_only_list(&dir, "emusen-qt::acceptance");
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        format!("run run-1\naccepted in tests, R-20261111-110: 3 × {line}\ntest one: 1 × {line}\ntest two: 2 × {line}\ntest three: 0 × {line}\n")
    );
    tally_tests_only(&dir, "emusen-qt::acceptance", "run-2", "one", &[("R-20261111-110", line, 1)]);
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        format!("run run-2\naccepted in tests, R-20261111-110: 1 × {line}\ntest one: 1 × {line}\n"),
        "a new run starts the list afresh"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn kit_7_109_a_harness_leaves_its_program_s_count_where_the_gates_list() {
    use emusen_qt_test::excuse::{accepted_dir, tests_only_list};
    use emusen_qt_test::harness::tests_only_process;
    let (program, run, _) = tests_only_process();
    drop(emusen_qt_test::Harness::new());
    let list = std::fs::read_to_string(tests_only_list(&accepted_dir(), &program)).expect("the program's list is written");
    assert!(list.starts_with(&format!("run {run}\naccepted in tests, R-20261111-110: ")), "{list}");
}
