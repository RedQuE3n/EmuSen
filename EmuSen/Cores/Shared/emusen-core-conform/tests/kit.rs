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
    Options { self_delimiting: true, image: image(&[1, 2, 3, 0x20, 0x41]), frames: 120, settings: String::new(), files: Vec::new(), script: None, on_disk: None }
}

fn verdicts(name: &str) -> Vec<(&'static str, bool)> {
    let report = run(&library(name), &options());
    for c in report.cases.iter().filter(|c| !c.passed) {
        eprintln!("{name} {} {}: {:?}", c.id, c.name, c.evidence);
    }
    report.cases.iter().map(|c| (c.id, c.passed)).collect()
}

const ALL: [&str; 15] = ["C1", "C2", "C3", "C4", "C5", "C6", "C7", "C8", "C9", "C10", "C11", "C12", "C13", "C14", "C15"];

/// The cases a core is run through: the fifteen, C16 where it claims MEDIA, and C17 to C21.
fn cases(media: bool) -> Vec<&'static str> {
    ALL.iter().chain(media.then_some(&"C16")).chain(&["C17", "C18", "C19", "C20", "C21"]).copied().collect()
}

#[test]
fn both_test_cores_pass_every_case() {
    for (name, media) in [("kit_test_core", true), ("kit_plain_core", false)] {
        let expected: Vec<_> = cases(media).into_iter().map(|c| (c, true)).collect();
        assert_eq!(verdicts(name), expected, "{name}: C16 runs where MEDIA is claimed");
    }
}

// C17 on an image that came from a file maps the file, as a host that lends it does, and runs the disc swap through
// a second mapping; on a core without BORROW it lends nothing and still overwrites its own bytes after create.
#[test]
fn c17_lends_a_mapped_image_where_borrow_is_claimed_and_overwrites_its_own_everywhere() {
    let file = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("kit-c17.v1tc");
    std::fs::write(&file, image(&(0..180).collect::<Vec<u8>>())).unwrap();
    let on_disk = emusen_core_conform::media::on_disk(&file).unwrap();
    let opts = Options { image: std::fs::read(&file).unwrap(), on_disk: Some(on_disk), ..options() };
    let of = |name: &str, opts: &Options| run(&library(name), opts).cases.into_iter().find(|c| c.id == "C17").unwrap();
    let lent = of("kit_test_core", &opts);
    assert!(lent.passed, "{:?}", lent.evidence);
    assert_eq!(lent.evidence, ["120 frames and 120 after a disc swap: the image overwritten after create and after the insert changes nothing; lent (1 file(s) mapped, 186 bytes), every frame's picture, sound and state as from a copy"]);
    let unmapped = of("kit_test_core", &options());
    assert!(unmapped.passed && unmapped.evidence[0].contains("lent (the kit's own bytes kept in place, the image having come from no file)"), "{:?}", unmapped.evidence);
    let plain = of("kit_plain_core", &opts);
    assert!(plain.passed, "{:?}", plain.evidence);
    assert_eq!(plain.evidence, ["120 frames: the image overwritten after create changes nothing; BORROW is not claimed, so nothing was lent"]);
}

