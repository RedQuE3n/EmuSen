//! The kit on the test cores: both pass every case, the core with seeded faults fails exactly the cases its faults
//! break, and a sidecar records the library as the host reads it.

use std::path::PathBuf;

use emusen_core_conform::{Options, Script, corpus, digest, run, sidecar};
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
    let path = sidecar::write(&lib, false).unwrap();
    assert_eq!(path, PathBuf::from(format!("{}.core.json", lib.display())));
    let v = json::parse(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(v.get("sidecar").and_then(Value::as_i64), Some(1));
    assert_eq!(v.get("library").and_then(Value::as_str), lib.file_name().and_then(|n| n.to_str()));
    assert_eq!(v.get("sha256").and_then(Value::as_str), Some(digest::sha256_hex(&std::fs::read(&lib).unwrap()).as_str()));
    assert_eq!(v.get("abi").and_then(Value::as_str), Some("1.0"));
    assert_eq!(v.get("capabilities").and_then(Value::as_i64), Some(0x3FFFF));
    assert_eq!(v.get("info").and_then(|i| i.get("id")).and_then(Value::as_str), Some("v1-test-core"));
    assert_eq!(v.get("settings").and_then(Value::as_array).map(|a| a.len()), Some(4));
    assert!(v.get("development").is_none());
    let marked = json::parse(&std::fs::read(sidecar::write(&lib, true).unwrap()).unwrap()).unwrap();
    assert_eq!(marked.get("development").map(|d| matches!(d, Value::Bool(true))), Some(true));
    assert_eq!(marked.get("info").and_then(|i| i.get("id")).and_then(Value::as_str), Some("v1-test-core"));
    std::fs::remove_dir_all(&dir).unwrap();
}

