// EmuSen_Galaxia.md §5.3a: a state forgotten, as when the player deletes it, takes the sidecar an earlier build wrote beside it.
use emusen_crystal::states;

#[test]
fn forgetting_a_state_removes_an_earlier_builds_sidecar_and_nothing_else() {
    let dir = std::env::temp_dir().join(format!("crystal-state-forget-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let state = dir.join("Cobalt Harbor (Synthetic).slot2.state");
    let other = dir.join("Cobalt Harbor (Synthetic).slot3.state");
    std::fs::write(&state, b"state").unwrap();
    std::fs::write(&other, b"state").unwrap();
    let (sidecar, kept) = (states::sidecar_path(state.to_str().unwrap()), states::sidecar_path(other.to_str().unwrap()));
    assert_eq!(sidecar, dir.join("Cobalt Harbor (Synthetic).slot2.json"));
    std::fs::write(&sidecar, b"{}").unwrap();
    std::fs::write(&kept, b"{}").unwrap();
    states::forget(None, state.to_str().unwrap()).unwrap();
    assert!(!sidecar.exists(), "the forgotten state's sidecar is gone");
    assert!(state.exists(), "the state itself is the front end's to delete");
    assert!(kept.exists() && other.exists(), "another state and its sidecar stay");
    // Forgetting again, with nothing left to remove, is no failure.
    states::forget(None, state.to_str().unwrap()).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
}
