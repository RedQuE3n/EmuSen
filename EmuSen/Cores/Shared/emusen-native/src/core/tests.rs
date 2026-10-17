//! `core_exports!` proven on the test core, through its C exports as a host calls them.

#[path = "../../examples/v1_test_core.rs"]
#[allow(dead_code)]
mod test_core;

use std::ptr::{null, null_mut};
use std::sync::Mutex;

use super::sys::*;
use super::{Core, schema};
use test_core::*;

/// The library log is one per process; tests that read it take turns.
static LIBRARY: Mutex<()> = Mutex::new(());

fn bytes(f: impl Fn(*mut u8, usize) -> i64) -> Vec<u8> {
    let n = f(null_mut(), 0);
    assert!(n >= 0, "status {n}");
    let mut buf = vec![0u8; n as usize];
    assert_eq!(f(buf.as_mut_ptr(), buf.len()), n);
    buf
}

fn text(f: impl Fn(*mut u8, usize) -> i64) -> String {
    String::from_utf8(bytes(f)).unwrap()
}

fn params(image: &[u8], settings: &str, files: &[FileEntry]) -> CreateParams {
    CreateParams {
        size: std::mem::size_of::<CreateParams>() as u32,
        host_abi_version: ABI_VERSION,
        image: image.as_ptr(),
        image_len: image.len(),
        settings: settings.as_ptr(),
        settings_len: settings.len(),
        files: files.as_ptr(),
        file_count: files.len(),
        file_size: std::mem::size_of::<FileEntry>(),
        pixel_formats: 1,
        error: null_mut(),
        error_len: 0,
        flags: 0,
        device: std::ptr::null(),
        system: std::ptr::null(),
    }
}

fn make(settings: &str) -> *mut Machine {
    let mut code = 1;
    let img = image(&[1, 2, 3]);
    let m = unsafe { emusen_core_create(&params(&img, settings, &[]), &mut code) };
    assert_eq!(code, 0);
    assert!(!m.is_null());
    m
}

fn drain_all(m: *mut Machine) -> Vec<Event> {
    let n = unsafe { emusen_core_events(m, null_mut(), 0, 0) } as usize;
    let mut out = vec![Event::default(); n];
    assert_eq!(unsafe { emusen_core_events(m, out.as_mut_ptr(), n, std::mem::size_of::<Event>()) }, n as i64);
    out
}

/// The events waiting but the drive's places, which the test core names every frame and one test reads.
fn drain_events(m: *mut Machine) -> Vec<Event> {
    drain_all(m).into_iter().filter(|e| e.kind != event::MEDIA_SEEK).collect()
}

fn digest(m: *mut Machine) -> Vec<u8> {
    let mut state = vec![0u8; unsafe { emusen_core_state_size(m, 0) } as usize];
    assert!(unsafe { emusen_core_state_save(m, 0, state.as_mut_ptr(), state.len()) } > 0);
    state.extend(bytes(|o, l| unsafe { emusen_core_frame_copy(m, o, l) }));
    state
}

