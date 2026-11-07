//! The golden corpus (`Platform/dianash/tests/golden/`, EmuSen_DianaSh.md §13.4) run through this crate alone:
//! every case recorded with no target whose lines need nothing but what the Rust shell has. A case is passed over
//! when the shell answers one of its lines "Unknown command" where the recording does not, since it then needs a
//! command a host provides, which nothing runs since the C# core host was retired (EmuSen_Backend.md §40.10.3). The
//! rest must match byte for byte, with their statuses, their continuations and their requests as the C# host actions
//! print them.

mod corpus;

use emusen_dianash::shell::{Config, Shell};

/// A shell with the modules a program links, as the platform library and `dianash` build it.
fn shell_with_modules(config: Config) -> Shell {
    let mut shell = Shell::new(config);
    shell.add_module(&dianash_text::MODULE).unwrap();
    shell
}

#[test]
fn every_case_the_rust_shell_can_run_alone_prints_its_recording() {
    let t = corpus::run_corpus(None, shell_with_modules);
    eprintln!("golden cases run by the Rust shell alone: {} passed, {} need a host's command, {} ask the file system's case, {} failed", t.passed, t.needs_host, t.case_of_the_system, t.failures.len());
    assert!(t.failures.is_empty(), "{}", t.failures.join("\n\n"));
    assert!(t.passed > 0);
}
