//! The parity tool: `emusen-crystal-debug FILE [NAME=VALUE...]` runs a script of the debugger's calls and prints each
//! answer, one line a call, for `CrystalDebugParityTests` to compare with the C# debugger's answers to the same script,
//! line for line (EmuSen_Backend.md §29.30). A `$NAME` in the script is the value given for it.

mod fake;
mod regs;

use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(file) = args.first() else {
        eprintln!("usage: emusen-crystal-debug FILE [NAME=VALUE...]");
        std::process::exit(2);
    };
    let mut text = match std::fs::read_to_string(file) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{file}: {e}");
            std::process::exit(2);
        }
    };
    for given in &args[1..] {
        if let Some((name, value)) = given.split_once('=') {
            text = text.replace(&format!("${name}"), value);
        }
    }
    let mut out = String::new();
    let mut script = fake::Script::new();
    for line in text.lines() {
        let line = line.trim_end_matches('\r');
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let (verb, rest) = line.split_once(' ').unwrap_or((line, ""));
        let answer = script.run(verb, rest);
        out.push_str(&format!("{line} => {answer}\n"));
    }
    std::io::stdout().write_all(out.as_bytes()).unwrap();
}
