//! The host on emusen-native's test cores, a core whose descriptors lie, and generated cores directories.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use emusen_core_host::latch::Inputs;
use emusen_core_host::library::LoadError;
use emusen_core_host::machine::Status;
use emusen_core_host::sidecar::{self, Sidecar};
use emusen_core_host::trust::{self, MANIFEST, Standing};
use emusen_core_host::{Create, Discovery, InputLatch, Library, Machine, StateLayout, Trust, listing, sha256};
use emusen_native::json::{self, Value};

fn example(name: &str) -> PathBuf {
    let profile = std::env::current_exe().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    let file = if cfg!(windows) { format!("{name}.dll") } else if cfg!(target_os = "macos") { format!("lib{name}.dylib") } else { format!("lib{name}.so") };
    profile.join("examples").join(file)
}

fn image(payload: &[u8]) -> Vec<u8> {
    let mut v = b"V1TC".to_vec();
    v.extend_from_slice(&(payload.len() as u16).to_le_bytes());
    v.extend_from_slice(payload);
    v
}

/// A fresh directory of its own under the target's scratch directory.
fn scratch(name: &str) -> PathBuf {
    static N: AtomicU32 = AtomicU32::new(0);
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("host-{}-{name}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// `example`'s library copied into `dir` as `file` with its sidecar, written in a build directory of its own as the
/// build writes it, so that the copy in `dir` is never loaded by the writing.
fn installed(dir: &Path, example_name: &str, file: &str, development: bool) -> PathBuf {
    let build = scratch("build").join(file);
    std::fs::copy(example(example_name), &build).unwrap();
    let written = sidecar::write(&build, development).unwrap();
    let to = dir.join(file);
    std::fs::copy(&build, &to).unwrap();
    std::fs::copy(&written, sidecar::path_for(&to)).unwrap();
    to
}

fn manifest(dir: &Path, libraries: &[&Path]) {
    let text: String = libraries.iter().map(|l| format!("{}  {}\n", sha256::file_hex(l).unwrap(), l.file_name().unwrap().to_str().unwrap())).collect();
    std::fs::write(dir.join(MANIFEST), text).unwrap();
}

#[test]
fn the_test_core_loads_and_passes_rule_4_and_its_report_names_it() {
    let lib = Library::load(&example("host_test_core")).unwrap();
    let c = lib.check().unwrap();
    assert_eq!(c.version, (1 << 16) | 15);
    assert_eq!(lib.report(), format!("{}, V1TestCore (test) 1.0.0, core ABI 1.15", example("host_test_core").display()));
    assert!(std::ptr::eq(lib, Library::load(&example("host_test_core")).unwrap()), "one library per path");
    let plain = Library::load(&example("host_plain_core")).unwrap();
    assert!(plain.check().is_ok() && plain.capabilities() == 0);
}

#[test]
fn a_library_without_the_required_exports_is_refused_in_both_words() {
    let e = Library::load(&example("host_partial_core")).err().unwrap();
    assert_eq!(e.to_string(), "lacks the required export emusen_core_capabilities");
    assert_eq!(e.host_words(), "lacks the required export emusen_core_capabilities");
    assert_eq!(LoadError::Missing { export: "x".into(), abi: None }.host_words(), "has no emusen_core_abi_version; it is not on the core ABI");
    assert_eq!(LoadError::Missing { export: "x".into(), abi: Some(2 << 16) }.host_words(), "speaks core ABI major 2, this host major 1");
    let missing = scratch("missing").join("libnone.so");
    assert!(matches!(Library::load(&missing), Err(LoadError::Unloadable(_))));
}

#[test]
fn each_lie_is_refused_by_rule_4_in_its_own_words() {
    let dir = scratch("liars");
    let cases: &[(u32, &str)] = &[
        (0, ""),
        (1, "speaks core ABI major 2, this host major 1"),
        (2, "lacks emusen_core_reset but claims its capability"),
        (3, "its info says abi 1.13, its export 1.15"),
        (4, "it requires the host obligation teleport, which this host does not know"),
        (5, "its info is not UTF-8 text under 1 MiB"),
        (6, "its info is not UTF-8 text under 1 MiB"),
        (7, "its info is not JSON: at byte"),
        (8, "its info lacks id, name, version or license"),
        (9, "its info does not validate: "),
        (10, "its settings schema is not JSON: at byte"),
        (11, "its settings schema does not validate: "),
        (12, "its id Liar is not [a-z0-9-]+"),
    ];
    for &(mode, words) in cases {
        let path = dir.join(format!("libliar{mode}.so"));
        std::fs::copy(example("host_liar_core"), &path).unwrap();
        let raw = unsafe { libloading::Library::new(&path) }.unwrap();
        unsafe { raw.get::<unsafe extern "C" fn(u32)>(b"liar_set\0").unwrap()(mode) };
        let lib = Library::load(&path).unwrap();
        match lib.check() {
            Ok(_) => assert_eq!(mode, 0, "mode {mode} passed"),
            Err(why) => {
                assert!(why.starts_with(words) && !words.is_empty(), "mode {mode}: {why}");
                assert_eq!(lib.report(), format!("libliar{mode}.so: {why}"));
            }
        }
        std::mem::forget(raw);
    }
}

#[test]
fn a_machine_runs_frames_states_spaces_and_input_through_the_calls() {
    let lib = Library::load(&example("host_test_core")).unwrap();
    let img = image(&[1, 2, 3]);
    let mut m = Machine::create(lib, &Create::new(&img)).unwrap();
    let mut picture = vec![0u8; 1 << 20];
    let mut sound = vec![0i16; 8192];
    for _ in 0..3 {
        m.advance().unwrap();
        m.present().unwrap();
        let fi = m.frame_info().unwrap();
        assert!(fi.width > 0 && fi.height > 0);
        assert_eq!(m.frame_copy(&mut picture).unwrap() as i64, fi.bytes);
        let (n, rate) = m.audio_drain(&mut sound).unwrap();
        assert!(n > 0 && rate > 0);
        m.events().unwrap();
    }
    assert_eq!(m.frame_count(), 3);
    let mut state = vec![0u8; m.state_size(0).unwrap()];
    let n = m.state_save(0, &mut state).unwrap();
    state.truncate(n);
    let mut ram = [0u8; 4];
    m.space_read(0, 0, &mut ram).unwrap();
    m.advance().unwrap();
    m.state_load(&state).unwrap();
    let mut again = [0u8; 4];
    m.space_read(0, 0, &mut again).unwrap();
    assert_eq!(ram, again, "a load puts the RAM back");
    let layout = StateLayout::parse("test", &m.layout(0).unwrap()).unwrap();
    assert_eq!(layout.length(), state.len());
    let latch = InputLatch::new(2, 1);
    latch.hold(0, 1, 1);
    latch.send(&mut m);
    let before = { let mut b = [0u8; 4]; m.space_read(0, 0, &mut b).unwrap(); b[3] };
    m.advance().unwrap();
    let after = { let mut b = [0u8; 4]; m.space_read(0, 0, &mut b).unwrap(); b[3] };
    assert_eq!(after, before.wrapping_add(1), "button A held reaches the machine");
    assert_eq!(m.space_write(1, 0, &[9]), Err(emusen_native::core::sys::status::READ_ONLY));
}

#[test]
fn a_refused_create_carries_the_cores_status_and_words() {
    let lib = Library::load(&example("host_test_core")).unwrap();
    let r = Machine::create(lib, &Create::new(&[])).err().unwrap();
    assert_eq!((r.status, r.words.as_str()), (emusen_native::core::sys::status::BAD_IMAGE, "the image is empty and no system was named to start empty"));
    let r = Machine::create(lib, &Create::new(b"junk!!")).err().unwrap();
    assert_eq!(r.words, "not a test core image");
}

// Since 1.15 (EmuSen_CoreAPI.md §6.40): the system a host routed an image to is passed beside it, and one the core does not list is refused with words.
#[test]
fn a_routed_system_goes_beside_the_image_and_an_unlisted_one_is_refused() {
    let lib = Library::load(&example("host_test_core")).unwrap();
    let image = [b"V1TC".as_slice(), &[3, 0, 1, 2, 3]].concat();
    let m = Machine::create(lib, &Create { system: Some(c"test"), ..Create::new(&image) }).unwrap();
    assert!(m.machine_info().unwrap().contains(r#""system":"test""#));
    let r = Machine::create(lib, &Create { system: Some(c"elsewhere"), ..Create::new(&image) }).err().unwrap();
    assert_eq!((r.status, r.words.as_str()), (emusen_native::core::sys::status::BAD_IMAGE, "the image was routed to elsewhere, which is not a system of this core"));
}

#[derive(Default)]
struct Recorder(Vec<String>);

impl Inputs for Recorder {
    fn set_buttons(&mut self, port: u32, mask: u32, changed: u32) -> Result<(), Status> {
        self.0.push(format!("buttons {port} {mask:#x} {changed:#x}"));
        Ok(())
    }
    fn set_axis(&mut self, port: u32, axis: u32, value: f64) -> Result<(), Status> {
        self.0.push(format!("axis {port} {axis} {value}"));
        Ok(())
    }
    fn set_present(&mut self, port: u32, present: bool) -> Result<(), Status> {
        self.0.push(format!("present {port} {present}"));
        Ok(())
    }
}

#[test]
fn the_latch_sends_changes_once_the_latest_axis_and_presence_only_when_it_changed() {
    let latch = InputLatch::new(2, 2);
    latch.hold(0, 0b11, 0b01);
    latch.hold(0, 0b10, 0b10);
    latch.move_axis(1, 1, 0.25);
    latch.move_axis(1, 1, -0.5);
    latch.connect(1, true);
    latch.connect(1, false);
    latch.hold(5, 1, 1);
    latch.move_axis(0, 2, 1.0);
    let mut r = Recorder::default();
    latch.send(&mut r);
    assert_eq!(r.0, ["buttons 0 0x3 0x3", "present 1 false", "axis 1 1 -0.5"], "port by port, presence before buttons before axes");
    let mut r = Recorder::default();
    latch.send(&mut r);
    assert!(r.0.is_empty(), "nothing is sent twice");
    latch.connect(1, true);
    latch.connect(1, false);
    latch.connect(1, false);
    latch.send(&mut r);
    assert!(r.0.is_empty(), "a plug and a pull that cancel send nothing");
    latch.grow(3);
    latch.hold(2, 4, 4);
    latch.send(&mut r);
    assert_eq!(r.0, ["buttons 2 0x4 0x4"]);
    assert_eq!(latch.buttons(0), 0b11);
    latch.resend_presence();
    let mut r = Recorder::default();
    latch.send(&mut r);
    assert_eq!(r.0, ["present 0 true", "present 1 false", "present 2 true"], "after a state load every port's presence, once");
    let mut r = Recorder::default();
    latch.send(&mut r);
    assert!(r.0.is_empty());
}

#[test]
fn the_layout_reads_and_writes_fields_and_names_differences() {
    let text = "0 1 u8 a\n1 2 i16 b\n3 4 i32[1] c\n7 8 f64 d\n15 3 u8[3] e\n";
    let l = StateLayout::parse("t", text).unwrap();
    let mut s = vec![0u8; l.length()];
    l.set(&mut s, "a", 200).unwrap();
    l.set(&mut s, "d", 1.5f64.to_bits() as i64).unwrap();
    l.bytes(&mut s, "e").unwrap()[1] = 7;
    s[1] = 0xFE;
    s[2] = 0xFF;
    assert_eq!(l.get(&s, "a").unwrap(), 200);
    assert_eq!(l.get(&s, "b").unwrap(), -2);
    assert_eq!(l.double(&s, "d").unwrap(), 1.5);
    assert_eq!(l.count("c").unwrap(), 1);
    assert_eq!(l.int(&s, "c", 0).unwrap(), 0);
    assert!(l.int(&s, "c", 1).is_err() && l.get(&s, "e").is_err() && l.field("z").is_err());
    let t = vec![0u8; l.length()];
    assert_eq!(l.differences(&s, &t), ["a: 200 against 0", "b: -2 against 0", &format!("d: {} against 0", 1.5f64.to_bits() as i64), "e[1]: 07 against 00"]);
    assert!(StateLayout::parse("t", "x y z\n").is_err());
}

fn listed(d: &Discovery) -> Value {
    json::parse(listing::both(d, &Trust::default()).as_bytes()).unwrap()
}

fn reports(v: &Value) -> Vec<String> {
    v.get("cores").and_then(Value::as_array).unwrap().iter().map(|c| c.get("report").and_then(Value::as_str).unwrap().to_owned()).collect()
}

#[test]
fn discovery_lists_from_sidecars_alone_and_opens_under_the_trust_rules() {
    let dir = scratch("tree");
    let test = installed(&dir, "host_test_core", "libtest.so", false);
    let plain = installed(&dir, "host_plain_core", "libplain.so", false);
    manifest(&dir, &[&test, &plain]);
    let d = Discovery::scan(std::slice::from_ref(&dir), false);
    assert_eq!(d.found().len(), 2);
    assert!(!Library::is_loaded(&dir.join("libtest.so")), "listing loads nothing");
    assert_eq!(d.found()[0].engine_name(), "V1PlainCore (test)", "ordinal order: libplain before libtest");
    assert_eq!(reports(&listed(&d)), ["not yet loaded", "not yet loaded"]);
    let trust = Trust::new(std::slice::from_ref(&dir), &[]);
    for c in d.found() {
        c.open(&trust).unwrap();
    }
    let v = listed(&d);
    assert_eq!(reports(&v)[1], format!("{}, V1TestCore (test) 1.0.0, core ABI 1.15", dir.join("libtest.so").display()));
    let engine = &v.get("engines").and_then(Value::as_array).unwrap()[1];
    assert_eq!(engine.get("systems"), Some(&Value::Array(vec![Value::Str("test".into())])));
    assert_eq!(engine.get("available"), Some(&Value::Bool(true)));
    assert_eq!(d.for_extension(".TST").count(), 2);
    assert_eq!(d.for_system("test").count(), 2);
    assert!(d.by_engine_name("V1TestCore (test)").is_some());
}

#[test]
fn a_missing_an_altered_and_an_unapproved_library_are_listed_and_refused_at_open() {
    let dir = scratch("faults");
    let test = installed(&dir, "host_test_core", "libtest.so", false);
    manifest(&dir, &[&test]);
    let altered = installed(&dir, "host_plain_core", "libaltered.so", false);
    let mut bytes = std::fs::read(&altered).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 0xFF;
    std::fs::write(&altered, bytes).unwrap();
    // The unapproved library is a copy of the test core under another id, so that the first id does not hide it.
    let unapproved = installed(&dir, "host_liar_core", "libunapproved.so", false);
    let missing = installed(&dir, "host_test_core", "libmissing.so", false);
    let text = std::fs::read_to_string(sidecar::path_for(&missing)).unwrap().replace("\"v1-test-core\"", "\"v1-missing-core\"");
    std::fs::write(sidecar::path_for(&missing), text).unwrap();
    std::fs::remove_file(&missing).unwrap();
    let d = Discovery::scan(std::slice::from_ref(&dir), false);
    assert_eq!(d.found().len(), 4, "every one is listed, as the C# host lists them");
    let trust = Trust::new(std::slice::from_ref(&dir), &[]);
    for c in d.found() {
        let _ = c.open(&trust);
    }
    let r = reports(&listed(&d));
    assert_eq!(r[0], "libaltered.so is not the library its sidecar describes (its SHA-256 differs); it was not loaded");
    assert_eq!(r[1], "libmissing.so is missing beside its sidecar");
    assert_eq!(r[3], "libunapproved.so is in no cores manifest and has not been approved; it was not loaded");
    assert!(!Library::is_loaded(&unapproved) && !Library::is_loaded(&altered));
    let approved = Trust::new(std::slice::from_ref(&dir), &[sha256::file_hex(&unapproved).unwrap().to_uppercase()]);
    let again = Discovery::scan(std::slice::from_ref(&dir), false);
    assert!(again.found()[3].open(&approved).is_ok(), "an approval by hash lets it load");
}

#[test]
fn a_loaded_library_is_judged_by_the_image_that_runs_not_by_the_file_now_on_disk() {
    let dir = scratch("replaced");
    let lib = installed(&dir, "host_test_core", "libtest.so", false);
    manifest(&dir, &[&lib]);
    let first = Discovery::scan(std::slice::from_ref(&dir), false);
    first.found()[0].open(&Trust::new(std::slice::from_ref(&dir), &[])).unwrap();
    // Another build of the same core, one byte longer, put in its place with its sidecar and the manifest rewritten.
    let was = sha256::file_hex(&lib).unwrap();
    let mut bytes = std::fs::read(&lib).unwrap();
    bytes.push(0);
    let fresh = dir.join("libtest.so.new");
    std::fs::write(&fresh, bytes).unwrap();
    std::fs::rename(&fresh, &lib).unwrap();
    let now = sha256::file_hex(&lib).unwrap();
    let text = std::fs::read_to_string(sidecar::path_for(&lib)).unwrap().replace(&was, &now);
    std::fs::write(sidecar::path_for(&lib), text).unwrap();
    manifest(&dir, &[&lib]);
    let second = Discovery::scan(std::slice::from_ref(&dir), false);
    let refused = second.found()[0].open(&Trust::new(std::slice::from_ref(&dir), &[])).err();
    assert_eq!(refused, Some("libtest.so is not the library its sidecar describes (its SHA-256 differs); it was not loaded"), "the image this process would run is the first file's, which nothing now vouches for");
}

#[test]
fn the_scan_keeps_the_first_id_skips_development_and_other_majors_and_files_that_are_not_sidecars() {
    let first = scratch("first");
    let second = scratch("second");
    installed(&first, "host_test_core", "libb.so", false);
    installed(&second, "host_test_core", "liba.so", false);
    installed(&second, "host_plain_core", "libdev.so", true);
    let other = installed(&second, "host_liar_core", "libother.so", false);
    let text = std::fs::read_to_string(sidecar::path_for(&other)).unwrap().replace("\"abi\":\"1.15\",\"capabilities\"", "\"abi\":\"2.0\",\"capabilities\"");
    std::fs::write(sidecar::path_for(&other), text).unwrap();
    std::fs::write(second.join("junk.core.json"), "{\"sidecar\":2}").unwrap();
    std::fs::write(second.join("path.core.json"), "{\"sidecar\":1,\"library\":\"../libb.so\",\"info\":{}}").unwrap();
    let dirs = [first.clone(), second.clone(), first.clone()];
    let d = Discovery::scan(&dirs, false);
    let libs: Vec<_> = d.found().iter().map(|c| c.sidecar.library_path.clone()).collect();
    assert_eq!(libs, [first.join("libb.so")]);
    let d = Discovery::scan(&dirs, true);
    let libs: Vec<_> = d.found().iter().map(|c| c.sidecar.library_path.clone()).collect();
    assert_eq!(libs, [first.join("libb.so"), second.join("libdev.so")]);
    assert!(Sidecar::read(&second.join("junk.core.json")).is_none() && Sidecar::read(&second.join("path.core.json")).is_none());
}

#[cfg(unix)]
#[test]
fn rule_3_refuses_a_link_out_a_file_others_may_write_a_foreign_header_and_a_huge_file() {
    use std::os::unix::fs::PermissionsExt;
    let outside = scratch("outside");
    let dir = scratch("rule3");
    let away = installed(&outside, "host_test_core", "libaway.so", false);
    std::os::unix::fs::symlink(&away, dir.join("liblink.so")).unwrap();
    let trust = Trust::new(std::slice::from_ref(&dir), &[]);
    assert_eq!(trust.check_file(&dir.join("liblink.so")), Err("lies outside the cores directories once its links are resolved".into()));
    let open = installed(&dir, "host_test_core", "libopen.so", false);
    std::fs::set_permissions(&open, std::fs::Permissions::from_mode(0o757)).unwrap();
    assert_eq!(trust.check_file(&open), Err("is writable by others".into()));
    let foreign = dir.join("libforeign.so");
    let mut bytes = std::fs::read(example("host_test_core")).unwrap();
    bytes[18] ^= 0x55;
    std::fs::write(&foreign, bytes).unwrap();
    assert_eq!(trust.check_file(&foreign), Err("is not a library for this machine's architecture".into()));
    let huge = dir.join("libhuge.so");
    std::fs::copy(example("host_test_core"), &huge).unwrap();
    std::fs::OpenOptions::new().write(true).open(&huge).unwrap().set_len((512 << 20) + 1).unwrap();
    assert_eq!(trust.check_file(&huge), Err("is larger than 512 MB".into()));
    let fine = installed(&dir, "host_test_core", "libfine.so", false);
    assert_eq!(trust.check_file(&fine), Ok(()));
    // Through discovery, the words follow the name.
    let alone = scratch("rule3-alone");
    let open = installed(&alone, "host_test_core", "libopen.so", false);
    std::fs::set_permissions(&open, std::fs::Permissions::from_mode(0o757)).unwrap();
    let d = Discovery::scan(std::slice::from_ref(&alone), false);
    let trust = Trust::new(std::slice::from_ref(&alone), &[]);
    assert_eq!(d.found()[0].open(&trust).err(), Some("libopen.so is writable by others; it was not loaded"));
}

#[test]
fn a_manifest_vouches_only_for_its_own_directory() {
    let a = scratch("vouch-a");
    let b = scratch("vouch-b");
    let la = installed(&a, "host_test_core", "libx.so", false);
    let lb = installed(&b, "host_test_core", "libx.so", false);
    manifest(&a, &[&la]);
    let trust = Trust::new(&[a.clone(), b.clone()], &[]);
    let h = sha256::file_hex(&la).unwrap();
    assert!(trust.vouches(&la, &h));
    assert!(!trust.vouches(&lb, &h), "b's copy is not in a manifest of b's");
    assert!(!trust.vouches(&a.join("liby.so"), &h), "a line names its file");
}

/// The tables are counted as the header counts them, pairs and triples, so a core reads no value past the caller's.
#[test]
fn a_debug_table_is_counted_in_pairs_and_triples() {
    let lib = Library::load(&example("host_test_core")).unwrap();
    lib.check().unwrap();
    let img = image(b"debug tables");
    let mut m = Machine::create(lib, &Create::new(&img)).unwrap();
    // A breakpoint at 100 alone; the pair after it in the same array, over every address, is not the table's.
    let table = [100, 100, 0, 255];
    m.debug_set_breakpoints(0, &table[..2]).unwrap();
    let (reasons, _, at, _) = m.debug_run_frame(0).unwrap();
    assert_eq!(reasons, 0, "the frame at 0 runs; a halt at {at} read the pair past the table");
    let ranges = [0u32, 4, 4, 0, 0, 0xFFFF];
    m.debug_set_ranges(0, &ranges[..3]).unwrap();
    m.debug_set_stack(&[1, 2]).unwrap();
}

/// The cheat tables are counted as the header counts them, triples and quads, so a core takes no entry past the
/// caller's: a poke after the table's one, in the same array, is never written (EmuSen_Backend.md §29.29).
#[test]
fn a_cheat_table_is_counted_in_triples_and_quads() {
    let lib = Library::load(&example("host_test_core")).unwrap();
    lib.check().unwrap();
    let img = image(b"cheat tables");
    let mut m = Machine::create(lib, &Create::new(&img)).unwrap();
    const NONE: u32 = emusen_native::core::sys::flags::NO_COMPARE;
    let pokes = [0, 5, 0x42, NONE, 0, 6, 0x43, NONE, 0, 7, 0x44, NONE, 0, 8, 0x45, NONE];
    assert_eq!(m.set_cheat_pokes(&pokes[..4]), Ok(1), "the core took more pokes than the table holds");
    let mut before = [0u8; 4];
    m.space_read(0, 5, &mut before).unwrap();
    m.advance().unwrap();
    let mut ram = [0u8; 4];
    m.space_read(0, 5, &mut ram).unwrap();
    assert_eq!(ram, [0x42, before[1], before[2], before[3]], "a poke past the table was written");
    let patches = [1, 2, NONE, 3, 4, NONE, 5, 6, NONE];
    assert_eq!(m.set_rom_patches(&patches[..3]), Ok(1), "the core took more patches than the table holds");
}

fn field<'a>(v: &'a Value, key: &str) -> &'a Value {
    v.get(key).unwrap_or_else(|| panic!("no {key}"))
}

#[test]
fn the_players_directory_comes_after_the_installation_its_manifest_unread_and_its_libraries_hashed() {
    let install = scratch("p-install");
    let player = scratch("p-player");
    let test = installed(&install, "host_test_core", "libtest.so", false);
    manifest(&install, &[&test]);
    // A player's copy of a project core's id is dropped, and is not listed as unread either.
    installed(&player, "host_test_core", "libshadow.so", false);
    let plain = installed(&player, "host_plain_core", "libplain.so", false);
    manifest(&player, &[&plain]);
    std::fs::copy(example("host_partial_core"), player.join("libbare.so")).unwrap();
    let stale = installed(&player, "host_liar_core", "libstale.so", false);
    let mut bytes = std::fs::read(&stale).unwrap();
    bytes.push(0);
    std::fs::write(&stale, bytes).unwrap();
    std::fs::write(player.join("notes.txt"), "not a library").unwrap();
    let (inst, mine) = (vec![install.clone()], vec![player.clone()]);
    let d = Discovery::scan_with_player(&inst, &mine, false);
    let names: Vec<_> = d.found().iter().map(|c| (c.sidecar.library_path.clone(), c.player)).collect();
    assert_eq!(names, [(test.clone(), false), (plain.clone(), true)]);
    let unread: Vec<_> = d.unread().iter().map(|u| (u.library_path.clone(), u.sha256.clone())).collect();
    assert_eq!(unread, [(player.join("libbare.so"), sha256::file_hex(&player.join("libbare.so")).unwrap()), (stale.clone(), sha256::file_hex(&stale).unwrap())]);
    assert!(d.lists_hash(&sha256::file_hex(&stale).unwrap().to_uppercase()) && !d.lists_hash(&"00".repeat(32)));
    let trust = Trust::new(&inst, &[]).with_player(&mine);
    assert_eq!(d.found()[0].standing(&trust), Standing::Manifest);
    assert_eq!(d.found()[1].standing(&trust), Standing::NeedsApproval, "a manifest in the player's directory is not read");
    assert_eq!(d.found()[1].open(&trust).err(), Some("libplain.so is in no cores manifest and has not been approved; it was not loaded"));
    assert!(d.found()[1].awaits_approval() && !d.found()[0].awaits_approval());
    let v = json::parse(listing::both(&d, &trust).as_bytes()).unwrap();
    let cores = field(&v, "cores").as_array().unwrap();
    assert_eq!(cores.len(), 4);
    assert_eq!(field(&cores[0], "directory"), &Value::Str("installation".into()));
    assert_eq!(field(&cores[0], "trust"), &Value::Str("manifest".into()));
    assert_eq!(field(&cores[0], "systems"), &Value::Array(vec![Value::Str("test".into())]));
    assert_eq!(field(&cores[1], "directory"), &Value::Str("player".into()));
    assert_eq!(field(&cores[1], "trust"), &Value::Str("needs_approval".into()));
    let bare = &cores[2];
    assert_eq!(field(bare, "trust"), &Value::Str("not_read".into()));
    assert_eq!(field(bare, "report"), &Value::Str("not yet read".into()));
    for k in ["sidecar", "abi", "engine", "name", "license"] {
        assert_eq!(field(bare, k), &Value::Null, "{k}");
    }
    let engines = field(&v, "engines").as_array().unwrap();
    assert_eq!(engines.len(), 2, "a library not yet read is no engine");
    assert_eq!(field(&engines[1], "trust"), &Value::Str("needs_approval".into()));
    let approved = Trust::new(&inst, &[sha256::file_hex(&plain).unwrap()]).with_player(&mine);
    let again = Discovery::scan_with_player(&inst, &mine, false);
    assert_eq!(again.found()[1].standing(&approved), Standing::Approved);
    assert!(again.found()[1].open(&approved).is_ok(), "approved, the player's library loads");
    let h = sha256::file_hex(&plain).unwrap();
    let text = std::fs::read_to_string(sidecar::path_for(&plain)).unwrap().replace(&h, &h.to_uppercase());
    std::fs::write(sidecar::path_for(&plain), text).unwrap();
    let capitals = Discovery::scan_with_player(&inst, &mine, false);
    assert_eq!(capitals.found()[1].standing(&approved), Standing::Approved, "a hash in capitals is the same hash");
}

#[test]
fn a_library_not_yet_read_is_read_once_approved_and_then_listed_by_its_sidecar() {
    let player = scratch("read-player");
    let lib = player.join("libread.so");
    std::fs::copy(example("host_plain_core"), &lib).unwrap();
    let mine = vec![player.clone()];
    let d = Discovery::scan_with_player(&[], &mine, false);
    let u = &d.unread()[0];
    assert_eq!(u.read(&Trust::new(&[], &[]).with_player(&mine)), Err("libread.so has not been approved; it was not read".into()));
    assert!(!Library::is_loaded(&lib), "nothing is loaded before the approval");
    let trust = Trust::new(&[], &[u.sha256.clone()]).with_player(&mine);
    assert_eq!(u.read(&trust), Ok(sidecar::path_for(&lib)));
    let after = Discovery::scan_with_player(&[], &mine, false);
    assert!(after.unread().is_empty());
    assert_eq!(after.found()[0].standing(&trust), Standing::Approved);
    assert!(after.found()[0].open(&trust).is_ok());
    // A library changed between the listing and the approval is not read.
    let other = scratch("read-changed");
    let changed = other.join("libchanged.so");
    std::fs::copy(example("host_test_core"), &changed).unwrap();
    let mine = vec![other.clone()];
    let d = Discovery::scan_with_player(&[], &mine, false);
    let listed = d.unread()[0].sha256.clone();
    let mut bytes = std::fs::read(&changed).unwrap();
    bytes.push(0);
    std::fs::write(&changed, bytes).unwrap();
    let trust = Trust::new(&[], &[listed]).with_player(&mine);
    assert_eq!(d.unread()[0].read(&trust), Err("libchanged.so changed after it was listed; it was not loaded".into()));
    assert!(!sidecar::path_for(&changed).exists());
}

#[test]
fn a_players_library_is_hashed_once_while_its_identity_length_and_time_stay() {
    let player = scratch("hash-once");
    let lib = player.join("libonce.so");
    std::fs::copy(example("host_plain_core"), &lib).unwrap();
    let mine = vec![player.clone()];
    let first = Discovery::scan_with_player(&[], &mine, false).unread()[0].sha256.clone();
    let time = std::fs::metadata(&lib).unwrap().modified().unwrap();
    let mut bytes = std::fs::read(&lib).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 0xFF;
    std::fs::write(&lib, &bytes).unwrap();
    std::fs::File::options().write(true).open(&lib).unwrap().set_modified(time).unwrap();
    assert_eq!(Discovery::scan_with_player(&[], &mine, false).unread()[0].sha256, first, "the record is kept while the file looks the same");
    std::fs::File::options().write(true).open(&lib).unwrap().set_modified(time + std::time::Duration::from_secs(1)).unwrap();
    assert_eq!(Discovery::scan_with_player(&[], &mine, false).unread()[0].sha256, sha256::file_hex(&lib).unwrap(), "and read again once it does not");
}

#[cfg(unix)]
#[test]
fn rule_3_refuses_a_library_under_a_directory_others_may_write_unless_it_is_sticky() {
    use std::os::unix::fs::PermissionsExt;
    let parent = scratch("open-parent");
    let dir = parent.join("cores");
    std::fs::create_dir(&dir).unwrap();
    let lib = installed(&dir, "host_test_core", "libin.so", false);
    let trust = Trust::new(std::slice::from_ref(&dir), &[]);
    assert_eq!(trust.check_file(&lib), Ok(()));
    let real = std::fs::canonicalize(&parent).unwrap();
    std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o777)).unwrap();
    assert_eq!(trust.check_file(&lib), Err(format!("lies under {}, which others may write", real.display())));
    std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o1777)).unwrap();
    assert_eq!(trust.check_file(&lib), Ok(()), "a sticky directory lets no one rename another's file");
    std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o777)).unwrap();
    assert_eq!(trust.check_file(&lib), Err(format!("lies under {}, which others may write", std::fs::canonicalize(&dir).unwrap().display())));
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(unix)]
#[test]
fn the_approvals_file_is_read_by_hash_its_stray_lines_skipped_and_refused_when_others_may_write_it() {
    use std::os::unix::fs::PermissionsExt;
    let dir = scratch("approvals");
    let file = dir.join(trust::APPROVALS);
    assert_eq!(trust::read_approvals(&file), Ok(Vec::new()), "no file, no approvals");
    let (a, b) = ("AB".repeat(32), "cd".repeat(32));
    std::fs::write(&file, format!("# the player's\n{a}  libone.so\nnonsense\n{b} *libtwo.so\n")).unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(trust::read_approvals(&file), Ok(vec!["ab".repeat(32), b.clone()]));
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o646)).unwrap();
    assert_eq!(trust::read_approvals(&file), Err(format!("{} is writable by others; its approvals were not read", file.display())));
}
