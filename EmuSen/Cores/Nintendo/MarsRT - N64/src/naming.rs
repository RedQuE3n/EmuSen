//! The shared naming rule (`emusen_native::naming`) over MarsRT's modules: every label and comment names the field on its line.

const SOURCES: [(&str, &str); 15] = [
    ("memory/ai.rs", include_str!("memory/ai.rs")),
    ("memory/bus.rs", include_str!("memory/bus.rs")),
    ("memory/controller.rs", include_str!("memory/controller.rs")),
    ("cpu/mod.rs", include_str!("cpu/mod.rs")),
    ("memory/dp.rs", include_str!("memory/dp.rs")),
    ("memory/isviewer.rs", include_str!("memory/isviewer.rs")),
    ("machine.rs", include_str!("machine.rs")),
    ("memory/mi.rs", include_str!("memory/mi.rs")),
    ("memory/pi.rs", include_str!("memory/pi.rs")),
    ("rdp/mod.rs", include_str!("rdp/mod.rs")),
    ("memory/save.rs", include_str!("memory/save.rs")),
    ("memory/si.rs", include_str!("memory/si.rs")),
    ("memory/sp.rs", include_str!("memory/sp.rs")),
    ("cpu/tlb.rs", include_str!("cpu/tlb.rs")),
    ("vi/mod.rs", include_str!("vi/mod.rs")),
];

/// Named on purpose: `SaveChip.Type` is `kind`, `type` being a keyword; a snapshot's words are no C# field.
const RENAMED: [(&str, &str); 2] = [("Type", "kind"), ("Words", "pending")];

#[test]
fn every_label_and_comment_names_the_field_on_its_line() {
    let found = emusen_native::naming::check(&SOURCES, &RENAMED, &["self.", "machine."]);
    assert!(found.wrong.is_empty(), "{}", found.wrong.join("\n"));
    assert!(found.writes > 200 && found.reads > 200, "only {} writes and {} reads were checked", found.writes, found.reads);
}
