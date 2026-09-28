//! The shared naming rule (`emusen_native::naming`) over MoonRT's modules: every label and comment names the field on its line.

const SOURCES: [(&str, &str); 8] = [
    ("apu/channels.rs", include_str!("apu/channels.rs")),
    ("apu/mod.rs", include_str!("apu/mod.rs")),
    ("cpu/mod.rs", include_str!("cpu/mod.rs")),
    ("machine.rs", include_str!("machine.rs")),
    ("memory/bus.rs", include_str!("memory/bus.rs")),
    ("memory/cartridge.rs", include_str!("memory/cartridge.rs")),
    ("memory/mappers.rs", include_str!("memory/mappers.rs")),
    ("ppu/mod.rs", include_str!("ppu/mod.rs")),
];

#[test]
fn every_label_and_comment_names_the_field_on_its_line() {
    let found = emusen_native::naming::check(&SOURCES, &[], &["self."]);
    assert!(found.wrong.is_empty(), "{}", found.wrong.join("\n"));
    assert!(found.writes > 150 && found.reads > 150, "only {} writes and {} reads were checked", found.writes, found.reads);
}
