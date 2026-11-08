//! What emusen-native's descriptor writer writes, checked against the schemas committed beside the core ABI's header.
//! See EmuSen_RustPlatform.md §47.22.

use emusen_core_schema::{matches, validate};
use emusen_native::core::schema;
use emusen_native::core::sys::{caps, flags};

// §6.2's replacement object and §6.4's firmware sources, written and validated.
#[test]
fn a_replacement_and_a_firmware_source_are_written_as_the_schemas_say() {
    use emusen_native::core::desc::{Firmware, FirmwareSource, MachineInfo, Replacement};
    let f = Firmware { which: 2, name: "a.rom".into(), label: "A".into(), size: 8, replacement: Some(Replacement::Accuracy { cost: "slower".into() }), ..Firmware::default() };
    let none = Firmware { replacement: Some(Replacement::None { cost: "runs without the chip".into() }), ..f.clone() };
    let doc = emusen_native::core::desc::firmware_json(&[f, none, Firmware { which: 3, name: "b.rom".into(), ..Firmware::default() }]);
    assert_eq!(validate(schema::FIRMWARE, &doc), Vec::<String>::new());
    assert!(doc.contains(r#""replacement":{"effect":"accuracy","cost":"slower"}"#), "{doc}");
    assert!(doc.contains(r#""effect":"none""#) && doc.matches("replacement").count() == 2, "{doc}");
    let m = MachineInfo { firmware: vec![(2, FirmwareSource::Replacement), (3, FirmwareSource::File)], ..MachineInfo::default() };
    let measured = emusen_native::core::desc::Measured { space_size: &|_| None, battery_len: &|_| None, capabilities: 0, picture_settles: 0, raw: &[], units: &[], telemetry: &[], named: &[], audio_rate_exact: None, snapshot_offered: true, outputs: &[] };
    let mi = emusen_native::core::desc::machine_info_json(&m, &measured, &[]);
    assert!(mi.contains(r#""firmware":[{"which":2,"source":"replacement"},{"which":3,"source":"file"}]"#), "{mi}");
    assert_eq!(validate(schema::MACHINE_INFO, &mi), Vec::<String>::new());
}

// Since 1.19 an entry's need is written only when the core gives one, so no earlier answer changes by a byte (§6.44).
#[test]
fn a_firmware_entrys_need_is_written_only_when_given_and_alternatives_of_any_size_share_it() {
    use emusen_native::core::desc::Firmware;
    let small = Firmware { which: 2, name: "a.rom".into(), label: "A".into(), size: 8, need: "bios".into(), ..Firmware::default() };
    let large = Firmware { which: 3, name: "b.rom".into(), size: 16, ..small.clone() };
    let own = Firmware { which: 4, name: "c.rom".into(), label: "C".into(), size: 8, ..Firmware::default() };
    let doc = emusen_native::core::desc::firmware_json(&[small, large, own]);
    assert_eq!(validate(schema::FIRMWARE, &doc), Vec::<String>::new());
    assert_eq!(doc.matches(r#""need":"bios""#).count(), 2, "{doc}");
    assert_eq!(doc.matches(r#""need""#).count(), 2, "{doc}");
}

// Since 1.12 an entry's md5 list is written only when it has a checksum, so no earlier answer changes by a byte (§6.32).
#[test]
fn a_firmware_entrys_md5_list_is_written_only_when_not_empty() {
    use emusen_native::core::desc::Firmware;
    let plain = Firmware { which: 1, name: "a.rom".into(), label: "A".into(), size: 8, ..Firmware::default() };
    let listed = Firmware { md5: vec!["0123456789abcdef0123456789abcdef".into(), "fedcba9876543210fedcba9876543210".into()], ..plain.clone() };
    let doc = emusen_native::core::desc::firmware_json(&[plain, listed]);
    assert_eq!(validate(schema::FIRMWARE, &doc), Vec::<String>::new());
    assert_eq!(doc.matches("\"md5\"").count(), 1, "{doc}");
    assert!(doc.contains(r#""md5":["0123456789abcdef0123456789abcdef","fedcba9876543210fedcba9876543210"]"#), "{doc}");
    assert!(!validate(schema::FIRMWARE, &doc.replace("0123456789abcdef0123", "0123456789ABCDEF0123")).is_empty());
}

// Since 1.5 a bound of settling pictures is written under the state when it is not 0, within the schema's range, which is the crate's maximum.
#[test]
fn a_picture_settling_bound_is_written_under_the_state_only_when_declared() {
    use emusen_native::core::desc::{MachineInfo, PICTURE_SETTLES_MAX};
    let m = MachineInfo::default();
    let of = |n: u32| emusen_native::core::desc::machine_info_json(&m, &emusen_native::core::desc::Measured { space_size: &|_| None, battery_len: &|_| None, capabilities: 0, picture_settles: n, raw: &[], units: &[], telemetry: &[], named: &[], audio_rate_exact: None, snapshot_offered: true, outputs: &[] }, &[]);
    assert!(!of(0).contains("picture_settles"), "{}", of(0));
    assert!(of(240).contains(r#""loads_from":[],"picture_settles":240}"#), "{}", of(240));
    assert_eq!(validate(schema::MACHINE_INFO, &of(PICTURE_SETTLES_MAX)), Vec::<String>::new());
    assert_eq!(validate(schema::MACHINE_INFO, &of(PICTURE_SETTLES_MAX + 1)), vec![format!("$.state.picture_settles: {} is above {PICTURE_SETTLES_MAX}", PICTURE_SETTLES_MAX + 1)]);
}

// A view is written as one more of a space's flags, which the schema leaves open, and a space without it is as before.
#[test]
fn a_view_is_a_flag_of_its_space() {
    use emusen_native::core::desc::{MachineInfo, Space};
    let m = MachineInfo { spaces: vec![Space { read_only: true, view: true, ..Space::new(0, "CPUBUS") }, Space::new(1, "WRAM")], ..MachineInfo::default() };
    let measured = emusen_native::core::desc::Measured { space_size: &|id| Some(if id == 0 { 1 << 24 } else { 1 << 16 }), battery_len: &|_| None, capabilities: 0, picture_settles: 0, raw: &[], units: &[], telemetry: &[], named: &[], audio_rate_exact: None, snapshot_offered: true, outputs: &[] };
    let mi = emusen_native::core::desc::machine_info_json(&m, &measured, &[]);
    assert!(mi.contains(r#"{"id":0,"name":"CPUBUS","size":16777216,"flags":["read_only","view"]}"#), "{mi}");
    assert!(mi.contains(r#"{"id":1,"name":"WRAM","size":65536,"flags":[]}"#), "{mi}");
    assert_eq!(validate(schema::MACHINE_INFO, &mi), Vec::<String>::new());
}

// Since 1.16: named files after the numbered ones, the exact rate, a withdrawn snapshot and an unrated setting, each valid.
#[test]
fn the_additions_of_1_16_are_written_and_valid() {
    use emusen_native::core::desc::{Battery, BatteryKind, Effect, MachineInfo, NamedFile, Scope, Setting, SettingKind, Choice};
    let m = MachineInfo { battery: vec![Battery::game(0, ".srm")], ..MachineInfo::default() };
    let named = [NamedFile { which: flags::FILE_NAMED, name: "save/notes.txt".into(), kind: BatteryKind::Console, removed: false }, NamedFile { which: flags::FILE_NAMED + 1, name: "save/old.txt".into(), kind: BatteryKind::Console, removed: true }];
    let measured = emusen_native::core::desc::Measured { space_size: &|_| None, battery_len: &|w| Some(w as i64 & 0xF), capabilities: caps::SNAPSHOT, picture_settles: 0, raw: &[], units: &[], telemetry: &[], named: &named, audio_rate_exact: Some((64_081, 2)), snapshot_offered: false, outputs: &[] };
    let mi = emusen_native::core::desc::machine_info_json(&m, &measured, &[]);
    assert!(mi.contains(r#""rate_exact":{"num":64081,"den":2}"#), "{mi}");
    assert!(mi.contains(r#"{"which":32768,"suffix":"","name":"save/notes.txt","length":0,"kind":"console"}"#), "{mi}");
    assert!(mi.contains(r#""name":"save/old.txt","length":1,"kind":"console","removed":true}"#), "{mi}");
    assert!(mi.contains(r#""kinds":[0]"#), "{mi}");
    assert_eq!(validate(schema::MACHINE_INFO, &mi), Vec::<String>::new());
    let unrated = Setting { key: "core_option".into(), label: "An option".into(), help: String::new(), kind: SettingKind::Choice(vec![Choice::new("a", "A")]), default: "a".into(), scope: Scope::Run, category: None, effect: Effect::Unrated, advanced: false, hidden: false, restart: false };
    assert_eq!(unrated.check(), Ok(()));
    let text = emusen_native::core::desc::settings_json(&[unrated]);
    assert!(text.contains(r#""effect":"unrated""#), "{text}");
    assert_eq!(validate(schema::SETTINGS, &text), Vec::<String>::new());
}

// Since 1.14 a space's byte order is written only where the core states one, so no earlier machine info changes by a byte (§6.39).
#[test]
fn a_spaces_byte_order_is_written_only_when_stated() {
    use emusen_native::core::desc::{BYTE_ORDER_BIG, BYTE_ORDER_LITTLE, MachineInfo, Space};
    let m = MachineInfo { spaces: vec![Space::new(0, "WRAM"), Space { byte_order: Some(BYTE_ORDER_LITTLE), ..Space::new(1, "Z80RAM") }, Space { byte_order: Some(BYTE_ORDER_BIG), ..Space::new(2, "VRAM") }], ..MachineInfo::default() };
    let measured = emusen_native::core::desc::Measured { space_size: &|_| Some(8), battery_len: &|_| None, capabilities: 0, picture_settles: 0, raw: &[], units: &[], telemetry: &[], named: &[], audio_rate_exact: None, snapshot_offered: true, outputs: &[] };
    let mi = emusen_native::core::desc::machine_info_json(&m, &measured, &[]);
    assert!(mi.contains(r#"{"id":0,"name":"WRAM","size":8,"flags":[]}"#), "{mi}");
    assert!(mi.contains(r#"{"id":1,"name":"Z80RAM","size":8,"flags":[],"byte_order":"little"}"#), "{mi}");
    assert!(mi.contains(r#"{"id":2,"name":"VRAM","size":8,"flags":[],"byte_order":"big"}"#), "{mi}");
    assert_eq!(validate(schema::MACHINE_INFO, &mi), Vec::<String>::new());
    assert_eq!(validate(schema::MACHINE_INFO, &mi.replace(r#""byte_order":"big""#, r#""byte_order":"Big""#)), vec![r#"$.spaces[2].byte_order: "Big" is not one of the enumeration"#]);
}

#[test]
fn a_schema_violation_is_reported_by_path() {
    assert_eq!(validate(schema::FIRMWARE, r#"[{"which":0,"name":"a","label":"b","size":1}]"#), vec!["$[0]: required is required", "$[0].which: 0 is below 1"]);
    assert_eq!(validate(schema::INFO, r#"{"abi":"1.0","id":"Bad Id","name":"","version":"","license":"","systems":[],"capabilities":[]}"#), vec![r#"$.id: "Bad Id" does not match ^[a-z0-9-]+$"#]);
    assert_eq!(matches("^[0-9]+[.][0-9]+$", "1.10"), Some(true));
    assert_eq!(matches("^a.b$", "a.b"), None);
}

// Since 1.8 (EmuSen_CoreAPI.md §6.28): a slot's port is written as a number, with its medium and label, and a file that sets none writes 1.7's form.
#[test]
fn a_slots_medium_label_and_port_are_written_only_where_set() {
    use emusen_native::core::desc::{Battery, Info, System, info_json};
    let info = Info { id: "x".into(), systems: vec![System { id: "n64".into(), ..Default::default() }], ..Default::default() };
    let text = info_json(&info, 0, &|_| Some(vec![Battery::game(0, ".eep"), Battery::removable(1, ".2.mpk").medium("n64.pak").label("Controller 2").port(1)]), &|_| Vec::new(), &|_| false);
    assert!(text.contains(r#""battery":[{"which":0,"suffix":".eep"},{"which":1,"suffix":".2.mpk","kind":"removable","medium":"n64.pak","label":"Controller 2","port":1}]"#), "{text}");
    assert_eq!(validate(schema::INFO, &text), Vec::<String>::new());
    let bad = info_json(&info, 0, &|_| Some(vec![Battery::console(0, ".brm").medium("Sega CD")]), &|_| Vec::new(), &|_| false);
    assert!(!validate(schema::INFO, &bad).is_empty(), "a medium outside [a-z0-9.]+ is refused by the schema");
}

// The writer's text is JSON a reader takes back to the values written (emusen-native's jsonw.rs pins its bytes).
#[test]
fn the_writers_text_reads_back_as_written() {
    let mut j = emusen_native::core::jsonw::Json::new();
    j.begin_object().field_str("a", "x\"\\\n\u{1}é").key("b").begin_array().int(-1).uint(2).bool(true).null().begin_object().end_object().end_array().field_strs("c", &["p", "q"]).end_object();
    let read: serde_json::Value = serde_json::from_str(&j.finish()).unwrap();
    assert_eq!(read, serde_json::json!({"a": "x\"\\\n\u{1}é", "b": [-1, 2, true, null, {}], "c": ["p", "q"]}));
}
