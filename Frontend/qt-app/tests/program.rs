//! The program's window through the harness (EmuSen_Qt.md §7, §8.1): Moonrise over Home at the start with the systems'
//! planets, a pad press skipping it, and Home holding the focus once it has gone and the library is read.

mod common;

use common::{sandbox, wait_for};
use emusen_client_fixture::library;
use emusen_kit_core::input::UiButton;
use emusen_qt::MAIN_URL;
use emusen_qt_test::Harness;

#[test]
fn moonrise_plays_over_home_and_a_press_skips_it() {
    let _world = sandbox("qt-app-program", &[("NES", "Synthetic.nes")]);
    assert!(emusen_qt::start::read(library()).splash, "the settings saved leave the splash on");

    let mut harness = Harness::new();
    let mut scene = harness.load(MAIN_URL, 1280, 800);
    assert_eq!(scene.property("moonrise", "visible"), "true", "Moonrise covers the window from the start");
    assert!(scene.set_property("program", "platformLibrary", library().to_str().unwrap()));
    wait_for(&mut scene, "moonrise", "running", "true", 5000);
    for _ in 0..100 {
        if !scene.property("moonrise", "caption").is_empty() {
            break;
        }
        scene.wait(50);
    }
    let caption = scene.property("moonrise", "caption");
    assert!(caption.ends_with("CONSOLES") && caption != "0 CONSOLES", "the caption counts the systems with an engine: {caption}");
    assert_eq!(scene.property("program", "splashShown"), "true");
    assert_eq!(scene.property("moonrise", "finished"), "false", "the splash is still drawing");

    scene.pad(UiButton::Accept);
    wait_for(&mut scene, "moonrise", "finished", "true", 1000);
    wait_for(&mut scene, "library", "loading", "false", 10000);
    wait_for(&mut scene, "moonrise", "visible", "false", 2000);
    assert_eq!(scene.property("program", "splashShown"), "false");
    assert_eq!(scene.property("home", "activeFocus"), "true", "Home takes the focus once the splash has gone");
    assert_eq!(scene.property("library", "selectedKey"), "all", "the press was the splash's alone");
    scene.assert_quiet();
}

#[test]
fn the_program_s_scenes_keep_the_qml_rules() {
    let qml = emusen_qt_test::lint::frontend().join("qt-app").join("qml");
    let lines = emusen_qt_test::lint::rules(&[(&qml, false)]);
    assert!(lines.is_empty(), "{}", lines.join("\n"));
    let complaints = emusen_qt_test::lint::qmllint(&[&qml]);
    assert!(complaints.is_empty(), "{}", complaints.join("\n"));
}

#[test]
fn qt_3_4_the_reference_theme_keeps_the_theme_rules() {
    // The package as any theme is held: its imports to the theme's list and §1.3's rules (EmuSen_Qt.md §3.5, §3.7).
    let package = emusen_qt_test::lint::frontend().join("themes").join("emusen");
    let mut found = Vec::new();
    for file in emusen_qml_rules::qml_files(&package) {
        let text = std::fs::read_to_string(&file).unwrap();
        found.extend(emusen_qml_rules::check_theme(&file.to_string_lossy(), &text).iter().map(ToString::to_string));
    }
    assert!(found.is_empty(), "{}", found.join("\n"));
    let complaints = emusen_qt_test::lint::qmllint(&[&package]);
    assert!(complaints.is_empty(), "{}", complaints.join("\n"));
}

/// What the program writes to stderr from its start until it says it has made its application and read its library
/// ("No game can be played", with no library), at most ten seconds; then it is stopped.
fn program_start_words() -> String {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};
    let home = std::env::temp_dir().join(format!("qt-app-main-thread-{}", std::process::id()));
    std::fs::create_dir_all(&home).unwrap();
    // No platform library, so no device is opened: the offscreen software scene alone, never the graphics card.
    let mut child = Command::new(env!("CARGO_BIN_EXE_emusen-qt"))
        .arg("--platform-library")
        .arg(home.join("no-such-library.so"))
        .env("HOME", &home)
        .env("QT_QPA_PLATFORM", "offscreen")
        .env("QT_QUICK_BACKEND", "software")
        .env("QT_FORCE_STDERR_LOGGING", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the program starts");
    let (sent, got) = std::sync::mpsc::channel();
    let err = child.stderr.take().unwrap();
    std::thread::spawn(move || {
        for line in BufReader::new(err).lines().map_while(Result::ok) {
            if sent.send(line).is_err() {
                break;
            }
        }
    });
    let mut words = Vec::new();
    let until = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while let Some(left) = until.checked_duration_since(std::time::Instant::now()) {
        match got.recv_timeout(left) {
            Ok(line) => {
                let made = line.contains("No game can be played");
                words.push(line);
                if made {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    let _ = child.kill();
    let _ = child.wait();
    let _ = std::fs::remove_dir_all(&home);
    words.join("\n")
}

#[test]
fn kit_7_105_the_program_itself_never_prints_the_main_thread_line() {
    // R-20261111-110 accepts the line in test programs only: the program makes its application on its own main thread.
    let words = program_start_words();
    assert!(words.contains("No game can be played"), "the program made its application and said so: {words}");
    assert!(!words.contains("QApplication was not created in the main() thread"), "the program printed the main-thread line:\n{words}");
}