// C18 on an image that came from a file maps it and advises the system at each place the drive names, as a host that
// reads ahead does; a core that names nothing passes with that said, and one that names a byte past its image fails.
#[test]
fn c18_checks_every_place_a_drive_names_and_runs_the_same_whether_the_host_acts() {
    let file = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("kit-c18.v1tc");
    std::fs::write(&file, image(&(0..200).collect::<Vec<u8>>())).unwrap();
    let on_disk = emusen_core_conform::media::on_disk(&file).unwrap();
    let opts = Options { image: std::fs::read(&file).unwrap(), on_disk: Some(on_disk), ..options() };
    let of = |name: &str, opts: &Options| run(&library(name), opts).cases.into_iter().find(|c| c.id == "C18").unwrap();
    let named = of("kit_test_core", &opts);
    assert!(named.passed, "{:?}", named.evidence);
    assert_eq!(named.evidence, ["120 frames: 120 places named, each a drive of machine info and a byte inside a file of the call; acted on (24 KiB advised of the mapped files) and skipped, every frame's picture, sound and state the same"]);
    let unmapped = of("kit_test_core", &options());
    assert!(unmapped.passed && unmapped.evidence[0].contains("(0 KiB advised of the mapped files)"), "an image from no file has nothing to advise: {:?}", unmapped.evidence);
    let silent = of("kit_plain_core", &opts);
    assert_eq!((silent.passed, &silent.evidence[..]), (true, &["120 frames: no MEDIA_SEEK was raised, so there was nothing to check or to act on".to_owned()][..]));
}

// Machines that differ and a truncated state accepted break C4, C7, C8 and C10, and through them C9's and C13's
// comparisons with a solo run, C12's rerun of C6-C8 and C17's and C18's of one run with another.
#[test]
fn the_seeded_faults_fail_the_cases_they_break_and_no_others() {
    let failing = ["C4", "C7", "C8", "C9", "C10", "C12", "C13", "C17", "C18"];
    assert_eq!(verdicts("kit_faulty_core"), cases(false).into_iter().map(|c| (c, !failing.contains(&c))).collect::<Vec<_>>());
}

// An exact setting that changes the state, a refusal of any host but 1.0, info that drifts, a writable read-only
// space, an observed frame with other sound, a lent image read otherwise than its copy and a drive that names a place
// past its image's end break C9, C12, C13, C14, C15, C17 and C18, a picture answered on a device nobody lent C20, and a
// raw picture its palette does not draw as the frame C21.
#[test]
fn the_faults_of_c9_and_c12_to_c18_fail_those_cases_and_no_others() {
    let failing = ["C9", "C12", "C13", "C14", "C15", "C17", "C18", "C20", "C21"];
    assert_eq!(verdicts("kit_faulty_full_core"), cases(true).into_iter().map(|c| (c, !failing.contains(&c))).collect::<Vec<_>>());
}

// Each of those faults alone, in a runner process of its own, fails exactly the case it is for; a core that takes
// only a host of 1.0 also refuses the hosts of 1.3 and 1.4 that C17 and C18 play to lend, and 1.9's that C20 plays.
#[test]
fn each_fault_of_c9_and_c12_to_c18_alone_fails_its_case_alone() {
    let file = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("kit-faulty-full.v1tc");
    std::fs::write(&file, image(&[1, 2, 3, 0x20, 0x41])).unwrap();
    for (fault, cases) in [(8, vec!["C9"]), (16, vec!["C12", "C17", "C18", "C20"]), (32, vec!["C13"]), (64, vec!["C14"]), (128, vec!["C15"]), (256, vec!["C17"]), (512, vec!["C18"]), (1 << 22, vec!["C14"]), (1 << 23, vec!["C20"]), (1 << 24, vec!["C21"])] {
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
        assert_eq!(failed, cases, "fault {fault}:\n{text}");
        if fault == 1 << 22 {
            assert!(text.contains("battery files 1 and 2 of system test are both of medium test.card, and differ in kind or extension (removable .card against console .mem)") && text.contains("belongs to port 7, which machine info does not list"), "{text}");
        }
        if fault == 256 {
            assert!(text.contains("a machine reading an image its host lent differs from the run on a copy at frame 0, in its state"), "{text}");
        }
        assert_eq!(out.status.code(), Some(1), "fault {fault}");
    }
}

/// The test core's payload whose machine has a card in its slot, removable battery file 1 (`CARD`, since 1.7).
const CARD: &[u8] = &[0xCD, 2, 3, 0x20, 0x41];

