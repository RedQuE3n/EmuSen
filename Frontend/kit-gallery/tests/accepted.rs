//! The gates' lists of accepted exemptions (EmuSen_Kit.md §7.102): one file a gate, so no gate's list overwrites another's.

use emusen_kit_gallery::accepted_list;

#[test]
fn kit_7_102_each_gate_s_accepted_list_survives_the_others() {
    let dir = std::env::temp_dir().join(format!("kit-gallery-accepted-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let gates = ["gate", "screens", "settings"];
    for gate in gates {
        std::fs::write(accepted_list(&dir, gate), format!("{gate}'s line")).unwrap();
    }
    for gate in gates {
        assert_eq!(std::fs::read_to_string(accepted_list(&dir, gate)).unwrap(), format!("{gate}'s line"), "{gate}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}