#[test]
fn every_descriptor_validates_against_its_committed_schema() {
    let m = make("Wide=true");
    let info = text(|o, l| unsafe { emusen_core_info(o, l) });
    let cases = [
        (schema::INFO, info.clone()),
        (schema::SETTINGS, text(|o, l| unsafe { emusen_core_settings_schema(o, l) })),
        (schema::FIRMWARE, text(|o, l| unsafe { emusen_core_firmware_for(image(&[0xB0]).as_ptr(), 7, o, l) })),
        (schema::MACHINE_INFO, text(|o, l| unsafe { emusen_core_machine_info(m, o, l) })),
        (schema::SETTING_NOTES, text(|o, l| unsafe { emusen_core_setting_notes(m, o, l) })),
        (schema::DISASSEMBLY, text(|o, l| unsafe { emusen_core_debug_disassemble(m, 0, 0, 0, 4, o, l) })),
    ];
    for (s, doc) in &cases {
        assert_eq!(schema::validate(s, doc), Vec::<String>::new(), "{doc}");
    }
    let v = crate::json::parse(info.as_bytes()).unwrap();
    assert_eq!(v.get("abi").and_then(|a| a.as_str()), Some("1.11"));
    let names: Vec<&str> = v.get("capabilities").unwrap().as_array().unwrap().iter().filter_map(|c| c.as_str()).collect();
    assert_eq!(names.len(), 23);
    assert!(
        info.contains(r#""battery":[{"which":0,"suffix":".srm"},{"which":1,"suffix":".card","kind":"removable","medium":"test.card","label":"Card slot"},{"which":2,"suffix":".mem","kind":"console","medium":"test.memory","label":"Inside the console"}]"#),
        "core info lists the system's battery files since 1.3, a removable one's kind since 1.7, a console one's and each slot's medium and label since 1.8, and nothing unset: {info}"
    );
    assert_eq!(super::desc::capability_names(unsafe { emusen_core_capabilities() }), names);
    let mi = crate::json::parse(cases[3].1.as_bytes()).unwrap();
    let spaces = mi.get("spaces").unwrap().as_array().unwrap();
    assert_eq!(spaces[0].get("size").and_then(|s| s.as_i64()), Some(unsafe { emusen_core_space_size(m, 0) }));
    assert_eq!(mi.get("battery").unwrap().as_array().unwrap()[0].get("length").and_then(|s| s.as_i64()), Some(16));
    assert_eq!(cases[4].1, r#"{"Wide":"The picture is drawn twice as wide."}"#);
    assert!(cases[2].1.contains("boot.rom"));
    unsafe { emusen_core_free(m) };
}

// §6.2's replacement object and §6.4's firmware sources, written and validated.
#[test]
fn a_replacement_and_a_firmware_source_are_written_as_the_schemas_say() {
    use super::desc::{Firmware, FirmwareSource, MachineInfo, Replacement};
    let f = Firmware { which: 2, name: "a.rom".into(), label: "A".into(), size: 8, replacement: Some(Replacement::Accuracy { cost: "slower".into() }), ..Firmware::default() };
    let none = Firmware { replacement: Some(Replacement::None { cost: "runs without the chip".into() }), ..f.clone() };
    let doc = super::desc::firmware_json(&[f, none, Firmware { which: 3, name: "b.rom".into(), ..Firmware::default() }]);
    assert_eq!(schema::validate(schema::FIRMWARE, &doc), Vec::<String>::new());
    assert!(doc.contains(r#""replacement":{"effect":"accuracy","cost":"slower"}"#), "{doc}");
    assert!(doc.contains(r#""effect":"none""#) && doc.matches("replacement").count() == 2, "{doc}");
    let m = MachineInfo { firmware: vec![(2, FirmwareSource::Replacement), (3, FirmwareSource::File)], ..MachineInfo::default() };
    let measured = super::desc::Measured { space_size: &|_| None, battery_len: &|_| None, capabilities: 0, picture_settles: 0, raw: &[] };
    let mi = super::desc::machine_info_json(&m, &measured, &[]);
    assert!(mi.contains(r#""firmware":[{"which":2,"source":"replacement"},{"which":3,"source":"file"}]"#), "{mi}");
    assert_eq!(schema::validate(schema::MACHINE_INFO, &mi), Vec::<String>::new());
}

// Since 1.5 a bound of settling pictures is written under the state when it is not 0, within the schema's range, which is the crate's maximum.
#[test]
fn a_picture_settling_bound_is_written_under_the_state_only_when_declared() {
    use super::desc::{MachineInfo, PICTURE_SETTLES_MAX};
    let m = MachineInfo::default();
    let of = |n: u32| super::desc::machine_info_json(&m, &super::desc::Measured { space_size: &|_| None, battery_len: &|_| None, capabilities: 0, picture_settles: n, raw: &[] }, &[]);
    assert!(!of(0).contains("picture_settles"), "{}", of(0));
    assert!(of(240).contains(r#""loads_from":[],"picture_settles":240}"#), "{}", of(240));
    assert_eq!(schema::validate(schema::MACHINE_INFO, &of(PICTURE_SETTLES_MAX)), Vec::<String>::new());
    assert_eq!(schema::validate(schema::MACHINE_INFO, &of(PICTURE_SETTLES_MAX + 1)), vec![format!("$.state.picture_settles: {} is above {PICTURE_SETTLES_MAX}", PICTURE_SETTLES_MAX + 1)]);
}

// A view is written as one more of a space's flags, which the schema leaves open, and a space without it is as before.
#[test]
fn a_view_is_a_flag_of_its_space() {
    use super::desc::{MachineInfo, Space};
    let m = MachineInfo { spaces: vec![Space { read_only: true, view: true, ..Space::new(0, "CPUBUS") }, Space::new(1, "WRAM")], ..MachineInfo::default() };
    let measured = super::desc::Measured { space_size: &|id| Some(if id == 0 { 1 << 24 } else { 1 << 16 }), battery_len: &|_| None, capabilities: 0, picture_settles: 0, raw: &[] };
    let mi = super::desc::machine_info_json(&m, &measured, &[]);
    assert!(mi.contains(r#"{"id":0,"name":"CPUBUS","size":16777216,"flags":["read_only","view"]}"#), "{mi}");
    assert!(mi.contains(r#"{"id":1,"name":"WRAM","size":65536,"flags":[]}"#), "{mi}");
    assert_eq!(schema::validate(schema::MACHINE_INFO, &mi), Vec::<String>::new());
}

#[test]
fn a_schema_violation_is_reported_by_path() {
    assert_eq!(schema::validate(schema::FIRMWARE, r#"[{"which":0,"name":"a","label":"b","size":1}]"#), vec!["$[0]: required is required", "$[0].which: 0 is below 1"]);
    assert_eq!(schema::validate(schema::INFO, r#"{"abi":"1.0","id":"Bad Id","name":"","version":"","license":"","systems":[],"capabilities":[]}"#), vec![r#"$.id: "Bad Id" does not match ^[a-z0-9-]+$"#]);
    assert_eq!(schema::matches("^[0-9]+[.][0-9]+$", "1.10"), Some(true));
    assert_eq!(schema::matches("^a.b$", "a.b"), None);
}

#[test]
fn the_test_cores_schema_keeps_the_accurate_default_rules() {
    for s in TestCore::settings_schema() {
        s.check().unwrap();
    }
    let mut bad = TestCore::settings_schema().remove(0);
    bad.default = "true".into();
    assert!(bad.check().unwrap_err().contains("hardware"));
}

#[test]
fn a_null_machine_is_null_everywhere() {
    let n = null_mut::<Machine>();
    let mut fi = FrameInfo { size: 56, ..FrameInfo::default() };
    unsafe {
        assert_eq!(emusen_core_advance(n, null_mut()), status::NULL);
        assert_eq!(emusen_core_free(n), status::NULL);
        assert_eq!(emusen_core_frame_count(n), status::NULL as i64);
        assert_eq!(emusen_core_frame_info(n, &mut fi), status::NULL);
        assert_eq!(emusen_core_state_size(n, 0), status::NULL as i64);
        assert_eq!(emusen_core_space_read(n, 0, 0, null_mut(), 0), status::NULL as i64);
        assert_eq!(emusen_core_events(n, null_mut(), 0, 0), status::NULL as i64);
        assert_eq!(emusen_core_machine_info(n, null_mut(), 0), status::NULL as i64);
        assert_eq!(emusen_core_debug_set(n, 0, i32::MIN, -1), status::NULL);
        assert_eq!(emusen_core_audio_rate(n), status::NULL);
        assert_eq!(emusen_core_set_settings(n, null(), 0), status::NULL);
    }
}

#[test]
fn create_refuses_with_a_status_and_nul_terminated_words() {
    let mut code = 0;
    let mut err = [0xAAu8; 12];
    let mut p = params(&[], "", &[]);
    (p.error, p.error_len) = (err.as_mut_ptr(), err.len());
    assert!(unsafe { emusen_core_create(&p, &mut code) }.is_null());
    assert_eq!(code, status::BAD_IMAGE, "since 1.11 an empty image names a system to start empty, or is refused before the core");
    assert_eq!(&err, b"the image i\0");
    assert!(unsafe { emusen_core_create(null(), &mut code) }.is_null());
    assert_eq!(code, status::NULL);
    let one = image(&[1]);
    let small = CreateParams { size: 80, ..params(&one, "", &[]) };
    assert!(unsafe { emusen_core_create(&small, &mut code) }.is_null());
    assert_eq!(code, status::BAD_STRUCT);
    let file = FileEntry { size: 24, which: 0, data: null(), len: 0 };
    let short_files = CreateParams { file_size: 16, ..params(&one, "", std::slice::from_ref(&file)) };
    assert!(unsafe { emusen_core_create(&short_files, &mut code) }.is_null());
    assert_eq!(code, status::BAD_STRUCT);
    let mut words = [0u8; 64];
    for (settings, want, says) in [("Nope=1", status::UNKNOWN_SETTING, "no setting Nope"), ("Ram=0", status::BAD_SETTING, "Ram=0 is not a whole number from 1 to 255"), ("Ram=2\nRam=3", status::BAD_SETTING, "Ram given twice"), ("junk", status::BAD_SETTING, "not key=value")] {
        let mut p = params(&one, settings, &[]);
        (p.error, p.error_len) = (words.as_mut_ptr(), words.len());
        assert!(unsafe { emusen_core_create(&p, &mut code) }.is_null());
        assert_eq!(code, want, "{settings}");
        let got = std::ffi::CStr::from_bytes_until_nul(&words).unwrap().to_str().unwrap();
        assert!(got.contains(says), "{got}");
    }
    let bad_file = FileEntry { size: 24, which: 5, data: [1u8].as_ptr(), len: 1 };
    let mut p = params(&one, "", std::slice::from_ref(&bad_file));
    (p.error, p.error_len) = (words.as_mut_ptr(), words.len());
    assert!(unsafe { emusen_core_create(&p, &mut code) }.is_null());
    assert_eq!((code, std::ffi::CStr::from_bytes_until_nul(&words).unwrap().to_str().unwrap()), (status::BAD_FILE, "file 5 is not this core's"));
    for (bytes, want) in [(b"garbage!".to_vec(), "not a test core image"), (one[..6].to_vec(), "the image is truncated")] {
        let mut p = params(&bytes, "", &[]);
        (p.error, p.error_len) = (words.as_mut_ptr(), words.len());
        assert!(unsafe { emusen_core_create(&p, &mut code) }.is_null());
        assert_eq!((code, std::ffi::CStr::from_bytes_until_nul(&words).unwrap().to_str().unwrap()), (status::BAD_IMAGE, want));
    }
}

#[test]
fn a_machine_made_with_no_settings_equals_one_made_with_every_default() {
    let defaults: String = TestCore::settings_schema().iter().map(|s| format!("{}={}\n", s.key, s.default)).collect();
    let (a, b) = (make(""), make(&defaults));
    for _ in 0..30 {
        unsafe {
            emusen_core_advance(a, null_mut());
            emusen_core_advance(b, null_mut());
        }
    }
    assert_eq!(digest(a), digest(b));
    unsafe {
        emusen_core_free(a);
        emusen_core_free(b);
    }
}

#[test]
fn changes_raise_their_events_and_the_queue_drains() {
    let m = make("");
    let first = drain_events(m);
    assert_eq!(first.iter().map(|e| (e.kind, e.a)).collect::<Vec<_>>(), vec![(event::LOG, 1)], "create's log record is announced");
    assert!(drain_events(m).is_empty());
    let t = b"Wide=true\nRate=48000";
    assert_eq!(unsafe { emusen_core_set_settings(m, t.as_ptr(), t.len()) }, 0);
    assert_eq!(drain_events(m).iter().map(|e| (e.kind, e.a, e.b)).collect::<Vec<_>>(), vec![(event::GEOMETRY, 16, 4)]);
    unsafe { emusen_core_set_buttons(m, 0, 1, 1) };
    unsafe { emusen_core_advance(m, null_mut()) };
    let kinds: Vec<(u32, i64)> = drain_events(m).iter().map(|e| (e.kind, e.a)).collect();
    assert_eq!(kinds, vec![(event::BATTERY, 0), (event::AUDIO_RATE, 48000)], "nothing was queued at the old rate");
    unsafe { emusen_core_free(m) };
}

#[test]
fn a_drain_never_crosses_a_rate_change_and_reports_its_rate() {
    let m = make("");
    unsafe { emusen_core_advance(m, null_mut()) };
    let t = b"Rate=48000";
    unsafe { emusen_core_set_settings(m, t.as_ptr(), t.len()) };
    unsafe { emusen_core_advance(m, null_mut()) };
    assert_eq!(unsafe { emusen_core_audio_buffered(m) }, 2 * (32000 / 60) + 2 * (48000 / 60));
    let mut out = vec![0i16; 4096];
    let mut rate = 0;
    assert_eq!(unsafe { emusen_core_audio_drain(m, out.as_mut_ptr(), out.len(), 4096, &mut rate) }, 2 * (32000 / 60));
    assert_eq!(rate, 32000);
    drain_events(m);
    assert_eq!(unsafe { emusen_core_audio_drain(m, out.as_mut_ptr(), out.len(), 4096, &mut rate) }, 2 * (48000 / 60));
    assert_eq!(rate, 48000);
    let mut peek = [0i16; 2];
    assert_eq!(unsafe { emusen_core_audio_peek(m, peek.as_mut_ptr(), 2) }, 0);
    unsafe { emusen_core_advance(m, null_mut()) };
    assert_eq!(drain_events(m).iter().map(|e| e.kind).collect::<Vec<_>>(), Vec::<u32>::new(), "the change was announced at the drain that crossed it");
    unsafe { emusen_core_free(m) };
}

#[test]
fn an_audio_rate_event_follows_the_drain_that_reaches_a_new_rate() {
    let m = make("");
    unsafe { emusen_core_advance(m, null_mut()) };
    let t = b"Rate=48000";
    unsafe { emusen_core_set_settings(m, t.as_ptr(), t.len()) };
    unsafe { emusen_core_advance(m, null_mut()) };
    drain_events(m);
    let mut out = vec![0i16; 4096];
    let mut rate = 0;
    unsafe { emusen_core_audio_drain(m, out.as_mut_ptr(), out.len(), 4096, &mut rate) };
    assert_eq!(drain_events(m).iter().map(|e| (e.kind, e.a)).collect::<Vec<_>>(), vec![(event::AUDIO_RATE, 48000)]);
    unsafe { emusen_core_free(m) };
}

#[test]
fn structs_below_their_size_are_refused_and_bytes_past_it_are_left_alone() {
    let m = make("Wide=true");
    let mut small = FrameInfo { size: 48, ..FrameInfo::default() };
    assert_eq!(unsafe { emusen_core_frame_info(m, &mut small) }, status::BAD_STRUCT);
    #[repr(C)]
    struct Bigger {
        info: FrameInfo,
        canary: u64,
    }
    let mut big = Bigger { info: FrameInfo { size: 64, ..FrameInfo::default() }, canary: 0xC0FFEE };
    unsafe { emusen_core_advance(m, null_mut()) };
    assert_eq!(unsafe { emusen_core_frame_info(m, &mut big.info) }, 0);
    assert_eq!((big.info.size, big.canary, big.info.width, big.info.stride, big.info.serial, big.info.aspect_num), (64, 0xC0FFEE, 16, 64, 1, 4));
    assert_eq!(big.info.bytes, unsafe { emusen_core_frame_copy(m, null_mut(), 0) });
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct BigEvent {
        e: Event,
        canary: u64,
    }
    let mut out = [BigEvent { e: Event { size: 32, ..Event::default() }, canary: 7 }; 2];
    assert_eq!(unsafe { emusen_core_events(m, out.as_mut_ptr() as *mut Event, 2, 20) }, status::BAD_STRUCT as i64);
    assert_eq!(unsafe { emusen_core_events(m, out.as_mut_ptr() as *mut Event, 2, 32) }, 2);
    assert_eq!((out[0].e.size, out[0].e.kind, out[0].canary), (32, event::LOG, 7));
    assert_eq!((out[1].e.size, out[1].e.kind, out[1].canary), (32, event::MEDIA_SEEK, 7), "the frame's place, its canary kept");
    unsafe { emusen_core_free(m) };
}

#[test]
fn state_saves_loads_and_refuses_with_words() {
    let m = make("");
    for _ in 0..5 {
        unsafe { emusen_core_advance(m, null_mut()) };
    }
    let size = unsafe { emusen_core_state_size(m, 0) };
    assert_eq!(unsafe { emusen_core_state_save(m, 0, null_mut(), 0) }, size, "a null out asks the length");
    let mut a = vec![0u8; size as usize];
    unsafe { emusen_core_state_save(m, 0, a.as_mut_ptr(), a.len()) };
    let mut snap = vec![0u8; unsafe { emusen_core_state_size(m, 1) } as usize];
    assert_eq!(unsafe { emusen_core_state_save(m, 1, snap.as_mut_ptr(), snap.len()) }, snap.len() as i64);
    assert_eq!(&snap[..4], b"TSN1");
    assert_eq!(unsafe { emusen_core_state_size(m, 2) }, status::NOT_SUPPORTED as i64);
    let mut short = [0u8; 8];
    assert_eq!(unsafe { emusen_core_state_save(m, 0, short.as_mut_ptr(), 8) }, status::BUFFER_TOO_SMALL as i64);
    unsafe { emusen_core_advance(m, null_mut()) };
    assert_eq!(unsafe { emusen_core_state_load(m, snap.as_ptr(), snap.len()) }, 0);
    let mut b = vec![0u8; size as usize];
    unsafe { emusen_core_state_save(m, 0, b.as_mut_ptr(), b.len()) };
    assert_eq!(a, b);
    assert_eq!(unsafe { emusen_core_state_load(m, b"XXXX".as_ptr(), 4) }, status::FOREIGN);
    assert_eq!(text(|o, l| unsafe { emusen_core_last_error(m, o, l) }), "not this core's state");
    assert!(text(|o, l| unsafe { emusen_core_state_layout(m, 1, o, l) }).contains("Snapshot"));
    unsafe { emusen_core_free(m) };
}

#[test]
fn settings_spaces_options_and_status_words_follow_the_header() {
    let m = make("");
    let t = b"Ram=4";
    assert_eq!(unsafe { emusen_core_set_settings(m, t.as_ptr(), t.len()) }, status::BAD_SETTING);
    assert_eq!(text(|o, l| unsafe { emusen_core_last_error(m, o, l) }), "Ram is read only when a game is loaded");
    let t = b"Wide=maybe";
    assert_eq!(unsafe { emusen_core_set_settings(m, t.as_ptr(), t.len()) }, status::BAD_SETTING);
    assert_eq!(unsafe { emusen_core_space_read(m, 0, 0, null_mut(), 9) }, 9, "a null out asks the length");
    assert_eq!(unsafe { emusen_core_space_read(m, 7, 0, null_mut(), 9) }, status::NO_SUCH_SPACE as i64);
    let mut ram = [0u8; 2];
    unsafe { emusen_core_space_read(m, 0, 0, ram.as_mut_ptr(), 2) };
    assert_eq!(ram, [7, 1], "the create-time default reached the machine and the payload follows it");
    assert_eq!(unsafe { emusen_core_space_write(m, 1, 0, [1u8].as_ptr(), 1) }, status::READ_ONLY as i64);
    assert_eq!(unsafe { emusen_core_set_options(m, !1) }, 0);
    unsafe { emusen_core_advance(m, null_mut()) };
    assert_eq!(bytes(|o, l| unsafe { emusen_core_frame_copy(m, o, l) })[0], 1, "a reserved option bit was not passed on");
    assert_eq!(text(|o, l| unsafe { emusen_core_status_text(STATUS_EMPTY_IMAGE, o, l) }), "the image is empty");
    assert_eq!(text(|o, l| unsafe { emusen_core_status_text(status::BAD_STRUCT, o, l) }), "a structure smaller than its version 1.0 size");
    assert_eq!(unsafe { emusen_core_status_text(-200, null_mut(), 0) }, status::NOT_SUPPORTED as i64);
    let quads = [0u32, 5, 0x99, flags::NO_COMPARE];
    assert_eq!(unsafe { emusen_core_set_cheat_pokes(m, quads.as_ptr(), 1) }, 1);
    unsafe { emusen_core_advance(m, null_mut()) };
    let mut b = [0u8];
    unsafe { emusen_core_space_read(m, 0, 5, b.as_mut_ptr(), 1) };
    assert_eq!(b[0], 0x99);
    unsafe { emusen_core_free(m) };
}

#[test]
fn logs_drain_whole_records_and_a_freed_machine_leaves_its_own_to_the_library() {
    let _turn = LIBRARY.lock().unwrap_or_else(|e| e.into_inner());
    unsafe { emusen_core_log_drain(null_mut(), vec![0u8; 1 << 20].as_mut_ptr(), 1 << 20) };
    let img = image(&[7; 77]);
    let mut code = 1;
    let m = unsafe { emusen_core_create(&params(&img, "", &[]), &mut code) };
    let record = "info\ttest\tcreated with 77 bytes\n";
    assert_eq!(unsafe { emusen_core_log_drain(m, null_mut(), 0) }, record.len() as i64);
    let mut small = [0u8; 4];
    assert_eq!(unsafe { emusen_core_log_drain(m, small.as_mut_ptr(), 4) }, 0, "only whole records");
    unsafe { emusen_core_free(m) };
    let library = text(|o, l| unsafe { emusen_core_log_drain(null_mut(), o, l) });
    assert_eq!(library.matches(record).count(), 1, "the freed machine's record moved to the library's queue: {library:?}");
    let mut q = super::outbox::LogQueue::default();
    for i in 0..super::outbox::LOG_CAPACITY + 3 {
        q.push(super::Level::Debug, "a\tb", &format!("{i}\nx"));
    }
    let mut out = vec![0u8; q.waiting()];
    assert_eq!(q.drain(&mut out), out.len());
    let lines: Vec<&str> = std::str::from_utf8(&out).unwrap().lines().collect();
    assert_eq!((lines[0], lines[1], lines.len()), ("warn\tlog\t3 records dropped", "debug\ta b\t3 x", super::outbox::LOG_CAPACITY + 1));
}

// Since 1.4: a drive's place reaches the host as one event, the drive and the file in `a` and the byte in `b`.
#[test]
fn a_drive_names_the_place_it_reads_next_in_one_event() {
    let m = make("");
    drain_all(m);
    let mut detail = 0;
    assert_eq!(unsafe { emusen_core_advance(m, &mut detail) }, 0);
    assert_eq!(unsafe { emusen_core_advance(m, &mut detail) }, 0);
    let named: Vec<(i64, i64)> = drain_all(m).iter().filter(|e| e.kind == event::MEDIA_SEEK).map(|e| (e.a, e.b)).collect();
    assert_eq!(named, vec![(flags::FILE_IMAGE as i64, 2), (flags::FILE_IMAGE as i64, 3)], "drive 0, the image, the byte each next frame reads");
    assert_eq!(unsafe { emusen_core_free(m) }, 0);
    let o = super::outbox::Outbox::default();
    {
        let _route = super::outbox::Route::to(&o);
        super::seek(3, Some(2), 0x1_2345_6789);
        super::seek(0, None, 7);
    }
    super::seek(1, None, 1);
    let got: Vec<(u32, i64, i64)> = o.borrow().events.iter().map(|e| (e.kind, e.a, e.b)).collect();
    assert_eq!(got, vec![(event::MEDIA_SEEK, 3 << 32 | (flags::FILE_MEDIA as i64 + 2), 0x1_2345_6789), (event::MEDIA_SEEK, flags::FILE_IMAGE as i64, 7)], "outside a machine's call nothing is queued");
}

#[test]
fn raising_an_event_allocates_nothing() {
    let mut o = super::outbox::Inner::default();
    let (at, room) = (o.events.as_slices().0.as_ptr(), o.events.capacity());
    assert!(room >= super::outbox::EVENT_CAPACITY);
    for i in 0..3 * super::outbox::EVENT_CAPACITY {
        o.push_event(event::MEDIA_SEEK, 0, i as i64);
        if i % 7 == 0 {
            o.events.pop_front();
        }
    }
    assert_eq!((o.events.capacity(), o.events.len()), (room, super::outbox::EVENT_CAPACITY));
    o.events.make_contiguous();
    assert_eq!(o.events.as_slices().0.as_ptr(), at, "the queue is where create put it");
}

#[test]
fn a_full_event_queue_drops_the_oldest_and_asks_for_everything_again() {
    let mut o = super::outbox::Inner::default();
    for i in 0..super::outbox::EVENT_CAPACITY + 10 {
        o.push_event(event::BATTERY, i as i64, 0);
    }
    assert_eq!(o.events.len(), super::outbox::EVENT_CAPACITY);
    assert_eq!(o.events.iter().filter(|e| e.kind == event::MACHINE_INFO).count(), 1);
    assert_eq!(o.events.back().map(|e| e.a), Some(super::outbox::EVENT_CAPACITY as i64 + 9));
}

#[test]
fn the_debugger_reports_the_processor_that_stopped_and_refuses_one_it_lacks() {
    let m = make("");
    let pairs = [0i32, 0];
    assert_eq!(unsafe { emusen_core_debug_set_breakpoints(m, 1, pairs.as_ptr(), 1) }, status::NOT_SUPPORTED);
    assert_eq!(unsafe { emusen_core_debug_set_breakpoints(m, 0, pairs.as_ptr(), 1) }, 0);
    let (mut proc_, mut pc, mut detail) = (9u32, 9u64, 9u64);
    assert_eq!(unsafe { emusen_core_debug_run_frame(m, 0, &mut proc_, &mut pc, &mut detail) }, crate::debug::stop::BREAKPOINT as i32);
    assert_eq!((proc_, pc, detail), (0, 0, 0));
    let mut regs = [0i64; 4];
    assert_eq!(unsafe { emusen_core_debug_registers(m, 0, regs.as_mut_ptr(), 4) }, 1);
    assert_eq!(unsafe { emusen_core_debug_registers(m, 3, regs.as_mut_ptr(), 4) }, status::NOT_SUPPORTED as i64);
    assert_eq!(unsafe { emusen_core_debug_set(m, crate::debug::flag::EACH, i32::MIN, -1) }, 0);
    assert_eq!(unsafe { emusen_core_debug_run_frame(m, 0, &mut proc_, &mut pc, &mut detail) }, (crate::debug::stop::EACH | crate::debug::stop::BREAKPOINT) as i32);
    unsafe { emusen_core_free(m) };
}

/// Range kinds 2 and 3 reach the read ranges of a core claiming `DEBUG_READS`; any kind above 3 is refused and changes
/// nothing, and the read log drains as (space, address, value, pc, kind) records.
#[test]
fn read_ranges_reach_a_core_that_claims_them_and_an_unknown_kind_is_refused() {
    use crate::debug::range;
    let m = make("");
    let t = [7u32, 0x10, 0x13];
    for kind in [range::WATCH, range::BREAK, range::READ_WATCH, range::READ_BREAK] {
        assert_eq!(unsafe { emusen_core_debug_set_ranges(m, kind, t.as_ptr(), 1) }, 0, "kind {kind}");
    }
    assert_eq!(unsafe { emusen_core_debug_set_ranges(m, 4, t.as_ptr(), 1) }, status::NOT_SUPPORTED);
    let h = unsafe { super::exports::core_of_mut::<TestCore>(m) }.unwrap().debug_hooks().unwrap();
    assert_eq!((h.read_watch_ranges.len(), h.read_break_ranges.len(), h.break_ranges.len()), (1, 1, 1));
    h.configure(crate::debug::flag::READS, i32::MIN, -1);
    crate::debug::Observer::read(h, 7, 0x12, 0xAB, 0x8000_0100);
    assert!(h.note_read_ahead(7, 0x13, 0xCD, 0x8000_0104));
    assert_eq!(unsafe { emusen_core_debug_reads(m, null_mut(), 0) }, 2);
    let mut out = [0u32; 10];
    assert_eq!(unsafe { emusen_core_debug_reads(m, out.as_mut_ptr(), out.len()) }, 2);
    assert_eq!(out, [7, 0x12, 0xAB, 0x8000_0100, range::READ_WATCH, 7, 0x13, 0xCD, 0x8000_0104, range::READ_BREAK]);
    assert_eq!(unsafe { emusen_core_debug_reads(m, null_mut(), 0) }, 0, "drained");
    unsafe { emusen_core_free(m) };
}

#[test]
fn the_crates_constants_are_the_headers_own_numbers() {
    assert_eq!(ABI_VERSION, 0x0001_000B);
    assert_eq!((std::mem::size_of::<CreateParams>(), std::mem::offset_of!(CreateParams, system)), (112, 104), "since 1.11");
    assert_eq!(caps::DEVICE_PICTURE, 1 << 21, "since 1.9");
    assert_eq!((caps::RAW_PICTURE, caps::UNASSIGNED & caps::RAW_PICTURE, std::mem::size_of::<RawFrame>(), std::mem::offset_of!(RawFrame, bytes)), (1 << 22, 0, 32, 24), "since 1.10");
    assert_eq!(flags::CREATE_SHARED_REMOVABLE, 2, "since 1.7, beside CREATE_LENT");
    assert_eq!(flags::CREATE_SHARED_CONSOLE, 4, "since 1.8, beside CREATE_SHARED_REMOVABLE");
    assert_eq!((event::MEDIA_SEEK, flags::FILE_IMAGE, crate::core::SEEK_SPAN), (8, 0xFFFF_FFFF, 1 << 20));
    assert_eq!((caps::BORROW, caps::UNASSIGNED & caps::BORROW, flags::CREATE_LENT, flags::MEDIA_LENT), (1 << 20, 0, 1, 2));
    assert_eq!((CreateParams::SIZE_1_0, std::mem::offset_of!(CreateParams, device), std::mem::size_of::<CreateParams>()), (88, 96, 112), "the device lent at 1.9 follows 1.3's flags and their padding, and 1.11's system follows it");
    assert_eq!((status::BAD_STRUCT, status::BAD_IMAGE, status::STATE_VERSION), (-263, -264, -4));
    assert_eq!((status::LID_CLOSED, status::WRONG_MEDIA, status::NO_SUCH_DRIVE, flags::FILE_MEDIA, flags::MEDIA_RESTORE), (-265, -266, -267, 65_536, 1));
    assert_eq!(EXPORTS.len(), 62);
    assert_eq!(EXPORTS.iter().filter(|e| e.1 == 0).count(), 32);
    assert_eq!(CAPABILITY_NAMES.len(), 23);
    assert_eq!((caps::MEDIA, caps::DEBUG_READS, caps::UNASSIGNED & (caps::MEDIA | caps::DEBUG_READS)), (1 << 18, 1 << 19, 0));
    assert_eq!(super::EXPORTING & !(caps::SNAPSHOT | caps::FRAME_SERIAL | caps::ROW_REPEAT | caps::BATTERY_DIRTY), super::EXPORTING);
}

#[test]
fn an_extension_reaches_the_core_behind_a_handle_and_null_is_none() {
    let m = make("");
    let mut detail = 0;
    assert_eq!(unsafe { emusen_core_advance(m, &mut detail) }, 0);
    let core = unsafe { super::exports::core_of::<TestCore>(m) }.expect("a live handle");
    assert_eq!(core.frame_count(), unsafe { emusen_core_frame_count(m) });
    assert!(unsafe { super::exports::core_of_mut::<TestCore>(m) }.is_some());
    assert!(unsafe { super::exports::core_of::<TestCore>(null()) }.is_none());
    assert!(unsafe { super::exports::core_of_mut::<TestCore>(null_mut()) }.is_none());
    unsafe { emusen_core_free(m) };
}

fn media(image: Option<&[u8]>, flags: u32) -> Media {
    Media {
        size: std::mem::size_of::<Media>() as u32,
        flags,
        image: image.map_or(null(), |i| i.as_ptr()),
        image_len: image.map_or(0, |i| i.len()),
        files: null(),
        file_count: 0,
        file_size: std::mem::size_of::<FileEntry>(),
    }
}

// §6.23, since 1.1: a disc goes in only with the lid open or for a state about to load; a state loads only over its disc.
#[test]
fn a_disc_is_changed_through_the_lid_and_a_state_keeps_to_its_disc() {
    let m = make("");
    let other = image(&[9, 9]);
    let lid = |m| text(|o, l| unsafe { emusen_core_machine_info(m, o, l) });
    assert!(lid(m).contains(r#""drives":[{"drive":0,"label":"Test drive","kind":"cd","lid":"closed","loaded":true}]"#), "{}", lid(m));
    assert_eq!(unsafe { emusen_core_media_insert(m, 0, &media(Some(&other), 0)) }, status::LID_CLOSED);
    assert_eq!(unsafe { emusen_core_media_lid(m, 1, 1) }, status::NO_SUCH_DRIVE);
    assert_eq!(unsafe { emusen_core_media_lid(m, 0, 2) }, status::NOT_SUPPORTED);
    let state = |m: *mut Machine| {
        let mut s = vec![0u8; unsafe { emusen_core_state_size(m, 0) } as usize];
        assert!(unsafe { emusen_core_state_save(m, 0, s.as_mut_ptr(), s.len()) } > 0);
        s
    };
    let first = state(m);
    assert_eq!(unsafe { emusen_core_media_lid(m, 0, 1) }, 0);
    assert_eq!(unsafe { emusen_core_media_insert(m, 0, &media(Some(&other), 0)) }, 0);
    assert_eq!(unsafe { emusen_core_media_lid(m, 0, 0) }, 0);
    let second = digest(m);
    assert_eq!(unsafe { emusen_core_state_load(m, first.as_ptr(), first.len()) }, status::WRONG_MEDIA, "the state was made on the first disc");
    assert_eq!(digest(m), second, "a refused load changes nothing");
    assert_eq!(unsafe { emusen_core_media_insert(m, 0, &media(Some(&image(&[1, 2, 3])), flags::MEDIA_RESTORE)) }, 0, "restore puts a disc in with the lid closed");
    assert_eq!(unsafe { emusen_core_state_load(m, first.as_ptr(), first.len()) }, 0);
    assert_eq!(unsafe { emusen_core_media_lid(m, 0, 1) }, 0);
    assert_eq!(unsafe { emusen_core_media_insert(m, 0, &media(None, 0)) }, 0);
    assert!(lid(m).contains(r#""lid":"open","loaded":false"#), "{}", lid(m));
    let mut short = media(None, 0);
    short.size = 8;
    assert_eq!(unsafe { emusen_core_media_insert(m, 0, &short) }, status::BAD_STRUCT);
    unsafe { emusen_core_free(m) };
}

// §6.26, since 1.6: a MEDIA core's machine with no drive writes no `drives`, and each media call is refused with
// NO_SUCH_DRIVE after the refusals of a malformed call, the machine unchanged.
#[test]
fn a_media_machine_with_no_drive_lists_none_and_refuses_every_media_call() {
    let mut code = 1;
    let img = image(&[CARTRIDGE, 2, 3]);
    let m = unsafe { emusen_core_create(&params(&img, "", &[]), &mut code) };
    assert_eq!(code, 0);
    let info = text(|o, l| unsafe { emusen_core_machine_info(m, o, l) });
    assert!(!info.contains("drives"), "{info}");
    let before = digest(m);
    for drive in [0, 1, u32::MAX] {
        assert_eq!(unsafe { emusen_core_media_lid(m, drive, 1) }, status::NO_SUCH_DRIVE);
        assert_eq!(unsafe { emusen_core_media_lid(m, drive, 0) }, status::NO_SUCH_DRIVE);
        assert_eq!(unsafe { emusen_core_media_insert(m, drive, &media(Some(&image(&[9, 9])), 0)) }, status::NO_SUCH_DRIVE);
        assert_eq!(unsafe { emusen_core_media_insert(m, drive, &media(None, flags::MEDIA_RESTORE)) }, status::NO_SUCH_DRIVE);
    }
    assert_eq!(unsafe { emusen_core_media_lid(m, 0, 2) }, status::NOT_SUPPORTED);
    let mut short = media(None, 0);
    short.size = 8;
    assert_eq!(unsafe { emusen_core_media_insert(m, 0, &short) }, status::BAD_STRUCT);
    assert_eq!((digest(m), text(|o, l| unsafe { emusen_core_machine_info(m, o, l) })), (before, info));
    let mut detail = 0;
    assert_eq!(unsafe { emusen_core_advance(m, &mut detail) }, 0, "the machine runs on");
    assert!(drain_all(m).iter().all(|e| e.kind != event::MEDIA_SEEK), "a machine with no drive names no place");
    unsafe { emusen_core_free(m) };
}

/// The byte of its kept image the test core read at its last frame.
fn rom_byte(m: *mut Machine) -> u8 {
    let mut b = [0u8];
    assert_eq!(unsafe { emusen_core_space_read(m, 0, 254, b.as_mut_ptr(), 1) }, 1);
    b[0]
}

fn advance(m: *mut Machine) {
    assert_eq!(unsafe { emusen_core_advance(m, null_mut()) }, 0);
}

// §6.24, since 1.3: a lent image is read where the host has it, and a run from it is the run from a copy.
#[test]
fn a_lent_image_is_read_in_place_and_runs_as_its_copy() {
    let mut code = 1;
    let mut lent_bytes = image(&(0..200).collect::<Vec<u8>>());
    let copied_bytes = lent_bytes.clone();
    let lent = unsafe { emusen_core_create(&CreateParams { flags: flags::CREATE_LENT, ..params(&lent_bytes, "", &[]) }, &mut code) };
    assert_eq!(code, 0);
    let copied = unsafe { emusen_core_create(&params(&copied_bytes, "", &[]), &mut code) };
    assert_eq!(code, 0);
    assert_eq!(digest(lent), digest(copied));
    for frame in 1..=450usize {
        advance(lent);
        advance(copied);
        assert_eq!(rom_byte(lent), copied_bytes[frame % copied_bytes.len()], "frame {frame} read its own byte of the image");
        assert_eq!(digest(lent), digest(copied), "frame {frame}");
    }
    assert_eq!(bytes(|o, l| unsafe { emusen_core_audio_drain(lent, o as *mut i16, l / 2, 1 << 20, null_mut()) * 2 }).len(), bytes(|o, l| unsafe { emusen_core_audio_drain(copied, o as *mut i16, l / 2, 1 << 20, null_mut()) * 2 }).len());
    // Not a host's right, since a lent image does not change: done here to see which machine reads the host's bytes.
    let mut copy_changed = copied_bytes.clone();
    let at = 451 % lent_bytes.len();
    (lent_bytes[at], copy_changed[at]) = (lent_bytes[at] ^ 0xFF, copy_changed[at] ^ 0xFF);
    advance(lent);
    advance(copied);
    assert_eq!((rom_byte(lent), rom_byte(copied)), (lent_bytes[at], copied_bytes[at]), "the lent machine reads in place, the other its copy");
    drop(copy_changed);
    assert_eq!((unsafe { emusen_core_free(lent) }, unsafe { emusen_core_free(copied) }), (0, 0));
}

// §4.2's size rule, used for the first time by 1.3's `flags`: a 1.0 host's 88 bytes are taken and nothing past them
// read; a later host's longer struct is taken and its tail ignored; one byte short of 1.0 is refused.
#[test]
fn create_params_of_an_older_and_a_newer_host_are_read_as_far_as_both_know_them() {
    let mut code = 1;
    let mut bytes = image(&[7; 40]);
    // The flag's own bytes say LENT, but an 88-byte struct ends before them: the core must copy.
    let older = CreateParams { size: 88, flags: flags::CREATE_LENT, ..params(&bytes, "", &[]) };
    let m = unsafe { emusen_core_create(&older, &mut code) };
    assert_eq!(code, 0);
    bytes[1] ^= 0xFF;
    advance(m);
    assert_eq!(rom_byte(m), bytes[1] ^ 0xFF, "an older host lends nothing");
    assert_eq!(unsafe { emusen_core_free(m) }, 0);
    bytes[1] ^= 0xFF;

    #[repr(C)]
    struct Newer {
        params: CreateParams,
        later: [u64; 4],
    }
    let newer = Newer { params: CreateParams { size: std::mem::size_of::<Newer>() as u32, ..params(&bytes, "", &[]) }, later: [u64::MAX; 4] };
    let m = unsafe { emusen_core_create(&newer.params, &mut code) };
    assert_eq!(code, 0);
    assert_eq!(unsafe { emusen_core_free(m) }, 0);

    let short = CreateParams { size: 87, ..params(&bytes, "", &[]) };
    assert!(unsafe { emusen_core_create(&short, &mut code) }.is_null());
    assert_eq!(code, status::BAD_STRUCT);
}

// §6.24: a loan stands until its drive's next successful insert or free, and a refused call lends nothing.
#[test]
fn a_lent_disc_is_let_go_when_its_drive_takes_another() {
    let mut code = 1;
    let (first, mut second, third) = (image(&[1; 30]), image(&[2; 30]), image(&[3; 30]));
    let m = unsafe { emusen_core_create(&CreateParams { flags: flags::CREATE_LENT, ..params(&first, "", &[]) }, &mut code) };
    assert_eq!(code, 0);
    let lent = flags::MEDIA_LENT;
    assert_eq!(unsafe { emusen_core_media_insert(m, 0, &media(Some(&second), lent)) }, status::LID_CLOSED, "refused: the first disc is still lent, the second not");
    advance(m);
    assert_eq!(rom_byte(m), first[1]);
    assert_eq!(unsafe { emusen_core_media_lid(m, 0, 1) }, 0);
    assert_eq!(unsafe { emusen_core_media_insert(m, 0, &media(Some(b"not an image"), lent)) }, status::BAD_IMAGE);
    assert_eq!(unsafe { emusen_core_media_insert(m, 0, &media(Some(&second), lent)) }, 0);
    drop(first);
    second[2] ^= 0xFF;
    advance(m);
    assert_eq!(rom_byte(m), second[2], "the second disc is read in place");
    assert_eq!(unsafe { emusen_core_media_insert(m, 0, &media(Some(&third), 0)) }, 0, "a disc that is not lent is copied");
    drop(second);
    let third_copy = third.clone();
    drop(third);
    advance(m);
    assert_eq!(rom_byte(m), third_copy[3]);
    assert_eq!(unsafe { emusen_core_media_insert(m, 0, &media(None, lent)) }, 0, "an empty drive lends nothing");
    assert_eq!(unsafe { emusen_core_free(m) }, 0);
}

// Since 1.8 (EmuSen_CoreAPI.md §6.28): a slot's port is written as a number, with its medium and label, and a file that sets none writes 1.7's form.
#[test]
fn a_slots_medium_label_and_port_are_written_only_where_set() {
    use super::desc::{Battery, Info, System, info_json};
    let info = Info { id: "x".into(), systems: vec![System { id: "n64".into(), ..Default::default() }], ..Default::default() };
    let text = info_json(&info, 0, &|_| Some(vec![Battery::game(0, ".eep"), Battery::removable(1, ".2.mpk").medium("n64.pak").label("Controller 2").port(1)]), &|_| false);
    assert!(text.contains(r#""battery":[{"which":0,"suffix":".eep"},{"which":1,"suffix":".2.mpk","kind":"removable","medium":"n64.pak","label":"Controller 2","port":1}]"#), "{text}");
    assert_eq!(schema::validate(include_str!("../../abi/v1/info.schema.json"), &text), Vec::<String>::new());
    let bad = info_json(&info, 0, &|_| Some(vec![Battery::console(0, ".brm").medium("Sega CD")]), &|_| false);
    assert!(!schema::validate(include_str!("../../abi/v1/info.schema.json"), &bad).is_empty(), "a medium outside [a-z0-9.]+ is refused by the schema");
}

fn raw_info(m: *mut Machine, raw: u32) -> (i32, RawFrame) {
    let mut f = RawFrame { size: std::mem::size_of::<RawFrame>() as u32, element: 99, ..RawFrame::default() };
    let r = unsafe { emusen_core_raw_info(m, raw, &mut f) };
    (r, f)
}

// §6.30, since 1.10: an encoding is kept only once selected, is the frame's own picture through its palette, and an
// encoding or a bit past machine info's list is refused with nothing changed.
#[test]
fn a_raw_picture_is_kept_only_when_selected_and_draws_the_frame_through_its_palette() {
    let m = make("");
    let mi = crate::json::parse(text(|o, l| unsafe { emusen_core_machine_info(m, o, l) }).as_bytes()).unwrap();
    let listed = mi.get("video").unwrap().get("raw").unwrap().as_array().unwrap();
    assert_eq!((listed.len(), listed[0].get("encoding").and_then(|e| e.as_str()), listed[0].get("element").and_then(|e| e.as_i64())), (1, Some(RAW_ENCODING), Some(1)));
    let palette: Vec<i64> = listed[0].get("palette").unwrap().as_array().unwrap().iter().filter_map(|v| v.as_i64()).collect();
    advance(m);
    assert_eq!(raw_info(m, 0), (0, RawFrame { size: 32, ..RawFrame::default() }), "nothing kept before a select, every field but size zero");
    assert_eq!(unsafe { emusen_core_raw_copy(m, 0, null_mut(), 0) }, 0);
    assert_eq!(unsafe { emusen_core_raw_select(m, 0b10) }, status::NOT_SUPPORTED, "a bit past the list");
    advance(m);
    assert_eq!(raw_info(m, 0).0, 0, "the refused select kept nothing");
    assert_eq!(unsafe { emusen_core_raw_select(m, 1) }, 0);
    assert_eq!(raw_info(m, 0).0, 0, "kept from the next advance, not for the frame already drawn");
    advance(m);
    let (r, f) = raw_info(m, 0);
    assert_eq!((r, f.element, f.width, f.height, f.stride, f.bytes), (1, 1, 8, 4, 8, 32));
    let raw = bytes(|o, l| unsafe { emusen_core_raw_copy(m, 0, o, l) });
    let frame = bytes(|o, l| unsafe { emusen_core_frame_copy(m, o, l) });
    for (v, p) in raw.iter().zip(frame.chunks_exact(4)) {
        assert_eq!(palette[*v as usize], (p[0] as i64) << 16 | (p[1] as i64) << 8 | p[2] as i64);
    }
    assert_eq!((raw_info(m, 1).0, unsafe { emusen_core_raw_copy(m, 1, null_mut(), 0) }), (status::NOT_SUPPORTED, status::NOT_SUPPORTED as i64));
    let mut short = RawFrame { size: 16, ..RawFrame::default() };
    assert_eq!(unsafe { emusen_core_raw_info(m, 0, &mut short) }, status::BAD_STRUCT);
    assert_eq!(unsafe { emusen_core_raw_info(null(), 0, &mut short) }, status::NULL);
    assert_eq!(unsafe { emusen_core_raw_select(m, 0) }, 0);
    advance(m);
    assert_eq!(raw_info(m, 0).0, 0, "a frame drawn after the encoding was let go has none");
    unsafe { emusen_core_free(m) };
}

/// Since 1.11 (EmuSen_CoreAPI.md §6.31): an empty image with a system that starts empty makes the console with nothing
/// in its drive; any other system, an unknown one or none is refused with BAD_IMAGE and words, before the core sees it.
#[test]
fn an_empty_image_starts_a_system_that_starts_empty_and_nothing_else() {
    let info = text(|o, l| unsafe { emusen_core_info(o, l) });
    assert!(info.contains(r#""starts_empty":true"#), "{info}");
    assert_eq!(schema::validate(include_str!("../../abi/v1/info.schema.json"), &info), Vec::<String>::new());
    let mut code = 1;
    let p = CreateParams { system: c"test".as_ptr(), ..params(&[], "", &[]) };
    let m = unsafe { emusen_core_create(&p, &mut code) };
    assert_eq!(code, 0);
    let mi = text(|o, l| unsafe { emusen_core_machine_info(m, o, l) });
    assert!(mi.contains(r#""loaded":false"#), "{mi}");
    assert_eq!(unsafe { emusen_core_free(m) }, 0);
    let mut words = [0u8; 96];
    for (system, says) in [(c"nope", "nope is not a system of this core")] {
        let mut p = CreateParams { system: system.as_ptr(), ..params(&[], "", &[]) };
        (p.error, p.error_len) = (words.as_mut_ptr(), words.len());
        assert!(unsafe { emusen_core_create(&p, &mut code) }.is_null());
        assert_eq!(code, status::BAD_IMAGE);
        assert!(std::ffi::CStr::from_bytes_until_nul(&words).unwrap().to_str().unwrap().contains(says));
    }
    let img = image(&[1]);
    let named = CreateParams { system: c"nope".as_ptr(), ..params(&img, "", &[]) };
    let m = unsafe { emusen_core_create(&named, &mut code) };
    assert_eq!(code, 0, "a system named beside an image is ignored");
    assert_eq!(unsafe { emusen_core_free(m) }, 0);
    let short = CreateParams { size: 104, system: c"test".as_ptr(), ..params(&[], "", &[]) };
    assert!(unsafe { emusen_core_create(&short, &mut code) }.is_null(), "a struct of 1.10 carries no system");
    assert_eq!(code, status::BAD_IMAGE);
}
