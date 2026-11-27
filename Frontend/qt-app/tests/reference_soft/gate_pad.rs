//! The gates' in-process pad (EmuSen_Qt.md §9.3.6): absent from a build without the `gate-pad` feature, which is every
//! build a player gets; with it, the FIFO's lines read as presses.

/// The words only the gate pad puts in the program: its variable, SDL's call that makes a virtual pad, and the pad's name.
const WORDS: [&str; 3] = ["EMUSEN_GATE_PAD", "SDL_AttachVirtualJoystick", "EmuSen gate pad"];

/// Which of `WORDS` the program built for this test holds.
fn words_in_program() -> Vec<&'static str> {
    let program = std::fs::read(env!("CARGO_BIN_EXE_emusen-qt")).expect("the program built for this test");
    WORDS.into_iter().filter(|word| program.windows(word.len()).any(|w| w == word.as_bytes())).collect()
}

#[test]
fn the_feature_is_not_a_default_one() {
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")).unwrap();
    let default = manifest.lines().find(|l| l.trim_start().starts_with("default")).unwrap_or("");
    assert!(!default.contains("gate-pad"), "a player's build would carry the gate pad: {default}");
}

#[cfg(not(feature = "gate-pad"))]
#[test]
fn a_build_without_the_feature_has_no_gate_pad() {
    assert_eq!(words_in_program(), Vec::<&str>::new(), "in a build without gate-pad");
}

#[cfg(feature = "gate-pad")]
#[test]
fn the_gate_pad_is_built_with_the_feature_and_reads_its_lines() {
    use emusen_kit_core::input::PadControl;
    use emusen_qt::gate_pad::parse;
    assert_eq!(words_in_program(), WORDS, "in a build with gate-pad");
    assert_eq!(parse("Right 1"), Some((PadControl::Right, true)));
    assert_eq!(parse("Guide 0"), Some((PadControl::Guide, false)));
    assert_eq!(parse("right 1"), None, "names as PadControl spells them");
    assert_eq!(parse("Right"), None);
    assert_eq!(parse("Right 2"), None);
    assert_eq!(parse("Right 1 x"), None);
}