// Since 1.7 (EmuSen_CoreAPI.md §6.27): a machine that lists a removable file keeps it as it stands across a state load
// under SHARED_REMOVABLE and passes C19, in either form; one that lists none passes with that said; a core that rolls
// the card back on such a load fails C19 alone, and on a machine with no card the same fault changes nothing.
#[test]
fn c19_holds_a_removable_file_as_it_stands_across_a_state_load() {
    let carded = run(&library("kit_test_core"), &Options { image: image(CARD), ..options() });
    let c19 = carded.cases.iter().find(|c| c.id == "C19").unwrap();
    assert!(carded.cases.iter().all(|c| c.passed), "{:?}", carded.cases.iter().filter(|c| !c.passed).map(|c| (c.id, &c.evidence)).collect::<Vec<_>>());
    assert_eq!(c19.evidence, ["1 removable file(s), file 1 (16 bytes): kept as they stood across a state made 120 frames into a machine whose files held other bytes"]);
    let plain = run(&library("kit_test_core"), &options());
    assert_eq!(plain.cases.iter().find(|c| c.id == "C19").unwrap().evidence, ["the machine lists no removable or console battery file, so nothing is kept apart"]);
    let (failed, text) = failed_on("kit_faulty_full_core", CARD, 1 << 20, 120);
    assert_eq!(failed, ["C19"], "{text}");
    assert!(text.contains("removable battery file 1 changed across the state load: 16 of its 16 bytes differ") && text.contains("reads as changed after the state load"), "{text}");
    let (failed, text) = failed_on("kit_faulty_full_core", &[1, 2, 3, 0x20, 0x41], 1 << 20, 120);
    assert!(failed.is_empty(), "no card, nothing to roll back:\n{text}");
}

/// The test core's payload whose machine has memory inside the console, battery file 2 (`CONSOLE_MEMORY`, since 1.8).
const CONSOLE_MEMORY: &[u8] = &[0xCE, 2, 3, 0x20, 0x41];

