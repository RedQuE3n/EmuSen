//! `dianash`, the standalone shell: a host of its own library with no service and no emulator in it, a line
//! editor and the shell's builtins in one file. `dianash -c LINE` runs one line and exits with its status.
//! See EmuSen_DianaSh.md §8.2 and §10.4.

mod editor;
#[allow(dead_code, unused_imports)]
mod terminal;

use emusen_dianash::shell::{self, Config, Shell, Submitted};
use emusen_galaxia::tree;
use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // The shell's `/` is Galaxia's home; the program starts there, as the C# shell does.
    let home = tree::tree().home();
    let _ = std::fs::create_dir_all(&home);
    let _ = std::env::set_current_dir(&home);

    let interactive = args.is_empty() && terminal::is_terminal();
    let mut shell = Shell::new(Config { interactive, ..Config::default() });
    shell.add_module(&dianash_text::MODULE).expect("the text tools' manifest");
    shell.add_module(&emusen_dev_module::MODULE).expect("the development tools' manifest");
    let session = shell.session_new().expect("a shell with sessions");
    shell.session_attach(session, Some("main")).expect("the first session");
    terminal::on_interrupt_cancel(shell.cancel_flag());

    if let [flag, lines @ ..] = args.as_slice() {
        if flag == "-c" {
            return run_lines(&mut shell, lines);
        }
        eprintln!("usage: dianash [-c LINE...]");
        return ExitCode::from(2);
    }

    if interactive {
        println!("{}", shell::banner());
        println!("--- DianaSh standalone shell - type 'help' or 'man <command>', 'shutdown' to quit ---");
    }
    loop {
        let prompt = shell.prompt(0).unwrap_or_else(|_| "DianaSh #: ".to_string());
        if interactive {
            print!("{prompt}");
            let _ = std::io::stdout().flush();
        }
        let history = shell.history(0).unwrap_or_default();
        let Some(line) = editor::read_line(prompt.chars().count(), &history) else { return ExitCode::SUCCESS };
        let submitted = shell.submit(0, &line);
        let output = shell.take_output();
        if !output.is_empty() {
            println!("{output}");
        }
        shell.drain_requests();
        if submitted == Ok(Submitted::Exit) {
            return ExitCode::SUCCESS;
        }
    }
}

/// Each argument a line, as typed at the prompt; the exit status is the last line's `$?`.
fn run_lines(shell: &mut Shell, lines: &[String]) -> ExitCode {
    for line in lines {
        let submitted = shell.submit(0, line);
        let output = shell.take_output();
        if !output.is_empty() {
            println!("{output}");
        }
        if submitted == Ok(Submitted::Exit) {
            break;
        }
    }
    let status = shell.last_status(0).unwrap_or(1);
    ExitCode::from(u8::try_from(status).unwrap_or(1))
}