/// A system marked in development is listed, and refused where it shares an extension with a system the core offers (EmuSen_CoreAPI.md §27.4).
#[test]
fn a_system_in_development_shares_no_extension_with_an_offered_one() {
    let info = |second: &str| format!(r#"{{"systems":[{{"id":"md","extensions":[".md",".bin"]}},{{"id":"mcd","extensions":[{second}],"development":true}},{{"id":"32x","extensions":[".32x"],"development":false}}]}}"#);
    assert_eq!(emusen_core_conform::systems_in_development(&info(r#"".iso""#)), (Vec::<String>::new(), vec!["mcd".to_owned()]));
    let (problems, ids) = emusen_core_conform::systems_in_development(&info(r#"".iso",".BIN""#));
    assert_eq!((problems.len(), ids), (1, vec!["mcd".to_owned()]));
    assert!(problems[0].contains(".BIN"), "{problems:?}");
    assert_eq!(emusen_core_conform::systems_in_development(r#"{"systems":[{"id":"snes","extensions":[".sfc"]}]}"#), (Vec::<String>::new(), Vec::<String>::new()));
}

/// Three images of the test core's format, written where the corpus form can read them, and a list naming them.
fn corpus_files(tag: &str) -> (PathBuf, Vec<(String, PathBuf)>) {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("kit-corpus-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let payloads: [&[u8]; 3] = [&[1, 2, 3, 0x20, 0x41], &[9, 8, 7, 6, 5, 4, 3, 2], &[0x55; 40]];
    let mut list = Vec::new();
    for (n, payload) in payloads.iter().enumerate() {
        let name = format!("image {n}.v1tc");
        std::fs::write(dir.join(&name), image(payload)).unwrap();
        list.push((name.clone(), dir.join(name)));
    }
    (dir, list)
}

fn run_corpus(name: &str, list: &[(String, PathBuf)], jobs: usize) -> corpus::Corpus {
    let first = std::fs::read(&list[0].1).unwrap();
    corpus::run(&library(name), &Options { image: first, ..options() }, &list[0].0, list, jobs, std::time::Duration::from_secs(120), &mut |_| {})
}

// The corpus form runs each case in one place: the core's once, the image's for every image, C4 in both by its parts; and the test cores pass both.
#[test]
fn a_corpus_runs_the_cores_cases_once_and_the_images_cases_for_every_image() {
    let (_, list) = corpus_files("pass");
    for name in ["kit_test_core", "kit_plain_core"] {
        let c = run_corpus(name, &list, 3);
        assert_eq!(c.core.cases.iter().map(|x| (x.id, x.passed)).collect::<Vec<_>>(), corpus::CORE_CASES.iter().map(|&id| (id, true)).collect::<Vec<_>>(), "{name}");
        assert_eq!(c.images.iter().map(|i| i.name.as_str()).collect::<Vec<_>>(), list.iter().map(|l| l.0.as_str()).collect::<Vec<_>>());
        for i in &c.images {
            assert_eq!(i.report.cases.iter().map(|x| (x.id, x.passed)).collect::<Vec<_>>(), corpus::IMAGE_CASES.iter().map(|&id| (id, true)).collect::<Vec<_>>(), "{name} {}", i.name);
            assert_eq!(i.report.image.as_deref(), Some(i.name.as_str()));
        }
        assert!(c.passed());
        let all: std::collections::BTreeSet<&str> = corpus::CORE_CASES.iter().chain(corpus::IMAGE_CASES.iter()).copied().collect();
        assert_eq!(all, ALL.iter().copied().collect());
    }
}

// For each image the corpus form's cases say what the single-image form says of it, verdict and evidence, whatever the number run at once; C4's parts together are the single form's C4.
#[test]
fn a_corpus_says_of_each_image_what_the_single_image_form_says() {
    let (_, list) = corpus_files("same");
    for name in ["kit_test_core", "kit_faulty_core", "kit_lenient_core"] {
        for jobs in [1, 3] {
            let c = run_corpus(name, &list, jobs);
            for (n, i) in c.images.iter().enumerate() {
                let single = run(&library(name), &Options { image: std::fs::read(&list[n].1).unwrap(), ..options() });
                let of = |id: &str| single.cases.iter().find(|x| x.id == id).unwrap();
                // The faulty core's machines each differ from the last, so its digests are no two runs' alike; its verdicts are.
                for case in i.report.cases.iter().filter(|x| x.id != "C4") {
                    assert_eq!(case.passed, of(case.id).passed, "{name} {} {} at {jobs}", i.name, case.id);
                    if name != "kit_faulty_core" {
                        assert_eq!(&case.evidence, &of(case.id).evidence, "{name} {} {} at {jobs}", i.name, case.id);
                    }
                }
                let c4_image = i.report.cases.iter().find(|x| x.id == "C4").unwrap();
                assert_eq!(c4_image.images.iter().map(|x| (x.image, x.outcome.name())).collect::<Vec<_>>(), of("C4").images[2..].iter().map(|x| (x.image, x.outcome.name())).collect::<Vec<_>>());
            }
            let reference = run(&library(name), &Options { image: std::fs::read(&list[0].1).unwrap(), ..options() });
            let of = |id: &str| reference.cases.iter().find(|x| x.id == id).unwrap();
            for case in c.core.cases.iter().filter(|x| !matches!(x.id, "C4" | "C13")) {
                assert_eq!(case.passed, of(case.id).passed, "{name} core {} at {jobs}", case.id);
                if name != "kit_faulty_core" && case.id != "C11" {
                    assert_eq!(&case.evidence, &of(case.id).evidence, "{name} core {} at {jobs}", case.id);
                }
            }
            let c4_core = c.core.cases.iter().find(|x| x.id == "C4").unwrap();
            assert_eq!(c4_core.images.iter().map(|x| (x.image, x.outcome.name())).collect::<Vec<_>>(), of("C4").images[..2].iter().map(|x| (x.image, x.outcome.name())).collect::<Vec<_>>());
            assert_eq!(c4_core.passed && c.images[0].report.cases[0].passed, of("C4").passed, "{name}: C4's parts against the whole");
            assert_eq!(c.core.cases.iter().find(|x| x.id == "C13").unwrap().passed, of("C13").passed);
        }
    }
}

// The faults fail in the corpus form the cases they fail in the single-image form, each where it is run.
#[test]
fn the_seeded_faults_fail_the_same_cases_in_a_corpus() {
    let (_, list) = corpus_files("faults");
    for (name, core, per_image) in [("kit_faulty_core", vec!["C4", "C9", "C10", "C12", "C13"], vec!["C7", "C8"]), ("kit_faulty_full_core", vec!["C9", "C12", "C13", "C15"], vec!["C14"])] {
        let c = run_corpus(name, &list, 2);
        assert_eq!(c.core.cases.iter().filter(|x| !x.passed).map(|x| x.id).collect::<Vec<_>>(), core, "{name}");
        for i in &c.images {
            assert_eq!(i.failed(), per_image, "{name} {}", i.name);
        }
        assert!(!c.passed());
    }
}

// The runner's corpus form: a line for each image, a report for each and the summary, and the exit status the verdict.
#[test]
fn the_runner_takes_a_directory_or_a_list_and_writes_a_report_for_each_image() {
    let (dir, list) = corpus_files("runner");
    let reports = dir.join("reports");
    let names = dir.join("some.txt");
    std::fs::write(&names, format!("# two of the three\n{}\n\n{}  # the last\n", list[0].0, list[2].0)).unwrap();
    for (images, count, from) in [(dir.clone(), 4, None), (names.clone(), 2, None), (names.clone(), 2, Some(dir.clone()))] {
        let _ = std::fs::remove_dir_all(&reports);
        let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_emusen-core-conform"));
        command.args(["--self-delimiting", "--frames", "120", "--jobs", "2", "--core"]).arg(library("kit_test_core")).arg("--images").arg(&images).arg("--image").arg(&list[0].1).arg("--report-dir").arg(&reports);
        if let Some(from) = &from {
            command.arg("--from").arg(from);
        }
        let out = command.output().unwrap();
        let text = String::from_utf8_lossy(&out.stdout);
        let summary = json::parse(&std::fs::read(reports.join("summary.json")).unwrap()).unwrap();
        let images_listed = summary.get("images").and_then(Value::as_array).unwrap();
        assert_eq!(images_listed.len(), count, "{text}");
        // The directory holds the list file too, which is no image of the format: refused, so its C4 passes and C6 fails.
        let failing = images_listed.iter().filter(|i| i.get("passed") != Some(&Value::Bool(true))).count();
        assert_eq!(failing, if count == 4 { 1 } else { 0 }, "{text}");
        assert_eq!(out.status.code(), Some(if failing == 0 { 0 } else { 1 }), "{text}");
        assert_eq!(summary.get("core").and_then(|c| c.get("cases")).and_then(Value::as_array).map(|a| a.len()), Some(11));
        for i in images_listed {
            let file = i.get("report").and_then(Value::as_str).unwrap();
            let report = json::parse(&std::fs::read(reports.join(file)).unwrap()).unwrap();
            assert_eq!(report.get("image"), i.get("image"));
            assert_eq!(report.get("cases").and_then(Value::as_array).map(|a| a.len()), Some(5));
        }
        assert_eq!(text.lines().filter(|l| l.starts_with("pass [") || l.starts_with("FAIL [")).count(), count, "{text}");
    }
    let missing = dir.join("missing.txt");
    std::fs::write(&missing, "image 0.v1tc\nno such image.v1tc\n").unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_emusen-core-conform")).arg("--core").arg(library("kit_test_core")).arg("--images").arg(&missing).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("no such image.v1tc"));
}

// A single image's report carries no image name, as before the corpus form.
#[test]
fn a_single_image_report_is_as_it_was() {
    let report = run(&library("kit_test_core"), &options());
    assert!(report.image.is_none());
    assert!(!report.json().contains("\"image\":\"image"));
    assert!(report.json().starts_with("{\"kit\":\"emusen-core-conform "));
    assert!(report.json().contains("\"library\":") && !json::parse(report.json().as_bytes()).unwrap().get("image").is_some());
}