// Since 1.8 (EmuSen_CoreAPI.md §6.28): a machine that lists a console file keeps it as it stands across a state load under
// SHARED_CONSOLE and passes C19; a core that rolls the console's memory back on such a load fails C19 alone, and the
// card's fault, with no card, changes nothing there.
#[test]
fn c19_holds_a_console_file_as_it_stands_across_a_state_load() {
    let held = run(&library("kit_test_core"), &Options { image: image(CONSOLE_MEMORY), ..options() });
    assert!(held.cases.iter().all(|c| c.passed), "{:?}", held.cases.iter().filter(|c| !c.passed).map(|c| (c.id, &c.evidence)).collect::<Vec<_>>());
    assert_eq!(held.cases.iter().find(|c| c.id == "C19").unwrap().evidence, ["1 console file(s), file 2 (16 bytes): kept as they stood across a state made 120 frames into a machine whose files held other bytes"]);
    let (failed, text) = failed_on("kit_faulty_full_core", CONSOLE_MEMORY, 1 << 21, 120);
    assert_eq!(failed, ["C19"], "{text}");
    assert!(text.contains("console battery file 2 changed across the state load: 16 of its 16 bytes differ"), "{text}");
    let (failed, text) = failed_on("kit_faulty_full_core", CONSOLE_MEMORY, 1 << 20, 120);
    assert!(failed.is_empty(), "the card's fault on a machine with no card:\n{text}");
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
    assert_eq!(v.get("cases").and_then(Value::as_array).map(|a| a.len()), Some(21));
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
    assert_eq!(v.get("abi").and_then(Value::as_str), Some("1.10"));
    assert_eq!(v.get("capabilities").and_then(Value::as_i64), Some(0x7FFFFF));
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
    corpus::run(&library(name), &Options { image: first, ..options() }, &list[0].0, list, jobs, u64::MAX, None, std::time::Duration::from_secs(120), &mut |_| {})
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
            let expected: Vec<_> = corpus::IMAGE_CASES.iter().filter(|&&id| id != "C16" || name == "kit_test_core").map(|&id| (id, true)).collect();
            let c17 = i.report.cases.iter().find(|x| x.id == "C17").unwrap();
            assert!(name != "kit_test_core" || c17.evidence[0].contains("1 file(s) mapped"), "a corpus's image is lent from its file: {:?}", c17.evidence);
            let c18 = i.report.cases.iter().find(|x| x.id == "C18").unwrap();
            assert!(c18.evidence[0].contains(if name == "kit_test_core" { "120 places named" } else { "no MEDIA_SEEK was raised" }), "{:?}", c18.evidence);
            assert_eq!(i.report.cases.iter().map(|x| (x.id, x.passed)).collect::<Vec<_>>(), expected, "{name} {}: C16 where MEDIA is claimed", i.name);
            assert_eq!(i.report.image.as_deref(), Some(i.name.as_str()));
        }
        assert!(c.passed());
        let all: std::collections::BTreeSet<&str> = corpus::CORE_CASES.iter().chain(corpus::IMAGE_CASES.iter()).copied().collect();
        assert_eq!(all, ALL.iter().chain(&["C16", "C17", "C18", "C19", "C20", "C21"]).copied().collect());
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
                let single = run(&library(name), &Options { image: std::fs::read(&list[n].1).unwrap(), on_disk: emusen_core_conform::media::on_disk(&list[n].1).ok(), ..options() });
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
    for (name, core, per_image) in [("kit_faulty_core", vec!["C4", "C9", "C10", "C12", "C13"], vec!["C7", "C8", "C17", "C18"]), ("kit_faulty_full_core", vec!["C9", "C12", "C13", "C15"], vec!["C14", "C17", "C18", "C20", "C21"])] {
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
            assert_eq!(report.get("cases").and_then(Value::as_array).map(|a| a.len()), Some(11), "the five image cases, C16, MEDIA being claimed, C17, C18, C19, C20 and C21");
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

/// The runner on `core` with the faults `mask` leaves on, over `frames`: the cases failed, and its output.
fn failed_with(core: &str, mask: u32, frames: u64) -> (Vec<String>, String) {
    failed_on(core, &[1, 2, 3, 0x20, 0x41], mask, frames)
}

/// The same on an image of `payload`.
fn failed_on(core: &str, payload: &[u8], mask: u32, frames: u64) -> (Vec<String>, String) {
    let file = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("kit-{core}-{:02x}-{mask}.v1tc", payload[0]));
    std::fs::write(&file, image(payload)).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_emusen-core-conform"))
        .args(["--self-delimiting", "--frames", &frames.to_string(), "--core"])
        .arg(library(core))
        .arg("--image")
        .arg(&file)
        .env("EMUSEN_TEST_CORE_FAULTS", mask.to_string())
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    (text.lines().filter_map(|l| l.strip_prefix("FAIL ")).filter_map(|l| l.split_whitespace().next()).map(str::to_owned).collect(), text)
}

// Since 1.5 (EmuSen_CoreAPI.md §28.10): pictures that differ for the three frames after a load pass where machine info
// declares three, and C7 and C8 say from which frame they compared; a picture that never settles, one that differs
// undeclared and sound that differs while it settles fail C7, C8 and C12, which runs them again (until C7 held each
// loaded machine to one that never loaded, §28.14, its two machines both loaded the state and agreed, and it passed);
// a bound past the maximum, or one the run is too short to check past, fails them too.
#[test]
fn pictures_that_settle_pass_within_their_bound_and_each_fault_of_the_rule_fails() {
    let (failed, text) = failed_with("kit_settling_core", 1024, 120);
    assert!(failed.is_empty(), "{text}");
    assert!(text.contains("compared from its frame 4 on, machine info's state.picture_settles being 3"), "{text}");
    let (failed, text) = failed_with("kit_settling_core", 0, 120);
    assert!(failed.is_empty() && !text.contains("picture_settles"), "a core declaring nothing is held as before: {text}");
    for (mask, cases) in [(1024 | 2048, vec!["C7", "C8", "C12"]), (4096, vec!["C7", "C8", "C12"]), (8192, vec!["C7", "C8", "C12"]), (16384, vec!["C7", "C8", "C12"])] {
        let (failed, text) = failed_with("kit_settling_core", mask, 120);
        assert_eq!(failed, cases, "faults {mask}:\n{text}");
    }
    let (failed, text) = failed_with("kit_settling_core", 1024, 6);
    assert_eq!(failed, ["C7", "C8", "C12"], "{text}");
    assert!(text.contains("run more than 6 frames"), "{text}");
}

