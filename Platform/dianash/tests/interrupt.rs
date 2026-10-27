//! The interrupt: a running line stops at its next check, prints `^C`, leaves `$?` at 130 and clears the flag
//! (EmuSen_DianaSh.md §10.2).

use emusen_dianash::shell::{Config, Shell};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

fn shell() -> Shell {
    let root = std::env::temp_dir().join(format!("dianash-interrupt-{:?}", std::thread::current().id()));
    std::fs::create_dir_all(&root).unwrap();
    let mut shell = Shell::new(Config { sessions: false, root: Some(root.to_string_lossy().into_owned()), ..Config::default() });
    shell.add_module(&dianash_text::MODULE).unwrap();
    shell
}

/// Runs `line`, interrupts it from another thread after 30 ms, and answers its output, status and the time past the interrupt.
fn interrupted(shell: &mut Shell, line: &str) -> (String, i32, Duration) {
    let flag = shell.cancel_flag();
    let raised = std::sync::Arc::new(std::sync::Mutex::new(None));
    let stamp = raised.clone();
    let thread = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(30));
        *stamp.lock().unwrap() = Some(Instant::now());
        flag.store(true, Ordering::Relaxed);
    });
    shell.submit(0, line).unwrap();
    let ended = Instant::now();
    thread.join().unwrap();
    let late = ended.duration_since(raised.lock().unwrap().unwrap());
    (shell.take_output(), shell.last_status(0).unwrap(), late)
}

fn run(shell: &mut Shell, line: &str) -> (String, i32) {
    shell.submit(0, line).unwrap();
    (shell.take_output(), shell.last_status(0).unwrap())
}

#[test]
fn a_while_true_stops_within_100_ms_with_130_and_a_caret_c() {
    let mut shell = shell();
    let (output, status, late) = interrupted(&mut shell, "while true; do echo x | grep x; done");
    assert!(late < Duration::from_millis(100), "{late:?}");
    assert_eq!(status, 130);
    assert!(output.ends_with("^C"), "{output:?}");
}

#[test]
fn the_next_command_runs_normally() {
    let mut shell = shell();
    interrupted(&mut shell, "while true; do true; done");
    assert_eq!(run(&mut shell, "echo ok"), ("ok".to_string(), 0));
}

#[test]
fn an_interrupt_with_nothing_running_does_nothing() {
    let mut shell = shell();
    shell.cancel_flag().store(true, Ordering::Relaxed);
    assert_eq!(run(&mut shell, "echo ok"), ("ok".to_string(), 0));
    assert!(!shell.cancel_flag().load(Ordering::Relaxed));
}

#[test]
fn until_and_for_and_a_pipeline_in_a_loop_stop_too() {
    for line in ["until false; do true; done", "for i in 1 2 3; do while true; do true; done; done", "while true; do echo a | grep a | grep a; done"] {
        let mut shell = shell();
        let (output, status, late) = interrupted(&mut shell, line);
        assert!(late < Duration::from_millis(100), "{line}: {late:?}");
        assert_eq!((status, output.ends_with("^C")), (130, true), "{line}");
    }
}

#[test]
fn a_sourced_script_and_a_substitution_stop_with_the_line() {
    let mut shell = shell();
    let (_, status, late) = interrupted(&mut shell, "echo $(while true; do true; done); echo after");
    assert!(late < Duration::from_millis(100), "{late:?}");
    assert_eq!(status, 130);
    assert_eq!(run(&mut shell, "echo ok").0, "ok");
}
