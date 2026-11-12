//! The gates' lists of what a ruling accepted (EmuSen_Kit.md §7.102, §7.105): one file a gate in one folder, so no gate's
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
fn kit_7_105_a_list_that_cannot_be_written_fails_its_gate() {
    let file = std::env::temp_dir().join(format!("qt-test-accepted-file-{}", std::process::id()));
    std::fs::write(&file, "a file where the folder should be").unwrap();
    write_accepted(&file, "gate", &["a line".to_owned()]);
}