// The corpus form's deadline is on progress (§28.10): a core whose image's cases take longer than the deadline, every
// frame ending, passes; a frame that never ends is reported hung once the deadline has passed with no frame run, and
// the images beside it pass.
#[test]
fn the_deadline_is_on_progress_not_on_the_whole_image() {
    let (_, list) = corpus_files("slow");
    let one = &list[..1];
    let started = std::time::Instant::now();
    let slow = corpus::run(&library("kit_slow_core"), &Options { image: std::fs::read(&one[0].1).unwrap(), frames: 60, ..options() }, &one[0].0, one, 1, u64::MAX, None, std::time::Duration::from_secs(1), &mut |_| {});
    assert!(slow.passed() && slow.images[0].seconds > 1.0, "{:?} in {:.1} s", slow.images[0].failed(), slow.images[0].seconds);
    eprintln!("the slow core's image took {:.1} s against a deadline of 1 s; the run {:.1} s", slow.images[0].seconds, started.elapsed().as_secs_f64());

    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("kit-corpus-hung");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut list = Vec::new();
    for (name, payload) in [("image 0.v1tc", vec![1, 2, 3]), ("image 1.v1tc", vec![0xEE, 1, 2])] {
        std::fs::write(dir.join(name), image(&payload)).unwrap();
        list.push((name.to_owned(), dir.join(name)));
    }
    let hung = corpus::run(&library("kit_hung_core"), &Options { image: std::fs::read(&list[0].1).unwrap(), ..options() }, &list[0].0, &list, 2, u64::MAX, None, std::time::Duration::from_secs(2), &mut |_| {});
    assert!(hung.core.passed() && hung.images[0].report.passed(), "{:?}", hung.images[0].failed());
    assert_eq!(hung.images[1].failed(), ["C6"]);
    assert_eq!(hung.images[1].report.cases[0].evidence, ["the image's cases ran no frame for 2 s; its thread was left running"]);
}

/// The test core's payload that makes a cartridge, whose machine lists no drive (`CARTRIDGE`, since 1.6).
const CARTRIDGE: &[u8] = &[0xCA, 2, 3, 0x20, 0x41];

