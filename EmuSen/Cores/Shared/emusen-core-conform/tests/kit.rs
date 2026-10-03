//! The kit on the test cores: both pass every case, the core with seeded faults fails exactly the cases its faults
//! break, and a sidecar records the library as the host reads it.

use std::path::PathBuf;

use emusen_core_conform::{Options, Script, digest, run, sidecar};
use emusen_native::json::{self, Value};

/// The examples cargo built beside this test.
fn library(name: &str) -> PathBuf {
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

fn options() -> Options {
    Options { self_delimiting: true, image: image(&[1, 2, 3, 0x20, 0x41]), frames: 120, settings: String::new(), files: Vec::new(), script: None }
}

fn verdicts(name: &str) -> Vec<(&'static str, bool)> {
    let report = run(&library(name), &options());
    for c in report.cases.iter().filter(|c| !c.passed) {
        eprintln!("{name} {} {}: {:?}", c.id, c.name, c.evidence);
    }
    report.cases.iter().map(|c| (c.id, c.passed)).collect()
}

const ALL: [&str; 15] = ["C1", "C2", "C3", "C4", "C5", "C6", "C7", "C8", "C9", "C10", "C11", "C12", "C13", "C14", "C15"];

#[test]
fn both_test_cores_pass_every_case() {
    for name in ["kit_test_core", "kit_plain_core"] {
        assert_eq!(verdicts(name), ALL.iter().map(|&c| (c, true)).collect::<Vec<_>>(), "{name}");
    }
}

// Machines that differ and a truncated state accepted break C4, C7, C8 and C10, and through them C9's and C13's
// comparisons with a solo run and C12's rerun of C6-C8.
#[test]
fn the_seeded_faults_fail_the_cases_they_break_and_no_others() {
    let failing = ["C4", "C7", "C8", "C9", "C10", "C12", "C13"];
    assert_eq!(verdicts("kit_faulty_core"), ALL.iter().map(|&c| (c, !failing.contains(&c))).collect::<Vec<_>>());
}

// An exact setting that changes the state, a refusal of any host but 1.0, info that drifts, a writable read-only
// space and an observed frame with other sound break C9, C12, C13, C14 and C15, each alone.
#[test]
fn the_faults_of_c9_and_c12_to_c15_fail_those_cases_and_no_others() {
    let failing = ["C9", "C12", "C13", "C14", "C15"];
    assert_eq!(verdicts("kit_faulty_full_core"), ALL.iter().map(|&c| (c, !failing.contains(&c))).collect::<Vec<_>>());
}

// Each of those faults alone, in a runner process of its own, fails exactly the case it is for.
#[test]
fn each_fault_of_c9_and_c12_to_c15_alone_fails_its_case_alone() {
    let file = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("kit-faulty-full.v1tc");
    std::fs::write(&file, image(&[1, 2, 3, 0x20, 0x41])).unwrap();
    for (fault, case) in [(8, "C9"), (16, "C12"), (32, "C13"), (64, "C14"), (128, "C15")] {
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_emusen-core-conform"))
            .args(["--self-delimiting", "--frames", "120", "--core"])
            .arg(library("kit_faulty_full_core"))
            .arg("--image")
            .arg(&file)
            .env("EMUSEN_TEST_CORE_FAULTS", fault.to_string())
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&out.stdout);
        let failed: Vec<&str> = text.lines().filter_map(|l| l.strip_prefix("FAIL ")).filter_map(|l| l.split_whitespace().next()).collect();
        assert_eq!(failed, vec![case], "fault {fault}:\n{text}");
        assert_eq!(out.status.code(), Some(1), "fault {fault}");
    }
}

// Refusal passes C4 whether or not the system's format is self-delimiting.
#[test]
fn c4_reports_each_malformed_image_as_refused_or_accepted_and_passes_refusal_under_either_rule() {
    for self_delimiting in [true, false] {
        let report = run(&library("kit_test_core"), &Options { self_delimiting, ..options() });
        let c4 = report.cases.iter().find(|c| c.id == "C4").unwrap();
        assert!(c4.passed, "{:?}", c4.evidence);
        assert_eq!(c4.images.iter().map(|i| (i.image, i.outcome.name())).collect::<Vec<_>>(), vec![("an empty image", "refused"), ("a garbage image", "refused"), ("the image truncated to half", "refused")]);
    }
}

// A core that runs a malformed image passes only where the system's format is open.
#[test]
fn an_accepted_malformed_image_passes_only_where_the_format_is_open() {
    for (self_delimiting, passes) in [(false, true), (true, false)] {
        let report = run(&library("kit_lenient_core"), &Options { self_delimiting, ..options() });
        let c4 = report.cases.iter().find(|c| c.id == "C4").unwrap();
        assert_eq!(c4.passed, passes, "{:?}", c4.evidence);
        assert!(c4.images.iter().any(|i| i.outcome.name() == "accepted and ran"), "{:?}", c4.images);
        assert!(report.json().contains("\"outcome\":\"accepted and ran\""));
    }
}

#[test]
fn a_report_says_what_was_run_and_a_missing_library_fails_c1() {
    let report = run(&library("kit_test_core"), &options());
    let v = json::parse(report.json().as_bytes()).unwrap();
    assert_eq!(v.get("passed"), Some(&Value::Bool(true)));
    assert_eq!(v.get("id").and_then(Value::as_str), Some("v1-test-core"));
    assert_eq!(v.get("cases").and_then(Value::as_array).map(|a| a.len()), Some(15));
    let none = run(&library("no_such_core"), &options());
    assert!(!none.passed());
    assert_eq!(none.cases[0].id, "C1");
}

#[test]
fn the_input_script_holds_releases_and_taps() {
    let s = Script::parse("# comment\nhold 3 0 1\nrelease 9 0 1\ntap 5 1 0x10\n").unwrap();
    assert_eq!(s.steps, vec![(3, 0, 1, 0), (5, 1, 0x10, 0), (6, 1, 0, 0x10), (9, 0, 0, 1)]);
    assert!(Script::parse("press 1 0 1").is_err());
}

#[test]
fn a_sidecar_carries_the_librarys_own_descriptors_and_hash() {
    let dir = std::env::temp_dir().join(format!("emusen-conform-sidecar-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let lib = dir.join(library("kit_test_core").file_name().unwrap());
    std::fs::copy(library("kit_test_core"), &lib).unwrap();
    let path = sidecar::write(&lib).unwrap();
    assert_eq!(path, PathBuf::from(format!("{}.core.json", lib.display())));
    let v = json::parse(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(v.get("sidecar").and_then(Value::as_i64), Some(1));
    assert_eq!(v.get("library").and_then(Value::as_str), lib.file_name().and_then(|n| n.to_str()));
    assert_eq!(v.get("sha256").and_then(Value::as_str), Some(digest::sha256_hex(&std::fs::read(&lib).unwrap()).as_str()));
    assert_eq!(v.get("abi").and_then(Value::as_str), Some("1.0"));
    assert_eq!(v.get("capabilities").and_then(Value::as_i64), Some(0x3FFFF));
    assert_eq!(v.get("info").and_then(|i| i.get("id")).and_then(Value::as_str), Some("v1-test-core"));
    assert_eq!(v.get("settings").and_then(Value::as_array).map(|a| a.len()), Some(4));
    std::fs::remove_dir_all(&dir).unwrap();
}
