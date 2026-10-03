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

fn drain_events(m: *mut Machine) -> Vec<Event> {
    let n = unsafe { emusen_core_events(m, null_mut(), 0, 0) } as usize;
    let mut out = vec![Event::default(); n];
    assert_eq!(unsafe { emusen_core_events(m, out.as_mut_ptr(), n, std::mem::size_of::<Event>()) }, n as i64);
    out
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
    assert_eq!(v.get("abi").and_then(|a| a.as_str()), Some("1.0"));
    let names: Vec<&str> = v.get("capabilities").unwrap().as_array().unwrap().iter().filter_map(|c| c.as_str()).collect();
    assert_eq!(names.len(), 18);
    assert_eq!(super::desc::capability_names(unsafe { emusen_core_capabilities() }), names);
    let mi = crate::json::parse(cases[3].1.as_bytes()).unwrap();
    let spaces = mi.get("spaces").unwrap().as_array().unwrap();
    assert_eq!(spaces[0].get("size").and_then(|s| s.as_i64()), Some(unsafe { emusen_core_space_size(m, 0) }));
    assert_eq!(mi.get("battery").unwrap().as_array().unwrap()[0].get("length").and_then(|s| s.as_i64()), Some(16));
    assert_eq!(cases[4].1, r#"{"Wide":"The picture is drawn twice as wide."}"#);
    assert!(cases[2].1.contains("boot.rom"));
    unsafe { emusen_core_free(m) };
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
    assert_eq!(code, STATUS_EMPTY_IMAGE);
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
    assert_eq!(unsafe { emusen_core_events(m, out.as_mut_ptr() as *mut Event, 2, 32) }, 1);
    assert_eq!((out[0].e.size, out[0].e.kind, out[0].canary, out[1].e.kind), (32, event::LOG, 7, 0));
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
    let m = make("");
    let waiting = unsafe { emusen_core_log_drain(m, null_mut(), 0) };
    assert_eq!(waiting, "info\ttest\tcreated\n".len() as i64);
    let mut small = [0u8; 4];
    assert_eq!(unsafe { emusen_core_log_drain(m, small.as_mut_ptr(), 4) }, 0, "only whole records");
    unsafe { emusen_core_free(m) };
    assert_eq!(text(|o, l| unsafe { emusen_core_log_drain(null_mut(), o, l) }), "info\ttest\tcreated\n");
    let mut q = super::outbox::LogQueue::default();
    for i in 0..super::outbox::LOG_CAPACITY + 3 {
        q.push(super::Level::Debug, "a\tb", &format!("{i}\nx"));
    }
    let mut out = vec![0u8; q.waiting()];
    assert_eq!(q.drain(&mut out), out.len());
    let lines: Vec<&str> = std::str::from_utf8(&out).unwrap().lines().collect();
    assert_eq!((lines[0], lines[1], lines.len()), ("warn\tlog\t3 records dropped", "debug\ta b\t3 x", super::outbox::LOG_CAPACITY + 1));
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

#[test]
fn the_crates_constants_are_the_headers_own_numbers() {
    assert_eq!(ABI_VERSION, 0x0001_0000);
    assert_eq!((status::BAD_STRUCT, status::BAD_IMAGE, status::STATE_VERSION), (-263, -264, -4));
    assert_eq!(EXPORTS.len(), 55);
    assert_eq!(EXPORTS.iter().filter(|e| e.1 == 0).count(), 32);
    assert_eq!(CAPABILITY_NAMES.len(), 18);
    assert_eq!(super::EXPORTING & !(caps::SNAPSHOT | caps::FRAME_SERIAL | caps::ROW_REPEAT | caps::BATTERY_DIRTY), super::EXPORTING);
}