// Since 1.6 (EmuSen_CoreAPI.md §6.26, §28.11): one MEDIA library whose disc lists a drive and whose cartridge lists
// none passes every case on both, in either form; C16 holds the drive rules on the disc and, on the cartridge, each
// media call refused with EMUSEN_NO_SUCH_DRIVE and nothing changed; the cartridge's drive names no place.
#[test]
fn a_media_core_passes_c16_on_a_machine_with_a_drive_and_on_one_with_none() {
    fn of<'a>(report: &'a emusen_core_conform::Report, id: &str) -> &'a emusen_core_conform::Case {
        report.cases.iter().find(|c| c.id == id).unwrap()
    }
    let disc = run(&library("kit_test_core"), &options());
    let cartridge = run(&library("kit_test_core"), &Options { image: image(CARTRIDGE), ..options() });
    for (name, report) in [("disc", &disc), ("cartridge", &cartridge)] {
        assert_eq!(report.cases.iter().map(|c| (c.id, c.passed)).collect::<Vec<_>>(), cases(true).into_iter().map(|c| (c, true)).collect::<Vec<_>>(), "{name}: {:?}", report.cases.iter().filter(|c| !c.passed).map(|c| &c.evidence).collect::<Vec<_>>());
    }
    assert!(of(&disc, "C16").evidence[0].starts_with("drive 0: insert refused closed"), "{:?}", of(&disc, "C16").evidence);
    assert_eq!(of(&cartridge, "C16").evidence, ["machine info lists no drive: the lid opened and closed and a disc, an empty drive and a restore inserted on drives 0 and 4294967295, each refused with EMUSEN_NO_SUCH_DRIVE, the machine unchanged and running 60 frames after"]);
    assert_eq!(of(&cartridge, "C18").evidence, ["120 frames: no MEDIA_SEEK was raised, so there was nothing to check or to act on"]);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("kit-corpus-cartridge");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let list: Vec<(String, PathBuf)> = [("disc.v1tc", &[1u8, 2, 3, 0x20, 0x41][..]), ("cartridge.v1tc", CARTRIDGE)]
        .iter()
        .map(|(name, payload)| {
            std::fs::write(dir.join(name), image(payload)).unwrap();
            (name.to_string(), dir.join(name))
        })
        .collect();
    let c = run_corpus("kit_test_core", &list, 2);
    assert!(c.passed(), "{:?}", c.images.iter().map(|i| i.failed()).collect::<Vec<_>>());
}

// The negative controls: a cartridge's machine that lists no drive but takes a media call, or answers one with a
// status of the fault band, fails C16 alone; one that names a place for the drive it lacks fails C18 alone. On a disc,
// whose machine lists its drive, the same faults change nothing.
#[test]
fn a_machine_listing_no_drive_that_takes_a_media_call_faults_or_names_a_place_fails() {
    for (mask, cases, words) in [
        (131072, vec!["C16"], "media_lid(0, open) answered 0, not EMUSEN_NO_SUCH_DRIVE"),
        (262144, vec!["C16"], "media_lid(0, open) answered -321, not EMUSEN_NO_SUCH_DRIVE"),
        (524288, vec!["C18"], "MEDIA_SEEK names drive 0, which machine info does not list ([])"),
    ] {
        let (failed, text) = failed_on("kit_faulty_full_core", CARTRIDGE, mask, 120);
        assert_eq!(failed, cases, "faults {mask}:\n{text}");
        assert!(text.contains(words), "faults {mask}:\n{text}");
        let (failed, text) = failed_on("kit_faulty_full_core", &[1, 2, 3, 0x20, 0x41], mask, 120);
        assert!(failed.is_empty(), "faults {mask} on a disc:\n{text}");
    }
}

// Since 2026-10-15 (EmuSen_CoreAPI.md §28.14): a state that leaves out what the picture shows from frame 90 on passes a
// round trip at frame 60, C8's middle and C7's one point before, over 120 frames; C7's points catch it at frame 90 and
// after, each failure naming its point, and over 240 frames C8's middle at 120 catches it too.
#[test]
fn a_state_wrong_only_late_in_the_run_fails_c7_at_its_points_where_one_middle_misses_it() {
    let (failed, text) = failed_with("kit_settling_core", 1 << 25, 120);
    assert_eq!(failed, ["C7", "C12"], "{text}");
    assert!(text.contains("frame 90: a machine just made and given the state saved there did not continue as the machine that saved it and never loaded, its pictures differing by frame 105"), "{text}");
    assert!(!text.contains("frame 75: ") && text.contains("saved at frames 15, 30, 45, 60, 75, 90, 105, 112 of 120"), "{text}");
    let (failed, text) = failed_with("kit_settling_core", 1 << 25, 240);
    assert_eq!(failed, ["C7", "C8", "C12"], "{text}");
    assert!(text.contains("across a save and load at frame 120 the run differs"), "{text}");
}
